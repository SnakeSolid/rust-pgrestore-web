mod error;

pub use self::error::PathManagerError;
pub use self::error::PathManagerResult;

use std::collections::HashMap;
use std::collections::HashSet;
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
        self.with_read(move |pathmanager| {
            pathmanager.query_paths(query, n, callback);

            Ok(())
        })
    }

    pub fn retain<F>(&self, callback: F) -> PathManagerResult<()>
    where
        F: Fn(&PathBuf) -> bool,
    {
        self.with_write(move |pathmanager| {
            pathmanager.retain(callback);

            Ok(())
        })
    }

    pub fn add_path(&self, path: &Path) -> PathManagerResult<()> {
        self.with_write(move |pathmanager| {
            pathmanager.add_path(path);

            Ok(())
        })
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
            .split(char::is_whitespace)
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

        let mut results: Vec<_> = query_results.into_iter().map(|(id, w)| (w, id)).collect();

        results.sort_by(|a, b| b.cmp(a));
        results
            .into_iter()
            .take(n)
            .filter_map(|(_, id)| self.paths.get(&id))
            .for_each(|p| callback(p));
    }

    pub fn retain<F>(&mut self, callback: F)
    where
        F: Fn(&PathBuf) -> bool,
    {
        let mut remove_ids = HashSet::new();

        for (id, _) in self.paths.iter().filter(|(_, path)| !callback(path)) {
            remove_ids.insert(id.clone());
        }

        for ids in self.index.values_mut() {
            ids.retain(|id| !remove_ids.contains(id));
        }

        self.paths.retain(|id, _| !remove_ids.contains(id));
        self.index.retain(|_, ids| !ids.is_empty());
    }

    pub fn add_path(&mut self, path: &Path) {
        let index = self.next_index;
        let path = path.to_path_buf();

        for component in path.iter().skip(1) {
            let name = component.to_string_lossy().to_lowercase();

            if let Some(ids) = self.index.get(&name) {
                if ids
                    .iter()
                    .filter_map(|id| self.paths.get(id))
                    .any(|p| p == &path)
                {
                    return;
                }
            }

            self.index
                .entry(name)
                .or_insert_with(|| Vec::with_capacity(4))
                .push(index);
        }

        self.paths.insert(index, path);
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

        assert_eq!(vec![path_2, path_3], result);
    }

    #[test]
    fn query_shold_add_path_only_once() {
        let path_1: PathBuf = "/test/dir/file.backup".into();
        let path_2: PathBuf = "/test/other/file.backup".into();
        let path_3: PathBuf = "/test/other.backup".into();
        let mut manager = PathManager::new();
        let mut result = Vec::new();

        manager.add_path(&path_1);
        manager.add_path(&path_2);
        manager.add_path(&path_2);
        manager.add_path(&path_3);
        manager.query_paths("file other", 2, |p| result.push(p.to_path_buf()));

        assert_eq!(vec![path_2, path_3], result);
    }

    #[test]
    fn clear_shold_retain_path() {
        let path_1: PathBuf = "/test/dir/file.backup".into();
        let path_2: PathBuf = "/test/other/file.backup".into();
        let path_3: PathBuf = "/test/different/file.backup".into();
        let mut manager = PathManager::new();
        let mut result = Vec::new();

        manager.add_path(&path_1);
        manager.add_path(&path_2);
        manager.add_path(&path_3);
        manager.retain(|path| path != &path_2);
        manager.query_paths("diff file", 3, |p| result.push(p.to_path_buf()));

        assert_eq!(vec![path_3, path_1], result);
    }
}
