//! Command line parsing, validation and help output compatible with the
//! parts of `@cliffy/command` sqlr uses: options with optional or required
//! values, enum types, conflicts, collected values, global options, simple
//! arguments and the generated help text.

use crate::colors::{bold, bright_blue, bright_magenta, green, italic, red, yellow};
use crate::logger;
use crate::table::Table;

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    /// Arguments definition, e.g. `[connection:string]` or `<enabled:boolean>`.
    pub arguments: &'static str,
    pub options: Vec<Opt>,
    /// Entries shown below the usage (`.meta(name, value)`).
    pub meta: Vec<(&'static str, &'static str)>,
    pub commands: Vec<Command>,
}

pub struct Opt {
    /// e.g. `-n, --name [name]`, `-i, --input-variable <var:string>`, `--table`.
    pub flags: &'static str,
    pub description: &'static str,
    /// Long names of options that can't be combined with this one.
    pub conflicts: &'static [&'static str],
    /// Collects repeated values into a list.
    pub collect: bool,
    /// Available on the command and all its subcommands.
    pub global: bool,
    /// Allowed values (enum type).
    pub values: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    String(String),
    List(Vec<String>),
}

#[derive(Debug, Default, PartialEq)]
pub struct Matches {
    /// Subcommand path, empty for the root command.
    pub command: Vec<String>,
    /// Options by long name.
    pub options: Vec<(String, Value)>,
    pub args: Vec<Value>,
}

impl Matches {
    fn get(&self, name: &str) -> Option<&Value> {
        self.options
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }

    pub fn flag(&self, name: &str) -> bool {
        matches!(self.get(name), Some(Value::Bool(true)))
    }

    /// The option value when one was given (not for a bare `-n`).
    pub fn string(&self, name: &str) -> Option<String> {
        match self.get(name) {
            Some(Value::String(value)) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn is_set(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn list(&self, name: &str) -> Vec<String> {
        match self.get(name) {
            Some(Value::List(values)) => values.clone(),
            Some(Value::String(value)) => vec![value.clone()],
            _ => Vec::new(),
        }
    }

    pub fn arg(&self, index: usize) -> Option<&Value> {
        self.args.get(index)
    }
}

pub enum Outcome {
    Run(Matches),
    Exit(i32),
}

#[derive(Clone, Copy, PartialEq)]
enum Arity {
    Flag,
    Optional,
    Required,
}

/// Parsed option definition.
struct Spec<'a> {
    short: Option<String>,
    long: String,
    arity: Arity,
    /// Value definition shown in help, e.g. `[type:ConnectorType]`.
    type_definition: String,
    type_name: String,
    opt: Option<&'a Opt>,
}

impl Spec<'_> {
    fn names(&self) -> Vec<String> {
        std::iter::once(format!("--{}", self.long))
            .chain(self.short.iter().map(|short| format!("-{short}")))
            .collect()
    }

    fn flag(&self) -> String {
        flag(&self.long)
    }
}

fn flag(name: &str) -> String {
    if name.len() > 1 {
        format!("--{name}")
    } else {
        format!("-{name}")
    }
}

fn spec(opt: &Opt) -> Spec<'_> {
    let mut short = None;
    let mut long = String::new();
    let mut type_definition = String::new();
    for part in opt.flags.split([',', ' ']).filter(|part| !part.is_empty()) {
        if let Some(name) = part.strip_prefix("--") {
            long = name.to_string();
        } else if let Some(name) = part.strip_prefix('-') {
            short = Some(name.to_string());
        } else {
            type_definition = part.to_string();
        }
    }
    let arity = if type_definition.starts_with('<') {
        Arity::Required
    } else if type_definition.starts_with('[') {
        Arity::Optional
    } else {
        Arity::Flag
    };
    let inner = type_definition.trim_matches(['<', '>', '[', ']']);
    let type_name = inner.split_once(':').map(|(_, t)| t).unwrap_or("string");
    Spec {
        short,
        long,
        arity,
        type_name: type_name.to_string(),
        type_definition,
        opt: Some(opt),
    }
}

fn builtin(short: &str, long: &str) -> Spec<'static> {
    Spec {
        short: Some(short.to_string()),
        long: long.to_string(),
        arity: Arity::Flag,
        type_definition: String::new(),
        type_name: String::new(),
        opt: None,
    }
}

