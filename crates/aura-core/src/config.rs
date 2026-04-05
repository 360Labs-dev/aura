//! # Aura Project Configuration (aura.toml)
//!
//! Parses the project configuration file. Like TypeScript's tsconfig.json.
//!
//! ## Format
//! ```toml
//! [app]
//! name = "MyApp"
//! version = "1.0.0"
//! aura-version = "0.1.0"
//!
//! [targets]
//! web = true
//! ios = true
//! android = true
//!
//! [theme]
//! default = "modern.dark"
//!
//! [dependencies]
//! "@aura/charts" = "^1.0.0"
//!
//! [[references]]
//! path = "../shared-lib"
//! ```

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// Parsed aura.toml configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct AuraConfig {
    pub app: Option<AppConfig>,
    pub targets: Option<TargetsConfig>,
    pub theme: Option<ThemeConfig>,
    pub dependencies: Option<HashMap<String, String>>,
    pub references: Option<Vec<ReferenceConfig>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "aura-version")]
    pub aura_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetsConfig {
    pub web: Option<bool>,
    pub ios: Option<bool>,
    pub android: Option<bool>,
    pub windows: Option<bool>,
    pub tui: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeConfig {
    pub default: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReferenceConfig {
    pub path: String,
    #[serde(default)]
    pub types_only: bool,
}

impl AuraConfig {
    /// Load config from an aura.toml file.
    pub fn load(project_root: &Path) -> Option<Self> {
        let path = project_root.join("aura.toml");
        let content = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Get the app name, falling back to directory name.
    pub fn app_name(&self, project_root: &Path) -> String {
        self.app
            .as_ref()
            .and_then(|a| a.name.clone())
            .unwrap_or_else(|| {
                project_root
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "MyApp".to_string())
            })
    }

    /// Get enabled target platforms.
    pub fn enabled_targets(&self) -> Vec<String> {
        let mut targets = Vec::new();
        if let Some(ref t) = self.targets {
            if t.web.unwrap_or(true) {
                targets.push("web".to_string());
            }
            if t.ios.unwrap_or(false) {
                targets.push("ios".to_string());
            }
            if t.android.unwrap_or(false) {
                targets.push("android".to_string());
            }
            if t.windows.unwrap_or(false) {
                targets.push("windows".to_string());
            }
            if t.tui.unwrap_or(false) {
                targets.push("tui".to_string());
            }
        } else {
            targets.push("web".to_string());
        }
        targets
    }

    /// Get project references.
    pub fn project_references(&self) -> Vec<&ReferenceConfig> {
        self.references
            .as_ref()
            .map(|r| r.iter().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[app]
name = "TestApp"
version = "1.0.0"
aura-version = "0.1.0"

[targets]
web = true
ios = true
android = false

[theme]
default = "modern.dark"

[dependencies]
"@aura/charts" = "^1.0.0"

[[references]]
path = "../shared"
"#;
        let config: AuraConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.app.as_ref().unwrap().name,
            Some("TestApp".to_string())
        );
        assert_eq!(config.enabled_targets(), vec!["web", "ios"]);
        assert_eq!(
            config.theme.as_ref().unwrap().default,
            Some("modern.dark".to_string())
        );
        assert_eq!(config.references.as_ref().unwrap().len(), 1);
        assert_eq!(config.references.as_ref().unwrap()[0].path, "../shared");
    }

    #[test]
    fn test_minimal_config() {
        let toml_str = r#"
[app]
name = "Mini"
"#;
        let config: AuraConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.app_name(Path::new(".")), "Mini");
        assert_eq!(config.enabled_targets(), vec!["web"]);
    }

    #[test]
    fn test_empty_config() {
        let config: AuraConfig = toml::from_str("").unwrap();
        assert!(config.app.is_none());
        assert_eq!(config.enabled_targets(), vec!["web"]);
    }
}
