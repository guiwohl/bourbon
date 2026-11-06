# 2026-11-06 — Quality tooling

## Decisions

| # | Topic | Decision | Reason |
|---|---|---|---|
| 1 | Task runner | **`just`**: one entry point for build, run, test and every check | One command to remember; humans and agents run the same thing |
| 2 | Formatting | **`rustfmt`**, checked by `just fmt-check` | One style, no debate |
| 3 | Lint | **`clippy` pedantic, plus `todo`, `unimplemented`, `dbg_macro`; every warning is an error** | Idiomatic code; no placeholders reach `main` |
| 4 | Architecture guard | **`just architecture` fails if `bourbon-core`'s dependency tree contains any `gpui*` crate. `cargo-deny` also bans `gpui-kit`, `gpui-component`, `gpui-base` and `gpui-pre` outside their known parents** | `bourbon-core` can never import the UI framework. The tree check covers every GPUI sub-crate and cannot go stale on a GPUI bump |
| 5 | Comment ban | **`just comments` fails on any `//` or `/*` in `crates/**/*.rs`, except `://` (URLs); it also fails if ripgrep itself errors** | Enforces the zero-comments rule mechanically |
| 6 | Tests | **`cargo nextest`** | Fast, isolated test runs |
| 7 | Test quality | **`cargo-mutants`**, run separately with `just mutants` | Proves tests fail when the code is broken; too slow for every check |

## Known limits

- The comment check is a text search: a string literal containing `//` (other than `://`) is a false positive. Restructure the string.
- `cargo-deny` bans list exact crate names and parents; a GPUI upgrade can change them. `just architecture` is the guard that does not depend on that list.

## Enforcement

- Locally: manual, no git hook.
- On GitHub: the `check` workflow plus a `main` ruleset (PR required, `check` required, no force push, no deletion). See [ci.md](../development/ci.md).
