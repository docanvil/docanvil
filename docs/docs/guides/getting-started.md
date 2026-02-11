# Getting Started

Welcome to docs! This guide walks you through setup and first steps.

## Installation

Install DocAnvil using Cargo:

```bash
cargo install docanvil
```

## Create a New Project

```bash
docanvil init docs
cd docs
```

## Start the Dev Server

```bash
docanvil serve
```

Open [http://localhost:3000](http://localhost:3000) in your browser. Changes to
any `.md` file will reload the page automatically.

## Build for Production

```bash
docanvil build
```

Static HTML is written to the `dist/` directory — deploy it anywhere.

See [[configuration]] for details on customizing your project, or head
back to the [[index|home page]].
