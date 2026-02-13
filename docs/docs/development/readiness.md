# ✅ DocAnvil 1.0 Readiness Checklist

**Purpose:**  
This checklist defines the minimum bar for releasing **DocAnvil 1.0.0** and committing to long-term stability guarantees for users.

DocAnvil is a Markdown-first, static documentation generator. Version 1.0 represents a commitment to a stable core: CLI, configuration, and content model.



## 🧱 Core Architecture

- [ ] Static site generation pipeline is stable and well-understood
- [ ] Build output is deterministic (same input → same output)
- [ ] No experimental features enabled by default
- [ ] Reasonable performance for small and medium documentation sites
- [ ] Clear separation between content, theme/layout, and configuration



## 🖋️ Content Model Stability

- [ ] Markdown syntax support is documented and complete
- [ ] Supported extensions (tables, footnotes, wiki-links, etc.) are frozen for 1.x
- [ ] Custom components API is stable
- [ ] Component behavior is consistent across themes
- [ ] Front-matter fields and their meanings are documented
- [ ] Sensible defaults exist for missing metadata

> **1.0 rule:** No breaking changes to the content model within 1.x.



## 🧭 Navigation & Structure

- [ ] Sidebar navigation produces predictable results
- [ ] Nested sections render correctly
- [ ] Deep links to headings are stable
- [ ] Table of contents behavior is consistent
- [ ] Large documentation trees remain usable



## 🔗 Linking & Validation

- [ ] Internal links are resolved consistently
- [ ] Broken links are detected during build
- [ ] Link errors are actionable and readable
- [ ] `--strict` mode works reliably for CI



## 🧰 CLI & Configuration

- [ ] CLI commands are stable and documented
- [ ] Flags and options have clear, consistent naming
- [ ] Exit codes are meaningful and reliable
- [ ] Error messages clearly explain what failed, where, and how to fix it
- [ ] Configuration format is documented and frozen
- [ ] Invalid configuration produces clear, actionable errors
- [ ] `docanvil doctor` detects common misconfigurations

> **1.0 rule:** No breaking CLI or configuration changes in 1.x.



## 🎨 Theming & Presentation

- [ ] Default theme is usable and accessible
- [ ] Theme customization points are documented
- [ ] Custom CSS does not require undocumented hooks
- [ ] Components degrade gracefully when styles are overridden
- [ ] Output HTML structure is stable enough for user-defined CSS



## 🔍 Search & SEO

- [ ] Search index is generated reliably
- [ ] Search results are accurate and performant
- [ ] SEO metadata is present and correct
- [ ] `sitemap.xml` is generated correctly
- [ ] `robots.txt` is generated correctly



## 📚 Documentation Quality

- [ ] DocAnvil documentation is built using DocAnvil itself
- [ ] Installation instructions are clear and current
- [ ] A minimal “Getting Started” guide exists
- [ ] Configuration reference documentation is complete
- [ ] Common workflows are documented
- [ ] Known limitations are documented
- [ ] Migration expectations are documented (even if empty)



## 🧪 Reliability & Testing

- [ ] Core functionality is covered by automated tests
- [ ] Builds fail loudly and early on errors
- [ ] Edge cases are handled gracefully
- [ ] No known data-loss or silent-failure bugs
- [ ] Critical paths have reasonable test coverage



## 📦 Packaging & Distribution

- [ ] Package installs cleanly from npm
- [ ] Version numbers are consistent across outputs
- [ ] CLI entrypoints behave consistently across platforms
- [ ] License is clear and included in the repository
- [ ] README accurately reflects current behavior



## 🧭 Project & Maintenance Signals

- [ ] Versioning policy is documented
- [ ] CHANGELOG exists and is maintained
- [ ] CONTRIBUTING guidelines exist
- [ ] Issue or discussion templates exist (recommended)
- [ ] Support expectations for the 1.x series are stated



## 🟢 Final 1.0 Confidence Check

Before tagging **v1.0.0**, the following should all be true:

- [ ] I would confidently recommend DocAnvil 1.0 to a colleague
- [ ] I am comfortable maintaining 1.x without breaking users
- [ ] The core mental model feels complete and stable
- [ ] Remaining roadmap items are extensions, not fixes



## 📌 Notes

Version 1.0 represents a commitment to stability, not feature completeness.  
New features will continue to land in 1.x as long as they are additive and opt-in.
