# Getting Started

Aura lets you define UI, state, actions, and models in one `.aura` file and compile that program to multiple targets.

## Prerequisites

- Rust and Cargo
- A browser for web preview
- `xcodebuild` if you want to inspect generated iOS output
- Android tooling if you want to inspect generated Android output

`aura doctor` will tell you which optional native tools are available on your machine.

## Create A Project

```bash
cargo run --offline -p aura-cli -- init hello-aura
cd hello-aura
```

That creates:

- `src/main.aura` for your app entry point
- `build/` for generated output
- `.aura-cache/` for incremental build metadata

## Preview The App

Use the web target for the fastest loop:

```bash
cargo run --offline -p aura-cli -- run
```

`aura run` watches the project root, rebuilds web output into `build/dev/`, and serves a preview on port `3000` by default.

## Build Explicitly

```bash
cargo run --offline -p aura-cli -- build --target web
cargo run --offline -p aura-cli -- build --target ios
cargo run --offline -p aura-cli -- build --target android
cargo run --offline -p aura-cli -- build --target all
```

If you pass a project directory, Aura resolves the project root automatically. If you pass a single file, Aura still writes output relative to the containing project.

## Format And Check The Environment

```bash
cargo run --offline -p aura-cli -- fmt .
cargo run --offline -p aura-cli -- fmt . --check
cargo run --offline -p aura-cli -- doctor
```

## Learn From A Real Example

The best current example for the project workflow is:

- [`examples/launchpad.aura`](../examples/launchpad.aura)

It shows:

- models
- state
- components
- actions
- list rendering
- filters
- segmented controls
- toggles

## What To Expect Today

- Web is the most complete target for trying Aura end to end.
- SwiftUI and Compose generation are improving and cover common app patterns, but they are still generated source outputs rather than a full native runtime workflow.
- The Agent API can inspect source or whole projects through JSON-RPC.
