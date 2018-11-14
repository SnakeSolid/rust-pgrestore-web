#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

extern crate env_logger;
extern crate iron;
extern crate mount;
extern crate reqwest;
extern crate router;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate staticfile;
extern crate structopt;
extern crate time;

mod config;
mod handler;
mod http;
mod jobmanager;
mod options;
mod server;
mod worker;

use options::Options;
use structopt::StructOpt;

fn main() {
    env_logger::init();

    let options = Options::from_args();

    match config::load(options.config_path()) {
        Ok(config) => match http::create(config.clone()) {
            Ok(http_client) => {
                let job_manager = jobmanager::create(config.clone());

                server::start(&options, config, job_manager, http_client);
            }
            Err(err) => error!("Failed to read configuration: {}", err),
        },
        Err(err) => error!("Failed to read configuration: {}", err),
    }
}
