use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

use super::{StorageError, StorageProcessor};

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct ChainInfo {
    id: String,
    community_id: i32,
    name: String,
    native_token_id: String,
    logo_url: String,
}

impl StorageProcessor {
    pub async fn load_support_chain_ids(&self) -> Result<Vec<String>, StorageError> {
        let chain_ids_row = sqlx::query(
            r#"
            SELECT id FROM chain
            "#,
        )
        .fetch_all(&self.conn)
        .await?;
        let mut res = Vec::new();
        for id_row in chain_ids_row {
            res.push(id_row.get("id"));
        }
        Ok(res)
    }

    pub async fn load_chains(&self) -> Result<Vec<ChainInfo>, StorageError> {
        let chain_infos = sqlx::query_as::<_, ChainInfo>(
            r#"
            SELECT id, community_id, name, native_token_id,logo_url FROM chain
            "#,
        )
        .fetch_all(&self.conn)
        .await?;
        Ok(chain_infos)
    }
}
