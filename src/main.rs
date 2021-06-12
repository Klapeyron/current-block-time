mod clients;
mod error;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use crate::clients::{EthBlockNumberResponse, EtherscanClient};
use crate::error::EthereumServiceError;

async fn current_block_time(req: Request<Body>) -> Result<Response<Body>, EthereumServiceError> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/currentBlockTime") => {
            let retry_strategy = ExponentialBackoff::from_millis(10).map(jitter).take(5);

            // Https support postponed, only tokio-native-tls added to Cargo.toml
            let client = Client::new();
            // API key hardcoded here for simplicity
            let api_key = "W934GMVKV4J82K9PUDWG2K2TY9FVH7YIC2";

            let etherscan_client = EtherscanClient::new(client, api_key);

            let EthBlockNumberResponse { block_number } =
                Retry::spawn(retry_strategy.clone(), || {
                    etherscan_client.latest_block_number()
                })
                .await?;

            let without_prefix = block_number.trim_start_matches("0x");
            let block_number = u64::from_str_radix(without_prefix, 16)?;

            let result = Retry::spawn(retry_strategy, || {
                etherscan_client.block_reward(block_number)
            })
            .await?;

            Ok(Response::new(serde_json::to_string(&result)?.into()))
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = (std::net::Ipv4Addr::UNSPECIFIED, 2137).into();

    let service = make_service_fn(|_| async {
        Ok::<_, EthereumServiceError>(service_fn(current_block_time))
    });

    let server = Server::bind(&addr).serve(service);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
