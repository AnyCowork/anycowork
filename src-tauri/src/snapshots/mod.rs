use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub timestamp: i64,
    pub file_hashes: HashMap<String, String>, // relative_path -> sha256_hash
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub new_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub deleted_files: Vec<String>,
}

pub struct SnapshotManager {
    root_path: PathBuf,
}

impl SnapshotManager {
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    pub fn create_snapshot(&self) -> Result<Snapshot, String> {
        let mut file_hashes = HashMap::new();

        let walker = WalkDir::new(&self.root_path).into_iter().filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // Skip hidden files/dirs and common large directories
            !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
                && name != "dist"
                && name != "out"
        });

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let relative_path = entry
                    .path()
                    .strip_prefix(&self.root_path)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .to_string();

                if let Ok(content) = fs::read(entry.path()) {
                    let mut hasher = Sha256::new();
                    hasher.update(&content);
                    let result = hasher.finalize();
                    let hash = hex::encode(result);
                    file_hashes.insert(relative_path, hash);
                }
            }
        }

        Ok(Snapshot {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            file_hashes,
        })
    }

    pub fn diff(&self, old: &Snapshot, new: &Snapshot) -> SnapshotDiff {
        let mut new_files = Vec::new();
        let mut modified_files = Vec::new();
        let mut deleted_files = Vec::new();

        // Check for new and modified
        for (path, new_hash) in &new.file_hashes {
            match old.file_hashes.get(path) {
                Some(old_hash) => {
                    if old_hash != new_hash {
                        modified_files.push(path.clone());
                    }
                }
                None => {
                    new_files.push(path.clone());
                }
            }
        }

        // Check for deleted
        for path in old.file_hashes.keys() {
            if !new.file_hashes.contains_key(path) {
                deleted_files.push(path.clone());
            }
        }

        SnapshotDiff {
            new_files,
            modified_files,
            deleted_files,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_snapshot_diff() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let manager = SnapshotManager::new(&root);

        // 1. Create file A
        let file_a = root.join("file_a.txt");
        let mut f = File::create(&file_a).unwrap();
        f.write_all(b"Hello").unwrap();

        let snap1 = manager.create_snapshot().unwrap();
        assert_eq!(snap1.file_hashes.len(), 1);

        // 2. Modify A, Create B
        let mut f = File::create(&file_a).unwrap(); // Overwrite
        f.write_all(b"Hello World").unwrap();

        let file_b = root.join("file_b.txt");
        let mut f = File::create(&file_b).unwrap();
        f.write_all(b"New File").unwrap();

        let snap2 = manager.create_snapshot().unwrap();
        assert_eq!(snap2.file_hashes.len(), 2);

        // 3. Diff
        let diff = manager.diff(&snap1, &snap2);
        assert!(diff.new_files.contains(&"file_b.txt".to_string()));
        assert!(diff.modified_files.contains(&"file_a.txt".to_string()));
        assert!(diff.deleted_files.is_empty());

        // 4. Delete B
        fs::remove_file(&file_b).unwrap();
        let snap3 = manager.create_snapshot().unwrap();

        let diff2 = manager.diff(&snap2, &snap3);
        assert!(diff2.deleted_files.contains(&"file_b.txt".to_string()));
    }
}
