# ADR-003 — Scope: -U0 anchoring + tree-sitter breadcrumb

**Decision.** Resolve the enclosing definition of each hunk in two parts:
1. Run a zero-context (`-U0`) diff so each hunk header anchors on the changed line.
2. Parse the current file with **tree-sitter** and walk the AST to a nested breadcrumb
   (`class Camera > setZoom()`, `impl Cam > fn zoom`).

**Context.** At the default context, git anchors the hunk header on the hunk's *first context
line*, which drifts the reported definition up to an outer scope — reporting the struct instead
of the method inside it. And git's line-based heuristic can't express nesting at all.

**Why.** `-U0` fixes the anchoring cheaply; tree-sitter provides the precise nested
breadcrumb git can't. Supported languages: Rust, TypeScript/TSX, JavaScript, Python, Go.

**Consequence.** Unsupported languages fall back to git's `-U0` hunk context — accurate for
top-level definitions, not for nested methods. Adding a language = a `Lang` variant +
`def_label` arm (see `architecture.md`).
