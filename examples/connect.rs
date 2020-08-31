use socks5_async::{SocksStream, TargetAddr};
use std::{
    boxed::Box,
    error::Error,
    net::{SocketAddr, SocketAddrV4},
};
use tokio::prelude::*;
extern crate pretty_env_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // SOCKS5 proxy server address
    let proxy: SocketAddr = "127.0.0.1:1080".parse().unwrap();

    // Target address
    let target: SocketAddrV4 = "127.0.0.1:3033".parse().unwrap();

    // Connect to server
    let stream = SocksStream::connect(
        proxy,
        TargetAddr::V4(target),
        Some(("user1".to_string(), "123456".to_string())), // Pass None if you want to use NoAuth method
    )
    .await?;

    // Use tcp stream ...

    Ok(())
}
