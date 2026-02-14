---
title: V1.0 Readiness
---
# ✅ DocAnvil 1.0 Readiness Checklist

**Purpose:**  
This checklist defines the minimum bar for releasing **DocAnvil 1.0.0** and committing to long-term stability guarantees for users.

DocAnvil is a Markdown-first, static documentation generator. Version 1.0 represents a commitment to a stable core: CLI, configuration, and content model.



## 🧱 Core Architecture

- Static site generation pipeline is stable and well-understood :::lozenge{type="success",text="Done"}
- Build output is deterministic (same input → same output) :::lozenge{type="default",text="Not started"}
- No experimental features enabled by default :::lozenge{type="default",text="Not started"}
- Reasonable performance for small and medium documentation sites :::lozenge{type="default",text="Not started"}
- Clear separation between content, theme/layout, and configuration :::lozenge{type="default",text="Not started"}



## 🖋️ Content Model Stability

- Markdown syntax support is documented and complete :::lozenge{type="default",text="Not started"}
- Supported extensions (tables, footnotes, wiki-links, etc.) are frozen for 1.x :::lozenge{type="default",text="Not started"}
- Custom components API is stable  :::lozenge{type="default",text="Not started"}
- Component behavior is consistent across themes :::lozenge{type="default",text="Not started"}
- Front-matter fields and their meanings are documented :::lozenge{type="default",text="Not started"}
- Sensible defaults exist for missing metadata :::lozenge{type="default",text="Not started"}

> **1.0 rule:** No breaking changes to the content model within 1.x.



## 🧭 Navigation & Structure

- Sidebar navigation produces predictable results :::lozenge{type="in-progress",text="In progress"}
- Nested sections render correctly :::lozenge{type="in-progress",text="In progress"}
- Deep links to headings are stable :::lozenge{type="default",text="Not started"}
- Table of contents behavior is consistent :::lozenge{type="default",text="Not started"}
- Large documentation trees remain usable :::lozenge{type="default",text="Not started"}



## 🔗 Linking & Validation

- Internal links are resolved consistently :::lozenge{type="default",text="Not started"}
- Broken links are detected during build :::lozenge{type="success",text="Done"}
- Link errors are actionable and readable :::lozenge{type="success",text="Done"}
- `--strict` mode works reliably for CI :::lozenge{type="in-progress",text="In progress"}



## 🧰 CLI & Configuration

- CLI commands are stable and documented :::lozenge{type="default",text="Not started"}
- Flags and options have clear, consistent naming :::lozenge{type="default",text="Not started"}
- Exit codes are meaningful and reliable :::lozenge{type="default",text="Not started"}
- Error messages clearly explain what failed, where, and how to fix it :::lozenge{type="default",text="Not started"}
- Configuration format is documented and frozen :::lozenge{type="default",text="Not started"}
- Invalid configuration produces clear, actionable errors :::lozenge{type="default",text="Not started"}
- `docanvil doctor` detects common misconfigurations :::lozenge{type="default",text="Not started"}

> **1.0 rule:** No breaking CLI or configuration changes in 1.x.



## 🎨 Theming & Presentation

- Default theme is usable and accessible :::lozenge{type="default",text="Not started"}
- Theme customization points are documented :::lozenge{type="default",text="Not started"}
- Custom CSS does not require undocumented hooks :::lozenge{type="default",text="Not started"}
- Components degrade gracefully when styles are overridden :::lozenge{type="default",text="Not started"}
- Output HTML structure is stable enough for user-defined CSS :::lozenge{type="default",text="Not started"}



## 🔍 Search & SEO

- Search index is generated reliably :::lozenge{type="success",text="Done"}
- Search results are accurate and performant :::lozenge{type="success",text="Done"}
- SEO metadata is present and correct :::lozenge{type="success",text="Done"}
- `sitemap.xml` is generated correctly :::lozenge{type="success",text="Done"}
- `robots.txt` is generated correctly :::lozenge{type="success",text="Done"}



## 📚 Documentation Quality

- DocAnvil documentation is built using DocAnvil itself :::lozenge{type="in-progress",text="In progress"}
- Installation instructions are clear and current :::lozenge{type="in-progress",text="In progress"}
- A minimal “Getting Started” guide exists :::lozenge{type="in-progress",text="In progress"}
- Configuration reference documentation is complete :::lozenge{type="in-progress",text="In progress"}
- Common workflows are documented :::lozenge{type="default",text="Not started"}
- Known limitations are documented :::lozenge{type="default",text="Not started"}
- Migration expectations are documented (even if empty) :::lozenge{type="in-progress",text="In progress"}



## 🧪 Reliability & Testing

- Core functionality is covered by automated tests :::lozenge{type="in-progress",text="In progress"}
- Builds fail loudly and early on errors :::lozenge{type="default",text="Not started"}
- Edge cases are handled gracefully :::lozenge{type="default",text="Not started"}
- No known data-loss or silent-failure bugs :::lozenge{type="default",text="Not started"}
- Critical paths have reasonable test coverage :::lozenge{type="in-progress",text="In progress"}



## 📦 Packaging & Distribution

- Package installs cleanly from cargo :::lozenge{type="in-progress",text="In progress"}
- Version numbers are consistent across outputs :::lozenge{type="default",text="Not started"}
- CLI entrypoints behave consistently across platforms :::lozenge{type="default",text="Not started"}
- License is clear and included in the repository :::lozenge{type="default",text="Not started"}
- README accurately reflects current behavior :::lozenge{type="default",text="Not started"}



## 🧭 Project & Maintenance Signals

- Versioning policy is documented :::lozenge{type="default",text="Not started"}
- CHANGELOG exists and is maintained :::lozenge{type="success",text="Done"}
- CONTRIBUTING guidelines exist :::lozenge{type="default",text="Not started"}
- Issue or discussion templates exist (recommended) :::lozenge{type="default",text="Not started"}
- Support expectations for the 1.x series are stated :::lozenge{type="default",text="Not started"}



## 🟢 Final 1.0 Confidence Check

Before tagging **v1.0.0**, the following should all be true:

- I would confidently recommend DocAnvil 1.0 to a colleague :::lozenge{type="default",text="Not started"}
- I am comfortable maintaining 1.x without breaking users :::lozenge{type="default",text="Not started"}
- The core mental model feels complete and stable :::lozenge{type="default",text="Not started"}
- Remaining roadmap items are extensions, not fixes :::lozenge{type="default",text="Not started"}



## 📌 Notes

Version 1.0 represents a commitment to stability, not feature completeness.  
New features will continue to land in 1.x as long as they are additive and opt-in.

[[Broken]]