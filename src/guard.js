const encoder = new TextEncoder();
const decoder =  new TextDecoder();

const getIv = () => 
  encoder.encode(Deno.osRelease() + Deno.hostname());

const getPasswordKey = async (password) => {
  const baseKey = await crypto.subtle.importKey(
    "raw",
    encoder.encode(password),
    "PBKDF2",
    false,
    ["deriveKey"]
  );

  const derivedKey = await crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: Deno.hostname() + Deno.osRelease(),
      iterations: 25000,
      hash: "SHA-256",
    },
    baseKey,
    { name: "AES-GCM", length: 256 },
    false,
    [ "encrypt", "decrypt" ]
  );

  return derivedKey;
}

const encrypt = async (input, password) => {
  const key = getPasswordKey(password);
  const result = await crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv: getIv(),
    },
    key,
    encoder.encode(input)
  );
 
  return decoder.decode(new Uint8Array(result));
}

const decrypt = async (input, password) => {
  const key = getPasswordKey(password);
  const result = await crypto.subtle.decrypt(
    {
      name: "AES-GCM",
      iv: getIv(),
    },
    key,
    encoder.encode(input)
  );

  return decoder.decode(result);
}

const guard = {
  encrypt,
  decrypt
};

export default guard;
