use socks5::SocksServer;
use std::{
    boxed::Box,
    error::Error,
    net::SocketAddr,
};
extern crate pretty_env_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Init log system
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    pretty_env_logger::init_timed();

    // Server address
    let address: SocketAddr = "127.0.0.1:1080".parse().unwrap();

    // users
    let users = vec![
        (String::from("user1"), String::from("123456")),
        (String::from("user2"), String::from("123456")),
    ];

    let mut socks5 = SocksServer::new(
        address,
        true, // Let users connect with no authentication
        Box::new(move |username, password| {
            // Authenticate user
            return users.contains(&(username, password));
        }),
    )
    .await;
    socks5.serve().await;

    Ok(())
}
