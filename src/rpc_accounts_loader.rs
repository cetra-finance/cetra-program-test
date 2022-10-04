use super::{AccountType, AccountsLoader};
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::error::Error;

pub struct RpcAccountsLoader {
    pub client: RpcClient,
}

impl RpcAccountsLoader {
    pub fn new_with_client(client: RpcClient) -> Self {
        RpcAccountsLoader { client }
    }
}

impl Default for RpcAccountsLoader {
    fn default() -> Self {
        RpcAccountsLoader {
            client: RpcClient::new(String::from("https://api.mainnet-beta.solana.com")),
        }
    }
}

#[async_trait]
impl AccountsLoader for RpcAccountsLoader {
    async fn get_accounts(
        &self,
        pubkeys: &Vec<[u8; 32]>,
    ) -> Result<Vec<Option<AccountType>>, Box<dyn Error>> {
        let mut accounts_result = Vec::new();

        for pubkeys_chunk in pubkeys.chunks(100) {
            let maybe_accounts = self.client.get_multiple_accounts_with_commitment(
                &pubkeys_chunk
                    .iter()
                    .map(|pubkey| Pubkey::new_from_array(*pubkey))
                    .collect::<Vec<Pubkey>>(),
                self.client.commitment(),
            )?;

            let result: Vec<Option<AccountType>> = maybe_accounts
                .value
                .iter()
                .map(|maybe_account| {
                    maybe_account.as_ref().map(|account| {
                        (
                            account.lamports,
                            account.data.clone(),
                            account.owner.to_bytes(),
                            account.executable,
                            account.rent_epoch,
                        )
                    })
                })
                .collect();

            accounts_result.extend(result);
        }

        Ok(accounts_result)
    }
}
