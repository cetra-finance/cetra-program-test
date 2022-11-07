use serde::{Deserialize, Serialize};
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureAccount {
    pub lamports: u64,
    pub data: Vec<String>,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: u64,
}

impl From<FixtureAccount> for Account {
    fn from(fixture_account: FixtureAccount) -> Self {
        Account {
            lamports: fixture_account.lamports,
            data: base64::decode(&fixture_account.data[0]).expect("Unable to decode base64 data"),
            owner: Pubkey::from_str(&fixture_account.owner)
                .expect("Unable to create pubkey from string"),
            executable: fixture_account.executable,
            rent_epoch: fixture_account.rent_epoch,
        }
    }
}

impl From<Account> for FixtureAccount {
    fn from(account: Account) -> Self {
        FixtureAccount {
            lamports: account.lamports,
            data: vec![base64::encode(account.data), String::from("base64")],
            owner: account.owner.to_string(),
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureAccountWrapper {
    pub pubkey: String,
    pub account: FixtureAccount,
}
