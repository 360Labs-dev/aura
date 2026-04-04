# Aura Language for VS Code

Syntax highlighting and language support for the [Aura programming language](https://github.com/360Labs-dev/aura).

## Features

- Syntax highlighting for `.aura` files
- Design token highlighting (`.accent`, `.bold`, `gap.md`, etc.)
- Security type highlighting (`secret`, `sanitized`, `email`, `url`, `token`)
- Real-time diagnostics via Aura Agent API
- Commands: Build, Run, Explain, Sketch

## Commands

- **Aura: Build** — compile the current file to web
- **Aura: Run** — start dev server with live reload
- **Aura: Explain** — show plain English description
- **Aura: Sketch** — generate app from a description

## Requirements

Install the Aura compiler:

```bash
git clone https://github.com/360Labs-dev/aura.git
cd aura && cargo install --path crates/aura-cli
```

## Installation

### From source
```bash
cd editors/vscode
npm install
# Press F5 in VS Code to launch Extension Development Host
```

### From VS Code Marketplace
Search for "Aura Language" in the Extensions panel.
