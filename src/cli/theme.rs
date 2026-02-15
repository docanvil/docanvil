use std::path::Path;

use dialoguer::{Input, Select};
use owo_colors::OwoColorize;

use super::color::{Rgb, darken, is_valid_hex, lighten, parse_hex, tint, to_hex, to_rgba};
use crate::config::Config;
use crate::error::Result;

/// Run the theme command: prompt for colors, generate CSS, update config.
pub fn run(project_root: &Path, overwrite: bool) -> Result<()> {
    let config_path = project_root.join("docanvil.toml");
    if !config_path.exists() {
        eprintln!(
            "{} No docanvil.toml found in {}",
            "error:".red().bold(),
            project_root.display()
        );
        eprintln!("  Run `docanvil init` first, or use `--path` to point to your project.");
        std::process::exit(1);
    }

    let config = Config::load(project_root)?;

    // Overwrite guard
    if !overwrite {
        let has_custom_css = config.theme.custom_css.is_some();
        let has_variables = !config.theme.variables.is_empty();
        if has_custom_css || has_variables {
            eprintln!(
                "{} Theme customizations already exist in docanvil.toml",
                "warning:".yellow().bold()
            );
            if has_custom_css {
                eprintln!(
                    "  custom_css = {:?}",
                    config.theme.custom_css.as_deref().unwrap_or("")
                );
            }
            if has_variables {
                eprintln!(
                    "  [theme.variables] has {} entries",
                    config.theme.variables.len()
                );
            }
            eprintln!();
            eprintln!("  Use {} to replace them.", "--overwrite".bold());
            std::process::exit(0);
        }
    }

    // Prompt for color mode
    let color_mode_options = &["Light only", "Dark only", "Both (light + dark with toggle)"];
    let color_mode_idx = Select::new()
        .with_prompt("Color mode")
        .items(color_mode_options)
        .default(0)
        .interact()
        .unwrap_or(0);

    let color_mode = match color_mode_idx {
        0 => "light",
        1 => "dark",
        _ => "both",
    };

    // Prompt for light colors (if light or both)
    let (light_primary_hex, light_secondary_hex, light_primary, light_secondary) =
        if color_mode != "dark" {
            let label = if color_mode == "both" {
                "Light mode primary"
            } else {
                "Primary color"
            };
            let (ph, sh, p, s) = prompt_colors(label, "#6366f1", "#f97316");
            (Some(ph), Some(sh), Some(p), Some(s))
        } else {
            (None, None, None, None)
        };

    // Prompt for dark colors (if dark or both)
    let (dark_primary_hex, dark_secondary_hex, dark_primary, dark_secondary) =
        if color_mode != "light" {
            let label = if color_mode == "both" {
                "Dark mode primary"
            } else {
                "Primary color"
            };
            let default_primary = "#818cf8";
            let (ph, sh, p, s) = prompt_colors(label, default_primary, "#f97316");
            (Some(ph), Some(sh), Some(p), Some(s))
        } else {
            (None, None, None, None)
        };

    generate_theme(
        project_root,
        &config,
        color_mode,
        light_primary_hex,
        light_secondary_hex,
        light_primary,
        light_secondary,
        dark_primary_hex,
        dark_secondary_hex,
        dark_primary,
        dark_secondary,
        overwrite,
    )
}

/// Prompt for primary + secondary hex colors with the given label prefix.
fn prompt_colors(
    label: &str,
    default_primary: &str,
    default_secondary: &str,
) -> (String, String, Rgb, Rgb) {
    let primary_input: String = Input::new()
        .with_prompt(format!("{label} color (hex)"))
        .default(default_primary.into())
        .validate_with(|input: &String| {
            if is_valid_hex(input) {
                Ok(())
            } else {
                Err("Enter a valid hex color, e.g. #6366f1")
            }
        })
        .interact_text()
        .unwrap_or_else(|_| default_primary.into());

    let secondary_input: String = Input::new()
        .with_prompt("Warning/secondary color (hex)")
        .default(default_secondary.into())
        .validate_with(|input: &String| {
            if is_valid_hex(input) {
                Ok(())
            } else {
                Err("Enter a valid hex color, e.g. #f97316")
            }
        })
        .interact_text()
        .unwrap_or_else(|_| default_secondary.into());

    let primary = parse_hex(&primary_input).expect("validated above");
    let secondary = parse_hex(&secondary_input).expect("validated above");

    (primary_input, secondary_input, primary, secondary)
}

