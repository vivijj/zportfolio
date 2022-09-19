use reqwest::{self, IntoUrl, Response, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DebankApiError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Unauthorized access key")]
    Unauthorized,
    #[error("Exceeded debank api ratelimit")]
    RateLimitExceeded,
    #[error("Hit debank api account capacity limit")]
    CapacityLimitExceeded,
    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainBalance {
    id: String,
    community_id: i64,
    name: String,
    logo_url: String,
    native_token_id: String,
    usd_value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebankChainBalance {
    usd_value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebankTotalBalance {
    total_usd_value: f64,
    chain_list: Vec<ChainBalance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebankTokenBalance {
    id: String,
    chain: String,
    name: String,
    symbol: String,
    decimals: i64,
    logo_url: String,
    protocol_id: String,
    is_core: bool,
    price: f64,
    pub amount: f64,
    pub raw_amount: f64,
    raw_amount_hex_str: String,
}

#[derive(Debug, Clone)]
pub struct DebankOpenAPI {
    api_url: Url,
    client: reqwest::Client,
    access_key: String,
}

impl DebankOpenAPI {
    pub fn new(api_url: impl IntoUrl, access_key: &str) -> Self {
        Self {
            api_url: api_url.into_url().unwrap(),
            client: reqwest::Client::new(),
            access_key: access_key.to_string(),
        }
    }

    fn handle_debank_response(&self, resp: Response) -> Result<Response, DebankApiError> {
        if resp.status().as_u16() != 200 {
            return match resp.status().as_u16() {
                401 => Err(DebankApiError::Unauthorized),
                403 => Err(DebankApiError::CapacityLimitExceeded),
                429 => Err(DebankApiError::RateLimitExceeded),
                _ => Err(DebankApiError::Unknown),
            };
        } else {
            Ok(resp)
        }
    }

    // get the balance of specific token
    pub async fn token_balance(
        &self,
        id: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<DebankTokenBalance, DebankApiError> {
        let url = self
            .api_url
            .join("/v1/user/token")
            .expect("failed to join url path");
        let mut resp = self
            .client
            .get(url)
            .header("AccessKey", &self.access_key)
            .query(&[("id", id), ("chain_id", chain_id), ("token_id", token_id)])
            .send()
            .await?;
        resp = self.handle_debank_response(resp)?;
        let res = resp.json::<DebankTokenBalance>().await?;
        Ok(res)
    }

    /// get the total balance on provide chains.
    pub async fn muti_chain_balance(
        &self,
        id: &str,
        chain_ids: &Vec<String>,
    ) -> Result<DebankTotalBalance, DebankApiError> {
        let url = self
            .api_url
            .join("/v1/user/total_balance")
            .expect("failed to join url path");

        let mut resp = self
            .client
            .get(url)
            .header("AccessKey", &self.access_key)
            .query(&[("id", id)])
            .send()
            .await?;
        resp = self.handle_debank_response(resp)?;
        let mut res = resp.json::<DebankTotalBalance>().await?;
        res.chain_list.retain(|item| {
            if chain_ids.contains(&item.id) {
                true
            } else {
                false
            }
        });
        res.total_usd_value = 0.0;
        for chain in res.chain_list.iter() {
            res.total_usd_value += chain.usd_value;
        }
        Ok(res)
    }

    pub async fn chain_balance(
        &self,
        id: &str,
        chain_id: &str,
    ) -> Result<DebankChainBalance, DebankApiError> {
        let url = self
            .api_url
            .join("/v1/user/chain_balance")
            .expect("failed to join url path");
        let mut resp = self
            .client
            .get(url)
            .header("AccessKey", &self.access_key)
            .query(&[("id", id), ("chain_id", chain_id)])
            .send()
            .await?;
        resp = self.handle_debank_response(resp)?;
        let res = resp.json::<DebankChainBalance>().await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use std::env;

    #[tokio::test]
    async fn test_total_balance() {
        dotenv().ok();
        let access_key = env::var("DEBANK_KEY").expect("Debank access key not exist");
        let dapi = DebankOpenAPI::new("https://pro-openapi.debank.com/v1", access_key.as_str());

        let ids = vec![
            "eth".to_string(),
            "bsc".to_string(),
            "heco".to_string(),
            "ftm".to_string(),
            "matic".to_string(),
            "avax".to_string(),
        ];
        let res = dapi
            .muti_chain_balance("0xa749cdefd2d9590549df709bbffec04a9bd35b42", &ids)
            .await
            .unwrap();
        println!("res is: {:?}", res)
    }
}
