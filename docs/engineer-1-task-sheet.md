# Engineer 1 Task Sheet

## Mission
Own the compiler and language-core track so Aura becomes a reliable multi-file language toolchain.

## Ownership
- `crates/aura-core/`
- `spec/`
- `tests/conformance/`

Do not edit:
- `crates/aura-cli/`
- `crates/aura-agent/`
- `crates/aura-backend-web/`
- `crates/aura-backend-swift/`
- `crates/aura-backend-compose/`
- `docs/`
- `examples/`

## Core Rule
You own public compiler APIs. Engineer 2 should consume them, not rework internals.

If an API needs to change, publish the change in a small PR first, with a short usage example.

## Priority 1: Real Project And Module System
### Goal
Move Aura from single-file compilation to real project compilation.

### Tasks
1. Implement project loading from `aura.toml`.
2. Support multi-file discovery under `src/`.
3. Implement import resolution and module naming.
4. Detect duplicate modules and circular imports.
5. Expose a stable project API from `aura-core`.

### Deliverables
- A `Project` model that represents all files and resolved imports.
- A compiler entry path that can parse and analyze a whole project.
- Tests covering:
  - happy path multi-file app
  - missing import
  - duplicate module
  - circular import

### Definition of Done
- A project with 3-5 `.aura` files compiles from the project root.
- Conformance includes at least one multi-file project case.

## Priority 2: Type System Hardening
### Goal
Replace temporary shortcuts with first-class type behavior.

### Tasks
1. Replace the current union-type shortcut with a real AST representation.
2. Add semantic support for union types.
3. Thread unions through HIR contracts where needed.
4. Tighten optional, collection, and inference behavior where current logic is weak.

### Deliverables
- Real union-type nodes in parser and semantic analysis.
- Tests for:
  - valid unions
  - invalid unions
  - union assignment
  - union narrowing if applicable

### Definition of Done
- No string-based union fallback remains in parser/type handling.

## Priority 3: Error System Upgrade
### Goal
Make compiler diagnostics a major advantage over TypeScript.

### Tasks
1. Improve module/import diagnostics.
2. Improve type error help text and fix suggestions.
3. Standardize error messages for parser and semantic failures.
4. Add snapshot tests for important diagnostics.

### Deliverables
- Error snapshots for at least 10 key cases.
- Strong location and help output for multi-file failures.

### Definition of Done
- Errors for common mistakes are clear enough that a new contributor can self-correct quickly.

## Priority 4: Incremental Compilation
### Goal
Make rebuilds fast enough for a real developer loop.

### Tasks
1. Expand cache logic from single-file checks to project graph invalidation.
2. Track dependencies between files.
3. Rebuild only changed modules and affected dependents.
4. Expose a stable cache/check API for CLI use.

### Deliverables
- Project-level invalidation logic.
- Tests for:
  - unchanged project
  - changed leaf module
  - changed shared dependency

### Definition of Done
- A leaf-file change does not trigger a full rebuild.

## Priority 5: Spec Alignment
### Goal
Keep spec and implementation synchronized.

### Tasks
1. Update `spec/language.md` for all syntax/type changes you make.
2. Add missing spec notes where implementation is now ahead of docs.
3. Keep design and type semantics aligned with actual compiler behavior.

### Definition of Done
- No known mismatch between implemented language syntax and the written spec in your owned areas.

## Handoff APIs For Engineer 2
Provide these as stable interfaces as early as possible:
- `load_project(root) -> Project`
- `analyze_project(project) -> diagnostics`
- `build_hir_for_project(project) -> HIR`
- `check_incremental(project) -> changed/clean`

## Coordination Rules
- Open small API-first PRs before large refactors.
- Do not edit backend code to force compatibility.
- If backend work exposes a missing compiler contract, add the contract in core instead of patching around it elsewhere.

## Weekly Suggested Order
### Week 1
- Project model
- file discovery
- import resolution

### Week 2
- multi-file semantic analysis
- project tests
- first stable APIs for CLI

### Week 3
- real union types
- type tests

### Week 4
- diagnostics upgrade
- error snapshots

### Week 5
- incremental compilation
- dependency invalidation tests

### Week 6
- spec cleanup
- polish and warning reduction in owned modules

## Success Criteria
You are done when Aura core can compile and analyze a real multi-file project, with reliable types, better errors, and stable APIs the rest of the toolchain can depend on.
