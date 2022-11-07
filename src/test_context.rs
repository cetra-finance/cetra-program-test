use super::{AccountsLoader, FixtureAccountWrapper};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    account::Account, bpf_loader_upgradeable, clock::Clock, commitment_config::CommitmentLevel,
    program_option::COption, program_pack::Pack, pubkey::Pubkey, rent::Rent,
    transaction::Transaction, transport,
};
use std::{
    error::Error,
    fs::File,
    io::{self, Write},
    path::Path,
    time::Duration,
};

pub struct TestContext {
    pub context: ProgramTestContext,
    accounts_loader_impl: Box<dyn AccountsLoader>,
    fixtures_path: String,
}

impl TestContext {
    pub fn new(
        context: ProgramTestContext,
        fixtures_path: String,
        accounts_loader_impl: Box<dyn AccountsLoader>,
    ) -> Self {
        TestContext {
            context,
            accounts_loader_impl,
            fixtures_path,
        }
    }

    async fn cache_accounts(
        &mut self,
        account_pubkeys: &Vec<Pubkey>,
    ) -> Result<(), Box<dyn Error>> {
        let mut undefined_accounts = Vec::new();

        for account_pubkey in account_pubkeys {
            let maybe_account = self
                .context
                .banks_client
                .get_account(*account_pubkey)
                .await?;

            if maybe_account.is_none() {
                undefined_accounts.push(*account_pubkey);
            }
        }

        let maybe_loaded_accounts = self
            .accounts_loader_impl
            .get_accounts(&undefined_accounts)
            .await?;

        let account_path = Path::new(&self.fixtures_path);
        for (maybe_loaded_account, pubkey) in maybe_loaded_accounts.iter().zip(&undefined_accounts)
        {
            if let Some(account) = maybe_loaded_account {
                if account.executable {
                    let (program_pubkey, _) = Pubkey::find_program_address(
                        &[pubkey.as_ref()],
                        &bpf_loader_upgradeable::id(),
                    );

                    let mut program_file =
                        File::create(account_path.join(format!("{}.bin", program_pubkey)))?;

                    let program_account = self
                        .accounts_loader_impl
                        .get_accounts(&vec![program_pubkey])
                        .await?[0]
                        .clone()
                        .expect("Unexpected invalid program buffer!");

                    program_file.write_all(&program_account.data)?;

                    self.context
                        .set_account(&program_pubkey, &program_account.into());
                }

                let mut account_file = File::create(account_path.join(format!("{}.json", pubkey)))?;
                account_file.write_all(
                    serde_json::to_string(&FixtureAccountWrapper {
                        pubkey: pubkey.to_string(),
                        account: account.clone().into(),
                    })?
                    .as_bytes(),
                )?;

                self.context.set_account(pubkey, &account.clone().into());
            }
        }

        Ok(())
    }

    /// Creates associated token account for `wallet` with provided `amount`,
    /// and `token_mint`.
    pub async fn create_ata(
        &mut self,
        wallet: &Pubkey,
        token_mint: &Pubkey,
        amount: u64,
    ) -> transport::Result<()> {
        let wallet_ata =
            spl_associated_token_account::get_associated_token_address(wallet, token_mint);

        let rent = self.get_rent().await;
        let rent_exempt_lamports = rent.minimum_balance(spl_token::state::Account::LEN);

        let (is_native, amount) = if token_mint == &spl_token::native_mint::id() {
            (
                COption::Some(rent_exempt_lamports),
                amount + rent_exempt_lamports,
            )
        } else {
            (COption::None, amount)
        };

        let account_data = spl_token::state::Account {
            mint: *token_mint,
            owner: *wallet,
            amount,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native,
            delegated_amount: 0,
            close_authority: COption::None,
        };
        let mut account_buffer = Vec::new();
        account_buffer.resize(spl_token::state::Account::LEN, 0);
        account_data.pack_into_slice(&mut account_buffer);

        self.context.set_account(
            &wallet_ata,
            &Account {
                lamports: rent_exempt_lamports,
                data: account_buffer,
                owner: spl_token::ID,
                executable: false,
                rent_epoch: u64::MAX,
            }
            .into(),
        );

        Ok(())
    }

    /// Adds `amount` of `token_mint` tokens for `wallet` associated token account.
    pub async fn add_ata_tokens(
        &mut self,
        wallet: &Pubkey,
        token_mint: &Pubkey,
        amount: u64,
    ) -> transport::Result<()> {
        let wallet_ata =
            spl_associated_token_account::get_associated_token_address(wallet, token_mint);

        let maybe_account = self.context.banks_client.get_account(wallet_ata).await?;
        if let Some(mut account) = maybe_account {
            let mut ata = spl_token::state::Account::unpack(&account.data)
                .expect("Unable to unpack spl token account!");
            ata.amount += amount;

            let mut ata_buffer = Vec::new();
            ata_buffer.resize(spl_token::state::Account::LEN, 0);
            ata.pack_into_slice(&mut ata_buffer);

            account.data = ata_buffer;

            self.context.set_account(&wallet_ata, &account.into())
        } else {
            return Err(transport::TransportError::Custom(
                "Associated token account not found!".to_string(),
            ));
        }

        Ok(())
    }

    /// Returns [balance](`spl_token::state::Account::amount`) of `wallet` associated token account for `token_mint`.
    pub async fn get_ata_balance(
        &mut self,
        wallet: &Pubkey,
        token_mint: &Pubkey,
    ) -> transport::Result<u64> {
        let wallet_ata =
            spl_associated_token_account::get_associated_token_address(wallet, token_mint);

        let maybe_account = self.context.banks_client.get_account(wallet_ata).await?;
        let amount = if let Some(account) = maybe_account {
            let ata = spl_token::state::Account::unpack(&account.data)
                .expect("Unable to unpack spl token account!");
            ata.amount
        } else {
            return Err(transport::TransportError::Custom(
                "Associated token account not found!".to_string(),
            ));
        };

        Ok(amount)
    }

    pub async fn get_account(&mut self, address: &Pubkey) -> io::Result<Option<Account>> {
        self.cache_accounts(&vec![*address])
            .await
            .expect("Failed to cache accounts");

        self.context
            .banks_client
            .get_account_with_commitment(*address, CommitmentLevel::default())
            .await
    }

    pub async fn process_transaction(&mut self, transaction: Transaction) -> transport::Result<()> {
        self.cache_accounts(&transaction.message.account_keys)
            .await
            .expect("Failed to cache accounts");

        self.context
            .banks_client
            .process_transaction(transaction)
            .await
    }

    pub async fn get_clock(&mut self) -> Clock {
        self.context
            .banks_client
            .get_sysvar::<Clock>()
            .await
            .expect("Unable to load clock sysvar")
    }

    pub async fn get_rent(&mut self) -> Rent {
        self.context
            .banks_client
            .get_rent()
            .await
            .expect("Unable to load rent sysvar")
    }

    pub async fn wait(&mut self, duration: Duration) {
        let start_timestamp = self.get_clock().await.unix_timestamp;

        loop {
            let clock = self.get_clock().await;

            if clock.unix_timestamp >= start_timestamp + duration.as_secs() as i64 {
                break;
            }

            self.context
                .warp_to_slot(clock.slot + 100)
                .expect("Unable to warp");
        }
    }
}
