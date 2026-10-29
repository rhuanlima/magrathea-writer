//! Testes de integração do Compilador de Manuscrito e Motores de Exportação (M6).

use magrathea_writer_lib::commands::*;
use magrathea_writer_lib::compiler::CompilerSettings;
use magrathea_writer_lib::state::AppState;
use std::fs;

#[test]
fn test_compiler_and_export_lifecycle() {
    let state = AppState::default();
    let unique_suffix: u64 = rand::random();
    let temp_dir = std::env::temp_dir().join(format!("test_compiler_{unique_suffix}"));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let passphrase = "correto-cavalo-bateria-grampo-compiler";

    // 1. Cria cofre
    let create_res = create_vault_internal(
        &state,
        &temp_dir.display().to_string(),
        "O Guia das Galaxias",
        passphrase,
    )
    .expect("deve criar cofre");

    assert_eq!(create_res.project_title, "O Guia das Galaxias");

    // 2. Cria itens no manuscrito (Parte -> Capítulo -> Cenas)
    let part_res = create_manuscript_item_internal(&state, "part", None, "Parte I: A Partida")
        .expect("deve criar parte");
    let chapter_res = create_manuscript_item_internal(
        &state,
        "chapter",
        Some(&part_res.id),
        "Capitulo 1: O Fim da Terra",
    )
    .expect("deve criar capitulo");
    let scene1_res = create_manuscript_item_internal(
        &state,
        "scene",
        Some(&chapter_res.id),
        "Cena 1: As Casas Amarelas",
    )
    .expect("deve criar cena 1");

    save_scene_content_internal(
        &state,
        &scene1_res.id,
        "---\nstatus: draft\n---\nArthur Dent acordou em uma quinta-feira fria e cinzenta.",
    )
    .expect("deve salvar cena 1");

    let scene2_res =
        create_manuscript_item_internal(&state, "scene", Some(&chapter_res.id), "Cena 2: O Trator")
            .expect("deve criar cena 2");

    save_scene_content_internal(
        &state,
        &scene2_res.id,
        "Um grande trator amarelo avançava sobre sua casa de campo.",
    )
    .expect("deve salvar cena 2");

    // Cria entidade de mundo para testar no dossiê/export
    create_world_entity_internal(
        &state,
        "character",
        "Arthur Dent",
        vec!["protagonista".to_string()],
    )
    .expect("deve criar entidade");

    // 3. Configura Dedicatória, Epígrafe, Referências e Sobre o Autor
    let mut settings = CompilerSettings::default();
    settings.title = Some("O Guia das Galáxias".to_string());
    settings.subtitle = Some("Edição Comemorativa".to_string());
    settings.author = Some("Douglas Adams".to_string());
    settings.dedication =
        Some("Para todos os mochileiros que nunca esquecem sua toalha.".to_string());
    settings.epigraph = Some("O tempo é uma ilusão. A hora do almoço duplamente.".to_string());
    settings.epigraph_source = Some("Ford Prefect".to_string());
    settings.afterword =
        Some("E assim termina o primeiro volume da trilogia de cinco.".to_string());
    settings.references = Some("Arquivos da Enciclopédia Galáctica e Guia Mochileiro.".to_string());
    settings.about_author =
        Some("Escritor e pensador britânico apaixonado por toalhas.".to_string());
    settings.include_scene_titles = true;

    save_compiler_settings_internal(&state, &settings).expect("deve salvar configurações");

    let loaded_settings = load_compiler_settings_internal(&state).expect("deve carregar");
    assert_eq!(
        loaded_settings.dedication.as_deref(),
        Some("Para todos os mochileiros que nunca esquecem sua toalha.")
    );

    // 4. Compilação do Manuscrito em memória
    let compiled = compile_manuscript_internal(&state, Some(settings.clone()))
        .expect("deve compilar manuscrito");

    assert!(compiled.total_words > 0);
    assert!(compiled.full_text.contains("O Guia das Galáxias"));
    assert!(compiled.full_text.contains("Dedicatória"));
    assert!(compiled.full_text.contains("Arthur Dent acordou"));
    assert!(compiled.full_text.contains("Um grande trator"));
    assert!(compiled.full_text.contains("Referências e Leituras"));

    // 5. M6-01: Exportar Markdown aberto (Obsidian)
    let export_md_dir = temp_dir.join("export_obsidian");
    let export_md_res =
        export_obsidian_markdown_internal(&state, &export_md_dir.display().to_string())
            .expect("deve exportar markdown aberto");

    assert!(export_md_res.files_count >= 3);
    assert!(export_md_dir.join("LIVRO_COMPILADO.md").exists());
    assert!(export_md_dir.join("manuscrito").exists());
    assert!(export_md_dir.join("mundo").exists());

    // 6. M6-06: Exportar DOCX
    let docx_file = temp_dir.join("livro.docx");
    let docx_res = export_docx_internal(
        &state,
        &docx_file.display().to_string(),
        Some(settings.clone()),
    )
    .expect("deve exportar docx");

    assert_eq!(docx_res.format, "docx");
    assert!(docx_res.byte_size > 500);
    assert!(docx_file.exists());

    // 7. M6-07: Exportar EPUB
    let epub_file = temp_dir.join("livro.epub");
    let epub_res = export_epub_internal(
        &state,
        &epub_file.display().to_string(),
        Some(settings.clone()),
    )
    .expect("deve exportar epub");

    assert_eq!(epub_res.format, "epub");
    assert!(epub_res.byte_size > 500);
    assert!(epub_file.exists());

    // 8. M6-05: Exportar PDF Nativo (.pdf)
    let pdf_direct_file = temp_dir.join("livro_nativo.pdf");
    let pdf_res = export_pdf_html_internal(
        &state,
        &pdf_direct_file.display().to_string(),
        Some(settings.clone()),
    )
    .expect("deve exportar pdf nativo");

    assert_eq!(pdf_res.format, "pdf");
    assert!(pdf_direct_file.exists());
    let pdf_bytes = fs::read(&pdf_direct_file).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-1.4"));
    assert!(pdf_bytes.len() > 800);

    // 8.1. Exportar HTML para compatibilidade com Print CSS
    let pdf_html_file = temp_dir.join("livro_diagramado.html");
    let html_res = export_pdf_html_internal(
        &state,
        &pdf_html_file.display().to_string(),
        Some(settings.clone()),
    )
    .expect("deve exportar pdf html");

    assert_eq!(html_res.format, "html_pdf");
    assert!(pdf_html_file.exists());
    let html_content = fs::read_to_string(&pdf_html_file).unwrap();
    assert!(html_content.contains("dedication-box"));
    assert!(html_content.contains("@page"));

    // 9. M6-08: Exportar TXT + Dossiê
    let txt_dir = temp_dir.join("export_txt");
    let txt_res =
        export_txt_dossier_internal(&state, &txt_dir.display().to_string(), Some(settings))
            .expect("deve exportar txt");

    assert_eq!(txt_res.format, "txt");
    assert!(txt_dir.join("manuscrito_completo.txt").exists());
    assert!(txt_dir.join("dossie_de_criacao.txt").exists());

    let _ = fs::remove_dir_all(temp_dir);
}
