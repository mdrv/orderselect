//! `orderselect` CLI binary.
//!
//! Reads options from positional args or stdin, runs the ordered multi-select
//! prompt, and outputs the selected items (newline-separated) to stdout.

use clap::Parser;
use orderselect::OrderedSelect;

/// Ordered multi-select prompt for the terminal.
///
/// Reads options from positional args or stdin (one per line). Outputs the
/// selected items in toggle order, newline-separated.
#[derive(Parser)]
#[command(name = "orderselect", version, about)]
struct Cli {
    /// Prompt message shown above the option list.
    #[arg(short, long)]
    message: Option<String>,

    /// Output selected indices (0-based) instead of values.
    #[arg(short, long)]
    indices: bool,

    /// Comma-separated default indices (e.g. "0,1,3").
    #[arg(long, value_delimiter = ',')]
    defaults: Option<Vec<usize>>,

    /// Maximum options visible at once (min 5).
    #[arg(long, default_value_t = 15)]
    page_size: usize,

    /// Options to select from. If omitted, reads from stdin (one per line).
    items: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    let items = if cli.items.is_empty() {
        read_stdin_lines()
    } else {
        cli.items
    };

    if items.is_empty() {
        eprintln!("orderselect: no options provided");
        std::process::exit(1);
    }

    let message = cli.message.as_deref().unwrap_or("Select:");
    let items_for_output = items.clone();

    let mut prompt = OrderedSelect::new(message, items).with_page_size(cli.page_size);

    if let Some(defaults) = &cli.defaults {
        prompt = prompt.with_defaults(defaults);
    }

    match prompt.prompt() {
        Ok(indices) => {
            if cli.indices {
                for i in indices {
                    println!("{i}");
                }
            } else {
                for i in indices {
                    if let Some(val) = items_for_output.get(i) {
                        println!("{val}");
                    }
                }
            }
        }
        Err(orderselect::Error::NotATerminal) => {
            eprintln!("orderselect: stdin is not a terminal");
            std::process::exit(2);
        }
        Err(orderselect::Error::Cancelled) => {
            std::process::exit(130);
        }
        Err(e) => {
            eprintln!("orderselect: {e}");
            std::process::exit(1);
        }
    }
}

fn read_stdin_lines() -> Vec<String> {
    use std::io::Read;
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        return Vec::new();
    }
    input
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}
