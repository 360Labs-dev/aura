//! # Aura CLI
//!
//! The main command-line interface for the Aura language.

use clap::{Parser, Subcommand};
use std::path::Path;

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

        /// Source file or directory
        #[arg(default_value = "src/main.aura")]
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
        #[arg(default_value = "src")]
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

    /// Package management
    Pkg {
        #[command(subcommand)]
        action: PkgCommands,
    },
}

#[derive(Subcommand)]
enum PkgCommands {
    Install { package: String },
    Update { package: Option<String> },
    Remove { package: String },
    Publish,
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
        Commands::Run { target, preview, port } => {
            eprintln!("  aura run not yet implemented (dev server coming in Phase 2)");
        }
        Commands::Fmt { path, check } => {
            eprintln!("  aura fmt not yet implemented");
        }
        Commands::Explain { file } => explain_command(&file),
        Commands::Diff { a, b } => diff_command(&a, &b),
        Commands::Init { name, template } => {
            eprintln!("  aura init not yet implemented");
        }
        Commands::Doctor => {
            eprintln!("  aura doctor not yet implemented");
        }
        Commands::Sketch { description } => {
            eprintln!("  aura sketch not yet implemented");
        }
        Commands::Pkg { action } => {
            eprintln!("  aura pkg not yet implemented");
        }
    }
}

fn build_command(target: &str, path: &str, output_dir: &str, format: Option<&str>) {
    let use_json = format == Some("json");

    eprintln!();
    eprintln!("  aura build v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("  {} → {}", path, target);
    eprintln!();

    // Read source
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  error: Cannot read '{}': {}", path, e);

            // Try to find .aura files in the path as a directory
            if Path::new(path).is_dir() {
                let main_file = Path::new(path).join("main.aura");
                if main_file.exists() {
                    match std::fs::read_to_string(&main_file) {
                        Ok(s) => {
                            eprintln!("  Found {}", main_file.display());
                            s
                        }
                        Err(e) => {
                            eprintln!("  error: Cannot read '{}': {}", main_file.display(), e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("  hint: No main.aura found in '{}'", path);
                    std::process::exit(1);
                }
            } else {
                std::process::exit(1);
            }
        }
    };

    // Parse
    eprintln!("  [1/4] Parsing...");
    let parse_result = aura_core::parser::parse(&source);

    if !parse_result.errors.is_empty() {
        eprintln!("  {} error(s) found:", parse_result.errors.len());
        for err in &parse_result.errors {
            if use_json {
                print_error_json(err);
            } else {
                print_error_text(err, &source, path);
            }
        }
        if parse_result.program.is_none() {
            std::process::exit(1);
        }
    }

    let program = match parse_result.program {
        Some(p) => p,
        None => {
            eprintln!("  error: Failed to parse program");
            std::process::exit(1);
        }
    };

    // Semantic analysis
    eprintln!("  [2/4] Analyzing...");
    let analysis = aura_core::semantic::SemanticAnalyzer::new().analyze(&program);

    if !analysis.errors.is_empty() {
        let error_count = analysis.errors.iter().filter(|e| e.is_error()).count();
        let warning_count = analysis.errors.len() - error_count;
        if error_count > 0 {
            eprintln!("  {} error(s), {} warning(s):", error_count, warning_count);
        } else {
            eprintln!("  {} warning(s):", warning_count);
        }
        for err in &analysis.errors {
            if use_json {
                print_error_json(err);
            } else {
                print_error_text(err, &source, path);
            }
        }
    }

    // Build HIR
    eprintln!("  [3/4] Building IR...");
    let hir = aura_core::hir::build_hir(&program);

    // Codegen
    eprintln!("  [4/4] Generating {}...", target);
    match target {
        "web" => {
            let output = aura_backend_web::compile_to_web(&hir);

            // Write output files
            let out_path = Path::new(output_dir);
            std::fs::create_dir_all(out_path).expect("Failed to create output directory");

            std::fs::write(out_path.join("index.html"), &output.html)
                .expect("Failed to write index.html");
            std::fs::write(out_path.join("styles.css"), &output.css)
                .expect("Failed to write styles.css");
            std::fs::write(out_path.join("app.js"), &output.js)
                .expect("Failed to write app.js");

            eprintln!();
            eprintln!("  Build complete:");
            eprintln!("    {}/index.html  ({} bytes)", output_dir, output.html.len());
            eprintln!("    {}/styles.css  ({} bytes)", output_dir, output.css.len());
            eprintln!("    {}/app.js      ({} bytes)", output_dir, output.js.len());
            eprintln!();
            eprintln!("  Open {}/index.html in a browser to preview.", output_dir);
        }
        _ => {
            eprintln!("  error: Target '{}' not yet implemented", target);
            eprintln!("  Available targets: web");
            std::process::exit(1);
        }
    }
}

fn print_error_text(err: &aura_core::AuraError, source: &str, file: &str) {
    let severity = match err.severity {
        aura_core::Severity::Error => "error",
        aura_core::Severity::Warning => "warning",
        aura_core::Severity::Info => "info",
    };

    // Find line and column from byte offset
    let (line, col) = byte_to_line_col(source, err.span.start);

    eprintln!("  {}[{}]: {}", severity, err.code, err.message);
    eprintln!("    --> {}:{}:{}", file, line, col);

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
    for (i, ch) in source.char_indices() {
        if i >= byte_offset {
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

fn explain_command(file: &str) {
    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  error: Cannot read '{}': {}", file, e);
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
        Ok(s) => s,
        Err(e) => {
            eprintln!("  error: Cannot read '{}': {}", file_a, e);
            std::process::exit(1);
        }
    };
    let source_b = match std::fs::read_to_string(file_b) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  error: Cannot read '{}': {}", file_b, e);
            std::process::exit(1);
        }
    };

    let result_a = aura_core::parser::parse(&source_a);
    let result_b = aura_core::parser::parse(&source_b);

    let program_a = match result_a.program {
        Some(p) => p,
        None => {
            eprintln!("  error: Failed to parse '{}'", file_a);
            std::process::exit(1);
        }
    };
    let program_b = match result_b.program {
        Some(p) => p,
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
