//! Cryptographic primitives, key derivation, and envelope encryption for Magrathea Writer.
//!
//! Follows architecture decisions established in ADR 0003:
//! - AEAD: XChaCha20-Poly1305 with 192-bit random nonces and AAD binding.
//! - KDF: Argon2id (RFC 9106) with memory-hard parameters (64 MiB, t=3, p=1).
//! - CSPRNG: OS-backed entropy via `rand`.
//! - Memory Hygiene: `Zeroize` and `ZeroizeOnDrop` on sensitive key material.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore;
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Key length in bytes for 256-bit symmetric keys.
pub const KEY_LEN: usize = 32;

/// Nonce length in bytes for XChaCha20-Poly1305 (192 bits).
pub const NONCE_LEN: usize = 24;

/// Salt length in bytes for Argon2id key derivation (128 bits).
pub const SALT_LEN: usize = 16;

/// Default Argon2id memory cost in KiB (64 MiB).
pub const DEFAULT_M_COST: u32 = 65536;

/// Default Argon2id time cost (iterations).
pub const DEFAULT_T_COST: u32 = 3;

/// Default Argon2id parallelism cost (threads/lanes).
pub const DEFAULT_P_COST: u32 = 1;

/// Cryptographic error types.
#[derive(Debug, PartialEq, Eq)]
pub enum CryptoError {
    /// Failure during password-based key derivation.
    KeyDerivationFailed(String),
    /// Failure during authenticated encryption.
    EncryptionFailed,
    /// Failure during authenticated decryption (bad key, corrupted ciphertext, or altered AAD).
    DecryptionFailed,
    /// Input buffer length does not match expected cryptographic size.
    InvalidLength { expected: usize, actual: usize },
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyDerivationFailed(err) => write!(f, "Argon2id key derivation failed: {err}"),
            Self::EncryptionFailed => write!(f, "Authenticated encryption failed"),
            Self::DecryptionFailed => write!(
                f,
                "Authenticated decryption failed (authentication tag mismatch or data corrupted)"
            ),
            Self::InvalidLength { expected, actual } => write!(
                f,
                "Invalid buffer length: expected {expected} bytes, received {actual} bytes"
            ),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Configuration parameters for Argon2id password-based key derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Argon2Params {
    /// Memory cost in KiB.
    pub memory_kib: u32,
    /// Number of iterations.
    pub iterations: u32,
    /// Parallelism lanes.
    pub parallelism: u32,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            memory_kib: DEFAULT_M_COST,
            iterations: DEFAULT_T_COST,
            parallelism: DEFAULT_P_COST,
        }
    }
}

/// A 256-bit secret key protected with automatic zeroization on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct SecretKey([u8; KEY_LEN]);

impl SecretKey {
    /// Creates a new `SecretKey` from raw bytes.
    pub fn new(bytes: [u8; KEY_LEN]) -> Self {
        Self(bytes)
    }

    /// Generates a new random `SecretKey` using the operating system's CSPRNG.
    pub fn generate() -> Self {
        let mut bytes = [0u8; KEY_LEN];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Accesses raw key slice for cryptographic operations.
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretKey([REDACTED])")
    }
}

/// A 192-bit nonce for XChaCha20-Poly1305.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nonce([u8; NONCE_LEN]);

impl Nonce {
    /// Creates a `Nonce` from an existing 24-byte array.
    pub fn new(bytes: [u8; NONCE_LEN]) -> Self {
        Self(bytes)
    }

    /// Generates a new random 192-bit `Nonce` using CSPRNG.
    pub fn generate() -> Self {
        let mut bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Accesses raw nonce bytes.
    pub fn as_bytes(&self) -> &[u8; NONCE_LEN] {
        &self.0
    }
}

/// Generates a random 128-bit salt for key derivation.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Derives a 256-bit Key Encryption Key (KEK) from a password using Argon2id.
pub fn derive_key(
    password: &[u8],
    salt: &[u8; SALT_LEN],
    params: &Argon2Params,
) -> Result<SecretKey, CryptoError> {
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(KEY_LEN),
    )
    .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);

    let mut derived = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password, salt, &mut derived)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(SecretKey::new(derived))
}

/// Encrypts plaintext using XChaCha20-Poly1305 with Associated Authenticated Data (AAD).
pub fn encrypt_payload(
    key: &SecretKey,
    nonce: &Nonce,
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_bytes())
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let xnonce = XNonce::from_slice(nonce.as_bytes());
    let payload = Payload {
        msg: plaintext,
        aad,
    };

    cipher
        .encrypt(xnonce, payload)
        .map_err(|_| CryptoError::EncryptionFailed)
}

/// Decrypts ciphertext using XChaCha20-Poly1305 and verifies authenticity with AAD.
pub fn decrypt_payload(
    key: &SecretKey,
    nonce: &Nonce,
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_bytes())
        .map_err(|_| CryptoError::DecryptionFailed)?;

    let xnonce = XNonce::from_slice(nonce.as_bytes());
    let payload = Payload {
        msg: ciphertext,
        aad,
    };

    cipher
        .decrypt(xnonce, payload)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Wraps (encrypts) a Data Encryption Key (DEK) with a Key Encryption Key (KEK).
pub fn wrap_key(
    kek: &SecretKey,
    dek: &SecretKey,
    aad_context: &[u8],
) -> Result<(Nonce, Vec<u8>), CryptoError> {
    let nonce = Nonce::generate();
    let wrapped = encrypt_payload(kek, &nonce, aad_context, dek.as_bytes())?;
    Ok((nonce, wrapped))
}

/// Unwraps (decrypts) a Data Encryption Key (DEK) using a Key Encryption Key (KEK).
pub fn unwrap_key(
    kek: &SecretKey,
    nonce: &Nonce,
    wrapped_dek: &[u8],
    aad_context: &[u8],
) -> Result<SecretKey, CryptoError> {
    let unwrapped_bytes = decrypt_payload(kek, nonce, aad_context, wrapped_dek)?;
    if unwrapped_bytes.len() != KEY_LEN {
        return Err(CryptoError::InvalidLength {
            expected: KEY_LEN,
            actual: unwrapped_bytes.len(),
        });
    }

    let mut key_bytes = [0u8; KEY_LEN];
    key_bytes.copy_from_slice(&unwrapped_bytes);
    Ok(SecretKey::new(key_bytes))
}
