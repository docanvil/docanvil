pub mod cdp;
pub mod pdf;

use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::error::Result;

/// Arguments for the `docanvil export` subcommand.
#[derive(Args)]
pub struct ExportArgs {
    #[command(subcommand)]
    pub format: ExportFormat,
}

/// Supported export formats.
#[derive(Subcommand)]
pub enum ExportFormat {
    /// Export documentation as a single PDF file
    Pdf {
        /// Output PDF file path
        #[arg(long)]
        out: PathBuf,
        /// Path to the project root
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Locale to export, or all; prompts interactively, otherwise uses the project default
        #[arg(long)]
        locale: Option<String>,
        /// Documentation version; prompts interactively, otherwise uses the current version
        #[arg(long = "doc-version")]
        version: Option<String>,
    },
}

/// Dispatch to the appropriate export format handler.
pub fn dispatch(args: &ExportArgs, quiet: bool) -> Result<()> {
    match &args.format {
        ExportFormat::Pdf {
            out,
            path,
            locale,
            version,
        } => pdf::run_with_version(path, out, locale.as_deref(), version.as_deref(), quiet),
    }
}
