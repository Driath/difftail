# Architecture

## Data flow

```
                    ┌─────────────────────────────────────────────┐
   filesystem  ───► │ notify-debouncer-mini   (main::main)         │
   write events     │   coalesce bursts (--debounce), drop .git    │
                    └───────────────────────┬─────────────────────┘
                                            │ one () per quiet tick
                                            ▼
                    ┌─────────────────────────────────────────────┐
                    │ scan()                              main.rs  │
                    │  git::changed_files  (ls-files -mo)          │
                    │  filter::allows      (--include/--exclude)   │
                    │  hash dedup          (skip unchanged file)   │
                    │  agents::detect/label(attribution, per tick) │
                    └───────────────┬─────────────────────────────┘
                                    │ per changed file
                    ┌───────────────▼─────────────────────────────┐
                    │  git::numstat            (±counts)           │
                    │  git::hunk_scopes  -U0   (line, git ctx)     │
                    │  scope::breadcrumb tree-sitter → nested def  │
                    │  git::diff_colored -U3   (display)           │
                    └───────────────┬─────────────────────────────┘
                                    ▼
                    ┌─────────────────────────────────────────────┐
                    │ render::change_block             render.rs   │
                    │  header: time · file · ±counts · agent       │
                    │  clean_diff: strip plumbing, ↳ scope / hunk  │
                    │  → plain stdout (native scrollback)          │
                    └─────────────────────────────────────────────┘
```

## Boundaries
- **`main.rs`** owns control flow (watch loop, seeding, per-file orchestration). No git or
  rendering logic lives here beyond wiring.
- **`git.rs`** is the only module that spawns `git`. Everything git-shaped (paths, diffs,
  counts, scopes) is a function here returning plain Rust values.
- **`scope.rs`** is the only tree-sitter consumer. Language support = extend `Lang` and
  `def_label`; nothing else changes.
- **`render.rs`** is the only place that writes to stdout and the only place that knows the
  block layout / ANSI.
- **`filter.rs`** / **`agents.rs`** are pure-ish helpers `scan` calls.

## Adding a language (scope)
1. Add the grammar crate to `Cargo.toml`.
2. Add a `Lang` variant + extension mapping in `scope::Lang::{from_path, language}`.
3. Add the definition node kinds and name fields in `scope::def_label`.
