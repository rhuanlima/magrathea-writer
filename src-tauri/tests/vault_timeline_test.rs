//! Testes de integração para a Linha do Tempo Literária (M7 / Épico E6).

use magrathea_writer_lib::commands::{
    create_manuscript_item_internal, create_vault_internal, create_world_entity_internal,
    get_vault_timeline_internal, save_scene_content_internal,
};
use magrathea_writer_lib::state::AppState;
use std::fs;

#[test]
fn test_vault_timeline_lifecycle_chronology_and_pacing() {
    let temp_dir =
        std::env::temp_dir().join(format!("magrathea_timeline_test_{}", rand::random::<u64>()));
    fs::create_dir_all(&temp_dir).unwrap();

    let state = AppState::default();
    let initial_pass = "senha_mestra_timeline_segura_1234!";

    // 1. Cria cofre de teste
    let _ = create_vault_internal(
        &state,
        &temp_dir.to_string_lossy(),
        "Crônicas de Alvorada",
        initial_pass,
    )
    .expect("deve criar cofre");

    // 2. Cria entidades de Worldbuilding para checagem cruzada de wikilinks
    let _char_jaci = create_world_entity_internal(
        &state,
        "character",
        "Jaci",
        vec!["protagonista".to_string()],
    )
    .expect("cria Jaci");

    let _loc_porto = create_world_entity_internal(
        &state,
        "location",
        "Porto de Pedra",
        vec!["cidade".to_string()],
    )
    .expect("cria Porto de Pedra");

    // Rebuild index para registrar entidades
    let _ = magrathea_writer_lib::commands::rebuild_vault_index_internal(&state);

    // 3. Cria estrutura do Manuscrito:
    // Parte 1
    //  └── Capítulo 1: O Início
    //       ├── Cena 1 (2026-10-01) - Presente
    //       └── Cena 2 (2026-10-05) - Presente
    //  └── Capítulo 2: Memórias e Futuro
    //       ├── Cena 3 (1999-05-10) - Flashback (analepse)
    //       ├── Cena 4 (2026-10-07) - Retorno ao presente
    //       ├── Cena 5 (2030-01-01) - Flashforward (prolepse)
    //       └── Cena 6 (2026-10-08) - Volta ao presente

    let part1 = create_manuscript_item_internal(&state, "part", None, "Parte 1: Origens")
        .expect("cria parte 1");

    let chap1 =
        create_manuscript_item_internal(&state, "chapter", Some(&part1.id), "Capítulo 1: O Início")
            .expect("cria cap 1");

    let sc1 =
        create_manuscript_item_internal(&state, "scene", Some(&chap1.id), "Cena 1: A Partida")
            .expect("cria cena 1");

    let sc2 =
        create_manuscript_item_internal(&state, "scene", Some(&chap1.id), "Cena 2: O Mar Revolto")
            .expect("cria cena 2");

    let chap2 = create_manuscript_item_internal(
        &state,
        "chapter",
        Some(&part1.id),
        "Capítulo 2: Lembranças",
    )
    .expect("cria cap 2");

    let sc3 = create_manuscript_item_internal(
        &state,
        "scene",
        Some(&chap2.id),
        "Cena 3: A Infância de Jaci",
    )
    .expect("cria cena 3");

    let sc4 = create_manuscript_item_internal(
        &state,
        "scene",
        Some(&chap2.id),
        "Cena 4: Encontro nas Docas",
    )
    .expect("cria cena 4");

    let sc5 = create_manuscript_item_internal(
        &state,
        "scene",
        Some(&chap2.id),
        "Cena 5: Visão do Futuro",
    )
    .expect("cria cena 5");

    let sc6 =
        create_manuscript_item_internal(&state, "scene", Some(&chap2.id), "Cena 6: O Retorno")
            .expect("cria cena 6");

    // 4. Salva conteúdos com datas no frontmatter e menções a personagens/locais
    save_scene_content_internal(
        &state,
        &sc1.id,
        "---\ntitle: \"Cena 1: A Partida\"\ndate: 2026-10-01\nsinopse: Partida de [[Porto de Pedra]].\n---\n[[Jaci]] arrumou seus pertences e olhou para o horizonte de [[Porto de Pedra]].",
    )
    .unwrap();

    save_scene_content_internal(
        &state,
        &sc2.id,
        "---\ntitle: \"Cena 2: O Mar Revolto\"\ndate: 2026-10-05\n---\nAs ondas colidiam violentamente contra a amurada.",
    )
    .unwrap();

    // Cena 3 em 1999 (Flashback!)
    save_scene_content_internal(
        &state,
        &sc3.id,
        "---\ntitle: \"Cena 3: A Infância de Jaci\"\ndate: 1999-05-10\npersonagens: [Jaci, Vovó]\n---\nEm uma manhã serena, vinte e sete anos antes, [[Jaci]] aprendeu a remar.",
    )
    .unwrap();

    // Cena 4 em 2026-10-07
    save_scene_content_internal(
        &state,
        &sc4.id,
        "---\ntitle: \"Cena 4: Encontro nas Docas\"\ndate: 2026-10-07\n---\nO navio aportou suavemente.",
    )
    .unwrap();

    // Cena 5 em 2030 (Flashforward!)
    save_scene_content_internal(
        &state,
        &sc5.id,
        "---\ntitle: \"Cena 5: Visão do Futuro\"\ndate: 2030-01-01\n---\nNos anos vindouros, a glória seria inquestionável.",
    )
    .unwrap();

    // Cena 6 em 2026-10-08 (Volta ao presente da história)
    save_scene_content_internal(
        &state,
        &sc6.id,
        "---\ntitle: \"Cena 6: O Retorno\"\ndate: 2026-10-08\n---\nJaci despertou daquele sonho profético.",
    )
    .unwrap();

    // Rebuild index com os novos conteúdos
    let _ = magrathea_writer_lib::commands::rebuild_vault_index_internal(&state);

    // 5. Executa extração e geração da Linha do Tempo (M7-01 a M7-05)
    let timeline = get_vault_timeline_internal(&state).expect("deve gerar timeline do cofre");

    // Validações M7-01 e M7-02
    assert_eq!(timeline.total_events, 6);
    assert_eq!(timeline.dated_events_count, 6);

    // Validação M7-03: Visão Dupla (Ordem do Livro vs. Ordem Cronológica)
    // Ordem do livro: sc1 -> sc2 -> sc3 -> sc4 -> sc5 -> sc6
    assert_eq!(
        timeline.narrative_order,
        vec![
            sc1.id.clone(),
            sc2.id.clone(),
            sc3.id.clone(),
            sc4.id.clone(),
            sc5.id.clone(),
            sc6.id.clone()
        ]
    );

    // Ordem cronológica da história:
    // sc3 (1999) -> sc1 (2026-10-01) -> sc2 (2026-10-05) -> sc4 (2026-10-07) -> sc6 (2026-10-08) -> sc5 (2030-01-01)
    assert_eq!(
        timeline.chronological_order,
        vec![
            sc3.id.clone(),
            sc1.id.clone(),
            sc2.id.clone(),
            sc4.id.clone(),
            sc6.id.clone(),
            sc5.id.clone()
        ]
    );

    // Detecção de Flashback e Flashforward
    let ev3 = timeline.events.iter().find(|e| e.id == sc3.id).unwrap();
    assert!(
        ev3.is_flashback,
        "Cena 3 (1999) deve ser marcada como flashback"
    );

    let ev5 = timeline.events.iter().find(|e| e.id == sc5.id).unwrap();
    assert!(
        ev5.is_flashforward,
        "Cena 5 (2030) deve ser marcada como flashforward"
    );

    assert_eq!(timeline.total_flashbacks, 1);
    assert_eq!(timeline.total_flashforwards, 1);

    // Validação M7-04: Ligação com personagens e locais
    let ev1 = timeline.events.iter().find(|e| e.id == sc1.id).unwrap();
    assert!(ev1.characters.contains(&"Jaci".to_string()));
    assert!(ev1.locations.contains(&"Porto de Pedra".to_string()));
    assert_eq!(
        ev1.summary.as_deref(),
        Some("Partida de [[Porto de Pedra]].")
    );

    assert!(timeline.all_characters.contains(&"Jaci".to_string()));
    assert!(timeline
        .all_locations
        .contains(&"Porto de Pedra".to_string()));

    // Validação M7-05: Distribuição e Ritmo por Capítulo
    assert_eq!(timeline.chapter_pacing.len(), 2);

    let cap1_pacing = timeline
        .chapter_pacing
        .iter()
        .find(|cp| cp.chapter_id == chap1.id)
        .unwrap();
    assert_eq!(cap1_pacing.scene_count, 2);
    assert_eq!(cap1_pacing.start_time.as_deref(), Some("2026-10-01"));
    assert_eq!(cap1_pacing.end_time.as_deref(), Some("2026-10-05"));

    let cap2_pacing = timeline
        .chapter_pacing
        .iter()
        .find(|cp| cp.chapter_id == chap2.id)
        .unwrap();
    assert_eq!(cap2_pacing.scene_count, 4);

    // Limpeza
    let _ = fs::remove_dir_all(&temp_dir);
}
