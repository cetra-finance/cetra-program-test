use async_trait::async_trait;
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::error::Error;

#[async_trait]
pub trait AccountsLoader {
    #[allow(clippy::ptr_arg)]
    async fn get_accounts(
        &self,
        pubkeys: &Vec<Pubkey>,
    ) -> Result<Vec<Option<Account>>, Box<dyn Error>>;
}
