<p align="center">
  <img src="https://img.shields.io/badge/lang-Aura-6C5CE7?style=for-the-badge" alt="Aura Language"/>
  <img src="https://img.shields.io/badge/status-alpha-orange?style=for-the-badge" alt="Alpha"/>
  <img src="https://img.shields.io/badge/platforms-Web%20%7C%20iOS%20%7C%20Android-blue?style=for-the-badge" alt="Platforms"/>
  <img src="https://img.shields.io/badge/tests-120%20passing-brightgreen?style=for-the-badge" alt="Tests"/>
  <img src="https://img.shields.io/npm/v/aura-lang?style=for-the-badge&color=CB3837" alt="npm"/>
  <img src="https://img.shields.io/github/license/360Labs-dev/aura?style=for-the-badge" alt="License"/>
</p>

# Aura

### Build native apps with AI, not boilerplate.

<p align="center">
  <strong>One compact source file in. Real web, iOS, and Android apps out.</strong>
</p>

<p align="center">
  Aura is an AI-native language and compiler for product UI. It gives coding agents a smaller search space, a stronger compiler contract, first-class design tokens, compile-time security types, and platform-native code generation.
</p>

<p align="center">
  <strong>Fewer tokens. Better first compile success. Native output on every platform.</strong>
</p>

Aura is built for the way agents actually generate software. Instead of spreading intent across components, styles, state containers, and platform glue, Aura lets the model express the product directly and compile it into SwiftUI, Jetpack Compose, and HTML/CSS/JS.

**What makes Aura different**

- **One source file to multiple native targets** with no shared runtime
- **Design built into the language** instead of bolted on through CSS or widget APIs
- **Compiler diagnostics for agents** with error suppression and fix confidence
- **Security types by default** for secrets, tokens, URLs, emails, and sanitized content

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

**That file compiles to native SwiftUI, Jetpack Compose, and HTML/CSS/JS** from a single source file. No shared runtime. No webview. No style framework duct-taped on later.

---

## Why Aura Exists

AI coding agents are fast. Modern app stacks are not.

Most UI stacks force agents to spend tokens and retries on framework glue before they can even express the product:

- **Too much boilerplate** before the first line of product logic
- **Cascading diagnostics** from one small mistake
- **Separate platform stacks** for web, iOS, and Android
- **Design split across another language** or another abstraction layer
- **Security left to conventions** instead of types and compiler checks

Aura gives the model a smaller target, clearer semantics, and a more reliable path from prompt to working app.

## Why It Wins

| | Typical stack | Aura |
|---|---|---|
| **Simple app size** | Multiple files and framework glue | **One compact source file** |
| **Cross-platform output** | Separate stacks or shared runtime | **Native codegen per platform** |
| **Design system** | CSS, Tailwind, modifiers, or theme APIs | **Built into the grammar** |
| **Error recovery** | Text diagnostics and cascades | **Suppressed cascades + fix suggestions** |
| **Security model** | Lint rules and runtime checks | **Compile-time security types** |
| **AI token cost** | High | **Much lower on UI-heavy tasks** |
| **Readability** | Framework-specific ceremony | **Close to product intent** |

---

## What Makes Aura Different

### 1. Error Poisoning (No More Error Cascades)

This is the single biggest improvement for AI coding agents.

In every other language, one typo produces a cascade of errors:

```
// TypeScript: 1 typo -> 5 errors
error TS2304: Cannot find name 'todoos'          ← root cause
error TS2345: Argument of type 'never' not...     ← cascade
error TS2339: Property 'done' does not exist...   ← cascade
error TS2322: Type 'never' is not assignable...   ← cascade
error TS7006: Parameter 'todo' implicitly has...  ← cascade
```

The AI sees 5 errors and tries to fix all 5. It makes 4 wrong changes that break more things.

