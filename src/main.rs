mod cli;
mod colors;
mod commands;
mod connection;
mod connectors;
mod date;
mod guard;
mod inspect;
mod json;
mod logger;
mod prompt;
mod scope;
mod storage;
mod table;
mod unicode_width;
mod update;
mod url;
mod value;

use cli::{Command, Matches, Opt, Outcome, Value};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn opt(flags: &'static str, description: &'static str) -> Opt {
    Opt {
        flags,
        description,
        conflicts: &[],
        collect: false,
        global: false,
        values: None,
    }
}

fn command(name: &'static str, description: &'static str) -> Command {
    Command {
        name,
        description,
        arguments: "",
        options: Vec::new(),
        meta: Vec::new(),
        commands: Vec::new(),
    }
}

fn connector_types() -> Option<Vec<String>> {
    Some(connectors::keys().into_iter().map(String::from).collect())
}

fn name_option() -> Opt {
    opt("-n, --name [name]", "Name of the connection")
}

fn connection_options() -> Vec<Opt> {
    vec![
        Opt {
            conflicts: &["type", "connection-string"],
            ..name_option()
        },
        Opt {
            values: connector_types(),
            ..opt("-t, --type [type:ConnectorType]", "Type of the connection")
        },
        opt(
            "-s, --connection-string [connection-string]",
            "Connection string",
        ),
    ]
}

fn named_command(name: &'static str, description: &'static str) -> Command {
    Command {
        arguments: "[connection:string]",
        options: vec![name_option()],
        ..command(name, description)
    }
}

fn root() -> Command {
    let add = Command {
        options: vec![
            name_option(),
            Opt {
                values: connector_types(),
                ..opt("-t, --type [type:ConnectorType]", "Type of the connection")
            },
            opt(
                "-s, --connection-string [connection-string]",
                "Connection string",
            ),
            Opt {
                values: Some(connection::color_values()),
                ..opt("-c, --color [color:ConnectionColor]", "Connection color")
            },
            opt("-e, --emoji [emoji]", "Connection emoji"),
        ],
        meta: vec![(
            "Connection Types",
            "Available types and connection string hints can be found using 'ls-types' command",
        )],
        ..command(
            "add",
            "Add new connection. Run without parameters to use wizard.",
        )
    };

    let mut describe_options = connection_options();
    describe_options.extend([
        opt(
            "-f, --filter [filter]",
            "Filter results by schema/table name",
        ),
        opt("--table", "Display results in table"),
        opt("--compact", "Display results in compact form"),
        opt("--tables-only", "List only table names without columns"),
    ]);
    let describe = Command {
        options: describe_options,
        ..command(
            "describe",
            "Describe all tables and columns available in database",
        )
    };

    let mut query_options = vec![
        Opt {
            collect: true,
            ..opt(
                "-i, --input-variable <var:string>",
                "Input variable in \"key: value\" format for {{key}} substitution in SQL",
            )
        },
        opt(
            "-o, --output-file [output-file]",
            "Output file where JSON results will be stored",
        ),
    ];
    query_options.extend(connection_options());
    query_options.extend([
        Opt {
            conflicts: &["compact"],
            ..opt("--table", "Display results as table")
        },
        Opt {
            conflicts: &["table"],
            ..opt("--compact", "Display results in compact form")
        },
        opt(
            "--ignore-input-validation",
            "Skip validation for missing input variables, allowing {{handlebars}} syntax to pass through to the database",
        ),
    ]);
    let query = Command {
        arguments: "[queryArg:string]",
        options: query_options,
        ..command("query", "Run query against specified database")
    };

    let skill = Command {
        options: vec![
            opt(
                "--init",
                "Create the skill in the current git repository, or in the home directory outside one",
            ),
            opt(
                "-n, --name <name:string>",
                "Pin the created skill to a connection (creates /sqlr-<name>)",
            ),
        ],
        ..command(
            "skill",
            "Print instructions for AI agents. Use --init to create a skill (/sqlr)",
        )
    };

    Command {
        options: vec![
            Opt {
                global: true,
                ..opt("--debug", "Enable debug logs")
            },
            Opt {
                global: true,
                ..opt(
                    "-g, --global",
                    "Use global scope for connections (ignore git repository scope)",
                )
            },
        ],
        commands: vec![
            add,
            named_command("rm", "Remove selected connection"),
            named_command("get", "Get details of selected connection"),
            command(
                "ls-types",
                "Get available connection types to be used with '--type' parameter in 'add' command",
            ),
            command("ls", "List all defined connections"),
            command("clear", "Remove all connections"),
            named_command("set", "Set default connection for current scope"),
            Command {
                arguments: "<enabled:boolean>",
                ..command(
                    "set-global",
                    "Toggle global mode (true = ignore git repository scope, false = use repository scope)",
                )
            },
            describe,
            query,
            command("update", "Update sqlr to the latest version"),
            skill,
            command("help", "Display detailed help text with usage instructions"),
        ],
        ..command(
            "sqlr",
            "Command line for interacting with SQL databases. Use '--help' for each command to list it's parameters.\n\n\
             For detailed usage instructions, workflows and guidance run 'sqlr help'.\n\
             AI agents: run 'sqlr help' first — it contains step-by-step instructions for operating sqlr.",
        )
    }
}

