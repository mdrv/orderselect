//! # orderselect
//!
//! Ordered multi-select prompt for Rust CLIs.
//!
//! Lets the user **select a subset** of options and **preserves the order** in
//! which they were toggled. No existing prompt library (inquire, dialoguer,
//! requestty) tracks toggle order — they either select without ordering or
//! reorder the full list.
//!
//! ## Quick start
//!
//! ```no_run
//! use orderselect::OrderedSelect;
//!
//! let options = vec!["linux-gnu", "linux-musl", "darwin", "windows"];
//! let indices = OrderedSelect::new("Select build targets", options)
//!     .with_defaults(&[0, 1])
//!     .prompt()
//!     .unwrap_or_default();
//! ```
//!
//! Recommended alongside [`inquire`](https://crates.io/crates/inquire) — both
//! use [`crossterm`](https://crates.io/crates/crossterm) under the hood, so
//! they coexist without terminal conflicts.

#![forbid(unsafe_code)]

mod error;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

use std::fmt::Display;
use std::io::{self, IsTerminal, Write};

use crossterm::{cursor, event, execute, terminal};

// ---------------------------------------------------------------------------
// Symbols
// ---------------------------------------------------------------------------

/// Unicode glyphs used in the prompt rendering.
///
/// Override these to customise the visual style or to use ASCII-only fallbacks.
#[derive(Debug, Clone)]
pub struct Symbols {
    /// Shown next to the item under the cursor (default `❯`).
    pub cursor: &'static str,
    /// Shown next to a selected item (default `◆`).
    pub checked: &'static str,
    /// Shown next to an unselected item (default `◇`).
    pub unchecked: &'static str,
}

impl Default for Symbols {
    fn default() -> Self {
        Self {
            cursor: "❯",
            checked: "◆",
            unchecked: "◇",
        }
    }
}

/// ASCII-only symbol set for environments without Unicode support.
pub const ASCII_SYMBOLS: Symbols = Symbols {
    cursor: ">",
    checked: "[*]",
    unchecked: "[ ]",
};

// ---------------------------------------------------------------------------
// OrderedSelect
// ---------------------------------------------------------------------------

/// Ordered multi-select prompt.
///
/// The user toggles items with `Space`; the **toggle order** is preserved —
/// the first item toggled is the first item in the returned `Vec`.
///
/// Create with [`OrderedSelect::new`], chain builder methods, then call
/// `.prompt()`.
pub struct OrderedSelect<'a, T> {
    message: &'a str,
    options: Vec<T>,
    defaults: Vec<usize>,
    help_message: Option<&'a str>,
    page_size: usize,
    should_loop: bool,
    symbols: Symbols,
}

impl<'a, T: Display> OrderedSelect<'a, T> {
    /// Create a new prompt.
    ///
    /// `message` is the heading line shown above the option list.
    /// `options` must be non-empty (calling `.prompt()` on an empty list
    /// returns [`Error::EmptyOptions`]).
    pub fn new(message: &'a str, options: Vec<T>) -> Self {
        Self {
            message,
            options,
            defaults: Vec::new(),
            help_message: None,
            page_size: 15,
            should_loop: true,
            symbols: Symbols::default(),
        }
    }

    /// Pre-select these indices when the prompt first renders.
    ///
    /// They appear in the given order. Out-of-range indices cause
    /// [`Error::InvalidDefault`] at prompt time.
    pub fn with_defaults(mut self, defaults: &[usize]) -> Self {
        self.defaults = defaults.to_vec();
        self
    }

    /// Show a custom help line below the options (replaces the default
    /// key-binding hint).
    pub fn with_help_message(mut self, msg: &'a str) -> Self {
        self.help_message = Some(msg);
        self
    }

    /// Maximum number of options visible at once (default 15, clamped to 5).
    ///
    /// When the list is longer, it scrolls to keep the cursor centered.
    pub fn with_page_size(mut self, size: usize) -> Self {
        self.page_size = size.max(5);
        self
    }

