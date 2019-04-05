mod error;

pub use self::error::DatabaseError;
pub use self::error::DatabaseResult;

use super::TableDescription;
use postgres::params::ConnectParams;
use postgres::params::Host;
use postgres::Connection;
use postgres::TlsMode;
use std::collections::HashSet;

#[derive(Debug)]
pub struct PostgreSQL {
    server: String,
    port: u16,
    user: String,
    password: String,
    database: String,
}

impl PostgreSQL {
    pub fn new(server: &str, port: u16, user: &str, password: &str, database: &str) -> PostgreSQL {
        PostgreSQL {
            server: server.into(),
            port,
            user: user.into(),
            password: password.into(),
            database: database.into(),
        }
    }

    pub fn drop_schemas(&self, schemas: &HashSet<String>) -> DatabaseResult<()> {
        let connection = self.connect()?;

        for schema in schemas {
            debug!("Drop schema {}", schema);

            // Escape double quotes from schema name
            let schema = schema.replace('"', "\"\'");

            let _rows_updated = connection
                .execute(
                    &format!("drop schema if exists \"{}\" cascade", schema),
                    &[],
                )
                .map_err(DatabaseError::query_execution_error)?;
        }

        Ok(())
    }

    pub fn create_schemas(&self, schemas: &HashSet<String>) -> DatabaseResult<()> {
        let connection = self.connect()?;

        for schema in schemas {
            debug!("Create schema: {}", schema);

            // Escape double quotes from schema name
            let schema = schema.replace('"', "\"\'");

            let _rows_updated = connection
                .execute(&format!("create schema if not exists \"{}\"", schema), &[])
                .map_err(DatabaseError::query_execution_error)?;
        }

        Ok(())
    }

    pub fn drop_tables(&self, tables: &HashSet<TableDescription>) -> DatabaseResult<()> {
        let connection = self.connect()?;

        for table in tables {
            let schema_name = table.schema();
            let table_name = table.name();

            debug!("Create table: {}.{}", schema_name, table_name);

            // Escape double quotes from schema and table names
            let schema_name = schema_name.replace('"', "\"\'");
            let table_name = table_name.replace('"', "\"\'");

            let _rows_updated = connection
                .execute(
                    &format!(
                        "drop table if exists \"{}\".\"{}\"",
                        schema_name, table_name
                    ),
                    &[],
                )
                .map_err(DatabaseError::query_execution_error)?;
        }

        Ok(())
    }

    fn connect(&self) -> DatabaseResult<Connection> {
        let password = Some(self.password.as_str()).filter(|w| !w.is_empty());
        let params = ConnectParams::builder()
            .port(self.port)
            .user(&self.user, password)
            .database(&self.database)
            .build(Host::Tcp(self.server.clone()));

        Connection::connect(params, TlsMode::None).map_err(DatabaseError::connection_error)
    }
}
