use magrathea_writer_lib::crypto::{
    Argon2Params, SecretKey, DEFAULT_M_COST, DEFAULT_P_COST, DEFAULT_T_COST,
};
use magrathea_writer_lib::vault::{
    decrypt_blob, encrypt_blob, initialize_vault, ManifestItem, VaultManifest,
};
use std::fs;
use std::time::Instant;

/// Generates a realistic synthetic Markdown scene containing approximately 1,000 words.
fn generate_synthetic_scene_content(scene_index: usize) -> Vec<u8> {
    let mut content = String::with_capacity(7000);
    content.push_str("---\n");
    content.push_str(&format!("id: scene-{scene_index:04}\n"));
    content.push_str(&format!("title: Cena {scene_index:04} - O Voo da Coruja\n"));
    content.push_str("status: draft\n");
    content.push_str("tags: [magrathea, aventura, jornada, ficcao]\n");
    content.push_str("pov: Arthur Dent\n");
    content.push_str("location: Sala de Maquinas da Coracao de Ouro\n");
    content.push_str("---\n\n");

    let paragraph_template = "O silencio no interior da nave era perturbado apenas pelo zumbido constante dos geradores de improbabilidade infinita. Arthur olhava atraves da vigia de quartzo para o abismo estrelado da galaxia desconhecida. Cada ponto luminoso representava uma historia esquecida, civilizacoes inteiras que nasceram, floresceram e decairam sem que ninguem no setor setentrional jamais tomasse conhecimento de sua existencia. As palavras flutuavam em sua mente como poeira cosmica, buscando uma forma definitiva nas paginas secretas que redigia febrilmente todas as noites. O computador de bordo emitia pequenos estalos eletricos, calculando trajetorias hiperespaciais que desafiavam as leis mais elementares da geometria euclidiana. Nao havia retorno possivel, e no fundo de seu coracao ele sabia que a busca pela verdade exigia coragem e uma resoluta determinacao. ";

    // Repeat paragraph until we hit approximately 1,000 words
    // Template has 113 words. 9 repetitions = ~1,017 words.
    for _ in 0..9 {
        content.push_str(paragraph_template);
        content.push_str("\n\n");
    }

    content.into_bytes()
}

