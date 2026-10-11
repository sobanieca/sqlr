//! Password based encryption of connection strings: AES-256-GCM with a key
//! derived by PBKDF2-HMAC-SHA256 from a random salt. The result is base64 of
//! `salt | nonce | ciphertext`.

use ring::aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;

const SALT_LEN: usize = 16;
const ITERATIONS: u32 = 600_000;

fn key(password: &str, salt: &[u8]) -> LessSafeKey {
    let mut key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(ITERATIONS).expect("non-zero iterations"),
        salt,
        password.as_bytes(),
        &mut key,
    );
    LessSafeKey::new(UnboundKey::new(&AES_256_GCM, &key).expect("valid key length"))
}

pub fn encrypt(input: &str, password: &str) -> Result<String, String> {
    let random = SystemRandom::new();
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    random
        .fill(&mut salt)
        .and_then(|_| random.fill(&mut nonce))
        .map_err(|_| "Failed to generate random data for encryption")?;

    let mut data = input.as_bytes().to_vec();
    key(password, &salt)
        .seal_in_place_append_tag(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut data)
        .map_err(|_| "Failed to encrypt connection")?;

    let mut output = Vec::with_capacity(SALT_LEN + NONCE_LEN + data.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&data);
    Ok(base64_encode(&output))
}

pub fn decrypt(input: &str, password: &str) -> Result<String, String> {
    let data = base64_decode(input).ok_or("Invalid encrypted data")?;
    if data.len() < SALT_LEN + NONCE_LEN {
        return Err("Invalid encrypted data".to_string());
    }
    let (salt, rest) = data.split_at(SALT_LEN);
    let (nonce, ciphertext) = rest.split_at(NONCE_LEN);
    let nonce = Nonce::try_assume_unique_for_key(nonce).map_err(|_| "Invalid encrypted data")?;
    let mut ciphertext = ciphertext.to_vec();
    let plain = key(password, salt)
        .open_in_place(nonce, Aad::empty(), &mut ciphertext)
        .map_err(|_| "Decryption failed")?;
    String::from_utf8(plain.to_vec()).map_err(|_| "Decryption failed".to_string())
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let bytes = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(bytes[0]) << 16) | (u32::from(bytes[1]) << 8) | u32::from(bytes[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let input = input.trim_end_matches('=');
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0;
    for c in input.bytes() {
        let value = ALPHABET.iter().position(|&a| a == c)? as u32;
        buffer = (buffer << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_base64() {
        for (plain, encoded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(plain.as_bytes()), encoded);
            assert_eq!(base64_decode(encoded).unwrap(), plain.as_bytes());
        }
        assert_eq!(base64_decode("Zm9v!"), None);
    }

    #[test]
    fn round_trips_and_rejects_wrong_passwords() {
        let encrypted = encrypt("postgres://u:p@localhost/db", "password123").unwrap();
        assert_ne!(
            encrypted,
            encrypt("postgres://u:p@localhost/db", "password123").unwrap()
        );
        assert_eq!(
            decrypt(&encrypted, "password123").unwrap(),
            "postgres://u:p@localhost/db"
        );
        assert!(decrypt(&encrypted, "password124").is_err());
        assert!(decrypt("garbage", "password123").is_err());
    }
}
