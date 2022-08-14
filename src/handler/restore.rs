use super::util::handle_request;
use super::HandlerError;
use crate::config::ConfigRef;
use crate::http::HttpClientRef;
use crate::jobmanager::JobManagerRef;
use crate::worker::RestoreWorker;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;

#[derive(Debug)]
pub struct RestoreHandler {
    config: ConfigRef,
    job_manager: JobManagerRef,
    http_client: HttpClientRef,
}

impl RestoreHandler {
    pub fn new(
        config: ConfigRef,
        job_manager: JobManagerRef,
        http_client: HttpClientRef,
    ) -> RestoreHandler {
        RestoreHandler {
            config,
            job_manager,
            http_client,
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

            match request.backup {
                Backup::Path { ref path } if path.is_empty() => {
                    return Err(HandlerError::new("Backup path must not be empty"));
                }
                Backup::Url { ref url } if url.is_empty() => {
                    return Err(HandlerError::new("Backup URL must not be empty"));
                }
                _ => {}
            }

            if request.database_name.is_empty() {
                return Err(HandlerError::new("Database name must not be empty"));
            }

            let create_database = match request.database {
                DatabaseType::Exists => false,
                DatabaseType::DropAndCreate => true,
            };
            let job_id = self
                .job_manager
                .next_jobid(&request.database_name)
                .map_err(|_| HandlerError::new("Failed to create job"))?;
            let worker = RestoreWorker::new(
                self.config.clone(),
                self.job_manager.clone(),
                destination,
                request.database_name.as_ref(),
                request.ignore_errors,
            );

            match (request.restore, request.backup) {
                (RestoreType::Full, Backup::Path { path }) => worker
                    .restore_file_full(job_id, path.as_ref(), create_database, create_database)
                    .map_err(|err| HandlerError::new(err.message()))?,
                (
                    RestoreType::Partial {
                        objects,
                        restore_schema,
                        restore_indexes,
                    },
                    Backup::Path { path },
                ) => worker
                    .restore_file_partial(
                        job_id,
                        path.as_ref(),
                        &objects,
                        restore_schema,
                        restore_indexes,
                        create_database,
                        create_database,
                    )
                    .map_err(|err| HandlerError::new(err.message()))?,
                (RestoreType::Full, Backup::Url { url }) => worker
                    .restore_url_full(
                        job_id,
                        &url,
                        self.http_client.clone(),
                        create_database,
                        create_database,
                    )
                    .map_err(|err| HandlerError::new(err.message()))?,
                (
                    RestoreType::Partial {
                        objects,
                        restore_schema,
                        restore_indexes,
                    },
                    Backup::Url { url },
                ) => worker
                    .restore_url_partial(
                        job_id,
                        &url,
                        self.http_client.clone(),
                        &objects,
                        restore_schema,
                        restore_indexes,
                        create_database,
                        create_database,
                    )
                    .map_err(|err| HandlerError::new(err.message()))?,
            }

            Ok(job_id)
        })
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    destination: usize,
    backup: Backup,
    database_name: String,
    database: DatabaseType,
    restore: RestoreType,
    ignore_errors: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Backup {
    Path { path: String },
    Url { url: String },
}

#[derive(Debug, Deserialize)]
enum DatabaseType {
    Exists,
    DropAndCreate,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum RestoreType {
    Full,
    Partial {
        objects: Vec<String>,
        restore_schema: bool,
        restore_indexes: bool,
    },
}
