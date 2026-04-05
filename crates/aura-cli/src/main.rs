//! # Aura CLI
//!
//! The main command-line interface for the Aura language.

use clap::{Parser, Subcommand};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "aura")]
#[command(about = "The Aura programming language — Design that radiates.")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile .aura files to target platforms
    Build {
        /// Target platform: web, ios, android, all
        #[arg(short, long, default_value = "web")]
        target: String,

        /// Project root, source directory, or source file
        #[arg(default_value = ".")]
        path: String,

        /// Output directory
        #[arg(short, long, default_value = "build")]
        output: String,

        /// Error format: text (default) or json (for AI agents)
        #[arg(long)]
        format: Option<String>,
    },

    /// Build and run with live preview
    Run {
        #[arg(short, long, default_value = "web")]
        target: String,
        #[arg(long)]
        preview: Option<String>,
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// Format .aura source files
    Fmt {
        #[arg(default_value = ".")]
        path: String,
        #[arg(long)]
        check: bool,
    },

    /// Convert .aura code to plain English description
    Explain { file: String },

    /// Semantic diff between two .aura files
    Diff { a: String, b: String },

    /// Scaffold a new Aura project
    Init {
        name: String,
        #[arg(short, long, default_value = "app")]
        template: String,
    },

    /// Diagnose environment issues
    Doctor,

    /// Generate a running prototype from a description
    Sketch { description: String },

    /// Start the Agent API server (JSON-RPC over stdin/stdout)
    Agent {
        #[command(subcommand)]
        action: AgentCommands,
    },

    /// Package management
    Pkg {
        #[command(subcommand)]
        action: PkgCommands,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Start JSON-RPC server on stdin/stdout
    Serve,
    /// Send a single request (for testing)
    Call {
        /// JSON-RPC method name
        method: String,
        /// JSON params
        #[arg(default_value = "{}")]
        params: String,
    },
}

#[derive(Subcommand)]
enum PkgCommands {
    Install { package: String },
    Update { package: Option<String> },
    Remove { package: String },
    Publish,
}

struct ProjectContext {
    target_path: PathBuf,
    project_root: PathBuf,
    display_path: String,
    project: aura_core::project::Project,
    sources: HashMap<String, String>,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            target,
            path,
            output,
            format,
        } => build_command(&target, &path, &output, format.as_deref()),
        Commands::Run {
            target,
            preview: _preview,
            port,
        } => run_command(&target, port),
        Commands::Fmt { path, check } => fmt_command(&path, check),
        Commands::Explain { file } => explain_command(&file),
        Commands::Diff { a, b } => diff_command(&a, &b),
        Commands::Init { name, template } => init_command(&name, &template),
        Commands::Doctor => doctor_command(),
        Commands::Sketch { description } => sketch_command(&description),
        Commands::Agent { action } => match action {
            AgentCommands::Serve => agent_serve(),
            AgentCommands::Call { method, params } => agent_call(&method, &params),
        },
        Commands::Pkg { action: _action } => {
            eprintln!("  aura pkg not yet implemented");
        }
    }
}

fn build_command(target: &str, path: &str, output_dir: &str, format: Option<&str>) {
    if try_build_command(target, path, output_dir, format).is_err() {
        std::process::exit(1);
    }
}

