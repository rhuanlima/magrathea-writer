//! Teste de segurança e persistência atômica cifrada: M1-04
//!
//! Requisito estrito do PRD (E1):
//! "Escrita atômica cifrada + autosave; nenhum temporário em claro (teste automatizado de varredura de strings)"

use magrathea_writer_lib::commands::{
    create_vault_internal, load_scene_content_internal, save_scene_content_internal,
};
use magrathea_writer_lib::state::AppState;
use std::fs;
use std::path::Path;

/// Varre recursivamente um diretório lendo todos os arquivos e verificando
/// que uma sequência de bytes (canary) NUNCA se encontra presente em claro no disco.
fn assert_no_plaintext_leak(dir: &Path, canary: &[u8]) {
    assert!(
        dir.exists(),
        "O diretório a ser inspecionado deve existir: {:?}",
        dir
    );

    let mut scanned_files = 0;
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if current.is_dir() {
            let entries = fs::read_dir(&current).expect("deve ler entradas do diretório");
            for entry in entries.flatten() {
                stack.push(entry.path());
            }
        } else if current.is_file() {
            scanned_files += 1;
            let file_bytes = fs::read(&current).unwrap_or_else(|e| {
                panic!("falha ao ler bytes do arquivo {:?}: {e}", current);
            });

            // Se o arquivo contiver o canary em claro, isso representa uma falha crítica de segurança
            if file_bytes
                .windows(canary.len())
                .any(|window| window == canary)
            {
                panic!(
                    "VAZAMENTO DE DADOS EM CLARO DETECTADO! O arquivo {:?} contém a string canary em claro!",
                    current
                );
            }
        }
    }

    assert!(
        scanned_files >= 2,
        "Deve ter inspecionado pelo menos o header e manifest.blob, total encontrado: {scanned_files}"
    );
}

#[test]
fn test_atomic_encrypted_write_and_zero_plaintext_leak_canary() {
    let state = AppState::default();
    let unique_id: u64 = rand::random();
    let temp_dir = std::env::temp_dir().join(format!("magrathea_canary_test_{unique_id}"));
    let _ = fs::remove_dir_all(&temp_dir);

    let fixture_passphrase = "frase-segura-canary-test-999";
    let canary_secret_prose =
        "CANARY_SECRET_PROSE_MAGRATHEA_2026_ESTE_E_O_PARAGRAFO_DO_AUTOR_TOTALMENTE_SECRETO";

    // 1. Criação do cofre
    let create_res = create_vault_internal(
        &state,
        &temp_dir.display().to_string(),
        "Obra Secreta",
        fixture_passphrase,
    )
    .expect("deve criar cofre");

    let vault_root = Path::new(&create_res.vault_path);

    // 2. Simulação de múltiplos salvamentos contínuos de autosave (20 iterações atômicas)
    for iteration in 1..=20 {
        let content = format!(
            "{canary_secret_prose}\n\nRevisão número {iteration}: O silêncio da noite escondia os mistérios da galáxia."
        );
        let save_res = save_scene_content_internal(&state, "cena_principal", &content)
            .expect("deve salvar com autosave");
        assert_eq!(save_res.scene_id, "cena_principal");
    }

    // 3. Salva uma segunda cena com outro trecho secreto
    let second_canary = "OUTRO_CANARY_CAPITULO_DOIS_SECREDO_ABSOLUTO_987";
    save_scene_content_internal(&state, "cena_segunda", second_canary)
        .expect("deve salvar segunda cena");

    // 4. Varredura estrita e automatizada de strings no disco em TODO o cofre
    assert_no_plaintext_leak(vault_root, canary_secret_prose.as_bytes());
    assert_no_plaintext_leak(vault_root, second_canary.as_bytes());

    // 5. Verifica que nenhum arquivo temporário (.tmp) residual ficou órfão no disco
    let blobs_dir = vault_root.join("blobs");
    let entries = fs::read_dir(&blobs_dir).expect("deve ler blobs");
    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        assert!(
            !file_name.ends_with(".tmp"),
            "Arquivo temporário residual encontrado em blobs/: {file_name}"
        );
        assert!(
            file_name.ends_with(".blob"),
            "Arquivo inválido encontrado em blobs/: {file_name}"
        );
    }

    // 6. Decifra os dados em memória e confirma a integridade total do conteúdo
    let loaded_first = load_scene_content_internal(&state, "cena_principal")
        .expect("deve carregar primeira cena decifrada");
    assert!(loaded_first.contains(canary_secret_prose));
    assert!(loaded_first.contains("Revisão número 20"));

    let loaded_second = load_scene_content_internal(&state, "cena_segunda")
        .expect("deve carregar segunda cena decifrada");
    assert_eq!(loaded_second, second_canary);

    // Limpeza
    let _ = fs::remove_dir_all(&temp_dir);
}
