use super::util::handle_request;
use super::HandlerError;

use config::ConfigRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use jobmanager::JobManagerRef;
use worker::Worker;

#[derive(Debug)]
pub struct RestoreHandler {
    config: ConfigRef,
    job_manager: JobManagerRef,
}

impl RestoreHandler {
    pub fn new(config: ConfigRef, job_manager: JobManagerRef) -> RestoreHandler {
        RestoreHandler {
            config,
            job_manager,
        }
    }
}

impl Handler for RestoreHandler {
    fn handle(&self, request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_request(request, move |request: Request| {
            let destination = &self
                .config
                .destinations()
                .get(request.destination)
                .ok_or_else(|| HandlerError::new("Invalid destination id"))?;

            if request.backup_path.is_empty() {
                return Err(HandlerError::new("Backup path must not be empty"));
            }

            if request.database_name.is_empty() {
                return Err(HandlerError::new("Database name must not be empty"));
            }

            let (drop_database, create_database) = match request.database {
                DatabaseType::Exists => (false, false),
                DatabaseType::Create => (false, true),
                DatabaseType::DropAndCreate => (true, true),
            };
            let job_id = self
                .job_manager
                .next_jobid()
                .map_err(|_| HandlerError::new("Failed to create job"))?;
            let worker = Worker::new(
                self.config.clone(),
                self.job_manager.clone(),
                destination,
                request.backup_path.as_ref(),
                request.database_name.as_ref(),
            );

            match request.restore {
                RestoreType::Full => worker
                    .restore_full(job_id, drop_database, create_database)
                    .map_err(|err| HandlerError::new(err.message()))?,
                _ => unimplemented!(),
            }

            Ok(job_id)
        })
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    destination: usize,
    backup_path: String,
    database_name: String,
    database: DatabaseType,
    restore: RestoreType,
}

#[derive(Debug, Deserialize)]
enum DatabaseType {
    Exists,
    Create,
    DropAndCreate,
}

#[serde(tag = "type")]
#[derive(Debug, Deserialize)]
enum RestoreType {
    Full,
    Schema { schema: Vec<String> },
    Tables { tables: Vec<String> },
}
