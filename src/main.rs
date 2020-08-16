#![forbid(unsafe_code)]

use std::{
    boxed::Box,
    error::Error,
    net::SocketAddr
};
use socks5::Socks5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    pretty_env_logger::init_timed();

    let ip = "127.0.0.1";
    let port:i32= 1080;
    let address:SocketAddr = format!("{}:{}", ip, port).as_str().parse().expect("Invalid socket address.");

    let mut socks5 = Socks5::new(address).await;
    socks5.serve().await;

    Ok(())
    
}
