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

    out.push_str(colored_diff);
    if !colored_diff.ends_with('\n') {
        out.push('\n');
    }

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let _ = lock.write_all(out.as_bytes());
    let _ = lock.flush();
}

/// The launch banner (dim, so it doesn't compete with the change feed).
pub fn banner(root: &std::path::Path) {
    println!("{DIM}▶ chrono-diff: {} — review feed, Ctrl-C to stop{RESET}", root.display());
}
