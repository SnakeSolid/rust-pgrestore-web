use serde_yaml::Error as YamlError;
use std::error::Error;
use std::fmt::Arguments;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn io_error(error: IoError) -> ConfigError {
        warn!("IO error - {}", error);

        ConfigError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn yaml_error(error: YamlError) -> ConfigError {
        warn!("YAML error - {}", error);

        ConfigError {
            message: format!("{}", error),
        }
    }

    pub fn format(args: Arguments) -> ConfigError {
        error!("{}", args);

        ConfigError {
            message: format!("{}", args),
        }
    }
}

impl Error for ConfigError {}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
