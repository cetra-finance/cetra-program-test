use super::{AccountsLoader, FixtureAccountWrapper, TestContext};
use solana_program_test::ProgramTest;
use solana_sdk::{account::Account, bpf_loader_upgradeable, pubkey::Pubkey, rent::Rent};
use std::{
    error::Error,
    fs::{create_dir_all, read_dir, File},
    io::Read,
    path::Path,
    str::FromStr,
};

pub struct ProgramTestLoader {
    pub program_test: ProgramTest,
    fixtures_path: String,
}

impl ProgramTestLoader {
    pub fn new_with_fixtures_path(fixtures_path: &str) -> Self {
        ProgramTestLoader {
            program_test: ProgramTest::default(),
            fixtures_path: fixtures_path.to_string(),
        }
    }

    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let active_dir = if let Ok(dir) = read_dir(&self.fixtures_path) {
            dir
        } else {
            create_dir_all(&self.fixtures_path)?;
            return Ok(());
        };

        for dir_entry in active_dir.flatten() {
            let mut account_file = File::open(dir_entry.path())?;
            let mut buffer = Vec::new();
            account_file.read_to_end(&mut buffer)?;

            match serde_json::from_slice::<FixtureAccountWrapper>(&buffer) {
                Ok(account_wrapper) => {
                    let account_pubkey = Pubkey::from_str(&account_wrapper.pubkey)?;
                    let account: Account = account_wrapper.account.into();

                    self.program_test
                        .add_account(account_pubkey, account.clone());

                    if account.executable {
                        let (program_pubkey, _) = Pubkey::find_program_address(
                            &[account_pubkey.as_ref()],
                            &bpf_loader_upgradeable::id(),
                        );

                        let program_path = Path::new(&self.fixtures_path);
                        let mut program_file =
                            File::open(program_path.join(format!("{}.bin", program_pubkey)))?;
                        let mut program_buffer = Vec::new();
                        program_file.read_to_end(&mut program_buffer)?;

                        self.program_test.add_account(
                            program_pubkey,
                            Account {
                                lamports: Rent::default().minimum_balance(buffer.len()).min(1),
                                data: program_buffer,
                                owner: bpf_loader_upgradeable::id(),
                                executable: false,
                                rent_epoch: u64::MAX,
                            },
                        );
                    }
                }
                Err(_) => continue,
            };
        }

        Ok(())
    }

    pub async fn start_with_context(
        self,
        account_loader_impl: Box<dyn AccountsLoader>,
    ) -> TestContext {
        TestContext::new(
            self.program_test.start_with_context().await,
            self.fixtures_path,
            account_loader_impl,
        )
    }
}

impl Default for ProgramTestLoader {
    fn default() -> Self {
        ProgramTestLoader {
            program_test: ProgramTest::default(),
            fixtures_path: String::from("tests/fixtures"),
        }
    }
}
