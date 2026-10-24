//! Development tasks, run through the cargo aliases in `.cargo/config.toml`:
//!
//! - `cargo verify`: formatting, lints and tests (run before committing)
//! - `cargo update-snapshots`: accept new CLI test output (all tests,
//!   including the ones that need the test databases)
//! - `cargo start-db` / `cargo stop-db`: test databases in Docker
//! - `cargo test-db`: tests against the test databases

use std::path::Path;
use std::process::{Command, ExitCode};

fn is_package_variable_of_xtask(name: &str) -> bool {
    name.starts_with("CARGO_PKG_")
        || name.starts_with("CARGO_MANIFEST_")
        || matches!(
            name,
            "CARGO_CRATE_NAME" | "CARGO_BIN_NAME" | "CARGO_PRIMARY_PACKAGE"
        )
}

fn cargo(args: &[&str], env: &[(&str, &str)]) -> bool {
    println!("> cargo {}", args.join(" "));
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = Command::new(cargo);
    for (name, _) in std::env::vars_os() {
        if name.to_str().is_some_and(is_package_variable_of_xtask) {
            command.env_remove(name);
        }
    }
    command
        .args(args)
        .envs(env.iter().copied())
        .status()
        .is_ok_and(|status| status.success())
}

fn run(program: &str, args: &[&str]) -> bool {
    println!("> {program} {}", args.join(" "));
    Command::new(program)
        .args(args)
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/db"))
        .status()
        .is_ok_and(|status| status.success())
}

/// Database tests compare dates in UTC.
const DB_TEST_ENV: &[(&str, &str)] = &[("TZ", "UTC")];

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    let ok = match task.as_str() {
        "verify" => {
            cargo(&["fmt", "--all", "--check"], &[])
                && cargo(
                    &[
                        "clippy",
                        "--workspace",
                        "--all-targets",
                        "--",
                        "-D",
                        "warnings",
                    ],
                    &[],
                )
                && cargo(&["test", "--workspace"], &[])
        }
        "update-snapshots" => cargo(
            &[
                "test",
                "--test",
                "cli",
                "--test",
                "databases",
                "--",
                "--include-ignored",
            ],
            &[("UPDATE_SNAPSHOTS", "1"), ("TZ", "UTC")],
        ),
        "start-db" => run("bash", &["start-db.sh"]),
        "stop-db" => run("docker", &["compose", "down", "--volumes"]),
        "test-db" => cargo(
            &["test", "--test", "databases", "--", "--ignored"],
            DB_TEST_ENV,
        ),
        _ => {
            eprintln!("Usage: cargo xtask <verify|update-snapshots|start-db|stop-db|test-db>");
            false
        }
    };
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
