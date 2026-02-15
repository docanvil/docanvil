# Roadmap

This roadmap outlines DocAnvil’s progression from early development (0.x) to a stable 1.0 release and beyond.

DocAnvil follows Semantic Versioning.  
See VERSIONING.md for the full policy.

Current version: 0.1.4

---

# Philosophy

0.x → Iterate boldly  
1.x → Evolve safely  
2.x → Redesign deliberately  

The goal is to reach a stable, production-ready 1.0 foundation before expanding the ecosystem.

---

# Path to 1.0

During 0.x:

- Breaking changes are permitted in MINOR releases.
- Patch releases must not introduce breaking behavior.
- The focus is stabilising core contracts before committing to long-term guarantees.

---

## 0.2.x — Core Engine Stabilisation

Goal: Harden and stabilise the document processing pipeline.

Focus:

- Stable Markdown parsing
- Stable frontmatter parsing
- Deterministic build output
- Consistent error handling
- Internal pipeline cleanup if required

Exit criteria:

- Same input + same version = identical output
- No expected architectural rewrites before 1.0

---

## 0.3.x — Diagnostics & Strict Mode

Goal: Make DocAnvil CI-ready.

Focus:

- Strict mode
- Structured diagnostics
- Clear warning categories
- Defined non-zero exit codes
- Consistent CLI error formatting

Exit criteria:

- Strict mode usable in CI pipelines
- Exit codes documented and stable

---

## 0.4.x — Configuration Stabilisation

Goal: Finalise configuration structure before 1.0.

Focus:

- Stabilise docanvil.toml schema
- Configuration validation
- Clear configuration error reporting
- Documentation alignment

Exit criteria:

- Configuration structure complete and unlikely to change

---

## 0.5.x — Output Contract Freeze

Goal: Lock the output structure.

Focus:

- Output directory layout finalised
- Asset handling stabilised
- Template resolution behaviour defined
- Output contract documented

Exit criteria:

- Output layout will not change in 1.x

---

## 0.6.x — Internal Refactor Window (Optional)

Goal: Final cleanup before stabilisation.

Focus:

- Internal API simplification
- Removal of experimental flags
- Codebase cleanup
- Performance baseline improvements

Last safe window for significant internal restructuring.

---

## 0.7.x — Deprecation Sweep

Goal: Remove unstable or experimental features.

Focus:

- Remove temporary flags
- Remove experimental configuration fields
- Finalise feature surface for 1.0

Breaking changes are still allowed here.

---

## 0.8.x — Hardening Phase

Goal: Stabilisation and polish.

Focus:

- Bug fixes
- Performance improvements
- Documentation refinement
- Edge-case validation
- Test coverage expansion

No intentional breaking changes.

---

## 0.9.x — Freeze & Release Preparation

Goal: Validate 1.0 readiness.

Focus:

- Strict mode audit
- Determinism verification
- Build reproducibility validation
- Changelog audit
- Migration notes prepared

0.9.x releases should be treated as release candidates.

---

# 1.0.0 — Stability Guarantee

Starting at 1.0.0, DocAnvil guarantees backward compatibility for:

- CLI flags
- Configuration structure
- Document frontmatter schema
- Output directory structure
- Exit code semantics
- Plugin API v1 (if introduced before or at 1.0)

1.0 represents:

A stable foundation suitable for production documentation pipelines.

---

# Post-1.0 Evolution

After 1.0, development shifts to additive improvements only within 1.x.

---

## 1.1.x — Extensibility Foundations

Planned focus:

- WASM plugin system (v1)
- Additional CLI flags (additive)
- Additional diagnostics
- Template enhancements

Must remain fully backward compatible.

---

## 1.2.x+ — Ecosystem Growth

Potential focus areas:

- Expanded plugin hooks
- Performance optimisations
- Incremental or partial builds
- Caching
- Plugin SDK crate
- Documentation tooling enhancements

Breaking changes require a MAJOR release.

---

# 2.0 and Beyond

A 2.0 release would only be considered if:

- Configuration redesign is required
- Plugin API v2 is introduced
- Output model changes fundamentally
- Major architectural shifts are necessary

2.0 should be deliberate and clearly justified.

---

# Summary

The strategy is:

1. Stabilise the core.
2. Lock contracts before 1.0.
3. Make 1.0 boring and reliable.
4. Expand safely and additively in 1.x.
