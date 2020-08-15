#![forbid(unsafe_code)]
use std::boxed::Box;
use std::error::Error;
use std::fmt;
use std::net::{
    Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use futures::future::try_join;

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

enum Command {
    Connect = 0x01,
    Bind = 0x02,
    UdpAssosiate = 0x3,
}

impl Command {
    fn from(n: usize) -> Option<Command> {
        match n {
            1 => Some(Command::Connect),
            2 => Some(Command::Bind),
            3 => Some(Command::UdpAssosiate),
            _ => None,
        }
    }
}

#[derive(PartialEq)]
enum AddrType {
    V4 = 0x01,
    Domain = 0x03,
    V6 = 0x04,
}

impl AddrType {
    fn from(n: usize) -> Option<AddrType> {
        match n {
            1 => Some(AddrType::V4),
            3 => Some(AddrType::Domain),
            4 => Some(AddrType::V6),
            _ => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
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
impl Error for Response {}
impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Something went wrong")
    }
}

pub enum Methods {
    NoAuth = 0x00,
    UserPass = 0x02,
    NoMethods = 0xFF,
}

const SOCKS_VERSION: u8 = 0x05;
const RESERVED: u8 = 0x00;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ip = "51.195.53.132";
    let port = 1080;

    let users = User::seed();

    println!("Listening on {}:{}", ip, port);
    let mut listener = TcpListener::bind((ip, port)).await.unwrap();

    loop {
        let users_instance = users.clone();
        if let Ok((socket, _address)) = listener.accept().await {
            tokio::spawn(async move {
                match auth(socket, users_instance).await {
                    Ok(_) => println!("Connection was successful"),
                    Err(err) => println!("Error: {}", err.to_string()),
                }
            });
        }
    }
}

async fn auth(mut socket: TcpStream, users: Vec<User>) -> Result<(), Box<dyn Error>> {
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

            let user = User { username, password };
            if users.contains(&user) {
                let response = [SOCKS_VERSION, Response::Success as u8];
                socket.write_all(&response).await?;

                // Serve the request
                serve(socket).await?;
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

    socket.write_all(&[SOCKS_VERSION, Response::Success as u8, RESERVED, 1, 127, 0, 0, 1, 0, 0]).await.unwrap();

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