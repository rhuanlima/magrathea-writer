//! Testes abrangentes de ciclo de vida, persistência e integridade do cofre (M1-01).
//!
//! Valida:
//! - Criação atômica do cofre (`header`, `manifest.blob`, `blobs/`).
//! - Desbloqueio por frase-senha (Slot 0) e chave de recuperação (Slot 1).
//! - Bloqueio de cofre com descarte seguro e zeroização de chave.
//! - Leitura, escrita e exclusão atômica de cenas cifradas (`blobs/<id>.blob`).
//! - Detecção de adulteração de bits e ataque de troca de blobs (anti-blob-swap AAD).
//! - Detecção de ataque de rollback de versão monotônica do manifesto.
//! - Varredura de integridade (`verify_integrity`) para blobs corrompidos, ausentes e órfãos.
//! - Formatação e parsing de tokens de chave de recuperação.

use magrathea_writer_lib::crypto::{
    Argon2Params, SecretKey, DEFAULT_M_COST, DEFAULT_P_COST, DEFAULT_T_COST,
};
use magrathea_writer_lib::vault::{ManifestItem, VaultError, VaultStorage};
use std::fs;
use std::path::PathBuf;

/// Helper para criar diretório temporário isolado por teste.
fn create_test_temp_dir(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let random_id: u64 = rand::random();
    dir.push(format!("magrathea_m1_test_{test_name}_{random_id}"));
    fs::create_dir_all(&dir).expect("falha ao criar diretório temporário de teste");
    dir
}

