//! Best-effort attribution: which coding agent is likely responsible for a change.
//!
//! The OS gives no writer PID with a filesystem event, and privileged tracing
//! (fs_usage/DTrace) is too heavy to require. Instead we detect *running agent
//! processes whose working directory is inside the repo* — a solid proxy for "who is
//! editing here". It's a heuristic (cwd is where an agent launched, not proof it wrote
//! a given file), so we only tag confidently: one in-repo agent → name it; several →
//! say how many; none → no tag (a manual/editor edit).

use std::path::Path;
use std::process::Command;

pub struct Agent {
    pub pid: u32,
    pub name: String,
}

/// Command-line tokens that identify a coding agent, matched as a whole path/argv
/// segment (so `/opt/bin/codex` and `node …/claude/cli.js` both hit).
const AGENT_TOKENS: &[&str] = &[
    "claude", "codex", "aider", "goose", "opencode", "cline", "cursor-agent", "amp",
    "gemini", "crush", "qwen", "cody",
];

/// Agent processes whose cwd is at or under `root`.
pub fn detect(root: &Path) -> Vec<Agent> {
    let out = match Command::new("ps").args(["-axo", "pid=,command="]).output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut agents = Vec::new();
    for line in text.lines() {
        let line = line.trim_start();
        let Some((pid_s, cmd)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let Ok(pid) = pid_s.parse::<u32>() else { continue };
        let Some(name) = agent_name(cmd) else { continue };
        if cwd_under(pid, root) {
            agents.push(Agent { pid, name });
        }
    }
    agents
}

/// A dim tag for the block header, or `None` when attribution isn't confident.
pub fn label(agents: &[Agent]) -> Option<String> {
    match agents.len() {
        0 => None,
        1 => Some(format!("{}·{}", agents[0].name, agents[0].pid)),
        n => Some(format!("{n} agents")),
    }
}

fn agent_name(cmd: &str) -> Option<String> {
    let seg_match = |token: &str| {
        cmd.split(|c: char| c == '/' || c.is_whitespace())
            .any(|seg| seg == token)
    };
    AGENT_TOKENS
        .iter()
        .find(|t| seg_match(t))
        .map(|t| t.trim_end_matches("-agent").to_string())
}

/// Whether process `pid`'s current working directory is at or under `root` (via lsof).
fn cwd_under(pid: u32, root: &Path) -> bool {
    let out = match Command::new("lsof")
        .args(["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return false,
    };
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some(path) = line.strip_prefix('n') {
            return Path::new(path).starts_with(root);
        }
    }
    false
}
