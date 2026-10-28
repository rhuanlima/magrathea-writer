//! Módulo de Linha do Tempo Literária (Timeline), Cronologia e Ritmo Narrativo (Milestone M7 / Épico E6).
//!
//! Fornece:
//! - M7-01: Extração resiliente de `date:`/`data:` (datas reais) e `time:`/`tempo:` (tempos relativos/ficcionais)
//!   com normalização em rank cronológico numérico comparável.
//! - M7-02: Modelo de eventos da timeline com metadados de cenas, personagens, locais e sinopses.
//! - M7-03: Visão dupla comutável: Ordem do Livro (Discurso narrativo / Sjuzhet) vs. Ordem Cronológica (Tempo da história / Fabula),
//!   com identificação automática de Flashbacks (analepses) e Flashforwards (prolepses).
//! - M7-04: Ligação de eventos a cenas, locais e personagens com extração cruzada do índice de worldbuilding.
//! - M7-05: Análise de distribuição e densidade de eventos por capítulo (pacing narrativo literário).

use crate::document::{split_frontmatter, Document};
use crate::index::VaultIndex;
use crate::vault::{ManifestItem, UnlockedVault, VaultError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

/// Representa um evento narrativo individual da linha do tempo literária.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineEvent {
    /// ID da cena associada.
    pub id: String,
    /// Título legível da cena.
    pub scene_title: String,
    /// ID do capítulo pai, se houver.
    pub chapter_id: Option<String>,
    /// Título do capítulo pai.
    pub chapter_title: Option<String>,
    /// ID da parte pai, se houver.
    pub part_id: Option<String>,
    /// Título da parte pai.
    pub part_title: Option<String>,
    /// Índice de aparição na ordem do manuscrito (0-based).
    pub book_order: usize,
    /// Valor bruto original informado no frontmatter (ex: "2026-10-28", "Dia 3, Noite", "Ano 42").
    pub raw_time: Option<String>,
    /// Tipo temporal ("real_date", "relative", "undated").
    pub time_type: String,
    /// Chave numérica ordenada cronologicamente.
    pub chronological_rank: i64,
    /// Posição na ordenação cronológica (0-based).
    pub chronological_order: usize,
    /// Indica se o evento é uma analepse (flashback) em relação à narrativa antecedente.
    pub is_flashback: bool,
    /// Indica se o evento é uma prolepse (flashforward) em relação à linha do tempo do enredo.
    pub is_flashforward: bool,
    /// Resumo ou sinopse da cena.
    pub summary: Option<String>,
    /// Lista de personagens participantes ou citados na cena.
    pub characters: Vec<String>,
    /// Lista de locais onde a cena se desenrola ou citados.
    pub locations: Vec<String>,
    /// Status da cena ("draft", "revised", "final", etc.).
    pub status: Option<String>,
    /// Contagem total de palavras no corpo da cena.
    pub word_count: usize,
}

/// Distribuição e ritmo de eventos em um capítulo específico (Pacing Narrativo).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterPacing {
    /// ID do capítulo.
    pub chapter_id: String,
    /// Título do capítulo.
    pub chapter_title: String,
    /// Título da parte pai (se houver).
    pub part_title: Option<String>,
    /// Quantidade de cenas contidas no capítulo.
    pub scene_count: usize,
    /// Total de palavras no capítulo.
    pub word_count: usize,
    /// Primeiro marco temporal registrado no capítulo.
    pub start_time: Option<String>,
    /// Último marco temporal registrado no capítulo.
    pub end_time: Option<String>,
    /// Densidade de eventos calculada (eventos por 1.000 palavras).
    pub event_density: f64,
    /// Personagens presentes ao longo do capítulo.
    pub characters: Vec<String>,
    /// Locais visitados ao longo do capítulo.
    pub locations: Vec<String>,
    /// Observação editorial de pacing (ex: "Ritmo acelerado", "Sem marcos temporais", "Salto temporal").
    pub pacing_note: Option<String>,
}

