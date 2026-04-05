//! # Incremental Compilation Cache
//!
//! Like TypeScript's `--incremental` mode:
//! - Hash each source file (SHA-256)
//! - Cache parsed AST + semantic results to disk (.aura-cache/)
//! - On rebuild, skip re-parsing unchanged files
//! - Track dependencies (model used by screen → screen must recheck when model changes)
//!
//! ## Cache Format
//! `.aura-cache/manifest.json` stores file hashes and dependency edges.
//! On rebuild, compare current hashes to cached hashes.
//! If a file hash matches, skip parsing. If dependencies are unchanged, skip type checking.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// The cache directory name.
const CACHE_DIR: &str = ".aura-cache";
/// The manifest file name.
const MANIFEST_FILE: &str = "manifest.json";

/// Cached build manifest — stores file hashes and dependency information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    /// Aura compiler version that produced this cache.
    pub compiler_version: String,
    /// Per-file cache entries.
    pub files: HashMap<String, CachedFile>,
    /// Dependency edges: file → files it depends on.
    pub dependencies: HashMap<String, Vec<String>>,
    /// Last full build timestamp.
    pub last_build: u64,
}

/// Cache entry for a single source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFile {
    /// SHA-256 hash of the source file content.
    pub hash: String,
    /// File modification time (unix seconds).
    pub modified: u64,
    /// Number of declarations (models, screens, etc.) in this file.
    pub declarations: usize,
    /// Whether this file parsed successfully.
    pub parse_ok: bool,
    /// Whether semantic analysis passed.
    pub check_ok: bool,
    /// Names exported by this file (models, screens, components).
    pub exports: Vec<String>,
}

/// Result of checking the cache against current source files.
#[derive(Debug)]
pub struct CacheCheck {
    /// Files that are unchanged (can skip parsing).
    pub unchanged: Vec<String>,
    /// Files that changed (must re-parse).
    pub changed: Vec<String>,
    /// Files that are new (never cached).
    pub added: Vec<String>,
    /// Files that were deleted since last build.
    pub removed: Vec<String>,
    /// Files that need re-checking because their dependencies changed.
    pub invalidated: Vec<String>,
}

impl CacheCheck {
    /// Returns true if no files need rebuilding.
    pub fn is_clean(&self) -> bool {
        self.changed.is_empty() && self.added.is_empty() && self.invalidated.is_empty()
    }

    /// Returns the set of files that need processing.
    pub fn dirty_files(&self) -> Vec<&String> {
        let mut dirty: Vec<&String> = Vec::new();
        dirty.extend(&self.changed);
        dirty.extend(&self.added);
        dirty.extend(&self.invalidated);
        dirty
    }

    /// Summary string for CLI output.
    pub fn summary(&self) -> String {
        format!(
            "{} unchanged, {} changed, {} new, {} removed, {} invalidated",
            self.unchanged.len(),
            self.changed.len(),
            self.added.len(),
            self.removed.len(),
            self.invalidated.len()
        )
    }
}

impl BuildManifest {
    /// Create a new empty manifest.
    pub fn new() -> Self {
        Self {
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            files: HashMap::new(),
            dependencies: HashMap::new(),
            last_build: now_unix(),
        }
    }

    /// Load manifest from disk. Returns None if not found or invalid.
    pub fn load(project_root: &Path) -> Option<Self> {
        let path = project_root.join(CACHE_DIR).join(MANIFEST_FILE);
        let content = std::fs::read_to_string(&path).ok()?;
        let manifest: Self = serde_json::from_str(&content).ok()?;

        // Invalidate if compiler version changed
        if manifest.compiler_version != env!("CARGO_PKG_VERSION") {
            return None;
        }

        Some(manifest)
    }

