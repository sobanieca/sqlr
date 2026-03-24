import { assertSnapshot } from "jsr:@std/testing@1.0.12/snapshot";

const CONNECTION_STRING =
  "postgres://world:world123@localhost:5432/world-db?sslmode=disable";

const POSTGRES_SSL_CONNECTION_STRING =
  "postgres://world:world123@localhost:5433/world-db?sslmode=require&application_name=sqlr";

const MYSQL_CONNECTION_STRING =
  "mysql://world:world123@localhost:3306/world-db";

const MSSQL_CONNECTION_STRING =
  "mssql://sa:World123!@localhost:1433/world-db?encrypt=false&trustServerCertificate=true";

const CLICKHOUSE_CONNECTION_STRING =
  "http://world:world123@localhost:8123?database=world";

const SQLITE_CONNECTION_STRING = "/tmp/sqlr-sqlite/world.db";

export const run = async (cmd, cwd, env) => {
  const command = new Deno.Command("sh", {
    args: ["-c", cmd],
    stdout: "piped",
    stderr: "piped",
    cwd,
    env,
  });

  const { code, stdout, stderr } = await command.output();

  const removeAnsi = (input) => {
    // deno-lint-ignore no-control-regex
    const ansiEscapeSequences = /\u001b\[[0-9;]*[a-zA-Z]/g;
    return input.replace(ansiEscapeSequences, "");
  };

  const removeVaryingOutput = (input) => {
    return input
      .replace(/Version: \d+\.\d+\.\d+/g, "Version: *")
      .replace(/Query executed in: \d+ms/g, "Query executed in: *ms")
      .replace(/Rows affected: \d+/g, "Rows affected: *");
  };

  let output = new TextDecoder().decode(stdout);
  let outputError = new TextDecoder().decode(stderr);

  output = removeVaryingOutput(removeAnsi(output));
  output = output.replace(/Downloading https?:\/\/[^\n]*\n?/g, "");
  outputError = removeAnsi(outputError);
  outputError = outputError.replace(/Download https?:\/\/[^\n]*\n?/g, "");
  outputError = outputError.replace(
    /\[\d{4}-\d{2}-\d{2}T[\d:.]+Z\]/g,
    "[*]",
  );
  outputError = outputError.replace(
    /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/g,
    "*",
  );
  outputError = outputError.replace(
    /^ +at .+$/gm,
    "    at *",
  );
  outputError = outputError.replace(
    /( {4}at \*\n)+/g,
    "    at *\n",
  );

  return {
    code,
    output,
    outputError,
  };
};

export const createTestRunner = (t) => {
  return async (sqlrCommand, cwd, env) => {
    const projectRoot = Deno.cwd().replace("/test", "");
    const mainPath = `${projectRoot}/main.js`;
    const absoluteCwd = cwd ? `${projectRoot}/${cwd}` : undefined;

    await t.step(sqlrCommand + (cwd ? ` [cwd: ${cwd}]` : ""), async () => {
      const { code, output, outputError } = await run(
        sqlrCommand.replace("sqlr", `deno run -A ${mainPath}`),
        absoluteCwd,
        env,
      );

      await assertSnapshot(t, { sqlrCommand, code, output, outputError });
    });
  };
};

const getTestDir = () =>
  Deno.cwd().endsWith("/test") ? Deno.cwd() : `${Deno.cwd()}/test`;

export const startService = async (service) => {
  const testDir = getTestDir();

  const args = service
    ? ["compose", "up", "-d", "--wait", service]
    : ["compose", "up", "-d", "--wait"];

  const up = new Deno.Command("docker", {
    args: args,
    cwd: testDir,
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stderr } = await up.output();
  if (code !== 0) {
    const err = new TextDecoder().decode(stderr);
    throw new Error(`Failed to start ${service || "services"}: ${err}`);
  }
};

export const stopService = async (service) => {
  const testDir = getTestDir();

  const args = service ? ["compose", "stop", service] : ["compose", "down"];

  const down = new Deno.Command("docker", {
    args: args,
    cwd: testDir,
    stdout: "piped",
    stderr: "piped",
  });

  await down.output();
};

export const startPostgres = () => startService("postgres");
export const stopPostgres = () => stopService("postgres");

export const startPostgresSsl = () => startService("postgres-ssl");
export const stopPostgresSsl = () => stopService("postgres-ssl");

export const startMysql = () => startService("mysql");
export const stopMysql = () => stopService("mysql");

export const startMssql = async () => {
  await startService("mssql");

  const testDir = getTestDir();
  const init = new Deno.Command("docker", {
    args: ["compose", "up", "mssql-init"],
    cwd: testDir,
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stderr } = await init.output();
  if (code !== 0) {
    const err = new TextDecoder().decode(stderr);
    throw new Error(`Failed to init mssql: ${err}`);
  }
};
export const stopMssql = () => stopService("mssql");

export const startClickhouse = () => startService("clickhouse");
export const stopClickhouse = () => stopService("clickhouse");

export const startSqlite = async () => {
  const dir = SQLITE_CONNECTION_STRING.replace(/\/[^/]+$/, "");
  await Deno.mkdir(dir, { recursive: true });
  await Deno.chmod(dir, 0o777);

  const testDir = getTestDir();
  const init = new Deno.Command("docker", {
    args: ["compose", "up", "sqlite-init"],
    cwd: testDir,
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stderr } = await init.output();
  if (code !== 0) {
    const err = new TextDecoder().decode(stderr);
    throw new Error(`Failed to init sqlite: ${err}`);
  }
};

export const stopSqlite = async () => {
  const dir = SQLITE_CONNECTION_STRING.replace(/\/[^/]+$/, "");
  try {
    await Deno.remove(dir, { recursive: true });
  } catch {
    // ignore
  }
};

export {
  assertSnapshot,
  CLICKHOUSE_CONNECTION_STRING,
  CONNECTION_STRING,
  MSSQL_CONNECTION_STRING,
  MYSQL_CONNECTION_STRING,
  POSTGRES_SSL_CONNECTION_STRING,
  SQLITE_CONNECTION_STRING,
};
