# CI and branch rules

## Workflow

`.github/workflows/check.yml` has two jobs:

| Job | When | Runs |
|---|---|---|
| `check` | every PR and push to `main` | `just check` with `NEXTEST_PROFILE=ci` |
| `mutants shard 0..3` | every PR | `cargo mutants --in-diff --shard k/4 --baseline skip` on the PR diff, 4 runners in parallel (the `check` job already proves the tests pass) |
| `mutants` | every PR | Passes only if every shard passed. This is the required check |

Before `just check`, one apt step runs `apt-get update` and installs `ripgrep`, `build-essential`, `clang`, `cmake`, `libfontconfig-dev`, `libwayland-dev`, `libx11-xcb-dev`, `libxkbcommon-x11-dev`. The list is taken from the apt branch of Zed's `script/linux` (keeping only what the lockfile needs: `yeslogic-fontconfig-sys`, `freetype-sys` via cmake, `wayland-sys`, `xcb`, `xkbcommon`). It could not be run locally; the first CI run is its proof. Tests are headless and need no display or GPU.

## Ruleset on `main`

| Rule | Effect |
|---|---|
| Pull request required | Nothing reaches `main` without a PR. No approvals needed (solo project) |
| Required status checks `check` and `mutants` | The PR can only merge with both green |
| Branch up to date | The PR branch must contain the latest `main` before merging; rebase first |
| Block force push | History of `main` cannot be rewritten |
| Block deletion | `main` cannot be deleted |

## Flow

Worktree branch → `just quick` while working → `just check` before the PR → push → open PR → `check` and `mutants` green → merge.
