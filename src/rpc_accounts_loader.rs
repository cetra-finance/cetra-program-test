use super::AccountsLoader;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::error::Error;

pub struct RpcAccountsLoader {
    pub client: RpcClient,
}

impl RpcAccountsLoader {
    pub fn new_with_client(client: RpcClient) -> Self {
        RpcAccountsLoader { client }
    }

    pub fn new_with_url(url: impl AsRef<str>) -> Self {
        RpcAccountsLoader {
            client: RpcClient::new(url.as_ref()),
        }
    }
}

impl Default for RpcAccountsLoader {
    fn default() -> Self {
        RpcAccountsLoader {
            client: RpcClient::new(String::from("https://solana-api.projectserum.com")),
        }
    }
}

#[async_trait]
impl AccountsLoader for RpcAccountsLoader {
    async fn get_accounts(
        &self,
        pubkeys: &Vec<Pubkey>,
    ) -> Result<Vec<Option<Account>>, Box<dyn Error>> {
        let mut accounts_result = Vec::new();

        for pubkeys_chunk in pubkeys.chunks(100) {
            let maybe_accounts = self
                .client
                .get_multiple_accounts_with_commitment(pubkeys_chunk, self.client.commitment())?;

            accounts_result.extend(maybe_accounts.value);
        }

        Ok(accounts_result)
    }
}