fn try_build_command(
    target: &str,
    path: &str,
    output_dir: &str,
    format: Option<&str>,
) -> Result<(), ()> {
    let use_json = format == Some("json");
    let context = load_project_context(path).map_err(|message| {
        eprintln!("  error: {}", message);
    })?;
    let output_dir = resolve_output_dir(output_dir, &context);
    let output_dir_display = output_dir.display().to_string();

    eprintln!();
    eprintln!("  aura build v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("  {} → {}", context.display_path, target);

    let current_files = aura_core::cache::hash_project_files(&context.target_path);
    if let Some(manifest) = aura_core::cache::BuildManifest::load(&context.project_root) {
        let check = manifest.check(&current_files);
        if check.is_clean() {
            eprintln!("  [cached] No changes detected — skipping rebuild");
            eprintln!();
            return Ok(());
        }
        eprintln!("  [incremental] {}", check.summary());
    }
    eprintln!();

    eprintln!("  [1/4] Loading project...");
    if print_project_load_errors(&context, use_json) {
        return Err(());
    }

    eprintln!("  [2/4] Analyzing...");
    let analysis = aura_core::semantic::SemanticAnalyzer::new().analyze(&context.project.program);
    if !analysis.errors.is_empty() {
        let error_count = analysis.errors.iter().filter(|err| err.is_error()).count();
        let warning_count = analysis.errors.len() - error_count;

        if error_count > 0 {
            eprintln!("  {} error(s), {} warning(s):", error_count, warning_count);
        } else {
            eprintln!("  {} warning(s):", warning_count);
        }

        let (source, file) = if context.project.files.len() == 1 {
            let file = &context.project.files[0];
            (
                context
                    .sources
                    .get(&file.path)
                    .map(String::as_str)
                    .unwrap_or(""),
                file.path.as_str(),
            )
        } else {
            ("", context.display_path.as_str())
        };

        for err in &analysis.errors {
            if use_json {
                print_error_json(err);
            } else {
                print_error_text(err, source, file);
            }
        }

        if error_count > 0 {
            return Err(());
        }
    }

    eprintln!("  [3/4] Building IR...");
    let hir = aura_core::hir::build_hir(&context.project.program);

    eprintln!("  [4/4] Generating {}...", target);
    match target {
        "web" => {
            let output = aura_backend_web::compile_to_web(&hir);
            let out_path = output_dir.as_path();
            std::fs::create_dir_all(out_path).map_err(|err| {
                eprintln!(
                    "  error: Failed to create output directory '{}': {}",
                    output_dir_display, err
                );
            })?;

            write_file(out_path.join("index.html"), &output.html, "index.html")?;
            write_file(out_path.join("styles.css"), &output.css, "styles.css")?;
            write_file(out_path.join("app.js"), &output.js, "app.js")?;

            eprintln!();
            eprintln!("  Build complete:");
            eprintln!(
                "    {}/index.html  ({} bytes)",
                output_dir_display,
                output.html.len()
            );
            eprintln!(
                "    {}/styles.css  ({} bytes)",
                output_dir_display,
                output.css.len()
            );
            eprintln!(
                "    {}/app.js      ({} bytes)",
                output_dir_display,
                output.js.len()
            );
            eprintln!();
            eprintln!(
                "  Open {}/index.html in a browser to preview.",
                output_dir_display
            );
        }
        "ios" | "swift" => {
            let output = aura_backend_swift::compile_to_swift(&hir);
            let out_path = output_dir.as_path();
            std::fs::create_dir_all(out_path).map_err(|err| {
                eprintln!(
                    "  error: Failed to create output directory '{}': {}",
                    output_dir_display, err
                );
            })?;

            write_file(
                out_path.join(&output.filename),
                &output.swift,
                output.filename.as_str(),
            )?;

            eprintln!();
            eprintln!("  Build complete:");
            eprintln!(
                "    {}/{}  ({} bytes)",
                output_dir_display,
                output.filename,
                output.swift.len()
            );
        }
        "android" | "compose" => {
            let output = aura_backend_compose::compile_to_compose(&hir);
            let out_path = output_dir.as_path();
            std::fs::create_dir_all(out_path).map_err(|err| {
                eprintln!(
                    "  error: Failed to create output directory '{}': {}",
                    output_dir_display, err
                );
            })?;

            write_file(
                out_path.join(&output.filename),
                &output.kotlin,
                output.filename.as_str(),
            )?;

            eprintln!();
            eprintln!("  Build complete:");
            eprintln!(
                "    {}/{}  ({} bytes)",
                output_dir_display,
                output.filename,
                output.kotlin.len()
            );
        }
        "all" => {
            let out_base = output_dir.as_path();

            let web_out = out_base.join("web");
            std::fs::create_dir_all(&web_out).map_err(|err| {
                eprintln!(
                    "  error: Failed to create web output directory '{}': {}",
                    web_out.display(),
                    err
                );
            })?;
            let web = aura_backend_web::compile_to_web(&hir);
            write_file(web_out.join("index.html"), &web.html, "web/index.html")?;
            write_file(web_out.join("styles.css"), &web.css, "web/styles.css")?;
            write_file(web_out.join("app.js"), &web.js, "web/app.js")?;

            let ios_out = out_base.join("ios");
            std::fs::create_dir_all(&ios_out).map_err(|err| {
                eprintln!(
                    "  error: Failed to create iOS output directory '{}': {}",
                    ios_out.display(),
                    err
                );
            })?;
            let ios = aura_backend_swift::compile_to_swift(&hir);
            write_file(
                ios_out.join(&ios.filename),
                &ios.swift,
                ios.filename.as_str(),
            )?;

            let android_out = out_base.join("android");
            std::fs::create_dir_all(&android_out).map_err(|err| {
                eprintln!(
                    "  error: Failed to create Android output directory '{}': {}",
                    android_out.display(),
                    err
                );
            })?;
            let android = aura_backend_compose::compile_to_compose(&hir);
            write_file(
                android_out.join(&android.filename),
                &android.kotlin,
                android.filename.as_str(),
            )?;

            eprintln!();
            eprintln!("  Build complete (all platforms):");
            eprintln!("    {}/web/         (HTML/CSS/JS)", output_dir_display);
            eprintln!("    {}/ios/         (SwiftUI)", output_dir_display);
            eprintln!("    {}/android/     (Jetpack Compose)", output_dir_display);
        }
        _ => {
            eprintln!("  error: Unknown target '{}'", target);
            eprintln!("  Available targets: web, ios, android, all");
            return Err(());
        }
    }

    save_build_manifest(&context, &current_files, &analysis.errors);
    Ok(())
}

fn write_file(path: PathBuf, content: &str, label: &str) -> Result<(), ()> {
    std::fs::write(&path, content).map_err(|err| {
        eprintln!(
            "  error: Failed to write {} ({}): {}",
            label,
            path.display(),
            err
        );
    })
}

fn resolve_output_dir(output_dir: &str, context: &ProjectContext) -> PathBuf {
    let output_path = Path::new(output_dir);
    if output_path.is_absolute() {
        output_path.to_path_buf()
    } else {
        context.project_root.join(output_path)
    }
}

fn load_project_context(path: &str) -> Result<ProjectContext, String> {
    let raw_path = Path::new(path);
    let target_path = if raw_path.exists() {
        if raw_path.is_file() {
            raw_path
                .parent()
                .and_then(find_project_root)
                .unwrap_or_else(|| raw_path.to_path_buf())
        } else {
            resolve_source_directory(raw_path).ok_or_else(|| invalid_project_path_message(raw_path))?
        }
    } else if path == "." {
        let cwd = std::env::current_dir().map_err(|err| err.to_string())?;
        resolve_source_directory(&cwd).ok_or_else(|| invalid_project_path_message(&cwd))?
    } else {
        return Err(format!("Path '{}' does not exist", path));
    };

    let project = aura_core::project::load_project(&target_path);
    let project_root = project.root.clone();
    let display_path = target_path.display().to_string();
    let mut sources = HashMap::new();

    for file in &project.files {
        if let Ok(source) = std::fs::read_to_string(&file.abs_path) {
            sources.insert(file.path.clone(), source);
        }
    }

    Ok(ProjectContext {
        target_path,
        project_root,
        display_path,
        project,
        sources,
    })
}

fn find_project_root(start: &Path) -> Option<PathBuf> {
    let start_dir = if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    };

    for dir in start_dir.ancestors() {
        if dir.join("aura.toml").exists() {
            return Some(dir.to_path_buf());
        }

        if dir.join("src").join("main.aura").exists() {
            return Some(dir.to_path_buf());
        }
    }

    None
}

