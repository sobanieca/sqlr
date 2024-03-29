import guard from "../src/guard.js";

const test = async (input, password) => {
  console.log(
    `Testing encrypt and decrypt for "${input}" input and "${password}" password`,
  );
  const encrypted = await guard.encrypt(input, password);
  console.log(`Encrypted data: ${encrypted}`);

  const decrypted = await guard.decrypt(encrypted, password);
  console.log(`Decrypted data: ${decrypted}`);
};

await test("Some dummy data to be encrypted", "myPassword");

await test("Some other dummy data", "myAnotherPassword!@#!@");
