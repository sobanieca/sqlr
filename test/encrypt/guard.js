let Deno = {
  hostname: () => "5.10.16.3-microsoft-standard-WSL2",
  osRelease: () => "AdamXPS13"
}

const encoder = new TextEncoder();
const decoder =  new TextDecoder();

const getIv = () => { 
  return encoder.encode(Deno.osRelease() + Deno.hostname()).slice(0, 12);
}

const getSalt = () => {
  return encoder.encode(Deno.hostname() + Deno.osRelease()).slice(0, 12);
}

const getPasswordKey = async (password) => {
  const baseKey = await crypto.subtle.importKey(
    "raw",
    encoder.encode(password),
    "PBKDF2",
    false,
    [ "deriveKey", "deriveBits" ]
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
    [ "encrypt", "decrypt" ]
  );

  return derivedKey;
}

const encrypt = async (input, password) => {
  const key = await getPasswordKey(password);
  let result = await crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv: getIv(),
    },
    key,
    encoder.encode(input)
  );

  console.log(result);
  result = btoa(encodeURIComponent(decoder.decode(result)));
  return result;
}

const decrypt = async (input, password) => {
  const key = await getPasswordKey(password);

  input = encoder.encode(decodeURIComponent(atob(input)));
  console.log(input);

  const result = await crypto.subtle.decrypt(
    {
      name: "AES-GCM",
      iv: getIv(),
    },
    key,
    input.buffer
  );

  return decoder.decode(result);
}

const guard = {
  encrypt,
  decrypt
};

export default guard;
