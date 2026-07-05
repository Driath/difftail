//! All git interaction: shell-outs to the `git` binary. No git library — the CLI
//! gives us perfect colored diffs and language-aware hunk scopes (xfuncname) for free.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolve the repository root for `path`, or `None` if it isn't a git work tree.
pub fn repo_root(path: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then(|| PathBuf::from(s))
}

/// Files git considers changed: modified (tracked) plus untracked, honoring .gitignore.
pub fn changed_files(root: &Path) -> Vec<String> {
    let out = match Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-mo", "--exclude-standard"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let mut seen = std::collections::HashSet::new();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .filter(|l| !l.is_empty() && seen.insert(l.clone()))
        .collect()
}

fn is_tracked(root: &Path, file: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--error-unmatch", "--", file])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run `git diff` for a single file. Untracked files diff against /dev/null so a
/// freshly-created file (e.g. one an agent just wrote) still renders as all-additions.
fn run_diff(root: &Path, file: &str, color: bool, extra: &[&str]) -> String {
    let color_flag = if color { "color.ui=always" } else { "color.ui=never" };
    let tracked = is_tracked(root, file);
    let mut c = Command::new("git");
    c.arg("-C").arg(root).args(["--no-pager", "-c", color_flag, "diff"]);
    c.args(extra);
    if tracked {
        c.args(["--", file]);
    } else {
        c.args(["--no-index", "--", "/dev/null", file]);
    }
    // --no-index returns exit 1 when files differ; we read stdout regardless.
    match c.output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => String::new(),
    }
}

pub fn diff_colored(root: &Path, file: &str) -> String {
    run_diff(root, file, true, &[])
}

pub fn diff_plain(root: &Path, file: &str) -> String {
    run_diff(root, file, false, &[])
}

/// (added, removed) line counts for a file.
pub fn numstat(root: &Path, file: &str) -> (u32, u32) {
    let out = run_diff(root, file, false, &["--numstat"]);
    for line in out.lines() {
        let mut it = line.split('\t');
        let a = it.next().unwrap_or("0");
        let r = it.next().unwrap_or("0");
        // "-" marks a binary file; treat as 0/0.
        let a = a.parse().unwrap_or(0);
        let r = r.parse().unwrap_or(0);
        return (a, r);
    }
    (0, 0)
}

pub fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
