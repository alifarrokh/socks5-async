#![forbid(unsafe_code)]
use std::error::Error;
use std::net::Shutdown;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    username: String,
    password: String,
}

impl User {
    pub fn seed() -> Vec<User> {
        vec![
            User {
                username: "ali".to_string(),
                password: "123456".to_string(),
            },
            User {
                username: "admin".to_string(),
                password: "123456".to_string(),
            },
        ]
    }
}

enum Response {
    Success = 0x00,
    Failure = 0x01,
    RuleFailure = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TtlExpired = 0x06,
    CommandNotSupported = 0x07,
    AddrTypeNotSupported = 0x08,
}

pub enum Methods {
    NoAuth = 0x00,
    UserPass = 0x02,
    NoMethods = 0xFF,
}

const SOCKS_VERSION: u8 = 0x05;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ip = "127.0.0.1";
    let port = 1080;

    let users = User::seed();

    println!("Listening on {}:{}", ip, port);
    let mut listener = TcpListener::bind((ip, port)).await.unwrap();

    loop {
        let users_instance = users.clone();
        if let Ok((mut socket, _address)) = listener.accept().await {
            tokio::spawn(async move {

                match serve(socket, users_instance).await {
                    Ok(_) => println!("Connection was successful"),
                    Err(err) => println!("Error: {}", err.to_string()),
                }

            });
        }
    }
}

async fn serve(mut socket: TcpStream, users: Vec<User>) -> Result<(), Box<dyn Error>> {
    let mut header = [0u8; 2];
    socket.read_exact(&mut header).await?;

    if header[0] != SOCKS_VERSION {
        socket.shutdown(Shutdown::Both)?;
    } else {
        let methods_count = header[1];
        let mut methods: Vec<u8> = Vec::with_capacity(methods_count as usize);
        for _ in 0..methods_count {
            let mut method = [0u8; 1];
            socket.read_exact(&mut method).await?;
            methods.push(method[0]);
        }

        let mut response = [0u8; 2];
        response[0] = SOCKS_VERSION;
        if methods.contains(&(Methods::UserPass as u8)) {
            response[1] = Methods::UserPass as u8;
            socket.write_all(&response).await?;

            // Aithenticate user with user/pass method
            let mut ulen = [0u8; 2];
            socket.read_exact(&mut ulen).await?;
            let ulen = ulen[1];
            let mut username: Vec<u8> = Vec::with_capacity(ulen as usize);
            for i in 0..ulen {
                username.push(0)
            }
            socket.read_exact(&mut username).await?;
            let username = String::from_utf8(username).unwrap();

            let mut plen = [0u8; 1];
            socket.read_exact(&mut plen).await?;
            let plen = plen[1];
            let mut password: Vec<u8> = Vec::with_capacity(plen as usize);
            for _ in 0..plen {
                password.push(0)
            }
            socket.read_exact(&mut password).await?;
            let password = String::from_utf8(password).unwrap();

            let user = User { username, password };
            if users.contains(&user) {
                let response = [SOCKS_VERSION, Response::Success as u8];
                socket.write_all(&response).await?;

                // TODO: Serve the request
            } else {
                let response = [SOCKS_VERSION, Response::Failure as u8];
                socket.write_all(&response).await?;
                socket.shutdown(Shutdown::Both)?;
            }
        } else {
            response[1] = Response::Failure as u8;
            socket.write_all(&response).await?;
            socket.shutdown(Shutdown::Both)?;
        }
    }

    Ok(())
}
