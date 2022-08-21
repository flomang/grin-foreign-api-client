use easy_jsonrpc_mw::{BoundMethod, Response};
// grin api foreign_rpc helper module
use grin_api::foreign_rpc::foreign_rpc;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::fmt;

// Demonstrate an example JSON-RCP call against grin foreign api.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr: SocketAddr = "127.0.0.1:3413" 
        .parse()
        .unwrap();

    let node_version = rpc(&server_addr, &foreign_rpc::get_version().unwrap())
        .await??;

    println!("hello grin!: {:?}", node_version);

    Ok(())
}

async fn rpc<R: Deserialize<'static>>(
    addr: &SocketAddr,
    method: &BoundMethod<'_, R>,
) -> Result<R, RpcErr> {
    let (request, tracker) = method.call();
    let json_response = post(addr, &request.as_request()).await?;
    let mut response = Response::from_json_response(json_response)?;
    Ok(tracker.get_return(&mut response)?)
}

async fn post(addr: &SocketAddr, body: &Value) -> Result<Value, reqwest::Error> {
    let client = Client::new();
    let response = client
        .post(&format!("http://{}/v2/foreign", addr))
        .json(body)
        .send()
        .await?;

    let json_response = response.error_for_status()?.json::<Value>().await?;

    Ok(json_response)
}


#[derive(Debug)]
enum RpcErr {
    Http(reqwest::Error),
    InvalidResponse,
}

impl From<easy_jsonrpc_mw::InvalidResponse> for RpcErr {
    fn from(_other: easy_jsonrpc_mw::InvalidResponse) -> Self {
        RpcErr::InvalidResponse
    }
}

impl From<easy_jsonrpc_mw::ResponseFail> for RpcErr {
    fn from(_other: easy_jsonrpc_mw::ResponseFail) -> Self {
        RpcErr::InvalidResponse
    }
}

impl From<reqwest::Error> for RpcErr {
    fn from(other: reqwest::Error) -> Self {
        RpcErr::Http(other)
    }
}

impl fmt::Display for RpcErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcErr::Http(e) => write!(f, "rpc encountered some http error: {}", e),
            _ => write!(f, "InvalidResponse"),
        }
    }
}

impl std::error::Error for RpcErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RpcErr::Http(e) => Some(e),
            _ => Some(self),
        }
    }
}