fn direct_aura_sources(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut files: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| entry.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("aura"))
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    files
}

fn resolve_source_directory(dir: &Path) -> Option<PathBuf> {
    if let Some(project_root) = find_project_root(dir) {
        return Some(project_root);
    }

    let direct_sources = direct_aura_sources(dir);
    match direct_sources.as_slice() {
        [] => None,
        [single] => Some(single.clone()),
        _ => Some(dir.to_path_buf()),
    }
}

fn invalid_project_path_message(path: &Path) -> String {
    format!(
        "Path '{}' is not an Aura project root or source directory.\n  hint: run this from a directory with `aura.toml`, `src/main.aura`, or direct `.aura` source files.",
        path.display()
    )
}

fn print_project_load_errors(context: &ProjectContext, use_json: bool) -> bool {
    let mut has_errors = false;

    for file in &context.project.files {
        if let Some(source) = context.sources.get(&file.path) {
            let parse_result = aura_core::parser::parse(source);
            if !parse_result.errors.is_empty() {
                has_errors = true;
                for err in &parse_result.errors {
                    if use_json {
                        print_error_json(err);
                    } else {
                        print_error_text(err, source, &file.path);
                    }
                }
            }
        }
    }

    for err in context
        .project
        .errors
        .iter()
        .filter(|err| err.span.start == 0 && err.span.end == 0)
    {
        has_errors |= err.is_error();
        if use_json {
            print_error_json(err);
        } else {
            print_error_text(err, "", &context.display_path);
        }
    }

    has_errors
        || context.project.files.is_empty()
        || context.project.files.iter().any(|f| f.program.is_none())
}

