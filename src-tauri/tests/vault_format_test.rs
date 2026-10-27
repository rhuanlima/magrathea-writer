use magrathea_writer_lib::crypto::{Argon2Params, SecretKey};
use magrathea_writer_lib::vault::{
    decrypt_blob, encrypt_blob, initialize_vault, ManifestItem, VaultError, VaultHeader,
    VaultManifest, MIN_ARGON2_ITERATIONS, MIN_ARGON2_MEMORY_KIB,
};

#[test]
fn test_vault_initialize_and_unlock_with_passphrase() {
    let vault_id = "550e8400-e29b-41d4-a716-446655440000";
    let created_at = "2026-10-27T10:00:00Z";
    let passphrase_input = b"magrathea_test_suite_key_alpha";
    let recovery_key = SecretKey::generate();

    // Fast params for unit test execution speed
    let test_params = Argon2Params {
        memory_kib: MIN_ARGON2_MEMORY_KIB,
        iterations: MIN_ARGON2_ITERATIONS,
        parallelism: 1,
    };

    let (header, original_dek) = initialize_vault(
        vault_id,
        created_at,
        passphrase_input,
        &recovery_key,
        Some(test_params),
    )
    .expect("vault initialization should succeed");

    assert_eq!(header.magic, "MAGRATHEA");
    assert_eq!(header.format_version, 1);
    assert_eq!(header.vault_id, vault_id);

    // Unlock with correct passphrase
    let unlocked_dek = header
        .unlock_with_passphrase(passphrase_input)
        .expect("unlock with correct passphrase should succeed");
    assert_eq!(unlocked_dek.as_bytes(), original_dek.as_bytes());

    // Unlock with wrong passphrase fails
    let wrong_result = header.unlock_with_passphrase(b"magrathea_test_suite_key_wrong");
    assert!(matches!(wrong_result, Err(VaultError::Crypto(_))));
}

#[test]
fn test_vault_unlock_with_recovery_key() {
    let vault_id = "550e8400-e29b-41d4-a716-446655440001";
    let created_at = "2026-10-27T10:00:00Z";
    let passphrase_input = b"magrathea_test_suite_key_forgotten";
    let recovery_key = SecretKey::generate();

    let test_params = Argon2Params {
        memory_kib: MIN_ARGON2_MEMORY_KIB,
        iterations: MIN_ARGON2_ITERATIONS,
        parallelism: 1,
    };

    let (header, original_dek) = initialize_vault(
        vault_id,
        created_at,
        passphrase_input,
        &recovery_key,
        Some(test_params),
    )
    .expect("vault initialization should succeed");

    // Unlock with recovery key
    let unlocked_dek = header
        .unlock_with_recovery_key(&recovery_key)
        .expect("unlock with recovery key should succeed");
    assert_eq!(unlocked_dek.as_bytes(), original_dek.as_bytes());

    // Unlock with wrong recovery key fails
    let wrong_recovery = SecretKey::generate();
    let wrong_result = header.unlock_with_recovery_key(&wrong_recovery);
    assert!(matches!(wrong_result, Err(VaultError::Crypto(_))));
}

#[test]
fn test_vault_header_validation_and_anti_downgrade() {
    let vault_id = "550e8400-e29b-41d4-a716-446655440002";
    let (mut header, _) = initialize_vault(
        vault_id,
        "2026-10-27T10:00:00Z",
        b"magrathea_test_suite_key_gamma",
        &SecretKey::generate(),
        Some(Argon2Params {
            memory_kib: MIN_ARGON2_MEMORY_KIB,
            iterations: MIN_ARGON2_ITERATIONS,
            parallelism: 1,
        }),
    )
    .unwrap();

    // Valid header passes
    assert!(header.validate().is_ok());

    // Bad magic fails
    let mut bad_magic = header.clone();
    bad_magic.magic = "INVALID_MAGIC".into();
    assert!(matches!(
        bad_magic.validate(),
        Err(VaultError::InvalidMagic(_))
    ));

    // Unsupported future format version fails
    let mut future_version = header.clone();
    future_version.format_version = 99;
    assert!(matches!(
        future_version.validate(),
        Err(VaultError::UnsupportedVersion(99))
    ));

    // Insecure memory cost fails (anti-downgrade attack)
    let mut weak_memory = header.clone();
    weak_memory.kdf_params.memory_kib = 1024;
    assert!(matches!(
        weak_memory.validate(),
        Err(VaultError::InsecureKdfParameters(_))
    ));

    // Insecure iterations count fails
    header.kdf_params.iterations = 1;
    assert!(matches!(
        header.validate(),
        Err(VaultError::InsecureKdfParameters(_))
    ));
}

