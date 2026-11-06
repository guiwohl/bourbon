# Dev loop and CPU budget

Date: 2026-11-06

Why: local tests, builds and mutants used over 90% of 16 cores and crashed other apps. A UI session needs to see real GPUI output fast.

| # | Topic | Decision | Why |
|---|---|---|---|
| 1 | UI preview | **The real app, rebuilt and restarted on save (`just dev`)**. No HTML mock | Only GPUI renders GPUI components exactly. Rebuild ≈ 5 s |
| 2 | Dev identity | **Dev build (`--features dev`) is `bourbon-dev` / "Bourbon Dev"** | Screenshots and window rules never hit the real app |
| 3 | Dev data | **Demo database in `target/dev-data`, seeded by `just dev-seed`** | Real notes are never touched; every screen has content |
| 4 | Mutants | **Run in CI on each PR, only on the PR diff (`--in-diff`). Required check `mutants`**. Locally optional, also diff-only | Heaviest local load; CI still blocks merges |
| 5 | Daily loop | **`just quick`**: fmt, comments, clippy and tests of changed crates only. `just check` before a PR; CI runs it too | Small runs while developing |
| 6 | CPU limits | **`nice -n 10` on cargo recipes; cargo `jobs = 12`; nextest 8 threads locally, all in CI** | Leaves cores and priority to the desktop |
| 7 | Build cache | **`sccache` and `mold` in the user's `~/.cargo/config.toml`** (not the repo: CI has neither) | Worktrees share compiled dependencies; faster links |
| 8 | Debug info | **No debug info for dependencies** (`[profile.dev.package."*"] debug = false`) | Smaller, faster links |
| 9 | Subagents | **At most 2 subagents compiling at once** | Avoid CPU spikes |