fn save_build_manifest(
    context: &ProjectContext,
    current_files: &[(String, String)],
    analysis_errors: &[aura_core::AuraError],
) {
    let mut manifest = aura_core::cache::BuildManifest::load(&context.project_root)
        .unwrap_or_else(aura_core::cache::BuildManifest::new);

    let current_hashes: HashMap<String, String> = current_files.iter().cloned().collect();
    let current_set: HashSet<String> = current_hashes.keys().cloned().collect();
    let stale_files: Vec<String> = manifest
        .files
        .keys()
        .filter(|path| !current_set.contains(path.as_str()))
        .cloned()
        .collect();

    for stale in stale_files {
        manifest.remove_file(&stale);
    }

    let check_ok = !analysis_errors.iter().any(|err| err.is_error());

    for file in &context.project.files {
        let Some(hash) = current_hashes.get(&file.path) else {
            continue;
        };

        let declarations = file
            .program
            .as_ref()
            .map(|program| program.app.members.len())
            .unwrap_or(0);
        let exports = file
            .program
            .as_ref()
            .map(|program| {
                program
                    .app
                    .members
                    .iter()
                    .map(export_name)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        manifest.update_file(
            &file.path,
            hash,
            declarations,
            file.program.is_some(),
            check_ok,
            exports,
        );

        let dependencies = file
            .imports
            .iter()
            .filter_map(|import| import.resolved_path.as_deref())
            .map(|resolved_path| {
                let resolved = Path::new(resolved_path);
                resolved
                    .strip_prefix(&context.project_root)
                    .unwrap_or(resolved)
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        manifest.set_dependencies(&file.path, dependencies);
    }

    manifest.last_build = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let _ = manifest.save(&context.project_root);
}

fn export_name(member: &aura_core::ast::AppMember) -> String {
    match member {
        aura_core::ast::AppMember::Model(model) => model.name.clone(),
        aura_core::ast::AppMember::Screen(screen) => screen.name.clone(),
        aura_core::ast::AppMember::Component(component) => component.name.clone(),
        aura_core::ast::AppMember::Fn(function) => function.name.clone(),
        aura_core::ast::AppMember::Const(constant) => constant.name.clone(),
        aura_core::ast::AppMember::State(state) => state.name.clone(),
        aura_core::ast::AppMember::ThemeRef(_) => "theme".to_string(),
        aura_core::ast::AppMember::NavigationDecl(_) => "navigation".to_string(),
        aura_core::ast::AppMember::RouteDecl(route) => route.pattern.clone(),
        aura_core::ast::AppMember::Style(style) => style.name.clone(),
        aura_core::ast::AppMember::Theme(theme) => theme.name.clone(),
    }
}

fn print_error_text(err: &aura_core::AuraError, source: &str, file: &str) {
    let severity = match err.severity {
        aura_core::Severity::Error => "error",
        aura_core::Severity::Warning => "warning",
        aura_core::Severity::Info => "info",
    };

    eprintln!("  {}[{}]: {}", severity, err.code, err.message);
    if source.is_empty() {
        eprintln!("    --> {}", file);
    } else {
        let (line, col) = byte_to_line_col(source, err.span.start);
        eprintln!("    --> {}:{}:{}", file, line, col);
    }

    if let Some(ref help) = err.help {
        eprintln!("    = help: {}", help);
    }

    if let Some(ref fix) = err.fix {
        eprintln!(
            "    = fix (confidence {:.0}%): replace with '{}'",
            fix.confidence * 100.0,
            fix.replacement
        );
    }
    eprintln!();
}

fn print_error_json(err: &aura_core::AuraError) {
    let json = serde_json::json!({
        "code": format!("{}", err.code),
        "severity": match err.severity {
            aura_core::Severity::Error => "error",
            aura_core::Severity::Warning => "warning",
            aura_core::Severity::Info => "info",
        },
        "message": err.message,
        "span": { "start": err.span.start, "end": err.span.end },
        "help": err.help,
        "fix": err.fix.as_ref().map(|f| serde_json::json!({
            "replacement": f.replacement,
            "confidence": f.confidence,
        })),
    });
    println!("{}", json);
}

fn byte_to_line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (index, ch) in source.char_indices() {
        if index >= byte_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn fmt_command(path: &str, check: bool) {
    let input_path = Path::new(path);
    if input_path.exists() && input_path.is_file() {
        let formatted = format_file(input_path, check);
        if check && !formatted {
            eprintln!("  {} needs formatting", input_path.display());
            std::process::exit(1);
        }
        if check && formatted {
            eprintln!("  {} is already formatted", input_path.display());
        }
        return;
    }

    let target_path = if input_path.exists() && input_path.is_dir() {
        find_project_root(input_path).unwrap_or_else(|| input_path.to_path_buf())
    } else if path == "." {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        find_project_root(&cwd).unwrap_or(cwd)
    } else {
        eprintln!("  error: Path '{}' does not exist", path);
        std::process::exit(1);
    };

    let project = aura_core::project::load_project(&target_path);
    if project.files.is_empty() {
        eprintln!(
            "  error: No .aura files found under '{}'",
            target_path.display()
        );
        std::process::exit(1);
    }

    let mut needs_formatting = Vec::new();
    for file in &project.files {
        if !format_file(&file.abs_path, true) {
            needs_formatting.push(file.path.clone());
        }
    }

    if check {
        if needs_formatting.is_empty() {
            eprintln!(
                "  All Aura files are formatted in {}",
                target_path.display()
            );
        } else {
            eprintln!(
                "  {} file(s) need formatting in {}",
                needs_formatting.len(),
                target_path.display()
            );
            for file in needs_formatting {
                eprintln!("    {}", file);
            }
            std::process::exit(1);
        }
        return;
    }

    for file in &project.files {
        let _ = format_file(&file.abs_path, false);
    }

    eprintln!(
        "  Formatted {} Aura file(s) in {}",
        project.files.len(),
        target_path.display()
    );
}

fn format_file(path: &Path, check: bool) -> bool {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("  error: Cannot read '{}': {}", path.display(), err);
            std::process::exit(1);
        }
    };

    let result = aura_core::parser::parse(&source);
    if let Some(ref program) = result.program {
        let formatted = aura_core::fmt::format(program);
        if check {
            formatted == source
        } else {
            std::fs::write(path, &formatted).expect("Failed to write formatted file");
            eprintln!("  Formatted: {}", path.display());
            true
        }
    } else {
        eprintln!(
            "  error: Cannot format '{}' — parse errors:",
            path.display()
        );
        for err in &result.errors {
            eprintln!("    {}", err.message);
        }
        std::process::exit(1);
    }
}

fn explain_command(file: &str) {
    let source = match std::fs::read_to_string(file) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("  error: Cannot read '{}': {}", file, err);
            std::process::exit(1);
        }
    };

    let result = aura_core::parser::parse(&source);
    if let Some(ref program) = result.program {
        let hir = aura_core::hir::build_hir(program);
        let explanation = aura_core::explain::explain(&hir);
        println!("{}", explanation);
    } else {
        eprintln!("  error: Failed to parse '{}'", file);
        for err in &result.errors {
            eprintln!("    {}", err.message);
        }
        std::process::exit(1);
    }
}

fn diff_command(file_a: &str, file_b: &str) {
    let source_a = match std::fs::read_to_string(file_a) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("  error: Cannot read '{}': {}", file_a, err);
            std::process::exit(1);
        }
    };
    let source_b = match std::fs::read_to_string(file_b) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("  error: Cannot read '{}': {}", file_b, err);
            std::process::exit(1);
        }
    };

    let result_a = aura_core::parser::parse(&source_a);
    let result_b = aura_core::parser::parse(&source_b);

    let program_a = match result_a.program {
        Some(program) => program,
        None => {
            eprintln!("  error: Failed to parse '{}'", file_a);
            std::process::exit(1);
        }
    };
    let program_b = match result_b.program {
        Some(program) => program,
        None => {
            eprintln!("  error: Failed to parse '{}'", file_b);
            std::process::exit(1);
        }
    };

    let hir_a = aura_core::hir::build_hir(&program_a);
    let hir_b = aura_core::hir::build_hir(&program_b);

    let changes = aura_core::diff::diff(&hir_a, &hir_b);

    println!("  Aura Semantic Diff");
    println!("  {} → {}", file_a, file_b);
    println!();
    print!("{}", aura_core::diff::format_diff(&changes));
}

