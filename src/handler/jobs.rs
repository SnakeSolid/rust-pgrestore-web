use super::util::handle_empty;
use super::HandlerError;

use config::ConfigRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use jobmanager::JobManagerRef;
use jobmanager::JobStatus;

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
                        job.status(),
                        job.stage(),
                    ))
                }).map_err(|_| HandlerError::new("Job manager error"))?;

            Ok(result)
        })
    }
}

#[derive(Debug, Serialize)]
struct JobData {
    jobid: usize,
    created: i64,
    status: String,
    stage: Option<String>,
}

impl JobData {
    fn new(jobid: usize, created: i64, status: &JobStatus, stage: Option<&String>) -> JobData {
        let status = match status {
            JobStatus::Pending => "Pending",
            JobStatus::InProgress => "InProgress",
            JobStatus::Complete { success } if *success => "Success",
            JobStatus::Complete { success } if !success => "Failed",
            _ => unreachable!(),
        };

        JobData {
            jobid,
            created,
            status: status.into(),
            stage: stage.cloned(),
        }
    }
}
