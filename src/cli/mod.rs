pub mod build;
pub mod init;
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
    Init {
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
    },
    /// Build static HTML site
    Build {
        /// Output directory
        #[arg(long, default_value = "dist")]
        out: PathBuf,
        /// Remove output directory before building
        #[arg(long)]
        clean: bool,
        /// Strict mode so warnings cause build failure
        #[arg(long)]
        strict: bool,
    },
}