fn sketch_command(description: &str) {
    eprintln!();
    eprintln!("  aura sketch");
    eprintln!("  Description: \"{}\"", description);
    eprintln!();

    let code = aura_core::sketch::sketch(description);

    let result = aura_core::parser::parse(&code);
    if result.program.is_none() {
        eprintln!("  warning: generated code has parse issues (template bug)");
    }

    let filename = "sketch.aura";
    std::fs::write(filename, &code).expect("Failed to write sketch.aura");

    eprintln!("  Generated: {} ({} lines)", filename, code.lines().count());
    eprintln!();

    println!("{}", code);

    eprintln!("  Building preview...");

    if let Some(ref program) = result.program {
        let hir = aura_core::hir::build_hir(program);
        let output = aura_backend_web::compile_to_web(&hir);

        let out_dir = "build/sketch";
        std::fs::create_dir_all(out_dir).ok();
        std::fs::write(format!("{}/index.html", out_dir), &output.html).ok();
        std::fs::write(format!("{}/styles.css", out_dir), &output.css).ok();
        std::fs::write(format!("{}/app.js", out_dir), &output.js).ok();

        eprintln!("  Preview: {}/index.html", out_dir);
    }

    eprintln!();
    eprintln!("  Open sketch.aura to customize, or run:");
    eprintln!("    aura build sketch.aura --target all");
}

