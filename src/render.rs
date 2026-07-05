//! Inline rendering of one change block. Prints to the normal terminal buffer
//! (NOT an alt-screen TUI) so the terminal's native scrollback is the history.

use chrono::Local;
use std::io::Write;

const CYAN_BOLD: &str = "\x1b[1;36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// One change: `━━ HH:MM:SS  <file>  (−rm +add) ━━`, a scope breadcrumb, then the diff.
pub fn change_block(path: &str, added: u32, removed: u32, scopes: &[String], colored_diff: &str) {
    let ts = Local::now().format("%H:%M:%S");
    let mut out = String::new();

    out.push('\n');
    out.push_str(&format!(
        "{CYAN_BOLD}━━ {ts}  {path}  {DIM}(−{removed} +{added}){RESET}{CYAN_BOLD} ━━{RESET}\n"
    ));

    if scopes.is_empty() {
        out.push_str(&format!("{DIM}   ↳ (top level){RESET}\n"));
    } else {
        // A few enclosing definitions the change lands in — the review breadcrumb.
        let shown: Vec<&str> = scopes.iter().take(4).map(String::as_str).collect();
        let more = scopes.len().saturating_sub(shown.len());
        let mut line = format!("{DIM}   ↳ {}", shown.join("  ·  "));
        if more > 0 {
            line.push_str(&format!("  (+{more} more)"));
        }
        line.push_str(RESET);
        out.push_str(&line);
        out.push('\n');
    }

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

/// Strip the git plumbing from a colored diff, keeping only what a reviewer wants:
/// the changed lines. Drops the whole extended header (`diff --git`, `index`,
/// `--- a/…`, `+++ b/…`, mode/rename lines) — the file is already in our own header —
/// and collapses each `@@ -a,b +c,d @@ ctx` hunk header to a tidy dim `@ <line>` marker
/// (the scope already lives in the breadcrumb above).
fn clean_diff(colored: &str) -> String {
    let mut out = String::new();
    // Everything between `diff --git` and the first `@@` of a file is header plumbing.
    let mut in_header = false;

    for line in colored.lines() {
        let plain = strip_ansi(line);
        if plain.starts_with("diff --git ") {
            in_header = true;
            continue;
        }
        if plain.starts_with("@@") {
            in_header = false;
            if let Some(start) = hunk_new_start(&plain) {
                out.push_str(&format!("{DIM}   @ {start}{RESET}\n"));
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

/// From a `@@ -a,b +c,d @@ …` header, the new-file start line `c`.
fn hunk_new_start(hunk: &str) -> Option<u32> {
    let plus = hunk.split_whitespace().find(|w| w.starts_with('+'))?;
    plus[1..].split(',').next()?.parse().ok()
}
