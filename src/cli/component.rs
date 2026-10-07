use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

use crate::components::ComponentRegistry;
use crate::components::templates::{builtin_names, builtin_source};
use crate::error::{Error, Result};

const DOCS_URL: &str =
    "https://docanvil.github.io/docanvil/en/writing/components.html#custom-components";

/// Arguments for the `docanvil component` subcommand.
#[derive(Args)]
pub struct ComponentArgs {
    #[command(subcommand)]
    pub action: ComponentAction,
}

#[derive(Subcommand)]
pub enum ComponentAction {
    /// List built-in and custom components
    List {
        /// Path to the project root
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Copy a built-in component's template into theme/components/ to customise it
    Eject {
        /// Built-in component(s) to eject, e.g. `note` or `tabs`
        names: Vec<String>,
        /// Eject every built-in component
        #[arg(long)]
        all: bool,
        /// Overwrite templates that already exist
        #[arg(long)]
        force: bool,
        /// Path to the project root
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

pub fn dispatch(args: &ComponentArgs, quiet: bool) -> Result<()> {
    match &args.action {
        ComponentAction::List { path } => list(path),
        ComponentAction::Eject {
            names,
            all,
            force,
            path,
        } => eject(path, names, *all, *force, quiet),
    }
}

/// Stems of `theme/components/*.html`, sorted.
fn user_template_names(project_root: &Path) -> Vec<String> {
    let dir = ComponentRegistry::user_template_dir(project_root);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "html"))
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .collect();
    names.sort();
    names
}

fn list(project_root: &Path) -> Result<()> {
    let builtins = builtin_names();
    let user = user_template_names(project_root);
    let width = builtins
        .iter()
        .chain(&user)
        .map(String::len)
        .max()
        .unwrap_or(0);

    println!("{}", "Built-in components".bold());
    for name in &builtins {
        if user.contains(name) {
            println!(
                "  {name:<width$}  {} theme/components/{name}.html",
                "overridden →".yellow()
            );
        } else {
            println!("  {name:<width$}  {}", "builtin".dimmed());
        }
    }

    let custom: Vec<&String> = user.iter().filter(|n| !builtins.contains(n)).collect();
    if !custom.is_empty() {
        println!();
        println!("{}", "Custom components".bold());
        for name in custom {
            println!("  {name:<width$}  theme/components/{name}.html");
        }
    }
    Ok(())
}

fn eject(project_root: &Path, names: &[String], all: bool, force: bool, quiet: bool) -> Result<()> {
    let builtins = builtin_names();
    let available = builtins.join(", ");

    let selected: Vec<String> = if all {
        builtins.clone()
    } else if names.is_empty() {
        return Err(Error::General(format!(
            "name a component to eject, or pass --all. Built-in components: {available}"
        )));
    } else {
        names.to_vec()
    };

    if let Some(unknown) = selected.iter().find(|n| !builtins.contains(n)) {
        return Err(Error::General(format!(
            "'{unknown}' isn't a built-in component. Built-in components: {available}"
        )));
    }

    let dir = ComponentRegistry::user_template_dir(project_root);
    std::fs::create_dir_all(&dir)?;

    for name in &selected {
        let target = dir.join(format!("{name}.html"));
        let display = format!("theme/components/{name}.html");
        if target.exists() && !force {
            eprintln!(
                "{}: skipped {display} — it already exists (pass --force to overwrite)",
                "warning".yellow().bold()
            );
            continue;
        }
        let source = builtin_source(name).unwrap_or_default();
        std::fs::write(&target, source)?;
        if !quiet {
            eprintln!("{} Ejected {} → {display}", "✓".green().bold(), name.bold());
        }
    }

    if !quiet {
        eprintln!(
            "  {}: edit the template to restyle the component — the comment at the top lists its variables. Docs: {DOCS_URL}",
            "next".dimmed()
        );
    }
    Ok(())
}