fn init_command(name: &str, template: &str) {
    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("  error: Directory '{}' already exists", name);
        std::process::exit(1);
    }

    let raw_name = dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let app_name = to_app_name(&raw_name);
    let app_title = to_display_name(&raw_name);

    std::fs::create_dir_all(dir.join("src")).expect("Failed to create project directory");

    let toml = format!(
        r#"[app]
name = "{}"
version = "0.1.0"
aura-version = "0.1.0"

[targets]
web = true
ios = true
android = true

[theme]
default = "modern.light"
"#,
        app_name
    );
    std::fs::write(dir.join("aura.toml"), toml).expect("Failed to write aura.toml");

    let main_aura = match template {
        "counter" => aura_core::sketch::sketch("counter app"),
        "todo" => aura_core::sketch::sketch("todo app with filter"),
        "chat" => aura_core::sketch::sketch("chat app"),
        _ => format!(
            r#"app {}
  theme: modern.light

  screen Main
    view
      column gap.lg padding.2xl align.center
        heading "{}" size.2xl .bold
        text "Edit src/main.aura, then run aura build or aura run." .secondary
        button "Get Started" .accent -> getStarted()

    action getStarted
      return
"#,
            app_name, app_title
        ),
    };
    let formatted_main = aura_core::parser::parse(&main_aura)
        .program
        .and_then(|program| {
            let formatted = aura_core::fmt::format(&program);
            if aura_core::parser::parse(&formatted).program.is_some() {
                Some(formatted)
            } else {
                None
            }
        })
        .unwrap_or(main_aura);
    std::fs::write(dir.join("src/main.aura"), formatted_main).expect("Failed to write main.aura");

    let readme = format!(
        "# {title}\n\nGenerated with `aura init`.\n\n## Project workflow\n\n```bash\naura run\naura build\naura build --target all\naura fmt\naura doctor\n```\n\n## Structure\n\n- `src/main.aura` — app entry point\n- `build/` — generated output\n- `.aura-cache/` — incremental build cache\n",
        title = app_title
    );
    std::fs::write(dir.join("README.md"), readme).expect("Failed to write README.md");

    std::fs::write(dir.join(".gitignore"), "build/\n.aura-cache/\n").ok();

    eprintln!();
    eprintln!("  Created project: {}/", name);
    eprintln!();
    eprintln!("  {}/aura.toml       Project configuration", name);
    eprintln!("  {}/src/main.aura   Entry point", name);
    eprintln!("  {}/README.md       Project workflow", name);
    eprintln!();
    eprintln!("  Next steps:");
    eprintln!("    cd {}", name);
    eprintln!("    aura run");
    eprintln!("    aura build");
    eprintln!("    aura build --target all");
}

