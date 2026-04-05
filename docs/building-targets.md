# Building For Targets

Aura can generate multiple target outputs from the same project.

## Web

```bash
cargo run --offline -p aura-cli -- build --target web
```

Output goes to `build/` by default:

- `index.html`
- `styles.css`
- `app.js`

For day-to-day work, prefer:

```bash
cargo run --offline -p aura-cli -- run
```

That gives you the watched web preview loop.

## iOS

```bash
cargo run --offline -p aura-cli -- build --target ios
```

Aura generates SwiftUI source into the build directory. This is useful for inspecting the generated structure and validating that your Aura app stays aligned with the current native backend coverage.

## Android

```bash
cargo run --offline -p aura-cli -- build --target android
```

Aura generates Jetpack Compose source into the build directory.

## All Targets

```bash
cargo run --offline -p aura-cli -- build --target all
```

Use this when you want a quick cross-target sanity check from one command.

## Project-Aware Paths

These all work:

```bash
cargo run --offline -p aura-cli -- build .
cargo run --offline -p aura-cli -- build ./src/main.aura
cargo run --offline -p aura-cli -- build /absolute/path/to/project
```

Aura resolves the project root and writes generated output relative to that project.

## Honest Target Guidance

- Web is the best-supported shipping path today.
- Native outputs are useful and increasingly aligned, especially for common UI constructs like state, buttons, lists, layout, segmented controls, and text input.
- If you are evaluating Aura for the first time, start with web preview and then inspect the generated SwiftUI and Compose outputs.
