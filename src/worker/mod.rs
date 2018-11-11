mod error;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;

use config::ConfigRef;
use config::Destination;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::thread::Builder;

#[derive(Debug)]
pub struct Worker {
    config: ConfigRef,
    destination: Destination,
    backup_path: PathBuf,
    database_name: String,
}

impl Worker {
    pub fn new(
        config: ConfigRef,
        destination: &Destination,
        backup_path: &Path,
        database_name: &str,
    ) -> Worker {
        Worker {
            config,
            destination: destination.clone(),
            backup_path: backup_path.into(),
            database_name: database_name.into(),
        }
    }

    fn create_database(&self) -> WorkerResult<()> {
        info!("Dropping database {}", self.database_name);

        let status = Command::new(self.config.createdb_path())
            .env("PGPASSWORD", self.destination.password())
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg(&self.database_name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                warn!("Failed to spawn process - {}", err);

                WorkerError::new(&format!("{}", err))
            })?.wait()
            .map_err(|err| {
                warn!("Failed to wait process - {}", err);

                WorkerError::new(&format!("{}", err))
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(WorkerError::new("Command returns non success exit code"))
        }
    }

    fn drop_database(&self) -> WorkerResult<()> {
        info!("Dropping database {}", self.database_name);

        let status = Command::new(self.config.dropdb_path())
            .env("PGPASSWORD", self.destination.password())
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg(&self.database_name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                warn!("Failed to spawn process - {}", err);

                WorkerError::new(&format!("{}", err))
            })?.wait()
            .map_err(|err| {
                warn!("Failed to wait process - {}", err);

                WorkerError::new(&format!("{}", err))
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(WorkerError::new("Command returns non success exit code"))
        }
    }

    fn restore_backup(&self) -> WorkerResult<()> {
        info!(
            "Restoring database {} from {}",
            self.database_name,
            self.backup_path.display()
        );

        let status = Command::new(self.config.pgrestore_path())
            .env("PGPASSWORD", self.destination.password())
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg(&self.database_name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                warn!("Failed to spawn process - {}", err);

                WorkerError::new(&format!("{}", err))
            })?.wait()
            .map_err(|err| {
                warn!("Failed to wait process - {}", err);

                WorkerError::new(&format!("{}", err))
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(WorkerError::new("Command returns non success exit code"))
        }
    }

    pub fn restore_full(
        self,
        job_id: usize,
        create_database: bool,
        drop_database: bool,
    ) -> WorkerResult<()> {
        let _ = Builder::new()
            .name(format!("worker #{}", job_id))
            .spawn(move || {
                if drop_database {
                    self.drop_database()?;
                }

                if create_database {
                    self.create_database()?;
                }

                self.restore_backup()
            }).map_err(|err| {
                warn!("Failed to start worker thread - {}", err);

                WorkerError::new(&format!("{}", err))
            })?;

        Ok(())
    }
}
