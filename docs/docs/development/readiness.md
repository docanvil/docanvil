---
{
  "title": "V1.0 Readiness"
}
---
# ✅ DocAnvil 1.0 Readiness Checklist

**Purpose:**  
This checklist defines the minimum bar for releasing **DocAnvil 1.0.0** and committing to long-term stability guarantees for users.

DocAnvil is a Markdown-first, static documentation generator. Version 1.0 represents a commitment to a stable core: CLI, configuration, and content model.



## 🧱 Core Architecture

- Static site generation pipeline is stable and well-understood :::lozenge{type="success",text="Done"}
- Build output is deterministic (same input → same output) :::lozenge{type="success",text="Done"}
- No experimental features enabled by default :::lozenge{type="success",text="Done"}
- Reasonable performance for small and medium documentation sites :::lozenge{type="success",text="Done"}
- Clear separation between content, theme/layout, and configuration :::lozenge{type="success",text="Done"}



## 🖋️ Content Model Stability

- Markdown syntax support is documented and complete :::lozenge{type="success",text="Done"}
- Supported extensions (tables, footnotes, wiki-links, etc.) are frozen for 1.x :::lozenge{type="success",text="Done"}
- Custom components API is stable  :::lozenge{type="success",text="Done"}
- Component behavior is consistent across themes :::lozenge{type="success",text="Done"}
- Front-matter fields and their meanings are documented :::lozenge{type="success",text="Done"}
- Sensible defaults exist for missing metadata :::lozenge{type="success",text="Done"}

> **1.0 rule:** No breaking changes to the content model within 1.x.



## 🧭 Navigation & Structure

- Sidebar navigation produces predictable results :::lozenge{type="success",text="Done"}
- Nested sections render correctly :::lozenge{type="success",text="Done"}
- Deep links to headings are stable :::lozenge{type="success",text="Done"}
- Table of contents behavior is consistent :::lozenge{type="success",text="Done"}
- Large documentation trees remain usable :::lozenge{type="success",text="Done"}



## 🔗 Linking & Validation

- Internal links are resolved consistently :::lozenge{type="success",text="Done"}
- Broken links are detected during build :::lozenge{type="success",text="Done"}
- Link errors are actionable and readable :::lozenge{type="success",text="Done"}
- `--strict` mode works reliably for CI :::lozenge{type="success",text="Done"}



## 🧰 CLI & Configuration

- CLI commands are stable and documented :::lozenge{type="success",text="Done"}
- Flags and options have clear, consistent naming :::lozenge{type="success",text="Done"}
- Exit codes are meaningful and reliable :::lozenge{type="success",text="Done"}
- Error messages clearly explain what failed, where, and how to fix it :::lozenge{type="in-progress",text="In progress"}
- Configuration format is documented and frozen :::lozenge{type="success",text="Done"}
- Invalid configuration produces clear, actionable errors :::lozenge{type="in-progress",text="In progress"}
- `docanvil doctor` detects common misconfigurations :::lozenge{type="success",text="Done"}

> **1.0 rule:** No breaking CLI or configuration changes in 1.x.



## 🎨 Theming & Presentation

- Default theme is usable and accessible :::lozenge{type="success",text="Done"}
- Theme customization points are documented :::lozenge{type="success",text="Done"}
- Custom CSS does not require undocumented hooks :::lozenge{type="in-progress",text="In progress"}
- Components degrade gracefully when styles are overridden :::lozenge{type="in-progress",text="In progress"}
- Output HTML structure is stable enough for user-defined CSS :::lozenge{type="in-progress",text="In progress"}



## 🔍 Search & SEO

- Search index is generated reliably :::lozenge{type="success",text="Done"}
- Search results are accurate and performant :::lozenge{type="success",text="Done"}
- SEO metadata is present and correct :::lozenge{type="success",text="Done"}
- `sitemap.xml` is generated correctly :::lozenge{type="success",text="Done"}
- `robots.txt` is generated correctly :::lozenge{type="success",text="Done"}



