mod command;
mod error;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;

use self::command::WorkerCommand;
use self::command::WorkerSettings;

use config::ConfigRef;
use config::Destination;
use http::HttpClientRef;
use http::HttpClientResult;
use http::PathHandle;
use jobmanager::JobManagerRef;
use std::collections::HashSet;
use std::path::Path;
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

    pub fn restore_file_schema(
        self,
        jobid: usize,
        backup_path: &Path,
        schema: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let backup_path = backup_path.to_path_buf();
        let schema = schema.to_owned();

        self.do_async(jobid, move |worker| {
            worker.execute_backup_schema(
                jobid,
                backup_path.as_ref(),
                &schema,
                drop_database,
                create_database,
            )
        })
    }

    pub fn restore_file_tables(
        self,
        jobid: usize,
        backup_path: &Path,
        tables: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let backup_path = backup_path.to_path_buf();
        let tables = tables.to_owned();

        self.do_async(jobid, move |worker| {
            worker.execute_backup_tables(
                jobid,
                backup_path.as_ref(),
                &tables,
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

    pub fn restore_url_schema(
        self,
        jobid: usize,
        url: &str,
        http_client: HttpClientRef,
        schema: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let url = url.to_string();
        let schema = schema.to_owned();

        self.do_async(jobid, move |worker| {
            let backup_path = worker.execute_download(jobid, || http_client.download(&url))?;

            worker.execute_backup_schema(
                jobid,
                backup_path.as_ref(),
                &schema,
                drop_database,
                create_database,
            )
        })
    }

    pub fn restore_url_tables(
        self,
        jobid: usize,
        url: &str,
        http_client: HttpClientRef,
        tables: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let url = url.to_string();
        let tables = tables.to_owned();

        self.do_async(jobid, move |worker| {
            let backup_path = worker.execute_download(jobid, || http_client.download(&url))?;

            worker.execute_backup_tables(
                jobid,
                backup_path.as_ref(),
                &tables,
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

        if drop_database {
            self.execute_step(jobid, || command.drop_database())?;
        }

        if create_database {
            self.execute_step(jobid, || command.create_database())?;
        }

        self.execute_step_soft(jobid, || command.restore_backup(backup_path.as_ref()))?;
        self.set_complete(jobid, true)
    }

    fn execute_backup_schema(
        self,
        jobid: usize,
        backup_path: &Path,
        schema: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let command = WorkerCommand::new(jobid, &self);

        if drop_database {
            self.execute_step(jobid, || command.drop_database())?;
        }

        if create_database {
            self.execute_step(jobid, || command.create_database())?;
        }

        for name in schema {
            self.execute_step(jobid, || command.create_schema(name))?;
            self.execute_step_soft(jobid, || command.restore_schema(name, &backup_path))?;
        }

        self.set_complete(jobid, true)
    }

    fn execute_backup_tables(
        self,
        jobid: usize,
        backup_path: &Path,
        tables: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let mut command = WorkerCommand::new(jobid, &self);

        if drop_database {
            self.execute_step(jobid, || command.drop_database())?;
        }

        if create_database {
            self.execute_step(jobid, || command.create_database())?;

            for name in &self.collect_schema_names(&tables) {
                self.execute_step(jobid, || command.create_schema(name))?;
                self.execute_step_soft(jobid, || command.restore_schema_only(name, &backup_path))?;
            }
        }

        for (schema, table) in &self.split_table_names(&tables) {
            self.execute_step(jobid, || command.truncate_table(schema, table))?;
            self.execute_step_soft(jobid, || command.restore_table(schema, table, &backup_path))?;
        }

        self.set_complete(jobid, true)
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

    fn execute_download<F>(&self, jobid: usize, callback: F) -> WorkerResult<PathHandle>
    where
        F: FnOnce() -> HttpClientResult<PathHandle>,
    {
        match callback() {
            Ok(path) => Ok(path),
            Err(err) => {
                self.job_manager()
                    .extend_stderr(jobid, format!("{}", err).as_bytes())
                    .map_err(WorkerError::extend_stdout_error)?;
                self.set_complete(jobid, false)?;

                Err(WorkerError::download_error(err))
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
        self.job_manager
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

    fn psql_path(&self) -> &str {
        self.config.commands().psql_path()
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
