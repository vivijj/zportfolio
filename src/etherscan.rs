use reqwest::{self, header, IntoUrl, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EtherscanApiError {
    #[error("Bad status code: {0}")]
    BadStatusCode(String),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalTransaction {
    pub time_stamp: String,
}

#[derive(Debug, Clone)]
pub struct EtherscanAPi {
    /// Client that executes HTTP requests
    client: reqwest::Client,
    /// Etherscan API key
    api_key: String,
    api_url: Url,
}



impl EtherscanAPi {
    pub fn new(api_key: &str, api_url: impl IntoUrl) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            api_url: api_url.into_url().unwrap(),
        }
    }
    /// account_age will return the timestamp when the account send its first tx
    pub async fn account_age(&self, id: &str) -> Result<i64, EtherscanApiError> {
        let res = self
            .client
            .get(self.api_url.clone())
            .header(header::ACCEPT, "application/json")
            .query(&[
                ("module", "account"),
                ("action", "txlist"),
                ("address", id),
                ("startblock", "0"),
                ("endblock", "99999999"),
                ("page", "1"),
                ("offset", "1"),
                ("sort", "asc"),
                ("api_key", self.api_key.as_str()),
            ])
            .send()
            .await?
            .json::<ResponseData<Vec<NormalTransaction>>>()
            .await?;
        match res {
            ResponseData::Error { result, .. } => {
                if result.starts_with("Max rate limit reached") {
                    Err(EtherscanApiError::RateLimitExceeded)
                } else {
                    Err(EtherscanApiError::Unknown(result))
                }
            }
            ResponseData::Success(res) => match res.status.as_str() {
                "0" => Ok(i64::MAX),
                "1" => Ok(res.result[0].time_stamp.parse::<i64>().unwrap()),
                err => Err(EtherscanApiError::BadStatusCode(err.to_string())),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response<T> {
    pub status: String,
    pub message: String,
    pub result: T,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseData<T> {
    Success(Response<T>),
    Error {
        status: String,
        message: String,
        result: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_account_age() {
        let apic = EtherscanAPi::new(
            "9B2BTDM982MDSPHABFNVMIGMD1RJI4U6IJ",
            "https://api.etherscan.io/api",
        );
        let k = apic
            .account_age("0xddbd2b932c763ba5b1b7ae3b362eac3e8d40121a")
            .await
            .unwrap();
        println!("k is: {}", k)
    }
}