## 📚 Documentation Quality

- DocAnvil documentation is built using DocAnvil itself :::lozenge{type="success",text="Done"}
- Installation instructions are clear and current :::lozenge{type="success",text="Done"}
- A minimal "Getting Started" guide exists :::lozenge{type="success",text="Done"}
- Configuration reference documentation is complete :::lozenge{type="success",text="Done"}
- Common workflows are documented :::lozenge{type="in-progress",text="In progress"}
- Known limitations are documented :::lozenge{type="default",text="Not started"}
- Migration expectations are documented (even if empty) :::lozenge{type="success",text="Done"}



## 🧪 Reliability & Testing

- Core functionality is covered by automated tests :::lozenge{type="in-progress",text="In progress"}
- Builds fail loudly and early on errors :::lozenge{type="success",text="Done"}
- Edge cases are handled gracefully :::lozenge{type="in-progress",text="In progress"}
- No known data-loss or silent-failure bugs :::lozenge{type="success",text="Done"}
- Critical paths have reasonable test coverage :::lozenge{type="in-progress",text="In progress"}



## 📦 Packaging & Distribution

- Package installs cleanly from cargo :::lozenge{type="success",text="Done"}
- Version numbers are consistent across outputs :::lozenge{type="default",text="Not started"}
- CLI entrypoints behave consistently across platforms :::lozenge{type="in-progress",text="In progress"}
- License is clear and included in the repository :::lozenge{type="success",text="Done"}
- README accurately reflects current behavior :::lozenge{type="success",text="Done"}



## 🧭 Project & Maintenance Signals

- Versioning policy is documented :::lozenge{type="success",text="Done"}
- CHANGELOG exists and is maintained :::lozenge{type="success",text="Done"}
- CONTRIBUTING guidelines exist :::lozenge{type="success",text="Done"}
- Issue or discussion templates exist (recommended) :::lozenge{type="success",text="Done"}
- Support expectations for the 1.x series are stated :::lozenge{type="default",text="Not started"}



## 🟢 Final 1.0 Confidence Check

Before tagging **v1.0.0**, the following should all be true:

- I would confidently recommend DocAnvil 1.0 to a colleague :::lozenge{type="default",text="Not started"}
- I am comfortable maintaining 1.x without breaking users :::lozenge{type="default",text="Not started"}
- The core mental model feels complete and stable :::lozenge{type="default",text="Not started"}
- Remaining roadmap items are extensions, not fixes :::lozenge{type="default",text="Not started"}



## 🚧 Key Gaps to Close Before V1

These are the most impactful items still outstanding. Closing these would meaningfully increase confidence in a 1.0 release.

**Blockers:**
- **Document HTML/CSS stability guarantees.** The theming system works well, but there's no documented commitment on which CSS classes and HTML structure are considered stable API. Users writing custom CSS need to know what won't break under them in 1.x.
- **Improve error messages with recovery suggestions.** Doctor diagnostics are excellent, but build/serve errors often say *what* went wrong without suggesting *how to fix it*. Bringing those up to the doctor's standard would make a real difference.

**High value:**
- **Add integration tests for the build pipeline.** All 119 tests are unit tests. A handful of end-to-end tests (config → build → verify output) would catch regressions that unit tests miss.
- **Document known limitations.** Users coming from other tools need to know what DocAnvil intentionally doesn't do (yet). Even a short list builds trust.
- **Add a deployment guide.** The docs mention deploying to GitHub Pages, Netlify, and S3 in passing but never walk through it. This is a common first question.
- **Add cross-platform CI.** Rust is inherently cross-platform, but there's no evidence of testing on Windows. A GitHub Actions matrix would close this gap cheaply.



## 📌 Notes

Version 1.0 represents a commitment to stability, not feature completeness.
New features will continue to land in 1.x as long as they are additive and opt-in.