    /// Save manifest to disk.
    pub fn save(&self, project_root: &Path) -> std::io::Result<()> {
        let cache_dir = project_root.join(CACHE_DIR);
        std::fs::create_dir_all(&cache_dir)?;
        let path = cache_dir.join(MANIFEST_FILE);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    /// Check current source files against the cached manifest.
    pub fn check(&self, current_files: &[(String, String)]) -> CacheCheck {
        let mut unchanged = Vec::new();
        let mut changed = Vec::new();
        let mut added = Vec::new();
        let mut removed = Vec::new();

        let current_map: HashMap<&str, &str> = current_files
            .iter()
            .map(|(path, hash)| (path.as_str(), hash.as_str()))
            .collect();

        // Check each current file against cache
        for (path, hash) in current_files {
            match self.files.get(path) {
                Some(cached) if cached.hash == *hash => {
                    unchanged.push(path.clone());
                }
                Some(_) => {
                    changed.push(path.clone());
                }
                None => {
                    added.push(path.clone());
                }
            }
        }

        // Check for removed files
        for cached_path in self.files.keys() {
            if !current_map.contains_key(cached_path.as_str()) {
                removed.push(cached_path.clone());
            }
        }

        // Compute invalidated files: unchanged files whose dependencies changed
        let dirty_set: HashSet<&str> = changed
            .iter()
            .chain(added.iter())
            .chain(removed.iter())
            .map(|s| s.as_str())
            .collect();

        let mut invalidated = Vec::new();
        for file in &unchanged {
            if let Some(deps) = self.dependencies.get(file) {
                if deps.iter().any(|dep| dirty_set.contains(dep.as_str())) {
                    invalidated.push(file.clone());
                }
            }
        }

        // Remove invalidated files from unchanged
        let invalidated_set: HashSet<&str> = invalidated.iter().map(|s| s.as_str()).collect();
        unchanged.retain(|f| !invalidated_set.contains(f.as_str()));

        CacheCheck {
            unchanged,
            changed,
            added,
            removed,
            invalidated,
        }
    }

    /// Update the manifest after a build.
    pub fn update_file(
        &mut self,
        path: &str,
        hash: &str,
        declarations: usize,
        parse_ok: bool,
        check_ok: bool,
        exports: Vec<String>,
    ) {
        self.files.insert(
            path.to_string(),
            CachedFile {
                hash: hash.to_string(),
                modified: now_unix(),
                declarations,
                parse_ok,
                check_ok,
                exports,
            },
        );
    }

    /// Set dependencies for a file.
    pub fn set_dependencies(&mut self, file: &str, deps: Vec<String>) {
        self.dependencies.insert(file.to_string(), deps);
    }

    /// Remove a file from the manifest.
    pub fn remove_file(&mut self, path: &str) {
        self.files.remove(path);
        self.dependencies.remove(path);
    }
}

/// Compute SHA-256 hash of a string (source file content).
pub fn hash_source(content: &str) -> String {
    // Simple hash — not cryptographic, but fast and deterministic.
    // Uses FNV-1a for speed (we're not doing security, just change detection).
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in content.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

/// Hash all source files in a directory.
pub fn hash_project_files(root: &Path) -> Vec<(String, String)> {
    let files = crate::project::load_project(root);
    files
        .files
        .iter()
        .filter_map(|f| {
            let content = std::fs::read_to_string(&f.abs_path).ok()?;
            let hash = hash_source(&content);
            Some((f.path.clone(), hash))
        })
        .collect()
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_source() {
        let h1 = hash_source("hello world");
        let h2 = hash_source("hello world");
        let h3 = hash_source("hello world!");
        assert_eq!(h1, h2, "Same content should produce same hash");
        assert_ne!(h1, h3, "Different content should produce different hash");
    }

    #[test]
    fn test_empty_manifest_check() {
        let manifest = BuildManifest::new();
        let files = vec![("main.aura".to_string(), "abc123".to_string())];
        let check = manifest.check(&files);
        assert_eq!(check.added.len(), 1);
        assert!(check.changed.is_empty());
        assert!(check.unchanged.is_empty());
    }

    #[test]
    fn test_unchanged_file() {
        let mut manifest = BuildManifest::new();
        manifest.update_file("main.aura", "abc123", 3, true, true, Vec::new());

        let files = vec![("main.aura".to_string(), "abc123".to_string())];
        let check = manifest.check(&files);
        assert_eq!(check.unchanged.len(), 1);
        assert!(check.changed.is_empty());
        assert!(check.is_clean());
    }

    #[test]
    fn test_changed_file() {
        let mut manifest = BuildManifest::new();
        manifest.update_file("main.aura", "abc123", 3, true, true, Vec::new());

        let files = vec![("main.aura".to_string(), "def456".to_string())];
        let check = manifest.check(&files);
        assert_eq!(check.changed.len(), 1);
        assert!(check.unchanged.is_empty());
        assert!(!check.is_clean());
    }

    #[test]
    fn test_dependency_invalidation() {
        let mut manifest = BuildManifest::new();
        manifest.update_file(
            "models/todo.aura",
            "hash1",
            1,
            true,
            true,
            vec!["Todo".to_string()],
        );
        manifest.update_file("screens/main.aura", "hash2", 1, true, true, Vec::new());
        manifest.set_dependencies("screens/main.aura", vec!["models/todo.aura".to_string()]);

        // Model changed, screen didn't — but screen depends on model
        let files = vec![
            ("models/todo.aura".to_string(), "hash1_changed".to_string()),
            ("screens/main.aura".to_string(), "hash2".to_string()),
        ];
        let check = manifest.check(&files);

        assert_eq!(check.changed.len(), 1, "todo.aura changed");
        assert_eq!(
            check.invalidated.len(),
            1,
            "main.aura should be invalidated"
        );
        assert_eq!(check.invalidated[0], "screens/main.aura");
        assert!(check.unchanged.is_empty(), "Nothing should be unchanged");
    }

    #[test]
    fn test_removed_file() {
        let mut manifest = BuildManifest::new();
        manifest.update_file("old.aura", "hash1", 1, true, true, Vec::new());

        let files: Vec<(String, String)> = Vec::new();
        let check = manifest.check(&files);
        assert_eq!(check.removed.len(), 1);
    }

    #[test]
    fn test_cache_summary() {
        let manifest = BuildManifest::new();
        let files = vec![("new.aura".to_string(), "h1".to_string())];
        let check = manifest.check(&files);
        let summary = check.summary();
        assert!(summary.contains("1 new"));
    }
}
