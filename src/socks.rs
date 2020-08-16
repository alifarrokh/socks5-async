#[allow(dead_code)]
use std::{error::Error, fmt};

// Const bytes
pub const VERSION5: u8 = 0x05;
pub const RESERVED: u8 = 0x00;

// Request command
pub enum Command {
    Connect = 0x01,
    Bind = 0x02,
    UdpAssosiate = 0x3,
}
impl Command {
    pub fn from(byte: usize) -> Option<Command> {
        match byte {
            1 => Some(Command::Connect),
            2 => Some(Command::Bind),
            3 => Some(Command::UdpAssosiate),
            _ => None,
        }
    }
}

// Request address type
#[derive(PartialEq)]
pub enum AddrType {
    V4 = 0x01,
    Domain = 0x03,
    V6 = 0x04,
}
impl AddrType {
    pub fn from(byte: usize) -> Option<AddrType> {
        match byte {
            1 => Some(AddrType::V4),
            3 => Some(AddrType::Domain),
            4 => Some(AddrType::V6),
            _ => None,
        }
    }
}

// Server response codes
#[derive(Debug)]
pub enum Response {
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

// Authentication methods
pub enum Methods {
    NoAuth = 0x00,
    UserPass = 0x02,
    NoMethods = 0xFF,
}