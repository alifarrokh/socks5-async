#![forbid(unsafe_code)]
use tokio::{
    net::{TcpListener, TcpStream},
    io::{AsyncReadExt, AsyncWriteExt}
};
use std::error::Error;

#[derive(Clone,Debug, PartialEq)]
pub struct User {
    username: String,
    password: String
}

impl User {
    pub fn seed() -> Vec<User> {
        vec![
            User{username: "ali".to_string(), password: "123456".to_string()},
            User{username: "admin".to_string(), password: "123456".to_string()},
        ]
    }
}

const SOCKS_VERSION: u8 = 0x05;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    let ip = "127.0.0.1" ;
    let port = 1080 ;

    let users = User::seed();

    println!("Listening on {}:{}", ip, port);
    let mut listener = TcpListener::bind((ip, port)).await.unwrap();

    loop {
        if let Ok((mut socket, _address)) = listener.accept().await {
            tokio::spawn(async move {
                socket.write_all(b"Hello World !").await;
                let mut headers = [0u8; 2];
                socket.read_exact(&mut headers).await;
                socket.write_all(&headers).await;
            });
        }
    }

}
