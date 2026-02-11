mod cli;
mod components;
mod config;
mod diagnostics;
mod error;
mod nav;
mod pipeline;
mod project;
mod render;
mod server;
mod theme;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Command::Init { name } => cli::init::run(name),
        Command::Serve { host, port } => cli::serve::run(host, *port),
        Command::Build { out, clean } => cli::build::run(out, *clean, cli.quiet),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
