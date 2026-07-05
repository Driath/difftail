# Roadmap

## Shipped (v0.1)
- Chronological inline review feed, native scrollback, silent baseline seeding.
- Git plumbing stripped; per-hunk scope line.
- Precise scope: `-U0` anchoring + tree-sitter breadcrumb (Rust, TS/TSX, JS, Python, Go).
- `--include` / `--exclude` path globs.
- Best-effort agent attribution tag (`· name·pid`).
- README, MIT license, Homebrew formula template + releasing guide.

## Next
- **More tree-sitter grammars** — C/C++, Java, Ruby, PHP, C#, Swift, Kotlin, Zic. Each is a
  `Lang` variant + `def_label` arm.
- **Attribution, sharper** — optional `fsmonitor`/`lsof`-open correlation; per-worktree
  attribution when agents run in separate git worktrees; a `--label` override for manual
  tagging of a feed.
- **Config** — a `--stat`/compact mode; `--since <time>`; colour theme; a config file for
  default excludes (lockfiles, build dirs).
- **Perf** — reuse notify's changed-path set to avoid re-diffing every dirty file each tick;
  parse each file once per tick and resolve all its hunks from one tree.
- **Packaging** — publish the GitHub repo + tap, prebuilt Homebrew bottles, a demo GIF.

## Known limitations (see decisions.md)
- Attribution is a cwd+cmdline heuristic, not proof of authorship.
- Non-tree-sitter languages fall back to git's `-U0` scope (top-level accurate, not nested).
