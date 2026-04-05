//! Dependency resolution: given a list of requirements, produce an install plan.

use std::collections::HashMap;

use super::registry::PackageManifest;

/// An installation plan — what packages to install/update.
#[derive(Debug)]
pub struct InstallPlan {
    /// Packages to install (name → version).
    pub install: Vec<(String, String)>,
    /// Packages that are already installed and up-to-date.
    pub already_installed: Vec<String>,
    /// Packages that couldn't be resolved.
    pub unresolved: Vec<(String, String)>,
}

/// Resolve dependencies from aura.toml.
///
/// Currently supports:
/// - Exact versions: "1.0.0"
/// - Caret versions: "^1.0.0" (any 1.x.y)
/// - Wildcard: "*" (any version)
pub fn resolve_dependencies(
    requirements: &HashMap<String, String>,
    installed: &[PackageManifest],
) -> InstallPlan {
    let installed_map: HashMap<&str, &str> = installed
        .iter()
        .map(|p| (p.name.as_str(), p.version.as_str()))
        .collect();

    let mut plan = InstallPlan {
        install: Vec::new(),
        already_installed: Vec::new(),
        unresolved: Vec::new(),
    };

    for (name, version_req) in requirements {
        if let Some(&installed_version) = installed_map.get(name.as_str()) {
            if version_matches(installed_version, version_req) {
                plan.already_installed.push(name.clone());
            } else {
                // Need to update
                plan.install.push((name.clone(), version_req.clone()));
            }
        } else {
            // Not installed — need to install
            plan.install.push((name.clone(), version_req.clone()));
        }
    }

    plan
}

/// Check if an installed version satisfies a version requirement.
fn version_matches(installed: &str, requirement: &str) -> bool {
    if requirement == "*" {
        return true;
    }
    if let Some(req) = requirement.strip_prefix('^') {
        // Caret: ^1.0.0 matches any 1.x.y
        let req_major = req.split('.').next().unwrap_or("0");
        let inst_major = installed.split('.').next().unwrap_or("0");
        req_major == inst_major
    } else if let Some(req) = requirement.strip_prefix('~') {
        // Tilde: ~1.2.0 matches any 1.2.x
        let req_parts: Vec<&str> = req.split('.').collect();
        let inst_parts: Vec<&str> = installed.split('.').collect();
        req_parts.get(0) == inst_parts.get(0) && req_parts.get(1) == inst_parts.get(1)
    } else {
        // Exact match
        installed == requirement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_exact() {
        assert!(version_matches("1.0.0", "1.0.0"));
        assert!(!version_matches("1.0.1", "1.0.0"));
    }

    #[test]
    fn test_version_caret() {
        assert!(version_matches("1.0.0", "^1.0.0"));
        assert!(version_matches("1.5.3", "^1.0.0"));
        assert!(!version_matches("2.0.0", "^1.0.0"));
    }

    #[test]
    fn test_version_tilde() {
        assert!(version_matches("1.2.0", "~1.2.0"));
        assert!(version_matches("1.2.5", "~1.2.0"));
        assert!(!version_matches("1.3.0", "~1.2.0"));
    }

    #[test]
    fn test_version_wildcard() {
        assert!(version_matches("1.0.0", "*"));
        assert!(version_matches("99.99.99", "*"));
    }

    #[test]
    fn test_resolve_empty() {
        let plan = resolve_dependencies(&HashMap::new(), &[]);
        assert!(plan.install.is_empty());
        assert!(plan.already_installed.is_empty());
    }

    #[test]
    fn test_resolve_new_package() {
        let mut reqs = HashMap::new();
        reqs.insert("@aura/charts".to_string(), "^1.0.0".to_string());
        let plan = resolve_dependencies(&reqs, &[]);
        assert_eq!(plan.install.len(), 1);
        assert_eq!(plan.install[0].0, "@aura/charts");
    }

    #[test]
    fn test_resolve_already_installed() {
        let mut reqs = HashMap::new();
        reqs.insert("@aura/charts".to_string(), "^1.0.0".to_string());
        let installed = vec![PackageManifest {
            name: "@aura/charts".to_string(),
            version: "1.2.3".to_string(),
            description: None, author: None, license: None,
            repository: None, dependencies: HashMap::new(), hash: None,
        }];
        let plan = resolve_dependencies(&reqs, &installed);
        assert!(plan.install.is_empty());
        assert_eq!(plan.already_installed.len(), 1);
    }
}
