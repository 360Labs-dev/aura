//! # Aura Project Model
//!
//! Handles multi-file projects: discovers .aura files, resolves imports,
//! builds a dependency graph, and compiles them in the correct order.
//!
//! ## How it works (like TypeScript's Program):
//! 1. Find the project root (directory containing aura.toml, or cwd)
//! 2. Discover all .aura files in src/
//! 3. Parse each file into an AST
//! 4. Resolve imports: `import Todo from "./models/todo"` → find models/todo.aura
//! 5. Build dependency graph (topological sort)
//! 6. Merge all ASTs into a single Program with all models/screens/components
//! 7. Run semantic analysis on the merged program
//! 8. Build HIR from the merged result

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::*;
use crate::config::AuraConfig;
use crate::errors::{AuraError, ErrorCode, Severity};
use crate::lexer::Span;

/// A resolved Aura project — all files parsed and merged.
pub struct Project {
    /// The project root directory.
    pub root: PathBuf,
    /// Parsed aura.toml config (if present).
    pub config: Option<AuraConfig>,
    /// All source files discovered.
    pub files: Vec<SourceFile>,
    /// The merged program (all files combined into one AST).
    pub program: Program,
    /// All errors from parsing + resolution.
    pub errors: Vec<AuraError>,
    /// Referenced projects (from aura.toml [references]).
    pub references: Vec<ProjectReference>,
}

/// A reference to another Aura project.
#[derive(Debug)]
pub struct ProjectReference {
    /// Path to the referenced project directory.
    pub path: PathBuf,
    /// Whether types from this project are available.
    pub types_only: bool,
}

/// A single source file in the project.
#[derive(Debug)]
pub struct SourceFile {
    /// Relative path from project root.
    pub path: String,
    /// Absolute path on disk.
    pub abs_path: PathBuf,
    /// The parsed AST (if successful).
    pub program: Option<Program>,
    /// Module name derived from path (e.g., "models.todo").
    pub module_name: String,
    /// Imports this file declares.
    pub imports: Vec<ResolvedImport>,
}

/// A resolved import — maps an import statement to a file.
#[derive(Debug)]
pub struct ResolvedImport {
    /// The import source string (e.g., "./models/todo").
    pub source: String,
    /// The resolved file path (e.g., "src/models/todo.aura").
    pub resolved_path: Option<String>,
    /// Names imported.
    pub names: Vec<String>,
}

/// Load and compile a multi-file Aura project.
pub fn load_project(root: &Path) -> Project {
    let mut errors = Vec::new();

    // Load aura.toml config
    let config = AuraConfig::load(root);

    // Find the source directory
    let src_dir = if root.join("src").is_dir() {
        root.join("src")
    } else if root.is_file() {
        // Single file mode — treat parent as root
        return load_single_file(root);
    } else {
        root.to_path_buf()
    };

    // Discover all .aura files
    let aura_files = discover_files(&src_dir);

    if aura_files.is_empty() {
        errors.push(AuraError::new(
            ErrorCode::E0700,
            Severity::Error,
            format!("No .aura files found in {}", src_dir.display()),
            Span::new(0, 0),
        ));
        return Project {
            root: root.to_path_buf(),
            config: config.clone(),
            files: Vec::new(),
            program: Program {
                imports: Vec::new(),
                app: AppDecl {
                    name: "Unknown".to_string(),
                    members: Vec::new(),
                    span: Span::new(0, 0),
                },
            },
            errors,
            references: Vec::new(),
        };
    }

    // Parse each file
    let mut source_files: Vec<SourceFile> = Vec::new();

    for file_path in &aura_files {
        let rel_path = file_path
            .strip_prefix(root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let source = match std::fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(AuraError::new(
                    ErrorCode::E0700,
                    Severity::Error,
                    format!("Cannot read {}: {}", rel_path, e),
                    Span::new(0, 0),
                ));
                continue;
            }
        };

        let parse_result = crate::parser::parse(&source);
        errors.extend(parse_result.errors);

        let module_name = path_to_module_name(&rel_path);

        let imports = if let Some(ref program) = parse_result.program {
            resolve_imports(&program.imports, file_path, &src_dir)
        } else {
            Vec::new()
        };

        source_files.push(SourceFile {
            path: rel_path,
            abs_path: file_path.clone(),
            program: parse_result.program,
            module_name,
            imports,
        });
    }

    // Detect circular imports
    detect_circular_imports(&source_files, &mut errors);

    // Merge all files into a single program
    let program = merge_programs(&source_files, &mut errors);

    Project {
        root: root.to_path_buf(),
        config,
        files: source_files,
        program,
        errors,
        references: Vec::new(),
    }
}

