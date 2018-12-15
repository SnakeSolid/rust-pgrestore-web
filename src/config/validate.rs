use super::ConfigError;
use super::ConfigRef;
use super::ConfigResult;

use std::path::PathBuf;

pub fn validate(config: ConfigRef) -> ConfigResult<()> {
    validate_number(config.max_jobs(), "jobs")?;
    validate_dir(config.joblogs_path().into(), "Jobs log")?;
    validate_number(config.restore_jobs(), "restore jobs")?;
    validate_dir(
        config.http_config().download_directory().into(),
        "HTTP downloads",
    )?;
    validate_file(config.commands().createdb_path().into(), "createdb")?;
    validate_file(config.commands().dropdb_path().into(), "dropdb")?;
    validate_file(config.commands().pgrestore_path().into(), "pgrestore")?;

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

fn validate_dir(path: PathBuf, name: &str) -> ConfigResult<()> {
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

fn validate_file(path: PathBuf, name: &str) -> ConfigResult<()> {
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
