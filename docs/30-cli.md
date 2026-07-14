# CLI

The `orderselect` binary reads options from positional args or stdin, runs the
ordered multi-select prompt, and outputs the selected items newline-separated.

## Installation

```sh
cargo install orderselect --features cli
# or build from source:
cargo build --release --features cli
```

## Usage

### From arguments

```sh
orderselect -m "Select build targets" -- \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu \
    aarch64-apple-darwin \
    x86_64-pc-windows-msvc
```

### From stdin (pipe)

```sh
echo -e "linux-gnu\ndarwin\nwindows" | orderselect -m "Select targets"
```

### With defaults

```sh
orderselect -m "Select targets" --defaults 0,2 linux-gnu darwin windows
```

Pre-selects items 0 and 2 (linux-gnu and windows).

## Flags

| Flag | Short | Description |
| --- | --- | --- |
| `--message <MSG>` | `-m` | Prompt message shown above options. |
| `--indices` | `-i` | Output indices (0-based) instead of values. |
| `--defaults <CSV>` | | Comma-separated default indices (e.g. `0,1,3`). |
| `--page-size <N>` | | Max options visible at once (default 15, min 5). |
| `--help` | `-h` | Show help. |
| `--version` | `-V` | Show version. |

## Output

Selected items are printed one per line, **in toggle order**:

```
linux-gnu
darwin
windows
```

With `--indices`:

```
0
2
3
```

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success — selection printed to stdout. |
| `1` | Error (no options, invalid defaults, I/O failure). |
| `2` | stdin is not a terminal. |
| `130` | User cancelled (Esc/q/Ctrl+C). |

## Shell script integration

```sh
#!/bin/bash
targets=$(orderselect -m "Build targets" -- linux-gnu darwin windows)
if [ $? -eq 0 ]; then
    echo "Building: $targets"
    for t in $targets; do
        cargo build --release --target "$t"
    done
fi
```

### Reading options from a file

```sh
orderselect -m "Pick tasks" < tasks.txt
```

Each non-empty line in `tasks.txt` becomes an option.
