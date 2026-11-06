# bourbon-cli

Binary `bourbon`. Depends only on `bourbon-core`; `just architecture` fails if it pulls any `gpui*` crate.

## Modules

| Module | Role |
|---|---|
| `command` | Pure `parse(&[String]) -> Command` (`LaunchApp`, `Get(id)`, `Help`, `Version`, `Unknown`) |
| `cli` | `run(arguments, database_path, launch_app, stdout, stderr) -> u8`; usage text; exit codes |
| `launch` | Spawns `bourbon-app` (sibling of the executable, else `PATH`), stdio null, not awaited |
| `main` | Wires `std::env::args`, `default_database_path`, real streams |

## Behaviour

| Input | stdout | stderr | Exit |
|---|---|---|---|
| `get <id>` of a live note or template | The note Markdown, same as the export | empty | 0 |
| `get <id>` unknown or trashed | empty | `note not found: <id>` | 1 |
| `get <malformed id>` | empty | `invalid note id: <id>` | 2 |
| `get` cannot open the database | empty | `cannot open database: ...` | 1 |
| no arguments | empty | empty (app launched) | 0 |
| no arguments, app missing | empty | `cannot launch bourbon-app: ...` | 1 |
| `--help`, `-h` | usage | empty | 0 |
| `--version`, `-V` | `bourbon <version>` | empty | 0 |
| anything else | empty | usage | 2 |

## Tests

Unit tests per module with a temp on-disk database. `tests/binary.rs` runs the real binary with `XDG_DATA_HOME` pointing at a temp directory, and a copy of the binary without a neighbouring `bourbon-app` and with an empty `PATH`.

## Copy command (app side)

The editor header "..." menu and the control pane action "Copy CLI command" write `bourbon get <id>` to the clipboard using `cli_command` in `bourbon-app`.
