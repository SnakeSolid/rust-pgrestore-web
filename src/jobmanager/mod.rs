mod error;
mod job;

pub use self::error::JobManagerError;
pub use self::error::JobManagerResult;
pub use self::job::Job;
pub use self::job::JobStatus;

use config::ConfigRef;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
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

    pub fn for_each<F>(&self, callback: F) -> JobManagerResult<()>
    where
        F: FnMut(usize, &Job),
    {
        self.with_read(move |jobmanager| Ok(jobmanager.for_each(callback)))
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
}

#[derive(Debug)]
struct JobManager {
    max_jobs: usize,
    joblogs_path: PathBuf,
    last_jobid: usize,
    jobs: HashMap<usize, Job>,
}

impl JobManager {
    fn new(config: ConfigRef) -> JobManager {
        JobManager {
            max_jobs: config.max_jobs(),
            joblogs_path: config.joblogs_path().into(),
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

    pub fn for_each<F>(&self, mut callback: F)
    where
        F: FnMut(usize, &Job),
    {
        for (&jobid, job) in &self.jobs {
            callback(jobid, job);
        }
    }

    fn next_jobid(&mut self) -> usize {
        self.last_jobid += 1;

        let (stdout_path, stderr_path) = prepare_job_logs(&self.joblogs_path, self.last_jobid);

        self.jobs
            .insert(self.last_jobid, Job::new(&stdout_path, &stderr_path));

        if self.last_jobid > self.max_jobs {
            let last_keep_jobid = self.last_jobid - self.max_jobs;

            for (&id, _) in self.jobs.iter().filter(|(&id, _)| id <= last_keep_jobid) {
                prepare_job_logs(&self.joblogs_path, id);
            }

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
}

fn prepare_job_logs(joblogs_path: &Path, jobid: usize) -> (PathBuf, PathBuf) {
    let stdout_path = joblogs_path.join(format!("job-{}-stdout.log", jobid));
    let stderr_path = joblogs_path.join(format!("job-{}-stderr.log", jobid));

    remove_job_output(&stdout_path);
    remove_job_output(&stderr_path);

    (stdout_path, stderr_path)
}

fn remove_job_output(path: &Path) {
    if path.exists() {
        debug!("Removing job output {}", path.display());

        if path.is_file() {
            if let Err(err) = fs::remove_file(&path) {
                warn!(
                    "Failed to remove job output path {} - {}",
                    path.display(),
                    err
                )
            }
        } else {
            warn!("Job output path {} is not a file", path.display(),)
        }
    }
}

pub fn create(config: ConfigRef) -> JobManagerRef {
    JobManagerRef {
        inner: Arc::new(RwLock::new(JobManager::new(config))),
    }
}
