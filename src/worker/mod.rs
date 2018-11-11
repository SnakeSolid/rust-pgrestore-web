mod error;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;

use config::ConfigRef;
use config::Destination;
use jobmanager::JobManagerRef;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::thread::Builder;

#[derive(Debug)]
pub struct Worker {
    config: ConfigRef,
    jobmanager: JobManagerRef,
    destination: Destination,
    backup_path: PathBuf,
    database_name: String,
}

impl Worker {
    pub fn new(
        config: ConfigRef,
        jobmanager: JobManagerRef,
        destination: &Destination,
        backup_path: &Path,
        database_name: &str,
    ) -> Worker {
        Worker {
            config,
            jobmanager,
            destination: destination.clone(),
            backup_path: backup_path.into(),
            database_name: database_name.into(),
        }
    }

    fn read_stdout(&self, jobid: usize, reader: &mut Read) -> WorkerResult<()> {
        let mut buffer = [0; 8192];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => return Ok(()),
                Ok(n) => self
                    .jobmanager
                    .extend_stdout(jobid, &buffer[..n])
                    .map_err(WorkerError::extend_stdout_error)?,
                Err(err) => return Err(WorkerError::io_error(err)),
            }
        }
    }

    fn read_stderr(&self, jobid: usize, reader: &mut Read) -> WorkerResult<()> {
        let mut buffer = [0; 8192];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => return Ok(()),
                Ok(n) => self
                    .jobmanager
                    .extend_stderr(jobid, &buffer[..n])
                    .map_err(WorkerError::extend_stderr_error)?,
                Err(err) => return Err(WorkerError::io_error(err)),
            }
        }
    }

    fn wait_command(&self, jobid: usize, mut command: Command) -> WorkerResult<()> {
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(WorkerError::spawn_command_error)?;

        if let Some(ref mut stdout) = child.stdout {
            self.read_stdout(jobid, stdout)?;
        }

        if let Some(ref mut stderr) = child.stderr {
            self.read_stderr(jobid, stderr)?;
        }

        let status = child.wait().map_err(WorkerError::wait_command_error)?;

        if status.success() {
            Ok(())
        } else {
            Err(WorkerError::new("Command returns non success exit code"))
        }
    }

    fn create_database(&self, jobid: usize) -> WorkerResult<()> {
        info!("Creating database {}", self.database_name);

        self.jobmanager
            .set_stage(jobid, "Create database")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().createdb_path());

        command
            .env("PGPASSWORD", self.destination.password())
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg(&self.database_name);

        self.wait_command(jobid, command)
    }

    fn drop_database(&self, jobid: usize) -> WorkerResult<()> {
        info!("Dropping database {}", self.database_name);

        self.jobmanager
            .set_stage(jobid, "Drop database")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().dropdb_path());

        command
            .env("PGPASSWORD", self.destination.password())
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg(&self.database_name);

        self.wait_command(jobid, command)
    }

    fn restore_backup(&self, jobid: usize) -> WorkerResult<()> {
        info!(
            "Restoring database {} from {}",
            self.database_name,
            self.backup_path.display()
        );

        self.jobmanager
            .set_stage(jobid, "Restore database")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().pgrestore_path());

        command
            .env("PGPASSWORD", self.destination.password())
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg(&self.database_name);

        self.wait_command(jobid, command)
    }

    fn execute_step<F>(&self, jobid: usize, callback: F) -> WorkerResult<()>
    where
        F: FnOnce() -> WorkerResult<()>,
    {
        match callback() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.set_complete(jobid, false)?;

                Err(err)
            }
        }
    }

    fn set_complete(&self, jobid: usize, complete: bool) -> WorkerResult<()> {
        self.jobmanager
            .set_complete(jobid, complete)
            .map_err(WorkerError::set_status_error)
    }

    pub fn restore_full(
        self,
        jobid: usize,
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let _ = Builder::new()
            .name(format!("worker #{}", jobid))
            .spawn(move || {
                if drop_database {
                    self.execute_step(jobid, || self.drop_database(jobid))?;
                }

                if create_database {
                    self.execute_step(jobid, || self.create_database(jobid))?;
                }

                self.execute_step(jobid, || self.restore_backup(jobid))?;
                self.set_complete(jobid, true)
            }).map_err(WorkerError::spawn_thread_error)?;

        Ok(())
    }
}
