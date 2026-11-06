cargo := "nice -n 10 cargo"
dev_data := justfile_directory() / "target/dev-data"
dev_database := dev_data / "bourbon/bourbon.db"

default:
    just --list

build:
    {{cargo}} build --workspace

run:
    {{cargo}} run -p bourbon-app

dev:
    #!/usr/bin/env bash
    [ -f "{{dev_database}}" ] || just dev-seed
    export XDG_DATA_HOME="{{dev_data}}"
    exec watchexec --restart --watch crates --exts rs,toml -- {{cargo}} run -q -p bourbon-app --features dev

dev-seed:
    #!/usr/bin/env bash
    rm -f "{{dev_database}}"
    mkdir -p "$(dirname "{{dev_database}}")"
    {{cargo}} run -q -p bourbon-core --example seed_demo -- "{{dev_database}}"

dev-shot:
    #!/usr/bin/env bash
    geometry=$(hyprctl -j clients | jq -r '.[] | select(.class == "bourbon-dev") | "\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"' | head -1)
    [ -n "$geometry" ] || { echo "Bourbon Dev window not found"; exit 1; }
    mkdir -p target/dev
    grim -g "$geometry" target/dev/shot.png
    echo target/dev/shot.png

test:
    {{cargo}} nextest run --workspace

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    {{cargo}} clippy --workspace --all-targets -- -D warnings

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

quick:
    #!/usr/bin/env bash
    set -e
    changed=$( { git diff --name-only origin/main...HEAD; git diff --name-only HEAD; git ls-files --others --exclude-standard; } )
    if grep -q '^crates/bourbon-core/' <<< "$changed"; then
        packages="--workspace"
    elif grep -q '^crates/bourbon-app/' <<< "$changed"; then
        packages="-p bourbon-app"
    else
        echo "no crate changed"
        exit 0
    fi
    just fmt-check comments
    {{cargo}} clippy $packages --all-targets -- -D warnings
    {{cargo}} nextest run $packages

mutants:
    #!/usr/bin/env bash
    set -e
    mkdir -p "${XDG_CACHE_HOME:-$HOME/.cache}/bourbon-mutants"
    git diff origin/main > target/mutants.diff
    TMPDIR="${XDG_CACHE_HOME:-$HOME/.cache}/bourbon-mutants" {{cargo}} mutants --workspace --test-tool nextest --in-diff target/mutants.diff

check: fmt-check lint deny architecture comments test
