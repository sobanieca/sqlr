import { Command, dirname, join } from "../deps.js";
import logger from "../logger.js";
import helpText from "./help.js";

const SKILL_TARGETS = [
  { dir: ".agents/skills", readBy: "Codex, Cursor" },
  { dir: ".claude/skills", readBy: "Claude Code, Cursor" },
];

const skillContent = (connection) => {
  const name = connection ? `sqlr-${connection}` : "sqlr";
  const target = connection ? `the \`${connection}\` database` : "a database";
  const select = connection
    ? `2. Run \`sqlr set ${connection}\` first so every command targets this
   connection.`
    : `2. \`sqlr ls\` lists connections, the default is marked. Switch with
   \`sqlr set <name>\`.`;

  return `---
name: ${name}
description: Run SQL queries and inspect the schema of ${target} with the sqlr CLI. Use when asked to query, inspect, report on or change data in ${target}.
---

# ${name}

\`sqlr\` runs SQL against connections saved on this machine. Connections are
referenced by name. You never need, and must never ask for, a connection
string or password.

## Workflow

1. Run \`sqlr skill\` once to read the full reference (all commands and flags).
${select}
3. \`sqlr describe --compact\` shows tables, columns, types and foreign keys.
   Narrow it with \`-f <table>\`. Read the schema before writing queries.
4. Run queries: \`sqlr "SELECT ... LIMIT 20"\` or
   \`sqlr path/to/query.sql -i "param: value"\`.
5. Use \`-o result.json\` for large results and \`--table\` for readable output.

## Rules

- Always add LIMIT to exploratory SELECT statements.
- Show UPDATE, DELETE, DROP and other destructive statements to the user and
  ask before running them.
- If a command prompts for an encryption password, stop and ask the user to
  run it, or to set SQLR_ENCRYPTION_PASSWORD.
`;
};

const agentInstructions = `
sqlr - instructions for AI agents

  Run 'sqlr skill --init' to create a skill so you can invoke it with
  /sqlr (Claude Code, Cursor) or $sqlr (Codex). Inside a git repository
  the skill is created in the repository, otherwise in the home directory.
  Add '-n <connection>' to create a skill pinned to one connection,
  for example /sqlr-prod.

  Connections are referenced by name. Connection strings and passwords are
  stored on disk by the user. Never ask for them and never put them into
  commands.

  Full reference follows.
`;

const exists = (path) => {
  try {
    Deno.statSync(path);
    return true;
  } catch {
    return false;
  }
};

const homeDir = () => Deno.env.get("HOME") || Deno.env.get("USERPROFILE");

const findSkillRoot = (start = Deno.cwd(), home = homeDir()) => {
  let dir = start;
  while (true) {
    if (exists(join(dir, ".git"))) {
      return { dir, reason: "git root", global: false };
    }
    const parent = dirname(dir);
    if (dir === home || parent === dir) break;
    dir = parent;
  }
  if (!home) {
    logger.error(
      "Not in a git repository and the home directory is unknown (set HOME).",
    );
    Deno.exit(1);
  }
  return { dir: home, reason: "not in a git repository", global: true };
};

const createSkill = async (connection) => {
  const skillName = connection ? `sqlr-${connection}` : "sqlr";
  const root = findSkillRoot();
  const files = SKILL_TARGETS.map((target) => ({
    target,
    path: join(target.dir, skillName, "SKILL.md"),
  }));

  for (const { path } of files) {
    const skillFile = join(root.dir, path);
    if (exists(skillFile)) {
      logger.error(
        `${skillFile} already exists. Delete it first if you want to regenerate it.`,
      );
      Deno.exit(1);
    }
  }

  for (const { path } of files) {
    const skillFile = join(root.dir, path);
    await Deno.mkdir(dirname(skillFile), { recursive: true });
    await Deno.writeTextFile(skillFile, skillContent(connection));
  }

  logger.info(
    root.global
      ? `Not in a git repository - installing for all projects in ${root.dir}`
      : `Project root: ${root.dir} (${root.reason})`,
  );
  for (const { path, target } of files) {
    logger.info(`Created ${path}  (${target.readBy})`);
  }
  logger.info("");
  logger.info(`Try: /${skillName} how many users signed up this week`);
};

export default new Command()
  .description(
    "Print instructions for AI agents. Use --init to create a skill (/sqlr)",
  )
  .option(
    "--init",
    "Create the skill in the current git repository, or in the home directory outside one",
  )
  .option(
    "-n, --name <name:string>",
    "Pin the created skill to a connection (creates /sqlr-<name>)",
  )
  .action(async ({ init, name }) => {
    if (init) {
      await createSkill(name);
      return;
    }
    console.log(agentInstructions);
    console.log(helpText.help);
  });
