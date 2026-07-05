//! Path filtering for the change feed: `--include` / `--exclude` globs. Lets a reviewer
//! focus a subtree (`--include 'packages/api/**'`) or silence noise (`--exclude '*.lock'`).

use globset::{Glob, GlobSet, GlobSetBuilder};

pub struct Filter {
    include: Option<GlobSet>,
    exclude: GlobSet,
}

impl Filter {
    /// Build from CLI patterns. Errors on an invalid glob (surfaced to the user).
    pub fn new(includes: &[String], excludes: &[String]) -> Result<Filter, String> {
        Ok(Filter {
            include: if includes.is_empty() { None } else { Some(build(includes)?) },
            exclude: build(excludes)?,
        })
    }

    /// Whether a repo-relative path should appear in the feed. A path is matched both
    /// as-is and by basename, so `*.lock` catches `crates/x/Cargo.lock` without `**/`.
    pub fn allows(&self, path: &str) -> bool {
        let base = path.rsplit('/').next().unwrap_or(path);
        if self.exclude.is_match(path) || self.exclude.is_match(base) {
            return false;
        }
        match &self.include {
            Some(set) => set.is_match(path) || set.is_match(base),
            None => true,
        }
    }
}

fn build(patterns: &[String]) -> Result<GlobSet, String> {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        b.add(Glob::new(p).map_err(|e| format!("invalid glob '{p}': {e}"))?);
    }
    b.build().map_err(|e| e.to_string())
}
