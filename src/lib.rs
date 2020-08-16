#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
mod socks;
mod users;

use futures::future::try_join;
use socks::{AddrType, Command, Method, Response, RESERVED, VERSION5};
use std::{
    boxed::Box,
    error::Error,
    net::{Shutdown, SocketAddr},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use users::User;

// Represnts a Socks5 Server
pub struct Socks5 {
    listener: TcpListener,
}
impl Socks5 {
    pub async fn new(socket_addr: SocketAddr) -> Socks5 {
        println!("Listening on {}", socket_addr);
        Socks5 {
            listener: TcpListener::bind(socket_addr).await.unwrap(),
        }
    }
    pub async fn serve(&mut self) {
        loop {
            if let Ok((socket, address)) = self.listener.accept().await {
                tokio::spawn(async move {
                    info!("Client connected: {}", address);
                    let mut client = SocksClient::new(socket);
                    match client.serve().await {
                        Ok(_) => info!("Request was served successfully."),
                        Err(err) => error!("{}", err.to_string()),
                    }
                });
            }
        }
    }
}

// Represents a Socks5 Client (conenction)
struct SocksClient {
    socket: TcpStream,
}
impl SocksClient {
    fn new(socket: TcpStream) -> SocksClient {
        SocksClient { socket }
    }

    fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        self.socket.shutdown(Shutdown::Both)?;
        warn!("Socket was shutdown.");
        Ok(())
    }

    async fn serve(&mut self) -> Result<(), Box<dyn Error>> {
        let mut header = [0u8; 2];
        self.socket.read_exact(&mut header).await?;

        // Accept only version 5
        if header[0] != VERSION5 {
            self.shutdown()?;
            Err(Response::Failure)?;
        }

        // Get available methods
        let methods = Method::get_available_methods(header[1], &mut self.socket).await?;

        // Authenticate the user
        self.auth(methods).await?;

        // Handle the request
        self.handle_req().await?;

        Ok(())
    }

    async fn auth(&mut self, methods: Vec<Method>) -> Result<(), Box<dyn Error>> {
        if methods.contains(&Method::UserPass) {
            // Authenticate with username/password
            self.socket
                .write_all(&[VERSION5, Method::UserPass as u8])
                .await?;

            // Read username
            let mut ulen = [0u8; 2];
            self.socket.read_exact(&mut ulen).await?;
            let ulen = ulen[1];
            let mut username: Vec<u8> = Vec::with_capacity(ulen as usize);
            for _ in 0..ulen {
                username.push(0)
            }
            self.socket.read_exact(&mut username).await?;
            let username = String::from_utf8(username).unwrap();

            // Read Password
            let mut plen = [0u8; 1];
            self.socket.read_exact(&mut plen).await?;
            let plen = plen[0];
            let mut password: Vec<u8> = Vec::with_capacity(plen as usize);
            for _ in 0..plen {
                password.push(0)
            }
            self.socket.read_exact(&mut password).await?;
            let password = String::from_utf8(password).unwrap();

            // Authenticate user
            let user = User::new(username, password);
            if User::auth(&user) {
                info!("User authenticated: {}", user.get_username());
                self.socket.write_all(&[1, Response::Success as u8]).await?;
            } else {
                self.socket
                    .write_all(&[VERSION5, Response::Failure as u8])
                    .await?;
                self.shutdown()?;
            }
        } else if methods.contains(&Method::NoAuth) { // TODO: disable it by default
            warn!("Client connected with no authentication");
            self.socket
                .write_all(&[VERSION5, Method::NoAuth as u8])
                .await?
        } else {
            self.socket
                .write_all(&[VERSION5, Response::Failure as u8])
                .await?;
            self.shutdown()?;
        }
        Ok(())
    }

    async fn handle_req(&mut self) -> Result<(), Box<dyn Error>> {
        // Read request header
        let mut data = [0u8; 3];
        self.socket.read(&mut data).await?;

        // Read socket address
        let addresses = AddrType::get_socket_addrs(&mut self.socket).await?;

        // Proccess the command
        match Command::from(data[1] as usize) {
            // Note: Currently only connect is accepted
            Some(Command::Connect) => self.cmd_connect(addresses).await?,
            _ => {
                self.shutdown()?;
                Err(Response::CommandNotSupported)?;
            }
        };

        Ok(())
    }

    async fn cmd_connect(&mut self, addrs: Vec<SocketAddr>) -> Result<(), Box<dyn Error>> {
        let mut dest = TcpStream::connect(&addrs[..]).await?;

        self.socket.write_all(&[VERSION5, Response::Success as u8, RESERVED, 1, 127, 0, 0, 1, 0, 0]).await.unwrap();
    
        let (mut ro, mut wo) = dest.split();
        let (mut ri, mut wi) = self.socket.split();
    
        let client_to_server = async {
            tokio::io::copy(&mut ri, &mut wo).await?;
            wo.shutdown().await
        };
    
        let server_to_client = async {
            tokio::io::copy(&mut ro, &mut wi).await?;
            wi.shutdown().await
        };
    
        try_join(client_to_server, server_to_client).await?;

        Ok(())
    }
}
