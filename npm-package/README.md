# aura-lang

**The Aura programming language** — AI-first, cross-platform, with a built-in design system.

Write once. Compile to native **SwiftUI** (iOS), **Jetpack Compose** (Android), and **HTML/CSS/JS** (Web).

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
          button "Add" .accent -> addTodo(input)
        each todos as todo
          row gap.md align.center
            checkbox todo.done
            text todo.title strike: todo.done
```

## Why Aura?

| | TypeScript+React Native | Aura |
|---|---|---|
| Lines for a todo app | ~67 | **~27** |
| Tokens (LLM cost) | ~256 | **~74** (70% less) |
| One typo → errors | 5 cascading errors | **1 error** + confidence-scored fix |
| Cross-platform | Shared runtime | **Native codegen** per platform |
| Design system | Import Tailwind | **Built into grammar** |
| Password as string? | Allowed | **Compile error** |

## Benchmarks

Real measurements — same app, four languages:

| App | Aura | TypeScript+RN | Swift+SwiftUI | Kotlin+Compose |
|---|---|---|---|---|
| Hello World | **4L / 8T** | 15L / 46T | 16L / 31T | 18L / 33T |
| Counter | **14L / 47T** | 32L / 133T | 34L / 67T | 46L / 110T |
| Todo List | **27L / 74T** | 67L / 256T | 49L / 122T | 53L / 222T |
| **Total** | **45L / 129T** | 114L / 435T | 99L / 220T | 117L / 365T |

```
Token reduction vs TypeScript+RN:    70% fewer tokens
Token reduction vs Swift+SwiftUI:    41% fewer tokens
Token reduction vs Kotlin+Compose:   65% fewer tokens
```

**Compilation speed:** ~112 us to compile a counter app to 3 native platforms (Web + iOS + Android). **100% first-compile success rate.**

For AI agents, 70% fewer tokens = 70% cheaper inference costs.

## Install the Compiler

The Aura compiler is written in Rust:

```bash
git clone https://github.com/360Labs-dev/aura.git
cd aura
cargo build --release
```

## What This Package Includes

- **TextMate grammar** — syntax highlighting for any editor
- **Example programs** — `.aura` files you can compile
- **Language reference** — keywords, types, design tokens exported as JS objects

```js
const aura = require('aura-lang');

// TextMate grammar path (for editor integration)
aura.grammarPath  // → .../syntaxes/aura.tmLanguage.json

// Language metadata
aura.keywords     // ['app', 'screen', 'view', 'model', ...]
aura.types        // ['text', 'int', 'secret', 'email', ...]
aura.designTokens // { spacing: ['xs','sm','md',...], color: ['accent',...], ... }
aura.viewElements // { layout: ['column','row',...], widgets: ['text',...], ... }
```

## Commands

```bash
aura build --target web                 # → HTML/CSS/JS
aura build --target ios                 # → SwiftUI
aura build --target android             # → Compose
aura build --target all                 # → all three
aura sketch "todo app with dark mode"   # → generate from English
aura run                                # → dev server
aura explain app.aura                   # → plain English
aura agent serve                        # → JSON-RPC for AI agents
```

## Links

- **GitHub**: https://github.com/360Labs-dev/aura
- **Language Spec**: https://github.com/360Labs-dev/aura/blob/main/spec/language.md
- **VS Code Extension**: `editors/vscode/` in the repo

## License

MIT
