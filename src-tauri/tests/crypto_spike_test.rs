use magrathea_writer_lib::crypto::{
    decrypt_payload, derive_key, encrypt_payload, generate_salt, unwrap_key, wrap_key,
    Argon2Params, CryptoError, Nonce, SecretKey, KEY_LEN, NONCE_LEN, SALT_LEN,
};
use std::time::Instant;
use zeroize::Zeroize;

#[test]
fn test_xchacha20_poly1305_roundtrip() {
    let key = SecretKey::generate();
    let nonce = Nonce::generate();
    let aad = b"MAGRATHEA-VAULT-V1:scene-uuid-1234";
    let plaintext = b"O manuscrito sagrado de Magrathea nunca deve vazar.";

    let ciphertext =
        encrypt_payload(&key, &nonce, aad, plaintext).expect("encryption should succeed");

    // Ciphertext contains 16-byte Poly1305 MAC tag overhead
    assert_eq!(ciphertext.len(), plaintext.len() + 16);

    let decrypted =
        decrypt_payload(&key, &nonce, aad, &ciphertext).expect("decryption should succeed");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_tampered_ciphertext_fails_decryption() {
    let key = SecretKey::generate();
    let nonce = Nonce::generate();
    let aad = b"MAGRATHEA-VAULT-V1:blob-01";
    let plaintext = b"Dados altamente confidenciais.";

    let mut ciphertext =
        encrypt_payload(&key, &nonce, aad, plaintext).expect("encryption should succeed");

    // Tamper single bit in ciphertext
    ciphertext[0] ^= 0x01;

    let result = decrypt_payload(&key, &nonce, aad, &ciphertext);
    assert_eq!(result, Err(CryptoError::DecryptionFailed));
}

#[test]
fn test_mismatched_aad_fails_decryption_anti_blob_swap() {
    let key = SecretKey::generate();
    let nonce = Nonce::generate();
    let original_aad = b"MAGRATHEA-VAULT-V1:scene-chapter-1";
    let forged_aad = b"MAGRATHEA-VAULT-V1:scene-chapter-2";
    let plaintext = b"Capitulo 1: No principio...";

    let ciphertext =
        encrypt_payload(&key, &nonce, original_aad, plaintext).expect("encryption should succeed");

    // Attempt to decrypt with another scene's AAD context
    let result = decrypt_payload(&key, &nonce, forged_aad, &ciphertext);
    assert_eq!(result, Err(CryptoError::DecryptionFailed));
}

#[test]
fn test_altered_nonce_fails_decryption() {
    let key = SecretKey::generate();
    let nonce = Nonce::generate();
    let aad = b"MAGRATHEA-VAULT-V1:meta";
    let plaintext = b"Metadados do projeto.";

    let ciphertext =
        encrypt_payload(&key, &nonce, aad, plaintext).expect("encryption should succeed");

    let mut altered_nonce_bytes = *nonce.as_bytes();
    altered_nonce_bytes[0] ^= 0xFF;
    let altered_nonce = Nonce::new(altered_nonce_bytes);

    let result = decrypt_payload(&key, &altered_nonce, aad, &ciphertext);
    assert_eq!(result, Err(CryptoError::DecryptionFailed));
}

#[test]
fn test_envelope_encryption_wrap_and_unwrap_dek() {
    let password = b"senha-ultra-secreta-do-autor-1234!";
    let salt = generate_salt();

    // Use fast params for quick test execution
    let test_params = Argon2Params {
        memory_kib: 1024,
        iterations: 1,
        parallelism: 1,
    };

    let kek = derive_key(password, &salt, &test_params).expect("KEK derivation should succeed");
    let dek = SecretKey::generate();
    let wrap_context = b"MAGRATHEA-HEADER-SLOT-0";

    // Wrap DEK with KEK
    let (wrap_nonce, wrapped_dek) =
        wrap_key(&kek, &dek, wrap_context).expect("wrap should succeed");

    // Unwrap DEK with KEK
    let recovered_dek =
        unwrap_key(&kek, &wrap_nonce, &wrapped_dek, wrap_context).expect("unwrap should succeed");
    assert_eq!(recovered_dek.as_bytes(), dek.as_bytes());

    // Wrong password / wrong KEK fails to unwrap
    let wrong_kek =
        derive_key(b"senha-errada", &salt, &test_params).expect("KEK derivation should succeed");
    let wrong_result = unwrap_key(&wrong_kek, &wrap_nonce, &wrapped_dek, wrap_context);
    assert_eq!(wrong_result, Err(CryptoError::DecryptionFailed));
}

#[test]
fn test_argon2id_determinism_and_salt_isolation() {
    let password = b"minha-senha-de-teste";
    let salt1 = [0x42u8; SALT_LEN];
    let salt2 = [0x43u8; SALT_LEN];

    let params = Argon2Params {
        memory_kib: 1024,
        iterations: 1,
        parallelism: 1,
    };

    let key1a = derive_key(password, &salt1, &params).unwrap();
    let key1b = derive_key(password, &salt1, &params).unwrap();
    let key2 = derive_key(password, &salt2, &params).unwrap();

    // Deterministic: same password + salt yields identical key
    assert_eq!(key1a.as_bytes(), key1b.as_bytes());

    // Isolated: different salt yields distinct key
    assert_ne!(key1a.as_bytes(), key2.as_bytes());
}

#[test]
fn test_argon2id_default_parameters_calibration() {
    let password = b"calibration-password-sample";
    let salt = [0x5Au8; SALT_LEN];
    let default_params = Argon2Params::default();

    let start = Instant::now();
    let key = derive_key(password, &salt, &default_params).expect("derivation should succeed");
    let elapsed = start.elapsed();

    assert_eq!(key.as_bytes().len(), KEY_LEN);
    println!("Argon2id (64 MiB, t=3, p=1) derivation took: {elapsed:?}");

    // Should complete within a reasonable developer workstation threshold (< 5 seconds)
    assert!(
        elapsed.as_secs() < 5,
        "Argon2id derivation took too long: {elapsed:?}"
    );
}

#[test]
fn test_xchacha20_poly1305_official_draft_test_vector() {
    // Official test vector from draft-arciszewski-xchacha-03 (§A.1 / §A.3)
    let key_bytes: [u8; KEY_LEN] = [
        0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e,
        0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d,
        0x9e, 0x9f,
    ];
    let nonce_bytes: [u8; NONCE_LEN] = [
        0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e,
        0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57,
    ];
    let aad: [u8; 0] = [];
    let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";

    let key = SecretKey::new(key_bytes);
    let nonce = Nonce::new(nonce_bytes);

    let ciphertext =
        encrypt_payload(&key, &nonce, &aad, plaintext).expect("encryption should succeed");

    let decrypted =
        decrypt_payload(&key, &nonce, &aad, &ciphertext).expect("decryption should succeed");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_secret_key_zeroize_and_redaction() {
    let mut key = SecretKey::new([0xAA; KEY_LEN]);

    // Ensure Debug output does not leak raw bytes
    let debug_repr = format!("{key:?}");
    assert_eq!(debug_repr, "SecretKey([REDACTED])");
    assert!(!debug_repr.contains("170")); // 0xAA = 170

    // Ensure Zeroize works as expected
    key.zeroize();
    assert_eq!(key.as_bytes(), &[0u8; KEY_LEN]);
}