```
// Aura: 1 typo -> 1 error
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

In TypeScript, you'd need ESLint rules that developers can disable. In Aura, it's **impossible to write insecure code** because the compiler prevents it.

### 4. One File, Three Native Platforms

```bash
$ aura build --target all

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
$ aura sketch "chat messenger app"    # Full chat UI with messages, input, timestamps
$ aura sketch "weather forecast"      # Weather display with icons, forecast grid
$ aura sketch "login screen"          # Auth form with email/password, security types
```

Every generated file compiles and runs immediately.

---

## Live Case Studies

**[See all case studies](https://apps-9fdnzbeqa-100rabhkrs-projects.vercel.app)**

Six apps generated with `aura sketch` and compiled with `aura build --target web`:

| App | Generated With | Lines | Demo |
|---|---|---|---|
| Chat Messenger | `aura sketch "chat app with dark mode"` | 28 | [Live](https://apps-9fdnzbeqa-100rabhkrs-projects.vercel.app/chat/) |
| Weather Forecast | `aura sketch "weather forecast"` | 30 | [Live](https://apps-9fdnzbeqa-100rabhkrs-projects.vercel.app/weather/) |
| Fitness Timer | `aura sketch "timer app with dark mode"` | 28 | [Live](https://apps-9fdnzbeqa-100rabhkrs-projects.vercel.app/timer/) |
| Photo Gallery | `aura sketch "photo gallery with dark mode"` | 24 | [Live](https://apps-9fdnzbeqa-100rabhkrs-projects.vercel.app/gallery/) |
| Login Screen | `aura sketch "login with dark mode"` | 30 | [Live](https://apps-9fdnzbeqa-100rabhkrs-projects.vercel.app/login/) |
| Settings Panel | `aura sketch "settings with dark mode"` | 30 | [Live](https://apps-9fdnzbeqa-100rabhkrs-projects.vercel.app/settings/) |

Every app above was:
1. Generated from a single English sentence by `aura sketch`
2. Compiled to HTML/CSS/JS by `aura build --target web`
3. Deployed as-is with no manual edits

---

## Quick Start

```bash
# Install from npm (language reference + syntax highlighting)
npm install aura-lang

# Install the compiler (requires Rust)
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
cd myapp && ../target/release/aura run
```

---

## All Commands

| Command | Description |
|---|---|
| `aura build --target web` | Compile the current Aura project to reactive HTML/CSS/JS |
| `aura build --target ios` | Compile the current Aura project to SwiftUI |
| `aura build --target android` | Compile the current Aura project to Jetpack Compose |
| `aura build --target all` | Build the current Aura project for all three platforms |
| `aura build path/to/file.aura --format json` | JSON error output for AI agents |
| `aura run` | Dev server on localhost:3000 |
| `aura sketch "description"` | Generate .aura from English |
| `aura init myapp` | Scaffold new project |
| `aura fmt` | Format all Aura files in the current project |
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

## Benchmarks

Real measurements from `cargo run --release --bin aura-bench`:

### Compilation Speed

Aura compiles to **3 native platforms simultaneously** in microseconds:

| Benchmark | Parse | Analyze | HIR | Codegen (3 targets) | **Total** |
|---|---|---|---|---|---|
| Hello World | 210 us | 58 us | 36 us | 166 us | **470 us** |
| Counter App | 29 us | 4 us | 8 us | 71 us | **112 us** |
| Todo List | 39 us | 11 us | 6 us | 56 us | **112 us** |

**100% first-compile success rate** with zero parse errors across all benchmarks.

### Code Size: Aura vs. Everyone Else

Same app, four languages. Lines (L) and tokens (T):

| App | Aura | TypeScript + React Native | Swift + SwiftUI | Kotlin + Compose |
|---|---|---|---|---|
| Hello World | **4L / 8T** | 15L / 46T | 16L / 31T | 18L / 33T |
| Counter | **14L / 47T** | 32L / 133T | 34L / 67T | 46L / 110T |
| Todo List | **27L / 74T** | 67L / 256T | 49L / 122T | 53L / 222T |
| **Total** | **45L / 129T** | 114L / 435T | 99L / 220T | 117L / 365T |

### Token Reduction (= LLM Cost Reduction)

Every token an AI generates costs money and time. Aura cuts that dramatically:

```
  vs TypeScript + React Native:    70% fewer tokens
  vs Swift + SwiftUI:              41% fewer tokens
  vs Kotlin + Jetpack Compose:     65% fewer tokens
```

**For AI coding agents, 70% fewer tokens means 70% cheaper and 70% faster.**

### Output Size

Each Aura program generates platform-native output:

| App | Web (HTML+CSS+JS) | iOS (Swift) | Android (Kotlin) |
|---|---|---|---|
| Hello World | 6,642 bytes | 272 bytes | 865 bytes |
| Counter | 7,167 bytes | 912 bytes | 1,355 bytes |
| Todo List | 7,734 bytes | 1,478 bytes | 1,973 bytes |

Run benchmarks yourself: `cargo run --release --bin aura-bench`

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
4. Run `cargo test`. All 120 tests should pass.
5. Pick an issue or propose a feature

---

<p align="center">
  <b>Built by <a href="https://github.com/360Labs-dev">360 Labs</a></b>
  <br/>
  MIT License
</p>