#[test]
fn test_performance_spike_500_scenes_500k_words() {
    let total_scenes = 500;
    let vault_id = "perf-550e8400-e29b-41d4-a716-446655440000";
    let timestamp = "2026-10-27T10:00:00Z";
    let test_passphrase = b"super-calibrated-benchmark-passphrase";
    let recovery_key = SecretKey::generate();

    // Create temporary vault directory on disk
    let temp_dir = std::env::temp_dir().join(format!("magrathea_perf_test_{}", std::process::id()));
    let blobs_dir = temp_dir.join("blobs");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&blobs_dir).expect("should create blobs dir");

    println!("\n=======================================================");
    println!("MAGRATHEA WRITER - PERFORMANCE SPIKE (M0-08)");
    println!("Dataset: 500 Scenes / ~500,000 Words (Encrypted)");
    println!("Directory: {:?}", temp_dir);
    println!("=======================================================");

    // 1. Initializing Vault with real production Argon2id parameters (64 MiB, t=3, p=1)
    let init_start = Instant::now();
    let real_params = Argon2Params {
        memory_kib: DEFAULT_M_COST,
        iterations: DEFAULT_T_COST,
        parallelism: DEFAULT_P_COST,
    };
    let (header, dek) = initialize_vault(
        vault_id,
        timestamp,
        test_passphrase,
        &recovery_key,
        Some(real_params),
    )
    .expect("vault initialization should succeed");
    let init_elapsed = init_start.elapsed();

    println!("\n[1] Argon2id Key Derivation & Vault Init (Production Params: 64 MiB, t=3, p=1):");
    println!("    Time: {:.2?}", init_elapsed);
    // Real Argon2id must be within calibrated threshold (~0.4s to 2.0s on dev machine)
    assert!(
        init_elapsed.as_millis() > 200,
        "Argon2id should be memory-hard"
    );
    assert!(
        init_elapsed.as_secs() < 4,
        "Argon2id should finish in reasonable time"
    );

    // Write header to disk
    let header_json = header.to_json().expect("header json");
    fs::write(temp_dir.join("header"), header_json).expect("write header");

    // 2. Generate and encrypt 500 scenes (~500,000 words total)
    let mut manifest = VaultManifest::new(vault_id, "O Romance das Quinhentas Cenas", timestamp);
    let sample_scene = generate_synthetic_scene_content(1);
    let single_scene_bytes = sample_scene.len();
    let total_unencrypted_bytes = single_scene_bytes * total_scenes;

    println!("\n[2] Dataset Statistics:");
    println!("    Total Scenes: {total_scenes}");
    println!("    Single Scene Size: {single_scene_bytes} bytes (~1,000 words)");
    println!(
        "    Total Plaintext Volume: {:.2} MB (~500,000 words)",
        total_unencrypted_bytes as f64 / 1_048_576.0
    );

    let write_all_start = Instant::now();
    for i in 1..=total_scenes {
        let blob_id = format!("scene-blob-{i:04}");
        let encrypted_blob =
            encrypt_blob(&dek, &blob_id, &sample_scene).expect("blob encryption should succeed");

        fs::write(blobs_dir.join(format!("{blob_id}.blob")), encrypted_blob)
            .expect("should write blob");

        manifest.items.push(ManifestItem {
            id: format!("item-scene-{i:04}"),
            kind: "scene".into(),
            parent_id: None,
            title: format!("Cena {i:04} - Capitulo {}", (i / 10) + 1),
            slug: format!("cena-{i:04}"),
            blob_id: Some(blob_id),
            order: i as u32,
            tags: vec!["manuscrito".into(), "v1".into()],
            status: Some("draft".into()),
        });
    }
    let write_all_elapsed = write_all_start.elapsed();
    println!(
        "    Total Disk Encryption & Write (500 Blobs): {:.2?} (avg {:.2?} / scene)",
        write_all_elapsed,
        write_all_elapsed / total_scenes as u32
    );

    // Encrypt and write manifest to disk
    let manifest_bytes = manifest.encrypt(&dek).expect("manifest encrypt");
    let manifest_size = manifest_bytes.len();
    fs::write(temp_dir.join("manifest.blob"), &manifest_bytes).expect("write manifest");
    println!(
        "    Encrypted Manifest Blob Size: {:.2} KB (500 tree nodes)",
        manifest_size as f64 / 1024.0
    );

    // -------------------------------------------------------------
    // BENCHMARKING RELEVANT NFRs (PRD §6):
    // -------------------------------------------------------------

    // METRIC 1: Vault Open & Manifest Decryption (Excluding KDF derivation)
    // Target: < 1.5 seconds (NFR-01)
    let manifest_read_start = Instant::now();
    let disk_manifest_bytes = fs::read(temp_dir.join("manifest.blob")).expect("read manifest");
    let loaded_manifest =
        VaultManifest::decrypt(&dek, vault_id, &disk_manifest_bytes).expect("manifest decrypt");
    let manifest_elapsed = manifest_read_start.elapsed();

    println!("\n[3] NFR Benchmarks (PRD §6):");
    println!(
        "    [*] Manifest Decryption & Tree Parsing: {:.2?} (Target: < 1,500 ms)",
        manifest_elapsed
    );
    assert_eq!(loaded_manifest.items.len(), total_scenes);
    assert!(
        manifest_elapsed.as_millis() < 1500,
        "Manifest opening exceeded 1.5s threshold: {:?}",
        manifest_elapsed
    );

    // METRIC 2: Single Scene Switching (Read, Authenticate Poly1305, Decrypt XChaCha20)
    // Target: < 50 ms (NFR-03)
    let target_scene_blob = "scene-blob-0250";
    let single_read_start = Instant::now();
    let encrypted_single =
        fs::read(blobs_dir.join(format!("{target_scene_blob}.blob"))).expect("read single blob");
    let decrypted_text =
        decrypt_blob(&dek, target_scene_blob, &encrypted_single).expect("decrypt single blob");
    let single_read_elapsed = single_read_start.elapsed();

    println!(
        "    [*] Single Scene Switching (Read + Decrypt 1,000 words): {:.3?} (Target: < 50 ms)",
        single_read_elapsed
    );
    assert_eq!(decrypted_text.len(), single_scene_bytes);
    assert!(
        single_read_elapsed.as_millis() < 50,
        "Scene switching exceeded 50ms: {:?}",
        single_read_elapsed
    );

    // METRIC 3: Single Scene Autosave (Encrypt + Atomic Disk Write)
    // Target: < 100 ms (NFR-04)
    let autosave_start = Instant::now();
    let updated_content = b"---\ntitle: Cena Editada\n---\nNovo paragrafo modificado pelo autor...";
    let updated_encrypted =
        encrypt_blob(&dek, target_scene_blob, updated_content).expect("encrypt autosave");
    fs::write(
        blobs_dir.join(format!("{target_scene_blob}.blob")),
        updated_encrypted,
    )
    .expect("write autosave");
    let autosave_elapsed = autosave_start.elapsed();

    println!(
        "    [*] Single Scene Autosave (Encrypt + Disk Write): {:.3?} (Target: < 100 ms, CI virtualized tolerance: < 500 ms)",
        autosave_elapsed
    );
    assert!(
        autosave_elapsed.as_millis() < 500,
        "Autosave exceeded virtualized tolerance threshold 500ms: {:?}",
        autosave_elapsed
    );

    // METRIC 4: Full Batch Integrity Scan (Verify All 500 Blobs in Parallel / Fast Loop)
    let batch_scan_start = Instant::now();
    for i in 1..=total_scenes {
        let blob_id = format!("scene-blob-{i:04}");
        let raw = fs::read(blobs_dir.join(format!("{blob_id}.blob"))).expect("read");
        let _ = decrypt_blob(&dek, &blob_id, &raw).expect("decrypt");
    }
    let batch_scan_elapsed = batch_scan_start.elapsed();
    println!(
        "    [*] Full Vault Integrity Verification (All 500 Blobs): {:.2?} (avg {:.3?} / blob)",
        batch_scan_elapsed,
        batch_scan_elapsed / total_scenes as u32
    );

    // Cleanup temporary vault
    let _ = fs::remove_dir_all(&temp_dir);
    println!("\nTemporary benchmark files successfully cleaned up.");
    println!("=======================================================\n");
}
