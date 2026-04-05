# Agent API

Aura exposes a JSON-RPC 2.0 server over standard input and output so editor tooling and AI agents can inspect source code or whole projects.

## Start The Server

```bash
cargo run --offline -p aura-cli -- agent serve
```

For one-off requests during development:

```bash
cargo run --offline -p aura-cli -- agent call ping
```

## Supported Methods

- `ping`
- `ast.get`
- `diagnostics.get`
- `completions.get`
- `hir.get`
- `explain`
- `sketch`
- `hover`
- `goto.definition`

## Source-Based Requests

Use `source` when you already have the Aura text in memory.

```bash
cargo run --offline -p aura-cli -- agent call ast.get '{"source":"app Hello\n  screen Main\n    view\n      text \"Hi\""}'
```

```bash
cargo run --offline -p aura-cli -- agent call diagnostics.get '{"source":"app Hello\n  screen Main\n    view\n      text \"Hi\""}'
```

## Project-Aware Requests

Use `path` when you want Aura to resolve a file or project root first.

```bash
cargo run --offline -p aura-cli -- agent call hir.get '{"path":"./examples/launchpad.aura"}'
```

```bash
cargo run --offline -p aura-cli -- agent call diagnostics.get '{"path":"./examples/launchpad.aura"}'
```

For project-aware requests, the response can include:

- `project_root`
- `files`
- per-diagnostic `location.file`

## Hover And Definition

These methods accept either `source` or `path`, plus a 1-based `line` and `column`.

```bash
cargo run --offline -p aura-cli -- agent call hover '{"path":"./examples/launchpad.aura","line":18,"column":16}'
```

```bash
cargo run --offline -p aura-cli -- agent call goto.definition '{"path":"./examples/launchpad.aura","line":18,"column":16}'
```

## Completion Contexts

`completions.get` currently supports these contexts:

- `design_token`
- `type`
- `view_element`

Example:

```bash
cargo run --offline -p aura-cli -- agent call completions.get '{"context":"view_element"}'
```

## Response Notes

- Diagnostics include an Aura error code, severity, message, source location, and optional fix suggestion with confidence.
- `hir.get` returns a compact summary of app, model, screen, and component structure.
- `explain` returns a plain-English explanation of the program.

## Best Use Today

The current Agent API is strongest for:

- editor diagnostics
- project inspection
- HIR inspection
- simple hover and go-to-definition support

It is useful now for tooling integration, especially when paired with project-aware `path` requests.