/// Helper para limpar diretório temporário após o teste.
fn cleanup_test_dir(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

/// Helper com parâmetros rápidos de KDF para acelerar a suíte de testes unitários.
fn fast_test_kdf_params() -> Argon2Params {
    Argon2Params {
        memory_kib: DEFAULT_M_COST, // 64 MiB (mínimo obrigatório contra downgrade)
        iterations: DEFAULT_T_COST, // 3 iterações (mínimo obrigatório)
        parallelism: DEFAULT_P_COST,
    }
}

#[test]
fn test_vault_create_and_directory_layout() {
    let temp_root = create_test_temp_dir("layout");
    let vault_path = temp_root.join("MeuRomance.magrathea");
    let passphrase = b"senha-mestra-literaria-segura";
    let recovery_key = SecretKey::generate();

    // 1. Criação do cofre
    let vault = VaultStorage::create(
        &vault_path,
        "Meu Romance",
        passphrase,
        &recovery_key,
        Some(fast_test_kdf_params()),
    )
    .expect("falha ao criar cofre");

    assert!(!vault.is_locked());
    assert_eq!(vault.header().magic, "MAGRATHEA");
    assert_eq!(vault.header().format_version, 1);
    assert!(!vault.header().vault_id.is_empty());

    // 2. Validação da estrutura em disco
    assert!(vault_path.exists());
    assert!(vault_path.join("header").exists());
    assert!(vault_path.join("manifest.blob").exists());
    assert!(vault_path.join("blobs").is_dir());

    // 3. Validação do manifesto inicial
    let manifest = vault.manifest().expect("manifesto deve estar acessível");
    assert_eq!(manifest.project_title, "Meu Romance");
    assert_eq!(manifest.monotonic_version, 1);
    assert!(manifest.items.is_empty());

    cleanup_test_dir(&temp_root);
}

#[test]
fn test_vault_unlock_with_passphrase_and_recovery_key() {
    let temp_root = create_test_temp_dir("unlock");
    let vault_path = temp_root.join("Projeto.magrathea");
    let passphrase = b"passphrase-teste-alpha-123";
    let recovery_key = SecretKey::generate();

    // Cria o cofre
    VaultStorage::create(
        &vault_path,
        "Obra Teste",
        passphrase,
        &recovery_key,
        Some(fast_test_kdf_params()),
    )
    .expect("falha ao criar cofre");

    // 1. Desbloqueio com frase-senha correta
    let vault_by_pass = VaultStorage::open_with_passphrase(&vault_path, passphrase)
        .expect("deve abrir com senha correta");
    assert!(!vault_by_pass.is_locked());
    assert_eq!(
        vault_by_pass.manifest().unwrap().project_title,
        "Obra Teste"
    );

    // 2. Falha de desbloqueio com frase-senha incorreta
    let wrong_pass_result =
        VaultStorage::open_with_passphrase(&vault_path, b"senha-completamente-errada");
    assert!(wrong_pass_result.is_err());
    match wrong_pass_result {
        Err(VaultError::Crypto(_)) => (),
        other => panic!("esperava VaultError::Crypto, obteve {other:?}"),
    }

    // 3. Desbloqueio com chave de recuperação correta (Slot 1)
    let vault_by_rec = VaultStorage::open_with_recovery_key(&vault_path, &recovery_key)
        .expect("deve abrir com chave de recuperação");
    assert!(!vault_by_rec.is_locked());
    assert_eq!(vault_by_rec.manifest().unwrap().project_title, "Obra Teste");

    // 4. Falha com chave de recuperação incorreta
    let wrong_rec_key = SecretKey::generate();
    let wrong_rec_result = VaultStorage::open_with_recovery_key(&vault_path, &wrong_rec_key);
    assert!(wrong_rec_result.is_err());
    match wrong_rec_result {
        Err(VaultError::Crypto(_)) => (),
        other => panic!("esperava VaultError::Crypto, obteve {other:?}"),
    }

    cleanup_test_dir(&temp_root);
}

#[test]
fn test_vault_lock_and_zeroization() {
    let temp_root = create_test_temp_dir("lock");
    let vault_path = temp_root.join("Livro.magrathea");
    let passphrase = b"chave-de-acesso-autor";
    let recovery_key = SecretKey::generate();

    let mut vault = VaultStorage::create(
        &vault_path,
        "Livro Secreto",
        passphrase,
        &recovery_key,
        Some(fast_test_kdf_params()),
    )
    .expect("falha ao criar cofre");

    // Escreve uma cena antes de bloquear
    let scene_id = "cena-001-id";
    vault
        .write_blob(scene_id, b"Conteudo confidencial em memoria")
        .expect("falha ao gravar cena");

    assert!(!vault.is_locked());
    assert!(vault.manifest().is_ok());

    // Bloqueia o cofre explicitamente
    vault.lock();

    // Verifica que todas as operações falham com VaultError::Locked
    assert!(vault.is_locked());
    assert_eq!(vault.manifest().unwrap_err(), VaultError::Locked);
    assert_eq!(vault.manifest_mut().unwrap_err(), VaultError::Locked);
    assert_eq!(vault.read_blob(scene_id).unwrap_err(), VaultError::Locked);
    assert_eq!(
        vault.write_blob(scene_id, b"tentativa").unwrap_err(),
        VaultError::Locked
    );
    assert_eq!(vault.delete_blob(scene_id).unwrap_err(), VaultError::Locked);
    assert_eq!(
        vault.save_manifest("2026-10-27T12:00:00Z").unwrap_err(),
        VaultError::Locked
    );
    assert_eq!(vault.verify_integrity().unwrap_err(), VaultError::Locked);

    cleanup_test_dir(&temp_root);
}

#[test]
fn test_scene_blob_crud_and_disk_opacity() {
    let temp_root = create_test_temp_dir("crud");
    let vault_path = temp_root.join("Manuscrito.magrathea");
    let passphrase = b"senha-para-manuscrito";
    let recovery_key = SecretKey::generate();

    let vault = VaultStorage::create(
        &vault_path,
        "Manuscrito",
        passphrase,
        &recovery_key,
        Some(fast_test_kdf_params()),
    )
    .expect("falha ao criar cofre");

    let blob_id = "e2a1b3c4-1111-2222-3333-444455556666";
    let scene_text = b"# Capitulo 1\n\nEra uma tarde nublada na estacao central.";

    // 1. Gravação do blob cifrado
    vault
        .write_blob(blob_id, scene_text)
        .expect("falha ao escrever blob");

    // 2. Leitura e decifragem correta
    let read_back = vault
        .read_blob(blob_id)
        .expect("falha ao decifrar blob gravado");
    assert_eq!(read_back, scene_text);

    // 3. Garantia de opacidade em disco: o arquivo gravado NAO pode conter o texto em claro
    let blob_file_path = vault_path.join("blobs").join(format!("{blob_id}.blob"));
    assert!(blob_file_path.exists());
    let raw_disk_bytes = fs::read(&blob_file_path).expect("falha ao ler blob bruto do disco");

    // Garante que o texto em claro não aparece em nenhum trecho do arquivo em disco
    assert!(
        !raw_disk_bytes
            .windows(b"tarde nublada".len())
            .any(|w| w == b"tarde nublada"),
        "Vazamento de dados: texto em claro encontrado no arquivo de blob em disco!"
    );

    // 4. Exclusão do blob
    vault.delete_blob(blob_id).expect("falha ao deletar blob");
    assert!(!blob_file_path.exists());

    // Leitura após exclusão retorna MissingBlob
    let missing_res = vault.read_blob(blob_id);
    assert_eq!(
        missing_res.unwrap_err(),
        VaultError::MissingBlob(blob_id.to_string())
    );

    cleanup_test_dir(&temp_root);
}

#[test]
fn test_anti_blob_swap_aad_binding() {
    let temp_root = create_test_temp_dir("blob_swap");
    let vault_path = temp_root.join("SwapTest.magrathea");
    let passphrase = b"senha-teste-aad";
    let recovery_key = SecretKey::generate();

    let vault = VaultStorage::create(
        &vault_path,
        "Swap Project",
        passphrase,
        &recovery_key,
        Some(fast_test_kdf_params()),
    )
    .expect("falha ao criar cofre");

    let blob_a = "blob-uuid-aaaa-1111";
    let blob_b = "blob-uuid-bbbb-2222";

    vault
        .write_blob(blob_a, b"Texto do Capitulo A")
        .expect("falha ao gravar blob A");
    vault
        .write_blob(blob_b, b"Texto do Capitulo B")
        .expect("falha ao gravar blob B");

    let path_a = vault_path.join("blobs").join(format!("{blob_a}.blob"));
    let path_b = vault_path.join("blobs").join(format!("{blob_b}.blob"));

    // Simula ataque externo: copia o conteúdo do blob A por cima do arquivo do blob B
    let bytes_a = fs::read(&path_a).expect("falha ao ler blob A");
    fs::write(&path_b, &bytes_a).expect("falha ao sobrescrever blob B");

    // A leitura de B deve FALHAR porque o AAD gravado no Poly1305 de A é "MAGRATHEA-V1:blob:blob-uuid-aaaa-1111"
    // e a leitura de B tenta autenticar com "MAGRATHEA-V1:blob:blob-uuid-bbbb-2222"
    let tampered_read = vault.read_blob(blob_b);
    assert!(
        tampered_read.is_err(),
        "Ataque de troca de blob deveria ter falhado por mismatch de AAD!"
    );
    match tampered_read {
        Err(VaultError::Crypto(_)) => (),
        other => panic!("esperava VaultError::Crypto, obteve {other:?}"),
    }

    cleanup_test_dir(&temp_root);
}

#[test]
fn test_bit_flip_payload_tampering() {
    let temp_root = create_test_temp_dir("bit_flip");
    let vault_path = temp_root.join("BitFlip.magrathea");
    let passphrase = b"senha-teste-bitflip";
    let recovery_key = SecretKey::generate();

    let vault = VaultStorage::create(
        &vault_path,
        "BitFlip Project",
        passphrase,
        &recovery_key,
        Some(fast_test_kdf_params()),
    )
    .expect("falha ao criar cofre");

    let blob_id = "blob-uuid-tamper-test";
    vault
        .write_blob(blob_id, b"Conteudo integro original")
        .expect("falha ao gravar blob");

    let path = vault_path.join("blobs").join(format!("{blob_id}.blob"));
    let mut bytes = fs::read(&path).expect("falha ao ler blob");

    // Inverte um único bit no meio do ciphertext
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x01;
    fs::write(&path, &bytes).expect("falha ao gravar blob adulterado");

    // Tentativa de leitura deve ser rejeitada pela verificação de tag Poly1305
    let tampered_read = vault.read_blob(blob_id);
    assert!(
        tampered_read.is_err(),
        "Leitura de blob adulterado deveria ter sido rejeitada!"
    );

    cleanup_test_dir(&temp_root);
}

#[test]
fn test_manifest_rollback_detection() {
    let temp_root = create_test_temp_dir("rollback");
    let vault_path = temp_root.join("RollbackTest.magrathea");
    let passphrase = b"senha-teste-rollback";
    let recovery_key = SecretKey::generate();

    let mut vault = VaultStorage::create(
        &vault_path,
        "Rollback Project",
        passphrase,
        &recovery_key,
        Some(fast_test_kdf_params()),
    )
    .expect("falha ao criar cofre");

    assert_eq!(vault.manifest().unwrap().monotonic_version, 1);

    // Salva atualização 1 -> monotonic_version passa a 2
    vault
        .save_manifest("2026-10-27T12:05:00Z")
        .expect("falha ao salvar manifesto");
    assert_eq!(vault.manifest().unwrap().monotonic_version, 2);

    // Salva atualização 2 -> monotonic_version passa a 3
    vault
        .save_manifest("2026-10-27T12:10:00Z")
        .expect("falha ao salvar manifesto");
    assert_eq!(vault.manifest().unwrap().monotonic_version, 3);

    // Simula ataque de rollback: um agente externo substitui manifest.blob por um arquivo
    // que possui versão superior simulada (ex: version 10), representando uma sessão divergente
    let manifest_path = vault_path.join("manifest.blob");
    let mut fake_manifest = vault.manifest().unwrap().clone();
    fake_manifest.monotonic_version = 10;
    let dek = vault.header().unlock_with_passphrase(passphrase).unwrap();
    let fake_bytes = fake_manifest.encrypt(&dek).unwrap();
    fs::write(&manifest_path, &fake_bytes).expect("falha ao simular ataque");

    // Quando o cofre tenta salvar o manifesto com sua versão local (3), deve detectar o conflito de versão
    let rollback_result = vault.save_manifest("2026-10-27T12:15:00Z");
    assert!(rollback_result.is_err());
    match rollback_result {
        Err(VaultError::RollbackDetected {
            expected_min,
            on_disk,
        }) => {
            assert_eq!(expected_min, 3);
            assert_eq!(on_disk, 10);
        }
        other => panic!("esperava VaultError::RollbackDetected, obteve {other:?}"),
    }

    cleanup_test_dir(&temp_root);
}

#[test]
fn test_verify_integrity_comprehensive() {
    let temp_root = create_test_temp_dir("integrity");
    let vault_path = temp_root.join("IntegrityTest.magrathea");
    let passphrase = b"senha-teste-integridade";
    let recovery_key = SecretKey::generate();

    let mut vault = VaultStorage::create(
        &vault_path,
        "Integrity Project",
        passphrase,
        &recovery_key,
        Some(fast_test_kdf_params()),
    )
    .expect("falha ao criar cofre");

    let blob_ok = "blob-valido-1";
    let blob_corrupted = "blob-corrompido-2";
    let blob_missing = "blob-ausente-3";
    let blob_orphan = "blob-orfao-4";

    // 1. Grava os blobs necessários
    vault
        .write_blob(blob_ok, b"Cena 1 ok")
        .expect("falha blob ok");
    vault
        .write_blob(blob_corrupted, b"Cena 2 integro inicialmente")
        .expect("falha blob corrompido");
    vault
        .write_blob(blob_orphan, b"Cena 4 nao registrada no manifesto")
        .expect("falha blob orfao");

    // 2. Registra no manifesto apenas os nós correspondentes
    {
        let manifest = vault.manifest_mut().unwrap();
        manifest.items.push(ManifestItem {
            id: "node-1".to_string(),
            kind: "scene".to_string(),
            parent_id: None,
            title: "Cena 1".to_string(),
            slug: "cena-1".to_string(),
            blob_id: Some(blob_ok.to_string()),
            order: 0,
            tags: vec![],
            status: None,
        });
        manifest.items.push(ManifestItem {
            id: "node-2".to_string(),
            kind: "scene".to_string(),
            parent_id: None,
            title: "Cena 2".to_string(),
            slug: "cena-2".to_string(),
            blob_id: Some(blob_corrupted.to_string()),
            order: 1,
            tags: vec![],
            status: None,
        });
        // Declara o nó 3 com blob que NÃO existe em disco
        manifest.items.push(ManifestItem {
            id: "node-3".to_string(),
            kind: "scene".to_string(),
            parent_id: None,
            title: "Cena 3".to_string(),
            slug: "cena-3".to_string(),
            blob_id: Some(blob_missing.to_string()),
            order: 2,
            tags: vec![],
            status: None,
        });
    }

    // 3. Adultera o arquivo do blob 2 em disco
    let corrupted_path = vault_path
        .join("blobs")
        .join(format!("{blob_corrupted}.blob"));
    let mut bytes = fs::read(&corrupted_path).unwrap();
    bytes[30] ^= 0xff; // Inverte bits
    fs::write(&corrupted_path, &bytes).unwrap();

    // 4. Executa a varredura integral de integridade
    let report = vault.verify_integrity().expect("falha ao rodar varredura");

    assert!(!report.is_clean());
    assert_eq!(report.total_items, 3);
    assert_eq!(report.valid_blobs, 1);
    assert_eq!(report.corrupted_blobs, vec![blob_corrupted.to_string()]);
    assert_eq!(report.missing_blobs, vec![blob_missing.to_string()]);
    assert_eq!(report.orphan_blobs, vec![blob_orphan.to_string()]);

    cleanup_test_dir(&temp_root);
}

#[test]
fn test_recovery_key_tokens_and_formatting() {
    let original_key = SecretKey::generate();

    // 1. Geração de token formatado em blocos (ex: "a1b2c3d4-e5f60718-...")
    let token = original_key.to_formatted_token();
    assert_eq!(token.len(), 64 + 7); // 64 caracteres hex + 7 hífens = 71
    let parts: Vec<&str> = token.split('-').collect();
    assert_eq!(parts.len(), 8);
    for part in parts {
        assert_eq!(part.len(), 8);
    }

    // 2. Parsing a partir do token formatado
    let parsed_from_token =
        SecretKey::from_hex(&token).expect("falha ao fazer parse do token com hifens");
    assert_eq!(original_key.as_bytes(), parsed_from_token.as_bytes());

    // 3. Parsing tolerante a espaços em branco e letras maiúsculas
    let spaced_token = format!("  {}  ", token.to_uppercase());
    let parsed_spaced =
        SecretKey::from_hex(&spaced_token).expect("falha ao fazer parse de chave com espacos");
    assert_eq!(original_key.as_bytes(), parsed_spaced.as_bytes());

    // 4. Rejeição de chave com tamanho inválido
    let invalid_len = "a1b2c3d4-too-short";
    assert!(SecretKey::from_hex(invalid_len).is_err());
}