fn run_command(target: &str, port: u16) {
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    if target != "web" {
        eprintln!("  error: `aura run` currently serves web output only.");
        eprintln!(
            "  hint: Use `aura run` for web preview and `aura build --target all` for multi-platform output."
        );
        std::process::exit(1);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = find_project_root(&cwd).unwrap_or(cwd);
    let project_label = project_root.display().to_string();
    let build_dir = project_root.join("build/dev");

    eprintln!();
    eprintln!("  aura run — dev server with file watching");
    eprintln!("  Project: {}", project_label);
    eprintln!();

    if try_build_command(
        target,
        &project_label,
        build_dir.to_string_lossy().as_ref(),
        None,
    )
    .is_err()
    {
        std::process::exit(1);
    }
    inject_reload_script(&build_dir);

    let changed = Arc::new(AtomicBool::new(false));
    let changed_clone = changed.clone();
    let root_clone = project_root.clone();
    let build_label = build_dir.to_string_lossy().to_string();

    std::thread::spawn(move || {
        use notify::{Event, EventKind, RecursiveMode, Watcher};
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    let should_rebuild = event.paths.iter().any(|path| {
                        path.extension().map(|ext| ext == "aura").unwrap_or(false)
                            || path
                                .file_name()
                                .map(|name| name == "aura.toml")
                                .unwrap_or(false)
                    });
                    if should_rebuild {
                        tx.send(()).ok();
                    }
                }
            }
        })
        .expect("Failed to create file watcher");

        watcher.watch(&root_clone, RecursiveMode::Recursive).ok();
        eprintln!("  Watching for file changes...");

        loop {
            if rx.recv().is_err() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
            while rx.try_recv().is_ok() {}

            eprintln!();
            eprintln!("  File changed — rebuilding...");
            match try_build_command("web", &root_clone.display().to_string(), &build_label, None) {
                Ok(()) => {
                    inject_reload_script(Path::new(&build_label));
                    changed_clone.store(true, Ordering::SeqCst);
                    eprintln!("  Ready — browser will reload automatically");
                }
                Err(()) => {
                    eprintln!("  Rebuild failed — keeping the last successful preview");
                }
            }
        }
    });

    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("  error: Cannot bind to {}: {}", addr, err);
            std::process::exit(1);
        }
    };

    eprintln!("  Server: http://localhost:{}", port);
    eprintln!("  Press Ctrl+C to stop");
    eprintln!();

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buf = [0u8; 4096];
            let read = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..read]);

            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");

            if path == "/__reload" {
                let should_reload = changed.swap(false, Ordering::SeqCst);
                let body = if should_reload { "yes" } else { "no" };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).ok();
                continue;
            }

            let file_path = if path == "/" || path == "/index.html" {
                build_dir.join("index.html")
            } else if path == "/ping" {
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
                stream.write_all(response.as_bytes()).ok();
                continue;
            } else {
                build_dir.join(path.trim_start_matches('/'))
            };

            let (status, content_type, body) = if let Ok(body) = std::fs::read(&file_path) {
                let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
                    Some("html") => "text/html; charset=utf-8",
                    Some("css") => "text/css; charset=utf-8",
                    Some("js") => "application/javascript; charset=utf-8",
                    Some("json") => "application/json; charset=utf-8",
                    _ => "application/octet-stream",
                };
                ("200 OK", content_type, body)
            } else {
                ("404 Not Found", "text/plain", b"Not found".to_vec())
            };

            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                status,
                content_type,
                body.len()
            );
            stream.write_all(response.as_bytes()).ok();
            stream.write_all(&body).ok();
        }
    }
}

fn inject_reload_script(build_dir: &Path) {
    let html_path = build_dir.join("index.html");
    let reload_script = "<script>setInterval(()=>fetch('/__reload').then(r=>r.text()).then(v=>{if(v==='yes')location.reload()}),500)</script>";

    if let Ok(html) = std::fs::read_to_string(&html_path) {
        if html.contains(reload_script) {
            return;
        }
        let patched = html.replace("</body>", &format!("{}\n</body>", reload_script));
        std::fs::write(&html_path, patched).ok();
    }
}

fn agent_serve() {
    let server = aura_agent::AgentServer::new();
    eprintln!("  Aura Agent API v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("  Listening on stdin/stdout (JSON-RPC 2.0)");
    eprintln!(
        "  Send {{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\",\"params\":{{}}}} to test"
    );
    eprintln!();

    let stdin = std::io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let response = server.handle_json(trimmed);
                println!("{}", response);
            }
            Err(err) => {
                eprintln!("  error reading stdin: {}", err);
                break;
            }
        }
    }
}

fn agent_call(method: &str, params_str: &str) {
    let params: serde_json::Value = serde_json::from_str(params_str).unwrap_or_else(|err| {
        eprintln!("  error: Invalid JSON params: {}", err);
        std::process::exit(1);
    });

    let server = aura_agent::AgentServer::new();
    let request = aura_agent::Request {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(1),
        method: method.to_string(),
        params,
    };
    let response = server.handle_request(&request);
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
}

