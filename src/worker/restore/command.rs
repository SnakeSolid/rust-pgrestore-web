use super::error::WorkerError;
use super::error::WorkerResult;
use crate::jobmanager::Job;
use crate::jobmanager::JobManagerRef;
use crate::jobmanager::JobStatus;
use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct WorkerCommand<'a> {
    jobid: usize,
    settings: &'a WorkerSettings,
}

impl<'a> WorkerCommand<'a> {
    pub fn new(jobid: usize, settings: &'a WorkerSettings) -> WorkerCommand<'a> {
        WorkerCommand { jobid, settings }
    }

    fn is_aborted(&self) -> WorkerResult<bool> {
        self.settings
            .job_manager()
            .map_job(self.jobid, |job| job.status() == &JobStatus::Aborted)
            .map_err(WorkerError::map_job_error)?
            .ok_or_else(|| WorkerError::new("Job not found"))
    }

    fn wait_command(&self, mut command: Command) -> WorkerResult<CommandStatus> {
        if self.is_aborted()? {
            return Ok(CommandStatus::Aborted);
        }

        let (stdout_path, stderr_path) = self
            .settings
            .job_manager()
            .map_job(self.jobid, to_job_paths)
            .map_err(WorkerError::map_job_error)?
            .ok_or_else(|| WorkerError::new("Job not found"))?;
        let stdout = open_file(&stdout_path)?;
        let stderr = open_file(&stderr_path)?;
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(WorkerError::spawn_command_error)?;

        loop {
            if self.is_aborted()? {
                child.kill().map_err(WorkerError::kill_command_error)?;

                return Ok(CommandStatus::Aborted);
            }

            match child.try_wait().map_err(WorkerError::wait_command_error)? {
                Some(status) if status.success() => return Ok(CommandStatus::Success),
                Some(_) => return Ok(CommandStatus::Failed),
                None => thread::sleep(Duration::from_secs(1)),
            }
        }
    }

    pub fn create_database(&self, template: Option<&String>) -> WorkerResult<CommandStatus> {
        info!("Creating database {}", self.settings.database_name());

        self.settings
            .job_manager()
            .set_stage(self.jobid, "Create database")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.settings.createdb_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.settings.password())
            .arg("--host")
            .arg(self.settings.host())
            .arg("--port")
            .arg(format!("{}", self.settings.port()))
            .arg("--username")
            .arg(self.settings.role());

        if let Some(template) = template {
            command.arg("--template").arg(template);
        }

        command.arg(&self.settings.database_name());

        self.wait_command(command)
    }

    pub fn drop_database(&self) -> WorkerResult<CommandStatus> {
        info!("Dropping database {}", self.settings.database_name());

        self.settings
            .job_manager()
            .set_stage(self.jobid, "Drop database")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.settings.dropdb_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.settings.password())
            .arg("--host")
            .arg(self.settings.host())
            .arg("--port")
            .arg(format!("{}", self.settings.port()))
            .arg("--username")
            .arg(self.settings.role())
            .arg("--if-exists")
            .arg(&self.settings.database_name());