/// Build the CSS variable block for a light palette.
fn light_palette_css(primary: &Rgb, secondary: &Rgb) -> String {
    let primary_light = lighten(primary, 0.10);
    let link_hover = darken(primary, 0.10);
    let sidebar_hover = tint(primary, 0.95);
    let sidebar_active_bg = tint(primary, 0.95);
    let sidebar_active_text = darken(primary, 0.10);
    let note_bg = tint(primary, 0.95);
    let note_border = lighten(primary, 0.10);
    let warning_bg = tint(secondary, 0.95);

    format!(
        "  /* Primary palette */
  --color-primary: {primary};
  --color-primary-light: {primary_light};
  --color-link: {primary};
  --color-link-hover: {link_hover};

  /* Sidebar */
  --color-sidebar-hover: {sidebar_hover};
  --color-sidebar-active-bg: {sidebar_active_bg};
  --color-sidebar-active-text: {sidebar_active_text};

  /* Admonitions */
  --color-note-bg: {note_bg};
  --color-note-border: {note_border};
  --color-warning-border: {warning_border};
  --color-warning-bg: {warning_bg};

  /* Accents */
  --color-mark-bg: {mark_bg};
  --nav-group-toggle-hover: {nav_toggle_hover};
  --color-focus-ring: {focus_ring};",
        primary = to_hex(primary),
        primary_light = to_hex(&primary_light),
        link_hover = to_hex(&link_hover),
        sidebar_hover = to_hex(&sidebar_hover),
        sidebar_active_bg = to_hex(&sidebar_active_bg),
        sidebar_active_text = to_hex(&sidebar_active_text),
        note_bg = to_hex(&note_bg),
        note_border = to_hex(&note_border),
        warning_border = to_hex(secondary),
        warning_bg = to_hex(&warning_bg),
        mark_bg = to_rgba(primary, 0.12),
        nav_toggle_hover = to_rgba(primary, 0.06),
        focus_ring = to_rgba(primary, 0.4),
    )
}

/// Build the CSS variable block for a dark palette.
fn dark_palette_css(primary: &Rgb, secondary: &Rgb) -> String {
    let primary_light = lighten(primary, 0.10);
    let link_hover = lighten(primary, 0.10);
    let sidebar_hover = tint(primary, 0.15);
    let sidebar_active_bg = tint(primary, 0.15);
    let sidebar_active_text = lighten(primary, 0.10);
    let note_bg = tint(primary, 0.15);
    let note_border = lighten(primary, 0.10);
    let warning_bg = tint(secondary, 0.15);

    format!(
        "  /* Dark backgrounds */
  --color-bg: #0f172a;
  --color-bg-secondary: #1e293b;
  --color-text: #f1f5f9;
  --color-text-muted: #94a3b8;
  --color-border: #334155;
  --color-code-bg: #1e293b;

  /* Primary palette */
  --color-primary: {primary};
  --color-primary-light: {primary_light};
  --color-link: {primary};
  --color-link-hover: {link_hover};

  /* Sidebar */
  --color-sidebar-hover: {sidebar_hover};
  --color-sidebar-active-bg: {sidebar_active_bg};
  --color-sidebar-active-text: {sidebar_active_text};

  /* Admonitions */
  --color-note-bg: {note_bg};
  --color-note-border: {note_border};
  --color-warning-border: {warning_border};
  --color-warning-bg: {warning_bg};

  /* Accents */
  --color-mark-bg: {mark_bg};
  --nav-group-toggle-hover: {nav_toggle_hover};
  --color-focus-ring: {focus_ring};",
        primary = to_hex(primary),
        primary_light = to_hex(&primary_light),
        link_hover = to_hex(&link_hover),
        sidebar_hover = to_hex(&sidebar_hover),
        sidebar_active_bg = to_hex(&sidebar_active_bg),
        sidebar_active_text = to_hex(&sidebar_active_text),
        note_bg = to_hex(&note_bg),
        note_border = to_hex(&note_border),
        warning_border = to_hex(secondary),
        warning_bg = to_hex(&warning_bg),
        mark_bg = to_rgba(primary, 0.2),
        nav_toggle_hover = to_rgba(primary, 0.12),
        focus_ring = to_rgba(primary, 0.4),
    )
}

