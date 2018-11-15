use super::error::WorkerError;
use super::error::WorkerResult;

use jobmanager::JobManagerRef;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

pub struct WorkerCommand<'a> {
    jobid: usize,
    settings: &'a WorkerSettings,
}

impl<'a> WorkerCommand<'a> {
    pub fn new(jobid: usize, settings: &'a WorkerSettings) -> WorkerCommand<'a> {
        WorkerCommand { jobid, settings }
    }

    fn read_stdout(&self, reader: &mut Read) -> WorkerResult<()> {
        let mut buffer = [0; 8192];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => return Ok(()),
                Ok(n) => self
                    .settings
                    .job_manager()
                    .extend_stdout(self.jobid, &buffer[..n])
                    .map_err(WorkerError::extend_stdout_error)?,
                Err(err) => return Err(WorkerError::io_error(err)),
            }
        }
    }

    fn read_stderr(&self, reader: &mut Read) -> WorkerResult<()> {
        let mut buffer = [0; 8192];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => return Ok(()),
                Ok(n) => self
                    .settings
                    .job_manager()
                    .extend_stderr(self.jobid, &buffer[..n])
                    .map_err(WorkerError::extend_stderr_error)?,
                Err(err) => return Err(WorkerError::io_error(err)),
            }
        }
    }

    fn wait_command(&self, mut command: Command) -> WorkerResult<()> {
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(WorkerError::spawn_command_error)?;

        if let Some(ref mut stdout) = child.stdout {
            self.read_stdout(stdout)?;
        }

        if let Some(ref mut stderr) = child.stderr {
            self.read_stderr(stderr)?;
        }

        let status = child.wait().map_err(WorkerError::wait_command_error)?;

        if status.success() {
            Ok(())
        } else {
            Err(WorkerError::new("Command returns non success exit code"))
        }
    }

    pub fn create_database(&self) -> WorkerResult<()> {
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
            .arg(self.settings.role())
            .arg(&self.settings.database_name());

        self.wait_command(command)
    }

    pub fn create_schema(&self, name: &str) -> WorkerResult<()> {
        info!(
            "Creating schema {} in database {}",
            name,
            self.settings.database_name()
        );

        self.settings
            .job_manager()
            .set_stage(self.jobid, "Create schema")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.settings.psql_path());

        command
            .env("PGPASSWORD", self.settings.password())
            .arg("--host")
            .arg(self.settings.host())
            .arg("--port")
            .arg(format!("{}", self.settings.port()))
            .arg("--username")
            .arg(self.settings.role())
            .arg("--command")
            .arg(format!("CREATE SCHEMA IF NOT EXISTS {}", name))
            .arg(&self.settings.database_name());

        self.wait_command(command)
    }

    pub fn drop_database(&self) -> WorkerResult<()> {
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
            .arg(&self.settings.database_name());

        self.wait_command(command)
    }

    pub fn restore_backup(&self, backup_path: &Path) -> WorkerResult<()> {
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
            .arg(&self.settings.database_name())
            .arg("--clean")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--jobs")
            .arg(format!("{}", self.settings.restore_jobs()))
            .arg(&backup_path);

        self.wait_command(command)
    }

    pub fn restore_schema(&self, name: &str, backup_path: &Path) -> WorkerResult<()> {
        info!(
            "Restoring schema {} to {} from {}",
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
            .arg("--clean")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--jobs")
            .arg(format!("{}", self.settings.restore_jobs()))
            .arg(backup_path);

        self.wait_command(command)
    }

    pub fn restore_schema_only(&self, name: &str, backup_path: &Path) -> WorkerResult<()> {
        info!(
            "Restoring schema only {} to {} from {}",
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

    pub fn truncate_table(&self, schema: &str, table: &str) -> WorkerResult<()> {
        info!(
            "Truncate table {}.{} in database {}",
            schema,
            table,
            self.settings.database_name()
        );

        self.settings
            .job_manager()
            .set_stage(self.jobid, "Truncate table")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.settings.psql_path());

        command
            .env("PGPASSWORD", self.settings.password())
            .arg("--host")
            .arg(self.settings.host())
            .arg("--port")
            .arg(format!("{}", self.settings.port()))
            .arg("--username")
            .arg(self.settings.role())
            .arg("--command")
            .arg(format!("TRUNCATE {}.{}", schema, table))
            .arg(&self.settings.database_name());

        self.wait_command(command)
    }

    pub fn restore_table(
        &mut self,
        schema: &str,
        table: &str,
        backup_path: &Path,
    ) -> WorkerResult<()> {
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
            .arg("--data-only")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg(backup_path);

        self.wait_command(command)
    }
}

pub trait WorkerSettings {
    fn createdb_path(&self) -> &str;
    fn dropdb_path(&self) -> &str;
    fn pgrestore_path(&self) -> &str;
    fn psql_path(&self) -> &str;
    fn restore_jobs(&self) -> usize;
    fn job_manager(&self) -> &JobManagerRef;
    fn host(&self) -> &str;
    fn port(&self) -> u16;
    fn role(&self) -> &str;
    fn password(&self) -> &str;
    fn database_name(&self) -> &str;
    fn ignore_errors(&self) -> bool;
}
