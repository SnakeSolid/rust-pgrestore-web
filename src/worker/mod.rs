mod error;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;

use config::ConfigRef;
use config::Destination;
use jobmanager::JobManagerRef;
use std::collections::HashSet;
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
    ignore_errors: bool,
}

impl Worker {
    pub fn new(
        config: ConfigRef,
        jobmanager: JobManagerRef,
        destination: &Destination,
        backup_path: &Path,
        database_name: &str,
        ignore_errors: bool,
    ) -> Worker {
        Worker {
            config,
            jobmanager,
            destination: destination.clone(),
            backup_path: backup_path.into(),
            database_name: database_name.into(),
            ignore_errors,
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
            .stdout(Stdio::null())
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

    fn do_create_database(&self, jobid: usize) -> WorkerResult<()> {
        info!("Creating database {}", self.database_name);

        self.jobmanager
            .set_stage(jobid, "Create database")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().createdb_path());

        command
            .env_clear()
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

    fn do_create_schema(&self, jobid: usize, name: &str) -> WorkerResult<()> {
        info!(
            "Creating schema {} in database {}",
            name, self.database_name
        );

        self.jobmanager
            .set_stage(jobid, "Create schema")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().psql_path());

        command
            .env("PGPASSWORD", self.destination.password())
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg("--command")
            .arg(format!("CREATE SCHEMA IF NOT EXISTS {}", name))
            .arg(&self.database_name);

        self.wait_command(jobid, command)
    }

    fn do_drop_database(&self, jobid: usize) -> WorkerResult<()> {
        info!("Dropping database {}", self.database_name);

        self.jobmanager
            .set_stage(jobid, "Drop database")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().dropdb_path());

        command
            .env_clear()
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

    fn do_restore_backup(&self, jobid: usize) -> WorkerResult<()> {
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
            .env_clear()
            .env("PGPASSWORD", self.destination.password())
            .arg("--verbose")
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg("--dbname")
            .arg(&self.database_name)
            .arg("--clean")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--jobs")
            .arg(format!("{}", self.config.restore_jobs()))
            .arg(&self.backup_path);

        self.wait_command(jobid, command)
    }

    fn do_restore_schema(&self, jobid: usize, name: &str) -> WorkerResult<()> {
        info!(
            "Restoring schema {} to {} from {}",
            name,
            self.database_name,
            self.backup_path.display(),
        );

        self.jobmanager
            .set_stage(jobid, &format!("Restore schema {}", name))
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().pgrestore_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.destination.password())
            .arg("--verbose")
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg("--dbname")
            .arg(&self.database_name)
            .arg("--schema")
            .arg(name)
            .arg("--clean")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--jobs")
            .arg(format!("{}", self.config.restore_jobs()))
            .arg(&self.backup_path);

        self.wait_command(jobid, command)
    }

    fn do_restore_schema_only(&self, jobid: usize, name: &str) -> WorkerResult<()> {
        info!(
            "Restoring schema only {} to {} from {}",
            name,
            self.database_name,
            self.backup_path.display(),
        );

        self.jobmanager
            .set_stage(jobid, &format!("Restore schema {}", name))
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().pgrestore_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.destination.password())
            .arg("--verbose")
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg("--dbname")
            .arg(&self.database_name)
            .arg("--schema")
            .arg(name)
            .arg("--schema-only")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--jobs")
            .arg(format!("{}", self.config.restore_jobs()))
            .arg(&self.backup_path);

