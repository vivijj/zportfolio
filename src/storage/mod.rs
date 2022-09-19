// Built-in deps
use std::env;

use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

pub mod chain;

pub mod token;
pub mod user;

use sqlx;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub fn get_database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

#[derive(Clone)]
pub struct StorageProcessor {
    conn: MySqlPool,
}

impl StorageProcessor {
    pub async fn new_from_pool() -> Self {
        let conn_pool = MySqlPoolOptions::new()
            .connect(&get_database_url())
            .await
            .unwrap();
        Self { conn: conn_pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    #[test]
    fn test_1() {
        dotenv().ok();
        let res = get_database_url();
        println!("{:?}", res)
    }

    #[tokio::test]
    async fn test_load_tokens() {
        dotenv().ok();
        let sp = StorageProcessor::new_from_pool().await;
        println!("finish init the pool");
        let res = sp.load_tokens().await.unwrap();
        print!("{:?}", res);
    }

    #[tokio::test]
    async fn test_load_chain_ids() {
        dotenv().ok();
        let sp = StorageProcessor::new_from_pool().await;
        let res = sp.load_support_chain_ids().await.unwrap();
        print!("{:?}", res);
    }

    #[tokio::test]
    async fn test_load_chains() {
        dotenv().ok();
        let sp = StorageProcessor::new_from_pool().await;
        let res = sp.load_chains().await.unwrap();
        print!("{:?}", res);
    }

    #[tokio::test]
    async fn test_load_token_ids_by_name() {
        dotenv().ok();
        let sp = StorageProcessor::new_from_pool().await;
        let res = sp.load_token_ids_by_name("ETH".to_string()).await.unwrap();
        println!("{:?}", res);
    }
}
