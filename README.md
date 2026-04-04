# Aura

**A programming language for AI coding agents with built-in design intelligence.**

Write once. Run native everywhere. Let AI do the coding.

```
app TodoApp
  theme: modern.dark

  model Todo
    title: text
    done: bool = false

  screen Main
    state todos: list[Todo] = []
    state input: text = ""

    view
      column gap.md padding.lg
        heading "My Tasks" size.xl
        row gap.sm
          textfield input placeholder: "Add task..."
          button "Add" accent -> todos.push(Todo(title: input))
        each todos as todo
          row gap.md align.center
            checkbox todo.done
            text todo.title strike: todo.done
            spacer
            button.icon "trash" danger -> todos.remove(todo)
```

This compiles to **native SwiftUI** (iOS/macOS), **Jetpack Compose** (Android), **HTML/CSS/JS** (Web), **WinUI** (Windows), and **Terminal UI** — from a single source file.

## Why Aura?

AI coding agents generate millions of lines of code daily. But they're writing in languages designed for humans — verbose, ambiguous, full of boilerplate. Aura is different:

| Problem | Existing Languages | Aura |
|---|---|---|
| AI generates syntactically valid but semantically wrong code | Common — implicit coercion, ASI, operator precedence surprises | **Zero ambiguity** — every valid program has exactly one parse |
| One typo causes 47 cascading errors, agent makes 47 wrong fixes | Standard compiler behavior | **Error poisoning** — one root cause = one error, cascades suppressed |
| Agent can't tell if its fix is right | Errors say what's wrong, not how to fix it | **Confidence-scored fixes** — 0.98 confidence = auto-apply |
| Cross-platform means 3 codebases | React Native, Flutter (shared runtime, not native) | **Multi-backend codegen** — truly native output per platform |
| Design requires separate tooling | CSS, Tailwind, design systems (all separate) | **Design tokens in the grammar** — `.accent`, `.bold`, `.rounded` are language constructs |
| Passwords stored as plaintext | Runtime checks, linters (optional, ignorable) | **Security types** — `secret` auto-hashes, can't be logged or serialized. Compile error. |

## Key Features

### AI-First Error System

```
  BEFORE (typical compiler):                AFTER (Aura):
  error: unknown variable 'tood'            error[E0103]: unknown variable 'tood'
  error: type error in push()                 fix: replace with 'todos' (confidence: 0.97)
  error: cannot call done on Error            suppressed: 3 downstream errors
  error: type mismatch in view
  error: incompatible types
  = 5 errors, agent tries to fix all 5      = 1 error, agent auto-fixes
```

### Built-in Design Language

Design tokens are part of the grammar — not a library, not CSS classes.

```
column gap.md padding.lg          // Spacing: .xs .sm .md .lg .xl .2xl
  heading "Title" size.2xl .bold  // Typography: size + weight as tokens
  text "Subtitle" .secondary      // Color: semantic, theme-aware
  button "Save" .accent .pill     // Shape: .sharp .rounded .smooth .pill
    -> save()
```

Tokens resolve to platform-native values:
| Token | iOS (pt) | Android (dp) | Web (rem) |
|-------|----------|-------------|-----------|
| `.md` spacing | 8 | 8 | 0.5 |
| `.lg` spacing | 16 | 16 | 1.0 |
| `.xl` text | 20pt | 20sp | 1.25rem |

### Security Types

Types that enforce security at compile time:

```
model User
  name: text
  email: email          // format-validated
  password: secret      // auto-hashed, never in API response, can't be logged
  bio: sanitized        // XSS-safe
  api_key: token        // auto-expiring, never serialized

// These are COMPILE ERRORS, not warnings:
text "Password: {user.password}"     // E0202: secret in interpolation
api.respond(user)                    // E0200: model with secret in response
```

### Multi-Platform Native Output

```
                    ┌─── SwiftUI (iOS/macOS)
                    │
  .aura → AST → IR ├─── Jetpack Compose (Android)
                    │
                    ├─── HTML/CSS/JS (Web)
                    │
                    ├─── WinUI/XAML (Windows)
                    │
                    └─── Terminal UI (CLI)
```

Not a shared runtime. Not a webview wrapper. Each backend generates idiomatic, platform-native code.

## Architecture

```
aura/
├── spec/                        # Language specification (source of truth)
├── crates/
│   ├── aura-core/               # Lexer, Parser, AST, Types, HIR, LIR, Errors
│   ├── aura-cli/                # CLI: build, run, fmt, explain, diff, sketch
│   ├── aura-backend-web/        # → HTML/CSS/JS
│   ├── aura-backend-swift/      # → SwiftUI
│   ├── aura-backend-compose/    # → Jetpack Compose
│   ├── aura-backend-win/        # → WinUI
│   ├── aura-backend-tui/        # → Terminal UI
│   ├── aura-lsp/                # Language Server Protocol
│   ├── aura-agent/              # AI Agent API (structured AST mutation)
│   └── aura-pkg/                # Package manager
├── examples/                    # Example .aura programs
├── benchmarks/                  # AI agent benchmark suite
└── tests/conformance/           # Shared backend test suite
```

**Compiler pipeline:**
```
.aura source → Lexer (logos) → Parser (chumsky) → AST → Semantic Analysis → HIR → LIR → Backend
```

**Two-tier IR:** HIR preserves semantic intent (SwiftUI/Compose consume directly). LIR breaks down to rendering primitives (HTML/CSS/WinUI consume this).

## Getting Started

> Aura is in active development. The compiler is not yet functional — we're building the specification and core infrastructure.

```bash
# Clone
git clone https://github.com/360Labs-dev/aura.git
cd aura

# Build
cargo build

# Run tests
cargo test

# See the CLI
cargo run -- --help
```

## Examples

See the [`examples/`](examples/) directory:

- **[`minimal.aura`](examples/minimal.aura)** — Hello World (6 lines)
- **[`todo.aura`](examples/todo.aura)** — Todo app with components, filtering, design tokens
- **[`weather.aura`](examples/weather.aura)** — Weather display with animations
- **[`chat.aura`](examples/chat.aura)** — Chat app with tabs, navigation, auth
- **[`ecommerce.aura`](examples/ecommerce.aura)** — Full e-commerce with cart and security types

## Roadmap

| Phase | Timeline | Deliverables |
|-------|----------|-------------|
| **0. Specification** | Complete | Language spec, EBNF grammar, type system, design tokens |
| **1. Compiler Core** | In Progress | Lexer, parser, type checker, error poisoning |
| **2. Web Backend** | Planned | HTML/CSS/JS codegen, `aura build --target web`, `aura explain`, `aura diff` |
| **3. Mobile** | Planned | SwiftUI + Compose backends, `aura theme-from`, multi-platform preview |
| **4. AI Agent Platform** | Planned | Agent API, LSP, `aura sketch`, benchmarks |
| **5. Ecosystem** | Planned | Package manager, WinUI/TUI backends, public launch |

## Technical Decisions

- **Written in Rust** — logos (lexer), chumsky (parser), cranelift (future codegen)
- **Indentation-significant** — 2 spaces, no braces, no semicolons
- **Structural type system** with inference
- **Immutable by default** — state mutation only in `action` blocks
- **Error poisoning** — one typo = one error, not a cascade
- **Spec-first development** — `spec/language.md` is the source of truth

## Contributing

Aura is open source under MIT/Apache-2.0 dual license. We welcome contributions.

The best place to start:
1. Read the [language specification](spec/language.md)
2. Look at the [examples](examples/)
3. Check the [benchmark tasks](benchmarks/tasks.md) to understand what we're optimizing for

## License

MIT OR Apache-2.0
