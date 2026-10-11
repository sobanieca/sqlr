//! ANSI colors with the semantics of `@std/fmt/colors`.

use std::sync::OnceLock;

/// Colors are disabled when `NO_COLOR` is set.
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty()))
}

/// Nested resets of the same kind re-open the outer style so it applies to
/// the whole string.
fn run(s: &str, open: &str, close: &str) -> String {
    if !enabled() {
        return s.to_string();
    }
    let open = format!("\x1b[{open}m");
    let close = format!("\x1b[{close}m");
    format!("{open}{}{close}", s.replace(&close, &open))
}

pub fn bold(s: &str) -> String {
    run(s, "1", "22")
}

pub fn dim(s: &str) -> String {
    run(s, "2", "22")
}

pub fn italic(s: &str) -> String {
    run(s, "3", "23")
}

pub fn underline(s: &str) -> String {
    run(s, "4", "24")
}

pub fn red(s: &str) -> String {
    run(s, "31", "39")
}

pub fn green(s: &str) -> String {
    run(s, "32", "39")
}

pub fn yellow(s: &str) -> String {
    run(s, "33", "39")
}

pub fn white(s: &str) -> String {
    run(s, "37", "39")
}

pub fn gray(s: &str) -> String {
    run(s, "90", "39")
}

pub fn bright_red(s: &str) -> String {
    run(s, "91", "39")
}

pub fn bright_green(s: &str) -> String {
    run(s, "92", "39")
}

pub fn bright_yellow(s: &str) -> String {
    run(s, "93", "39")
}

pub fn bright_blue(s: &str) -> String {
    run(s, "94", "39")
}

pub fn bright_magenta(s: &str) -> String {
    run(s, "95", "39")
}

pub fn rgb24(s: &str, color: u32) -> String {
    let (r, g, b) = ((color >> 16) & 0xff, (color >> 8) & 0xff, color & 0xff);
    run(s, &format!("38;2;{r};{g};{b}"), "39")
}

/// Removes ANSI escape sequences (`ESC [ ... letter`).
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            while let Some(&next) = chars.peek() {
                chars.next();
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}
