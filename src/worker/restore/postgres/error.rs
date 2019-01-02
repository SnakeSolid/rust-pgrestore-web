use postgres::Error as PgError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError { message: String },
    QueryExecutionError { message: String },
}

impl DatabaseError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn connection_error(error: PgError) -> DatabaseError {
        DatabaseError::ConnectionError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn query_execution_error(error: PgError) -> DatabaseError {
        DatabaseError::QueryExecutionError {
            message: format!("{}", error),
        }
    }
}

impl Error for DatabaseError {}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            DatabaseError::ConnectionError { message } => write!(f, "{}", message),
            DatabaseError::QueryExecutionError { message } => write!(f, "{}", message),
        }
    }
}
