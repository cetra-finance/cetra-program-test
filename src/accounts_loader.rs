use super::AccountType;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait AccountsLoader {
    #[allow(clippy::ptr_arg)]
    async fn get_accounts(
        &self,
        pubkeys: &Vec<[u8; 32]>,
    ) -> Result<Vec<Option<AccountType>>, Box<dyn Error>>;
}
