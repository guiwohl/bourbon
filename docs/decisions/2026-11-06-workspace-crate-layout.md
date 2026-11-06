# 2026-11-06 — Workspace crate layout and build order

## Decisions

| # | Topic | Decision | Reason |
|---|---|---|---|
| 1 | Build order | **The whole app comes first; the CLI is built last** | The app is the product. The CLI must then be trivial to add on top of what already exists |
| 2 | Code layout | **A Cargo workspace with separate crates** (a crate is Rust's unit of compilation and dependency, like a package) | The compiler enforces the boundaries, instead of discipline |
| 3 | `bourbon-core` | **Domain (note, notebook, tag, trash), business rules and SQLite access. Never depends on GPUI** | Testable without opening a window; shared by every entry point |
| 4 | `bourbon-app` | **All GPUI UI. Depends on `bourbon-core`; holds no business rule** | The UI only presents and forwards user intent |
| 5 | `bourbon-cli` | **Built last. Depends only on `bourbon-core`** | `bourbon get <id>` becomes a thin entry point over already-tested code |

## Dependency rule

```
bourbon-app ──► bourbon-core ◄── bourbon-cli
```

Dependencies only point toward `bourbon-core`. Nothing in `bourbon-core` imports GPUI or anything from the other crates.
