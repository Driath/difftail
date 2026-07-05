//! Inline rendering of one change block. Prints to the normal terminal buffer
//! (NOT an alt-screen TUI) so the terminal's native scrollback is the history.

use chrono::Local;
use std::io::Write;

const CYAN_BOLD: &str = "\x1b[1;36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// One change: `━━ HH:MM:SS  <file>  (−rm +add) ━━`, then the diff with the enclosing
/// scope printed inline before each hunk that lands in a new definition.
pub fn change_block(path: &str, added: u32, removed: u32, colored_diff: &str) {
    let ts = Local::now().format("%H:%M:%S");
    let mut out = String::new();

    out.push('\n');
    out.push_str(&format!(
        "{CYAN_BOLD}━━ {ts}  {path}  {DIM}(−{removed} +{added}){RESET}{CYAN_BOLD} ━━{RESET}\n"
    ));
    out.push_str(&clean_diff(colored_diff));

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let _ = lock.write_all(out.as_bytes());
    let _ = lock.flush();
}

/// The launch banner (dim, so it doesn't compete with the change feed).
pub fn banner(root: &std::path::Path) {
    println!("{DIM}▶ chrono-diff: {} — review feed, Ctrl-C to stop{RESET}", root.display());
}

/// Strip the git plumbing from a colored diff, keeping only what a reviewer wants.
/// Drops the whole extended header (`diff --git`, `index`, `--- a/…`, `+++ b/…`,
/// mode/rename lines) — the file is already in our block header. Replaces each
/// `@@ -a,b +c,d @@ ctx` hunk header with a dim `↳ ctx` scope line (the language-aware
/// enclosing definition), so every hunk shows *which* definition it edits. No line
/// numbers (noise). Consecutive hunks in the same definition print the scope once.
fn clean_diff(colored: &str) -> String {
    let mut out = String::new();
    // Everything between `diff --git` and the first `@@` of a file is header plumbing.
    let mut in_header = false;
    let mut last_scope: Option<String> = None;

    for line in colored.lines() {
        let plain = strip_ansi(line);
        if plain.starts_with("diff --git ") {
            in_header = true;
            continue;
        }
        if plain.starts_with("@@") {
            in_header = false;
            let ctx = hunk_scope(&plain);
            // Only surface a scope line when there's a real enclosing definition;
            // a top-level hunk (empty ctx) prints nothing. Dedupe consecutive hunks.
            if !ctx.is_empty() && last_scope.as_deref() != Some(ctx.as_str()) {
                out.push_str(&format!("{DIM}   ↳ {ctx}{RESET}\n"));
            }
            last_scope = (!ctx.is_empty()).then_some(ctx);
            continue;
        }
        if in_header {
            continue; // index / --- / +++ / mode / rename / binary-notice
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Remove ANSI CSI escape sequences (`\x1b[…m`) from a line. UTF-8 safe: we iterate
/// by char so multibyte code content is never split. Used only to inspect prefixes.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Consume up to and including the sequence's final letter (e.g. the `m`).
            for n in chars.by_ref() {
                if n.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The language-aware enclosing definition git prints after the second `@@`.
fn hunk_scope(hunk: &str) -> String {
    hunk.splitn(3, "@@").nth(2).map(|s| s.trim().to_string()).unwrap_or_default()
}
