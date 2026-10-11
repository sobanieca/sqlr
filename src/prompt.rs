//! Interactive prompts modeled after `@cliffy/prompt` (Input, Secret,
//! Select and Toggle): same rendering, figures, colors and key bindings.
//! Prompts are written to stdout and read keys from stdin in raw mode.

use crate::colors;
use std::io::{BufRead, IsTerminal, Write};

const POINTER_SMALL: &str = "›";
const POINTER: &str = "❯";
const CROSS: &str = "✘";
const CLEAR: &str = "\x1b[G\x1b[0J";
const HIDE_CURSOR: &str = "\x1b[?25l";
const SHOW_CURSOR: &str = "\x1b[?25h";
const MAX_ROWS: usize = 10;

/// Asks for a line of text. Returns an empty string when nothing is entered.
pub fn input(message: &str) -> Result<String, String> {
    if !std::io::stdin().is_terminal() {
        let value = read_line()?.trim().to_string();
        write_out(&format!("{}\n", success_line(message, &value)));
        return Ok(value);
    }
    run(&mut TextPrompt::new(message, false))
}

/// Asks for a hidden value (passwords).
pub fn secret(message: &str) -> Result<String, String> {
    if !std::io::stdin().is_terminal() {
        let value = read_line()?;
        let masked = "*".repeat(value.chars().count());
        write_out(&format!("{}\n", success_line(message, &masked)));
        return Ok(value);
    }
    run(&mut TextPrompt::new(message, true))
}

/// Lets the user pick one of `options` (name shown, value returned).
pub fn select(message: &str, options: &[(String, String)]) -> Result<String, String> {
    if options.is_empty() {
        return Err("No options to select from.".to_string());
    }
    require_terminal(message)?;
    run(&mut SelectPrompt::new(message, options))
}

/// Yes/no question.
pub fn toggle(message: &str) -> Result<bool, String> {
    require_terminal(message)?;
    run(&mut TogglePrompt::new(message))
}

fn require_terminal(message: &str) -> Result<(), String> {
    if std::io::stdin().is_terminal() {
        Ok(())
    } else {
        Err(format!(
            "Cannot prompt \"{message}\": stdin is not a terminal (interactive input required)."
        ))
    }
}

/// Reads one line from a non-interactive stdin, without the line ending.
fn read_line() -> Result<String, String> {
    let mut line = String::new();
    let read = std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| format!("Failed to read from stdin: {e}"))?;
    if read == 0 {
        return Err("Failed to read from stdin: no more input.".to_string());
    }
    let len = line.trim_end_matches(['\r', '\n']).len();
    line.truncate(len);
    Ok(line)
}

fn write_out(s: &str) {
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(s.as_bytes());
    let _ = stdout.flush();
}

fn header(message: &str) -> String {
    format!("{}{}", colors::yellow("? "), colors::bold(message))
}

fn pointer() -> String {
    colors::bright_blue(POINTER_SMALL)
}

/// The line left on screen once a prompt is answered.
fn success_line(message: &str, formatted: &str) -> String {
    format!(
        "{} {} {}",
        header(message),
        pointer(),
        colors::green(formatted)
    )
}

/// Restores the original terminal settings when dropped.
struct RawMode {
    original: libc::termios,
}

impl RawMode {
    /// Like Deno's `stdin.setRaw(true)`, but keeps output post-processing so
    /// `\n` still moves to the start of the next line.
    fn enable() -> Result<Self, String> {
        let mut original = std::mem::MaybeUninit::<libc::termios>::uninit();
        // SAFETY: tcgetattr fills the struct on success.
        let original = unsafe {
            if libc::tcgetattr(libc::STDIN_FILENO, original.as_mut_ptr()) != 0 {
                return Err(format!(
                    "Failed to read terminal settings: {}",
                    std::io::Error::last_os_error()
                ));
            }
            original.assume_init()
        };
        let mut raw = original;
        raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
        raw.c_cflag |= libc::CS8;
        raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        // SAFETY: raw is a valid termios copied from the current settings.
        if unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &raw) } != 0 {
            return Err(format!(
                "Failed to enable raw mode: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(Self { original })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // SAFETY: restores the settings read in `enable`.
        unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &self.original) };
    }
}

fn columns() -> Option<usize> {
    // SAFETY: TIOCGWINSZ fills a winsize struct.
    unsafe {
        let mut size: libc::winsize = std::mem::zeroed();
        (libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) == 0 && size.ws_col > 0)
            .then_some(size.ws_col as usize)
    }
}