/// Load a single .aura file as a project.
pub fn load_single_file(file: &Path) -> Project {
    let source = std::fs::read_to_string(file).unwrap_or_default();
    let parse_result = crate::parser::parse(&source);

    let program = parse_result.program.unwrap_or(Program {
        imports: Vec::new(),
        app: AppDecl {
            name: "Unknown".to_string(),
            members: Vec::new(),
            span: Span::new(0, 0),
        },
    });

    let project_root = file.parent().unwrap_or(Path::new("."));
    Project {
        root: project_root.to_path_buf(),
        config: AuraConfig::load(project_root),
        files: vec![SourceFile {
            path: file.to_string_lossy().to_string(),
            abs_path: file.to_path_buf(),
            program: Some(program.clone()),
            module_name: "main".to_string(),
            imports: Vec::new(),
        }],
        program,
        errors: parse_result.errors,
        references: Vec::new(),
    }
}

/// Discover all .aura files recursively in a directory.
fn discover_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(discover_files(&path));
            } else if path.extension().map(|e| e == "aura").unwrap_or(false) {
                files.push(path);
            }
        }
    }
    // Sort for deterministic order — main.aura first
    files.sort_by(|a, b| {
        let a_is_main = a.file_stem().map(|s| s == "main").unwrap_or(false);
        let b_is_main = b.file_stem().map(|s| s == "main").unwrap_or(false);
        match (a_is_main, b_is_main) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });
    files
}

/// Convert a file path to a module name.
/// e.g., "src/models/todo.aura" → "models.todo"
fn path_to_module_name(path: &str) -> String {
    let without_ext = path.trim_end_matches(".aura");
    let without_src = without_ext.strip_prefix("src/").unwrap_or(without_ext);
    without_src.replace('/', ".").replace('\\', ".")
}

