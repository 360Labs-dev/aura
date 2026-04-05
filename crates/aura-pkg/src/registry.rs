//! Package registry: local storage and manifest format.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Where packages are stored locally.
const PACKAGES_DIR: &str = ".aura-packages";

/// A package manifest (from the package's aura.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    /// Dependencies: "@scope/name" → "^1.0.0"
    pub dependencies: HashMap<String, String>,
    /// SHA-256 hash of the package contents (for verification).
    pub hash: Option<String>,
}

/// A resolved package with its local path.
#[derive(Debug, Clone)]
pub struct Package {
    pub manifest: PackageManifest,
    pub path: PathBuf,
    pub source_files: Vec<String>,
}

impl Package {
    /// Load a package from a local directory.
    pub fn load(dir: &Path) -> Option<Self> {
        let toml_path = dir.join("aura.toml");
        let content = std::fs::read_to_string(&toml_path).ok()?;
        let config: aura_core::config::AuraConfig = toml::from_str(&content).ok()?;

        let name = config.app_name(dir);
        let version = config
            .app
            .as_ref()
            .and_then(|a| a.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string());

        // Find all .aura files
        let src_dir = dir.join("src");
        let source_files = if src_dir.exists() {
            find_aura_files(&src_dir)
                .iter()
                .map(|p| {
                    p.strip_prefix(dir)
                        .unwrap_or(p)
                        .to_string_lossy()
                        .to_string()
                })
                .collect()
        } else {
            Vec::new()
        };

        let deps = config.dependencies.unwrap_or_default();

        Some(Package {
            manifest: PackageManifest {
                name,
                version,
                description: None,
                author: None,
                license: None,
                repository: None,
                dependencies: deps,
                hash: None,
            },
            path: dir.to_path_buf(),
            source_files,
        })
    }

    /// Get the local package directory for a project.
    pub fn packages_dir(project_root: &Path) -> PathBuf {
        project_root.join(PACKAGES_DIR)
    }

    /// List installed packages.
    pub fn list_installed(project_root: &Path) -> Vec<Package> {
        let pkgs_dir = Self::packages_dir(project_root);
        if !pkgs_dir.exists() {
            return Vec::new();
        }

        let mut packages = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&pkgs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(pkg) = Package::load(&path) {
                        packages.push(pkg);
                    }
                }
            }
        }
        packages
    }

    /// Install a package from a local path.
    pub fn install_from_path(project_root: &Path, source_path: &Path) -> Result<Package, String> {
        let pkg = Package::load(source_path)
            .ok_or_else(|| format!("Not a valid Aura package: {}", source_path.display()))?;

        let dest = Self::packages_dir(project_root).join(&pkg.manifest.name);
        if dest.exists() {
            return Err(format!(
                "Package '{}' already installed. Use `aura pkg update` to update.",
                pkg.manifest.name
            ));
        }

        // Copy package files
        std::fs::create_dir_all(&dest)
            .map_err(|e| format!("Failed to create package dir: {}", e))?;
        copy_dir(source_path, &dest).map_err(|e| format!("Failed to copy package: {}", e))?;

        Ok(Package::load(&dest).unwrap_or(pkg))
    }

    /// Remove an installed package.
    pub fn remove(project_root: &Path, name: &str) -> Result<(), String> {
        let pkg_dir = Self::packages_dir(project_root).join(name);
        if !pkg_dir.exists() {
            return Err(format!("Package '{}' not installed", name));
        }
        std::fs::remove_dir_all(&pkg_dir).map_err(|e| format!("Failed to remove: {}", e))
    }

    /// Generate a package manifest for the current project.
    pub fn pack(project_root: &Path) -> Result<PackageManifest, String> {
        let config = aura_core::config::AuraConfig::load(project_root)
            .ok_or_else(|| "No aura.toml found. Run `aura init` first.".to_string())?;

        let name = config.app_name(project_root);
        let version = config
            .app
            .as_ref()
            .and_then(|a| a.version.clone())
            .unwrap_or_else(|| "0.1.0".to_string());

        // Hash all source files for integrity
        let src_dir = project_root.join("src");
        let files = if src_dir.exists() {
            find_aura_files(&src_dir)
        } else {
            Vec::new()
        };
        let mut hash_input = String::new();
        for file in &files {
            if let Ok(content) = std::fs::read_to_string(file) {
                hash_input.push_str(&content);
            }
        }
        let hash = aura_core::cache::hash_source(&hash_input);

        Ok(PackageManifest {
            name,
            version,
            description: None,
            author: None,
            license: None,
            repository: None,
            dependencies: config.dependencies.unwrap_or_default(),
            hash: Some(hash),
        })
    }
}

fn find_aura_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_aura_files(&path));
            } else if path.extension().map(|e| e == "aura").unwrap_or(false) {
                files.push(path);
            }
        }
    }
    files
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_installed_empty() {
        let packages = Package::list_installed(Path::new("/tmp/nonexistent"));
        assert!(packages.is_empty());
    }

    #[test]
    fn test_pack_manifest() {
        // Use the multifile test project
        let result = Package::pack(Path::new("../../tests/conformance/multifile"));
        assert!(result.is_ok(), "Pack failed: {:?}", result.err());
        let manifest = result.unwrap();
        assert_eq!(manifest.name, "MultiFileTest");
        assert!(manifest.hash.is_some());
    }
}