/// Reads one chunk of key presses, waiting for incomplete UTF-8 sequences.
fn read_keys() -> Result<Vec<Key>, String> {
    let mut data = Vec::new();
    let mut buffer = [0u8; 1024];
    loop {
        // SAFETY: reads at most buffer.len() bytes into buffer.
        let read =
            unsafe { libc::read(libc::STDIN_FILENO, buffer.as_mut_ptr().cast(), buffer.len()) };
        if read < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(format!("Failed to read from stdin: {error}"));
        }
        if read == 0 {
            return Err("Failed to read from stdin: no more input.".to_string());
        }
        data.extend_from_slice(&buffer[..read as usize]);
        match std::str::from_utf8(&data) {
            Ok(text) => return Ok(parse_keys(text)),
            Err(e) if e.error_len().is_none() => continue,
            Err(_) => return Ok(parse_keys(&String::from_utf8_lossy(&data))),
        }
    }
}

/// A key press decoded like `@cliffy/keycode`.
#[derive(Debug, Default, PartialEq)]
struct Key {
    name: String,
    char: Option<char>,
    sequence: String,
    ctrl: bool,
    meta: bool,
    shift: bool,
}

impl Key {
    /// Matches a key name, a raw sequence or a shortcut like `ctrl+w`.
    fn is(&self, key_name: &str) -> bool {
        if key_name.len() > 1
            && let Some((modifiers, name)) = key_name.rsplit_once('+')
        {
            return self.name == name
                && modifiers.split('+').all(|modifier| match modifier {
                    "ctrl" => self.ctrl,
                    "meta" | "alt" => self.meta,
                    "shift" => self.shift,
                    _ => false,
                });
        }
        self.name == key_name || self.sequence == key_name
    }

    fn is_any(&self, key_names: &[&str]) -> bool {
        key_names.iter().any(|name| self.is(name))
    }
}

fn parse_keys(input: &str) -> Vec<Key> {
    let chars: Vec<char> = input.chars().collect();
    let mut keys = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        let next = |i: &mut usize| {
            *i += 1;
            chars.get(*i - 1).copied()
        };
        let mut ch = next(&mut i);
        let mut escaped = false;
        let mut key = Key::default();

        if ch == Some('\x1b') && i < chars.len() {
            escaped = true;
            ch = next(&mut i);
            if ch == Some('\x1b') {
                ch = next(&mut i);
            }
        }

        if escaped && matches!(ch, Some('O' | '[')) {
            let mut code = ch.unwrap().to_string();
            let mut modifier = 0;
            if ch == Some('O') {
                ch = next(&mut i);
                if let Some(digit) = ch.and_then(|c| c.to_digit(10)) {
                    modifier = digit as i32 - 1;
                    ch = next(&mut i);
                }
                code.extend(ch);
            } else {
                ch = next(&mut i);
                if ch == Some('[') {
                    code.push('[');
                    ch = next(&mut i);
                }
                let cmd_start = i - 1;
                let is_digit = |c: Option<char>| c.is_some_and(|c| c.is_ascii_digit());
                if is_digit(ch) {
                    ch = next(&mut i);
                    if is_digit(ch) {
                        ch = next(&mut i);
                        if is_digit(ch) {
                            ch = next(&mut i);
                        }
                    }
                }
                if ch == Some(';') {
                    ch = next(&mut i);
                    if is_digit(ch) {
                        next(&mut i);
                    }
                }
                let end = i.min(chars.len());
                let cmd: String = chars[cmd_start.min(end)..end].iter().collect();
                let (suffix, parsed) = decode_command(&cmd);
                code.push_str(&suffix);
                modifier = parsed;
            }

            key.ctrl = modifier & 4 != 0;
            key.meta = modifier & 10 != 0;
            key.shift = modifier & 1 != 0;
            if let Some(name) = key_name(&code) {
                key.name = name.to_string();
            } else if let Some(name) = shift_key_name(&code) {
                key.name = name.to_string();
                key.shift = true;
            } else if let Some(name) = ctrl_key_name(&code) {
                key.name = name.to_string();
                key.ctrl = true;
            } else {
                key.name = "undefined".to_string();
            }
        } else if let Some(name) = ch.and_then(special_key_name) {
            key.name = name.to_string();
            key.meta = escaped;
            if name == "space" {
                key.char = ch;
            }
        } else if let Some(c) = ch.filter(|&c| !escaped && c <= '\x1a') {
            let letter = char::from(c as u8 + b'a' - 1);
            key.name = letter.to_string();
            key.ctrl = true;
            key.char = Some(letter);
        } else if let Some(c) = ch.filter(|c| c.is_ascii_alphanumeric()) {
            key.name = c.to_ascii_lowercase().to_string();
            key.shift = c.is_ascii_uppercase();
            key.meta = escaped;
            key.char = Some(c);
        } else if escaped {
            if ch.is_none() {
                key.name = "escape".to_string();
            }
            key.meta = true;
        } else if let Some(c) = ch {
            key.name = c.to_string();
            key.char = Some(c);
        }

        let end = i.min(chars.len());
        key.sequence = chars[start..end].iter().collect();
        keys.push(key);
    }
    keys
}

