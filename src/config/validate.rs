use super::ConfigError;
use super::ConfigRef;
use super::ConfigResult;

use std::path::Path;

#[allow(clippy::needless_pass_by_value)]
pub fn validate(config: ConfigRef) -> ConfigResult<()> {
    validate_number(config.max_jobs(), "jobs")?;
    validate_dir(config.joblogs_path(), "Jobs log")?;
    validate_number(config.restore_jobs(), "restore jobs")?;
    validate_dir(config.http_client().download_directory(), "HTTP downloads")?;
    validate_file(config.commands().createdb_path(), "createdb")?;
    validate_file(config.commands().dropdb_path(), "dropdb")?;
    validate_file(config.commands().pgrestore_path(), "pgrestore")?;

    if let Some(indexes_path) = config.indexes_path() {
        validate_file(indexes_path, "indexes_path")?;
    }

    Ok(())
}

fn validate_number(value: usize, name: &str) -> ConfigResult<()> {
    if value > 0 {
        Ok(())
    } else {
        Err(ConfigError::format(format_args!(
            "Numbers of {} must be greater than zero, but {} given",
            name, value
        )))
    }
}

fn validate_dir<P>(path: P, name: &str) -> ConfigResult<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.exists() {
        Err(ConfigError::format(format_args!(
            "File {} ({}) is not exists",
            name,
            path.display(),
        )))
    } else if !path.is_dir() {
        Err(ConfigError::format(format_args!(
            "File {} ({}) is not a directory",
            name,
            path.display()
        )))
    } else {
        Ok(())
    }
}

fn validate_file<P>(path: P, name: &str) -> ConfigResult<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.exists() {
        Err(ConfigError::format(format_args!(
            "{} directory ({}) is not exists",
            name,
            path.display(),
        )))
    } else if !path.is_file() {
        Err(ConfigError::format(format_args!(
            "{} directory ({}) is not a file",
            name,
            path.display()
        )))
    } else {
        Ok(())
    }
}
