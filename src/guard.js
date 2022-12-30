import logger from "./logger.js";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

const getIv = () => {
  const ivCore = (Deno.osRelease() + Deno.hostname()).padStart(12, "0");
  logger.debug(`Generating initial vector for encryption with ${ivCore}`);
  return encoder.encode(ivCore).slice(0, 12);
};

const getSalt = () => {
  const saltCore = (Deno.hostname() + Deno.osRelease()).padStart(12, "0");
  logger.debug(`Generating salt with ${saltCore}`);
  return encoder.encode(saltCore).slice(0, 12);
};

const getPasswordKey = async (password) => {
  const baseKey = await crypto.subtle.importKey(
    "raw",
    encoder.encode(password),
    "PBKDF2",
    false,
    ["deriveKey", "deriveBits"],
  );

  const derivedKey = await crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: getSalt(),
      iterations: 25000,
      hash: "SHA-256",
    },
    baseKey,
    { name: "AES-GCM", length: 256 },
    false,
    ["encrypt", "decrypt"],
  );

  logger.debug("Generated password key");
  return derivedKey;
};

const encrypt = async (input, password) => {
  const key = await getPasswordKey(password);
  const result = await crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv: getIv(),
    },
    key,
    encoder.encode(input),
  );

  return btoa(new Uint8Array(result).join("+"));
};

const decrypt = async (input, password) => {
  const key = await getPasswordKey(password);

  input = Uint8Array.from(atob(input).split("+")).buffer;

  const result = await crypto.subtle.decrypt(
    {
      name: "AES-GCM",
      iv: getIv(),
    },
    key,
    input,
  );

  return decoder.decode(result);
};

const guard = {
  encrypt,
  decrypt,
};

export default guard;
