//! End-to-end tests against the test databases in `tests/db` (start them with
//! `cargo start-db`). Run with `cargo test-db`.

mod support;

use support::{CLICKHOUSE, MSSQL, MYSQL, POSTGRES, POSTGRES_SSL, SQLITE, Session};

const POSTGRES_TYPES: &str = r#"SELECT 1::int2 AS i2, 2::int4 AS i4, 9007199254740993::int8 AS i8, 1.5::float4 AS f4, 2.25::float8 AS f8, 12.345::numeric AS num, true AS b, 'txt'::text AS t, 'ab'::char(3) AS bp, 'vc'::varchar AS vc, '2024-01-02'::date AS d, '2024-01-02 03:04:05.678'::timestamp AS ts, '2024-01-02 03:04:05.678+02'::timestamptz AS tstz, '03:04:05'::time AS tm, '03:04:05+02'::timetz AS tmtz, '{\"a\":1,\"b\":[1,2]}'::json AS j, '{\"a\":1}'::jsonb AS jb, '\\x0102ff'::bytea AS by, ARRAY[1,2,3] AS arr, ARRAY['a','b'] AS tarr, ARRAY[1.5,2.5]::float8[] AS farr, 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'::uuid AS u, '1 day 02:03:04'::interval AS iv, NULL AS n, '192.168.0.1'::inet AS ip, 12.5::money AS m, continent AS en, 1::oid AS o, 'NaN'::float8 AS nan, 'Infinity'::float8 AS inf, '1e21'::float8 AS big, 0.1::float4 AS f4b, 'a\"b''c' AS q FROM country ORDER BY code LIMIT 1"#;
const MYSQL_TYPES: &str = r#"SELECT 1 AS i, CAST(3 AS UNSIGNED) AS u, 9007199254740993 AS big, 12.345 AS dec1, CAST(1.5 AS FLOAT) AS f, CAST(2.25 AS DOUBLE) AS dbl, 'txt' AS t, DATE '2024-01-02' AS d, TIMESTAMP '2024-01-02 03:04:05.678' AS dt, TIME '03:04:05' AS tm, CAST('{\"a\":1,\"b\":[1,2]}' AS JSON) AS j, X'0102FF' AS bin, b'101' AS bit, NULL AS n, true AS b, population AS pop, surface_area AS sa, gnp AS gnp, indep_year AS iy, continent AS cont, life_expectancy AS le FROM country ORDER BY code LIMIT 2"#;
const MSSQL_TYPES: &str = r#"SELECT CAST(1 AS int) i, CAST(9007199254740993 AS bigint) big, CAST(12.345 AS decimal(10,3)) dec1, CAST(1.5 AS real) r, CAST(2.25 AS float) f, CAST(1 AS bit) b, N'txt' t, 'vc' vc, CAST('2024-01-02' AS date) d, CAST('2024-01-02 03:04:05.678' AS datetime2) dt2, CAST('2024-01-02 03:04:05.677' AS datetime) dt, CAST('2024-01-02 03:04:05' AS smalldatetime) sdt, CAST('2024-01-02T03:04:05.678+02:00' AS datetimeoffset) dto, CAST('03:04:05' AS time) tm, CAST(0x0102FF AS varbinary(10)) bin, CAST('A0EEBC99-9C0B-4EF8-BB6D-6BB9BD380A11' AS uniqueidentifier) u, CAST(12.5 AS money) m, NULL n, CAST(3 AS tinyint) ti, CAST(4 AS smallint) si, CAST(1.25 AS numeric(5,2)) nu, CAST('x' AS char(3)) ch, CAST('<a>1</a>' AS xml) x"#;
const CLICKHOUSE_TYPES: &str = r#"SELECT toInt32(1) i, toInt64(9007199254740993) big, toUInt8(3) u8, toUInt64(5) u64, toFloat32(1.5) f, toFloat64(2.25) d, toDecimal64(12.345, 3) dec1, 'txt' t, toDate('2024-01-02') dt, toDateTime('2024-01-02 03:04:05', 'UTC') ts, [1,2,3] arr, NULL n, true b, map('a',1) m, tuple(1,'a') tup, toUUID('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11') u, toNullable(toInt32(7)) ni"#;
const SQLITE_TYPES: &str = r#"SELECT 1 AS i, 1.5 AS r, 'txt' AS t, X'0102FF' AS b, NULL AS n, 9007199254740991 AS big, code, population, life_expectancy FROM country ORDER BY code LIMIT 2"#;

