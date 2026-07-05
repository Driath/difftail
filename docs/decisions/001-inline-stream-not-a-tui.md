# ADR-001 — Inline stream, not a full-screen TUI

**Decision.** Print each change to the normal terminal buffer. No alt-screen, no cursor
addressing.

**Context.** Every mature terminal diff viewer (diffpane, hunk, lumen) is an alt-screen TUI
showing a *snapshot* of the current tree, grouped by file. That paradigm has no time axis and
captures the scrollback.

**Why.** Printing inline makes the terminal's **native scrollback the chronological history** —
oldest change at the top, newest at the bottom. This is the core product decision and the
reason chrono-diff exists as its own tool.

**Consequence.** No live-updating UI; the feed only grows. Rendering is plain `write_all` in
`render.rs`.
