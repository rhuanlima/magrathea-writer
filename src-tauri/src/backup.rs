//! Módulo de backup automático cifrado rotativo com detecção de mesmo disco (M5-05).
//!
//! Cria cópias de segurança integrais cifradas do cofre literário com timestamp, rotação
//! automática configurável e alerta se a pasta de destino estiver no mesmo disco/partição.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::vault::current_timestamp_iso;

/// Informações sobre um backup existente.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupInfo {
    pub file_name: String,
    pub file_path: String,
    pub timestamp: String,
    pub byte_size: u64,
    pub is_same_disk: bool,
}

/// Resultado da execução de uma rotina de backup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupResult {
    pub backup_file: String,
    pub destination_dir: String,
    pub byte_size: u64,
    pub timestamp: String,
    pub is_same_disk: bool,
    pub rotated_count: usize,
}

/// Detecta se dois caminhos residem no mesmo volume/disco/ponto de montagem.
pub fn is_same_disk(vault_path: &Path, backup_path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let v_meta = fs::metadata(vault_path);
        let b_meta = fs::metadata(backup_path).or_else(|_| {
            let mut cur = backup_path;
            while let Some(parent) = cur.parent() {
                if parent.exists() {
                    return fs::metadata(parent);
                }
                cur = parent;
            }
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            ))
        });

        if let (Ok(v), Ok(b)) = (v_meta, b_meta) {
            return v.dev() == b.dev();
        }
    }

    #[cfg(windows)]
    {
        let v_canon = vault_path
            .canonicalize()
            .unwrap_or_else(|_| vault_path.to_path_buf());
        let b_canon = backup_path
            .canonicalize()
            .unwrap_or_else(|_| backup_path.to_path_buf());

        let v_prefix = v_canon.components().next();
        let b_prefix = b_canon.components().next();
        if let (Some(v), Some(b)) = (v_prefix, b_prefix) {
            return v == b;
        }
    }

    // Heurística padrão
    false
}

/// Copia recursivamente um diretório preservando arquivos e subpastas.
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_child = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_child)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), dest_child)?;
        }
    }
    Ok(())
}

/// Calcula o tamanho total em bytes de um arquivo ou diretório.
fn calculate_dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    total += calculate_dir_size(&entry.path());
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

/// Formata um SystemTime como string ISO básica sem dependência externa.
fn format_system_time(time: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let mins = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{mins:02}:{seconds:02}Z")
}

/// Cria um backup integral cifrado do cofre em um diretório de destino escolhido pelo autor.
pub fn create_encrypted_backup_internal(
    vault_path: &Path,
    destination_dir: &Path,
    project_title: &str,
    max_rotated_backups: usize,
) -> Result<BackupResult, String> {
    if !vault_path.exists() {
        return Err(format!(
            "Caminho do cofre inexistente: {}",
            vault_path.display()
        ));
    }

    fs::create_dir_all(destination_dir).map_err(|e| {
        format!(
            "Falha ao acessar ou criar pasta de destino de backup: {}",
            e
        )
    })?;

    let same_disk = is_same_disk(vault_path, destination_dir);
    let timestamp_iso = current_timestamp_iso();
    let safe_title = project_title
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .trim()
        .to_string();

    let backup_name = format!(
        "{}_backup_{:08x}{:08x}.magrathea_backup",
        safe_title,
        rand::random::<u32>(),
        rand::random::<u32>()
    );

    let backup_target = destination_dir.join(&backup_name);

    // Como o cofre (.magrathea) já possui todos os dados cifrados com XChaCha20-Poly1305,
    // a cópia recursiva preserva integridade e sigilo absoluto
    copy_dir_all(vault_path, &backup_target)
        .map_err(|e| format!("Falha ao copiar arquivos para o backup: {}", e))?;

    let total_bytes = calculate_dir_size(&backup_target);

    // 2. Aplica rotação automática mantendo apenas os `max_rotated_backups` mais recentes
    let mut rotated_count = 0;
    if max_rotated_backups > 0 {
        if let Ok(existing_backups) =
            list_backups_internal(vault_path, destination_dir, &safe_title)
        {
            if existing_backups.len() > max_rotated_backups {
                let to_delete = existing_backups.len() - max_rotated_backups;
                for old_backup in existing_backups.iter().take(to_delete) {
                    let path = PathBuf::from(&old_backup.file_path);
                    if path.is_dir() {
                        let _ = fs::remove_dir_all(&path);
                    } else {
                        let _ = fs::remove_file(&path);
                    }
                    rotated_count += 1;
                }
            }
        }
    }

    Ok(BackupResult {
        backup_file: backup_name,
        destination_dir: destination_dir.display().to_string(),
        byte_size: total_bytes,
        timestamp: timestamp_iso,
        is_same_disk: same_disk,
        rotated_count,
    })
}

/// Lista backups existentes ordenados cronologicamente do mais antigo para o mais recente.
pub fn list_backups_internal(
    vault_path: &Path,
    destination_dir: &Path,
    title_filter: &str,
) -> Result<Vec<BackupInfo>, String> {
    if !destination_dir.exists() {
        return Ok(Vec::new());
    }

    let same_disk = is_same_disk(vault_path, destination_dir);
    let mut backups = Vec::new();

    let entries = fs::read_dir(destination_dir)
        .map_err(|e| format!("Falha ao ler diretório de backups: {}", e))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".magrathea_backup")
            && (title_filter.is_empty() || name.contains(title_filter))
        {
            let path = entry.path();
            let size = if path.is_dir() {
                calculate_dir_size(&path)
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            };

            let created_iso = entry
                .metadata()
                .and_then(|m| m.modified())
                .map(format_system_time)
                .unwrap_or_else(|_| current_timestamp_iso());

            backups.push(BackupInfo {
                file_name: name,
                file_path: path.display().to_string(),
                timestamp: created_iso,
                byte_size: size,
                is_same_disk: same_disk,
            });
        }
    }

    // Ordena pelo timestamp crescente (mais antigo primeiro para rotação)
    backups.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(backups)
}
