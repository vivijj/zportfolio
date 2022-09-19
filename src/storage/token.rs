use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

use super::{StorageError, StorageProcessor};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TokenInfo {
    id: String,
    chain: String,
    name: String,
    symbol: String,
    decimals: i32,
    logo_url: String,
    protocol_id: String,
    is_core: bool,
}

impl StorageProcessor {
    /// Loads all the stored tokens from the database.
    pub async fn load_tokens(&self) -> Result<Vec<TokenInfo>, StorageError> {
        let tokens = sqlx::query_as::<_, TokenInfo>(
            r#"
            SELECT id, chain, name, symbol,decimals,logo_url,protocol_id,is_core FROM token
            "#,
        )
        .fetch_all(&self.conn)
        .await?;
        Ok(tokens)
    }

    /// get token ids on different chain by its name, only vote token have this method.
    pub async fn load_token_ids_by_name(
        &self,
        id: String,
    ) -> Result<HashMap<String, String>, StorageError> {
        let res = sqlx::query(
            r#"
            SELECT * FROM vote_token
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.conn)
        .await?;

        let chain_ids = self.load_support_chain_ids().await?;
        let mut token_ids = HashMap::<String, String>::new();

        for chain_id in chain_ids {
            let ids: Option<String> = res.try_get(chain_id.as_str()).unwrap_or(None);
            if let Some(i) = ids {
                token_ids.insert(chain_id, i);
            }
        }
        Ok(token_ids)
    }
}
