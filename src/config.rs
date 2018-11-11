use serde_yaml::Error as YamlError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fs::File;
use std::io::Error as IoError;
use std::path::Path;
use std::sync::Arc;

pub type ConfigRef = Arc<Config>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    createdb_path: String,
    dropdb_path: String,
    pgrestore_path: String,
    destinations: Vec<Destination>,
}

impl Config {
    pub fn createdb_path(&self) -> &str {
        &self.createdb_path
    }

    pub fn dropdb_path(&self) -> &str {
        &self.dropdb_path
    }

    pub fn pgrestore_path(&self) -> &str {
        &self.pgrestore_path
    }

    pub fn destinations(&self) -> &[Destination] {
        &self.destinations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    pub fn io_error(error: IoError) -> ConfigError {
        warn!("IO error - {}", error);

        ConfigError {
            message: format!("{}", error),
        }
    }

    pub fn yaml_error(error: YamlError) -> ConfigError {
        warn!("YAML error - {}", error);

        ConfigError {
            message: format!("{}", error),
        }
    }
}

impl Error for ConfigError {}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
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
