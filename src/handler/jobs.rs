use super::util::handle_empty;
use super::HandlerError;
use crate::config::ConfigRef;
use crate::jobmanager::JobManagerRef;
use crate::jobmanager::JobStatus;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;

#[derive(Debug)]
pub struct JobsHandler {
    config: ConfigRef,
    job_manager: JobManagerRef,
}

impl JobsHandler {
    pub fn new(config: ConfigRef, job_manager: JobManagerRef) -> JobsHandler {
        JobsHandler {
            config,
            job_manager,
        }
    }
}

impl Handler for JobsHandler {
    fn handle(&self, _request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_empty(move || {
            let mut result: Vec<JobData> = Vec::new();

            self.job_manager
                .for_each(|jobid, job| {
                    result.push(JobData::new(
                        jobid,
                        job.created(),
                        job.modified(),
                        job.database_name(),
                        job.status(),
                        job.stage(),
                    ))
                })
                .map_err(|_| HandlerError::new("Job manager error"))?;

            Ok(result)
        })
    }
}

#[derive(Debug, Serialize)]
struct JobData {
    jobid: usize,
    created: i64,
    modified: i64,
    database_name: String,
    status: String,
    stage: Option<String>,
}

impl JobData {
    fn new(
        jobid: usize,
        created: i64,
        modified: i64,
        database_name: &str,
        status: &JobStatus,
        stage: Option<&String>,
    ) -> JobData {
        let status = match status {
            JobStatus::Pending => "Pending",
            JobStatus::Aborted => "Aborted",
            JobStatus::InProgress => "InProgress",
            JobStatus::Complete { success } if *success => "Success",
            JobStatus::Complete { success } if !success => "Failed",
            _ => unreachable!(),
        };

        JobData {
            jobid,
            created,
            modified,
            database_name: database_name.into(),
            status: status.into(),
            stage: stage.cloned(),
        }
    }
}
