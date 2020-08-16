use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "socks5")]
pub struct Options {
    #[structopt(long = "ip", short = "i", default_value = "127.0.0.1")]
    pub ip: String,

    #[structopt(long = "port", short = "p", default_value = "1080")]
    pub port: u16,

    #[structopt(long = "no-auth")]
    pub no_auth: bool,
}