/// Core logic extracted for testability.
#[allow(clippy::too_many_arguments)]
fn generate_theme(
    project_root: &Path,
    config: &Config,
    color_mode: &str,
    light_primary_hex: Option<String>,
    light_secondary_hex: Option<String>,
    light_primary: Option<Rgb>,
    light_secondary: Option<Rgb>,
    dark_primary_hex: Option<String>,
    dark_secondary_hex: Option<String>,
    dark_primary: Option<Rgb>,
    dark_secondary: Option<Rgb>,
    overwrite: bool,
) -> Result<()> {
    let css = match color_mode {
        "light" => {
            let p = light_primary.unwrap();
            let s = light_secondary.unwrap();
            let ph = light_primary_hex.as_deref().unwrap();
            let sh = light_secondary_hex.as_deref().unwrap();
            format!(
                "/* DocAnvil custom theme — generated by `docanvil theme`\n\
                 *\n\
                 * Mode:      light\n\
                 * Primary:   {ph}\n\
                 * Secondary: {sh}\n\
                 */\n\n\
                 :root {{\n{vars}\n}}\n",
                vars = light_palette_css(&p, &s),
            )
        }
        "dark" => {
            let p = dark_primary.unwrap();
            let s = dark_secondary.unwrap();
            let ph = dark_primary_hex.as_deref().unwrap();
            let sh = dark_secondary_hex.as_deref().unwrap();
            format!(
                "/* DocAnvil custom theme — generated by `docanvil theme`\n\
                 *\n\
                 * Mode:      dark\n\
                 * Primary:   {ph}\n\
                 * Secondary: {sh}\n\
                 */\n\n\
                 :root {{\n{vars}\n}}\n",
                vars = dark_palette_css(&p, &s),
            )
        }
        _ => {
            // "both"
            let lp = light_primary.unwrap();
            let ls = light_secondary.unwrap();
            let dp = dark_primary.unwrap();
            let ds = dark_secondary.unwrap();
            let lph = light_primary_hex.as_deref().unwrap();
            let lsh = light_secondary_hex.as_deref().unwrap();
            let dph = dark_primary_hex.as_deref().unwrap();
            let dsh = dark_secondary_hex.as_deref().unwrap();
            format!(
                "/* DocAnvil custom theme — generated by `docanvil theme`\n\
                 *\n\
                 * Mode:            both (light + dark)\n\
                 * Light primary:   {lph}\n\
                 * Light secondary: {lsh}\n\
                 * Dark primary:    {dph}\n\
                 * Dark secondary:  {dsh}\n\
                 */\n\n\
                 /* Light mode (default) */\n\
                 :root {{\n{light_vars}\n}}\n\n\
                 /* Dark mode — explicit toggle */\n\
                 [data-theme=\"dark\"] {{\n{dark_vars}\n}}\n\n\
                 /* Dark mode — OS preference (when no explicit choice) */\n\
                 @media (prefers-color-scheme: dark) {{\n\
                 \x20 :root:not([data-theme=\"light\"]) {{\n{dark_vars_indented}\n\x20 }}\n\
                 }}\n",
                light_vars = light_palette_css(&lp, &ls),
                dark_vars = dark_palette_css(&dp, &ds),
                dark_vars_indented = indent(&dark_palette_css(&dp, &ds), "  "),
            )
        }
    };

    // Determine output path
    let css_rel = config
        .theme
        .custom_css
        .as_deref()
        .unwrap_or("theme/custom.css");
    let css_path = project_root.join(css_rel);

    // Ensure parent directory exists
    if let Some(parent) = css_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&css_path, css)?;

    // Update docanvil.toml using toml_edit to preserve formatting
    update_config(
        &project_root.join("docanvil.toml"),
        css_rel,
        color_mode,
        overwrite,
    )?;

    // Print success
    eprintln!();
    eprintln!("{}", "Theme generated successfully!".green().bold());
    eprintln!();
    eprintln!("  Mode:      {}", color_mode.bold());
    match color_mode {
        "light" => {
            eprintln!(
                "  Primary:   {}",
                light_primary_hex.as_deref().unwrap().bold()
            );
            eprintln!(
                "  Secondary: {}",
                light_secondary_hex.as_deref().unwrap().bold()
            );
        }
        "dark" => {
            eprintln!(
                "  Primary:   {}",
                dark_primary_hex.as_deref().unwrap().bold()
            );
            eprintln!(
                "  Secondary: {}",
                dark_secondary_hex.as_deref().unwrap().bold()
            );
        }
        _ => {
            eprintln!(
                "  Light:     {} / {}",
                light_primary_hex.as_deref().unwrap().bold(),
                light_secondary_hex.as_deref().unwrap().bold()
            );
            eprintln!(
                "  Dark:      {} / {}",
                dark_primary_hex.as_deref().unwrap().bold(),
                dark_secondary_hex.as_deref().unwrap().bold()
            );
        }
    }
    eprintln!("  CSS:       {}", css_rel.dimmed());
    eprintln!();
    eprintln!("  Run {} to preview your theme.", "docanvil serve".bold());

    Ok(())
}

/// Indent each line of `text` by prepending `prefix`.
fn indent(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Update docanvil.toml: set custom_css, color_mode, and optionally remove [theme.variables].
fn update_config(
    config_path: &Path,
    css_rel: &str,
    color_mode: &str,
    overwrite: bool,
) -> Result<()> {
    let contents = std::fs::read_to_string(config_path)?;
    let mut doc = contents
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| crate::error::Error::Render(format!("failed to parse docanvil.toml: {e}")))?;

    // Ensure [theme] table exists
    if doc.get("theme").is_none() {
        doc["theme"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    let theme = doc["theme"].as_table_mut().unwrap();

    // Set custom_css if not already set (or if overwriting)
    if overwrite || theme.get("custom_css").is_none() {
        theme["custom_css"] = toml_edit::value(css_rel);
    }

    // Always set color_mode
    theme["color_mode"] = toml_edit::value(color_mode);

    // If overwriting, remove [theme.variables]
    if overwrite {
        theme.remove("variables");
    }

    std::fs::write(config_path, doc.to_string())?;

    Ok(())
}
