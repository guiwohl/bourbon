# justfile

Every build, run and quality task goes through `just`. Run `just` alone to list the recipes.

## Recipes

| Recipe | Runs | Use it to |
|---|---|---|
| `just` | `just --list` | See every recipe |
| `just build` | `cargo build --workspace` | Compile everything |
| `just run` | `cargo run -p bourbon-app` | Run the app |
| `just test` | `cargo nextest run --workspace --no-tests=warn` | Run all tests; warns, not fails, while none exist |
| `just fmt` | `cargo fmt --all` | Format the code |
| `just fmt-check` | `cargo fmt --all -- --check` | Fail if anything is unformatted |
| `just lint` | `cargo clippy --workspace --all-targets -- -D warnings` | Fail on any clippy warning (pedantic, `todo!`, `unimplemented!`, `dbg!`) |
| `just deny` | `cargo deny check bans` | Fail if a crate other than `bourbon-app` depends on GPUI |
| `just comments` | ripgrep over `crates/**/*.rs` | Fail if any comment exists in code, or if ripgrep errors |
| `just check` | `fmt-check`, `lint`, `deny`, `comments`, `test` | **Run before every commit.** Must pass |
| `just mutants` | `cargo mutants --workspace` | Prove tests catch broken code. Slow; run before a PR touching logic |

## Required tools

Rust comes from `rustup` (with `clippy` and `rustfmt`); never install the pacman `rust` package. Everything else:

```
omarchy pkg add just cargo-nextest cargo-deny ripgrep
cargo install --locked cargo-mutants
```

GPUI system libraries get listed here when GPUI is added.
