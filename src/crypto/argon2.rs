use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use argon2::Argon2;
use rand::Rng;
use std::convert::TryFrom;
use thiserror::Error;

// --- Constants ---
const SALT_LEN: usize = 16;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("AES Error")]
    AesError(#[from] aes_gcm::Error),
    #[error("Argon2 Error")]
    Argon2Error(#[from] argon2::Error),
    #[error("Conversion Error")]
    ConversionError(#[from] std::array::TryFromSliceError),
    #[error("Infallible")]
    Infallible(#[from] std::convert::Infallible),
    #[error("Invalid Format")]
    InvalidFormat,
}

/// Encyrpt the input data with a password.
///
/// Parameters:
/// * data: Input data.
/// * password: Password used to encrypt the input data.
///
/// Returns:
/// The encrypted data or an encryption error.
pub fn encrypt(data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
    // --- Generate the salt ---
    let mut salt = [0u8; SALT_LEN];
    rand::rng().fill_bytes(&mut salt);

    // --- Generate the key ---
    let mut key_bytes = [0u8; KEY_LEN];
    Argon2::default().hash_password_into(password.as_bytes(), &salt, &mut key_bytes)?;
    let key: Key<Aes256Gcm> = key_bytes.try_into()?;

    // --- Generate the nonce ---
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::try_from(nonce_bytes)?;

    // --- Encrypt data ---
    let cipher = Aes256Gcm::new(&key);
    let bytes = cipher.encrypt(&nonce, data)?;

    // --- Result ---
    let mut output = Vec::with_capacity(SALT_LEN + NONCE_LEN + bytes.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&bytes);
    Ok(output)
}

/// Decrypt the input data with a password.
///
/// Parameters:
/// * data: Input data.
/// * password: Password used to decrypt the input data.
///
/// Returns:
/// The decrypted data or an encryption error.
pub fn decrypt(data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
    // --- Validate data length ---
    if data.len() < SALT_LEN + NONCE_LEN {
        return Err(CryptoError::InvalidFormat);
    }

    // --- Parse parameters ---
    let salt = &data[0..SALT_LEN];
    let nonce_bytes = &data[SALT_LEN..SALT_LEN + NONCE_LEN];
    let nonce = Nonce::try_from(nonce_bytes)?;
    let bytes = &data[SALT_LEN + NONCE_LEN..];

    // --- Recover key ---
    let mut key_bytes = [0u8; KEY_LEN];
    Argon2::default().hash_password_into(password.as_bytes(), salt, &mut key_bytes)?;
    let key: Key<Aes256Gcm> = key_bytes.try_into()?;

    // --- Decrypt data ---
    let cipher = Aes256Gcm::new(&key);
    let plain = cipher.decrypt(&nonce, bytes)?;

    // --- Result ---
    Ok(plain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let plain = b"Hello World!\nThis is a test.";
        let pass = "Super Secret!";

        let enc = encrypt(plain, pass).unwrap();
        assert_ne!(plain[..], enc);

        let dec = decrypt(&enc, pass).unwrap();
        assert_eq!(plain[..], dec);
    }

}
