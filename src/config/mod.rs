mod error;
mod validate;

pub use self::error::ConfigError;
pub use self::error::ConfigResult;
pub use self::validate::validate;

use std::fs::File;
use std::path::Path;
use std::sync::Arc;

pub type ConfigRef = Arc<Config>;

#[derive(Debug, Deserialize)]
pub struct Config {
    max_jobs: usize,
    joblogs_path: String,
    restore_jobs: usize,
    search_config: SearchConfig,
    http_config: HttpConfig,
    commands: Commands,
    destinations: Vec<Destination>,
}

impl Config {
    pub fn max_jobs(&self) -> usize {
        self.max_jobs
    }

    pub fn joblogs_path(&self) -> &str {
        &self.joblogs_path
    }

    pub fn restore_jobs(&self) -> usize {
        self.restore_jobs
    }

    pub fn search_config(&self) -> &SearchConfig {
        &self.search_config
    }

    pub fn http_config(&self) -> &HttpConfig {
        &self.http_config
    }

    pub fn commands(&self) -> &Commands {
        &self.commands
    }

    pub fn destinations(&self) -> &[Destination] {
        &self.destinations
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchConfig {
    interval: u64,
    recursion_limit: Option<usize>,
    directories: Vec<String>,
    extensions: Vec<String>,
}

impl SearchConfig {
    pub fn interval(&self) -> u64 {
        self.interval
    }

    pub fn recursion_limit(&self) -> Option<usize> {
        self.recursion_limit
    }

    pub fn directories(&self) -> &[String] {
        &self.directories
    }

    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    download_directory: String,
    #[serde(default)]
    root_certificates: Vec<String>,
    #[serde(default)]
    accept_invalid_hostnames: bool,
    #[serde(default)]
    accept_invalid_certs: bool,
}

impl HttpConfig {
    pub fn download_directory(&self) -> &str {
        &self.download_directory
    }

    pub fn root_certificates(&self) -> &[String] {
        &self.root_certificates
    }

    pub fn accept_invalid_hostnames(&self) -> bool {
        self.accept_invalid_hostnames
    }

    pub fn accept_invalid_certs(&self) -> bool {
        self.accept_invalid_certs
    }
}

#[derive(Debug, Deserialize)]
pub struct Commands {
    createdb_path: String,
    dropdb_path: String,
    pgrestore_path: String,
}

impl Commands {
    pub fn createdb_path(&self) -> &str {
        &self.createdb_path
    }

    pub fn dropdb_path(&self) -> &str {
        &self.dropdb_path
    }

    pub fn pgrestore_path(&self) -> &str {
        &self.pgrestore_path
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Destination {
    host: String,
    port: u16,
    role: String,
    password: String,
}

impl Destination {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

pub fn load<P>(path: P) -> ConfigResult<ConfigRef>
where
    P: AsRef<Path>,
{
    let reader = File::open(path).map_err(ConfigError::io_error)?;
    let config = serde_yaml::from_reader(reader).map_err(ConfigError::yaml_error)?;

    Ok(Arc::new(config))
}