/// Visão consolidada da linha do tempo do cofre com ambas as ordenações e métricas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VaultTimeline {
    /// Lista completa de eventos estruturados.
    pub events: Vec<TimelineEvent>,
    /// Lista de IDs de cenas dispostos na sequência exata de leitura do livro.
    pub narrative_order: Vec<String>,
    /// Lista de IDs de cenas dispostos em ordem cronológica histórica real.
    pub chronological_order: Vec<String>,
    /// Análise de ritmo e densidade por capítulo.
    pub chapter_pacing: Vec<ChapterPacing>,
    /// Catálogo ordenado de todos os personagens com eventos mapeados.
    pub all_characters: Vec<String>,
    /// Catálogo ordenado de todos os locais com eventos mapeados.
    pub all_locations: Vec<String>,
    /// Total geral de eventos mapeados.
    pub total_events: usize,
    /// Total de cenas com datas/tempos definidos pelo autor.
    pub dated_events_count: usize,
    /// Total de flashbacks identificados.
    pub total_flashbacks: usize,
    /// Total de flashforwards identificados.
    pub total_flashforwards: usize,
}

/// Analisa uma string temporal arbitrária e gera seu tipo e rank cronológico ordenável.
///
/// Suporta:
/// - Datas reais ISO 8601 (ex: "2026-10-28", "2026-10-28 14:30", "1890-04-12", "-0500-01-01").
/// - Datas em formato comum (ex: "28/10/2026", "28-10-2026").
/// - Anos isolados (ex: "1984", "Ano 2049", "Ano 42").
/// - Tempos relativos e ficcionais (ex: "Dia 1", "Dia 2, Noite", "Semana 3", "Mês 4", "Outono").
/// - Cenas não datadas: herdam rank derivado de sua posição na narrativa.
pub fn parse_time_to_rank(raw_opt: Option<&str>, book_index: usize) -> (String, i64) {
    let raw = match raw_opt {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return ("undated".to_string(), (book_index as i64) * 100_000),
    };

    let lower = raw.to_lowercase();

    // 1. Tentar parsear data real no formato ISO (YYYY-MM-DD...) ou DD/MM/YYYY
    if let Some(rank) = try_parse_calendar_date(&lower) {
        return ("real_date".to_string(), rank);
    }

    // 2. Tentar parsear tempo relativo / ficcional (Dia X, Semana Y, Ano Z, etc.)
    if let Some(rank) = try_parse_relative_time(&lower) {
        return ("relative".to_string(), rank);
    }

    // 3. Se houver apenas números isolados no texto
    if let Some(rank) = try_parse_isolated_number(&lower) {
        return ("relative".to_string(), rank);
    }

    // 4. Fallback para texto não-numérico: rank derivado da posição narrativa com leve offset
    ("relative".to_string(), (book_index as i64) * 100_000)
}

/// Tenta converter uma data de calendário (ISO ou brasileira) em um rank numérico ordenável.
fn try_parse_calendar_date(text: &str) -> Option<i64> {
    // Formato ISO: YYYY-MM-DD ou YYYY-MM-DD HH:MM
    let parts: Vec<&str> = text.split([' ', 't', 'T']).collect();
    let date_str = parts[0];
    let time_str = parts.get(1).copied();

    // Subdividido por '-'
    let ymd_hyphen: Vec<&str> = date_str.split('-').collect();
    if ymd_hyphen.len() >= 3 {
        // Pode ser ano negativo: "-0500-01-01"
        let (year, month, day) = if date_str.starts_with('-') && ymd_hyphen.len() == 4 {
            let y: i64 = ymd_hyphen[1].parse().ok()?;
            let m: i64 = ymd_hyphen[2].parse().ok()?;
            let d: i64 = ymd_hyphen[3].parse().ok()?;
            (-y, m, d)
        } else {
            let y: i64 = ymd_hyphen[0].parse().ok()?;
            let m: i64 = ymd_hyphen[1].parse().ok()?;
            let d: i64 = ymd_hyphen[2].parse().ok()?;
            (y, m, d)
        };

        let (hour, minute) = parse_optional_hour_minute(time_str);
        return Some(build_calendar_rank(year, month, day, hour, minute));
    }

    // Subdividido por '/': DD/MM/YYYY
    let slash_parts: Vec<&str> = date_str.split('/').collect();
    if slash_parts.len() == 3 {
        let p0: i64 = slash_parts[0].parse().ok()?;
        let p1: i64 = slash_parts[1].parse().ok()?;
        let p2: i64 = slash_parts[2].parse().ok()?;

        let (year, month, day) = if p2 > 31 {
            // DD/MM/YYYY
            (p2, p1, p0)
        } else if p0 > 31 {
            // YYYY/MM/DD
            (p0, p1, p2)
        } else {
            (p2, p1, p0)
        };

        let (hour, minute) = parse_optional_hour_minute(time_str);
        return Some(build_calendar_rank(year, month, day, hour, minute));
    }

    None
}

