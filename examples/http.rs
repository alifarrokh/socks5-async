use socks5_async::{SocksStream, TargetAddr};
use std::{
    boxed::Box,
    error::Error,
    net::{SocketAddr},
};
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // Enter a valid SOCKS5 server
    let proxy_addr: SocketAddr = "127.0.0.1:1080".parse().unwrap();

    // Send request to google.com
    let target_addr = TargetAddr::Domain((String::from("google.com"), 80));
    
    // Pass Some((String, String)) to authenticate with username & password
    let userpass = None;

    // Connect to server
    let mut stream = SocksStream::connect(proxy_addr, target_addr, userpass).await?;

    // Send a simple HTTP request
    stream.write(b"GET / HTTP/1.1").await?;
    stream.write(&[0x0d, 0x0a, 0x0d, 0x0a]).await?;
    
    // Read response
    let mut buf = [0; 256]; // 256 bytes => only read few first headers
    stream.read(&mut buf).await.expect("Unable to read the response");
    println!("{}", std::str::from_utf8(&buf).unwrap());

    Ok(())
}