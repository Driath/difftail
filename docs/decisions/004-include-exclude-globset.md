# ADR-004 — --include / --exclude via globset

**Decision.** Path filtering (`filter.rs`) uses the `globset` crate, matching each path both
as-is and by basename.

**Why.** Basename matching means `*.lock` catches `crates/x/Cargo.lock` without the user
writing `**/`. `globset` handles `**` and brace expansion correctly — not worth hand-rolling.

**Consequence.** `--include` (repeatable) restricts the feed to matches; `--exclude`
(repeatable) drops matches; exclude wins over include. An invalid glob exits with a clear
error at startup.