/// `-n <name>` or the `[connection]` argument.
fn connection_name(matches: &Matches) -> Option<String> {
    matches.string("name").or_else(|| match matches.arg(0) {
        Some(Value::String(name)) => Some(name.clone()),
        _ => None,
    })
}

fn execute(root: &Command, matches: &Matches) -> Result<(), String> {
    let is_global = matches.flag("global");
    let Some(name) = matches.command.first() else {
        logger::print_line(&cli::help(root, VERSION, &[]));
        return Ok(());
    };
    match name.as_str() {
        "add" => commands::add::execute(commands::add::Options {
            name: matches.string("name"),
            connection_type: matches.string("type"),
            connection_string: matches.string("connection-string"),
            color: matches.string("color"),
            emoji: matches.string("emoji"),
            is_global,
        }),
        "rm" => commands::rm::execute(connection_name(matches), is_global),
        "get" => commands::get::execute(connection_name(matches), is_global),
        "ls-types" => {
            commands::ls_types::execute();
            Ok(())
        }
        "ls" => commands::ls::execute(is_global),
        "clear" => commands::clear::execute(is_global),
        "set" => {
            commands::set::execute(connection_name(matches), matches.is_set("name"), is_global)
        }
        "set-global" => {
            commands::set_global::execute(matches!(matches.arg(0), Some(Value::Bool(true))))
        }
        "describe" => commands::describe::execute(commands::describe::Options {
            name: matches.string("name"),
            table: matches.flag("table"),
            compact: matches.flag("compact"),
            tables_only: matches.flag("tables-only"),
            connection_type: matches.string("type"),
            filter: matches.string("filter"),
            connection_string: matches.string("connection-string"),
            is_global,
        }),
        "query" => commands::query::execute(commands::query::Options {
            query: match matches.arg(0) {
                Some(Value::String(query)) => Some(query.clone()),
                _ => None,
            },
            input_variables: matches.list("input-variable"),
            output_file: matches.string("output-file"),
            name: matches.string("name"),
            table: matches.flag("table"),
            compact: matches.flag("compact"),
            connection_type: matches.string("type"),
            connection_string: matches.string("connection-string"),
            ignore_input_validation: matches.flag("ignore-input-validation"),
            is_global,
        }),
        "update" => update::execute(),
        "skill" => commands::skill::execute(matches.flag("init"), matches.string("name")),
        "help" => {
            logger::print_line(commands::HELP_TEXT);
            Ok(())
        }
        other => Err(format!("Unknown command '{other}'")),
    }
}

/// Anything that isn't an option or a command is a query:
/// `sqlr "SELECT 1"` is `sqlr query "SELECT 1"`.
fn with_implicit_query(root: &Command, args: Vec<String>) -> Vec<String> {
    match args.first() {
        Some(first)
            if !first.starts_with('-')
                && !root.commands.iter().any(|command| command.name == first) =>
        {
            std::iter::once("query".to_string()).chain(args).collect()
        }
        _ => args,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    logger::init(args.iter().any(|arg| arg == "--debug"));
    logger::debug("Debug mode enabled");
    let _ = rustls::crypto::ring::default_provider().install_default();

    let root = root();
    let args = with_implicit_query(&root, args);
    let matches = match cli::run(&root, VERSION, &args) {
        Outcome::Run(matches) => matches,
        Outcome::Exit(code) => std::process::exit(code),
    };

    let is_update = matches.command.first().is_some_and(|name| name == "update");
    let update_check = if is_update {
        None
    } else {
        update::start_background_check()
    };

    let code = match execute(&root, &matches) {
        Ok(()) => 0,
        Err(message) => {
            logger::error(message);
            1
        }
    };

    update::notify(update_check);
    std::process::exit(code);
}
