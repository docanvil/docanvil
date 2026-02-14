mod cli;
mod components;
mod config;
mod diagnostics;
mod error;
mod nav;
mod pipeline;
mod project;
mod render;
mod search;
mod seo;
mod server;
mod theme;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Command::New { name } => cli::new::run(name),
        Command::Serve { host, port, path } => cli::serve::run(host, *port, path),
        Command::Build { out, clean, strict, path } => cli::build::run(path, out, *clean, cli.quiet, *strict),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
