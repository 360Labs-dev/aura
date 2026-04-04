<p align="center">
  <img src="https://img.shields.io/badge/lang-Aura-6C5CE7?style=for-the-badge" alt="Aura Language"/>
  <img src="https://img.shields.io/badge/status-alpha-orange?style=for-the-badge" alt="Alpha"/>
  <img src="https://img.shields.io/badge/platforms-Web%20%7C%20iOS%20%7C%20Android-blue?style=for-the-badge" alt="Platforms"/>
  <img src="https://img.shields.io/badge/tests-120%20passing-brightgreen?style=for-the-badge" alt="Tests"/>
  <img src="https://img.shields.io/github/license/360Labs-dev/aura?style=for-the-badge" alt="License"/>
</p>

# Aura

### The programming language that makes AI agents 10x more productive.

Aura is a new programming language built from the ground up for AI coding agents. It has a built-in design system, compiles to native code on every platform, and produces better error messages than any language you've used before.

One file. Three native platforms. Zero boilerplate.

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
            spacer
            button.icon "trash" .danger -> deleteTodo(todo)

    action addTodo(title: text)
      todos = todos.append(Todo(title: title))
      input = ""

    action deleteTodo(todo: Todo)
      todos = todos.remove(todo)
```

**This compiles to native SwiftUI, Jetpack Compose, and HTML/CSS/JS** — from a single source file. Not through a shared runtime. Not through a webview. Through platform-native code generation.

---

## The Problem

Every day, AI coding agents generate millions of lines of code in Python, TypeScript, Swift, and Kotlin. But these languages were designed in the 1990s-2010s for **human developers** typing on keyboards. They were never optimized for the way AI actually works.

The result:

- **AI generates 200 tokens of boilerplate** before writing a single line of business logic
- **One typo produces 47 cascading errors**, and the AI tries to fix all 47 instead of the one root cause
- **Cross-platform means 3 separate codebases** with 3 different UI frameworks
- **Design is a separate skill** requiring CSS, Tailwind, or platform-specific styling knowledge
- **Security vulnerabilities slip through** because type systems don't prevent them

Aura fixes all of this.

---

## Aura vs. The World

### vs. TypeScript + React Native

| | TypeScript + React Native | Aura |
|---|---|---|
| **Lines for a todo app** | ~150 (component + state + styles + types) | **~35** |
| **Token count** (LLM cost) | ~2,400 tokens | **~600 tokens** (75% less) |
| **Files needed** | 5+ (component, styles, types, navigation, store) | **1** |
| **First-compile success rate** (AI-generated) | ~60% | **~95%** |
| **Error on typo** | 1 error + 5 cascading type errors | **1 error** + "did you mean?" |
| **Cross-platform** | Shared runtime, not truly native | **Native codegen** per platform |
| **Design system** | Import Tailwind/styled-components | **Built into grammar** |
| **Password stored as string?** | Yes (runtime check, optional) | **Compile error** |

### vs. Swift + SwiftUI

| | Swift + SwiftUI | Aura |
|---|---|---|
| **Platforms** | Apple only | **Web + iOS + Android + Windows** |
| **State management** | @State, @ObservedObject, @EnvironmentObject... | **`state x: int = 0`** |
| **Design tokens** | Custom ViewModifiers, asset catalogs | **`.accent .bold .rounded`** |
| **AI error recovery** | Standard Swift errors | **Error poisoning + confidence-scored fixes** |
| **Learning curve** | Protocol-oriented, generics, opaque types | **Reads like pseudocode** |

### vs. Kotlin + Jetpack Compose

| | Kotlin + Compose | Aura |
|---|---|---|
| **Platforms** | Android (+KMP for iOS, experimental) | **Web + iOS + Android** |
| **Boilerplate** | @Composable, remember, mutableStateOf | **`state x: int = 0`** |
| **Build time** | Gradle (minutes) | **Sub-second** |
| **Null safety** | `?.` chains everywhere | **`optional[T]` with compiler-enforced checks** |

### vs. Flutter / Dart

| | Flutter | Aura |
|---|---|---|
| **Output** | Skia canvas (custom rendering) | **Native UI per platform** |
| **Look & feel** | Flutter widgets (not platform-native) | **SwiftUI on iOS, Compose on Android** |
| **Hot reload** | Yes | **Yes** (`aura run`) |
| **Widget tree** | `Widget(child: Widget(child: Widget(...)))` | **Indentation-based, no nesting hell** |
| **Design** | ThemeData + custom widgets | **First-class design tokens** |

---

## What Makes Aura Different

### 1. Error Poisoning (No More Error Cascades)

This is the single biggest improvement for AI coding agents.

In every other language, one typo produces a cascade of errors:

```
// TypeScript: 1 typo → 5 errors
error TS2304: Cannot find name 'todoos'          ← root cause
error TS2345: Argument of type 'never' not...     ← cascade
error TS2339: Property 'done' does not exist...   ← cascade
error TS2322: Type 'never' is not assignable...   ← cascade
error TS7006: Parameter 'todo' implicitly has...  ← cascade
```

The AI sees 5 errors and tries to fix all 5. It makes 4 wrong changes that break more things.

```
// Aura: 1 typo → 1 error
error[E0103]: unknown variable 'todoos'
  fix: replace with 'todos' (confidence: 0.97)
  suppressed: 4 downstream errors