/// Splits the command part of a CSI sequence into its code suffix and
/// modifier (`24;5~` -> `24~`, 4; `1;5D` -> `D`, 4).
fn decode_command(cmd: &str) -> (String, i32) {
    let b = cmd.as_bytes();
    let digits = b.iter().take_while(|c| c.is_ascii_digit()).count();
    let modifier = |d: u8| {
        let n = (d - b'0') as i32;
        if n == 0 { 0 } else { n - 1 }
    };
    let is_end = |c: &u8| matches!(c, b'~' | b'^' | b'$');
    if digits == 3 && b.len() == 4 && b[3] == b'~' {
        return (cmd.to_string(), 0);
    }
    if (1..=2).contains(&digits) {
        let number = &cmd[..digits];
        match &b[digits..] {
            [end] if is_end(end) => return (format!("{number}{}", *end as char), 0),
            [b';', d, end] if d.is_ascii_digit() && is_end(end) => {
                return (format!("{number}{}", *end as char), modifier(*d));
            }
            _ => {}
        }
    }
    match b {
        [l] if l.is_ascii_alphabetic() => ((*l as char).to_string(), 0),
        [d, l] if d.is_ascii_digit() && l.is_ascii_alphabetic() => {
            ((*l as char).to_string(), modifier(*d))
        }
        [_, b';', d, l]
            if b[0].is_ascii_digit() && d.is_ascii_digit() && l.is_ascii_alphabetic() =>
        {
            ((*l as char).to_string(), modifier(*d))
        }
        _ => (cmd.to_string(), 0),
    }
}

fn key_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "[P" | "OP" | "[11~" | "[[A" => "f1",
        "[Q" | "OQ" | "[12~" | "[[B" => "f2",
        "[R" | "OR" | "[13~" | "[[C" => "f3",
        "[S" | "OS" | "[14~" | "[[D" => "f4",
        "[200~" => "paste-start",
        "[201~" => "paste-end",
        "[[E" | "[15~" => "f5",
        "[17~" => "f6",
        "[18~" => "f7",
        "[19~" => "f8",
        "[20~" => "f9",
        "[21~" => "f10",
        "[23~" => "f11",
        "[24~" => "f12",
        "[A" | "OA" => "up",
        "[B" | "OB" => "down",
        "[C" | "OC" => "right",
        "[D" | "OD" => "left",
        "[E" | "OE" => "clear",
        "[F" | "OF" | "[4~" | "[8~" => "end",
        "[H" | "OH" | "[1~" | "[7~" => "home",
        "[2~" => "insert",
        "[3~" => "delete",
        "[5~" | "[[5~" => "pageup",
        "[6~" | "[[6~" => "pagedown",
        _ => return None,
    })
}

fn shift_key_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "[a" => "up",
        "[b" => "down",
        "[c" => "right",
        "[d" => "left",
        "[e" => "clear",
        "[2$" => "insert",
        "[3$" => "delete",
        "[5$" => "pageup",
        "[6$" => "pagedown",
        "[7$" => "home",
        "[8$" => "end",
        "[Z" => "tab",
        _ => return None,
    })
}

fn ctrl_key_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "Oa" => "up",
        "Ob" => "down",
        "Oc" => "right",
        "Od" => "left",
        "Oe" => "clear",
        "[2^" => "insert",
        "[3^" => "delete",
        "[5^" => "pageup",
        "[6^" => "pagedown",
        "[7^" => "home",
        "[8^" => "end",
        _ => return None,
    })
}

