# Design decisions (ADR log)

Short records of the choices that shaped chrono-diff. Newest first.

## ADR-006 — Homebrew via build-from-source formula
The formula (`Formula/chrono-diff.rb`) runs `cargo install` against a release tarball rather
than shipping prebuilt bottles. Simpler to maintain for a single-author tap; users need the
Rust toolchain (`depends_on "rust" => :build`). Revisit with bottles if build time hurts.

## ADR-005 — Attribution is a cwd+cmdline heuristic
The OS delivers no writer PID with a filesystem event, and privileged tracing (fs_usage/
DTrace) is too heavy to require. `agents.rs` instead detects running agent processes whose
**cwd is inside the repo** and only names one when unambiguous (`name·pid`; else `N agents`;
else nothing). It's a proxy for "who's working here", not proof of authorship — kept
deliberately conservative so a wrong name never misleads a review.

## ADR-004 — `--include`/`--exclude` via globset
Path filtering uses `globset`, matching each path both as-is and by basename so `*.lock`
catches nested lockfiles without `**/`. Chosen over hand-rolled glob matching for correct
`**`/brace handling.

## ADR-003 — Scope: `-U0` anchoring + tree-sitter breadcrumb
Two-part scope resolution. (a) Run a zero-context (`-U0`) diff so each hunk header anchors on
the changed line — at default context git drifts the reported definition up to an outer scope
(struct instead of the method inside it). (b) Parse the file with **tree-sitter** and walk the
AST for a nested breadcrumb (`class Camera > setZoom()`) that git's line heuristic can't
produce. git's `-U0` context is the fallback for unsupported languages.

## ADR-002 — Shell out to `git`, not a git library
`git.rs` invokes the `git` binary. This gives perfect colored diffs and language-aware
xfuncname hunk scopes for free, and keeps the dependency surface tiny. A git library (git2)
would re-implement diff rendering and coloring for no gain here.

## ADR-001 — Inline stream, not a full-screen TUI
Every mature terminal diff viewer (diffpane, hunk, lumen) is an alt-screen TUI showing a
*snapshot* of the current tree. chrono-diff prints to the normal buffer so the terminal's
**native scrollback is the chronological history** — oldest change at top, newest at bottom.
This is the core product decision and the reason the tool exists.