/// Extrai horas e minutos de uma string opcional (ex: "14:30" ou "14h").
fn parse_optional_hour_minute(time_str: Option<&str>) -> (i64, i64) {
    if let Some(t) = time_str {
        let clean = t.replace(['h', 'H'], ":");
        let parts: Vec<&str> = clean.split(':').collect();
        if parts.len() >= 2 {
            let h: i64 = parts[0].trim().parse().unwrap_or(0);
            let m: i64 = parts[1].trim().parse().unwrap_or(0);
            return (h, m);
        } else if let Ok(h) = parts[0].trim().parse::<i64>() {
            return (h, 0);
        }
    }
    (0, 0)
}

/// Constrói um rank ordenável absoluto baseado em ano, mês, dia, hora e minuto.
fn build_calendar_rank(year: i64, month: i64, day: i64, hour: i64, minute: i64) -> i64 {
    // 1 ano = 1_000_000_000
    // 1 mês = 10_000_000
    // 1 dia = 100_000
    // 1 hora = 1_000
    // 1 minuto = 1
    year * 1_000_000_000 + month * 10_000_000 + day * 100_000 + hour * 1_000 + minute
}

/// Extrai e pontua termos de tempo relativo em ficção e fantasia.
fn try_parse_relative_time(text: &str) -> Option<i64> {
    let mut total_rank: i64 = 0;
    let mut found_any = false;

    // Detecta anos / eras
    if let Some(pos) = text.find("ano") {
        if let Some(n) = extract_next_number(&text[pos..]) {
            total_rank += n * 1_000_000_000;
            found_any = true;
        }
    } else if let Some(pos) = text.find("year") {
        if let Some(n) = extract_next_number(&text[pos..]) {
            total_rank += n * 1_000_000_000;
            found_any = true;
        }
    }

    // Detecta meses
    if let Some(pos) = text.find("mês").or_else(|| text.find("mes")) {
        if let Some(n) = extract_next_number(&text[pos..]) {
            total_rank += n * 10_000_000;
            found_any = true;
        }
    } else if let Some(pos) = text.find("month") {
        if let Some(n) = extract_next_number(&text[pos..]) {
            total_rank += n * 10_000_000;
            found_any = true;
        }
    }

    // Detecta semanas
    if let Some(pos) = text.find("semana") {
        if let Some(n) = extract_next_number(&text[pos..]) {
            total_rank += n * 7 * 100_000;
            found_any = true;
        }
    } else if let Some(pos) = text.find("week") {
        if let Some(n) = extract_next_number(&text[pos..]) {
            total_rank += n * 7 * 100_000;
            found_any = true;
        }
    }

    // Detecta dias ("dia 5", "day 5", "d+3", "d-2")
    if let Some(pos) = text.find("dia") {
        if let Some(n) = extract_next_number(&text[pos..]) {
            total_rank += n * 100_000;
            found_any = true;
        }
    } else if let Some(pos) = text.find("day") {
        if let Some(n) = extract_next_number(&text[pos..]) {
            total_rank += n * 100_000;
            found_any = true;
        }
    } else if let Some(pos) = text.find("d+") {
        if let Some(n) = extract_next_number(&text[pos + 2..]) {
            total_rank += n * 100_000;
            found_any = true;
        }
    } else if let Some(pos) = text.find("d-") {
        if let Some(n) = extract_next_number(&text[pos + 2..]) {
            total_rank -= n * 100_000;
            found_any = true;
        }
    }

    // Períodos do dia
    if text.contains("madrugada") || text.contains("dawn") {
        total_rank += 4_000;
        found_any = true;
    } else if text.contains("manhã") || text.contains("manha") || text.contains("morning") {
        total_rank += 9_000;
        found_any = true;
    } else if text.contains("meio-dia") || text.contains("noon") {
        total_rank += 12_000;
        found_any = true;
    } else if text.contains("tarde") || text.contains("afternoon") {
        total_rank += 15_000;
        found_any = true;
    } else if text.contains("crepúsculo") || text.contains("dusk") || text.contains("anoitecer") {
        total_rank += 18_000;
        found_any = true;
    } else if text.contains("noite") || text.contains("night") {
        total_rank += 21_000;
        found_any = true;
    }

    // Se detectou horas exatas no texto (ex: "14:30" ou "14h")
    let (h, m) = parse_optional_hour_minute(Some(text));
    if h > 0 || m > 0 {
        total_rank += h * 1_000 + m;
        found_any = true;
    }

    if found_any {
        Some(total_rank)
    } else {
        None
    }
}