fn special_key_name(c: char) -> Option<&'static str> {
    Some(match c {
        '\r' => "return",
        '\n' => "enter",
        '\t' => "tab",
        '\x08' | '\x7f' => "backspace",
        '\x1b' => "escape",
        ' ' => "space",
        _ => return None,
    })
}

/// Terminal column width of a string, ignoring ANSI escape sequences.
fn display_width(s: &str) -> usize {
    colors::strip_ansi(s).chars().map(char_width).sum()
}

fn char_width(c: char) -> usize {
    let code = c as u32;
    if code < 0x20
        || (0x7f..=0x9f).contains(&code)
        || (0x300..=0x36f).contains(&code)
        || (0x200b..=0x200f).contains(&code)
        || (0x20d0..=0x20ff).contains(&code)
        || (0xfe00..=0xfe0f).contains(&code)
        || (0xe0100..=0xe01ef).contains(&code)
    {
        0
    } else if is_wide(code) {
        2
    } else {
        1
    }
}

/// East Asian wide characters and emoji with default emoji presentation.
fn is_wide(code: u32) -> bool {
    const RANGES: &[(u32, u32)] = &[
        (0x1100, 0x115f),
        (0x231a, 0x231b),
        (0x2329, 0x232a),
        (0x23e9, 0x23ec),
        (0x23f0, 0x23f0),
        (0x23f3, 0x23f3),
        (0x25fd, 0x25fe),
        (0x2614, 0x2615),
        (0x2648, 0x2653),
        (0x267f, 0x267f),
        (0x2693, 0x2693),
        (0x26a1, 0x26a1),
        (0x26aa, 0x26ab),
        (0x26bd, 0x26be),
        (0x26c4, 0x26c5),
        (0x26ce, 0x26ce),
        (0x26d4, 0x26d4),
        (0x26ea, 0x26ea),
        (0x26f2, 0x26f3),
        (0x26f5, 0x26f5),
        (0x26fa, 0x26fa),
        (0x26fd, 0x26fd),
        (0x2705, 0x2705),
        (0x270a, 0x270b),
        (0x2728, 0x2728),
        (0x274c, 0x274c),
        (0x274e, 0x274e),
        (0x2753, 0x2755),
        (0x2757, 0x2757),
        (0x2795, 0x2797),
        (0x27b0, 0x27b0),
        (0x27bf, 0x27bf),
        (0x2b1b, 0x2b1c),
        (0x2b50, 0x2b50),
        (0x2b55, 0x2b55),
        (0x2e80, 0x303e),
        (0x3041, 0x3247),
        (0x3250, 0x4dbf),
        (0x4e00, 0xa4c6),
        (0xa960, 0xa97c),
        (0xac00, 0xd7a3),
        (0xf900, 0xfaff),
        (0xfe10, 0xfe19),
        (0xfe30, 0xfe6b),
        (0xff01, 0xff60),
        (0xffe0, 0xffe6),
        (0x1b000, 0x1b001),
        (0x1f004, 0x1f004),
        (0x1f0cf, 0x1f0cf),
        (0x1f18e, 0x1f18e),
        (0x1f191, 0x1f19a),
        (0x1f200, 0x1f251),
        (0x1f300, 0x1f64f),
        (0x1f680, 0x1f6ff),
        (0x1f7e0, 0x1f7eb),
        (0x1f900, 0x1faff),
        (0x20000, 0x3fffd),
    ];
    RANGES
        .iter()
        .any(|&(start, end)| (start..=end).contains(&code))
}

trait Prompt {
    type Value;

    /// First line of the prompt and the cursor column on it.
    fn message(&self) -> (String, usize);

    fn body(&self) -> Option<String> {
        None
    }

    fn hides_cursor(&self) -> bool;

    /// Handles a key other than submit and Ctrl+C.
    fn handle(&mut self, key: &Key);

    /// Validates the current value: the result and its formatted form, or an
    /// error to show below the prompt.
    fn submit(&self) -> Result<(Self::Value, String), String>;

    fn title(&self) -> &str;
}

/// One render of a prompt, like Cliffy's `GenericPrompt.render`.
fn frame<P: Prompt>(
    prompt: &P,
    error: Option<&str>,
    first_run: bool,
    columns: Option<usize>,
) -> String {
    let (message, cursor_x) = prompt.message();
    let footer = error.map(|e| colors::red(&format!("{}{e}", colors::bold(&format!("{CROSS} ")))));
    let content = [Some(message), prompt.body(), footer]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let lines: usize = content
        .split('\n')
        .map(|line| match columns {
            Some(columns) => display_width(line).div_ceil(columns).max(1),
            None => 1,
        })
        .sum();

    let mut out = String::new();
    if !first_run || error.is_some() {
        out.push_str(CLEAR);
    }
    out.push_str(&content);
    if lines > 1 {
        out.push_str(&format!("\x1b[{}A", lines - 1));
    }
    out.push_str(&format!("\x1b[{cursor_x}G"));
    if prompt.hides_cursor() {
        out.push_str(HIDE_CURSOR);
    }
    out
}

