//! End-to-end tests that don't need databases: run the `sqlr` binary and
//! compare the output with snapshots in `tests/snapshots`.

mod support;

use support::Session;

#[test]
fn help_and_types() {
    let mut s = Session::new("help");
    s.run("sqlr");
    s.run("sqlr --help");
    s.run("sqlr ls-types");
    s.run("sqlr query --help");
    s.run("sqlr describe --help");
    s.run("sqlr add --help");
    s.run("sqlr set --help");
    s.run("sqlr skill --help");
    s.assert_snapshot("help_and_types");
}

#[test]
fn help_text() {
    let mut s = Session::new("help-text");
    s.run("sqlr help");
    s.run("sqlr skill");
    s.assert_snapshot("help_text");
}

#[test]
fn invalid_arguments() {
    let mut s = Session::new("invalid");
    s.run("sqlr query --foo");
    s.run("sqlr query -t oracle -s x \"SELECT 1\"");
    s.run("sqlr query --table --compact -t sqlite -s x \"SELECT 1\"");
    s.run("sqlr set-global maybe");
    s.run("sqlr set-global");
    s.assert_snapshot("invalid_arguments");
}

#[test]
fn add_list_and_clear() {
    let mut s = Session::new("main");
    s.run(r#"sqlr add -n conn-one -t postgresql -s "postgres://u:p@localhost:5432/db1""#);
    s.run(r#"sqlr add -n conn-two -t mysql -s "mysql://u:p@localhost:3306/db2""#);
    s.run("sqlr ls");
    s.run("sqlr clear");
    s.run("sqlr ls");
    s.assert_snapshot("add_list_and_clear");
}

#[test]
fn add_and_remove_by_name() {
    let mut s = Session::new("add-rm");
    s.run(r#"sqlr add -n test-pg -t postgresql -s "postgres://u:p@localhost:5432/db1""#);
    s.run(r#"sqlr add -n test-mysql -t mysql -s "mysql://u:p@localhost:3306/db2""#);
    s.run(r#"sqlr add -n test-mssql -t mssql -s "mssql://sa:World123!@localhost:1433/db""#);
    s.run(
        r#"sqlr add -n test-clickhouse -t clickhouse -s "http://u:p@localhost:8123?database=db""#,
    );
    s.run(r#"sqlr add -n test-sqlite -t sqlite -s "/tmp/db.sqlite""#);
    s.run("sqlr ls");
    s.run("sqlr rm -n test-pg");
    s.run("sqlr rm -n test-mysql");
    s.run("sqlr rm test-mssql");
    s.run("sqlr rm -n test-clickhouse");
    s.run("sqlr rm -n test-sqlite");
    s.run("sqlr ls");
    s.assert_snapshot("add_and_remove_by_name");
}

#[test]
fn invalid_connection_names() {
    let mut s = Session::new("names");
    s.run(r#"sqlr add -n a_b -t sqlite -s "/tmp/db.sqlite""#);
    s.run(r#"sqlr add -n ok-name -t sqlite -s "/tmp/db.sqlite""#);
    s.run(r#"sqlr add -n ok-name -t sqlite -s "/tmp/db.sqlite""#);
    s.run("sqlr get missing");
    s.run("sqlr set missing");
    s.assert_snapshot("invalid_connection_names");
}

#[test]
fn styled_connections() {
    let mut s = Session::new("styled");
    s.run(r#"sqlr add -n red-db -t sqlite -s "/tmp/db.sqlite" -c red -e "🚀""#);
    s.run(r#"sqlr add -n plain-db -t sqlite -s "/tmp/db.sqlite" -c none"#);
    s.run("sqlr set red-db");
    s.run("sqlr ls");
    s.run("sqlr get");
    s.assert_snapshot("styled_connections");
}

#[test]
fn connection_string_encoding() {
    let mut s = Session::new("encoding");
    s.run(
        r#"sqlr add -n enc-mssql -t mssql -s "mssql://sa:World123!%3B@localhost:1433/db?encrypt=true&trustServerCertificate=false""#,
    );
    s.run("sqlr get enc-mssql");
    s.run(
        r#"sqlr add -n enc-pg -t postgresql -s "postgres://localhost:5432/db?user=admin&password=p%40ss%3Aw%23rd&application_name=sqlr&sslmode=prefer""#,
    );
    s.run("sqlr get enc-pg");
    s.run(
        r#"sqlr add -n enc-mysql -t mysql -s 'mysql://u:p%40ss@localhost:3306/db?ssl={"rejectUnauthorized":false}'"#,
    );
    s.run("sqlr get enc-mysql");
    s.run(
        r#"sqlr add -n enc-ch -t clickhouse -s "https://user:p%40ss%3Bword@localhost:8443?database=mydb""#,
    );
    s.run("sqlr get enc-ch");
    s.run("sqlr clear");
    s.assert_snapshot("connection_string_encoding");
}

#[test]
fn encrypted_connection() {
    let mut s = Session::new("encrypted");
    let env = [("SQLR_ENCRYPTION_PASSWORD", "test-encryption-password123")];
    let wrong = [("SQLR_ENCRYPTION_PASSWORD", "wrong-password-123")];
    s.run_env(
        r#"sqlr add -n encrypted-pg -t postgresql -s "postgres://u:p@localhost:5432/db1""#,
        &env,
    );
    s.run_env("sqlr get encrypted-pg", &env);
    s.run_env("sqlr get encrypted-pg", &wrong);
    let storage =
        std::fs::read_to_string(s.path().join("config/sqlr/storage.json")).unwrap_or_default();
    assert!(
        storage.is_empty() || !storage.contains("localhost:5432"),
        "connection string must be stored encrypted: {storage}"
    );
    s.run_env(
        r#"sqlr add -n short-pwd -t postgresql -s "postgres://u:p@localhost:5432/db1""#,
        &[("SQLR_ENCRYPTION_PASSWORD", "short")],
    );
    s.run("sqlr clear");
    s.assert_snapshot("encrypted_connection");
}

#[test]
fn scoped_connections() {
    let mut s = Session::new("scoped");
    let repo = s.dir("repo");
    let other = s.dir("other");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    std::fs::create_dir_all(other.join(".git")).unwrap();
    let nested = s.dir("repo/nested/dir");

    s.run_in(
        r#"sqlr add -n repo-conn -t postgresql -s "postgres://u:p@localhost:5432/db1""#,
        &repo,
        &[],
    );
    s.run_in("sqlr ls", &repo, &[]);
    s.run_in("sqlr ls", &nested, &[]);
    s.run_in("sqlr ls", &other, &[]);
    s.run_in("sqlr ls -g", &repo, &[]);
    s.run_in(
        r#"sqlr add -n global-conn -t mysql -s "mysql://u:p@localhost:3306/db2" -g"#,
        &repo,
        &[],
    );
    s.run_in("sqlr ls -g", &repo, &[]);
    s.run_in("sqlr ls -g", &other, &[]);
    s.run_in("sqlr ls", &other, &[]);
    s.run_in("sqlr clear", &repo, &[]);
    s.run_in("sqlr ls", &repo, &[]);
    s.run_in("sqlr ls -g", &repo, &[]);
    s.assert_snapshot("scoped_connections");
}

#[test]
fn global_mode() {
    let mut s = Session::new("global-mode");
    let repo = s.dir("repo");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    s.run_in(
        r#"sqlr add -n global-db -t postgresql -s "postgres://u:p@localhost:5432/db1" -g"#,
        &repo,
        &[],
    );
    s.run_in("sqlr ls", &repo, &[]);
    s.run_in("sqlr set-global true", &repo, &[]);
    s.run_in("sqlr ls", &repo, &[]);
    s.run_in("sqlr set-global false", &repo, &[]);
    s.run_in("sqlr ls", &repo, &[]);
    s.assert_snapshot("global_mode");
}

#[test]
fn default_connection() {
    let mut s = Session::new("set");
    let repo = s.dir("repo");
    let other = s.dir("other");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    std::fs::create_dir_all(other.join(".git")).unwrap();

    s.run_in(
        r#"sqlr add -n repo-db -t postgresql -s "postgres://u:p@localhost:5432/db1""#,
        &repo,
        &[],
    );
    s.run_in("sqlr set -n repo-db", &repo, &[]);
    s.run_in("sqlr get", &repo, &[]);
    s.run_in("sqlr ls", &repo, &[]);
    s.run_in(
        r#"sqlr add -n other-db -t mysql -s "mysql://u:p@localhost:3306/db2""#,
        &other,
        &[],
    );
    s.run_in("sqlr set -n other-db", &other, &[]);
    s.run_in("sqlr get", &other, &[]);
    s.run_in(
        r#"sqlr add -n global-db -t postgresql -s "postgres://u:p@localhost:5432/db1" -g"#,
        &repo,
        &[],
    );
    s.run_in("sqlr set -n global-db -g", &repo, &[]);
    s.run_in("sqlr get", &other, &[]);
    s.run_in("sqlr set-global false", &repo, &[]);
    s.run_in("sqlr set", &repo, &[]);
    s.run_in("sqlr rm repo-db", &repo, &[]);
    s.run_in("sqlr ls", &repo, &[]);
    s.assert_snapshot("default_connection");
}

#[test]
fn query_input_validation() {
    let mut s = Session::new("query-input");
    let fixtures = support::fixtures();
    s.run_in(
        r#"sqlr query -t sqlite -s /nonexistent/db.sqlite test-query-vars.sql -i "col: 1""#,
        &fixtures,
        &[],
    );
    s.run_in(
        r#"sqlr query -t sqlite -s /nonexistent/db.sqlite test-query-vars.sql -i "col""#,
        &fixtures,
        &[],
    );
    s.run(r#"sqlr -t sqlite -s /nonexistent/db.sqlite"#);
    s.run("sqlr query");
    s.assert_snapshot("query_input_validation");
}

#[test]
fn skill_init() {
    let mut s = Session::new("skill");
    let repo = s.dir("repo");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    let service = s.dir("repo/services/api/src");
    std::fs::write(repo.join("services/api/CLAUDE.md"), "# api\n").unwrap();

    s.run_in("sqlr skill --init", &service, &[]);
    let skill = std::fs::read_to_string(repo.join(".agents/skills/sqlr/SKILL.md")).unwrap();
    let claude_skill = std::fs::read_to_string(repo.join(".claude/skills/sqlr/SKILL.md")).unwrap();
    assert_eq!(skill, claude_skill);
    assert!(!repo.join("services/api/.claude").exists());
    s.note(".agents/skills/sqlr/SKILL.md", &skill);

    let services = repo.join("services");
    s.run_in("sqlr skill --init -n prod", &services, &[]);
    let pinned = std::fs::read_to_string(repo.join(".claude/skills/sqlr-prod/SKILL.md")).unwrap();
    s.note(".claude/skills/sqlr-prod/SKILL.md", &pinned);
    s.run_in("sqlr skill --init -n prod", &services, &[]);

    let notes = s.dir("home/notes");
    s.run_in("sqlr skill --init", &notes, &[]);
    assert!(s.home.join(".agents/skills/sqlr/SKILL.md").exists());
    assert!(s.home.join(".claude/skills/sqlr/SKILL.md").exists());
    assert!(!notes.join(".claude").exists());
    s.assert_snapshot("skill_init");
}
