//! Portable specification artifacts, kept separate from searchable reference pages.
use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::openapi::{Api, Location};
use crate::project::PageInventory;

pub(super) struct Download {
    pub name: String,
    pub bytes: Arc<[u8]>,
}

impl Download {
    pub fn format(&self) -> &'static str {
        match self.name.rsplit('.').next() {
            Some("zip") => "ZIP",
            Some("yaml" | "yml") => "YAML",
            _ => "JSON",
        }
    }
}

fn error(message: impl std::fmt::Display) -> Error {
    Error::General(format!("Cannot prepare OpenAPI download: {message}"))
}

pub(super) fn plan(api: &Api) -> Result<Download> {
    let graph = &api.graph;
    // A file-based self-reference can depend on its original directory spelling.
    // Bundle those too, so even aliased/symlinked references become portable.
    if graph.dependencies().count() == 1 && graph.references().all(|r| r.spelling.starts_with('#'))
    {
        let extension = graph
            .root
            .file
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_ascii_lowercase();
        return Ok(Download {
            name: format!("openapi.{extension}"),
            bytes: Arc::from(graph.source(&graph.root.file).unwrap().as_bytes()),
        });
    }
    let mut names = BTreeMap::from([(graph.root.file.clone(), "openapi.json".to_owned())]);
    for (index, file) in graph
        .dependencies()
        .filter(|file| *file != graph.root.file)
        .enumerate()
    {
        names.insert(file.to_owned(), format!("references/{:03}.json", index + 1));
    }
    let mut documents = BTreeMap::new();
    for file in names.keys() {
        let value = graph
            .value(&Location {
                file: file.clone(),
                pointer: String::new(),
            })
            .map_err(error)?
            .clone();
        documents.insert(file.clone(), value);
    }
    for reference in graph.references() {
        let source_name = &names[&reference.source.file];
        let target_name = &names[&reference.target.file];
        let file = if reference.source.file == reference.target.file {
            String::new()
        } else if source_name == "openapi.json" {
            target_name.clone()
        } else if target_name == "openapi.json" {
            "../openapi.json".into()
        } else {
            target_name.strip_prefix("references/").unwrap().to_owned()
        };
        let fragment = reference
            .target
            .pointer
            .bytes()
            .map(|byte| {
                if byte.is_ascii_alphanumeric() || b"/-._~".contains(&byte) {
                    (byte as char).to_string()
                } else {
                    format!("%{byte:02X}")
                }
            })
            .collect::<String>();
        *documents
            .get_mut(&reference.source.file)
            .unwrap()
            .pointer_mut(&reference.source.pointer)
            .unwrap() = format!("{file}#{fragment}").into();
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default())
        .unix_permissions(0o644);
    // Entry order and metadata are deterministic; source paths never become names.
    let mut entries: Vec<_> = names.iter().collect();
    entries.sort_by(|a, b| a.1.cmp(b.1));
    for (file, name) in entries {
        writer.start_file(name, options).map_err(error)?;
        let bytes = serde_json::to_vec_pretty(&documents[file]).map_err(error)?;
        writer.write_all(&bytes)?;
        writer.write_all(b"\n")?;
    }
    Ok(Download {
        name: "openapi.zip".into(),
        bytes: Arc::from(writer.finish().map_err(error)?.into_inner()),
    })
}