fn run<P: Prompt>(prompt: &mut P) -> Result<P::Value, String> {
    let raw = RawMode::enable()?;
    let columns = columns();
    let mut first_run = true;
    let mut error: Option<String> = None;
    loop {
        write_out(&frame(prompt, error.as_deref(), first_run, columns));
        first_run = false;
        error = None;
        let keys = match read_keys() {
            Ok(keys) => keys,
            Err(e) => {
                write_out(SHOW_CURSOR);
                return Err(e);
            }
        };
        for key in keys {
            if key.name == "c" && key.ctrl {
                write_out(&format!("{CLEAR}{SHOW_CURSOR}"));
                drop(raw);
                std::process::exit(130);
            }
            if key.is_any(&["enter", "return"]) {
                match prompt.submit() {
                    Ok((value, formatted)) => {
                        write_out(&format!(
                            "{CLEAR}{}\n{SHOW_CURSOR}",
                            success_line(prompt.title(), &formatted)
                        ));
                        return Ok(value);
                    }
                    Err(e) => error = Some(e),
                }
            } else {
                prompt.handle(&key);
            }
        }
    }
}

/// Input and Secret: an editable line of text.
struct TextPrompt<'a> {
    message: &'a str,
    secret: bool,
    value: Vec<char>,
    index: usize,
}

impl<'a> TextPrompt<'a> {
    fn new(message: &'a str, secret: bool) -> Self {
        Self {
            message,
            secret,
            value: Vec::new(),
            index: 0,
        }
    }

    fn word_start(&self) -> usize {
        let mut i = self.index;
        while i > 0 && self.value[i - 1] == ' ' {
            i -= 1;
        }
        while i > 0 && self.value[i - 1] != ' ' {
            i -= 1;
        }
        i
    }

    fn word_end(&self) -> usize {
        let mut i = self.index;
        while i < self.value.len() && self.value[i] != ' ' {
            i += 1;
        }
        while i < self.value.len() && self.value[i] == ' ' {
            i += 1;
        }
        i
    }
}

impl Prompt for TextPrompt<'_> {
    type Value = String;

    fn title(&self) -> &str {
        self.message
    }

    fn message(&self) -> (String, usize) {
        let message = format!("{} {} ", header(self.message), pointer());
        let (input, before_cursor) = if self.secret {
            (colors::underline(&"*".repeat(self.value.len())), self.index)
        } else {
            let text: String = self.value.iter().collect();
            let before: String = self.value[..self.index].iter().collect();
            // Input appends the (always empty) dimmed suggestion.
            (
                colors::underline(&text) + &colors::dim(""),
                display_width(&before),
            )
        };
        let x = display_width(&message) + before_cursor + 1;
        (message + &input, x)
    }

    fn hides_cursor(&self) -> bool {
        false
    }

    fn handle(&mut self, key: &Key) {
        if key.is_any(&["alt+left", "ctrl+left"]) {
            self.index = self.word_start();
        } else if key.is_any(&["alt+right", "ctrl+right"]) {
            self.index = self.word_end();
        } else if key.is_any(&["ctrl+w", "alt+backspace"]) {
            let start = self.word_start();
            self.value.drain(start..self.index);
            self.index = start;
        } else if key.is("alt+d") {
            let end = self.word_end();
            self.value.drain(self.index..end);
        } else if key.is("left") {
            self.index = self.index.saturating_sub(1);
        } else if key.is("right") {
            self.index = (self.index + 1).min(self.value.len());
        } else if key.is("delete") {
            if self.index < self.value.len() {
                self.value.remove(self.index);
            }
        } else if key.is("backspace") {
            if self.index > 0 {
                self.index -= 1;
                self.value.remove(self.index);
            }
        } else if let Some(c) = key.char.filter(|_| !key.meta && !key.ctrl) {
            self.value.insert(self.index, c);
            self.index += 1;
        }
    }

    fn submit(&self) -> Result<(String, String), String> {
        let value: String = self.value.iter().collect();
        if self.secret {
            Ok((value, "*".repeat(self.value.len())))
        } else {
            let value = value.trim().to_string();
            Ok((value.clone(), value))
        }
    }
}

