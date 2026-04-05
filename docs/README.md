# Aura Docs

Aura is a UI-first language for AI coding agents. These docs focus on the parts of the project that are usable today from the owned CLI, backend, and agent surfaces.

## Start Here

- [Getting Started](./getting-started.md)
- [Building For Targets](./building-targets.md)
- [Agent API](./agent-api.md)
- [Aura vs TypeScript](./aura-vs-typescript.md)

## Reference App

- [`examples/launchpad.aura`](../examples/launchpad.aura) is the current reference example for the project workflow described in these docs.

## Current Reality

- `aura run` is the best day-one path because it builds and serves the web target with file watching.
- `aura build --target ios` and `aura build --target android` generate native output files, but the browser preview loop is currently the strongest experience.
- The docs only describe workflows that exist in the current repo.