    /// Whether the cursor wraps from last to first and vice-versa (default
    /// `true`).
    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.should_loop = should_loop;
        self
    }

    /// Override the default Unicode glyphs.
    pub fn with_symbols(mut self, symbols: Symbols) -> Self {
        self.symbols = symbols;
        self
    }

    // -- prompt entry points ----------------------------------------------

    /// Run the prompt and return the **selected indices** in toggle order.
    ///
    /// Returns [`Error::NotATerminal`] if stdin is not a TTY, and
    /// [`Error::Cancelled`] if the user presses `Esc`, `q`, or `Ctrl+C`.
    pub fn prompt(self) -> Result<Vec<usize>> {
        self.check_preconditions()?;
        self.run()
    }

    /// Run the prompt and return **cloned values** in toggle order.
    ///
    /// Convenience wrapper around [`prompt`](Self::prompt) for when you need
    /// the actual option values rather than indices.
    pub fn prompt_values(self) -> Result<Vec<T>>
    where
        T: Clone,
    {
        self.check_preconditions()?;
        let indices = self.run()?;
        Ok(indices
            .iter()
            .filter_map(|&i| self.options.get(i).cloned())
            .collect())
    }

    // -- internal ---------------------------------------------------------

    fn validate(&self) -> Result<()> {
        if self.options.is_empty() {
            return Err(Error::EmptyOptions);
        }
        for &i in &self.defaults {
            if i >= self.options.len() {
                return Err(Error::InvalidDefault {
                    index: i,
                    len: self.options.len(),
                });
            }
        }
        Ok(())
    }

    fn check_preconditions(&self) -> Result<()> {
        self.validate()?;
        if !io::stdin().is_terminal() {
            return Err(Error::NotATerminal);
        }
        Ok(())
    }

    /// Core interaction loop. Borrows `&self` so that `prompt_values` can
    /// still read `self.options` after this returns.
    fn run(&self) -> Result<Vec<usize>> {
        let n = self.options.len();
        let mut cur: usize = 0;
        let mut selected: Vec<usize> = self.defaults.clone();
        let mut stdout = io::stdout();

        let _ = terminal::enable_raw_mode();
        let _ = execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide);

        let result: Option<Vec<usize>> = loop {
            let (tw, th) = terminal::size().unwrap_or((80, 24));
            let tw = tw as usize;
            let th = th as usize;

            let _ = execute!(
                stdout,
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0)
            );

            let msg = truncate(self.message, tw.saturating_sub(2));
            let _ = write!(stdout, "  {msg}\r\n\r\n");

            // visible window — clamped to terminal height (msg + blank + options + blank + help)
            let avail_h = th.saturating_sub(4).max(1);
            let page = self.page_size.min(n).min(avail_h);
            let half = page / 2;
            let start = if n <= page || cur < half {
                0
            } else if cur >= n.saturating_sub(half) {
                n.saturating_sub(page)
            } else {
                cur - half
            };
            let end = (start + page).min(n);

            for i in start..end {
                let marker = if i == cur { self.symbols.cursor } else { " " };
                let (check, order) = match selected.iter().position(|&s| s == i) {
                    Some(pos) => (self.symbols.checked, format!("{}", pos + 1)),
                    None => (self.symbols.unchecked, String::from(" ")),
                };
                let prefix_w = 3 + order.len() + 2 + check.chars().count() + 1;
                let text_max = tw.saturating_sub(prefix_w).max(5);
                let text = truncate(&format!("{}", self.options[i]), text_max);
                let _ = write!(stdout, " {marker} {order}. {check} {text}\r\n");
            }

            let help = self.help_message.unwrap_or(if tw < 50 {
                "\u{2191}\u{2193}/sp/a/n/enter/esc"
            } else {
                "\u{2191}\u{2193} move \u{00b7} space toggle \u{00b7} a all \u{00b7} \
                 n none \u{00b7} enter confirm \u{00b7} esc cancel"
            });
            let help = truncate(help, tw.saturating_sub(2));
            let _ = write!(stdout, "\r\n  {help}\r\n");
            let _ = stdout.flush();

            let ev = match event::read() {
                Ok(ev) => ev,
                Err(_) => break None,
            };

            if let event::Event::Key(key) = ev {
                match key.code {
                    event::KeyCode::Up | event::KeyCode::Char('k') => {
                        if self.should_loop && cur == 0 {
                            cur = n - 1;
                        } else {
                            cur = cur.saturating_sub(1);
                        }
                    }
                    event::KeyCode::Down | event::KeyCode::Char('j') => {
                        if self.should_loop && cur + 1 >= n {
                            cur = 0;
                        } else if cur + 1 < n {
                            cur += 1;
                        }
                    }
                    event::KeyCode::Char(' ') => toggle(&mut selected, cur),
                    event::KeyCode::Char('a') => selected = (0..n).collect(),
                    event::KeyCode::Char('n') => selected.clear(),
                    event::KeyCode::Char('c')
                        if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                    {
                        break None;
                    }
                    event::KeyCode::Esc | event::KeyCode::Char('q') => break None,
                    event::KeyCode::Enter => break Some(selected.clone()),
                    _ => {}
                }
            }
        };

        let _ = execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();

        match result {
            Some(indices) => Ok(indices),
            None => Err(Error::Cancelled),
        }
    }
}

