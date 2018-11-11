#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

extern crate env_logger;
extern crate iron;
extern crate mount;
extern crate router;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate staticfile;
extern crate structopt;
extern crate time;

mod config;
mod handler;
mod worker;
mod options;
mod server;

use options::Options;
use structopt::StructOpt;

fn main() {
    env_logger::init();

    let options = Options::from_args();

    match config::load(options.config_path()) {
        Ok(config) => {
            server::start(&options, config);
        }
        Err(err) => error!("Failed to read configuration: {}", err),
    }
}