fn doctor_command() {
    eprintln!();
    eprintln!("  Aura Doctor v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("  Checking development environment...");
    eprintln!();

    let mut all_ok = true;

    let rust_ok = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .is_ok();
    if rust_ok {
        let version = std::process::Command::new("rustc")
            .arg("--version")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_default();
        eprintln!("  [ok] Rust: {}", version);
    } else {
        eprintln!("  [!!] Rust: NOT FOUND — install from https://rustup.rs");
        all_ok = false;
    }

    let cargo_ok = std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .is_ok();
    if cargo_ok {
        eprintln!("  [ok] Cargo: installed");
    } else {
        eprintln!("  [!!] Cargo: NOT FOUND");
        all_ok = false;
    }

    match std::process::Command::new("node").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            eprintln!("  [ok] Node.js: {} (for web dev server)", version);
        }
        _ => {
            eprintln!("  [--] Node.js: not found (optional, for web dev server)");
        }
    }

    match std::process::Command::new("xcodebuild")
        .arg("-version")
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            eprintln!("  [ok] Xcode: {} (for iOS/macOS target)", version);
        }
        _ => {
            eprintln!("  [--] Xcode: not found (needed for --target ios)");
        }
    }

    match std::env::var("ANDROID_HOME").or_else(|_| std::env::var("ANDROID_SDK_ROOT")) {
        Ok(path) => {
            eprintln!("  [ok] Android SDK: {} (for Android target)", path);
        }
        Err(_) => {
            eprintln!("  [--] Android SDK: not found (needed for --target android)");
        }
    }

    eprintln!("  [ok] Aura: v{}", env!("CARGO_PKG_VERSION"));

    if let Ok(cwd) = std::env::current_dir() {
        if let Some(project_root) = find_project_root(&cwd) {
            let project = aura_core::project::load_project(&project_root);
            let project_errors = project.errors.iter().filter(|err| err.is_error()).count();
            if project_errors == 0 && !project.files.is_empty() {
                eprintln!(
                    "  [ok] Project: {} ({} Aura files)",
                    project_root.display(),
                    project.files.len()
                );
            } else {
                eprintln!(
                    "  [!!] Project: {} ({} blocking issue(s))",
                    project_root.display(),
                    project_errors.max(1)
                );
                all_ok = false;
            }
        } else {
            eprintln!(
                "  [--] Project: no Aura project detected in {}",
                cwd.display()
            );
        }
    }

    eprintln!();
    if all_ok {
        eprintln!("  All required tools are installed.");
    } else {
        eprintln!("  Some required tools are missing. Install them and run `aura doctor` again.");
    }
}

fn to_app_name(name: &str) -> String {
    let mut words = name
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            let head = chars
                .next()
                .map(|ch| ch.to_ascii_uppercase().to_string())
                .unwrap_or_default();
            let tail = chars.as_str().to_ascii_lowercase();
            format!("{}{}", head, tail)
        })
        .collect::<String>();

    if words.is_empty() {
        words = "MyApp".to_string();
    }

    if words
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("App{}", words)
    } else {
        words
    }
}

fn to_display_name(name: &str) -> String {
    let title = name
        .split(|ch: char| ch == '-' || ch == '_' || ch.is_whitespace())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            let head = chars
                .next()
                .map(|ch| ch.to_ascii_uppercase().to_string())
                .unwrap_or_default();
            let tail = chars.as_str().to_ascii_lowercase();
            format!("{}{}", head, tail)
        })
        .collect::<Vec<_>>()
        .join(" ");

    if title.is_empty() {
        "My App".to_string()
    } else {
        title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_app_name_kebab_case_expected() {
        assert_eq!(to_app_name("task-flow"), "TaskFlow");
    }

    #[test]
    fn test_to_app_name_numeric_prefix_expected() {
        assert_eq!(to_app_name("360-labs"), "App360Labs");
    }

    #[test]
    fn test_find_project_root_walks_up_expected() {
        let unique = format!(
            "aura-cli-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let nested = root.join("src/screens");

        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.join("aura.toml"), "[app]\nname = \"Test\"\n").unwrap();

        assert_eq!(find_project_root(&nested), Some(root.clone()));

        let _ = std::fs::remove_file(root.join("aura.toml"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn test_load_project_context_rejects_repo_like_directory_without_project_markers() {
        let unique = format!(
            "aura-cli-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let nested = root.join("examples");

        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(
            nested.join("demo.aura"),
            "app Demo\n  screen Main\n    view\n      text \"Hi\"",
        )
        .unwrap();

        let err = match load_project_context(root.to_string_lossy().as_ref()) {
            Ok(_) => panic!("expected invalid project directory to be rejected"),
            Err(err) => err,
        };
        assert!(err.contains("not an Aura project root or source directory"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn test_load_project_context_accepts_source_directory_with_direct_aura_files() {
        let unique = format!(
            "aura-cli-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let main_file = root.join("main.aura");

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            &main_file,
            "app Demo\n  screen Main\n    view\n      text \"Hi\"",
        )
        .unwrap();

        let context = load_project_context(root.to_string_lossy().as_ref()).unwrap();
        assert_eq!(context.target_path, main_file);
        assert_eq!(context.project.files.len(), 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn test_resolve_source_directory_prefers_single_direct_aura_file() {
        let unique = format!(
            "aura-cli-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let nested = root.join("examples");
        let main_file = root.join("sketch.aura");

        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(
            &main_file,
            "app Demo\n  screen Main\n    view\n      text \"Hi\"",
        )
        .unwrap();
        std::fs::write(
            nested.join("other.aura"),
            "app Other\n  screen Main\n    view\n      text \"Nested\"",
        )
        .unwrap();

        assert_eq!(resolve_source_directory(&root), Some(main_file.clone()));

        let _ = std::fs::remove_dir_all(root);
    }
}