/// Extrai o primeiro número inteiro positivo encontrado após uma posição na string.
fn extract_next_number(s: &str) -> Option<i64> {
    let mut num_str = String::new();
    let mut started = false;

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            started = true;
        } else if started {
            break;
        }
    }

    if !num_str.is_empty() {
        num_str.parse().ok()
    } else {
        None
    }
}

/// Tenta encontrar qualquer número isolado presente na string.
fn try_parse_isolated_number(s: &str) -> Option<i64> {
    let mut num_str = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
        } else if !num_str.is_empty() {
            break;
        }
    }
    if !num_str.is_empty() {
        let val: i64 = num_str.parse().ok()?;
        Some(val * 100_000)
    } else {
        None
    }
}

/// Extrai a lista de personagens a partir do frontmatter e dos wikilinks no texto.
fn extract_characters_and_locations(
    doc: &Document,
    index_opt: Option<&VaultIndex>,
) -> (Vec<String>, Vec<String>) {
    let mut characters_set = BTreeSet::new();
    let mut locations_set = BTreeSet::new();

    // 1. Extrai de campos explícitos do frontmatter
    let char_keys = [
        "characters",
        "personagens",
        "character",
        "personagem",
        "pov",
    ];
    for key in char_keys {
        if let Some(val) = doc.extra.get(key) {
            extract_strings_from_yaml_value(val, &mut characters_set);
        }
    }

    let loc_keys = [
        "locations",
        "locais",
        "location",
        "local",
        "cenario",
        "setting",
    ];
    for key in loc_keys {
        if let Some(val) = doc.extra.get(key) {
            extract_strings_from_yaml_value(val, &mut locations_set);
        }
    }

    // 2. Extrai de wikilinks presentes no corpo da cena correlacionados com o índice
    if let Some(index) = index_opt {
        let links = crate::index::extract_wikilinks(&doc.body);
        for link in links {
            if let Some(entity) = index.resolve_wikilink(&link.target) {
                match entity.kind.as_str() {
                    "character" => {
                        characters_set.insert(entity.title.clone());
                    }
                    "location" => {
                        locations_set.insert(entity.title.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    (
        characters_set.into_iter().collect(),
        locations_set.into_iter().collect(),
    )
}

/// Coleta strings de um valor arbitrário YAML (pode ser lista de strings ou string única).
fn extract_strings_from_yaml_value(val: &serde_yaml::Value, set: &mut BTreeSet<String>) {
    match val {
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                if let Some(s) = item.as_str() {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        set.insert(trimmed.to_string());
                    }
                }
            }
        }
        serde_yaml::Value::String(s) => {
            // Pode ser lista separada por vírgula
            for item in s.split(',') {
                let trimmed = item.trim();
                if !trimmed.is_empty() {
                    set.insert(trimmed.to_string());
                }
            }
        }
        _ => {}
    }
}

/// Extrai a sinopse ou primeiro parágrafo significativo da cena.
fn extract_summary(doc: &Document) -> Option<String> {
    // 1. Verifica no frontmatter ("summary", "sinopse", "description")
    for key in ["summary", "sinopse", "description"] {
        if let Some(val) = doc.extra.get(key) {
            if let Some(s) = val.as_str() {
                if !s.trim().is_empty() {
                    return Some(s.trim().to_string());
                }
            }
        }
    }

    // 2. Extrai do primeiro parágrafo da prosa Markdown
    for line in doc.body.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('*') {
            let max_len = 220;
            if trimmed.chars().count() > max_len {
                let truncated: String = trimmed.chars().take(max_len).collect();
                return Some(format!("{truncated}..."));
            }
            return Some(trimmed.to_string());
        }
    }

    None
}

/// Extrai tags inline ocultas de data e tempo diretamente do corpo do texto da cena
/// (ex: `#data_20261027`, `#data_2026-10-27`, `#tempo_manha`, `#date_20261027`).
pub fn extract_inline_time_tag(body: &str) -> Option<String> {
    for word in body.split_whitespace() {
        if let Some(tag) = word.strip_prefix('#') {
            let clean =
                tag.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
            if let Some(val) = clean
                .strip_prefix("data_")
                .or_else(|| clean.strip_prefix("date_"))
            {
                if val.len() == 8 && val.chars().all(|c| c.is_ascii_digit()) {
                    return Some(format!("{}-{}-{}", &val[0..4], &val[4..6], &val[6..8]));
                }
                return Some(val.to_string());
            } else if let Some(val) = clean
                .strip_prefix("tempo_")
                .or_else(|| clean.strip_prefix("time_"))
            {
                return Some(val.replace('_', " "));
            }
        }
    }
    None
}

/// Gera a linha do tempo completa do cofre a partir dos itens do manifesto e seus documentos.
pub fn build_vault_timeline(
    vault: &UnlockedVault,
    index_opt: Option<&VaultIndex>,
) -> Result<VaultTimeline, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }

    let manifest = vault.manifest()?;

    // 1. Identifica hierarquia de partes, capítulos e cenas
    let mut part_titles = HashMap::new();
    let mut chapter_titles = HashMap::new();
    let mut chapter_part_map = HashMap::new();

    for item in &manifest.items {
        if item.kind == "part" {
            part_titles.insert(item.id.clone(), item.title.clone());
        } else if item.kind == "chapter" {
            chapter_titles.insert(item.id.clone(), item.title.clone());
            if let Some(ref p_id) = item.parent_id {
                chapter_part_map.insert(item.id.clone(), p_id.clone());
            }
        }
    }

    // 2. Filtra cenas na sequência da árvore do manuscrito
    let mut scenes_in_order = Vec::new();
    collect_scenes_in_book_order(&manifest.items, None, &mut scenes_in_order);

    let mut raw_events = Vec::new();
    let mut all_chars_set = BTreeSet::new();
    let mut all_locs_set = BTreeSet::new();

    // 3. Processa cada cena em ordem de livro
    for (book_idx, scene_item) in scenes_in_order.iter().enumerate() {
        let (chapter_id, chapter_title, part_id, part_title) =
            resolve_hierarchy(scene_item, &chapter_titles, &part_titles, &chapter_part_map);

        let mut raw_time = None;
        let mut doc = Document::default();
        let mut word_count = 0;

        if let Some(ref blob_id) = scene_item.blob_id {
            if let Ok(raw_bytes) = vault.read_blob(blob_id) {
                let text = String::from_utf8_lossy(&raw_bytes);
                match Document::parse(&text) {
                    Ok(parsed) => {
                        // Extrai date/time
                        raw_time = parsed
                            .date
                            .clone()
                            .or_else(|| parsed.time.clone())
                            .or_else(|| {
                                parsed
                                    .extra
                                    .get("data")
                                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                            })
                            .or_else(|| {
                                parsed
                                    .extra
                                    .get("tempo")
                                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                            });

                        let (_, body) = split_frontmatter(&text);
                        if raw_time.is_none() {
                            raw_time = extract_inline_time_tag(body);
                        }
                        word_count = body.split_whitespace().count();
                        doc = parsed;
                    }
                    Err(_) => {
                        // Resiliência literária: se o YAML contiver erros (ex: dois-pontos sem aspas no título),
                        // extrai marcos temporais linha a linha para não desarmar a linha do tempo.
                        let (yaml_part, body) = split_frontmatter(&text);
                        word_count = body.split_whitespace().count();
                        if let Some(yaml_str) = yaml_part {
                            for line in yaml_str.lines() {
                                let l = line.trim();
                                if let Some(stripped) = l
                                    .strip_prefix("date:")
                                    .or_else(|| l.strip_prefix("data:"))
                                    .or_else(|| l.strip_prefix("time:"))
                                    .or_else(|| l.strip_prefix("tempo:"))
                                {
                                    let val = stripped.trim().trim_matches('"').trim_matches('\'');
                                    if !val.is_empty() {
                                        raw_time = Some(val.to_string());
                                        break;
                                    }
                                }
                            }
                        }
                        if raw_time.is_none() {
                            raw_time = extract_inline_time_tag(body);
                        }
                    }
                }
            }
        }

        let (time_type, chronological_rank) = parse_time_to_rank(raw_time.as_deref(), book_idx);

        let (characters, locations) = extract_characters_and_locations(&doc, index_opt);
        for c in &characters {
            all_chars_set.insert(c.clone());
        }
        for l in &locations {
            all_locs_set.insert(l.clone());
        }

        let summary = extract_summary(&doc);

        raw_events.push(TimelineEvent {
            id: scene_item.id.clone(),
            scene_title: scene_item.title.clone(),
            chapter_id,
            chapter_title,
            part_id,
            part_title,
            book_order: book_idx,
            raw_time,
            time_type,
            chronological_rank,
            chronological_order: 0, // calculado a seguir
            is_flashback: false,    // calculado a seguir
            is_flashforward: false, // calculado a seguir
            summary,
            characters,
            locations,
            status: scene_item.status.clone(),
            word_count,
        });
    }

    // 4. Calcula Ordem Cronológica e índices relativos
    let mut chrono_indices: Vec<usize> = (0..raw_events.len()).collect();
    chrono_indices.sort_by(|&a, &b| {
        let rank_cmp = raw_events[a]
            .chronological_rank
            .cmp(&raw_events[b].chronological_rank);
        if rank_cmp != std::cmp::Ordering::Equal {
            rank_cmp
        } else {
            raw_events[a].book_order.cmp(&raw_events[b].book_order)
        }
    });

    for (pos, &idx) in chrono_indices.iter().enumerate() {
        raw_events[idx].chronological_order = pos;
    }

    // 5. Identifica Flashforwards e Flashbacks ao longo da ordem do livro
    let mut total_flashbacks = 0;
    let mut total_flashforwards = 0;

    // Primeiro passo: identificar flashforwards (prolepses: saltos no futuro que depois retornam)
    for i in 0..raw_events.len() {
        let current_rank = raw_events[i].chronological_rank;
        let is_dated = raw_events[i].time_type != "undated";

        if is_dated && i < raw_events.len() - 1 {
            let next_rank = raw_events[i + 1].chronological_rank;
            let prev_rank = if i > 0 {
                raw_events[i - 1].chronological_rank
            } else {
                current_rank
            };
            if current_rank > prev_rank && next_rank < current_rank && next_rank >= prev_rank {
                raw_events[i].is_flashforward = true;
                total_flashforwards += 1;
            }
        }
    }

    // Segundo passo: identificar flashbacks (analepses: retorno ao passado)
    for i in 0..raw_events.len() {
        let current_rank = raw_events[i].chronological_rank;
        let is_dated = raw_events[i].time_type != "undated";

        if is_dated && !raw_events[i].is_flashforward {
            let prev_was_ff = i > 0 && raw_events[i - 1].is_flashforward;
            let prev_baseline_rank = if i >= 2 && prev_was_ff {
                raw_events[i - 2].chronological_rank
            } else if i > 0 {
                raw_events[i - 1].chronological_rank
            } else {
                current_rank
            };

            if i > 0 && current_rank < prev_baseline_rank {
                raw_events[i].is_flashback = true;
                total_flashbacks += 1;
            }
        }
    }

    // 6. Constrói listas de IDs em ambas as ordenações
    let narrative_order: Vec<String> = raw_events.iter().map(|e| e.id.clone()).collect();
    let chronological_order: Vec<String> = chrono_indices
        .iter()
        .map(|&idx| raw_events[idx].id.clone())
        .collect();

    // 7. Calcula Pacing Narrativo por Capítulo (M7-05)
    let chapter_pacing = calculate_chapter_pacing(&raw_events, &manifest.items);

    let dated_events_count = raw_events
        .iter()
        .filter(|e| e.time_type != "undated")
        .count();

    Ok(VaultTimeline {
        total_events: raw_events.len(),
        dated_events_count,
        total_flashbacks,
        total_flashforwards,
        narrative_order,
        chronological_order,
        chapter_pacing,
        all_characters: all_chars_set.into_iter().collect(),
        all_locations: all_locs_set.into_iter().collect(),
        events: raw_events,
    })
}

