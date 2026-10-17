//! Local OpenAPI loading and normalization, independent of publishing.
//!
//! This is a foundation, not a complete OpenAPI/schema validator. See
//! `planning/openapi/foundation.md` for the supported subset and boundaries.
mod graph;
mod model;
pub(crate) mod publish;
mod render;

pub use graph::{DocumentGraph, Location, OpenApiError, Reference, Version};
pub use model::{Api, Operation, Parameter, SourcedValue, Warning};

use std::path::Path;

/// Load a local JSON/YAML specification and its referenced documents, then
/// normalize operations. No network access, rendering, or source files written.
pub fn load(path: &Path) -> Result<Api, OpenApiError> {
    Api::from_graph(DocumentGraph::load(path)?)
}
