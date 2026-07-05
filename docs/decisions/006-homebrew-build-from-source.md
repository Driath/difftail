# ADR-006 — Homebrew via a build-from-source formula

**Decision.** The formula (`Formula/chrono-diff.rb`) runs `cargo install` against a release
tarball rather than shipping prebuilt bottles.

**Why.** Simpler to maintain for a single-author tap; no per-platform bottle build/CI.

**Consequence.** Users need the Rust toolchain (`depends_on "rust" => :build`) and pay a
compile at install time. Revisit with prebuilt bottles if build time becomes a pain point.
