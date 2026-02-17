mod cli;
mod components;
mod config;
mod diagnostics;
mod doctor;
mod error;
mod nav;
mod pipeline;
mod project;
mod render;
mod search;
mod seo;
mod server;
mod theme;
mod util;

use clap::Parser;
use cli::{Cli, Command};
use owo_colors::OwoColorize;

fn main() {
    // Exit code 5 for panics (internal error / bug).
    std::panic::set_hook(Box::new(|info| {
        eprintln!("error: internal error (this is a bug)");
        eprintln!("{info}");
        eprintln!();
        eprintln!("Please report this at https://github.com/docanvil/docanvil/issues");
        std::process::exit(5);
    }));

    let cli = Cli::parse();

    let result = match &cli.command {
        Command::New { name } => cli::new::run(name),
        Command::Doctor { fix, strict, path } => cli::doctor::run(path, *fix, *strict, cli.quiet),
        Command::Theme { path, overwrite } => cli::theme::run(path, *overwrite),
        Command::Serve { host, port, path } => cli::serve::run(host, *port, path),
        Command::Build {
            out,
            clean,
            strict,
            path,
        } => cli::build::run(path, out, *clean, cli.quiet, *strict),
    };

    if let Err(e) = result {
        eprintln!("{}: {e}", "error".red().bold());
        if let Some(hint) = e.hint() {
            eprintln!("  {}: {hint}", "hint".dimmed());
        }
        std::process::exit(e.exit_code());
    }
}
