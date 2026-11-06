default:
    just --list

build:
    cargo build --workspace

run:
    cargo run -p bourbon-app

test:
    cargo nextest run --workspace

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

deny:
    cargo deny check bans

comments:
    #!/usr/bin/env bash
    rg '(^|[^:])(//|/\*)' crates/ --type rust
    [ $? -eq 1 ]

architecture:
    #!/usr/bin/env bash
    dependencies=$(cargo tree -p bourbon-core -e normal --prefix none) || exit 1
    rg '^gpui' <<< "$dependencies"
    [ $? -eq 1 ]

mutants:
    mkdir -p "${XDG_CACHE_HOME:-$HOME/.cache}/bourbon-mutants"
    TMPDIR="${XDG_CACHE_HOME:-$HOME/.cache}/bourbon-mutants" cargo mutants --workspace

check: fmt-check lint deny architecture comments test