/// Varre a árvore do manuscrito recursivamente preservando a ordem ordenada de exibição do autor.
fn collect_scenes_in_book_order<'a>(
    items: &'a [ManifestItem],
    parent_id: Option<&str>,
    scenes: &mut Vec<&'a ManifestItem>,
) {
    let mut children: Vec<&'a ManifestItem> = items
        .iter()
        .filter(|i| i.parent_id.as_deref() == parent_id)
        .collect();

    // Ordenação estável pelo order
    children.sort_by_key(|i| i.order);

    for child in children {
        if child.kind == "scene" {
            scenes.push(child);
        } else {
            // Continua aprofundando para partes e capítulos
            collect_scenes_in_book_order(items, Some(&child.id), scenes);
        }
    }
}

/// Identifica título e IDs de capítulo e parte pai da cena.
fn resolve_hierarchy(
    item: &ManifestItem,
    chapter_titles: &HashMap<String, String>,
    part_titles: &HashMap<String, String>,
    chapter_part_map: &HashMap<String, String>,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    if let Some(ref p_id) = item.parent_id {
        if let Some(ch_title) = chapter_titles.get(p_id) {
            let part_id = chapter_part_map.get(p_id).cloned();
            let part_title = part_id
                .as_ref()
                .and_then(|pid| part_titles.get(pid).cloned());
            return (
                Some(p_id.clone()),
                Some(ch_title.clone()),
                part_id,
                part_title,
            );
        } else if let Some(pt_title) = part_titles.get(p_id) {
            return (None, None, Some(p_id.clone()), Some(pt_title.clone()));
        }
    }
    (None, None, None, None)
}

