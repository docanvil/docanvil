pub mod build;
pub mod color;
pub mod doctor;
pub mod export;
pub mod new;
pub mod serve;
pub mod theme;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::cli::export::ExportArgs;

#[derive(Clone, Debug, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable coloured output (default)
    #[default]
    Human,
    /// Checkstyle XML (compatible with reviewdog, GitHub Actions)
    Checkstyle,
    /// JUnit XML (compatible with test result reporters)
    Junit,
}

#[derive(Parser)]
#[command(
    name = "docanvil",
    about = "Forge beautiful static documentation from Markdown",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Enable verbose output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Scaffold a new documentation project
    New {
        /// Project name / directory
        name: String,
    },
    /// Start dev server with hot reload
    Serve {
        /// Host address to bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on
        #[arg(long, default_value_t = 3000)]
        port: u16,
        /// Path to the project root
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Diagnose project configuration and content issues
    Doctor {
        /// Automatically apply safe fixes
        #[arg(long)]
        fix: bool,
        /// Exit with code 1 if any warnings or errors are found (for CI)
        #[arg(long)]
        strict: bool,
        /// Path to the project root
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Output format for diagnostics
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Generate a custom color theme interactively
    Theme {
        /// Path to the project root
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Overwrite existing theme customizations
        #[arg(long)]
        overwrite: bool,
    },
    /// Build static HTML site
    Build {
        /// Output directory
        #[arg(long, default_value = "dist")]
        out: PathBuf,
        /// Remove output directory before building
        #[arg(long)]
        clean: bool,
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
        /// Path to the project root
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Export documentation to another format
    Export(ExportArgs),
}
