# difftail — verified source of truth

> Canonical reference. Every claim traced to a symbol in `src/`. When other docs disagree
> with the code, **the code wins** — grep the symbol.

## What it is
A CLI that watches a git work tree and prints a **chronological, inline review feed**: one
block per changed file, streamed to the normal terminal buffer (native scrollback = history).
Not a full-screen/alt-screen TUI. Built to review what coding agents do to a repo live.

## The pipeline (per change)
1. **Watch** — `notify` via `notify-debouncer-mini` pushes filesystem events; a debounce
   window (`--debounce`, default 300ms) coalesces write bursts (`main::main`). Events whose
   paths are all under `.git` are ignored.
2. **Detect** — on each tick `scan` calls `git::changed_files` (`git ls-files -mo
   --exclude-standard` → modified + untracked, gitignore-honoring). A per-file content hash
   (`git::hash_str` over the plain diff) is kept in `state`; a file whose hash is unchanged
   since last tick is skipped (`main::scan`).
3. **Filter** — `filter::Filter::allows` drops files by `--exclude` / keeps only `--include`
   globs (matched against full path and basename).
4. **Attribute** — `agents::detect` finds coding-agent processes whose cwd is under the repo;
   `agents::label` yields `name·pid` (one), `N agents` (several), or nothing.
5. **Scope** — `git::hunk_scopes` runs a **`-U0`** diff so each hunk header anchors on the
   changed line; `scope::breadcrumb` parses the current file with tree-sitter and walks the
   AST to build the enclosing-definition breadcrumb. Fallback: the `-U0` git hunk context.
6. **Render** — `render::change_block` prints the inline block; `render::clean_diff` strips
   git plumbing and emits a `↳ scope` line per hunk.

## Module map (`src/`)
| File | Responsibility |
|------|----------------|
| `main.rs` | CLI (clap), watch loop, `scan` orchestration, baseline seeding |
| `git.rs` | git shell-outs: repo root, changed files, colored/plain diff, numstat, `-U0` hunk scopes, hash |
| `scope.rs` | tree-sitter breadcrumb (Rust/TS/TSX/JS/Python/Go); `def_label` per language |
| `render.rs` | inline block render, `clean_diff` (strip plumbing, per-hunk scope), ANSI strip |
| `filter.rs` | `--include`/`--exclude` globset filtering |
| `agents.rs` | best-effort agent attribution (cwd+cmdline heuristic) |

## Verified behaviors
- **Baseline is silent.** `scan(.., seeding=true)` at launch records hashes without printing;
  only post-launch edits stream (`main::main`).
- **Two noise filters.** Debounce (batches raw events) + content-hash dedup (never reprints an
  unchanged diff). Even at `--debounce 0` no duplicate blocks — the hash catches them.
- **Scope accuracy needs `-U0`.** At default context git anchors the hunk header on the hunk's
  first *context* line, drifting scope to an outer definition; `-U0` anchors on the change.
- **Native scroll.** All output is plain stdout writes (`render`); no alt-screen, no cursor
  control — the terminal scrollback is the review history.

## Non-negotiables
- **Inline, never alt-screen** — native scrollback is the product.
- **git is the diff source** — shell-out to `git` (perfect color + xfuncname), not a git lib.
- **Attribution stays conservative** — only name an agent when unambiguous
  (see `decisions/005-attribution-heuristic.md`).
