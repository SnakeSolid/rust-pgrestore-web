mod command;
mod entity;
mod error;
mod index;
mod postgres;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;
pub use self::postgres::DatabaseError;
pub use self::postgres::PostgreSQL;

use self::command::CommandStatus;
use self::command::WorkerCommand;
use self::command::WorkerSettings;
use self::entity::EntityList;
use self::entity::IndexDescription;
use self::entity::TableDescription;
use crate::config::ConfigRef;
use crate::config::Destination;
use crate::http::HttpClientRef;
use crate::http::HttpClientResult;
use crate::http::PathHandle;
use crate::jobmanager::JobManagerRef;
use std::collections::HashSet;
use std::fmt::Arguments;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::thread::Builder;

#[derive(Debug)]
pub struct Worker {
    config: ConfigRef,
    job_manager: JobManagerRef,
    destination: Destination,
    database_name: String,
    ignore_errors: bool,
}

impl Worker {
    pub fn new(
        config: ConfigRef,
        job_manager: JobManagerRef,
        destination: &Destination,
        database_name: &str,
        ignore_errors: bool,
    ) -> Worker {
        Worker {
            config,
            job_manager,
            destination: destination.clone(),
            database_name: database_name.into(),
            ignore_errors,
        }
    }

    pub fn restore_file_full(
        self,
        jobid: usize,
        backup_path: &Path,
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let backup_path = backup_path.to_path_buf();

        self.do_async(jobid, move |worker| {
            worker.execute_backup_full(jobid, &backup_path, drop_database, create_database)
        })
    }

    pub fn restore_file_partial(
        self,
        jobid: usize,
        backup_path: &Path,
        objects: &[String],
        restore_schema: bool,
        restore_indexes: bool,
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let backup_path = backup_path.to_path_buf();
        let objects = objects.to_owned();

        self.do_async(jobid, move |worker| {
            worker.execute_backup_partial(
                jobid,
                backup_path.as_ref(),
                &objects,
                restore_schema,
                restore_indexes,
                drop_database,
                create_database,
            )
        })
    }

    pub fn restore_url_full(
        self,
        jobid: usize,
        url: &str,
        http_client: HttpClientRef,
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let url = url.to_string();

        self.do_async(jobid, move |worker| {
            let backup_path = worker.execute_download(jobid, || http_client.download(&url))?;

            worker.execute_backup_full(jobid, backup_path.as_ref(), drop_database, create_database)
        })
    }

    pub fn restore_url_partial(
        self,
        jobid: usize,
        url: &str,
        http_client: HttpClientRef,
        objects: &[String],
        restore_schema: bool,
        restore_indexes: bool,
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let url = url.to_string();
        let objects = objects.to_owned();

        self.do_async(jobid, move |worker| {
            let backup_path = worker.execute_download(jobid, || http_client.download(&url))?;

            worker.execute_backup_partial(
                jobid,
                backup_path.as_ref(),
                &objects,
                restore_schema,
                restore_indexes,
                drop_database,
                create_database,
            )
        })
    }

    fn execute_backup_full(
        self,
        jobid: usize,
        backup_path: &Path,
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let command = WorkerCommand::new(jobid, &self);

        self.check_backup_path(jobid, &backup_path)?;

        if drop_database {
            self.execute_step(jobid, || command.drop_database())?;
        }

        if create_database {
            let template = self.config.templates().full();

            self.execute_step(jobid, || command.create_database(template))?;
        }

        self.execute_step_soft(jobid, || {
            command.restore_backup(backup_path, !create_database)
        })?;

        self.set_complete(jobid, true)
    }

