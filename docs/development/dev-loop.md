# UI dev loop

How to iterate on the UI and see the real GPUI result.

| Step | Command | What happens |
|---|---|---|
| Start | `just dev` | Seeds the demo database if missing, opens **Bourbon Dev**, and rebuilds and restarts it on every save of `.rs`/`.toml` in `crates/` (≈ 5 s) |
| Reset data | `just dev-seed` | Recreates `target/dev-data/bourbon/bourbon.db` from `crates/bourbon-core/examples/seed_demo.rs` |
| Screenshot | `just dev-shot` | Saves the Bourbon Dev window to `target/dev/shot.png` |

## Why it is safe

- `--features dev` gives the window app id `bourbon-dev` and title "Bourbon Dev" (`window.rs`), so `dev-shot` and Hyprland rules never match the real app.
- `XDG_DATA_HOME=target/dev-data` points the database there; real notes in `~/.local/share/bourbon` are never opened.

## Demo data

Notebooks Inbox, Work › Projects, Personal; tags idea, urgent, reference; every status; pinned notes; an untitled note; one note in Trash; a "Daily log {{date}}" template.
