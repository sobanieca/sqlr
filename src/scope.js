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

const resetScopeCache = () => {
  cachedScope = null;
};

export {
  getScopePrefix,
  GLOBAL_MODE_KEY,
  isGlobalModeEnabled,
  resetScopeCache,
  resolveScope,
  setGlobalMode,
};
