# Aura vs TypeScript

Aura is trying to remove the glue code that usually surrounds UI-heavy app work.

## What Aura Compresses

In a typical TypeScript UI stack, you spread one feature across:

- component files
- state management code
- model types
- styling files or utility classes
- event handlers
- framework-specific routing or navigation

Aura puts those concerns into one language:

- models
- screens
- components
- state
- actions
- design tokens

## Why That Matters

For AI coding agents and fast UI iteration, fewer layers means:

- less context switching
- fewer chances to drift between types and UI
- easier project-level inspection
- cleaner multi-target generation

## A Concrete Example

Open [`examples/launchpad.aura`](../examples/launchpad.aura).

That single file defines:

- the data model
- reusable row UI
- local state
- filtering logic
- state mutations
- design intent

In a TypeScript stack, the same feature usually becomes a small folder of files. In Aura, it stays readable as one unit of product intent.

## Honest Limits Today

Aura is still early. Right now:

- web is the most complete target
- native generation is useful but not yet a full production pipeline
- some advanced examples in the repo are ahead of backend fidelity

The value proposition is already clear, though: Aura can express UI-heavy product logic more directly than a conventional TypeScript stack, especially when an AI agent needs to reason across the whole feature at once.
