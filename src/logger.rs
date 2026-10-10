//! Console logger with the colors of the `@std/log` handler used by the Deno
//! version: info is white, warnings yellow, errors red and debug output
//! (only with `--debug`) gray. Everything goes to stdout.

use crate::colors;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG: AtomicBool = AtomicBool::new(false);

pub fn init(debug: bool) {
    DEBUG.store(debug, Ordering::Relaxed);
}

pub fn is_debug() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

/// Writes a line to stdout, ignoring errors such as a closed pipe.
pub fn print_line(line: &str) {
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{line}");
    let _ = stdout.flush();
}

pub fn debug(message: impl AsRef<str>) {
    if is_debug() {
        print_line(&colors::gray(message.as_ref()));
    }
}

pub fn info(message: impl AsRef<str>) {
    print_line(&colors::white(message.as_ref()));
}

pub fn warn(message: impl AsRef<str>) {
    print_line(&colors::bright_yellow(message.as_ref()));
}

pub fn error(message: impl AsRef<str>) {
    print_line(&colors::bright_red(message.as_ref()));
}
