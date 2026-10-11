//! Snapshot test harness: runs `sqlr` through `sh -c` in an isolated
//! environment (own home, storage and working directory) and compares the
//! normalized output with files in `tests/snapshots`.
//!
//! `UPDATE_SNAPSHOTS=1` rewrites the snapshots. `SQLR_TEST_BIN_DIR` runs the
//! tests against another `sqlr` executable (e.g. a shim for the Deno version
//! that passes `--location http://$SQLR_TEST_LOCATION/` to isolate storage).

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

pub const POSTGRES: &str =
    "postgres://localhost:5432/world-db?user=world&password=world123&application_name=sqlr";
pub const POSTGRES_SSL: &str = "postgres://localhost:5433/world-db?user=world&password=world123&application_name=sqlr&sslmode=prefer";
pub const MYSQL: &str = "mysql://world:world123@localhost:3306/world-db";
pub const MSSQL: &str =
    "mssql://sa:World123!@localhost:1433/world-db?encrypt=false&trustServerCertificate=true";
pub const CLICKHOUSE: &str = "http://world:world123@localhost:8123?database=world";
pub const SQLITE: &str = "/tmp/sqlr-sqlite/world.db";

pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn bin_dir() -> PathBuf {
    match std::env::var_os("SQLR_TEST_BIN_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => Path::new(env!("CARGO_BIN_EXE_sqlr"))
            .parent()
            .expect("binary directory")
            .to_path_buf(),
    }
}

/// An isolated environment: temporary home, storage and working directory.
/// Commands of one session share the stored connections.
pub struct Session {
    root: PathBuf,
    pub home: PathBuf,
    pub work: PathBuf,
    location: String,
    transcript: String,
}

impl Session {
    pub fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("sqlr-test-{name}-{}-{id}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let home = root.join("home");
        let work = root.join("work");
        std::fs::create_dir_all(&home).expect("create home");
        std::fs::create_dir_all(&work).expect("create work dir");
        let root = root.canonicalize().expect("temp dir path");
        Session {
            home: root.join("home"),
            work: root.join("work"),
            location: format!("sqlr-test-{name}-{}-{id}", std::process::id()),
            root,
            transcript: String::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Creates `relative` (and parents) inside the session directory.
    pub fn dir(&self, relative: &str) -> PathBuf {
        let path = self.root.join(relative);
        std::fs::create_dir_all(&path).expect("create dir");
        path
    }

    /// Runs `command` in `cwd` and returns (exit code, stdout, stderr), all
    /// normalized.
    pub fn exec(&self, command: &str, cwd: &Path, env: &[(&str, &str)]) -> (i32, String, String) {
        let path = format!(
            "{}:{}",
            bin_dir().display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(command)
            .current_dir(cwd)
            .env("PATH", path)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", self.root.join("config"))
            .env("XDG_CACHE_HOME", self.root.join("cache"))
            .env("TZ", "UTC")
            .env("SQLR_NO_UPDATE_CHECK", "1")
            .env("SQLR_TEST_LOCATION", &self.location)
            .env_remove("NO_COLOR")
            .env_remove("SQLR_ENCRYPTION_PASSWORD")
            .stdin(Stdio::null());
        for (key, value) in env {
            cmd.env(key, value);
        }
        let output = cmd.output().expect("failed to run sh");
        (
            output.status.code().unwrap_or(-1),
            self.normalize(&String::from_utf8_lossy(&output.stdout)),
            self.normalize(&String::from_utf8_lossy(&output.stderr)),
        )
    }

    /// Runs a command and appends its output to the session transcript.
    pub fn run_in(&mut self, command: &str, cwd: &Path, env: &[(&str, &str)]) -> String {
        let (code, stdout, stderr) = self.exec(command, cwd, env);
        let location = match cwd.strip_prefix(&self.root) {
            Ok(relative) if relative != Path::new("work") => {
                format!(" [cwd: {}]", relative.display())
            }
            _ => String::new(),
        };
        let command = command.replace(&self.root.display().to_string(), "<tmp>");
        let output = format!(
            "$ {command}{location}\nexit code: {code}\n--- stdout\n{stdout}--- stderr\n{stderr}\n"
        );
        self.transcript.push_str(&output);
        stdout
    }

    pub fn run(&mut self, command: &str) -> String {
        let work = self.work.clone();
        self.run_in(command, &work, &[])
    }

    pub fn run_env(&mut self, command: &str, env: &[(&str, &str)]) -> String {
        let work = self.work.clone();
        self.run_in(command, &work, env)
    }

    /// Adds a free-form note (e.g. file contents) to the transcript.
    pub fn note(&mut self, title: &str, content: &str) {
        self.transcript
            .push_str(&format!("--- {title}\n{}\n", self.normalize(content)));
    }

    pub fn assert_snapshot(&self, name: &str) {
        assert_snapshot(name, &self.transcript);
    }

    fn normalize(&self, s: &str) -> String {
        let s = strip_ansi(s).replace(&self.root.display().to_string(), "<tmp>");
        let mut out = String::with_capacity(s.len());
        for line in s.split_inclusive('\n') {
            out.push_str(&normalize_line(line));
        }
        out
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Masks values that change between runs or versions.
fn normalize_line(line: &str) -> String {
    if let Some(rest) = line.strip_prefix("Query executed in: ") {
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if digits > 0 && rest[digits..].starts_with("ms") {
            return format!("Query executed in: *ms{}", &rest[digits + 2..]);
        }
    }
    if let Some(rest) = line.strip_prefix("Version: ") {
        let version = rest.split_whitespace().next().unwrap_or_default();
        return format!("Version: *{}", &rest[version.len()..]);
    }
    line.to_string()
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
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

pub fn assert_snapshot(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.snap"));
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().expect("snapshot dir")).expect("snapshot dir");
        std::fs::write(&path, actual).expect("failed to write snapshot");
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing snapshot {}, run `cargo update-snapshots`\n\n{actual}",
            path.display()
        )
    });
    if expected != actual {
        panic!(
            "snapshot {name} does not match (run `cargo update-snapshots` to accept)\n--- expected\n{expected}\n--- actual\n{actual}"
        );
    }
}
