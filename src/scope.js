import logger from "./logger.js";

let cachedScope = null;

const resolveScope = () => {
  if (cachedScope) return cachedScope;

  const home = Deno.env.get("HOME");
  let dir = Deno.cwd();

  while (true) {
    try {
      Deno.statSync(`${dir}/.git`);
      cachedScope = dir;
      return cachedScope;
    } catch {
      // .git not found at this level
    }

    if (dir === home || dir === "/") {
      cachedScope = "global";
      return cachedScope;
    }

    const parent = dir.substring(0, dir.lastIndexOf("/")) || "/";
    if (parent === dir) {
      cachedScope = "global";
      return cachedScope;
    }
    dir = parent;
  }
};

const GLOBAL_MODE_KEY = "sqlr:global-mode";

const isGlobalModeEnabled = () =>
  localStorage.getItem(GLOBAL_MODE_KEY) === "true";

const setGlobalMode = (enabled) => {
  logger.debug(`Setting global mode to ${enabled}`);
  if (enabled) {
    localStorage.setItem(GLOBAL_MODE_KEY, "true");
  } else {
    localStorage.removeItem(GLOBAL_MODE_KEY);
  }
};

const getScopePrefix = (isGlobal) => {
  const effective = isGlobal || isGlobalModeEnabled();
  const scope = effective ? "global" : resolveScope();
  return `scope:${scope}:`;
};

const getDefaultConnectionKey = (isGlobal) => {
  const effective = isGlobal || isGlobalModeEnabled();
  const scope = effective ? "global" : resolveScope();
  return `sqlr:default-connection:${scope}`;
};

const getDefaultConnection = (isGlobal) =>
  localStorage.getItem(getDefaultConnectionKey(isGlobal));

const setDefaultConnection = (connectionName, isGlobal) => {
  logger.debug(
    `Setting default connection to '${connectionName}' (global: ${!!isGlobal})`,
  );
  localStorage.setItem(getDefaultConnectionKey(isGlobal), connectionName);
};

const clearDefaultConnection = (isGlobal) => {
  logger.debug(`Clearing default connection (global: ${!!isGlobal})`);
  localStorage.removeItem(getDefaultConnectionKey(isGlobal));
};

const resetScopeCache = () => {
  cachedScope = null;
};

export {
  clearDefaultConnection,
  getDefaultConnection,
  getScopePrefix,
  GLOBAL_MODE_KEY,
  isGlobalModeEnabled,
  resetScopeCache,
  resolveScope,
  setDefaultConnection,
  setGlobalMode,
};
