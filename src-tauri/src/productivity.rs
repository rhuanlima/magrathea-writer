//! Módulo de metas e métricas de produtividade literária (Épico 8).
//!
//! Os dados de metas e o histórico diário de palavras escritas e tempo de sessão
//! são armazenados de forma estritamente atômica e cifrados no cofre em `productivity.blob`
//! utilizando XChaCha20-Poly1305 com a chave DEK ativa e autenticados com AAD vinculado ao Vault ID.

use crate::crypto::{decrypt_payload, encrypt_payload, Nonce, NONCE_LEN};
use crate::vault::{atomic_write, UnlockedVault, VaultError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Entrada de métricas agregadas por dia.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DailyLogEntry {
    /// Total de palavras escritas e contabilizadas no dia específico.
    pub words_written: u32,
    /// Duração agregada da sessão no dia em segundos.
    pub session_duration_seconds: u32,
}

/// Configurações e registros de metas armazenados no cofre.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductivityConfig {
    /// Meta total de palavras do romance ou projeto literário (ex: 80.000).
    pub project_target_words: u32,
    /// Prazo final estipulado pelo autor em formato ISO (ex: "2026-12-31").
    pub project_deadline: Option<String>,
    /// Meta de produção diária estipulada em palavras (ex: 1.000).
    pub daily_target_words: u32,
    /// Mapa histórico de logs indexado por data ("YYYY-MM-DD").
    pub daily_logs: HashMap<String, DailyLogEntry>,
}

impl Default for ProductivityConfig {
    fn default() -> Self {
        Self {
            project_target_words: 50_000,
            project_deadline: None,
            daily_target_words: 1_000,
            daily_logs: HashMap::new(),
        }
    }
}

/// Visão agregada entregue ao frontend para alimentar o HUD e o Dashboard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductivityOverview {
    /// Meta total de palavras do manuscrito.
    pub project_target_words: u32,
    /// Prazo final configurado.
    pub project_deadline: Option<String>,
    /// Meta diária configurada em palavras.
    pub daily_target_words: u32,
    /// Total de palavras acumuladas nas cenas ativas do manuscrito.
    pub total_manuscript_words: u32,
    /// Palavras escritas hoje.
    pub today_words: u32,
    /// Tempo de sessão ativo hoje em segundos.
    pub today_seconds: u32,
    /// Média móvel diária de palavras nos últimos 7 dias com atividade.
    pub seven_days_avg_words: f64,
    /// Média móvel diária de palavras nos últimos 30 dias com atividade.
    pub thirty_days_avg_words: f64,
    /// Histórico completo de datas para o gráfico do dashboard.
    pub daily_logs: HashMap<String, DailyLogEntry>,
}

/// Carrega a configuração e log de produtividade do cofre cifrado.
///
/// Se o arquivo `productivity.blob` ainda não existir no cofre, retorna a configuração padrão.
pub fn load_productivity_config(vault: &UnlockedVault) -> Result<ProductivityConfig, VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let dek = vault.dek()?;
    let path = vault.root_path().join("productivity.blob");

    if !path.exists() {
        return Ok(ProductivityConfig::default());
    }

    let blob_bytes = fs::read(&path).map_err(|e| VaultError::Io(e.to_string()))?;
    if blob_bytes.len() < NONCE_LEN {
        return Err(VaultError::CorruptedBlob);
    }

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&blob_bytes[..NONCE_LEN]);
    let nonce = Nonce::new(nonce_bytes);
    let ciphertext = &blob_bytes[NONCE_LEN..];

    let aad = format!("MAGRATHEA-V1:productivity:{}", vault.header().vault_id);
    let plaintext = decrypt_payload(dek, &nonce, aad.as_bytes(), ciphertext)?;

    let config: ProductivityConfig =
        serde_json::from_slice(&plaintext).map_err(|e| VaultError::Serialization(e.to_string()))?;

    Ok(config)
}

/// Cifra e persiste as metas e logs no cofre de forma estritamente atômica.
pub fn save_productivity_config(
    vault: &UnlockedVault,
    config: &ProductivityConfig,
) -> Result<(), VaultError> {
    if vault.is_locked() {
        return Err(VaultError::Locked);
    }
    let dek = vault.dek()?;
    let path = vault.root_path().join("productivity.blob");

    let json_bytes =
        serde_json::to_vec_pretty(config).map_err(|e| VaultError::Serialization(e.to_string()))?;

    let nonce = Nonce::generate();
    let aad = format!("MAGRATHEA-V1:productivity:{}", vault.header().vault_id);
    let ciphertext = encrypt_payload(dek, &nonce, aad.as_bytes(), &json_bytes)?;

    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(nonce.as_bytes());
    blob.extend_from_slice(&ciphertext);

    atomic_write(&path, &blob)?;
    Ok(())
}

