use reqwest::Error as ReqwestError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;
use std::sync::PoisonError;

pub type HttpClientResult<T> = Result<T, HttpClientError>;

#[derive(Debug)]
pub struct HttpClientError {
    message: String,
}

impl HttpClientError {
    pub fn reqwest_error(error: ReqwestError) -> HttpClientError {
        warn!("Reqwest error - {}", error);

        HttpClientError {
            message: format!("{}", error),
        }
    }

    pub fn mutex_lock_error<T>(error: PoisonError<T>) -> HttpClientError {
        warn!("Mutex lock error - {}", error);

        HttpClientError {
            message: format!("{}", error),
        }
    }

    pub fn io_error(error: IoError) -> HttpClientError {
        warn!("IO error - {}", error);

        HttpClientError {
            message: format!("{}", error),
        }
    }
}

impl Error for HttpClientError {}

impl Display for HttpClientError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
