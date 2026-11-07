# justfile

Every build, run and quality task goes through `just`. Run `just` alone to list the recipes. Cargo recipes run under `nice -n 10`.

## Recipes

| Recipe | Runs | Use it to |
|---|---|---|
| `just` | `just --list` | See every recipe |
| `just build` | `cargo build --workspace` | Compile everything |
| `just run` | `cargo run -p bourbon-app` | Run the app |
| `just dev` | `watchexec` + `cargo run -p bourbon-app --features dev` | UI loop: rebuild and restart on every save in `crates/`, on the demo database. See [dev-loop.md](dev-loop.md) |
| `just dev-seed` | `seed_demo` example of `bourbon-core` | Recreate the demo database in `target/dev-data` |
| `just dev-shot` | `hyprctl` + `grim` | Screenshot the Bourbon Dev window to `target/dev/shot.png` |
| `just quick` | fmt-check, comments, clippy and nextest on the changed crates | **Daily loop.** Core changed → whole workspace; app, cli, markdown, text, editor and/or ui → those packages (ui also runs `bourbon-app`); nothing → skips |
| `just test` | `cargo nextest run --workspace` | Run all tests; fails if none exist |
| `just fmt` | `cargo fmt --all` | Format the code |
| `just fmt-check` | `cargo fmt --all -- --check` | Fail if anything is unformatted |
| `just lint` | `cargo clippy --workspace --all-targets -- -D warnings` | Fail on any clippy warning (pedantic, `todo!`, `unimplemented!`, `dbg!`) |
| `just deny` | `cargo deny check bans` | Fail if a crate other than `bourbon-app` depends on GPUI |
| `just architecture` | `cargo tree -p <crate> -e normal --prefix none` + ripgrep, for `bourbon-core`, `bourbon-cli`, `bourbon-markdown` and `bourbon-text`; the same for `bourbon-ui` against `gpui-kit` and `gpui-component` | Fail if any crate depends, even transitively, on any `gpui*` crate, if `bourbon-ui` depends on `gpui-kit` or `gpui-component`, or if cargo or ripgrep errors |
| `just comments` | ripgrep over `crates/**/*.rs` | Fail if any comment exists in code, or if ripgrep errors |
| `just check` | `fmt-check`, `lint`, `deny`, `architecture`, `comments`, `test` | **Run before a PR.** Must pass; CI runs it too |
| `just mutants` | `cargo mutants --in-diff` against `origin/main`, `TMPDIR` in `~/.cache/bourbon-mutants` | Optional locally; CI runs it on every PR (required check `mutants`). Must report 0 missed; the only excluded mutants are listed in `.cargo/mutants.toml` (`main`, `run`, the sidebar indentation and the pin glyph, which headless tests cannot observe) |

## Required tools

Rust comes from `rustup` (with `clippy` and `rustfmt`); never install the pacman `rust` package. Everything else:

```
omarchy pkg add just cargo-nextest cargo-deny ripgrep watchexec mold sccache jq
cargo install --locked cargo-mutants
```

The GPUI build needs these Arch packages: `fontconfig`, `freetype2`, `wayland`, `libxcb`, `libxkbcommon-x11`, plus `clang` and `cmake`. CI installs the Ubuntu equivalents, see [ci.md](ci.md).

## Local machine setup

`~/.cargo/config.toml` (user level, not in the repo because CI has neither tool):

```toml
[build]
rustc-wrapper = "sccache"
jobs = 12

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

`.config/nextest.toml` caps local tests at 8 threads; CI uses the `ci` profile (all cores).

## Install

- `just install`: release-builds and installs `bourbon-app` and `bourbon` into `~/.local/bin`, copies `packaging/bourbon.desktop` to `~/.local/share/applications/` and the icons to `~/.local/share/icons/hicolor/`, then refreshes the icon cache and desktop database when those tools exist.
- `just uninstall`: removes all of the above.