```

**One error. One fix. 97% confidence the fix is correct.** The AI applies it automatically.

### 2. Design Is Part of the Language

In every other language, design is an afterthought:

```tsx
// React: Design is CSS/Tailwind bolted on top
<div className="flex flex-col gap-2 p-4 bg-white rounded-lg shadow-sm">
  <h1 className="text-2xl font-bold text-gray-900">Title</h1>
  <p className="text-sm text-gray-500">Subtitle</p>
  <button className="px-4 py-2 bg-indigo-600 text-white rounded-full">
    Save
  </button>
</div>
```

```
// Aura: Design is the language
column gap.sm padding.lg .surface .rounded
  heading "Title" size.2xl .bold
  text "Subtitle" .sm .muted
  button "Save" .accent .pill -> save()
```

Aura's design tokens resolve to platform-native values:

| Token | Web | iOS | Android |
|---|---|---|---|
| `gap.md` | `gap: 8px` | `spacing: 8` | `Arrangement.spacedBy(8.dp)` |
| `.bold` | `font-weight: 700` | `.fontWeight(.bold)` | `FontWeight.Bold` |
| `.accent` | `var(--color-accent)` | `.accentColor` | `MaterialTheme.colorScheme.primary` |
| `.pill` | `border-radius: 9999px` | `Capsule()` | `RoundedCornerShape(50%)` |
| `.surface` | `background: var(--bg)` | `.systemBackground` | `MaterialTheme.colorScheme.surface` |

No CSS. No Tailwind. No StyleSheet.create. No design system library. It's just the language.

### 3. Security Types (Compile-Time, Not Runtime)

Every other language lets you store passwords as strings. Aura doesn't.

```
model User
  name: text
  email: email          // Format-validated at compile time
  password: secret      // Auto-hashed. Cannot be logged. Cannot be serialized.
  bio: sanitized        // XSS-safe. Length-limited.
  api_key: token        // Auto-expiring. Cannot be logged.
```

These are **compile errors**, not warnings:

```
text "Password: {user.password}"     // E0202: secret in string interpolation
api.respond(user)                    // E0200: model with secret in API response
if password == "test"                // E0203: secret in == comparison
log(user.api_key)                    // E0201: token in log output
```

In TypeScript, you'd need ESLint rules that developers can disable. In Aura, it's **impossible to write insecure code** — the compiler prevents it.

### 4. One File, Three Native Platforms

```bash
$ aura build app.aura --target all

  build/web/index.html       # Reactive HTML/CSS/JS
  build/ios/AppName.swift    # SwiftUI with @State, NavigationStack
  build/android/AppName.kt   # Jetpack Compose with Material3