        self.wait_command(jobid, command)
    }

    fn do_truncate_table(&self, jobid: usize, schema: &str, table: &str) -> WorkerResult<()> {
        info!(
            "Truncate table {}.{} in database {}",
            schema, table, self.database_name
        );

        self.jobmanager
            .set_stage(jobid, "Truncate table")
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().psql_path());

        command
            .env("PGPASSWORD", self.destination.password())
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg("--command")
            .arg(format!("TRUNCATE {}.{}", schema, table))
            .arg(&self.database_name);

        self.wait_command(jobid, command)
    }

    fn do_restore_table(&self, jobid: usize, schema: &str, table: &str) -> WorkerResult<()> {
        info!(
            "Restoring table {}.{} to {} from {}",
            schema,
            table,
            self.database_name,
            self.backup_path.display(),
        );

        self.jobmanager
            .set_stage(jobid, &format!("Restore table {}.{}", schema, table))
            .map_err(WorkerError::set_stage_error)?;

        let mut command = Command::new(self.config.commands().pgrestore_path());

        command
            .env_clear()
            .env("PGPASSWORD", self.destination.password())
            .arg("--verbose")
            .arg("--host")
            .arg(self.destination.host())
            .arg("--port")
            .arg(format!("{}", self.destination.port()))
            .arg("--username")
            .arg(self.destination.role())
            .arg("--dbname")
            .arg(&self.database_name)
            .arg("--schema")
            .arg(schema)
            .arg("--table")
            .arg(table)
            .arg("--data-only")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg(&self.backup_path);

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

    fn execute_step_soft<F>(&self, jobid: usize, callback: F) -> WorkerResult<()>
    where
        F: FnOnce() -> WorkerResult<()>,
    {
        match callback() {
            Ok(()) => Ok(()),
            Err(_) if self.ignore_errors => Ok(()),
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

    fn collect_schema_names(&self, tables: &[String]) -> HashSet<String> {
        let mut result = HashSet::new();

        for table in tables {
            if let Some(index) = table.find('.') {
                result.insert(table[..index].into());
            }
        }

        result
    }

    fn split_table_names(&self, tables: &[String]) -> Vec<(String, String)> {
        let mut result = Vec::new();

        for table in tables {
            if let Some(index) = table.find('.') {
                result.push((table[..index].into(), table[index + 1..].into()));
            }
        }

        result
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
                    self.execute_step(jobid, || self.do_drop_database(jobid))?;
                }

                if create_database {
                    self.execute_step(jobid, || self.do_create_database(jobid))?;
                }

                self.execute_step_soft(jobid, || self.do_restore_backup(jobid))?;
                self.set_complete(jobid, true)
            })
            .map_err(WorkerError::spawn_thread_error)?;

        Ok(())
    }

    pub fn restore_schema(
        self,
        jobid: usize,
        schema: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let schema = schema.to_owned();
        let _ = Builder::new()
            .name(format!("worker #{}", jobid))
            .spawn(move || {
                if drop_database {
                    self.execute_step(jobid, || self.do_drop_database(jobid))?;
                }

                if create_database {
                    self.execute_step(jobid, || self.do_create_database(jobid))?;
                }

                for name in &schema {
                    self.execute_step(jobid, || self.do_create_schema(jobid, name))?;
                    self.execute_step_soft(jobid, || self.do_restore_schema(jobid, name))?;
                }

                self.set_complete(jobid, true)
            })
            .map_err(WorkerError::spawn_thread_error)?;

        Ok(())
    }

    pub fn restore_tables(
        self,
        jobid: usize,
        tables: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let tables = tables.to_owned();
        let _ = Builder::new()
            .name(format!("worker #{}", jobid))
            .spawn(move || {
                if drop_database {
                    self.execute_step(jobid, || self.do_drop_database(jobid))?;
                }

                if create_database {
                    self.execute_step(jobid, || self.do_create_database(jobid))?;
                }

                for name in &self.collect_schema_names(&tables) {
                    self.execute_step(jobid, || self.do_create_schema(jobid, name))?;
                    self.execute_step_soft(jobid, || self.do_restore_schema_only(jobid, name))?;
                }

                for (schema, table) in &self.split_table_names(&tables) {
                    self.execute_step(jobid, || self.do_truncate_table(jobid, schema, table))?;
                    self.execute_step_soft(jobid, || self.do_restore_table(jobid, schema, table))?;
                }

                self.set_complete(jobid, true)
            })
            .map_err(WorkerError::spawn_thread_error)?;

        Ok(())
    }
}
