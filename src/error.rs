use thiserror::Error;

#[derive(Error, Debug)]
pub enum EthereumServiceError {
    #[error("Http handle error: {0}")]
    HyperError(#[from] hyper::Error),
    #[error("Http request build error: {0}")]
    HyperRequestBuildError(#[from] hyper::http::Error),
    #[error("Block number parse error: {0}")]
    BlockNumberParseError(#[from] std::num::ParseIntError),
    #[error("Unable to parse http body")]
    HttpBodyParseError(#[from] serde_json::Error),
    #[error("Server response with invalid status")]
    InvalidResponseStatus,
}
