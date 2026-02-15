# Versioning Policy

DocAnvil follows **Semantic Versioning (SemVer 2.0.0)**.

Version format:

MAJOR.MINOR.PATCH  
Example: 1.2.3

---

# Pre-1.0 (0.x)

While below 1.0.0, DocAnvil is considered unstable.

- Breaking changes MAY occur in MINOR releases.
- Configuration, CLI flags, and internal APIs may evolve.
- Patch releases must not introduce breaking behavior.

Version bump rules during 0.x:

| Change Type | Bump |
|-------------|------|
| Bug fix | PATCH |
| Backward-compatible feature | MINOR |
| Breaking change | MINOR |

Breaking changes should always be documented clearly.

---

# 1.0 Stability Guarantee

Starting at 1.0.0, DocAnvil guarantees backward compatibility for:

- CLI flags
- Configuration file structure
- Document frontmatter schema
- Output directory structure
- WASM plugin API (v1)

Breaking changes to these require a MAJOR version bump.

---

# Versioning After 1.0

## MAJOR

Increment MAJOR for:

- Incompatible CLI changes
- Incompatible configuration changes
- Output structure changes
- Plugin API version changes
- Removal of stable features

## MINOR

Increment MINOR for:

- New features
- New CLI flags or config fields (additive)
- New plugin hooks (additive)
- New diagnostics

Must remain backward compatible.

## PATCH

Increment PATCH for:

- Bug fixes
- Performance improvements
- Diagnostic improvements
- Internal refactoring

PATCH releases must not change behavior beyond fixing bugs.

---

# Plugin API Versioning

The WASM plugin interface is versioned via:

docanvil_api_version() -> i32

- DocAnvil 1.x supports Plugin API v1.
- New plugin API versions require a MAJOR DocAnvil release.
- Changes within a plugin API version are strictly additive.

---

# Deprecation Policy (1.x+)

- Features are deprecated before removal.
- Deprecations remain for at least one MINOR release.
- Removal requires a MAJOR release.

---

# Strict Mode and Exit Codes

After 1.0:

- Strict mode treats warnings as errors.
- Non-zero exit codes indicate build failure.
- Exit codes are considered part of the stable CLI interface.

---

# Build Determinism (1.x+)

Same input + same DocAnvil version should produce identical output  
(excluding timestamps or non-deterministic metadata).

---

# Philosophy

0.x → Iterate boldly  
1.x → Evolve safely  
2.x → Redesign deliberately
