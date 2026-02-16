# Contributing to DocAnvil

First of all — thank you for your interest in contributing.

DocAnvil aims to make it incredibly easy to produce beautiful, well-crafted documentation with minimal effort.  
Contributions that improve usability, polish, performance, extensibility, or developer experience are all welcome.

---

# Contribution Philosophy

DocAnvil is built around a few core principles:

- Minimal user input, maximum quality output.
- Strong defaults over excessive configuration.
- Stability and predictability.
- Clean, maintainable Rust code.
- Thoughtful evolution through versioning discipline.

All contributions should align with these principles.

---

# What Can Be Contributed?

We welcome:

- Bug fixes
- Feature proposals
- Feature implementations
- Performance improvements
- Documentation improvements
- Theme improvements
- Plugin development
- Developer tooling improvements

If you are unsure whether something fits the project direction, open an issue to discuss first.

---

# Contribution Process

DocAnvil follows a discussion-first workflow.

## 1. Open an Issue

Before starting work:

- Open an issue describing the problem or proposal.
- For features, explain the motivation and expected impact.
- For bugs, provide reproduction steps and environment details.

This ensures alignment before implementation begins.

## 2. Discussion & Approval

Core maintainers will review the issue and discuss:

- Scope
- Alignment with roadmap
- Design approach
- Whether it belongs in core or as a plugin

Once there is agreement, you can begin implementation.

## 3. Submit a Pull Request

When submitting a PR:

- Reference the related issue.
- Keep changes focused and minimal.
- Avoid unrelated refactoring.
- Provide a clear summary of changes.

All PRs require review from the core team before merging.

The `master` branch is protected and cannot be pushed to directly.

---

# Code Standards

DocAnvil is written in Rust and follows standard Rust conventions.

Before opening a PR, you must:

- Run `cargo fmt`
- Run `cargo clippy`
- Run `cargo test`
- Ensure no warnings remain
- Ensure strict mode builds succeed

Commands:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

# Testing Requirements

All contributions must include appropriate tests.

- New features must include tests.
- Bug fixes must include regression tests.
- Tests must pass locally before opening a PR.
- Strict mode must pass where applicable.

If a feature affects documentation output, example documentation may also need updating.

# Core vs Plugin Contributions

DocAnvil supports extensibility through plugins.

When proposing a feature:
- Consider whether it belongs in core or as a plugin.
- Core should remain focused, stable, and minimal.
- Advanced or niche functionality may belong in plugins.

Some plugins may be bundled if they are considered fundamental to the user experience.

Plugin development guidelines may be maintained separately.

# Roadmap & Versioning Alignment

DocAnvil follows Semantic Versioning.
See VERSIONING.md and ROADMAP.md for details.

When contributing:
- Avoid introducing breaking changes without prior discussion.
- Be mindful of the stability guarantees after 1.0.
- Respect output, CLI, and configuration contracts.

Breaking changes require explicit maintainer approval.

# Documentation Contributions

Documentation improvements are highly encouraged.

Good documentation:
- Is clear and minimal.
- Follows the project's tone.
- Reinforces the philosophy of ease-of-use.

If you change behavior, update relevant documentation.

# Coding Style
- Prefer clarity over cleverness.
- Avoid unnecessary abstraction.
- Keep modules focused and small.
- Write code that future maintainers can easily understand.

DocAnvil prioritizes maintainability over micro-optimizations unless performance is demonstrably impacted.

# Community Expectations

We aim for:
- Respectful discussions
- Constructive feedback
- Collaborative problem solving

Disagreement is normal. Keep it technical and solution-oriented.

# License

By contributing to DocAnvil, you agree that your contributions will be licensed under the MIT License.

See LICENSE for details.

# Final Notes

DocAnvil is evolving toward a stable 1.0 foundation and a strong extensible ecosystem.

We value contributors who:
- Think long-term
- Care about user experience
- Respect versioning discipline
- Improve polish and predictability

Thank you for helping build something focused, reliable, and beautifully minimal.