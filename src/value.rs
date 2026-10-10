//! Values returned by the database connectors. They model the JavaScript
//! values produced by the Node.js drivers used by the Deno version of sqlr,
//! so results print the same way (`Deno.inspect`, `JSON.stringify` and
//! `String(value)`).

use crate::date;
use crate::inspect::Node;
use crate::json;

#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    /// JavaScript `Date`: milliseconds since the Unix epoch, NaN when invalid.
    Date(f64),
    /// Node.js `Buffer`.
    Buffer(Vec<u8>),
    /// `Uint8Array` (SQLite blobs).
    Uint8Array(Vec<u8>),
    Array(Vec<Cell>),
    Object(Object),
}

/// An object with JavaScript property order: integer-like keys first in
/// ascending order, other keys in insertion order. Assigning an existing key
/// keeps its position.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Object {
    /// Constructor name shown by `Deno.inspect` (e.g. `PostgresInterval`).
    pub class: Option<String>,
    pub entries: Vec<(String, Cell)>,
}

/// A result row: column name to value.
pub type Row = Object;

impl Object {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_class(class: &str) -> Self {
        Object {
            class: Some(class.to_string()),
            entries: Vec::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Cell> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn insert(&mut self, key: impl Into<String>, value: Cell) {
        let key = key.into();
        if let Some(slot) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            slot.1 = value;
            return;
        }
        let position = match array_index(&key) {
            Some(index) => self
                .entries
                .iter()
                .position(|(k, _)| array_index(k).is_none_or(|other| other > index))
                .unwrap_or(self.entries.len()),
            None => self.entries.len(),
        };
        self.entries.insert(position, (key, value));
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &Cell> {
        self.entries.iter().map(|(_, v)| v)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn array_index(key: &str) -> Option<u32> {
    if key.is_empty() || (key.len() > 1 && key.starts_with('0')) {
        return None;
    }
    key.parse::<u32>().ok().filter(|index| *index != u32::MAX)
}

impl From<&json::Value> for Cell {
    fn from(value: &json::Value) -> Self {
        match value {
            json::Value::Null => Cell::Null,
            json::Value::Bool(b) => Cell::Bool(*b),
            json::Value::Number(n) => Cell::Number(*n),
            json::Value::String(s) => Cell::String(s.clone()),
            json::Value::Array(items) => Cell::Array(items.iter().map(Cell::from).collect()),
            json::Value::Object(object) => {
                let mut result = Object::new();
                for (key, value) in object.iter() {
                    result.insert(key.clone(), Cell::from(value));
                }
                Cell::Object(result)
            }
        }
    }
}

/// `Deno.inspect` representation.
pub fn to_node(cell: &Cell) -> Node {
    match cell {
        Cell::Undefined => Node::Undefined,
        Cell::Null => Node::Null,
        Cell::Bool(b) => Node::Bool(*b),
        Cell::Number(n) => Node::Number(*n),
        Cell::String(s) => Node::String(s.clone()),
        Cell::Date(ms) => Node::Date(date::to_iso_string(*ms).unwrap_or("Invalid Date".into())),
        Cell::Buffer(bytes) => Node::Raw(inspect_buffer(bytes)),
        Cell::Uint8Array(bytes) => Node::TypedArray(
            format!("Uint8Array({})", bytes.len()),
            bytes.iter().map(|b| Node::Number(f64::from(*b))).collect(),
        ),
        Cell::Array(items) => Node::Array(items.iter().map(to_node).collect()),
        Cell::Object(object) => object_node(object),
    }
}

pub fn object_node(object: &Object) -> Node {
    Node::Object {
        class: object.class.clone(),
        entries: object
            .entries
            .iter()
            .map(|(k, v)| (k.clone(), to_node(v)))
            .collect(),
    }
}

/// Node.js `Buffer` inspect output: `<Buffer 01 02 ff>`.
fn inspect_buffer(bytes: &[u8]) -> String {
    const MAX_BYTES: usize = 50;
    let mut out = String::from("<Buffer");
    for byte in bytes.iter().take(MAX_BYTES) {
        out.push_str(&format!(" {byte:02x}"));
    }
    if bytes.len() > MAX_BYTES {
        out.push_str(&format!(" ... {} more bytes", bytes.len() - MAX_BYTES));
    }
    out.push('>');
    out
}

/// `String(value)`, e.g. in template literals.
pub fn to_js_string(cell: &Cell) -> String {
    match cell {
        Cell::Undefined => "undefined".to_string(),
        Cell::Null => "null".to_string(),
        other => to_display_string(other),
    }
}

/// `String(value)` with `Array.prototype.join` semantics for nested values:
/// `null` and `undefined` become empty strings. Used for `--compact` and
/// `--table`.
pub fn to_display_string(cell: &Cell) -> String {
    match cell {
        Cell::Undefined | Cell::Null => String::new(),
        Cell::Bool(b) => b.to_string(),
        Cell::Number(n) => json::format_number(*n),
        Cell::String(s) => s.clone(),
        Cell::Date(ms) => date::to_js_string(*ms),
        Cell::Buffer(bytes) => String::from_utf8_lossy(bytes).into_owned(),
        Cell::Uint8Array(bytes) => bytes
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(","),
        Cell::Array(items) => items
            .iter()
            .map(to_display_string)
            .collect::<Vec<_>>()
            .join(","),
        Cell::Object(_) => "[object Object]".to_string(),
    }
}

/// `JSON.stringify(rows, null, 2)`.
pub fn rows_to_json(rows: &[Row]) -> String {
    let cells = Cell::Array(rows.iter().cloned().map(Cell::Object).collect());
    let mut out = String::new();
    write_json(&mut out, &cells, 0);
    out
}

fn write_json(out: &mut String, cell: &Cell, level: usize) {
    match cell {
        Cell::Undefined | Cell::Null => out.push_str("null"),
        Cell::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Cell::Number(n) if n.is_finite() => out.push_str(&json::format_number(*n)),
        Cell::Number(_) => out.push_str("null"),
        Cell::String(s) => out.push_str(&json::stringify(&json::Value::String(s.clone()))),
        Cell::Date(ms) => match date::to_iso_string(*ms) {
            Some(iso) => out.push_str(&json::stringify(&json::Value::String(iso))),
            None => out.push_str("null"),
        },
        Cell::Buffer(bytes) => {
            let mut object = Object::new();
            object.insert("type", Cell::String("Buffer".into()));
            object.insert(
                "data",
                Cell::Array(bytes.iter().map(|b| Cell::Number(f64::from(*b))).collect()),
            );
            write_json(out, &Cell::Object(object), level);
        }
        Cell::Uint8Array(bytes) => {
            let mut object = Object::new();
            for (index, byte) in bytes.iter().enumerate() {
                object.insert(index.to_string(), Cell::Number(f64::from(*byte)));
            }
            write_json(out, &Cell::Object(object), level);
        }
        Cell::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                newline(out, level + 1);
                write_json(out, item, level + 1);
            }
            newline(out, level);
            out.push(']');
        }
        Cell::Object(object) => {
            if object.is_empty() {
                out.push_str("{}");
                return;
            }
            let entries: Vec<&(String, Cell)> = object
                .entries
                .iter()
                .filter(|(_, value)| *value != Cell::Undefined)
                .collect();
            if entries.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                newline(out, level + 1);
                out.push_str(&json::stringify(&json::Value::String(key.clone())));
                out.push_str(": ");
                write_json(out, value, level + 1);
            }
            newline(out, level);
            out.push('}');
        }
    }
}

fn newline(out: &mut String, level: usize) {
    out.push('\n');
    out.push_str(&"  ".repeat(level));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_keys_like_javascript() {
        let mut row = Object::new();
        row.insert("b", Cell::Number(1.0));
        row.insert("10", Cell::Number(2.0));
        row.insert("a", Cell::Number(3.0));
        row.insert("2", Cell::Number(4.0));
        row.insert("b", Cell::Number(5.0));
        let keys: Vec<&String> = row.keys().collect();
        assert_eq!(keys, ["2", "10", "b", "a"]);
        assert_eq!(row.get("b"), Some(&Cell::Number(5.0)));
    }

    #[test]
    fn stringifies_like_json_stringify() {
        let mut row = Object::new();
        row.insert("n", Cell::Null);
        row.insert("d", Cell::Date(0.0));
        row.insert("b", Cell::Buffer(vec![1, 255]));
        row.insert("u", Cell::Uint8Array(vec![]));
        row.insert("nan", Cell::Number(f64::NAN));
        row.insert("e", Cell::Array(vec![]));
        row.insert("skipped", Cell::Undefined);
        assert_eq!(
            rows_to_json(&[row]),
            "[\n  {\n    \"n\": null,\n    \"d\": \"1970-01-01T00:00:00.000Z\",\n    \"b\": {\n      \"type\": \"Buffer\",\n      \"data\": [\n        1,\n        255\n      ]\n    },\n    \"u\": {},\n    \"nan\": null,\n    \"e\": []\n  }\n]"
        );
        assert_eq!(rows_to_json(&[]), "[]");
    }

    #[test]
    fn joins_like_javascript() {
        let cell = Cell::Array(vec![
            Cell::Number(1.0),
            Cell::Null,
            Cell::Array(vec![Cell::String("a".into()), Cell::Bool(true)]),
        ]);
        assert_eq!(to_display_string(&cell), "1,,a,true");
        assert_eq!(
            to_display_string(&Cell::Object(Object::new())),
            "[object Object]"
        );
    }

    #[test]
    fn inspects_buffers() {
        assert_eq!(inspect_buffer(&[1, 2, 255]), "<Buffer 01 02 ff>");
        assert_eq!(inspect_buffer(&[]), "<Buffer>");
        assert!(inspect_buffer(&[0; 52]).ends_with(" 00 ... 2 more bytes>"));
    }
}
