# ADR-002 — Shell out to git, not a git library

**Decision.** `git.rs` invokes the `git` binary for every git operation.

**Why.** The CLI gives perfect colored diffs and language-aware xfuncname hunk scopes for
free, and keeps the dependency surface tiny. A git library (git2) would force us to
re-implement diff formatting and coloring for no gain here.

**Consequence.** `git` must be on PATH. Each changed file costs a few short `git` invocations
per tick — acceptable, and gated by the content-hash dedup so unchanged files aren't re-diffed.
