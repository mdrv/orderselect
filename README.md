# orderselect

Ordered multi-select prompt for Rust CLIs.

Lets the user **select a subset** of options and **preserves the order** in
which they were toggled. No existing prompt library (inquire, dialoguer,
requestty) tracks toggle order.

## Quick start

```toml
[dependencies]
orderselect = "0.1"
```

```rust
use orderselect::OrderedSelect;

let options = vec!["linux-gnu", "linux-musl", "darwin", "windows"];
let selected = OrderedSelect::new("Select build targets", options)
    .with_defaults(&[0, 1])
    .prompt_values()
    .unwrap_or_default();

println!("{:?}", selected); // ["linux-gnu", "linux-musl", "darwin"]
```

Recommended alongside [`inquire`](https://crates.io/crates/inquire) — both use
`crossterm`, so they coexist without terminal conflicts.

## CLI

```sh
cargo install orderselect --features cli

echo -e "linux-gnu\ndarwin\nwindows" | orderselect -m "Select targets"
```

Outputs selected items newline-separated, in toggle order.

## Why?

| Library | Subset? | Ordered? |
| --- | --- | --- |
| `inquire::MultiSelect` | Yes | No |
| `dialoguer::Sort` | No | Yes |
| `requestty::OrderSelect` | No | Yes |
| **`orderselect`** | **Yes** | **Yes** |

## Docs

- [Overview](docs/00-overview.md)
- [Design](docs/10-design.md)
- [API Reference](docs/20-api.md)
- [CLI](docs/30-cli.md)

## License

MIT
