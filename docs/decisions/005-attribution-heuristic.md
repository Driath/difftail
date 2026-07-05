# ADR-005 — Attribution is a cwd+cmdline heuristic

**Decision.** `agents.rs` attributes a change to a coding agent by detecting running agent
processes whose **cwd is inside the repo**, and only names one when unambiguous: one in-repo
agent → `· name·pid`, several → `· N agents`, none → no tag.

**Context.** The OS delivers no writer PID with a filesystem event, and privileged tracing
(fs_usage/DTrace) is too heavy to require of every user.

**Why.** Running-agent-with-cwd-in-repo is a solid proxy for "who's working here" that needs
no privileges. Staying conservative (name only when unambiguous) means a wrong name never
misleads a review.

**Consequence.** It is a proxy, not proof of authorship. An agent that edits files outside its
cwd, or two agents in one repo, degrade gracefully to `N agents` / no tag. Sharper attribution
(worktree-based, fsmonitor, `--label`) is on the roadmap.
