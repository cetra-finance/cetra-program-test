use super::{AccountsLoader, FixtureAccountWrapper, TestContext};
use solana_program_test::ProgramTest;
use solana_sdk::{account::Account, bpf_loader, pubkey::Pubkey, rent::Rent};
use std::{
    error::Error,
    fs::{create_dir_all, read_dir, File},
    io::Read,
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
            let mut file_path = dir_entry.path();
            file_path.set_extension("");
            let file_name = file_path
                .file_name()
                .expect("Unable obtain file name")
                .to_str()
                .expect("Unable convert file name to string")
                .to_string();

            let mut account_file = File::open(dir_entry.path())?;
            let mut buffer = Vec::new();
            account_file.read_to_end(&mut buffer)?;

            let fixture_account: FixtureAccountWrapper = match serde_json::from_slice(&buffer) {
                Ok(account) => account,
                Err(_) => {
                    let pubkey = Pubkey::from_str(&file_name)?;

                    self.program_test.add_account(
                        pubkey,
                        Account {
                            lamports: Rent::default().minimum_balance(buffer.len()).min(1),
                            data: buffer,
                            owner: bpf_loader::id(),
                            executable: true,
                            rent_epoch: 0,
                        },
                    );

                    continue;
                }
            };

            self.program_test.add_account(
                Pubkey::from_str(&fixture_account.pubkey)?,
                fixture_account.account.into(),
            );
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
