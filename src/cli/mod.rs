pub mod build;
pub mod doctor;
pub mod new;
pub mod serve;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
}