/// Contabiliza palavras totais em um texto literário bruto excluindo frontmatter YAML.
pub fn count_words_in_markdown(text: &str) -> u32 {
    let body = if let Some(stripped) = text.strip_prefix("---\n") {
        if let Some(end_idx) = stripped.find("\n---\n") {
            &stripped[end_idx + 5..]
        } else if let Some(end_idx) = stripped.find("\n---") {
            &stripped[end_idx + 4..]
        } else {
            text
        }
    } else if let Some(stripped) = text.strip_prefix("---\r\n") {
        if let Some(end_idx) = stripped.find("\r\n---\r\n") {
            &stripped[end_idx + 7..]
        } else if let Some(end_idx) = stripped.find("\r\n---") {
            &stripped[end_idx + 5..]
        } else {
            text
        }
    } else {
        text
    };

    body.split_whitespace().count() as u32
}

/// Calcula o total acumulado de palavras em todas as cenas ativas do manuscrito.
pub fn calculate_total_manuscript_words(vault: &UnlockedVault) -> Result<u32, VaultError> {
    let manifest = vault.manifest()?;
    let mut total_words = 0u32;

    for item in &manifest.items {
        if item.kind == "scene" {
            if let Some(blob_id) = &item.blob_id {
                if let Ok(bytes) = vault.read_blob(blob_id) {
                    if let Ok(text) = String::from_utf8(bytes) {
                        total_words += count_words_in_markdown(&text);
                    }
                }
            }
        }
    }

    Ok(total_words)
}

/// Gera uma visão geral calculada de metas e médias agregadas para o frontend.
pub fn get_productivity_overview(
    vault: &UnlockedVault,
    today_date_iso: &str,
) -> Result<ProductivityOverview, VaultError> {
    let config = load_productivity_config(vault)?;
    let total_manuscript_words = calculate_total_manuscript_words(vault).unwrap_or(0);

    let today_entry = config
        .daily_logs
        .get(today_date_iso)
        .cloned()
        .unwrap_or_default();

    // Calcula médias móveis dos últimos 7 e 30 dias a partir do histórico registrado
    let mut log_dates: Vec<&String> = config.daily_logs.keys().collect();
    log_dates.sort();

    let seven_days_logs: Vec<&DailyLogEntry> = log_dates
        .iter()
        .rev()
        .take(7)
        .filter_map(|d| config.daily_logs.get(*d))
        .collect();

    let seven_days_avg_words = if !seven_days_logs.is_empty() {
        let sum: u32 = seven_days_logs.iter().map(|e| e.words_written).sum();
        sum as f64 / seven_days_logs.len() as f64
    } else {
        0.0
    };

    let thirty_days_logs: Vec<&DailyLogEntry> = log_dates
        .iter()
        .rev()
        .take(30)
        .filter_map(|d| config.daily_logs.get(*d))
        .collect();

    let thirty_days_avg_words = if !thirty_days_logs.is_empty() {
        let sum: u32 = thirty_days_logs.iter().map(|e| e.words_written).sum();
        sum as f64 / thirty_days_logs.len() as f64
    } else {
        0.0
    };

    Ok(ProductivityOverview {
        project_target_words: config.project_target_words,
        project_deadline: config.project_deadline,
        daily_target_words: config.daily_target_words,
        total_manuscript_words,
        today_words: today_entry.words_written,
        today_seconds: today_entry.session_duration_seconds,
        seven_days_avg_words,
        thirty_days_avg_words,
        daily_logs: config.daily_logs,
    })
}

/// Registra progresso da sessão atual (palavras escritas e tempo de digitação) para a data especificada.
pub fn record_productivity_session(
    vault: &UnlockedVault,
    date_iso: &str,
    words_delta: u32,
    session_seconds: u32,
) -> Result<ProductivityOverview, VaultError> {
    let mut config = load_productivity_config(vault)?;

    let entry = config
        .daily_logs
        .entry(date_iso.to_string())
        .or_insert_with(DailyLogEntry::default);

    entry.words_written = entry.words_written.saturating_add(words_delta);
    entry.session_duration_seconds = entry
        .session_duration_seconds
        .saturating_add(session_seconds);

    save_productivity_config(vault, &config)?;
    get_productivity_overview(vault, date_iso)
}

/// Atualiza as metas de escrita (total do projeto, prazo e meta diária).
pub fn update_productivity_targets(
    vault: &UnlockedVault,
    project_target: u32,
    deadline: Option<String>,
    daily_target: u32,
    today_date_iso: &str,
) -> Result<ProductivityOverview, VaultError> {
    let mut config = load_productivity_config(vault)?;
    config.project_target_words = project_target;
    config.project_deadline = deadline;
    config.daily_target_words = daily_target;

    save_productivity_config(vault, &config)?;
    get_productivity_overview(vault, today_date_iso)
}