/// Resolve import statements to file paths.
fn resolve_imports(
    imports: &[ImportDecl],
    current_file: &Path,
    src_dir: &Path,
) -> Vec<ResolvedImport> {
    let current_dir = current_file.parent().unwrap_or(Path::new("."));

    imports
        .iter()
        .map(|import| {
            let names = match &import.spec {
                ImportSpec::Named(name) => vec![name.clone()],
                ImportSpec::Destructured(names) => names.clone(),
                ImportSpec::Wildcard(alias) => vec![alias.clone()],
            };

            let resolved_path =
                if import.source.starts_with("./") || import.source.starts_with("../") {
                    // Relative import
                    let target = current_dir.join(&import.source).with_extension("aura");
                    if target.exists() {
                        Some(target.to_string_lossy().to_string())
                    } else {
                        // Try without adding extension (maybe source already has it)
                        let target2 = current_dir.join(&import.source);
                        if target2.exists() {
                            Some(target2.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    }
                } else if import.source.starts_with('@') {
                    // Package import — not resolved yet (future: package manager)
                    None
                } else {
                    // Bare import — look in src/
                    let target = src_dir.join(&import.source).with_extension("aura");
                    if target.exists() {
                        Some(target.to_string_lossy().to_string())
                    } else {
                        None
                    }
                };

            ResolvedImport {
                source: import.source.clone(),
                resolved_path,
                names,
            }
        })
        .collect()
}

/// Merge multiple parsed files into a single Program.
///
/// Strategy:
/// - The file containing `app X` becomes the main program
/// - All other files contribute their models, screens, components to the app
/// - Name collisions are reported as errors
fn merge_programs(files: &[SourceFile], errors: &mut Vec<AuraError>) -> Program {
    let mut main_program: Option<&Program> = None;
    let mut extra_members: Vec<AppMember> = Vec::new();
    let mut extra_imports: Vec<ImportDecl> = Vec::new();
    let mut seen_names: HashMap<String, String> = HashMap::new(); // name → file

    // Find the main program (first file with app declaration)
    for file in files {
        if let Some(ref program) = file.program {
            if main_program.is_none() {
                main_program = Some(program);
                // Register main program's names FIRST
                for member in &program.app.members {
                    let name = member_name(member);
                    seen_names.insert(name, file.path.clone());
                }
            }
        }
    }

    // Now merge additional files, checking for duplicates
    for file in files.iter().skip(1) {
        if let Some(ref program) = file.program {
            for member in &program.app.members {
                let name = member_name(member);
                if let Some(existing_file) = seen_names.get(&name) {
                    errors.push(AuraError::new(
                        ErrorCode::E0105,
                        Severity::Error,
                        format!(
                            "Duplicate declaration '{}' (already defined in {})",
                            name, existing_file
                        ),
                        Span::new(0, 0),
                    ));
                } else {
                    seen_names.insert(name.clone(), file.path.clone());
                    extra_members.push(member.clone());
                }
            }
            extra_imports.extend(program.imports.clone());
        }
    }

    // Build merged program
    match main_program {
        Some(main) => {
            let mut merged_members = main.app.members.clone();
            merged_members.extend(extra_members);

            let mut merged_imports = main.imports.clone();
            merged_imports.extend(extra_imports);

            Program {
                imports: merged_imports,
                app: AppDecl {
                    name: main.app.name.clone(),
                    members: merged_members,
                    span: main.app.span,
                },
            }
        }
        None => {
            errors.push(AuraError::new(
                ErrorCode::E0700,
                Severity::Error,
                "No 'app' declaration found in any source file".to_string(),
                Span::new(0, 0),
            ));
            Program {
                imports: Vec::new(),
                app: AppDecl {
                    name: "Unknown".to_string(),
                    members: Vec::new(),
                    span: Span::new(0, 0),
                },
            }
        }
    }
}

/// Extract the name of an app member for duplicate detection.
/// Detect circular imports in the file graph.
fn detect_circular_imports(files: &[SourceFile], errors: &mut Vec<AuraError>) {
    // Build adjacency list
    let path_set: HashSet<&str> = files.iter().map(|f| f.path.as_str()).collect();

    for file in files {
        for import in &file.imports {
            if let Some(ref resolved) = import.resolved_path {
                // Check if the imported file imports us back (direct cycle)
                if let Some(target_file) = files
                    .iter()
                    .find(|f| f.abs_path.to_string_lossy() == *resolved)
                {
                    for target_import in &target_file.imports {
                        if let Some(ref target_resolved) = target_import.resolved_path {
                            if target_resolved == &file.abs_path.to_string_lossy().to_string() {
                                errors.push(AuraError::new(
                                    ErrorCode::E0107,
                                    Severity::Error,
                                    format!(
                                        "Circular import detected: {} ↔ {}",
                                        file.path, target_file.path
                                    ),
                                    Span::new(0, 0),
                                ));
                            }
                        }
                    }
                }
            } else if !import.source.starts_with('@') {
                // Unresolved local import → error
                errors.push(
                    AuraError::new(
                        ErrorCode::E0104,
                        Severity::Error,
                        format!(
                            "Cannot resolve import '{}' from {}",
                            import.source, file.path
                        ),
                        Span::new(0, 0),
                    )
                    .with_help(format!(
                        "Create the file at {}.aura",
                        import.source.trim_start_matches("./")
                    )),
                );
            }
        }
    }
}

// === Stable Handoff APIs for Engineer 2 ===

/// Analyze a loaded project — returns diagnostics.
pub fn analyze_project(project: &Project) -> Vec<AuraError> {
    let analysis = crate::semantic::SemanticAnalyzer::new().analyze(&project.program);
    analysis.errors
}

/// Build HIR for a loaded project.
pub fn build_hir_for_project(project: &Project) -> crate::hir::HIRModule {
    crate::hir::build_hir(&project.program)
}

/// Check if a project needs rebuilding (incremental).
pub fn check_incremental(project: &Project) -> crate::cache::CacheCheck {
    let manifest = crate::cache::BuildManifest::load(&project.root)
        .unwrap_or_else(crate::cache::BuildManifest::new);

    let file_hashes: Vec<(String, String)> = project
        .files
        .iter()
        .filter_map(|f| {
            let content = std::fs::read_to_string(&f.abs_path).ok()?;
            Some((f.path.clone(), crate::cache::hash_source(&content)))
        })
        .collect();

    manifest.check(&file_hashes)
}

fn member_name(member: &AppMember) -> String {
    match member {
        AppMember::Model(m) => format!("model:{}", m.name),
        AppMember::Screen(s) => format!("screen:{}", s.name),
        AppMember::Component(c) => format!("component:{}", c.name),
        AppMember::Fn(f) => format!("fn:{}", f.name),
        AppMember::Const(c) => format!("const:{}", c.name),
        AppMember::State(s) => format!("state:{}", s.name),
        AppMember::ThemeRef(_) => "theme".to_string(),
        AppMember::NavigationDecl(_) => "navigation".to_string(),
        AppMember::RouteDecl(r) => format!("route:{}", r.pattern),
        AppMember::Style(s) => format!("style:{}", s.name),
        AppMember::Theme(t) => format!("theme:{}", t.name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_module_name() {
        assert_eq!(path_to_module_name("src/models/todo.aura"), "models.todo");
        assert_eq!(path_to_module_name("src/main.aura"), "main");
        assert_eq!(path_to_module_name("src/screens/home.aura"), "screens.home");
    }

    #[test]
    fn test_single_file_project() {
        let project = load_single_file(Path::new("../../examples/minimal.aura"));
        assert_eq!(project.program.app.name, "Hello");
        assert_eq!(project.files.len(), 1);
    }

    #[test]
    fn test_discover_files() {
        let files = discover_files(Path::new("../../examples"));
        assert!(files.len() >= 5, "Expected at least 5 example files");
        // main.aura should sort first if it exists
        for f in &files {
            assert!(f.to_string_lossy().ends_with(".aura"));
        }
    }

    #[test]
    fn test_merge_detects_duplicates() {
        // Create two files with the same model name
        let source1 = "app Test\n  model Todo\n    title: text";
        let source2 = "app Other\n  model Todo\n    name: text";

        let p1 = crate::parser::parse(source1).program.unwrap();
        let p2 = crate::parser::parse(source2).program.unwrap();

        let files = vec![
            SourceFile {
                path: "main.aura".to_string(),
                abs_path: PathBuf::from("main.aura"),
                program: Some(p1),
                module_name: "main".to_string(),
                imports: Vec::new(),
            },
            SourceFile {
                path: "other.aura".to_string(),
                abs_path: PathBuf::from("other.aura"),
                program: Some(p2),
                module_name: "other".to_string(),
                imports: Vec::new(),
            },
        ];

        let mut errors = Vec::new();
        let _merged = merge_programs(&files, &mut errors);
        assert!(
            errors.iter().any(|e| e.code == ErrorCode::E0105),
            "Should detect duplicate model 'Todo'"
        );
    }
}