fn overlaps(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

/// Reject file/directory conflicts as well as exact file collisions, in build/doctor.
pub(crate) fn validate_assets(
    inventory: &PageInventory,
    root: &Path,
    prefix: Option<&str>,
    config: &Config,
) -> Result<()> {
    if inventory.generated_files.is_empty() {
        return Ok(());
    }
    let mut assets: Vec<PathBuf> = crate::render::assets::static_output_paths(root)?
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let mut directories = Vec::new();
    let static_dir = root.join("static");
    if static_dir.is_dir() {
        for entry in walkdir::WalkDir::new(&static_dir).follow_links(true) {
            let entry = entry.map_err(|e| Error::General(e.to_string()))?;
            if entry.file_type().is_dir() {
                directories.push(entry.path().strip_prefix(&static_dir).unwrap().to_owned());
            }
        }
    }
    let asset_dir = root.join("assets");
    if asset_dir.is_dir() {
        for entry in walkdir::WalkDir::new(&asset_dir).follow_links(true) {
            let entry = entry.map_err(|e| Error::General(e.to_string()))?;
            if entry.file_type().is_file() {
                assets.push(entry.path().strip_prefix(root).unwrap().to_owned());
            } else if entry.file_type().is_dir() {
                directories.push(entry.path().strip_prefix(root).unwrap().to_owned());
            }
        }
    }
    if prefix.is_none()
        && let Some(css) = &config.theme.custom_css
        && root.join(css).is_file()
    {
        assets.push(PathBuf::from(css));
    }
    for artifact in inventory.generated_files.keys() {
        for page in inventory.pages.values().chain(inventory.drafts.values()) {
            if overlaps(artifact, &page.output_path) {
                return Err(Error::General(format!(
                    "OpenAPI download {} collides with page {}",
                    artifact.display(),
                    page.output_path.display()
                )));
            }
        }
        for directory in &directories {
            let path = prefix
                .map(|prefix| Path::new(prefix).join(directory))
                .unwrap_or_else(|| directory.clone());
            // Parent directories are shared normally; a directory occupying a
            // download's filename (even empty) cannot coexist with the file.
            if path.starts_with(artifact) {
                return Err(Error::General(format!(
                    "OpenAPI download {} collides with asset directory {}",
                    artifact.display(),
                    path.display()
                )));
            }
        }
        for asset in &assets {
            let path = prefix
                .map(|prefix| Path::new(prefix).join(asset))
                .unwrap_or_else(|| asset.clone());
            if overlaps(artifact, &path) {
                return Err(Error::General(format!(
                    "OpenAPI download {} collides with asset {}",
                    artifact.display(),
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

pub(crate) fn write(inventory: &PageInventory, output: &Path) -> Result<()> {
    for (path, bytes) in &inventory.generated_files {
        let path = output.join(path);
        if path.exists() {
            return Err(Error::General(format!(
                "OpenAPI download would overwrite {}",
                path.display()
            )));
        }
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn single_document_keeps_original_bytes_from_the_loaded_input() {
        for (extension, source) in [
            (
                "json",
                "{\n  \"openapi\":\"3.1.0\", \"info\":{\"title\":\"Test\",\"version\":\"1\"}, \"paths\":{}\n}\n",
            ),
            (
                "yaml",
                "# Keep this comment\nopenapi: 3.1.0\ninfo: {title: Test, version: '1'}\npaths: {}\n",
            ),
            (
                "yml",
                "openapi: 3.0.3\ninfo: {title: Test, version: '1'}\npaths: {}\n",
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join(format!("source.{extension}"));
            std::fs::write(&path, source).unwrap();
            let api = crate::openapi::load(&path).unwrap();
            std::fs::write(&path, "Changed after loading").unwrap();
            let download = plan(&api).unwrap();
            assert_eq!(download.name, format!("openapi.{extension}"));
            assert_eq!(download.bytes.as_ref(), source.as_bytes());
        }
    }

    fn extract(bytes: &[u8], root: &Path) -> Vec<String> {
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut names = Vec::new();
        for index in 0..zip.len() {
            let mut entry = zip.by_index(index).unwrap();
            let relative = entry.enclosed_name().unwrap();
            assert!(!relative.is_absolute());
            let path = root.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::io::copy(&mut entry, &mut std::fs::File::create(&path).unwrap()).unwrap();
            names.push(entry.name().unwrap().into_owned());
        }
        names
    }

    #[test]
    fn archive_is_deterministic_portable_and_rewrites_only_real_references() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("private-name/api");
        let shared = dir.path().join("private-name/shared");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&shared).unwrap();
        let root = json!({"openapi":"3.1.0","info":{"title":"Test","version":"1"},"paths":{},"components":{"schemas":{
            "Pet":{"$ref":"../shared/pet.yaml"}, "Other":{"$ref":"other.json#/type"}
        }}});
        // A boolean schema in a referenced JSON Pointer exercises non-object documents/targets.
        std::fs::write(source.join("other.json"), r#"{"type":true}"#).unwrap();
        std::fs::write(source.join("spec.json"), root.to_string()).unwrap();
        std::fs::write(shared.join("pet.yaml"), "type: object\nproperties:\n  friend:\n    $ref: '../api/spec.json#/components/schemas/Pet'\n  'snow % ☃':\n    type: string\n  alias:\n    $ref: '#/properties/snow%20%25%20%E2%98%83'\nexample:\n  $ref: 'literal-not-a-reference.json'\n").unwrap();
        let api = crate::openapi::load(&source.join("spec.json")).unwrap();
        let first = plan(&api).unwrap();
        assert_eq!(first.name, "openapi.zip");
        assert_eq!(first.bytes, plan(&api).unwrap().bytes);
        let extracted = tempfile::tempdir().unwrap();
        let names = extract(&first.bytes, extracted.path());
        assert_eq!(
            names,
            ["openapi.json", "references/001.json", "references/002.json"]
        );
        assert!(names.iter().all(|name| !name.contains("private-name")));
        let reloaded = crate::openapi::load(&extracted.path().join("openapi.json")).unwrap();
        assert_eq!(reloaded.graph.dependencies().count(), 3);
        assert_eq!(
            reloaded.graph.references().count(),
            api.graph.references().count()
        );
        let texts = names
            .iter()
            .map(|name| std::fs::read_to_string(extracted.path().join(name)).unwrap())
            .collect::<String>();
        assert!(texts.contains("literal-not-a-reference.json"));
        assert!(!texts.contains("private-name"));
        assert!(!texts.contains("../shared/pet.yaml"));
        assert!(texts.contains("../openapi.json#/components/schemas/Pet"));
        assert!(texts.contains("#/properties/snow%20%25%20%E2%98%83"));
    }

    #[test]
    fn file_based_self_references_are_bundled_without_relying_on_source_names() {
        let dir = tempfile::tempdir().unwrap();
        let source = json!({"openapi":"3.1.0","info":{"title":"Test","version":"1"},"paths":{},"components":{"schemas":{
            "Pet":{"type":"object","properties":{"child":{"$ref":"original.json#/components/schemas/Pet"}}}
        }}});
        std::fs::write(dir.path().join("original.json"), source.to_string()).unwrap();
        let api = crate::openapi::load(&dir.path().join("original.json")).unwrap();
        let download = plan(&api).unwrap();
        assert_eq!(download.name, "openapi.zip");
        let output = tempfile::tempdir().unwrap();
        assert_eq!(extract(&download.bytes, output.path()), ["openapi.json"]);
        crate::openapi::load(&output.path().join("openapi.json")).unwrap();
    }
}
