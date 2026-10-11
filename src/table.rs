//! Text tables: port of the `@cliffy/table` layout (word wrapping, column
//! widths, padding, borders and the flex shrink used by help output). Cells
//! may contain ANSI colors. Row and column spans are not supported.

use crate::colors::strip_ansi;
use crate::unicode_width;

const TOP: &str = "─";
const TOP_MID: &str = "┬";
const TOP_LEFT: &str = "┌";
const TOP_RIGHT: &str = "┐";
const BOTTOM: &str = "─";
const BOTTOM_MID: &str = "┴";
const BOTTOM_LEFT: &str = "└";
const BOTTOM_RIGHT: &str = "┘";
const LEFT: &str = "│";
const LEFT_MID: &str = "├";
const MID: &str = "─";
const MID_MID: &str = "┼";
const RIGHT: &str = "│";
const RIGHT_MID: &str = "┤";
const MIDDLE: &str = "│";

/// A setting for all columns or per column (missing columns use `None`).
#[derive(Clone, Debug)]
enum PerColumn<T> {
    All(T),
    Each(Vec<T>),
}

impl<T: Copy> PerColumn<T> {
    fn get(&self, index: usize) -> Option<T> {
        match self {
            PerColumn::All(value) => Some(*value),
            PerColumn::Each(values) => values.get(index).copied(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Table {
    header: Option<Vec<String>>,
    rows: Vec<Vec<String>>,
    indent: usize,
    border: bool,
    max_col_width: PerColumn<usize>,
    min_col_width: PerColumn<usize>,
    padding: PerColumn<usize>,
    max_width: Option<usize>,
    flex_shrink: PerColumn<f64>,
}

impl Default for Table {
    fn default() -> Self {
        Table {
            header: None,
            rows: Vec::new(),
            indent: 0,
            border: false,
            max_col_width: PerColumn::All(usize::MAX),
            min_col_width: PerColumn::All(0),
            padding: PerColumn::All(1),
            max_width: None,
            flex_shrink: PerColumn::All(0.0),
        }
    }
}

struct Cell {
    value: String,
    unclosed_ansi_runs: String,
}

struct Layout {
    padding: Vec<usize>,
    width: Vec<usize>,
    columns: usize,
    rows: Vec<Vec<Cell>>,
}

impl Table {
    pub fn new() -> Self {
        Self::default()
    }

    /// `Table.from(rows)`
    pub fn from_rows(rows: Vec<Vec<String>>) -> Self {
        Table {
            rows,
            ..Self::default()
        }
    }

    pub fn header(mut self, header: Vec<String>) -> Self {
        self.header = Some(header);
        self
    }

    pub fn body(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn max_col_width(mut self, width: usize) -> Self {
        self.max_col_width = PerColumn::All(width);
        self
    }

    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = PerColumn::All(padding);
        self
    }

    pub fn padding_each(mut self, padding: &[usize]) -> Self {
        self.padding = PerColumn::Each(padding.to_vec());
        self
    }

    pub fn max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn flex_shrink_each(mut self, weights: &[f64]) -> Self {
        self.flex_shrink = PerColumn::Each(weights.to_vec());
        self
    }

    fn layout(&self) -> Layout {
        let mut rows: Vec<Vec<Cell>> = self
            .header
            .iter()
            .chain(self.rows.iter())
            .map(|row| {
                row.iter()
                    .map(|value| Cell {
                        value: value.clone(),
                        unclosed_ansi_runs: String::new(),
                    })
                    .collect()
            })
            .collect();
        let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
        for row in &mut rows {
            while row.len() < columns {
                row.push(Cell {
                    value: String::new(),
                    unclosed_ansi_runs: String::new(),
                });
            }
        }

        let mut padding = Vec::with_capacity(columns);
        let mut width = Vec::with_capacity(columns);
        let mut min_col_width = Vec::with_capacity(columns);
        for index in 0..columns {
            let min = self.min_col_width.get(index).unwrap_or(0);
            let max = self.max_col_width.get(index).unwrap_or(usize::MAX);
            let longest = longest(index, &rows, max);
            min_col_width.push(min);
            width.push(max.min(min.max(longest)));
            padding.push(self.padding.get(index).unwrap_or(0));
        }

        if let Some(max_width) = self.max_width {
            let total_padding = if self.border {
                padding.iter().sum::<usize>() * 2 + columns + 1
            } else {
                padding[..columns.saturating_sub(1)].iter().sum::<usize>()
            };
            let max_allowable = max_width as i64 - total_padding as i64;
            if (width.iter().sum::<usize>() as i64) > max_allowable {
                self.apply_flex_shrink(&mut width, &min_col_width, max_allowable);
            }
        }

        Layout {
            padding,
            width,
            columns,
            rows,
        }
    }

    fn apply_flex_shrink(&self, width: &mut [usize], min_col_width: &[usize], max_allowable: i64) {
        let shrink: Vec<f64> = (0..width.len())
            .map(|i| {
                self.flex_shrink
                    .get(i)
                    .filter(|w| w.is_finite() && *w > 0.0)
                    .unwrap_or(0.0)
            })
            .collect();
        loop {
            let total = width.iter().sum::<usize>() as i64;
            if total <= max_allowable {
                break;
            }
            let overflow = (total - max_allowable) as f64;
            let scaled_total: f64 = width
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    if shrink[i] > 0.0 && *w > min_col_width[i] {
                        shrink[i] * *w as f64
                    } else {
                        0.0
                    }
                })
                .sum();
            if scaled_total == 0.0 {
                break;
            }
            let mut changed = false;
            for i in 0..width.len() {
                if shrink[i] > 0.0 && width[i] > min_col_width[i] {
                    let shrunk = (width[i] as f64 * (1.0 - overflow * shrink[i] / scaled_total))
                        .floor();
                    let target = (min_col_width[i].max(1) as f64).max(shrunk) as usize;
                    if target != width[i] {
                        width[i] = target;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    fn render_rows(&self, layout: &mut Layout) -> String {
        let mut result = String::new();
        for row_index in 0..layout.rows.len() {
            result.push_str(&self.render_row(row_index, layout, false));
        }
        result.pop();
        result
    }

    fn render_row(&self, row_index: usize, layout: &mut Layout, is_multiline: bool) -> String {
        let mut result = String::new();
        if !is_multiline && row_index == 0 && self.border {
            result.push_str(&self.render_border_row(None, Some(row_index), layout));
        }

        result.push_str(&" ".repeat(self.indent));
        let mut is_multiline_row = false;
        for col_index in 0..layout.columns {
            result.push_str(&self.render_cell(col_index, row_index, layout));
            if !layout.rows[row_index][col_index].value.is_empty() {
                is_multiline_row = true;
            }
        }
        if layout.columns > 0 && self.border {
            result.push_str(RIGHT);
        }
        result.push('\n');

        if is_multiline_row {
            return result + &self.render_row(row_index, layout, true);
        }

        let row_count = layout.rows.len();
        if self.border && row_count > 1 && row_index < row_count - 1 {
            result.push_str(&self.render_border_row(Some(row_index), Some(row_index + 1), layout));
        }
        if self.border && row_index == row_count - 1 {
            result.push_str(&self.render_border_row(Some(row_index), None, layout));
        }
        result
    }

    fn render_cell(&self, col_index: usize, row_index: usize, layout: &mut Layout) -> String {
        let mut result = String::new();
        if self.border {
            result.push_str(if col_index == 0 { LEFT } else { MIDDLE });
        }

        let max_length = layout.width[col_index];
        let padding = layout.padding[col_index];
        let current = render_cell_value(&mut layout.rows[row_index][col_index], max_length);

        if self.border {
            result.push_str(&" ".repeat(padding));
        }
        result.push_str(&current);
        if self.border || col_index < layout.columns - 1 {
            result.push_str(&" ".repeat(padding));
        }
        result
    }

    fn render_border_row(&self, prev: Option<usize>, next: Option<usize>, layout: &Layout) -> String {
        let mut result = String::new();
        let has_prev = prev.is_some();
        let has_next = next.is_some();
        for col_index in 0..layout.columns {
            if col_index == 0 {
                result.push_str(match (has_prev, has_next) {
                    (true, true) => LEFT_MID,
                    (true, false) => BOTTOM_LEFT,
                    (false, true) => TOP_LEFT,
                    (false, false) => " ",
                });
            } else {
                result.push_str(match (has_prev, has_next) {
                    (true, true) => MID_MID,
                    (true, false) => BOTTOM_MID,
                    (false, true) => TOP_MID,
                    (false, false) => " ",
                });
            }
            let length = layout.padding[col_index] * 2 + layout.width[col_index];
            let line = match (has_prev, has_next) {
                (true, true) => MID,
                (true, false) => BOTTOM,
                (false, true) => TOP,
                (false, false) => " ",
            };
            result.push_str(&line.repeat(length));
            if col_index == layout.columns - 1 {
                result.push_str(match (has_prev, has_next) {
                    (true, true) => RIGHT_MID,
                    (true, false) => BOTTOM_RIGHT,
                    (false, true) => TOP_RIGHT,
                    (false, false) => " ",
                });
            }
        }
        if result.is_empty() {
            return result;
        }
        format!("{}{result}\n", " ".repeat(self.indent))
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut layout = self.layout();
        if layout.rows.is_empty() {
            return Ok(());
        }
        f.write_str(&self.render_rows(&mut layout))
    }
}

/// Renders the part of the cell that fits in `max_length` and keeps the rest
/// in the cell for the next line.
fn render_cell_value(cell: &mut Cell, max_length: usize) -> String {
    if max_length == 0 {
        cell.value.clear();
        return String::new();
    }
    let value = std::mem::take(&mut cell.value);
    let length = max_length.min(str_length(&value));
    let mut words = consume_words(length, &value);
    let break_word = str_length(&words) > length;
    if break_word {
        words = consume_chars(length, &words);
    }
    let skip = words.len() + usize::from(!break_word);
    cell.value = value.get(skip..).unwrap_or_default().to_string();

    let mut words = format!("{}{words}", cell.unclosed_ansi_runs);
    let (current_suffix, next_prefix) = unclosed_ansi_runs(&words);
    words.push_str(&current_suffix);
    cell.unclosed_ansi_runs = next_prefix;

    let fill = max_length.saturating_sub(str_length(&words));
    words + &" ".repeat(fill)
}

pub fn str_length(s: &str) -> usize {
    unicode_width::width(&strip_ansi(s))
}

/// Widest line of a column, considering only the words that fit in
/// `max_width`.
fn longest(index: usize, rows: &[Vec<Cell>], max_width: usize) -> usize {
    rows.iter()
        .flat_map(|row| row[index].value.split('\n'))
        .map(|line| str_length(&consume_words(max_width, line)))
        .max()
        .unwrap_or(0)
}

/// The longest prefix of whole words of the first line not longer than
/// `length` (at least one word).
pub fn consume_words(length: usize, content: &str) -> String {
    let mut consumed = String::new();
    let first_line = content.split('\n').next().unwrap_or_default();
    for (i, word) in first_line.split(' ').enumerate() {
        if !consumed.is_empty() && str_length(&consumed) + str_length(word) >= length {
            break;
        }
        if i > 0 {
            consumed.push(' ');
        }
        consumed.push_str(word);
    }
    consumed
}

/// The longest prefix of the first line not longer than `length` (at least
/// one character), keeping ANSI codes attached to the following character.
fn consume_chars(length: usize, content: &str) -> String {
    let first_line = content.split('\n').next().unwrap_or_default();
    let mut consumed = String::new();
    for token in ansi_tokens(first_line) {
        if !consumed.is_empty() && str_length(&consumed) + str_length(token) > length {
            break;
        }
        consumed.push_str(token);
    }
    consumed
}

/// Splits into runs of ANSI codes and single characters.
fn ansi_tokens(s: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < s.len() {
        let mut end = index;
        while let Some(code_end) = sgr_end(s, end) {
            end = code_end;
        }
        if end == index {
            end = index + s[index..].chars().next().map_or(1, char::len_utf8);
        }
        tokens.push(&s[index..end]);
        index = end;
    }
    tokens
}

/// End of an SGR sequence (`ESC [ params m`) starting at `start`.
fn sgr_end(s: &str, start: usize) -> Option<usize> {
    let rest = s.get(start..)?.strip_prefix("\x1b[")?;
    let params = rest.find('m')?;
    rest[..params]
        .chars()
        .all(|c| c.is_ascii_digit() || c == ';')
        .then_some(start + 2 + params + 1)
}

/// The kind of style an SGR code belongs to (the code that closes it).
fn sgr_kind(params: &str) -> Option<&'static str> {
    let first: u32 = params.split(';').next()?.parse().ok()?;
    match first {
        0 if params == "0" => Some("0"),
        1 | 2 | 22 => Some("22"),
        3 | 23 => Some("23"),
        4 | 24 => Some("24"),
        7 | 27 => Some("27"),
        8 | 28 => Some("28"),
        9 | 29 => Some("29"),
        30..=39 | 90..=97 => Some("39"),
        40..=49 | 100..=107 => Some("49"),
        _ => None,
    }
}

/// Suffix closing the ANSI styles left open in `text`, and the prefix that
/// re-opens them on the next line.
fn unclosed_ansi_runs(text: &str) -> (String, String) {
    let mut unclosed: Vec<(&'static str, String)> = Vec::new();
    let mut index = 0;
    while let Some(offset) = text[index..].find("\x1b[") {
        let start = index + offset;
        match sgr_end(text, start) {
            Some(end) => {
                let params = &text[start + 2..end - 1];
                if let Some(kind) = sgr_kind(params) {
                    unclosed.retain(|(k, _)| *k != kind);
                    unclosed.push((kind, params.to_string()));
                }
                index = end;
            }
            None => index = start + 2,
        }
    }
    unclosed.retain(|(kind, content)| content != kind);
    let suffix = unclosed
        .iter()
        .rev()
        .map(|(kind, _)| format!("\x1b[{kind}m"))
        .collect();
    let prefix = unclosed
        .iter()
        .map(|(_, content)| format!("\x1b[{content}m"))
        .collect();
    (suffix, prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(row: &[&str]) -> Vec<String> {
        row.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn renders_bordered_tables() {
        let table = Table::new()
            .header(strings(&["code", "name"]))
            .body(vec![strings(&["ABW", "Aruba"]), strings(&["AFG", "Afghanistan"])])
            .max_col_width(80)
            .padding(1)
            .indent(2)
            .border(true)
            .to_string();
        assert_eq!(
            table,
            "  ┌──────┬─────────────┐
  │ code │ name        │
  ├──────┼─────────────┤
  │ ABW  │ Aruba       │
  ├──────┼─────────────┤
  │ AFG  │ Afghanistan │
  └──────┴─────────────┘"
        );
    }

    #[test]
    fn renders_empty_tables() {
        let table = Table::new()
            .header(strings(&["Name", "Type"]))
            .body(vec![])
            .max_col_width(25)
            .padding(1)
            .indent(2)
            .border(true)
            .to_string();
        assert_eq!(
            table,
            "  ┌──────┬──────┐
  │ Name │ Type │
  └──────┴──────┘"
        );
        assert_eq!(Table::new().to_string(), "");
    }

    #[test]
    fn wraps_words_and_breaks_long_words() {
        let table = Table::new()
            .header(strings(&["Name", "Type", "Connection string"]))
            .body(vec![strings(&[
                "enc-mssql",
                "mssql",
                "mssql://sa:World123!;@localhost:1433/db?encrypt=true&trustServerCertificate=false",
            ])])
            .max_col_width(80)
            .padding(1)
            .indent(2)
            .border(true)
            .to_string();
        assert_eq!(
            table,
            "  ┌───────────┬───────┬──────────────────────────────────────────────────────────────────────────────────┐
  │ Name      │ Type  │ Connection string                                                                │
  ├───────────┼───────┼──────────────────────────────────────────────────────────────────────────────────┤
  │ enc-mssql │ mssql │ mssql://sa:World123!;@localhost:1433/db?encrypt=true&trustServerCertificate=fals │
  │           │       │ e                                                                                │
  └───────────┴───────┴──────────────────────────────────────────────────────────────────────────────────┘"
        );

        let table = Table::new()
            .header(strings(&["Type", "Connection string hint"]))
            .body(vec![strings(&[
                "postgresql",
                "postgres://host:port/database_name?user=user&password=password(urlencoded)&application_name=sqlr\nAdditional url parameters:\n'options' - additional values for connection (options=--cluster=your_cluster_name)",
            ])])
            .max_col_width(80)
            .padding(1)
            .indent(2)
            .border(true)
            .to_string();
        assert_eq!(
            table,
            "  ┌────────────┬──────────────────────────────────────────────────────────────────────────────────┐
  │ Type       │ Connection string hint                                                           │
  ├────────────┼──────────────────────────────────────────────────────────────────────────────────┤
  │ postgresql │ postgres://host:port/database_name?user=user&password=password(urlencoded)&appli │
  │            │ cation_name=sqlr                                                                 │
  │            │ Additional url parameters:                                                       │
  │            │ 'options' - additional values for connection                                     │
  │            │ (options=--cluster=your_cluster_name)                                            │
  └────────────┴──────────────────────────────────────────────────────────────────────────────────┘"
        );
    }

    #[test]
    fn renders_borderless_tables() {
        let table = Table::from_rows(vec![
            strings(&["Usage:", "sqlr"]),
            strings(&["Version:", "1.0.0"]),
        ])
        .padding(1)
        .to_string();
        assert_eq!(table, "Usage:   sqlr \nVersion: 1.0.0");
    }

    #[test]
    fn ignores_colors_in_widths_and_carries_them_across_lines() {
        let table = Table::new()
            .header(vec![crate::colors::bright_green("id")])
            .body(vec![strings(&["1"])])
            .padding(1)
            .border(true)
            .to_string();
        assert_eq!(
            strip_ansi(&table),
            "┌────┐\n│ id │\n├────┤\n│ 1  │\n└────┘"
        );
        assert_eq!(
            unclosed_ansi_runs("\x1b[31mred \x1b[1mbold"),
            ("\x1b[22m\x1b[39m".to_string(), "\x1b[31m\x1b[1m".to_string())
        );
        assert_eq!(unclosed_ansi_runs("\x1b[31mred\x1b[39m"), (String::new(), String::new()));
    }

    #[test]
    fn measures_wide_characters() {
        assert_eq!(str_length("\x1b[38;2;255;0;0m██\x1b[39m Red"), 6);
        assert_eq!(str_length("\u{1F680} red-db"), 9);
        assert_eq!(consume_words(9, "This is an example."), "This is");
        assert_eq!(consume_chars(3, "\x1b[31mabcdef"), "\x1b[31mabc");
    }
}
