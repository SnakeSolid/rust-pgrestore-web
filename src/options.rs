use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "pgrestore-web")]
pub struct Options {
    #[structopt(
        short = "c",
        long = "config-path",
        name = "CONFIG_PATH",
        help = "Use CONFIG_PATH as configeration file",
        default_value = "config.yaml",
        parse(from_os_str)
    )]
    config_path: PathBuf,

    #[structopt(
        short = "a",
        long = "address",
        name = "ADDR",
        help = "Listen on given address",
        default_value = "localhost"
    )]
    address: String,

    #[structopt(
        short = "p",
        long = "port",
        name = "PORT",
        help = "Listen on given port",
        default_value = "8080"
    )]
    port: u16,
}

impl Options {
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