    fn execute_backup_partial(
        self,
        jobid: usize,
        backup_path: &Path,
        objects: &[String],
        restore_schema: bool,
        restore_indexes: bool,
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let command = WorkerCommand::new(jobid, &self);

        self.check_backup_path(jobid, &backup_path)?;

        if drop_database {
            self.execute_step(jobid, || command.drop_database())?;
        }

        let entities = EntityList::parse(objects);
        let full_schemas = entities.full_schemas();
        let table_schemas = entities.table_schemas();
        let tables = entities.tables();

        if create_database {
            let template = self.config.templates().partial();

            self.execute_step(jobid, || command.create_database(template))?;
        } else {
            self.execute_step(jobid, || self.cleanup_schemas(jobid, full_schemas))?;
        }

        // Restore schema's related to required tables before restoring table.
        if restore_schema {
            for name in table_schemas {
                self.execute_step_soft(jobid, || command.restore_schema_only(name, &backup_path))?;
            }
        } else {
            self.execute_step(jobid, || self.create_schemas(jobid, table_schemas))?;
        }

        // Create empty schema's in database.
        self.execute_step(jobid, || self.create_schemas(jobid, full_schemas))?;

        // Restore
        for name in full_schemas {
            self.execute_step(jobid, || command.restore_schema_data(name, &backup_path))?;
        }

        // Drop required table to make sure that restored columns will be the same as in backup.
        self.cleanup_tables(jobid, tables)?;

        for table in tables {
            self.execute_step(jobid, || {
                command.restore_table(table.schema(), table.name(), &backup_path)
            })?;
        }

        // Restore indexes for all table.
        if restore_indexes {
            if let Some(indexes_path) = self.config.indexes_path() {
                let indexes = index::read_indexes(indexes_path, &tables)?;

                for indexe in indexes {
                    self.execute_step(jobid, || {
                        command.restore_index(indexe.schema(), indexe.name(), &backup_path)
                    })?;
                }
            } else {
                return Err(WorkerError::new(
                    "Indexes path not defined in configuration.",
                ));
            }
        }

        self.set_complete(jobid, true)
    }

    fn create_schemas(
        &self,
        jobid: usize,
        schemas: &HashSet<String>,
    ) -> WorkerResult<CommandStatus> {
        let postgres = PostgreSQL::new(
            self.destination.host(),
            self.destination.port(),
            self.destination.role(),
            self.destination.password(),
            &self.database_name,
        );

        self.job_manager
            .set_stage(jobid, "Creating schema's")
            .map_err(WorkerError::set_stage_error)?;

        postgres
            .create_schemas(schemas)
            .map_err(WorkerError::query_execution_error)?;

        Ok(CommandStatus::Success)
    }

    fn cleanup_schemas(
        &self,
        jobid: usize,
        schemas: &HashSet<String>,
    ) -> WorkerResult<CommandStatus> {
        let postgres = PostgreSQL::new(
            self.destination.host(),
            self.destination.port(),
            self.destination.role(),
            self.destination.password(),
            &self.database_name,
        );

        self.job_manager
            .set_stage(jobid, "Cleaning schema's")
            .map_err(WorkerError::set_stage_error)?;

        postgres
            .drop_schemas(schemas)
            .map_err(WorkerError::query_execution_error)?;

        Ok(CommandStatus::Success)
    }

    fn cleanup_tables(
        &self,
        jobid: usize,
        tables: &HashSet<TableDescription>,
    ) -> WorkerResult<CommandStatus> {
        let postgres = PostgreSQL::new(
            self.destination.host(),
            self.destination.port(),
            self.destination.role(),
            self.destination.password(),
            &self.database_name,
        );

        self.job_manager
            .set_stage(jobid, "Cleaning tables")
            .map_err(WorkerError::set_stage_error)?;

        postgres
            .drop_tables(tables)
            .map_err(WorkerError::query_execution_error)?;

        Ok(CommandStatus::Success)
    }

    fn write_error(&self, jobid: usize, args: Arguments) -> WorkerResult<()> {
        let stderr_path: PathBuf = self
            .job_manager
            .map_job(jobid, |job| job.stderr_path().into())
            .map_err(WorkerError::map_job_error)?
            .ok_or_else(|| WorkerError::new("Job not found"))?;
        let mut stdout = OpenOptions::new()
            .create(true)
            .append(true)
            .open(stderr_path)
            .map_err(WorkerError::io_error)?;

        stdout
            .write_fmt(format_args!("{}\n", args))
            .map_err(WorkerError::io_error)?;

        Ok(())
    }

