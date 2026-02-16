# 🔨 DocAnvil

**Forge beautiful static documentation from Markdown.**

DocAnvil turns your Markdown files into a polished, searchable documentation site with a single command. Built in Rust for speed, designed for developers who want great docs without the setup overhead.

**[📖 Documentation](https://docanvil.github.io/docanvil/)** &middot; **[💻 GitHub](https://github.com/docanvil/docanvil)**

## 🚀 Quickstart

```bash
cargo install docanvil
docanvil new my-docs
cd my-docs
docanvil serve
```

Open [localhost:3000](http://localhost:3000) — your site is live with hot reloading. ⚡

When you're ready to deploy:

```bash
docanvil build
```

Static HTML goes to `dist/`, ready to host anywhere.

## ✨ Why DocAnvil?

Most documentation tools either look great but lock you in, or give you control but demand hours of configuration. DocAnvil gives you both — a site you'll be proud of out of the box, with the flexibility to make it your own.

- 🏎️ **Instant results** — scaffold, serve, and start writing in under a minute
- 🎨 **Beautiful by default** — clean, readable design with dark mode, syntax highlighting, and responsive layout
- 🔄 **Live reloading** — see every edit reflected instantly in your browser
- 📝 **Rich Markdown** — tabs, admonitions, code groups, diagrams, popovers, and wikilinks built in
- 🎛️ **Fully customisable** — themes, templates, and CSS variables let you match any brand
- 🌍 **Static output** — deploy to GitHub Pages, Netlify, S3, or anywhere that serves HTML
- ⚡ **Fast** — Rust-powered builds that scale with your content

## 📚 Learn More

Head to the **[full documentation](https://docanvil.github.io/docanvil/)** for guides on configuration, theming, components, navigation, and more.

## 📄 License

MIT