struct SelectPrompt<'a> {
    message: &'a str,
    options: &'a [(String, String)],
    index: usize,
    offset: usize,
}

impl<'a> SelectPrompt<'a> {
    fn new(message: &'a str, options: &'a [(String, String)]) -> Self {
        Self {
            message,
            options,
            index: 0,
            offset: 0,
        }
    }

    fn height(&self) -> usize {
        self.options.len().min(MAX_ROWS)
    }

    fn select_previous(&mut self) {
        if self.options.len() < 2 {
            return;
        }
        if self.index > 0 {
            self.index -= 1;
            if self.index < self.offset {
                self.offset -= 1;
            }
        } else {
            self.index = self.options.len() - 1;
            self.offset = self.options.len() - self.height();
        }
    }

    fn select_next(&mut self) {
        if self.options.len() < 2 {
            return;
        }
        if self.index < self.options.len() - 1 {
            self.index += 1;
            if self.index >= self.offset + self.height() {
                self.offset += 1;
            }
        } else {
            self.index = 0;
            self.offset = 0;
        }
    }

    fn select_previous_page(&mut self) {
        let height = self.height();
        if self.offset >= height {
            self.index -= height;
            self.offset -= height;
        } else if self.offset > 0 {
            self.index -= self.offset;
            self.offset = 0;
        } else {
            self.index = 0;
        }
    }

    fn select_next_page(&mut self) {
        let height = self.height();
        let len = self.options.len();
        if self.offset + 2 * height < len {
            self.index += height;
            self.offset += height;
        } else if self.offset + height < len {
            let offset = len - height;
            self.index += offset - self.offset;
            self.offset = offset;
        } else {
            self.index = len - 1;
        }
    }
}

impl Prompt for SelectPrompt<'_> {
    type Value = String;

    fn title(&self) -> &str {
        self.message
    }

    fn message(&self) -> (String, usize) {
        (header(self.message), 0)
    }

    fn body(&self) -> Option<String> {
        let lines: Vec<String> = (self.offset..self.offset + self.height())
            .map(|i| {
                let name = &self.options[i].0;
                if i == self.index {
                    format!("{} {name}", colors::bright_blue(POINTER))
                } else {
                    format!("  {}", colors::dim(name))
                }
            })
            .collect();
        Some(lines.join("\n"))
    }

    fn hides_cursor(&self) -> bool {
        true
    }

    fn handle(&mut self, key: &Key) {
        if key.is_any(&["down", "j", "d", "n", "2"]) {
            self.select_next();
        } else if key.is_any(&["up", "k", "u", "p", "8"]) {
            self.select_previous();
        } else if key.is_any(&["pagedown", "right", "l"]) {
            self.select_next_page();
        } else if key.is_any(&["pageup", "left", "h"]) {
            self.select_previous_page();
        }
    }

    fn submit(&self) -> Result<(String, String), String> {
        let (name, value) = &self.options[self.index];
        Ok((value.clone(), name.clone()))
    }
}

struct TogglePrompt<'a> {
    message: &'a str,
    status: Option<bool>,
}

impl<'a> TogglePrompt<'a> {
    fn new(message: &'a str) -> Self {
        Self {
            message,
            status: None,
        }
    }
}

impl Prompt for TogglePrompt<'_> {
    type Value = bool;

    fn title(&self) -> &str {
        self.message
    }

    fn message(&self) -> (String, usize) {
        let status = match self.status {
            Some(true) => colors::dim("No / ") + &colors::underline("Yes"),
            Some(false) => colors::underline("No") + &colors::dim(" / Yes"),
            None => colors::dim("No / Yes"),
        };
        (
            format!("{} {} {status}", header(self.message), pointer()),
            0,
        )
    }

    fn hides_cursor(&self) -> bool {
        true
    }

    fn handle(&mut self, key: &Key) {
        if key.is_any(&["left", "n"]) {
            self.status = Some(false);
        } else if key.is_any(&["right", "y", "j", "s", "o"]) {
            self.status = Some(true);
        }
    }

    fn submit(&self) -> Result<(bool, String), String> {
        match self.status {
            Some(value) => Ok((value, if value { "Yes" } else { "No" }.to_string())),
            None => Err("Invalid answer.".to_string()),
        }
    }
}
