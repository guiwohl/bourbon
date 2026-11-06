# 2026-11-06 — Quality tooling

## Decisions

| # | Topic | Decision | Reason |
|---|---|---|---|
| 1 | Task runner | **`just`**: one entry point for build, run, test and every check | One command to remember; humans and agents run the same thing |
| 2 | Formatting | **`rustfmt`**, checked by `just fmt-check` | One style, no debate |
| 3 | Lint | **`clippy` pedantic, plus `todo`, `unimplemented`, `dbg_macro`; every warning is an error** | Idiomatic code; no placeholders reach `main` |
| 4 | Architecture guard | **`cargo-deny` bans: `gpui`, `gpui-pre` and `gpui-component` may only be depended on by `bourbon-app`** | `bourbon-core` can never import the UI framework |
| 5 | Comment ban | **`just comments` fails on any `//` or `/*` in `crates/**/*.rs`, except `://` (URLs); it also fails if ripgrep itself errors** | Enforces the zero-comments rule mechanically |
| 6 | Tests | **`cargo nextest`** | Fast, isolated test runs |
| 7 | Test quality | **`cargo-mutants`**, run separately with `just mutants` | Proves tests fail when the code is broken; too slow for every check |

## Known limits

- The comment check is a text search: a string literal containing `//` (other than `://`) is a false positive. Restructure the string.
- `just test` passes with a warning while the workspace has no tests yet (`--no-tests=warn`).
- `just deny` prints `unused-wrapper` warnings until `bourbon-app` actually depends on a GPUI crate. Expected.
- GPUI pulls sub-crates (e.g. `gpui_*` platform crates) that the three banned names may not cover. Re-check the real dependency tree when GPUI is added.
- The ban was proven on 2026-11-06 with a stand-in crate: a dependency from `bourbon-core` fails with `error[banned]`; the same dependency from `bourbon-app` passes.

## Enforcement

- Locally: manual, no git hook.
- On GitHub: the `check` workflow plus a `main` ruleset (PR required, `check` required, no force push, no deletion). See [ci.md](../development/ci.md).
