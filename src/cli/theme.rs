use std::path::Path;

use dialoguer::{Input, Select};
use owo_colors::OwoColorize;

use super::color::{Rgb, darken, is_valid_hex, lighten, parse_hex, tint, to_hex, to_rgba};
use crate::config::Config;
use crate::error::Result;

/// Paired hex strings and parsed RGB values for a color palette.
struct ThemeColors {
    primary_hex: String,
    secondary_hex: String,
    primary: Rgb,
    secondary: Rgb,
}

/// Resolved color mode with associated palettes — no Option unwraps needed.
enum ThemeMode {
    Light(ThemeColors),
    Dark(ThemeColors),
    Both {
        light: ThemeColors,
        dark: ThemeColors,
    },
}

impl ThemeMode {
    fn color_mode_str(&self) -> &str {
        match self {
            ThemeMode::Light(_) => "light",
            ThemeMode::Dark(_) => "dark",
            ThemeMode::Both { .. } => "both",
        }
    }
}

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
        return Err(crate::error::Error::ConfigNotFound(config_path));
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
            return Ok(());
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

    // Prompt for colors based on mode
    let mode = match color_mode {
        "light" => ThemeMode::Light(prompt_colors("Primary", "#6366f1", "#f97316")?),
        "dark" => ThemeMode::Dark(prompt_colors("Primary", "#818cf8", "#f97316")?),
        _ => ThemeMode::Both {
            light: prompt_colors("Light mode primary", "#6366f1", "#f97316")?,
            dark: prompt_colors("Dark mode primary", "#818cf8", "#f97316")?,
        },
    };

    generate_theme(project_root, &config, mode, overwrite)
}

/// Prompt for primary + secondary hex colors with the given label prefix.
fn prompt_colors(
    label: &str,
    default_primary: &str,
    default_secondary: &str,
) -> Result<ThemeColors> {
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

    let primary = parse_hex(&primary_input).ok_or_else(|| {
        crate::error::Error::General(format!("invalid primary color: {primary_input}"))
    })?;
    let secondary = parse_hex(&secondary_input).ok_or_else(|| {
        crate::error::Error::General(format!("invalid secondary color: {secondary_input}"))
    })?;

    Ok(ThemeColors {
        primary_hex: primary_input,
        secondary_hex: secondary_input,
        primary,
        secondary,
    })
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
fn generate_theme(
    project_root: &Path,
    config: &Config,
    mode: ThemeMode,
    overwrite: bool,
) -> Result<()> {
    let css = match &mode {
        ThemeMode::Light(c) => {
            format!(
                "/* DocAnvil custom theme — generated by `docanvil theme`\n\
                 *\n\
                 * Mode:      light\n\
                 * Primary:   {ph}\n\
                 * Secondary: {sh}\n\
                 */\n\n\
                 :root {{\n{vars}\n}}\n",
                ph = c.primary_hex,
                sh = c.secondary_hex,
                vars = light_palette_css(&c.primary, &c.secondary),
            )
        }
        ThemeMode::Dark(c) => {
            format!(
                "/* DocAnvil custom theme — generated by `docanvil theme`\n\
                 *\n\
                 * Mode:      dark\n\
                 * Primary:   {ph}\n\
                 * Secondary: {sh}\n\
                 */\n\n\
                 :root {{\n{vars}\n}}\n",
                ph = c.primary_hex,
                sh = c.secondary_hex,
                vars = dark_palette_css(&c.primary, &c.secondary),
            )
        }
        ThemeMode::Both {
            light: lc,
            dark: dc,
        } => {
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
                lph = lc.primary_hex,
                lsh = lc.secondary_hex,
                dph = dc.primary_hex,
                dsh = dc.secondary_hex,
                light_vars = light_palette_css(&lc.primary, &lc.secondary),
                dark_vars = dark_palette_css(&dc.primary, &dc.secondary),
                dark_vars_indented = indent(&dark_palette_css(&dc.primary, &dc.secondary), "  "),
            )
        }
    };

    let color_mode = mode.color_mode_str();

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
    match &mode {
        ThemeMode::Light(c) | ThemeMode::Dark(c) => {
            eprintln!("  Primary:   {}", c.primary_hex.bold());
            eprintln!("  Secondary: {}", c.secondary_hex.bold());
        }
        ThemeMode::Both {
            light: lc,
            dark: dc,
        } => {
            eprintln!(
                "  Light:     {} / {}",
                lc.primary_hex.bold(),
                lc.secondary_hex.bold()
            );
            eprintln!(
                "  Dark:      {} / {}",
                dc.primary_hex.bold(),
                dc.secondary_hex.bold()
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

    let theme = doc["theme"].as_table_mut().ok_or_else(|| {
        crate::error::Error::Render(
            "[theme] in docanvil.toml must be a table, not a scalar value".to_string(),
        )
    })?;

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
