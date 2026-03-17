import { getScopePrefix } from "./scope.js";

const getItem = (name, isGlobal) => {
  const prefix = getScopePrefix(isGlobal);
  return localStorage.getItem(`${prefix}${name}`);
};

const setItem = (name, value, isGlobal) => {
  const prefix = getScopePrefix(isGlobal);
  localStorage.setItem(`${prefix}${name}`, value);
};

const removeItem = (name, isGlobal) => {
  const prefix = getScopePrefix(isGlobal);
  localStorage.removeItem(`${prefix}${name}`);
};

const getAllKeys = (isGlobal) => {
  const prefix = getScopePrefix(isGlobal);
  const keys = [];
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key.startsWith(prefix)) {
      keys.push(key.substring(prefix.length));
    }
  }
  return keys;
};

const clear = (isGlobal) => {
  const prefix = getScopePrefix(isGlobal);
  const keysToRemove = [];
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key.startsWith(prefix)) {
      keysToRemove.push(key);
    }
  }
  for (const key of keysToRemove) {
    localStorage.removeItem(key);
  }
};

export default { getItem, setItem, removeItem, getAllKeys, clear };
