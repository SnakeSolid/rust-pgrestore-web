use crate::config::ConfigError;
use crate::http::HttpClientError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type ApplicationResult = Result<(), ApplicationError>;

#[derive(Debug)]
pub enum ApplicationError {
    LoadConfigError { message: String },
    ConfigError { message: String },
    HttpClientError { message: String },
}

impl ApplicationError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn read_config_error(error: ConfigError) -> ApplicationError {
        error!("Failed to read configuration - {}", error);

        ApplicationError::LoadConfigError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn config_error(error: ConfigError) -> ApplicationError {
        error!("Invalid configuration - {}", error);

        ApplicationError::ConfigError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn http_client_error(error: HttpClientError) -> ApplicationError {
        error!("HTTP client error - {}", error);

        ApplicationError::HttpClientError {
            message: format!("{}", error),
        }
    }
}

impl Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ApplicationError::LoadConfigError { message } => write!(f, "{}", message),
            ApplicationError::ConfigError { message } => write!(f, "{}", message),
            ApplicationError::HttpClientError { message } => write!(f, "{}", message),
        }
    }
}
