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
mod error;
mod handler;
mod http;
mod jobmanager;
mod options;
mod pathmanager;
mod server;
mod worker;

use error::ApplicationError;
use error::ApplicationResult;
use options::Options;
use structopt::StructOpt;

fn main() -> ApplicationResult {
    env_logger::init();

    let options = Options::from_args();
    let config =
        config::load(options.config_path()).map_err(ApplicationError::read_config_error)?;

    config::validate(config.clone()).map_err(ApplicationError::config_error)?;

    let path_manager = pathmanager::create();
    let http_client = http::create(config.clone()).map_err(ApplicationError::http_client_error)?;
    let job_manager = jobmanager::create(config.clone());

    worker::start_search(config.clone(), path_manager.clone());
    server::start(&options, config, job_manager, path_manager, http_client);

    Ok(())
}
