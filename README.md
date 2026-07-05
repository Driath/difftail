# chrono-diff

**A real-time review feed for a git repo.** Watch the working tree and, on every change,
print an inline block — timestamp, file, ±line counts, the *language-aware enclosing
definition* the change lands in, and the colored diff. Built for reviewing what coding
agents (Claude Code, Codex, aider, …) do to your codebase *as it happens*.

Not a full-screen TUI: `chrono-diff` prints to the normal terminal buffer, so the
**terminal's native scrollback is your history** — oldest change at the top, newest at
the bottom.

```
━━ 20:14:23  src/camera.ts  (−1 +1) · codex·11465 ━━
   ↳ class Camera > setZoom()
   private fov = 55;
   setZoom(z: number): void {
-    this.fov = 55 * z;
+    this.fov = 44 * z;
     this.clamp();
   }
```

## Why

Every mature terminal diff viewer (diffpane, hunk, lumen, …) is a full-screen TUI showing
a *snapshot* of the current working tree, grouped by file. None of them give a
**chronological, plain-printed stream** of changes over time — which is exactly what you
want when several agents are editing and you want to *review the sequence of what
happened*. `chrono-diff` is that stream.

## Install

```sh
# From source (Rust toolchain required)
cargo install --path .

# Or, once published, via Homebrew (see Formula/chrono-diff.rb)
brew install Driath/tap/chrono-diff
```

## Usage

```sh
chrono-diff                     # watch the current repo
chrono-diff /path/to/repo       # watch another repo
chrono-diff --exclude '*.lock'  # hide noise
chrono-diff --include 'packages/api/**'   # focus a subtree
chrono-diff --debounce 150      # snappier (default 300ms)
```

`Ctrl-C` to stop. The pre-existing diff is seeded as a **silent baseline** at launch, so
only edits made *after* launch stream in.

| Flag | Meaning |
|------|---------|
| `--include <glob>` | Only show files matching (repeatable). |
| `--exclude <glob>` | Hide files matching (repeatable). |
| `--debounce <ms>` | Coalesce bursts of writes (default 300). |

## What each block shows

- **Timestamp** — when the change was observed.
- **File** — repo-relative path.
- **±counts** — lines added / removed.
- **Attribution** — `· name·pid` when exactly one coding agent has its cwd in the repo
  (best-effort; see *Limitations*).
- **Scope** (`↳`) — the enclosing definition each hunk edits, resolved by **tree-sitter**
  for Rust, TypeScript/TSX, JavaScript, Python and Go (e.g. `class Camera > setZoom()`,
  `impl Cam > fn zoom`). Other languages fall back to git's `-U0` hunk scope.
- **Diff** — the colored changed lines, with git plumbing stripped.

## How it works

1. **Watch** — `notify` (FSEvents on macOS) pushes filesystem events; a debounce window
   coalesces write bursts.
2. **Detect** — on each tick, `git ls-files -mo` lists changed + new files; a per-file
   content hash suppresses re-printing an unchanged diff.
3. **Scope** — a zero-context (`-U0`) diff anchors each hunk on the changed line, then
   tree-sitter walks the AST to build the enclosing-definition breadcrumb.
4. **Render** — an inline block per changed file, printed to stdout (native scroll).

## Limitations

- **Attribution is a heuristic.** The OS provides no writer PID with a filesystem event,
  and privileged tracing (fs_usage/DTrace) is too heavy to require. `chrono-diff` detects
  *running agent processes whose cwd is inside the repo* and only names one when
  unambiguous. It's a proxy for "who's working here", not proof a given file was written
  by that process.
- **Scope fallback.** Languages without a tree-sitter grammar here use git's `-U0`
  heuristic, which is accurate for top-level definitions but not for methods nested inside
  classes.

## License

MIT