/// Options of a command in help order: help, version (root only), global
/// options of the parents and the command's own options.
fn options_of<'a>(root: &'a Command, path: &[&'a Command]) -> Vec<Spec<'a>> {
    let mut specs = vec![builtin("h", "help")];
    if path.is_empty() {
        specs.push(builtin("V", "version"));
    }
    let command = path.last().copied().unwrap_or(root);
    let parents = std::iter::once(root).chain(path.iter().copied());
    for parent in parents.take(path.len()) {
        specs.extend(parent.options.iter().filter(|opt| opt.global).map(spec));
    }
    specs.extend(command.options.iter().map(spec));
    specs
}

/// Parses `args` like Cliffy's `command.parse()`. Help, version and
/// validation errors are printed here.
pub fn run(root: &Command, version: &str, args: &[String]) -> Outcome {
    let mut path: Vec<&Command> = Vec::new();
    let mut matches = Matches::default();
    let mut positional: Vec<String> = Vec::new();
    let mut rest = args.iter().peekable();
    let mut literal = false;

    while let Some(arg) = rest.next() {
        let command = path.last().copied().unwrap_or(root);
        if literal || !is_option(arg) {
            if !literal && positional.is_empty() && !command.commands.is_empty() {
                if let Some(sub) = command.commands.iter().find(|c| c.name == arg) {
                    path.push(sub);
                    matches.command.push(sub.name.to_string());
                    continue;
                }
            }
            positional.push(arg.clone());
            continue;
        }
        if arg == "--" {
            literal = true;
            continue;
        }

        let specs = options_of(root, &path);
        let (name, inline) = match arg.split_once('=') {
            Some((name, value)) => (name, Some(value.to_string())),
            None => (arg.as_str(), None),
        };
        let found = specs.iter().find(|spec| spec.names().iter().any(|n| n == name));
        let Some(spec) = found else {
            let names: Vec<String> = specs.iter().flat_map(Spec::names).collect();
            return fail(
                root,
                version,
                &path,
                format!(
                    "Unknown option \"{name}\".{}",
                    did_you_mean(" Did you mean option", name, &names)
                ),
            );
        };

        match spec.long.as_str() {
            "help" => {
                logger::print_line(&help_for(root, version, &path));
                return Outcome::Exit(0);
            }
            "version" => {
                if spec.short.as_deref() == Some(name.trim_start_matches('-')) {
                    logger::print_line(version);
                } else {
                    logger::print_line(&format!(
                        "{} {}",
                        bold(root.name),
                        bright_blue(version)
                    ));
                }
                return Outcome::Exit(0);
            }
            _ => {}
        }

        let value = match (spec.arity, inline) {
            (Arity::Flag, Some(value)) => {
                return fail(
                    root,
                    version,
                    &path,
                    format!(
                        "Option \"{}\" doesn't take a value, but got \"{value}\".",
                        spec.flag()
                    ),
                );
            }
            (Arity::Flag, None) => Value::Bool(true),
            (_, Some(value)) => Value::String(value),
            (arity, None) => match rest.peek() {
                Some(next) if !is_option(next) => Value::String(rest.next().cloned().unwrap_or_default()),
                _ if arity == Arity::Optional => Value::Bool(true),
                _ => {
                    return fail(
                        root,
                        version,
                        &path,
                        format!("Missing value for option \"{}\".", spec.flag()),
                    );
                }
            },
        };

        if let (Value::String(value), Some(values)) =
            (&value, spec.opt.and_then(|opt| opt.values.as_ref()))
            && !values.contains(value)
        {
            return fail(
                root,
                version,
                &path,
                format!(
                    "Option \"{}\" must be of type \"{}\", but got \"{value}\". Expected values: {}",
                    spec.flag(),
                    spec.type_name,
                    quoted_list(values)
                ),
            );
        }

        let collect = spec.opt.is_some_and(|opt| opt.collect);
        match matches.options.iter_mut().find(|(key, _)| *key == spec.long) {
            Some((_, Value::List(values))) if collect => {
                if let Value::String(value) = value {
                    values.push(value);
                }
            }
            Some(_) => {
                return fail(
                    root,
                    version,
                    &path,
                    format!(
                        "Option \"{}\" can only occur once, but was found several times.",
                        spec.flag()
                    ),
                );
            }
            None => {
                let value = match value {
                    Value::String(value) if collect => Value::List(vec![value]),
                    value => value,
                };
                matches.options.push((spec.long.clone(), value));
            }
        }
    }

    if let Err(message) = validate_conflicts(root, &path, &matches) {
        return fail(root, version, &path, message);
    }
    match parse_arguments(root, &path, positional) {
        Ok(args) => matches.args = args,
        Err(message) => return fail(root, version, &path, message),
    }
    Outcome::Run(matches)
}

fn is_option(arg: &str) -> bool {
    arg.starts_with('-') && arg.len() > 1 && arg.parse::<f64>().is_err()
}

fn validate_conflicts(root: &Command, path: &[&Command], matches: &Matches) -> Result<(), String> {
    for spec in options_of(root, path) {
        let Some(opt) = spec.opt else { continue };
        if !matches.is_set(&spec.long) {
            continue;
        }
        if let Some(conflict) = opt.conflicts.iter().find(|name| matches.is_set(name)) {
            return Err(format!(
                "Option \"{}\" conflicts with option \"{}\".",
                spec.flag(),
                flag(conflict)
            ));
        }
    }
    Ok(())
}

struct Argument {
    name: String,
    type_name: String,
    optional: bool,
}

fn parse_argument_definitions(definition: &str) -> Vec<Argument> {
    definition
        .split_whitespace()
        .map(|arg| {
            let inner = arg.trim_matches(['<', '>', '[', ']']);
            let (name, type_name) = inner.split_once(':').unwrap_or((inner, "string"));
            Argument {
                name: name.to_string(),
                type_name: type_name.to_string(),
                optional: arg.starts_with('['),
            }
        })
        .collect()
}

fn parse_arguments(
    root: &Command,
    path: &[&Command],
    positional: Vec<String>,
) -> Result<Vec<Value>, String> {
    let command = path.last().copied().unwrap_or(root);
    let definitions = parse_argument_definitions(command.arguments);
    if definitions.is_empty() {
        if !positional.is_empty() {
            return Err(format!(
                "No arguments allowed for command \"{}\".",
                command_path(root, path)
            ));
        }
        return Ok(Vec::new());
    }
    if positional.len() > definitions.len() {
        return Err(format!(
            "Too many arguments: {}",
            positional[definitions.len()..].join(" ")
        ));
    }
    let missing: Vec<&str> = definitions
        .iter()
        .skip(positional.len())
        .filter(|definition| !definition.optional)
        .map(|definition| definition.name.as_str())
        .collect();
    if !missing.is_empty() {
        return Err(format!("Missing argument(s): {}", missing.join(", ")));
    }
    definitions
        .iter()
        .zip(positional)
        .map(|(definition, value)| {
            if definition.type_name != "boolean" {
                return Ok(Value::String(value));
            }
            match value.as_str() {
                "true" | "1" => Ok(Value::Bool(true)),
                "false" | "0" => Ok(Value::Bool(false)),
                _ => Err(format!(
                    "Argument \"{}\" must be of type \"boolean\", but got \"{value}\". Expected values: \"true\", \"false\", \"1\", \"0\"",
                    definition.name
                )),
            }
        })
        .collect()
}

fn fail(root: &Command, version: &str, path: &[&Command], message: String) -> Outcome {
    logger::print_line(&help_for(root, version, path));
    eprintln!("{}", red(&format!("  {}: {message}\n", bold("error"))));
    Outcome::Exit(2)
}

fn quoted_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// ` Did you mean option "--table"?` for the closest name (Levenshtein
/// distance, case insensitive, first one wins on ties).
fn did_you_mean(message: &str, given: &str, candidates: &[String]) -> String {
    let given = given.to_lowercase();
    candidates
        .iter()
        .min_by_key(|candidate| levenshtein(&given, &candidate.to_lowercase()))
        .map(|closest| format!("{message} \"{closest}\"?"))
        .unwrap_or_default()
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            current.push((previous[j] + cost).min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}

fn command_path(root: &Command, path: &[&Command]) -> String {
    std::iter::once(root.name)
        .chain(path.iter().map(|command| command.name))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Help text of the command at `path` (subcommand names).
pub fn help(root: &Command, version: &str, path: &[&str]) -> String {
    let mut commands: Vec<&Command> = Vec::new();
    for name in path {
        let parent = commands.last().copied().unwrap_or(root);
        match parent.commands.iter().find(|command| command.name == *name) {
            Some(command) => commands.push(command),
            None => break,
        }
    }
    help_for(root, version, &commands)
}

/// Terminal width as Cliffy reads it (150 when not a terminal).
fn columns() -> usize {
    for fd in [libc::STDOUT_FILENO, libc::STDERR_FILENO, libc::STDIN_FILENO] {
        // SAFETY: TIOCGWINSZ fills a winsize struct.
        unsafe {
            let mut size: libc::winsize = std::mem::zeroed();
            if libc::ioctl(fd, libc::TIOCGWINSZ, &mut size) == 0 && size.ws_col > 0 {
                return usize::from(size.ws_col);
            }
        }
    }
    150
}

const INDENT: usize = 2;

fn help_for(root: &Command, version: &str, path: &[&Command]) -> String {
    let width = columns();
    let command = path.last().copied().unwrap_or(root);
    let mut out = String::new();

    let usage = highlight_arguments(command.arguments);
    let usage = if usage.is_empty() {
        command_path(root, path)
    } else {
        format!("{} {usage}", command_path(root, path))
    };
    out.push('\n');
    out.push_str(
        &Table::from_rows(vec![
            vec![bold("Usage:"), bright_magenta(&usage)],
            vec![bold("Version:"), yellow(version)],
        ])
        .padding(1)
        .to_string(),
    );
    out.push('\n');

    if !command.meta.is_empty() {
        let rows = command
            .meta
            .iter()
            .map(|(name, value)| vec![format!("{}{value}", bold(&format!("{name}: ")))])
            .collect();
        out.push('\n');
        out.push_str(&Table::from_rows(rows).padding(1).to_string());
        out.push('\n');
    }

    if !command.description.is_empty() {
        out.push_str(&label("Description"));
        out.push_str(
            &Table::from_rows(vec![vec![command.description.to_string()]])
                .indent(INDENT)
                .max_col_width(width - INDENT)
                .padding(1)
                .to_string(),
        );
        out.push('\n');
    }

    let specs = options_of(root, path);
    if !specs.is_empty() {
        out.push_str(&label("Options"));
        let has_types = specs.iter().any(|spec| !spec.type_definition.is_empty());
        let rows = specs
            .iter()
            .map(|spec| {
                let flags = spec_flags(spec)
                    .iter()
                    .map(|flag| bright_blue(flag))
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut row = vec![flags];
                if has_types {
                    row.push(highlight_arguments(&spec.type_definition));
                }
                row.push(red(&bold("-")));
                row.push(spec_description(spec).to_string());
                row.push(option_hints(spec));
                row
            })
            .collect();
        let table = Table::from_rows(rows)
            .indent(INDENT)
            .max_width(width - INDENT);
        let table = if has_types {
            table.padding_each(&[2, 2, 1, 2]).flex_shrink_each(&[0.0, 0.0, 0.0, 1.0, 1.0])
        } else {
            table.padding_each(&[2, 1, 2]).flex_shrink_each(&[0.0, 0.0, 1.0, 1.0])
        };
        out.push_str(&table.to_string());
        out.push('\n');
    }

    if !command.commands.is_empty() {
        out.push_str(&label("Commands"));
        let has_arguments = command.commands.iter().any(|c| !c.arguments.is_empty());
        let rows = command
            .commands
            .iter()
            .map(|sub| {
                let mut row = vec![bright_blue(sub.name)];
                if has_arguments {
                    row.push(highlight_arguments(sub.arguments));
                }
                row.push(red(&bold("-")));
                row.push(sub.description.lines().next().unwrap_or_default().to_string());
                row
            })
            .collect();
        let table = Table::from_rows(rows)
            .indent(INDENT)
            .max_width(width - INDENT);
        let table = if has_arguments {
            table.padding_each(&[2, 2, 1, 2]).flex_shrink_each(&[0.0, 0.0, 0.0, 1.0])
        } else {
            table.padding_each(&[2, 1, 2]).flex_shrink_each(&[0.0, 0.0, 1.0])
        };
        out.push_str(&table.to_string());
        out.push('\n');
    }

    out
}

fn label(label: &str) -> String {
    format!("\n{}\n\n", bold(&format!("{label}:")))
}

fn spec_flags(spec: &Spec) -> Vec<String> {
    let mut flags: Vec<String> = spec.short.iter().map(|short| format!("-{short}")).collect();
    flags.push(format!("--{}", spec.long));
    flags
}

fn spec_description(spec: &Spec) -> &'static str {
    match (spec.opt, spec.long.as_str()) {
        (Some(opt), _) => opt.description,
        (None, "help") => "Show this help.",
        (None, _) => "Show the version number for this program.",
    }
}

fn option_hints(spec: &Spec) -> String {
    let Some(opt) = spec.opt else {
        return String::new();
    };
    let mut hints = Vec::new();
    if !opt.conflicts.is_empty() {
        let conflicts = opt
            .conflicts
            .iter()
            .map(|name| flag(name))
            .collect::<Vec<_>>()
            .join(", ");
        hints.push(format!("{}{}", red(&bold("Conflicts: ")), italic(&conflicts)));
    }
    if let Some(values) = &opt.values {
        let values = values
            .iter()
            .map(|value| green(&format!("\"{value}\"")))
            .collect::<Vec<_>>()
            .join(", ");
        hints.push(format!("{}{values}", bold("Values: ")));
    }
    if hints.is_empty() {
        String::new()
    } else {
        format!("({})", hints.join(", "))
    }
}

/// Colors an arguments definition and hides the types:
/// `[type:ConnectorType]` becomes `[type]`.
fn highlight_arguments(definition: &str) -> String {
    parse_argument_definitions(definition)
        .iter()
        .map(|arg| {
            let (open, close) = if arg.optional { ("[", "]") } else { ("<", ">") };
            format!(
                "{}{}{}",
                yellow(open),
                bright_magenta(&arg.name),
                yellow(close)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt(flags: &'static str) -> Opt {
        Opt {
            flags,
            description: "",
            conflicts: &[],
            collect: false,
            global: false,
            values: None,
        }
    }

    fn tree() -> Command {
        Command {
            name: "sqlr",
            description: "",
            arguments: "",
            options: vec![Opt {
                global: true,
                ..opt("-g, --global")
            }],
            meta: vec![],
            commands: vec![
                Command {
                    name: "query",
                    description: "",
                    arguments: "[queryArg:string]",
                    options: vec![
                        Opt {
                            collect: true,
                            ..opt("-i, --input-variable <var:string>")
                        },
                        Opt {
                            conflicts: &["type"],
                            ..opt("-n, --name [name]")
                        },
                        Opt {
                            values: Some(vec!["mysql".into(), "sqlite".into()]),
                            ..opt("-t, --type [type:ConnectorType]")
                        },
                    ],
                    meta: vec![],
                    commands: vec![],
                },
                Command {
                    name: "set-global",
                    description: "",
                    arguments: "<enabled:boolean>",
                    options: vec![],
                    meta: vec![],
                    commands: vec![],
                },
                Command {
                    name: "ls",
                    description: "",
                    arguments: "",
                    options: vec![],
                    meta: vec![],
                    commands: vec![],
                },
            ],
        }
    }

    fn parse(args: &[&str]) -> Result<Matches, Vec<String>> {
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        let root = tree();
        match run(&root, "1.0.0", &args) {
            Outcome::Run(matches) => Ok(matches),
            Outcome::Exit(code) => Err(vec![code.to_string()]),
        }
    }

    #[test]
    fn parses_options_and_arguments() {
        let matches = parse(&["-g", "query", "-i", "a: 1", "--input-variable=b: 2", "-n", "SELECT 1"]).unwrap();
        assert_eq!(matches.command, ["query"]);
        assert!(matches.flag("global"));
        assert_eq!(matches.list("input-variable"), ["a: 1", "b: 2"]);
        assert_eq!(matches.string("name").as_deref(), Some("SELECT 1"));
        assert_eq!(matches.arg(0), None);

        let matches = parse(&["query", "-n", "-t", "mysql", "SELECT -1"]);
        assert!(matches.is_err(), "name conflicts with type");

        let matches = parse(&["query", "-t", "mysql", "--", "-1"]).unwrap();
        assert_eq!(matches.arg(0), Some(&Value::String("-1".into())));

        let matches = parse(&["set-global", "0"]).unwrap();
        assert_eq!(matches.arg(0), Some(&Value::Bool(false)));
    }

    #[test]
    fn rejects_invalid_input() {
        for args in [
            &["query", "--foo"][..],
            &["query", "-t", "oracle"],
            &["query", "-i"],
            &["query", "a", "b"],
            &["ls", "extra"],
            &["set-global"],
            &["set-global", "maybe"],
            &["query", "-t", "mysql", "-t", "sqlite"],
            &["ls", "-g=1"],
        ] {
            assert!(parse(args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn suggests_the_closest_option() {
        let names: Vec<String> = ["--help", "-h", "--table", "--compact"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            did_you_mean(" Did you mean option", "--tabel", &names),
            " Did you mean option \"--table\"?"
        );
        assert_eq!(
            did_you_mean(" Did you mean option", "--foo", &names),
            " Did you mean option \"--help\"?"
        );
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }
}
