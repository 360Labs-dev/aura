# Aura Language — Project Conventions

## What is Aura?

Aura is a programming language for AI coding agents with built-in design language and multi-platform native compilation. File extension: `.aura`. CLI: `aura build`, `aura run`, `aura design`.

## Architecture

```
  .aura source → Lexer → Parser → AST → Semantic Analysis → HIR → LIR → Backend → Native code
                 (logos)  (chumsky)                          │       │
                                                        SwiftUI  HTML/CSS
                                                        Compose  WinUI
```

### Key Decisions (locked, do not change without discussion)

- **Bootstrap language:** Rust
- **Lexer:** logos crate
- **Parser:** chumsky crate
- **IR:** Two-tier — HIR (semantic intent) + LIR (rendering primitives)
- **Backends:** SwiftUI, Jetpack Compose, HTML/CSS/JS, WinUI, Terminal UI
- **Indentation:** 2 spaces (tabs are a compile error)
- **Security types:** Built-in (`secret`, `sanitized`, `email`, `url`, `token`)
- **Error system:** Aggressive poisoning + confidence-scored fix suggestions

## Project Structure

```
aura/
├── spec/                    # Language specification (source of truth)
├── crates/
│   ├── aura-core/           # Lexer, Parser, AST, Types, HIR, LIR, Errors
│   ├── aura-cli/            # CLI binary (aura build/run/test/etc.)
│   ├── aura-backend-web/    # HTML/CSS/JS codegen
│   ├── aura-backend-swift/  # SwiftUI codegen
│   ├── aura-backend-compose/# Jetpack Compose codegen
│   ├── aura-backend-win/    # WinUI codegen
│   ├── aura-backend-tui/    # Terminal UI codegen
│   ├── aura-lsp/            # Language Server Protocol
│   ├── aura-agent/          # AI Agent API
│   └── aura-pkg/            # Package manager
├── tests/
│   └── conformance/         # Shared .aura test files (all backends)
├── examples/                # Example .aura programs
├── docs/                    # User-facing documentation
└── benchmarks/              # AI agent benchmark definitions
```

## Coding Standards

### Rust

- Follow standard Rust conventions (rustfmt, clippy)
- All public APIs have doc comments
- Error types are specific (no `anyhow` in library crates, only in CLI)
- Use `thiserror` for error type definitions
- AST nodes use arena allocation (`bumpalo`) for performance
- All IR node types derive `Debug`, `Clone`, `PartialEq`

### Testing

- Every new parser rule needs: happy path + error path + edge case test
- Backend changes must pass the conformance suite (`tests/conformance/`)
- Fuzzing runs on every PR via `cargo-fuzz`
- Test names follow `test_{component}_{scenario}_{expected}` pattern

### Error Messages

Every compiler error must include:
1. Error code (E0xxx)
2. Human-readable message
3. Source location with caret
4. Help text with suggestion
5. Machine-readable fix with confidence score (for AI agents)

### Design Token Changes

Changes to the design token vocabulary require:
1. Update `spec/language.md` Section 6 + Appendix B
2. Update `aura-core/src/design/tokens.rs`
3. Update ALL backends' `resolve_design_token()` implementations
4. Update conformance tests

### Commit Messages

Format: `area: short description`
Areas: `core`, `parser`, `types`, `hir`, `lir`, `backend-web`, `backend-swift`, `backend-compose`, `cli`, `lsp`, `agent`, `pkg`, `spec`, `docs`, `tests`

Examples:
- `parser: add support for when-expression in views`
- `types: implement secret type compile-time checks`
- `backend-web: emit CSS custom properties for design tokens`

## Spec-First Development

The specification (`spec/language.md`) is the source of truth. When implementing a feature:
1. Ensure it's specified in the spec
2. Write conformance tests from the spec
3. Implement in aura-core
4. Implement in backends
5. Verify conformance tests pass
