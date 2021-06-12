use hyper::body::Buf;
use hyper::{client::HttpConnector, Body, Client, Method, Request, Uri};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::error::EthereumServiceError;

/// Hyper client responsible for quering of `etherscan.io` REST API
pub struct EtherscanClient {
    client: Client<HttpConnector>,
    api_key: String,
}

impl EtherscanClient {
    pub fn new(client: Client<HttpConnector>, api_key: impl AsRef<str> + Display) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
        }
    }

    /// Returns the number of most recent block
    pub async fn latest_block_number(
        &self,
    ) -> Result<EthBlockNumberResponse, EthereumServiceError> {
        let request = EtherscanRequestBuilder::default()
            .api_key(&self.api_key)
            .build("proxy", "eth_blockNumber")?;

        let response = self.client.request(request).await?;

        let body = hyper::body::aggregate(response.into_body()).await?;

        Ok(serde_json::from_reader(body.reader())?)
    }

    /// Returns information about block and uncle rewards
    pub async fn block_reward(
        &self,
        block_number: u64,
    ) -> Result<EthBlockRewardResponse, EthereumServiceError> {
        let request = EtherscanRequestBuilder::default()
            .api_key(&self.api_key)
            .query("blockno", block_number)
            .build("block", "getblockreward")?;

        let response = self.client.request(request).await?;

        let body = hyper::body::aggregate(response.into_body()).await?;

        let body: EthResponse<EthBlockRewardResponse> = serde_json::from_reader(body.reader())?;

        Ok(body
            .try_into_result()
            .ok_or(EthereumServiceError::InvalidResponseStatus)?)
    }
}

/// Simple ResuestBuilder similar to reqwest one but with predefined
/// Request and Uri structures.
/// It can be replaced by derive_builder crate
#[derive(Debug, Default)]
struct EtherscanRequestBuilder {
    path_and_query: String,
}

impl EtherscanRequestBuilder {
    fn api_key(self, api_key: impl Display) -> Self {
        Self {
            path_and_query: format!("{}&apikey={}", self.path_and_query, api_key),
            ..self
        }
    }

    fn query(self, key: impl Display, value: impl Display) -> Self {
        Self {
            path_and_query: format!("{}&{}={}", self.path_and_query, key, value),
            ..self
        }
    }

    fn build(
        self,
        module: impl Display,
        action: impl Display,
    ) -> Result<Request<Body>, hyper::http::Error> {
        let uri = Uri::builder()
            .scheme("http")
            .authority("api.etherscan.io")
            .path_and_query(format!(
                "/api?module={}&action={}&{}",
                module, action, self.path_and_query,
            ))
            .build()?;

        Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::empty())
    }
}

/// Helper type to extract nested data from etherscan API
///
/// Strcture of most (if all) of the message bodies is constructed
/// in a form of { status: "0", message: "OK", result: { } }.
/// For simplicity the validation of the status is postponed
#[derive(Debug, Serialize, Deserialize)]
struct EthResponse<T> {
    status: String,
    message: String,
    result: T,
}

impl<T> EthResponse<T> {
    /// Converts entire body into result part droping the rest
    /// Fails if response status is invalid
    pub fn try_into_result(self) -> Option<T> {
        match self.status.as_str() {
            "1" => Some(self.result),
            _ => None,
        }
    }
}

/// Response from proxy/eth_blockNumber action
#[derive(Debug, Serialize, Deserialize)]
pub struct EthBlockNumberResponse {
    #[serde(rename = "result")]
    pub block_number: String,
}

/// Response from block/getblockreward action
///
/// Additional fields that are not necessary have been removed
#[derive(Debug, Serialize, Deserialize)]
pub struct EthBlockRewardResponse {
    #[serde(rename = "timeStamp")]
    pub timestamp: String,
}
