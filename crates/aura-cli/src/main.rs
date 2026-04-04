//! # Aura CLI
//!
//! The main command-line interface for the Aura language.
//!
//! ## Commands
//! - `aura build` — compile .aura files to target platforms
//! - `aura run` — build and run with live preview
//! - `aura test` — run test suites
//! - `aura fmt` — format .aura files
//! - `aura explain` — convert code to plain English
//! - `aura diff` — semantic diff between versions
//! - `aura design` — preview design system
//! - `aura playground` — browser-based live editor
//! - `aura pkg` — package management
//! - `aura init` — scaffold a new project
//! - `aura doctor` — diagnose environment issues

use clap::{Parser, Subcommand};

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
        /// Target platform(s): web, ios, android, windows, tui, all
        #[arg(short, long, default_value = "web")]
        target: String,

        /// Source directory
        #[arg(default_value = "src")]
        path: String,

        /// Output directory
        #[arg(short, long, default_value = "build")]
        output: String,

        /// Emit errors in JSON format for AI agents
        #[arg(long)]
        format: Option<String>,
    },

    /// Build and run with live preview
    Run {
        /// Target platform for preview
        #[arg(short, long, default_value = "web")]
        target: String,

        /// Preview all platforms side by side
        #[arg(long)]
        preview: Option<String>,

        /// Port for dev server
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// Format .aura source files
    Fmt {
        /// Files or directories to format
        #[arg(default_value = "src")]
        path: String,

        /// Check formatting without modifying files
        #[arg(long)]
        check: bool,
    },

    /// Convert .aura code to plain English description
    Explain {
        /// File to explain
        file: String,
    },

    /// Semantic diff between two .aura files or versions
    Diff {
        /// First file or git ref
        a: String,
        /// Second file or git ref
        b: String,
    },

    /// Scaffold a new Aura project
    Init {
        /// Project name
        name: String,

        /// Template: app, component, theme
        #[arg(short, long, default_value = "app")]
        template: String,
    },

    /// Diagnose environment issues
    Doctor,

    /// Generate a running prototype from a description
    Sketch {
        /// Natural language description of the app
        description: String,
    },

    /// Package management
    Pkg {
        #[command(subcommand)]
        action: PkgCommands,
    },
}

#[derive(Subcommand)]
enum PkgCommands {
    /// Install a package
    Install { package: String },
    /// Update packages
    Update { package: Option<String> },
    /// Remove a package
    Remove { package: String },
    /// Publish the current package
    Publish,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { target, path, output, format } => {
            println!("  Aura v{}", env!("CARGO_PKG_VERSION"));
            println!("  Building {} → {}", path, target);
            println!();
            println!("  Phase 1: Compiling...");
            // TODO: Implement compilation pipeline
            println!("  Build target: {}", target);
            println!("  Output: {}/", output);
            if let Some(fmt) = format {
                println!("  Error format: {}", fmt);
            }
            println!();
            println!("  (Compiler not yet implemented — see spec/language.md)");
        }
        Commands::Run { target, preview, port } => {
            println!("  Aura Dev Server");
            println!("  Target: {}", target);
            if let Some(p) = preview {
                println!("  Preview: {}", p);
            }
            println!("  Port: {}", port);
            println!();
            println!("  (Dev server not yet implemented)");
        }
        Commands::Fmt { path, check } => {
            if check {
                println!("  Checking formatting: {}", path);
            } else {
                println!("  Formatting: {}", path);
            }
        }
        Commands::Explain { file } => {
            println!("  Explaining: {}", file);
            println!("  (aura explain not yet implemented)");
        }
        Commands::Diff { a, b } => {
            println!("  Semantic diff: {} → {}", a, b);
            println!("  (aura diff not yet implemented)");
        }
        Commands::Init { name, template } => {
            println!("  Creating new Aura project: {}", name);
            println!("  Template: {}", template);
            println!("  (aura init not yet implemented)");
        }
        Commands::Doctor => {
            println!("  Aura Doctor");
            println!("  Checking environment...");
            println!("  (aura doctor not yet implemented)");
        }
        Commands::Sketch { description } => {
            println!("  Aura Sketch");
            println!("  Description: {}", description);
            println!("  (aura sketch not yet implemented)");
        }
        Commands::Pkg { action } => {
            match action {
                PkgCommands::Install { package } => println!("  Installing: {}", package),
                PkgCommands::Update { package } => println!("  Updating: {}", package.unwrap_or_else(|| "all".to_string())),
                PkgCommands::Remove { package } => println!("  Removing: {}", package),
                PkgCommands::Publish => println!("  Publishing..."),
            }
        }
    }
}
