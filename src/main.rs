//! chrono-diff — a real-time review feed for a git repo.
//!
//! Watches the working tree and, on every change, prints an inline block per file:
//! timestamp, path, ±line counts, the language-aware enclosing scope(s) the change
//! lands in, and the colored diff. Pre-existing changes are seeded as a silent
//! baseline at launch, so only edits made *after* launch stream in — chronologically,
//! oldest at the top, newest at the bottom, in the terminal's native scrollback.

mod git;
mod render;
mod scope;

use clap::Parser;
use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::DebounceEventResult;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "chrono-diff", version, about)]
struct Args {
    /// Repo (or subdir) to watch. Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Debounce window in milliseconds (coalesce bursts of writes).
    #[arg(long, default_value_t = 300)]
    debounce: u64,
}

/// Diff every changed file; print a block only for files whose diff changed since
/// last scan. `seeding` records the baseline silently (no output).
fn scan(root: &std::path::Path, state: &mut HashMap<String, u64>, seeding: bool) {
    for f in git::changed_files(root) {
        let plain = git::diff_plain(root, &f);
        if plain.is_empty() {
            // File went back to clean — forget it so a later re-edit prints again.
            state.remove(&f);
            continue;
        }
        let h = git::hash_str(&plain);
        if state.get(&f) == Some(&h) {
            continue; // unchanged since last scan
        }
        state.insert(f.clone(), h);
        if seeding {
            continue;
        }
        let (added, removed) = git::numstat(root, &f);
        // Precise per-hunk scope: tree-sitter breadcrumb from the current file source,
        // falling back to git's -U0 hunk context for unsupported languages.
        let hunks = git::hunk_scopes(root, &f);
        let source = std::fs::read_to_string(root.join(&f)).unwrap_or_default();
        let resolved: Vec<(u32, String)> = hunks
            .iter()
            .map(|(line, ctx)| {
                let label = scope::breadcrumb(std::path::Path::new(&f), &source, *line)
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| ctx.clone());
                (*line, label)
            })
            .collect();
        let colored = git::diff_colored(root, &f);
        render::change_block(&f, added, removed, &colored, &resolved);
    }
}

fn main() {
    let args = Args::parse();

    let root = match git::repo_root(&args.path) {
        Some(r) => r,
        None => {
            eprintln!("chrono-diff: not a git repository: {}", args.path.display());
            std::process::exit(1);
        }
    };

    // Seed the baseline: everything already dirty is recorded but NOT printed.
    let mut state: HashMap<String, u64> = HashMap::new();
    scan(&root, &mut state, true);

    // Filesystem watch → debounce → notify the main thread to re-scan.
    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(args.debounce), move |res: DebounceEventResult| {
        if let Ok(events) = res {
            // Ignore pure .git churn (index/lock writes) — only user/agent edits matter.
            let relevant = events
                .iter()
                .any(|e| !e.path.components().any(|c| c.as_os_str() == ".git"));
            if relevant {
                let _ = tx.send(());
            }
        }
    })
    .expect("chrono-diff: failed to start file watcher");

    debouncer
        .watcher()
        .watch(&root, RecursiveMode::Recursive)
        .expect("chrono-diff: failed to watch repo");

    render::banner(&root);

    for _ in rx {
        scan(&root, &mut state, false);
    }
}
