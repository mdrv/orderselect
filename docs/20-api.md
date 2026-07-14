# API Reference

## `OrderedSelect<T>`

The main prompt type. Generic over `T: Display`.

### Construction

```rust
use orderselect::OrderedSelect;

let prompt = OrderedSelect::new("Select build targets", vec![
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]);
```

### Builder methods

| Method | Default | Description |
| --- | --- | --- |
| `.with_defaults(&[usize])` | `[]` | Pre-selected indices (order preserved). |
| `.with_help_message(&str)` | built-in key hints | Custom help line below options. |
| `.with_page_size(usize)` | `15` | Max visible options (clamped to 5). |
| `.should_loop(bool)` | `true` | Cursor wraps last ↔ first. |
| `.with_symbols(Symbols)` | Unicode glyphs | Custom cursor/checked/unchecked marks. |

### Prompt methods

```rust
// Returns selected indices in toggle order
let indices: Vec<usize> = prompt.prompt()?;

// Returns cloned values in toggle order (requires T: Clone)
let values: Vec<&str> = prompt.prompt_values()?;
```

### Full example

```rust
use orderselect::{OrderedSelect, Symbols, ASCII_SYMBOLS};

let options = vec!["rust", "go", "python", "node", "bun"];
let selected = OrderedSelect::new("Pick languages (in priority order)", options)
    .with_defaults(&[0])              // rust pre-selected
    .with_page_size(10)
    .should_loop(false)
    .prompt_values()?;                // → Vec<&str>

println!("Selected: {:?}", selected);
```

## `Error`

```rust
pub enum Error {
    NotATerminal,                                    // stdin is not a TTY
    Cancelled,                                       // user pressed Esc/q/Ctrl+C
    Io(std::io::Error),                              // I/O failure
    EmptyOptions,                                    // no options provided
    InvalidDefault { index: usize, len: usize },     // default index out of range
}
```

### Non-TTY fallback pattern

```rust
use orderselect::{OrderedSelect, Error};

match OrderedSelect::new("Pick", items).prompt() {
    Ok(indices) => process(indices),
    Err(Error::NotATerminal) => fallback_to_defaults(),
    Err(Error::Cancelled) => std::process::exit(130),
    Err(e) => panic!("{e}"),
}
```

## `Symbols`

```rust
pub struct Symbols {
    pub cursor: &'static str,      // default: "❯"
    pub checked: &'static str,     // default: "◆"
    pub unchecked: &'static str,   // default: "◇"
}
```

Use `ASCII_SYMBOLS` for environments without Unicode:

```rust
use orderselect::ASCII_SYMBOLS;

OrderedSelect::new("Pick", opts)
    .with_symbols(ASCII_SYMBOLS)
    .prompt()?;
```

## Using alongside `inquire`

Both crates use `crossterm`, so they work together seamlessly:

```rust
use inquire::Confirm;
use orderselect::OrderedSelect;

// Use inquire for simple prompts
let deploy = Confirm::new("Deploy after build?").prompt()?;

// Use orderselect for ordered selection
let targets = OrderedSelect::new("Build targets", vec![
    "linux-gnu", "linux-musl", "darwin", "windows",
])
.with_defaults(&[0, 1])
.prompt_values()?;
```
