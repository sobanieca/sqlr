import { assertSnapshot } from "jsr:@std/testing@1.0.12/snapshot";

const CONNECTION_STRING =
  "postgres://world:world123@localhost:5432/world-db?sslmode=disable";

export const run = async (cmd, cwd) => {
  const command = new Deno.Command("sh", {
    args: ["-c", cmd],
    stdout: "piped",
    stderr: "piped",
    cwd: cwd,
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
  outputError = removeAnsi(outputError);
  outputError = outputError.replace(/Download https?:\/\/[^\n]*\n?/g, "");

  return {
    code,
    output,
    outputError,
  };
};

export const createTestRunner = (t) => {
  return async (sqlrCommand, cwd) => {
    const projectRoot = Deno.cwd().replace("/test", "");
    const mainPath = `${projectRoot}/main.js`;
    const absoluteCwd = cwd ? `${projectRoot}/${cwd}` : undefined;

    await t.step(sqlrCommand + (cwd ? ` [cwd: ${cwd}]` : ""), async () => {
      const { code, output, outputError } = await run(
        sqlrCommand.replace("sqlr", `deno run -A ${mainPath}`),
        absoluteCwd,
      );

      await assertSnapshot(t, { sqlrCommand, code, output, outputError });
    });
  };
};

export const startPostgres = async () => {
  const testDir = Deno.cwd().endsWith("/test")
    ? Deno.cwd()
    : `${Deno.cwd()}/test`;

  const up = new Deno.Command("docker", {
    args: ["compose", "up", "-d", "--wait"],
    cwd: testDir,
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stderr } = await up.output();
  if (code !== 0) {
    const err = new TextDecoder().decode(stderr);
    throw new Error(`Failed to start postgres: ${err}`);
  }
};

export const stopPostgres = async () => {
  const testDir = Deno.cwd().endsWith("/test")
    ? Deno.cwd()
    : `${Deno.cwd()}/test`;

  const down = new Deno.Command("docker", {
    args: ["compose", "down"],
    cwd: testDir,
    stdout: "piped",
    stderr: "piped",
  });

  await down.output();
};

export { CONNECTION_STRING };
