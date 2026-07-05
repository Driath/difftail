//! Inline rendering of one change block. Prints to the normal terminal buffer
//! (NOT an alt-screen TUI) so the terminal's native scrollback is the history.

use chrono::Local;
use std::io::Write;

const CYAN_BOLD: &str = "\x1b[1;36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// One change: `━━ HH:MM:SS  <file>  (−rm +add) ━━`, then the diff with the accurate
/// enclosing scope printed inline before each hunk. `hunks` are (line, scope) pairs
/// from a `-U0` diff (see `git::hunk_scopes`) — precise per-change definitions.
pub fn change_block(
    path: &str,
    added: u32,
    removed: u32,
    colored_diff: &str,
    hunks: &[(u32, String)],
) {
    let ts = Local::now().format("%H:%M:%S");
    let mut out = String::new();

    out.push('\n');
    out.push_str(&format!(
        "{CYAN_BOLD}━━ {ts}  {path}  {DIM}(−{removed} +{added}){RESET}{CYAN_BOLD} ━━{RESET}\n"
    ));
    out.push_str(&clean_diff(colored_diff, hunks));

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
/// mode/rename lines) — the file is already in our block header. Before each displayed
/// hunk, prints a dim `↳ <definition>` scope line resolved from the precise `-U0`
/// `hunks` that fall inside the displayed hunk's line range. No line numbers (noise);
/// top-level hunks with no enclosing definition print no scope line; a scope identical
/// to the previously printed one is not repeated.
fn clean_diff(colored: &str, hunks: &[(u32, String)]) -> String {
    let mut out = String::new();
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
            let (start, count) = new_range(&plain);
            let end = start + count.max(1);

            // Accurate scopes: the -U0 hunks whose changed line lands in this hunk.
            let mut printed_any = false;
            for (line_no, scope) in hunks {
                if *line_no >= start && *line_no < end && !scope.is_empty() {
                    if last_scope.as_deref() != Some(scope.as_str()) {
                        out.push_str(&format!("{DIM}   ↳ {scope}{RESET}\n"));
                        last_scope = Some(scope.clone());
                    }
                    printed_any = true;
                }
            }
            // Fallback to the displayed hunk's own context if -U0 gave nothing here.
            if !printed_any {
                let ctx = hunk_scope(&plain);
                if !ctx.is_empty() {
                    if last_scope.as_deref() != Some(ctx.as_str()) {
                        out.push_str(&format!("{DIM}   ↳ {ctx}{RESET}\n"));
                        last_scope = Some(ctx);
                    }
                } else {
                    last_scope = None;
                }
            }
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

/// New-file `(start, count)` from a `@@ -a,b +c,d @@` header (count defaults to 1).
fn new_range(hunk: &str) -> (u32, u32) {
    if let Some(plus) = hunk.split_whitespace().find(|w| w.starts_with('+')) {
        let mut it = plus[1..].split(',');
        let start = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let count = it.next().and_then(|s| s.parse().ok()).unwrap_or(1);
        return (start, count);
    }
    (0, 1)
}
