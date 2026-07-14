# Design

## Interaction model: toggle-order

The core interaction is **toggle-order**: pressing `Space` adds an item to the
selection (or removes it if already selected). The **order of toggling** is
preserved — the first item toggled is the first item in the result.

```
 ❯ 1. ◆ first-selected-item
   2. ◆ second-selected-item
   3. ◇ unselected-item
   4. ◆ third-selected-item
```

The number prefix (`1.`, `2.`, `3.`) is the **selection position**, not the list
position. Toggling item 1 off renumbers items 2 and 3 to 1 and 2.

### Why toggle-order over pick-up-and-place?

`requestty::OrderSelect` uses a "pick-up-and-place" model: press `Space` to grab
an item, move the cursor, press `Space` again to drop it. This reorders the full
list but is heavier for the common case of "select a few items in priority
order."

Toggle-order is faster for subset selection (just press Space on each item you
want, in order) and conceptually simpler. For full-list reordering, use
`dialoguer::Sort` or `requestty::OrderSelect` — they serve that case well.

## Key bindings

| Key | Action |
| --- | --- |
| `↑` / `k` | Move cursor up |
| `↓` / `j` | Move cursor down |
| `Space` | Toggle selection (add/remove, preserves order) |
| `a` | Select all |
| `n` | Clear selection |
| `Enter` | Confirm and return selection |
| `Esc` / `q` | Cancel (returns `Error::Cancelled`) |
| `Ctrl+C` | Cancel (returns `Error::Cancelled`) |

When `should_loop` is `true` (default), the cursor wraps from last to first and
vice-versa.

## Terminal handling

The prompt uses the **alternate screen** + **raw mode** via `crossterm`:

1. `enable_raw_mode()` — read keys without waiting for Enter.
2. `EnterAlternateScreen` — render on a clean buffer, restore on exit.
3. `cursor::Hide` — hide the cursor during interaction.
4. On exit (confirm or cancel): `LeaveAlternateScreen`, `cursor::Show`,
   `disable_raw_mode()`.

This ensures the terminal is always restored, even on panic (the restore calls
use `let _ =` to ignore errors during cleanup).

## Pagination

When the option list exceeds `page_size` (default 15), the view scrolls to keep
the cursor roughly centered:

- If the cursor is near the top (< `page_size / 2`), the window starts at 0.
- If near the bottom, the window ends at the last item.
- Otherwise, the cursor is centered in the visible window.

## Architecture

```
src/
├── lib.rs     OrderedSelect<T> builder + run() interaction loop
├── error.rs   Error enum (5 variants)
└── bin/
    └── orderselect.rs   CLI binary (feature-gated behind "cli")
```

### Library is backend-only

The library depends solely on `crossterm` — no `inquire`, no `console`, no
`dialoguer`. This keeps the dependency tree minimal and avoids version conflicts.

### CLI is feature-gated

The CLI binary requires `clap`, which is optional. Users who only need the
library pay no compilation cost for clap:

```toml
# Library only (default)
orderselect = "0.1"

# Library + CLI
orderselect = { version = "0.1", features = ["cli"] }
```

### Borrow-based prompt loop

`run(&self)` borrows `self` rather than consuming it. This allows
`prompt_values()` to access `self.options` after the interaction loop returns,
without cloning the options upfront:

```rust
pub fn prompt_values(self) -> Result<Vec<T>> {
    let indices = self.run()?;          // borrows self
    Ok(indices.iter()                    // still has self.options
        .filter_map(|&i| self.options.get(i).cloned())
        .collect())
}
```

`prompt()` consumes `self` (the common case), while `prompt_values()` needs the
borrow-based internal API.
