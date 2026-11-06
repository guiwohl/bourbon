# justfile

Every build, run and quality task goes through `just`. Run `just` alone to list the recipes.

## Recipes

| Recipe | Runs | Use it to |
|---|---|---|
| `just` | `just --list` | See every recipe |
| `just build` | `cargo build --workspace` | Compile everything |
| `just run` | `cargo run -p bourbon-app` | Run the app |
| `just test` | `cargo nextest run --workspace` | Run all tests; fails if none exist |
| `just fmt` | `cargo fmt --all` | Format the code |
| `just fmt-check` | `cargo fmt --all -- --check` | Fail if anything is unformatted |
| `just lint` | `cargo clippy --workspace --all-targets -- -D warnings` | Fail on any clippy warning (pedantic, `todo!`, `unimplemented!`, `dbg!`) |
| `just deny` | `cargo deny check bans` | Fail if a crate other than `bourbon-app` depends on GPUI |
| `just architecture` | `cargo tree -p bourbon-core -e normal --prefix none` + ripgrep | Fail if `bourbon-core` depends, even transitively, on any `gpui*` crate, or if cargo or ripgrep errors |
| `just comments` | ripgrep over `crates/**/*.rs` | Fail if any comment exists in code, or if ripgrep errors |
| `just check` | `fmt-check`, `lint`, `deny`, `architecture`, `comments`, `test` | **Run before every commit.** Must pass |
| `just mutants` | `cargo mutants --workspace`, with `TMPDIR` in `~/.cache/bourbon-mutants` | Prove tests catch broken code. Slow; run before a PR touching logic. Uses the disk cache because `/tmp` is a small tmpfs. Must report 0 missed; the only excluded mutants are listed in `.cargo/mutants.toml` (`main`, `run` and the sidebar indentation, which headless tests cannot observe) |

## Required tools

Rust comes from `rustup` (with `clippy` and `rustfmt`); never install the pacman `rust` package. Everything else:

```
omarchy pkg add just cargo-nextest cargo-deny ripgrep
cargo install --locked cargo-mutants
```

The GPUI build needs these Arch packages: `fontconfig`, `freetype2`, `wayland`, `libxcb`, `libxkbcommon-x11`, plus `clang` and `cmake`. CI installs the Ubuntu equivalents, see [ci.md](ci.md).
