#![forbid(unsafe_code)]

use std::{
    boxed::Box,
    error::Error,
    net::SocketAddr
};
use structopt::StructOpt;
use socks5::{Socks5, options};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    pretty_env_logger::init_timed();

    let options = options::Options::from_args();

    let address:SocketAddr = format!("{}:{}", options.ip, options.port).as_str().parse().expect("Invalid socket address.");

    let mut socks5 = Socks5::new(address, options).await;
    socks5.serve().await;

    Ok(())
    
}