/// The scenario every database runs: queries in all output formats, files,
/// variables, describe and errors.
fn database_scenario(s: &mut Session, db: &str, cs: &str, limit_query: &str, types: &str) {
    let fixtures = support::fixtures();
    let output = s.path().join(format!("output-{db}.json"));
    let output = output.display();
    let q = |args: &str| format!(r#"sqlr query -t {db} -s "{cs}" {args}"#);
    let d = |args: &str| format!(r#"sqlr describe -t {db} -s "{cs}" {args}"#);

    s.run(&q(r#""SELECT 1 as test""#));
    s.run(&q(&format!(r#""{limit_query}""#)));
    s.run(&q(&format!(r#""{limit_query}" --table"#)));
    s.run(&q(&format!(r#""{limit_query}" --compact"#)));
    s.run_in(&q("test-query.sql"), &fixtures, &[]);
    s.run_in(
        &q(r#"test-query-vars.sql -i "col: 1" -i "alias: test""#),
        &fixtures,
        &[],
    );
    s.run(&q(&format!(r#""SELECT 1 as test" -o {output}"#)));
    s.note(
        "output file",
        &std::fs::read_to_string(s.path().join(format!("output-{db}.json"))).unwrap_or_default(),
    );
    s.run(&d(r#"-f "country""#));
    s.run(&d(r#"-f "country" --table"#));
    s.run(&d(r#"-f "country" --compact"#));
    s.run(&d(r#"-f "country" --tables-only"#));
    s.run_in(&q("test-query-handlebars.sql"), &fixtures, &[]);
    s.run_in(
        &q("test-query-handlebars.sql --ignore-input-validation"),
        &fixtures,
        &[],
    );
    s.run(&q(r#""INVALID SQL QUERY""#));

    s.run(&q(&format!(r#""{types}""#)));
    s.run(&q(&format!(r#""{types}" --compact"#)));
    s.run(&q(&format!(r#""{types}" --table"#)));
    s.run(&q(&format!(r#""{types}" -o {output}"#)));
    s.note(
        "output file",
        &std::fs::read_to_string(s.path().join(format!("output-{db}.json"))).unwrap_or_default(),
    );
}

#[test]
#[ignore = "requires test databases"]
fn postgresql() {
    let mut s = Session::new("postgresql");
    database_scenario(
        &mut s,
        "postgresql",
        POSTGRES,
        "SELECT code, name FROM country ORDER BY code LIMIT 3",
        POSTGRES_TYPES,
    );
    s.run(&format!(
        r#"sqlr query -t postgresql -s "{POSTGRES}" "SELECT 1 AS a; SELECT 2 AS b""#
    ));
    s.run(&format!(
        r#"sqlr query -t postgresql -s "{POSTGRES}" "SELECT * FROM country WHERE code = 'XXX'""#
    ));
    s.run(&format!(
        r#"sqlr query -t postgresql -s "{POSTGRES}" "CREATE TEMP TABLE t (id int)""#
    ));
    s.run(r#"sqlr query -t postgresql -s "postgres://localhost:5432/world-db?user=world&password=wrong" "SELECT 1""#);
    s.assert_snapshot("postgresql");
}

#[test]
#[ignore = "requires test databases"]
fn postgresql_ssl() {
    let mut s = Session::new("postgresql-ssl");
    s.run(&format!(
        r#"sqlr query -t postgresql -s "{POSTGRES_SSL}" "SELECT 1 as test""#
    ));
    s.run(&format!(
        r#"sqlr query -t postgresql -s "{POSTGRES_SSL}" "SELECT code, name FROM country ORDER BY code LIMIT 3""#
    ));
    s.run(&format!(
        r#"sqlr describe -t postgresql -s "{POSTGRES_SSL}" -f "country""#
    ));
    s.assert_snapshot("postgresql_ssl");
}

#[test]
#[ignore = "requires test databases"]
fn mysql() {
    let mut s = Session::new("mysql");
    database_scenario(
        &mut s,
        "mysql",
        MYSQL,
        "SELECT code, name FROM country ORDER BY code LIMIT 3",
        MYSQL_TYPES,
    );
    s.run(&format!(
        r#"sqlr query -t mysql -s "{MYSQL}" "UPDATE country SET name = name WHERE code = 'ABW'""#
    ));
    s.assert_snapshot("mysql");
}

#[test]
#[ignore = "requires test databases"]
fn mssql() {
    let mut s = Session::new("mssql");
    database_scenario(
        &mut s,
        "mssql",
        MSSQL,
        "SELECT TOP 3 code, name FROM country ORDER BY code",
        MSSQL_TYPES,
    );
    s.run(&format!(
        r#"sqlr query -t mssql -s "{MSSQL}" "UPDATE country SET name = name WHERE code = 'ABW'""#
    ));
    s.assert_snapshot("mssql");
}

#[test]
#[ignore = "requires test databases"]
fn clickhouse() {
    let mut s = Session::new("clickhouse");
    database_scenario(
        &mut s,
        "clickhouse",
        CLICKHOUSE,
        "SELECT code, name FROM country ORDER BY code LIMIT 3",
        CLICKHOUSE_TYPES,
    );
    s.assert_snapshot("clickhouse");
}

#[test]
#[ignore = "requires test databases"]
fn sqlite() {
    let mut s = Session::new("sqlite");
    database_scenario(
        &mut s,
        "sqlite",
        SQLITE,
        "SELECT code, name FROM country ORDER BY code LIMIT 3",
        SQLITE_TYPES,
    );
    let copy = s.path().join("copy.db");
    std::fs::copy(SQLITE, &copy).expect("copy sqlite database");
    let copy = copy.display().to_string();
    s.run(&format!(
        r#"sqlr query -t sqlite -s "{copy}" "UPDATE country SET name = name WHERE continent = 'Asia'""#
    ));
    s.assert_snapshot("sqlite");
}

#[test]
#[ignore = "requires test databases"]
fn stored_connection() {
    let mut s = Session::new("stored");
    s.run(&format!(
        r#"sqlr add -n world -t postgresql -s "{POSTGRES}" -c blue -e "📦""#
    ));
    s.run("sqlr ls");
    s.run("sqlr set world");
    s.run(r#"sqlr "SELECT 1 AS test""#);
    s.run(r#"sqlr describe --tables-only -f city"#);
    s.run(r#"sqlr query -n world "SELECT 2 AS test""#);
    s.run_env(
        &format!(r#"sqlr add -n secure -t sqlite -s "{SQLITE}""#),
        &[("SQLR_ENCRYPTION_PASSWORD", "test-encryption-password123")],
    );
    s.run_env(
        r#"sqlr -n secure "SELECT count(*) AS n FROM country""#,
        &[("SQLR_ENCRYPTION_PASSWORD", "test-encryption-password123")],
    );
    s.assert_snapshot("stored_connection");
}