/// Calcula a densidade de eventos e ritmo de cada capítulo.
fn calculate_chapter_pacing(
    events: &[TimelineEvent],
    manifest_items: &[ManifestItem],
) -> Vec<ChapterPacing> {
    let mut pacing_list = Vec::new();

    // Identifica capítulos existentes no manuscrito
    let chapters: Vec<&ManifestItem> = manifest_items
        .iter()
        .filter(|i| i.kind == "chapter")
        .collect();

    for ch in chapters {
        let ch_events: Vec<&TimelineEvent> = events
            .iter()
            .filter(|e| e.chapter_id.as_deref() == Some(&ch.id))
            .collect();

        let scene_count = ch_events.len();
        let word_count: usize = ch_events.iter().map(|e| e.word_count).sum();

        let mut characters_set = BTreeSet::new();
        let mut locations_set = BTreeSet::new();

        let mut first_time: Option<String> = None;
        let mut last_time: Option<String> = None;

        for ev in &ch_events {
            for c in &ev.characters {
                characters_set.insert(c.clone());
            }
            for l in &ev.locations {
                locations_set.insert(l.clone());
            }
            if let Some(ref t) = ev.raw_time {
                if first_time.is_none() {
                    first_time = Some(t.clone());
                }
                last_time = Some(t.clone());
            }
        }

        // Densidade: eventos a cada 1.000 palavras
        let event_density = if word_count > 0 {
            (scene_count as f64 / word_count as f64) * 1000.0
        } else {
            scene_count as f64
        };

        let part_title = ch_events.first().and_then(|e| e.part_title.clone());

        let pacing_note = if scene_count == 0 {
            Some("Capítulo vazio sem cenas vinculadas.".to_string())
        } else if first_time.is_none() {
            Some("Sem marcos temporais explícitos definidos.".to_string())
        } else if event_density > 2.0 {
            Some("Ritmo acelerado (alta densidade de eventos por palavra).".to_string())
        } else if word_count > 5000 && scene_count <= 2 {
            Some("Ritmo contemplativo (cenas longas e detalhadas).".to_string())
        } else {
            None
        };

        pacing_list.push(ChapterPacing {
            chapter_id: ch.id.clone(),
            chapter_title: ch.title.clone(),
            part_title,
            scene_count,
            word_count,
            start_time: first_time,
            end_time: last_time,
            event_density: (event_density * 100.0).round() / 100.0,
            characters: characters_set.into_iter().collect(),
            locations: locations_set.into_iter().collect(),
            pacing_note,
        });
    }

    pacing_list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_real_dates_and_ranking() {
        let (kind1, rank1) = parse_time_to_rank(Some("2026-10-28"), 0);
        let (kind2, rank2) = parse_time_to_rank(Some("2026-10-29 14:00"), 1);
        let (kind3, rank3) = parse_time_to_rank(Some("1995-05-10"), 2);

        assert_eq!(kind1, "real_date");
        assert_eq!(kind2, "real_date");
        assert_eq!(kind3, "real_date");

        // 2026-10-28 < 2026-10-29
        assert!(rank1 < rank2);
        // 1995-05-10 < 2026-10-28
        assert!(rank3 < rank1);
    }

    #[test]
    fn test_parse_relative_fictional_dates() {
        let (kind1, rank1) = parse_time_to_rank(Some("Dia 1, Manhã"), 0);
        let (kind2, rank2) = parse_time_to_rank(Some("Dia 1, Noite"), 1);
        let (kind3, rank3) = parse_time_to_rank(Some("Dia 2, Tarde"), 2);
        let (kind4, rank4) = parse_time_to_rank(Some("Ano 42, Dia 5"), 3);

        assert_eq!(kind1, "relative");
        assert_eq!(kind2, "relative");
        assert_eq!(kind3, "relative");
        assert_eq!(kind4, "relative");

        // Dia 1 Manhã < Dia 1 Noite < Dia 2 Tarde
        assert!(rank1 < rank2);
        assert!(rank2 < rank3);
        // Ano 42 deve ser muito maior que Dia 2
        assert!(rank3 < rank4);
    }

    #[test]
    fn test_extract_inline_time_tag() {
        let body = "O vento soprava frio. #data_20261027\nA batalha começou.";
        let tag = extract_inline_time_tag(body);
        assert_eq!(tag, Some("2026-10-27".to_string()));

        let body2 = "Ao amanhecer. #tempo_manha todos partiram.";
        let tag2 = extract_inline_time_tag(body2);
        assert_eq!(tag2, Some("manha".to_string()));
    }

    #[test]
    fn test_undated_scenes_fallback() {
        let (kind1, rank1) = parse_time_to_rank(None, 0);
        let (kind2, rank2) = parse_time_to_rank(Some(""), 1);

        assert_eq!(kind1, "undated");
        assert_eq!(kind2, "undated");
        assert!(rank1 < rank2);
    }
}