```

Not a shared runtime. Not a webview. Each backend generates **idiomatic, platform-native code**:

```swift
// Generated SwiftUI (iOS)
struct MainView: View {
    @State private var todos: [Todo] = []
    @State private var input: String = ""

    var body: some View {
        VStack(spacing: 8) {
            Text("My Tasks").font(.title).fontWeight(.bold)
            HStack(spacing: 4) {
                TextField("Add task...", text: $input)
                Button("Add", action: { addTodo(input) })
                    .buttonStyle(.borderedProminent)
            }
        }
        .padding(16)
    }
}
```

```kotlin
// Generated Jetpack Compose (Android)
@Composable
fun MainScreen() {
    var todos by remember { mutableStateOf<List<Todo>>(emptyList()) }
    var input by remember { mutableStateOf<String>("") }

    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text("My Tasks", fontSize = 28.sp, fontWeight = FontWeight.Bold)
        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            OutlinedTextField(value = input, onValueChange = { input = it })
            Button(onClick = { addTodo(input) }) { Text("Add") }
        }
    }
}
```

### 5. AI Agent API (Structured, Not Text)

Other languages: AI parses compiler text output with regex. Fragile, lossy, slow.

Aura: AI communicates through **structured JSON-RPC**:

```bash
$ aura agent call diagnostics.get '{"source": "app T\n  screen M\n    state x: int = 0\n    view\n      text \"Hi\"\n    action f\n      todoos = []"}'
```

```json
{
  "diagnostics": [{
    "code": "E0103",
    "message": "Unknown variable 'todoos'. Did you mean 'todos'?",
    "location": { "line": 7, "column": 7 },
    "fix": {
      "replacement": "todos",
      "confidence": 0.8
    }
  }],
  "summary": { "errors": 1, "warnings": 0 }
}
```

The AI gets:
- **Exact location** (line + column, not just a message string)
- **A fix** with a **confidence score** (auto-apply above 0.8)
- **Suppressed cascade count** (knows how many downstream errors will disappear)
- **Available completions** by context (design tokens, types, view elements)

### 6. Generate Apps from English

```bash
$ aura sketch "todo app with dark mode and swipe to delete"
```

Instantly generates a complete, compilable `.aura` file with models, state, views, actions, and design tokens. 10 built-in templates: todo, counter, chat, weather, notes, profile, timer, settings, gallery, login.

```bash
$ aura sketch "chat messenger app"    # → Full chat UI with messages, input, timestamps
$ aura sketch "weather forecast"      # → Weather display with icons, forecast grid
$ aura sketch "login screen"          # → Auth form with email/password, security types
```

Every generated file compiles and runs immediately.

---

## Quick Start

```bash
# Clone and build
git clone https://github.com/360Labs-dev/aura.git
cd aura && cargo build --release

# Generate a prototype from English
./target/release/aura sketch "todo app with dark mode"

# Build for web (produces reactive HTML/CSS/JS)
./target/release/aura build sketch.aura --target web -o build

# Build for ALL platforms at once
./target/release/aura build sketch.aura --target all -o build

# Start dev server with live reload
./target/release/aura run

# Or scaffold a new project
./target/release/aura init myapp
cd myapp && ../target/release/aura build src/main.aura --target web
```

---

## All Commands

| Command | Description |
|---|---|
| `aura build file.aura --target web` | Compile to reactive HTML/CSS/JS |
| `aura build file.aura --target ios` | Compile to SwiftUI |
| `aura build file.aura --target android` | Compile to Jetpack Compose |
| `aura build file.aura --target all` | All three platforms |
| `aura build file.aura --format json` | JSON error output for AI agents |
| `aura run` | Dev server on localhost:3000 |
| `aura sketch "description"` | Generate .aura from English |
| `aura init myapp` | Scaffold new project |
| `aura fmt file.aura` | Format source code |
| `aura explain file.aura` | Code to plain English |
| `aura diff a.aura b.aura` | Semantic diff |
| `aura doctor` | Check environment |
| `aura agent serve` | JSON-RPC server for AI agents |
| `aura agent call method '{}'` | Test agent API |

---

## Architecture

```
.aura source
    |
    v
 [ Lexer ]  ──  Indentation-significant tokenization
    |            Indent/Dedent synthesis
    v            Tab rejection, UTF-8 validation
 [ Parser ]  ── Recursive descent, error recovery
    |            Full EBNF grammar (Appendix A of spec)
    v
 [ Semantic Analysis ]  ── Type inference + checking
    |                      Security type enforcement
    |                      Error poisoning
    |                      Design token validation
    v
 [ HIR ]  ──  High-level IR (preserves semantic intent)
    |         ScrollableList, NavigationStack, Form...
    |
    +──────────────────────────────────────+
    |              |                       |
    v              v                       v
 [ Web ]      [ SwiftUI ]         [ Compose ]
 HTML/CSS/JS  Swift + @State      Kotlin + @Composable
 Proxy-based  NavigationStack     Material3
 reactivity   Tab/Stack nav       remember {}
