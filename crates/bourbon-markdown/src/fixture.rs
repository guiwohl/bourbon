pub const NOTE: &str = "\
# App hangs when pasting a 5,000-row table

Pasting a large table into a note freezes the editor. See [[perf budget]] and [the budget](bourbon://note/abcdefghij12).

## Repro

1. Copy 5,000 rows from a spreadsheet.
2. Open a **new** note.
3. Paste with `Ctrl+V`.

## Environment

- Linux, Wayland
- *Release* build
- ~~debug~~ build

## What the profiler says

> Most of the time is spent in `reflow`.

```rust
fn reflow(rows: &[Row]) -> usize {
    rows.len()
}
```

| Rows | Time |
|:-----|-----:|
| 500 | 40 ms |
| 5,000 | 4 s |

- [ ] add a benchmark
- [x] reproduce the hang
";
