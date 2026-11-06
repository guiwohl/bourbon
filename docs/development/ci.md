# CI and branch rules

## Workflow

`.github/workflows/check.yml` runs `just check` on every pull request and every push to `main`. Job name: `check`.

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
