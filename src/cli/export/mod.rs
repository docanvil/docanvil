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
        /// Locale to export (for i18n projects; defaults to the configured default locale)
        #[arg(long)]
        locale: Option<String>,
    },
}

/// Dispatch to the appropriate export format handler.
pub fn dispatch(args: &ExportArgs, quiet: bool) -> Result<()> {
    match &args.format {
        ExportFormat::Pdf { out, path, locale } => pdf::run(path, out, locale.as_deref(), quiet),
    }
}