fn toggle(selected: &mut Vec<usize>, index: usize) {
    if let Some(pos) = selected.iter().position(|&s| s == index) {
        selected.remove(pos);
    } else {
        selected.push(index);
    }
}

/// Truncate a string to `max_chars` columns (by char count), appending an
/// ellipsis (…) when truncated.
fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    if max_chars <= 1 {
        return "\u{2026}".to_string();
    }
    let truncated: String = s.chars().take(max_chars - 1).collect();
    format!("{truncated}\u{2026}")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_options_errors() {
        let result = OrderedSelect::new("test", Vec::<String>::new()).validate();
        assert!(matches!(result, Err(Error::EmptyOptions)));
    }

    #[test]
    fn valid_defaults_pass() {
        let os = OrderedSelect::new("test", vec!["a", "b", "c"]).with_defaults(&[0, 2]);
        assert!(os.validate().is_ok());
    }

    #[test]
    fn out_of_range_default_errors() {
        let os = OrderedSelect::new("test", vec!["a", "b"]).with_defaults(&[5]);
        assert!(matches!(
            os.validate(),
            Err(Error::InvalidDefault { index: 5, len: 2 })
        ));
    }

    #[test]
    fn toggle_adds_and_removes() {
        let mut sel = vec![];
        toggle(&mut sel, 0);
        toggle(&mut sel, 2);
        assert_eq!(sel, vec![0, 2]);

        toggle(&mut sel, 0);
        assert_eq!(sel, vec![2]);

        toggle(&mut sel, 2);
        assert!(sel.is_empty());
    }

    #[test]
    fn toggle_preserves_order() {
        let mut sel = vec![];
        toggle(&mut sel, 3);
        toggle(&mut sel, 1);
        toggle(&mut sel, 0);
        assert_eq!(sel, vec![3, 1, 0]);
    }

    #[test]
    fn defaults_preserve_order() {
        let os = OrderedSelect::new("test", vec!["a", "b", "c", "d"]).with_defaults(&[2, 0, 3]);
        assert_eq!(os.defaults, vec![2, 0, 3]);
    }

    #[test]
    fn page_size_clamped_to_5() {
        let os = OrderedSelect::new("test", vec!["a"]).with_page_size(2);
        assert_eq!(os.page_size, 5);
    }

    #[test]
    fn symbols_default() {
        let s = Symbols::default();
        assert_eq!(s.cursor, "\u{276f}");
        assert_eq!(s.checked, "\u{25c6}");
        assert_eq!(s.unchecked, "\u{25c7}");
    }

    #[test]
    fn ascii_symbols() {
        assert_eq!(ASCII_SYMBOLS.cursor, ">");
        assert!(ASCII_SYMBOLS.checked.starts_with('['));
    }

    #[test]
    fn prompt_on_non_tty_errors() {
        let result = OrderedSelect::new("test", vec!["a", "b"]).prompt();
        assert!(matches!(result, Err(Error::NotATerminal) | Ok(_)));
    }
}
