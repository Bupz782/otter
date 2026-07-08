use domain::ports::storage_port::StorageError;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Return the directory that contains `.sql` migration files.
///
/// `OTTER_MIGRATIONS_DIR` is honored at runtime so containers and
/// deployments can point at the bundled migrations without relying on
/// `CARGO_MANIFEST_DIR`. When the variable is unset the directory is resolved
/// from the crate manifest at compile time, which works for local builds and
/// tests.
pub fn migrations_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OTTER_MIGRATIONS_DIR") {
        PathBuf::from(dir)
    } else {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations"))
    }
}

/// List every `.sql` file in the migrations directory, sorted lexicographically.
pub fn migration_files() -> Result<Vec<PathBuf>, StorageError> {
    let dir = migrations_dir();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| {
            StorageError::InitFailed(format!(
                "failed to read migrations dir {}: {}",
                dir.display(),
                e
            ))
        })?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "sql").unwrap_or(false))
        .collect();
    entries.sort();
    Ok(entries)
}

/// Extract the numeric version prefix from a migration filename such as
/// `0001_init.sql`.
pub fn migration_version(path: &Path) -> Result<i64, StorageError> {
    let file_stem = path
        .file_stem()
        .ok_or_else(|| {
            StorageError::InitFailed(format!("invalid migration path: {}", path.display()))
        })?
        .to_string_lossy();
    file_stem
        .split('_')
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            StorageError::InitFailed(format!("invalid migration filename: {}", path.display()))
        })
}

/// Current Unix timestamp in seconds, used to record when a migration was applied.
pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
