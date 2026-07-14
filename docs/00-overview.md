# Overview

`orderselect` is a Rust library (and CLI) for **ordered multi-select** — an
interactive prompt that lets the user select a subset of options and records
the order in which they were chosen.

## The gap

No existing Rust prompt library combines **subset selection** with **ordering**:

| Library | Select subset? | Preserve order? |
| --- | --- | --- |
| `inquire::MultiSelect` | Yes | No (original list order) |
| `dialoguer::MultiSelect` | Yes | No (original list order) |
| `dialoguer::Sort` | No (all items) | Yes (reorder all) |
| `requestty::OrderSelect` | No (all items) | Yes (pick-up-and-place) |
| `inquire-reorder` | No (all items) | Yes (fork of inquire) |
| **`orderselect`** | **Yes** | **Yes (toggle order)** |

## Why not fork inquire?

`inquire-reorder` adds a `Reorder` prompt by forking `inquire` — users must
replace their entire `inquire` dependency. It has zero dependents.

`orderselect` is a **companion**, not a replacement:

```toml
[dependencies]
inquire = "0.7"        # for Select, Confirm, Text, etc.
orderselect = "0.1"    # for OrderedSelect
```

Both use `crossterm` under the hood, so they coexist in the same binary without
terminal conflicts.

## What it looks like

```
  Select build targets

 ❯ 1. ◆ x86_64-unknown-linux-gnu
   2. ◇ aarch64-unknown-linux-gnu
   3. ◆ aarch64-apple-darwin
   4. ◇ x86_64-pc-windows-msvc

  ↑↓ move · space toggle · a all · n none · enter confirm · esc cancel
```

The number next to each item is its **selection order** — `1` was toggled first,
`2` second. Toggling again removes the item and renumbers subsequent selections.

## Use cases

- **Build target selection** — "I want linux-gnu first, then darwin, then
  windows" (the original use case, from [lhg](https://github.com/.../lhg)).
- **Priority ordering** — "rank these features by importance" (subset + order).
- **Pipeline configuration** — "which steps to run, and in what order?"
- **Task queues** — "pick tasks for today, ordered by priority."

## License

MIT.
