#![forbid(unsafe_code)]
#[macro_use] extern crate lazy_static;
mod users;
mod socks;

use std::{
    boxed::Box,
    error::Error,
    net::{
        Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs,
    }
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use futures::future::try_join;
use users::User;
use socks::{VERSION5, RESERVED, AddrType, Command, Methods, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ip = "127.0.0.1";
    let port = 1080;

    println!("Listening on {}:{}", ip, port);
    let mut listener = TcpListener::bind((ip, port)).await.unwrap();

    loop {
        if let Ok((socket, _address)) = listener.accept().await {
            tokio::spawn(async move {
                match auth(socket).await {
                    Ok(_) => println!("Connection was successful"),
                    Err(err) => println!("Error: {}", err.to_string()),
                }
            });
        }
    }
}

async fn auth(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut header = [0u8; 2];
    socket.read_exact(&mut header).await?;

    if header[0] != VERSION5 {
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
        response[0] = VERSION5;
        if methods.contains(&(Methods::NoAuth as u8)) {
            response[1] = Methods::NoAuth as u8;
            socket.write_all(&response).await?;
            serve(socket).await?;
        } else if methods.contains(&(Methods::UserPass as u8)) {
            response[1] = Methods::UserPass as u8;
            socket.write_all(&response).await?;

            // Aithenticate user with user/pass method
            let mut ulen = [0u8; 2];
            socket.read_exact(&mut ulen).await?;
            let ulen = ulen[1];
            let mut username: Vec<u8> = Vec::with_capacity(ulen as usize);
            for _ in 0..ulen {
                username.push(0)
            }
            socket.read_exact(&mut username).await?;
            let username = String::from_utf8(username).unwrap();

            let mut plen = [0u8; 1];
            socket.read_exact(&mut plen).await?;
            let plen = plen[0];
            let mut password: Vec<u8> = Vec::with_capacity(plen as usize);
            for _ in 0..plen {
                password.push(0)
            }
            socket.read_exact(&mut password).await?;
            let password = String::from_utf8(password).unwrap();

            let user = User::new(username, password);
            if User::auth(&user) {
                let response = [1, Response::Success as u8];
                socket.write_all(&response).await?;

                // Serve the request
                serve(socket).await?;
            } else {
                let response = [VERSION5, Response::Failure as u8];
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

async fn serve(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut data = [0u8; 4];
    socket.read(&mut data).await?;

    // Note: Currently only connect is accepted
    let mut _command = Command::Connect;
    match Command::from(data[1] as usize) {
        Some(Command::Connect) => {
            Ok(())
        }
        _ => {
            socket.shutdown(Shutdown::Both)?;
            Err(Response::CommandNotSupported)
        }
    }?;

    let mut addr_type: AddrType = AddrType::V6;
    match AddrType::from(data[3] as usize) {
        Some(addr) => {
            addr_type = addr;
            Ok(())
        }
        None => {
            socket.shutdown(Shutdown::Both)?;
            Err(Response::AddrTypeNotSupported)
        }
    }?;

    let addr;
    if let AddrType::Domain = addr_type {
        let mut dlen = [0u8; 1];
        socket.read_exact(&mut dlen).await?;
        let mut domain = vec![0u8; dlen[0] as usize];
        socket.read_exact(&mut domain).await?;
        addr = domain;
    } else if let AddrType::V4 = addr_type {
        let mut v4 = [0u8; 4];
        socket.read_exact(&mut v4).await?;
        addr = Vec::from(v4);
    } else {
        let mut v6 = [0u8; 16];
        socket.read_exact(&mut v6).await?;
        addr = Vec::from(v6);
    }

    let mut port = [0u8; 2];
    socket.read_exact(&mut port).await?;
    let port = (u16::from(port[0]) << 8) | u16::from(port[1]);

    let socket_addr = addr_to_socket(&addr_type, &addr, port)?;

    let mut dest = TcpStream::connect(&socket_addr[..]).await?;

    socket.write_all(&[VERSION5, Response::Success as u8, RESERVED, 1, 127, 0, 0, 1, 0, 0]).await.unwrap();

    let (mut ro, mut wo) = dest.split();
    let (mut ri, mut wi) = socket.split();

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

fn addr_to_socket(addr_type: &AddrType, addr: &[u8], port: u16) -> Result<Vec<SocketAddr>, Box<dyn Error>> {
    match addr_type {
        AddrType::V6 => {
            let new_addr = (0..8).map(|x| {
                (u16::from(addr[(x * 2)]) << 8) | u16::from(addr[(x * 2) + 1])
            }).collect::<Vec<u16>>();


            Ok(vec![SocketAddr::from(
                SocketAddrV6::new(
                    Ipv6Addr::new(
                        new_addr[0], new_addr[1], new_addr[2], new_addr[3], new_addr[4], new_addr[5], new_addr[6], new_addr[7]), 
                    port, 0, 0)
            )])
        },
        AddrType::V4 => {
            Ok(vec![SocketAddr::from(SocketAddrV4::new(Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]), port))])
        },
        AddrType::Domain => {
            let mut domain = String::from_utf8_lossy(&addr[..]).to_string();
            domain.push_str(&":");
            domain.push_str(&port.to_string());

            Ok(domain.to_socket_addrs()?.collect())
        }

    }
}