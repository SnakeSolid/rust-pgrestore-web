use config::ConfigError;
use http::HttpClientError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type ApplicationResult = Result<(), ApplicationError>;

#[derive(Debug)]
pub enum ApplicationError {
    LoadConfigError { message: String },
    HttpClientError { message: String },
}

impl ApplicationError {
    pub fn read_config_error(error: ConfigError) -> ApplicationError {
        error!("Failed to read configuration - {}", error);

        ApplicationError::LoadConfigError {
            message: error.description().into(),
        }
    }

    pub fn http_client_error(error: HttpClientError) -> ApplicationError {
        error!("HTTP client error - {}", error);

        ApplicationError::HttpClientError {
            message: error.description().into(),
        }
    }
}

impl Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ApplicationError::LoadConfigError { message } => write!(f, "{}", message),
            ApplicationError::HttpClientError { message } => write!(f, "{}", message),
        }
    }
}
