# Engineer 2 Task Sheet

## Mission
Own the developer experience, generated outputs, and adoption track so Aura feels usable and impressive to try.

## Ownership
- `crates/aura-cli/`
- `crates/aura-agent/`
- `crates/aura-backend-web/`
- `crates/aura-backend-swift/`
- `crates/aura-backend-compose/`
- `docs/`
- `examples/`

Do not edit:
- `crates/aura-core/`
- `spec/`
- `tests/conformance/`

## Core Rule
Consume compiler APIs exposed by Engineer 1. Do not reach into compiler internals unless explicitly agreed.

If a core API is missing, request it instead of creating a workaround in CLI or backends.

## Priority 1: CLI And Project Workflow
### Goal
Make Aura easy to create, build, run, and inspect as a real project.

### Tasks
1. Refactor CLI commands to use project-aware compiler APIs from Engineer 1.
2. Make `aura build` work cleanly from project root.
3. Make `aura run` reliable for project-based apps.
4. Improve `aura init` so generated projects are immediately runnable.
5. Improve command output and failure messages.

### Deliverables
- Project-root workflow for:
  - `aura init`
  - `aura build`
  - `aura run`
  - `aura fmt`
  - `aura doctor`

### Definition of Done
- A new user can initialize and run an Aura app without hand-editing file paths.

## Priority 2: Web Backend Production Pass
### Goal
Make web output strong enough to serve as Aura's first serious shipping target.

### Tasks
1. Improve generated HTML/CSS/JS structure and readability.
2. Strengthen support for:
  - state updates
  - forms
  - loops
  - conditionals
  - actions
  - navigation
3. Improve generated styling output and design-token usage.
4. Add or improve source-map/debug hooks if supported by current core APIs.

### Deliverables
- Cleaner output structure.
- Better behavior in non-trivial example apps.
- Snapshot updates and backend tests where needed.

### Definition of Done
- A medium-complexity demo app runs well in browser without manual patching.

## Priority 3: Native Backend Parity
### Goal
Keep SwiftUI and Compose credible for the currently supported Aura feature set.

### Tasks
1. Keep Swift and Compose codegen aligned with web for supported features.
2. Expand coverage for:
  - models
  - state
  - actions
  - lists/each
  - buttons
  - layout
  - basic navigation
3. Add snapshot tests for important output changes.

### Deliverables
- Fewer backend mismatches for common app patterns.
- Snapshot coverage for newly supported constructs.

### Definition of Done
- The same conformance fixtures generate sensible output across web, Swift, and Compose.

## Priority 4: Agent API Maturation
### Goal
Make Aura feel AI-native, not just compiler-driven.

### Tasks
1. Strengthen JSON-RPC ergonomics and error handling.
2. Add project-aware agent requests when Engineer 1 exposes project APIs.
3. Improve existing methods:
  - `diagnostics.get`
  - `completions.get`
  - `hir.get`
  - `hover`
  - `goto.definition`
4. Keep request/response shapes stable and well-documented.

### Deliverables
- Cleaner server behavior for real project inputs.
- Tests for project-aware requests when available.

### Definition of Done
- Agent server is useful for tooling and editor integrations, not just demos.

## Priority 5: Docs, Examples, And Adoption
### Goal
Make Aura easy to understand and compelling to try.

### Tasks
1. Create one polished reference app that shows why Aura beats TypeScript for UI-heavy work.
2. Update docs for:
  - getting started
  - creating a project
  - building for web/iOS/android
  - using the agent API
  - Aura vs TypeScript
3. Keep examples aligned with currently working language features.

### Deliverables
- Strong demo app.
- Docs a new user can actually follow.
- Examples that compile and look good.

### Definition of Done
- A new user can understand Aura's value in under 15 minutes.

## Expected Dependencies From Engineer 1
You should plan to consume:
- project loading API
- project analysis API
- project/HIR build API
- incremental cache/check API

Do not block on perfect final versions. Ask for minimal stable interfaces early.

## Coordination Rules
- Do not edit `aura-core` to fix backend or CLI problems without approval from Engineer 1.
- If you hit a compiler limitation, write it down as an API request with a concrete use case.
- Keep backend changes isolated by crate.
- Keep docs honest: only document what works now.

## Weekly Suggested Order
### Week 1
- CLI cleanup
- `init` polish
- `run` polish

### Week 2
- project-aware CLI integration once APIs land
- better example app foundation

### Week 3
- web backend production pass
- improved snapshots

### Week 4
- Swift/Compose parity work
- backend polish

### Week 5
- agent API project-awareness
- docs pass

### Week 6
- polished demo app
- final adoption docs
- warning reduction in owned modules

## Success Criteria
You are done when Aura feels pleasant to try: initialize a project, run it, inspect it, generate web/native outputs, and see a strong demo that makes the case for Aura as a TypeScript alternative for app UI work.
