use std::path::Path;
use std::path::PathBuf;
use time;

#[derive(Debug)]
pub struct Job {
    created: i64,
    modified: i64,
    status: JobStatus,
    database_name: String,
    stage: Option<String>,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

impl Job {
    pub fn new(database_name: &str, stdout_path: &Path, stderr_path: &Path) -> Job {
        let created = time::get_time().sec;
        let modified = created;

        Job {
            created,
            modified,
            database_name: database_name.into(),
            status: JobStatus::Pending,
            stage: None,
            stdout_path: stdout_path.into(),
            stderr_path: stderr_path.into(),
        }
    }

    pub fn set_status(&mut self, status: JobStatus) {
        self.modified = time::get_time().sec;
        self.status = status;
    }

    pub fn set_stage(&mut self, stage: &str) {
        self.modified = time::get_time().sec;
        self.stage = Some(stage.into());
    }

    pub fn created(&self) -> i64 {
        self.created
    }

    pub fn modified(&self) -> i64 {
        self.modified
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn status(&self) -> &JobStatus {
        &self.status
    }

    pub fn stage(&self) -> Option<&String> {
        self.stage.as_ref()
    }

    pub fn stdout_path(&self) -> &Path {
        &self.stdout_path
    }

    pub fn stderr_path(&self) -> &Path {
        &self.stderr_path
    }
}

#[derive(Debug)]
pub enum JobStatus {
    Pending,
    InProgress,
    Complete { success: bool },
}

impl JobStatus {
    pub fn in_progress() -> JobStatus {
        JobStatus::InProgress
    }

    pub fn complete(success: bool) -> JobStatus {
        JobStatus::Complete { success }
    }
}