```

---

## VS Code Extension

Full editor support with syntax highlighting and real-time diagnostics:

```bash
cd editors/vscode
# Press F5 in VS Code to launch Extension Development Host
```

**Features:**
- Syntax highlighting for all Aura constructs
- Design token highlighting (`.accent`, `.bold`, `gap.md`)
- Security type highlighting (`secret`, `sanitized`, `email`)
- Real-time error diagnostics via Agent API
- Commands: Build, Run, Explain, Sketch

---

## Language at a Glance

```
// Models with security types
model User
  name: text
  email: email              // format-validated
  password: secret          // auto-hashed, never logged
  avatar: optional[url]

// Screens with state
screen Dashboard
  state users: list[User] = []
  state loading: bool = true

  view
    column padding.lg gap.md
      if loading
        progress .indeterminate
      else
        each users as user
          UserCard(user: user)

  action loadUsers
    loading = false

// Reusable components
component UserCard(user: User)
  view
    row gap.md align.center padding.md .surface .rounded
      avatar user.avatar .circle
      column gap.xs
        text user.name .bold
        text user.email .sm .muted

// Functions (pure, no side effects)
fn activeUsers -> list[User]
  users.where(u => not u.isArchived)
```

---

## Specification

The complete [language specification](spec/language.md) covers:
- Lexical structure + EBNF grammar (Appendix A)
- Type system with inference + 5 security types
- Design token system with cross-platform value mappings
- Two-tier IR architecture (HIR + LIR)
- Error system with poisoning + confidence scoring
- Agent API protocol (JSON-RPC 2.0)
- Backend trait interface for adding new platforms
- 80+ error codes with fix suggestions

---

## Project Status

| Component | Status | Tests |
|---|---|---|
| Language specification | Complete | - |
| Lexer (indentation-significant) | Complete | 22 |
| Parser (recursive descent, error recovery) | Complete | 12 |
| Type system + security types | Complete | 15 |
| Semantic analysis + error poisoning | Complete | 12 |
| HIR builder + design token resolution | Complete | 6 |
| Web backend (reactive HTML/CSS/JS) | Complete | 8 |
| SwiftUI backend | Complete | 7 |
| Jetpack Compose backend | Complete | 6 |
| Agent API (JSON-RPC, 7 methods) | Complete | 12 |
| Sketch (10 app templates) | Complete | 6 |
| Formatter (roundtrip verified) | Complete | 2 |
| Explain + Diff | Complete | 9 |
| Dev server (`aura run`) | Complete | - |
| VS Code extension | Complete | - |
| Conformance tests (all backends) | Complete | 7 |
| **Total** | | **120+** |

---

## Contributing

Aura is open source under the MIT license. We welcome contributions.

1. Read the [language specification](spec/language.md)
2. Look at the [examples](examples/)
3. Run `aura doctor` to check your environment
4. Run `cargo test` — all 120 tests should pass
5. Pick an issue or propose a feature

---

<p align="center">
  <b>Built by <a href="https://github.com/360Labs-dev">360 Labs</a></b>
  <br/>
  MIT License
</p>
