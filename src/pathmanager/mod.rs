mod error;

pub use self::error::PathManagerError;
pub use self::error::PathManagerResult;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct PathManagerRef {
    inner: Arc<RwLock<PathManager>>,
}

impl PathManagerRef {
    fn with_read<F, T>(&self, callback: F) -> PathManagerResult<T>
    where
        F: FnOnce(&PathManager) -> PathManagerResult<T>,
    {
        match self.inner.write() {
            Ok(ref pathmanager) => callback(pathmanager),
            Err(err) => {
                warn!("Failed to acquire write lock - {}", err);

                Err(PathManagerError::new("Failed to acquire write lock"))
            }
        }
    }

    fn with_write<F, T>(&self, callback: F) -> PathManagerResult<T>
    where
        F: FnOnce(&mut PathManager) -> PathManagerResult<T>,
    {
        match self.inner.write() {
            Ok(ref mut pathmanager) => callback(pathmanager),
            Err(err) => {
                warn!("Failed to acquire write lock - {}", err);

                Err(PathManagerError::new("Failed to acquire write lock"))
            }
        }
    }

    pub fn query_paths<F>(&self, query: &str, n: usize, callback: F) -> PathManagerResult<()>
    where
        F: FnMut(&Path),
    {
        self.with_read(move |pathmanager| Ok(pathmanager.query_paths(query, n, callback)))
    }

    pub fn clear(&self) -> PathManagerResult<()> {
        self.with_write(move |pathmanager| Ok(pathmanager.clear()))
    }

    pub fn add_path(&self, path: &Path) -> PathManagerResult<()> {
        self.with_write(move |pathmanager| Ok(pathmanager.add_path(path)))
    }
}

#[derive(Debug)]
struct PathManager {
    paths: HashMap<usize, PathBuf>,
    index: HashMap<String, Vec<usize>>,
    next_index: usize,
}

impl PathManager {
    fn new() -> PathManager {
        PathManager {
            paths: HashMap::new(),
            index: HashMap::new(),
            next_index: 0,
        }
    }

    pub fn query_paths<F>(&self, query: &str, n: usize, mut callback: F)
    where
        F: FnMut(&Path),
    {
        let mut query_results: HashMap<usize, u64> = HashMap::new();

        for word in query
            .to_lowercase()
            .split(|c| " _\\/".contains(c))
            .filter(|w| !w.is_empty())
        {
            let weight = word.len();

            for (_, ids) in self.index.iter().filter(|(k, _)| k.contains(word)) {
                for &id in ids {
                    let w = query_results.entry(id).or_insert(0);

                    *w += weight as u64;
                }
            }
        }

        let mut results: Vec<_> = query_results.into_iter().collect();

        results.sort_by(|a, b| b.cmp(a));
        results
            .into_iter()
            .take(n)
            .filter_map(|(id, _)| self.paths.get(&id))
            .for_each(|p| callback(p));
    }

    pub fn clear(&mut self) {
        self.paths.clear();
        self.index.clear();
    }

    pub fn add_path(&mut self, path: &Path) {
        let index = self.next_index;
        let path = path.to_path_buf();

        self.paths.insert(index, path.clone());

        for component in &path {
            let name = component.to_string_lossy().to_lowercase();

            self.index
                .entry(name)
                .or_insert_with(|| Vec::with_capacity(4))
                .push(index);
        }

        self.next_index += 1;
    }
}

pub fn create() -> PathManagerRef {
    PathManagerRef {
        inner: Arc::new(RwLock::new(PathManager::new())),
    }
}

#[cfg(test)]
mod tests {
    use super::PathManager;

    use std::path::PathBuf;

    #[test]
    fn query_shold_return_matched() {
        let path: PathBuf = "/test/dir/file.backup".into();
        let mut manager = PathManager::new();
        let mut result = Vec::new();

        manager.add_path(&path);
        manager.query_paths("dir", 2, |p| result.push(p.to_path_buf()));

        assert_eq!(vec![path], result);
    }

    #[test]
    fn query_shold_return_only_matched() {
        let path_1: PathBuf = "/test/dir/file.backup".into();
        let path_2: PathBuf = "/test/file.backup".into();
        let mut manager = PathManager::new();
        let mut result = Vec::new();

        manager.add_path(&path_1);
        manager.add_path(&path_2);
        manager.query_paths("dir", 2, |p| result.push(p.to_path_buf()));

        assert_eq!(vec![path_1], result);
    }

    #[test]
    fn query_shold_return_relevant_paths() {
        let path_1: PathBuf = "/test/dir/file.backup".into();
        let path_2: PathBuf = "/test/other/file.backup".into();
        let mut manager = PathManager::new();
        let mut result = Vec::new();

        manager.add_path(&path_1);
        manager.add_path(&path_2);
        manager.query_paths("file other", 2, |p| result.push(p.to_path_buf()));

        assert_eq!(vec![path_2, path_1], result);
    }

    #[test]
    fn query_shold_return_empty() {
        let manager = PathManager::new();
        let mut result = Vec::new();

        manager.query_paths("file other", 2, |p| result.push(p.to_path_buf()));

        assert_eq!(Vec::<PathBuf>::new(), result);
    }

    #[test]
    fn query_shold_return_n_results() {
        let path_1: PathBuf = "/test/dir/file.backup".into();
        let path_2: PathBuf = "/test/other/file.backup".into();
        let path_3: PathBuf = "/test/other.backup".into();
        let mut manager = PathManager::new();
        let mut result = Vec::new();

        manager.add_path(&path_1);
        manager.add_path(&path_2);
        manager.add_path(&path_3);
        manager.query_paths("file other", 2, |p| result.push(p.to_path_buf()));

        assert_eq!(vec![path_3, path_2], result);
    }
}
