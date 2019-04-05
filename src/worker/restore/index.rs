use super::IndexDescription;
use super::TableDescription;
use super::WorkerError;
use super::WorkerResult;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

pub fn read_indexes<P>(
    path: P,
    tables: &HashSet<TableDescription>,
) -> WorkerResult<HashSet<IndexDescription>>
where
    P: AsRef<Path>,
{
    let file = File::open(path).map_err(WorkerError::io_error)?;
    let reader = BufReader::new(file);
    let mut result = HashSet::new();

    for line in reader.lines() {
        let line = line.map_err(WorkerError::io_error)?;
        let parts: Vec<_> = line.splitn(3, ',').collect();

        if parts.len() < 3 {
            return Err(WorkerError::new(&format!(
                "Incorrect index row `{}`, expected 3 values separated by comma.",
                line
            )));
        }

        let table = TableDescription::new(parts[0], parts[1]);

        if tables.contains(&table) {
            let index = IndexDescription::new(parts[0], parts[2]);

            result.insert(index);
        }
    }

    Ok(result)
}