    fn check_backup_path(&self, jobid: usize, path: &Path) -> WorkerResult<()> {
        if !path.exists() {
            self.write_error(
                jobid,
                format_args!("Path {} does not exists", path.display()),
            )?;
            self.set_complete(jobid, false)?;

            return Err(WorkerError::new("Path does not exists"));
        }

        if !path.is_file() {
            self.write_error(jobid, format_args!("Path {} is not a file", path.display()))?;
            self.set_complete(jobid, false)?;

            return Err(WorkerError::new("Path is not a file"));
        }

        Ok(())
    }

    fn execute_step<F>(&self, jobid: usize, callback: F) -> WorkerResult<()>
    where
        F: FnOnce() -> WorkerResult<CommandStatus>,
    {
        match callback() {
            Ok(CommandStatus::Success) => Ok(()),
            Ok(CommandStatus::Aborted) => {
                self.write_error(jobid, format_args!("Job aborted"))?;
                self.set_aborted(jobid)?;

                Err(WorkerError::new("Job aborted"))
            }
            Ok(CommandStatus::Failed) => {
                self.set_complete(jobid, false)?;

                Err(WorkerError::new("Job failed"))
            }
            Err(err) => {
                self.set_complete(jobid, false)?;

                Err(err)
            }
        }
    }

    fn execute_download<F>(&self, jobid: usize, callback: F) -> WorkerResult<PathHandle>
    where
        F: FnOnce() -> HttpClientResult<PathHandle>,
    {
        self.job_manager
            .set_stage(jobid, "Download file")
            .map_err(WorkerError::set_stage_error)?;

        match callback() {
            Ok(path) => Ok(path),
            Err(err) => {
                self.write_error(jobid, format_args!("{}", err))?;
                self.set_complete(jobid, false)?;

                Err(WorkerError::download_error(err))
            }
        }
    }

    fn execute_step_soft<F>(&self, jobid: usize, callback: F) -> WorkerResult<()>
    where
        F: FnOnce() -> WorkerResult<CommandStatus>,
    {
        if self.ignore_errors {
            match callback() {
                Ok(CommandStatus::Success) | Ok(CommandStatus::Failed) | Err(_) => Ok(()),
                Ok(CommandStatus::Aborted) => {
                    self.write_error(jobid, format_args!("Job aborted"))?;
                    self.set_aborted(jobid)?;

                    Err(WorkerError::new("Job aborted"))
                }
            }
        } else {
            self.execute_step(jobid, callback)
        }
    }

    fn set_aborted(&self, jobid: usize) -> WorkerResult<()> {
        self.job_manager
            .set_aborted(jobid)
            .map_err(WorkerError::set_status_error)
    }

    fn set_complete(&self, jobid: usize, complete: bool) -> WorkerResult<()> {
        self.job_manager
            .set_complete(jobid, complete)
            .map_err(WorkerError::set_status_error)
    }

    fn do_async<F>(self, jobid: usize, callback: F) -> WorkerResult<()>
    where
        F: FnOnce(Worker) -> WorkerResult<()> + Send,
        F: Send + 'static,
    {
        let _ = Builder::new()
            .name(format!("restore worker #{}", jobid))
            .spawn(move || callback(self))
            .map_err(WorkerError::spawn_thread_error)?;

        Ok(())
    }
}

impl WorkerSettings for Worker {
    fn createdb_path(&self) -> &str {
        self.config.commands().createdb_path()
    }

    fn dropdb_path(&self) -> &str {
        self.config.commands().dropdb_path()
    }

    fn pgrestore_path(&self) -> &str {
        self.config.commands().pgrestore_path()
    }

    fn restore_jobs(&self) -> usize {
        self.config.restore_jobs()
    }

    fn job_manager(&self) -> &JobManagerRef {
        &self.job_manager
    }

    fn host(&self) -> &str {
        self.destination.host()
    }

    fn port(&self) -> u16 {
        self.destination.port()
    }

    fn role(&self) -> &str {
        self.destination.role()
    }

    fn password(&self) -> &str {
        self.destination.password()
    }

    fn database_name(&self) -> &str {
        &self.database_name
    }

    fn ignore_errors(&self) -> bool {
        self.ignore_errors
    }
}
