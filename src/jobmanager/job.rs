use time;

#[derive(Debug)]
pub struct Job {
    created: i64,
    status: JobStatus,
    stage: Option<String>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Job {
    pub fn new() -> Job {
        let created = time::get_time().sec;

        Job {
            created,
            status: JobStatus::Pending,
            stage: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    pub fn set_status(&mut self, status: JobStatus) {
        self.status = status;
    }

    pub fn set_stage(&mut self, stage: &str) {
        self.stage = Some(stage.into());
    }

    pub fn extend_stdout(&mut self, buffer: &[u8]) {
        self.stdout.extend(buffer);
    }

    pub fn extend_stderr(&mut self, buffer: &[u8]) {
        self.stderr.extend(buffer);
    }

    pub fn created(&self) -> i64 {
        self.created
    }

    pub fn status(&self) -> &JobStatus {
        &self.status
    }

    pub fn stage(&self) -> Option<&String> {
        self.stage.as_ref()
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
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