        self.wait_command(command)
    }

    pub fn restore_backup(&self, backup_path: &Path, clean: bool) -> WorkerResult<CommandStatus> {
        info!(
            "Restoring database {} from {}",
            self.settings.database_name(),
            backup_path.display()
        );

        self.settings
            .job_manager()
            .set_stage(self.jobid, "Restore database")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.settings.pgrestore_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.settings.password())
            .arg("--verbose")
            .arg("--host")
            .arg(self.settings.host())
            .arg("--port")
            .arg(format!("{}", self.settings.port()))
            .arg("--username")
            .arg(self.settings.role())
            .arg("--dbname")
            .arg(&self.settings.database_name());

        if clean {
            command.arg("--clean");
        }

        command
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--jobs")
            .arg(format!("{}", self.settings.restore_jobs()))
            .arg(&backup_path);

        self.wait_command(command)
    }

    pub fn restore_schema_only(
        &self,
        name: &str,
        backup_path: &Path,
    ) -> WorkerResult<CommandStatus> {
        info!(
            "Restoring schema only ({}) to {} from {}",
            name,
            self.settings.database_name(),
            backup_path.display(),
        );

        self.settings
            .job_manager()
            .set_stage(self.jobid, &format!("Restore schema {}", name))
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.settings.pgrestore_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.settings.password())
            .arg("--verbose")
            .arg("--host")
            .arg(self.settings.host())
            .arg("--port")
            .arg(format!("{}", self.settings.port()))
            .arg("--username")
            .arg(self.settings.role())
            .arg("--dbname")
            .arg(&self.settings.database_name())
            .arg("--schema")
            .arg(name)
            .arg("--schema-only")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--jobs")
            .arg(format!("{}", self.settings.restore_jobs()))
            .arg(backup_path);

        self.wait_command(command)
    }

    pub fn restore_schema_data(
        &self,
        name: &str,
        backup_path: &Path,
    ) -> WorkerResult<CommandStatus> {
        info!(
            "Restoring schema and data ({}) to {} from {}",
            name,
            self.settings.database_name(),
            backup_path.display(),
        );

        self.settings
            .job_manager()
            .set_stage(self.jobid, &format!("Restore schema {}", name))
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.settings.pgrestore_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.settings.password())
            .arg("--verbose")
            .arg("--host")
            .arg(self.settings.host())
            .arg("--port")
            .arg(format!("{}", self.settings.port()))
            .arg("--username")
            .arg(self.settings.role())
            .arg("--dbname")
            .arg(&self.settings.database_name())
            .arg("--schema")
            .arg(name)
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--jobs")
            .arg(format!("{}", self.settings.restore_jobs()))
            .arg(backup_path);

        self.wait_command(command)
    }

    pub fn restore_table(
        &self,
        schema: &str,
        table: &str,
        backup_path: &Path,
    ) -> WorkerResult<CommandStatus> {
        info!(
            "Restoring table {}.{} to {} from {}",
            schema,
            table,
            self.settings.database_name(),
            backup_path.display(),
        );

        self.settings
            .job_manager()
            .set_stage(self.jobid, &format!("Restore table {}.{}", schema, table))
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.settings.pgrestore_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.settings.password())
            .arg("--verbose")
            .arg("--host")
            .arg(self.settings.host())
            .arg("--port")
            .arg(format!("{}", self.settings.port()))
            .arg("--username")
            .arg(self.settings.role())
            .arg("--dbname")
            .arg(&self.settings.database_name())
            .arg("--schema")
            .arg(schema)
            .arg("--table")
            .arg(table)
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg(backup_path);

        self.wait_command(command)
    }

    pub fn restore_index(
        &self,
        schema: &str,
        index: &str,
        backup_path: &Path,
    ) -> WorkerResult<CommandStatus> {
        info!(
            "Restoring index {}.{} to {} from {}",
            schema,
            index,
            self.settings.database_name(),
            backup_path.display(),
        );

        self.settings
            .job_manager()
            .set_stage(self.jobid, &format!("Restore index {}.{}", schema, index))
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.settings.pgrestore_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.settings.password())
            .arg("--verbose")
            .arg("--host")
            .arg(self.settings.host())
            .arg("--port")
            .arg(format!("{}", self.settings.port()))
            .arg("--username")
            .arg(self.settings.role())
            .arg("--dbname")
            .arg(&self.settings.database_name())
            .arg("--schema")
            .arg(schema)
            .arg("--index")
            .arg(index)
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg(backup_path);

        self.wait_command(command)
    }
}

fn to_job_paths(job: &Job) -> (PathBuf, PathBuf) {
    let stdout_path = job.stdout_path().into();
    let stderr_path = job.stderr_path().into();

    (stdout_path, stderr_path)
}

fn open_file(path: &Path) -> WorkerResult<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(WorkerError::io_error)
}

#[derive(Debug)]
pub enum CommandStatus {
    Aborted,
    Success,
    Failed,
}

pub trait WorkerSettings: Debug {
    fn createdb_path(&self) -> &str;
    fn dropdb_path(&self) -> &str;
    fn pgrestore_path(&self) -> &str;
    fn restore_jobs(&self) -> usize;
    fn job_manager(&self) -> &JobManagerRef;
    fn host(&self) -> &str;
    fn port(&self) -> u16;
    fn role(&self) -> &str;
    fn password(&self) -> &str;
    fn database_name(&self) -> &str;
    fn ignore_errors(&self) -> bool;
}
