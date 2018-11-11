mod error;
mod job;

pub use self::error::JobManagerError;
pub use self::error::JobManagerResult;
pub use self::job::Job;
pub use self::job::JobStatus;

use config::ConfigRef;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct JobManagerRef {
    inner: Arc<RwLock<JobManager>>,
}

impl JobManagerRef {
    fn with_read<F, T>(&self, callback: F) -> JobManagerResult<T>
    where
        F: FnOnce(&JobManager) -> JobManagerResult<T>,
    {
        match self.inner.write() {
            Ok(ref jobmanager) => callback(jobmanager),
            Err(err) => {
                warn!("Failed to acquire write lock - {}", err);

                Err(JobManagerError::new("Failed to acquire write lock"))
            }
        }
    }

    fn with_write<F, T>(&self, callback: F) -> JobManagerResult<T>
    where
        F: FnOnce(&mut JobManager) -> JobManagerResult<T>,
    {
        match self.inner.write() {
            Ok(ref mut jobmanager) => callback(jobmanager),
            Err(err) => {
                warn!("Failed to acquire write lock - {}", err);

                Err(JobManagerError::new("Failed to acquire write lock"))
            }
        }
    }

    pub fn map_job<T, F>(&self, jobid: usize, callback: F) -> JobManagerResult<Option<T>>
    where
        F: FnOnce(&Job) -> T,
    {
        self.with_read(move |jobmanager| Ok(jobmanager.map_job(jobid, callback)))
    }

    pub fn next_jobid(&self) -> JobManagerResult<usize> {
        self.with_write(move |jobmanager| Ok(jobmanager.next_jobid()))
    }

    pub fn set_stage(&self, jobid: usize, stage: &str) -> JobManagerResult<()> {
        self.with_write(move |jobmanager| Ok(jobmanager.set_stage(jobid, stage)))
    }

    pub fn set_complete(&self, jobid: usize, success: bool) -> JobManagerResult<()> {
        self.with_write(move |jobmanager| Ok(jobmanager.set_complete(jobid, success)))
    }

    pub fn extend_stdout(&self, jobid: usize, buffer: &[u8]) -> JobManagerResult<()> {
        self.with_write(move |jobmanager| Ok(jobmanager.extend_stdout(jobid, buffer)))
    }

    pub fn extend_stderr(&self, jobid: usize, buffer: &[u8]) -> JobManagerResult<()> {
        self.with_write(move |jobmanager| Ok(jobmanager.extend_stderr(jobid, buffer)))
    }
}

#[derive(Debug)]
struct JobManager {
    config: ConfigRef,
    n_jobs: usize,
    last_jobid: usize,
    jobs: HashMap<usize, Job>,
}

impl JobManager {
    fn new(config: ConfigRef) -> JobManager {
        JobManager {
            config,
            n_jobs: 10,
            last_jobid: 0,
            jobs: HashMap::new(),
        }
    }

    fn map_job<T, F>(&self, jobid: usize, callback: F) -> Option<T>
    where
        F: FnOnce(&Job) -> T,
    {
        match self.jobs.get(&jobid) {
            Some(job) => Some(callback(job)),
            None => None,
        }
    }

    fn next_jobid(&mut self) -> usize {
        self.last_jobid += 1;
        self.jobs.insert(self.last_jobid, Job::default());

        if self.last_jobid > self.n_jobs {
            let last_keep_jobid = self.last_jobid - self.n_jobs;

            self.jobs.retain(|&id, _| id > last_keep_jobid);
        }

        self.last_jobid
    }

    fn set_stage(&mut self, jobid: usize, stage: &str) {
        match self.jobs.get_mut(&jobid) {
            Some(job) => {
                debug!("Set job {} stage: {}", jobid, stage);

                job.set_status(JobStatus::in_progress());
                job.set_stage(stage);
            }
            None => {}
        }
    }

    fn set_complete(&mut self, jobid: usize, success: bool) {
        match self.jobs.get_mut(&jobid) {
            Some(job) => {
                debug!("Set job {} complete with {}", jobid, success);

                job.set_status(JobStatus::complete(success));
            }
            None => {}
        }
    }

    fn extend_stdout(&mut self, jobid: usize, buffer: &[u8]) {
        match self.jobs.get_mut(&jobid) {
            Some(job) => {
                debug!("Extend job {} STDOUT with {} bytes", jobid, buffer.len());

                job.extend_stdout(buffer);
            }
            None => {}
        }
    }

    fn extend_stderr(&mut self, jobid: usize, buffer: &[u8]) {
        match self.jobs.get_mut(&jobid) {
            Some(job) => {
                debug!("Extend job {} STDERR with {} bytes", jobid, buffer.len());

                job.extend_stderr(buffer);
            }
            None => {}
        }
    }
}

pub fn create(config: ConfigRef) -> JobManagerRef {
    JobManagerRef {
        inner: Arc::new(RwLock::new(JobManager::new(config))),
    }
}
