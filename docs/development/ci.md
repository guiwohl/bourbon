# CI and branch rules

## Workflow

`.github/workflows/check.yml` runs `just check` on every pull request and every push to `main`. Job name: `check`.

Before `just check`, one apt step runs `apt-get update` and installs `ripgrep`, `build-essential`, `clang`, `cmake`, `libfontconfig-dev`, `libwayland-dev`, `libx11-xcb-dev`, `libxkbcommon-x11-dev`. The list is taken from the apt branch of Zed's `script/linux` (keeping only what the lockfile needs: `yeslogic-fontconfig-sys`, `freetype-sys` via cmake, `wayland-sys`, `xcb`, `xkbcommon`). It could not be run locally; the first CI run is its proof. Tests are headless and need no display or GPU.

## Ruleset on `main`

| Rule | Effect |
|---|---|
| Pull request required | Nothing reaches `main` without a PR. No approvals needed (solo project) |
| Required status check `check` | The PR can only merge with the workflow green |
| Branch up to date | The PR branch must contain the latest `main` before merging; rebase first |
| Block force push | History of `main` cannot be rewritten |
| Block deletion | `main` cannot be deleted |

## Flow

Worktree branch → `just check` locally → push branch → open PR → `check` green → merge.