#[test]
fn test_vault_header_json_roundtrip() {
    let vault_id = "550e8400-e29b-41d4-a716-446655440003";
    let (header, original_dek) = initialize_vault(
        vault_id,
        "2026-10-27T10:00:00Z",
        b"magrathea_test_suite_key_delta",
        &SecretKey::generate(),
        Some(Argon2Params {
            memory_kib: MIN_ARGON2_MEMORY_KIB,
            iterations: MIN_ARGON2_ITERATIONS,
            parallelism: 1,
        }),
    )
    .unwrap();

    let json_str = header.to_json().expect("JSON serialization should succeed");
    let parsed_header =
        VaultHeader::from_json(&json_str).expect("JSON deserialization should succeed");

    assert_eq!(parsed_header, header);

    // Ensure parsed header can still unlock the DEK
    let unlocked = parsed_header
        .unlock_with_passphrase(b"magrathea_test_suite_key_delta")
        .expect("unlocked from parsed header");
    assert_eq!(unlocked.as_bytes(), original_dek.as_bytes());
}

#[test]
fn test_encrypted_manifest_lifecycle_and_monotonic_versioning() {
    let vault_id = "550e8400-e29b-41d4-a716-446655440004";
    let dek = SecretKey::generate();

    let mut manifest = VaultManifest::new(
        vault_id,
        "O Mochileiro das Galáxias",
        "2026-10-27T10:00:00Z",
    );
    assert_eq!(manifest.monotonic_version, 1);

    // Add a scene item
    manifest.items.push(ManifestItem {
        id: "item-uuid-1".into(),
        kind: "scene".into(),
        parent_id: None,
        title: "Cena 1: A Terra e o Trator".into(),
        slug: "cena-1-a-terra-e-o-trator".into(),
        blob_id: Some("blob-uuid-1".into()),
        order: 0,
        tags: vec!["arthur".into(), "ford".into()],
        status: Some("draft".into()),
    });

    // Encrypt manifest blob
    let blob_bytes = manifest
        .encrypt(&dek)
        .expect("manifest encryption should succeed");

    // Decrypt manifest blob
    let decrypted_manifest = VaultManifest::decrypt(&dek, vault_id, &blob_bytes)
        .expect("manifest decryption should succeed");
    assert_eq!(decrypted_manifest, manifest);

    // Monotonic version bumping
    manifest.bump_version("2026-10-27T10:05:00Z");
    assert_eq!(manifest.monotonic_version, 2);

    // Tampering manifest blob fails
    let mut tampered_bytes = blob_bytes.clone();
    tampered_bytes[30] ^= 0x01;
    let tamper_result = VaultManifest::decrypt(&dek, vault_id, &tampered_bytes);
    assert!(matches!(tamper_result, Err(VaultError::Crypto(_))));

    // Forged vault ID in AAD fails
    let wrong_vault_result = VaultManifest::decrypt(&dek, "different-vault-id-uuid", &blob_bytes);
    assert!(matches!(wrong_vault_result, Err(VaultError::Crypto(_))));
}

#[test]
fn test_content_blob_encryption_and_anti_swap_protection() {
    let dek = SecretKey::generate();
    let blob_id_a = "scene-alpha-uuid";
    let blob_id_b = "scene-beta-uuid";
    let scene_text = b"---\ntitle: Cena Alpha\n---\nTexto literario confindencial...";

    // Encrypt Scene A
    let blob_a_bytes =
        encrypt_blob(&dek, blob_id_a, scene_text).expect("blob encryption should succeed");

    // Decrypt Scene A with correct blob_id
    let decrypted =
        decrypt_blob(&dek, blob_id_a, &blob_a_bytes).expect("blob decryption should succeed");
    assert_eq!(decrypted, scene_text);

    // Anti-blob-swap attack: Attempting to decrypt Blob A using Blob B's ID fails due to AAD mismatch
    let swap_attack = decrypt_blob(&dek, blob_id_b, &blob_a_bytes);
    assert!(matches!(swap_attack, Err(VaultError::Crypto(_))));

    // Truncated blob fails with CorruptedBlob
    let truncated = &blob_a_bytes[..10];
    let corrupt_result = decrypt_blob(&dek, blob_id_a, truncated);
    assert_eq!(corrupt_result, Err(VaultError::CorruptedBlob));
}
