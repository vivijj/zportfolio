use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::{StorageError, StorageProcessor};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserInfo {
    id: String,
    pub activation_time: i64,
}

impl StorageProcessor {
    pub async fn load_user_info(&self, user_id: &str) -> Result<UserInfo, StorageError> {
        let id = user_id.to_lowercase(); // use address in normalized format
        let user_info = sqlx::query_as::<_, UserInfo>(
            r#"
            SELECT id, activation_time FROM user
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.conn)
        .await?;
        Ok(user_info)
    }

    pub async fn set_user_info(
        &self,
        user_id: &str,
        activation_time: i64,
    ) -> Result<(), StorageError> {
        let id = user_id.to_lowercase();
        sqlx::query(
            r#"
            INSERT INTO user (id, activation_time)
            VALUES ( ?, ? )
            "#,
        )
        .bind(id)
        .bind(activation_time)
        .execute(&self.conn)
        .await?;
        Ok(())
    }
}
