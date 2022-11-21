import guard from "../src/guard.js";


const test = (input, password) => {
  console.log(`Testing encrypt and decrypt for "${input}" input and "${password}" password`);
  const encrypted = guard.encrypt(input, password);
  console.log(`Encrypted data: ${encrypted}`);

  console.log(`Decrypted data: ${guard.decrypt(encrypted, "myPassword")}`);

 
}

test("Some dummy data to be encrypted", "myPassword");

test("Some other dummy data", "myAnotherPassword");

