import guard from "../guard.js";

console.log(Deno.osRelease());
console.log(Deno.hostname());

const test = async (input, password) => {
  console.log(`Testing encrypt and decrypt for "${input}" input and "${password}" password`);
  const encrypted = await guard.encrypt(input, password);
  console.log(`Encrypted data: ${encrypted}`);

  const decrypted = await guard.decrypt(encrypted, "myPassword");
  console.log(`Decrypted data: ${decrypted}`);
}

await test("Some dummy data to be encrypted", "myPassword");

//await test("Some other dummy data", "myAnotherPassword");

