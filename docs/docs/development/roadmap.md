# Roadmap

DocAnvil follows [Semantic Versioning](https://semver.org). Here's where things stand and where they're headed.

**Current version: 0.3.1**

---

## Philosophy

**0.x** — Iterate boldly. Ship fast, learn, break things when needed.
**1.x** — Evolve safely. Additive improvements only, no breaking changes.
**2.x** — Redesign deliberately. Only if the foundation needs rethinking.

The goal is to reach a stable, production-ready 1.0 before expanding the ecosystem.

---

## Path to 1.0

During 0.x, breaking changes are permitted in minor releases. Patch releases won't introduce breaking behavior. The focus is on stabilizing core contracts before committing to long-term guarantees.

### 0.2.x — Core Engine Stabilization

Harden the document processing pipeline. Stable Markdown and front matter parsing, deterministic build output, and consistent error handling.

**Done when:** same input + same version = identical output, with no major architectural rewrites expected before 1.0.

### 0.3.x — Diagnostics & Strict Mode *(current)*

Make DocAnvil CI-ready. Strict mode, structured diagnostics, clear warning categories, well-defined exit codes, and consistent CLI error formatting.

**Done when:** `--strict` mode is reliable for CI pipelines and exit codes are documented.

### 0.4.x — Configuration Stabilization

Lock down the `docanvil.toml` schema. Configuration validation, clear error reporting, and documentation alignment.

**Done when:** the configuration structure is complete and unlikely to change.

### 0.5.x — Output Contract Freeze

Finalize the output directory layout, asset handling, and template resolution behavior.

**Done when:** the output layout is documented and won't change in 1.x.

### 0.6.x — Internal Refactor Window *(optional)*

Last safe window for significant internal restructuring — API simplification, removal of experimental flags, codebase cleanup, performance baseline.

### 0.7.x — Deprecation Sweep

Remove temporary flags, experimental config fields, and anything that shouldn't ship in 1.0. Breaking changes are still fair game here.

### 0.8.x — Hardening

Bug fixes, performance improvements, documentation polish, edge-case handling, and expanded test coverage. No intentional breaking changes.

### 0.9.x — Release Candidates

Strict mode audit, determinism verification, build reproducibility, changelog review, and migration notes. Treat 0.9.x releases as release candidates.

---

## 1.0.0 — The Stability Promise

Starting at 1.0.0, DocAnvil guarantees backward compatibility for:

- CLI flags and commands
- Configuration structure (`docanvil.toml`)
- Front matter schema
- Output directory layout
- Exit code semantics

1.0 means a stable foundation you can build production documentation pipelines on.

---

## After 1.0

Development shifts to additive improvements only within 1.x.

### 1.1.x — Extensibility

WASM plugin system (v1), additional CLI flags, more diagnostics, and template enhancements. Fully backward compatible.

### 1.2.x+ — Ecosystem Growth

Plugin hooks, performance optimizations, incremental builds, caching, and a plugin SDK crate. Breaking changes require a major release.

---

## 2.0 and Beyond

A 2.0 release would only happen if there's a clear need — configuration redesign, plugin API v2, fundamental output model changes, or major architectural shifts. It should be deliberate and well-justified.

---

## Summary

1. Stabilize the core
2. Lock contracts before 1.0
3. Make 1.0 boring and reliable
4. Expand safely in 1.x
