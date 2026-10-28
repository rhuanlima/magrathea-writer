//! Comandos IPC expostos para o frontend via ponte Tauri.

use crate::crypto::SecretKey;
use crate::index::{BacklinkEntry, EntityEntry, IndexOverviewResponse, VaultIndex};
use crate::refactor::{
    apply_rename, preview_rename, rollback_rename, ApplyRenameRefactorResponse,
    RenameRefactorPreviewResponse,
};
use crate::state::AppState;
use crate::vault::VaultStorage;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Resposta retornada após a criação bem-sucedida de um cofre.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateVaultResponse {
    /// Identificador UUID único do cofre.
    pub vault_id: String,
    /// Título do romance ou projeto literário.
    pub project_title: String,
    /// Chave de recuperação de emergência formatada em token para exibição única.
    pub recovery_key_token: String,
    /// Caminho absoluto do cofre no disco.
    pub vault_path: String,
}

/// Estado atual do cofre em memória.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultStatusResponse {
    /// Indica se existe um cofre carregado em memória.
    pub is_open: bool,
    /// Indica se o cofre está bloqueado.
    pub is_locked: bool,
    /// UUID do cofre ativo, se houver.
    pub vault_id: Option<String>,
    /// Título do projeto ativo, se houver.
    pub project_title: Option<String>,
    /// Caminho do cofre no disco, se houver.
    pub vault_path: Option<String>,
}

/// Retorna mensagem inicial de boas-vindas para verificação da ponte IPC.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Magrathea Writer.", name)
}

/// Obtém o diretório padrão sugerido para a criação de novos cofres.
#[tauri::command]
pub fn get_default_vault_dir() -> String {
    let mut default_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(user_home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let docs = PathBuf::from(user_home).join("Documents");
        if docs.exists() && docs.is_dir() {
            default_dir = docs;
        }
    }
    default_dir.display().to_string()
}

/// Cria e inicializa um novo cofre `.magrathea` em disco e o ativa em memória.
#[tauri::command]
pub fn create_vault(
    state: tauri::State<'_, AppState>,
    dir_path: String,
    project_title: String,
    passphrase: String,
) -> Result<CreateVaultResponse, String> {
    create_vault_internal(&state, &dir_path, &project_title, &passphrase)
}

/// Lógica interna de criação de cofre desacoplada do container de injeção Tauri.
pub fn create_vault_internal(
    state: &AppState,
    dir_path: &str,
    project_title: &str,
    passphrase: &str,
) -> Result<CreateVaultResponse, String> {
    let trimmed_title = project_title.trim();
    if trimmed_title.is_empty() {
        return Err("O título do projeto não pode ser vazio.".to_string());
    }

    if passphrase.is_empty() {
        return Err("A senha mestra não pode ser vazia.".to_string());
    }

    let raw_path = PathBuf::from(dir_path.trim());
    let target_vault_path = resolve_vault_path(&raw_path, trimmed_title);

    let recovery_key = SecretKey::generate();
    let recovery_token = recovery_key.to_formatted_token();

    let unlocked_vault = VaultStorage::create(
        &target_vault_path,
        trimmed_title,
        passphrase.as_bytes(),
        &recovery_key,
        None,
    )
    .map_err(|e| format!("Falha ao criar cofre: {e}"))?;

    let vault_id = unlocked_vault.header().vault_id.clone();
    let final_path = target_vault_path.display().to_string();

    if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
        *last_path_guard = Some(final_path.clone());
    }
    if let Ok(mut last_title_guard) = state.last_project_title.lock() {
        *last_title_guard = Some(trimmed_title.to_string());
    }

    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    if let Ok(idx) = VaultIndex::build_from_vault(&unlocked_vault) {
        if let Ok(mut idx_guard) = state.index.lock() {
            *idx_guard = Some(idx);
        }
    }
    *guard = Some(unlocked_vault);

    Ok(CreateVaultResponse {
        vault_id,
        project_title: trimmed_title.to_string(),
        recovery_key_token: recovery_token,
        vault_path: final_path,
    })
}

/// Obtém o estado e os metadados do cofre atualmente ativo ou bloqueado.
#[tauri::command]
pub fn get_vault_status(state: tauri::State<'_, AppState>) -> Result<VaultStatusResponse, String> {
    get_vault_status_internal(&state)
}

/// Lógica interna para obter o status atual do cofre em memória.
pub fn get_vault_status_internal(state: &AppState) -> Result<VaultStatusResponse, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    match &*guard {
        Some(vault) => Ok(VaultStatusResponse {
            is_open: true,
            is_locked: vault.is_locked(),
            vault_id: Some(vault.header().vault_id.clone()),
            project_title: vault.manifest().ok().map(|m| m.project_title.clone()),
            vault_path: Some(vault.root_path().display().to_string()),
        }),
        None => {
            let last_path = state.last_vault_path.lock().ok().and_then(|p| p.clone());
            let last_title = state.last_project_title.lock().ok().and_then(|t| t.clone());
            Ok(VaultStatusResponse {
                is_open: false,
                is_locked: true,
                vault_id: None,
                project_title: last_title,
                vault_path: last_path,
            })
        }
    }
}

/// Bloqueia o cofre atual em memória, descartando e zeroizando a chave DEK.
#[tauri::command]
pub fn lock_vault(state: tauri::State<'_, AppState>) -> Result<(), String> {
    lock_vault_internal(&state)
}

/// Lógica interna para bloquear o cofre e zeroizar a DEK.
pub fn lock_vault_internal(state: &AppState) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    if let Some(ref mut vault) = *guard {
        if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
            *last_path_guard = Some(vault.root_path().display().to_string());
        }
        if let Ok(manifest) = vault.manifest() {
            if let Ok(mut last_title_guard) = state.last_project_title.lock() {
                *last_title_guard = Some(manifest.project_title.clone());
            }
        }
        vault.lock();
    }
    *guard = None;

    if let Ok(mut idx_guard) = state.index.lock() {
        *idx_guard = None;
    }

    if let Ok(mut snap_guard) = state.refactor_snapshots.lock() {
        snap_guard.clear();
    }

    if let Ok(mut snap_guard) = state.refactor_snapshots.lock() {
        snap_guard.clear();
    }

    Ok(())
}

/// Abre um cofre existente decifrando o Slot 0 com a frase-senha mestra.
#[tauri::command]
pub fn open_vault_with_passphrase(
    state: tauri::State<'_, AppState>,
    vault_path: String,
    passphrase: String,
) -> Result<VaultStatusResponse, String> {
    open_vault_with_passphrase_internal(&state, &vault_path, &passphrase)
}

/// Lógica interna para abertura de cofre com frase-senha.
pub fn open_vault_with_passphrase_internal(
    state: &AppState,
    vault_path: &str,
    passphrase: &str,
) -> Result<VaultStatusResponse, String> {
    let path = PathBuf::from(vault_path.trim());
    if !path.exists() {
        return Err("O caminho especificado para o cofre não foi encontrado.".to_string());
    }

    let unlocked_vault = VaultStorage::open_with_passphrase(&path, passphrase.as_bytes())
        .map_err(|_| "Senha incorreta ou cofre corrompido.".to_string())?;

    let vault_id = unlocked_vault.header().vault_id.clone();
    let project_title = unlocked_vault
        .manifest()
        .map(|m| m.project_title.clone())
        .unwrap_or_else(|_| "Sem Título".to_string());
    let canonical_path = unlocked_vault.root_path().display().to_string();

    if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
        *last_path_guard = Some(canonical_path.clone());
    }
    if let Ok(mut last_title_guard) = state.last_project_title.lock() {
        *last_title_guard = Some(project_title.clone());
    }

    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    if let Ok(idx) = VaultIndex::build_from_vault(&unlocked_vault) {
        if let Ok(mut idx_guard) = state.index.lock() {
            *idx_guard = Some(idx);
        }
    }
    *guard = Some(unlocked_vault);

    Ok(VaultStatusResponse {
        is_open: true,
        is_locked: false,
        vault_id: Some(vault_id),
        project_title: Some(project_title),
        vault_path: Some(canonical_path),
    })
}

/// Abre um cofre existente decifrando o Slot 1 com a chave de recuperação de emergência.
#[tauri::command]
pub fn open_vault_with_recovery_key(
    state: tauri::State<'_, AppState>,
    vault_path: String,
    recovery_key: String,
) -> Result<VaultStatusResponse, String> {
    open_vault_with_recovery_key_internal(&state, &vault_path, &recovery_key)
}

/// Lógica interna para abertura de cofre com chave de recuperação.
pub fn open_vault_with_recovery_key_internal(
    state: &AppState,
    vault_path: &str,
    recovery_key: &str,
) -> Result<VaultStatusResponse, String> {
    let path = PathBuf::from(vault_path.trim());
    if !path.exists() {
        return Err("O caminho especificado para o cofre não foi encontrado.".to_string());
    }

    let key = SecretKey::from_hex(recovery_key.trim())
        .map_err(|_| "Chave de recuperação inválida ou malformatada.".to_string())?;

    let unlocked_vault = VaultStorage::open_with_recovery_key(&path, &key)
        .map_err(|_| "Chave de recuperação incorreta ou cofre corrompido.".to_string())?;

    let vault_id = unlocked_vault.header().vault_id.clone();
    let project_title = unlocked_vault
        .manifest()
        .map(|m| m.project_title.clone())
        .unwrap_or_else(|_| "Sem Título".to_string());
    let canonical_path = unlocked_vault.root_path().display().to_string();

    if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
        *last_path_guard = Some(canonical_path.clone());
    }
    if let Ok(mut last_title_guard) = state.last_project_title.lock() {
        *last_title_guard = Some(project_title.clone());
    }

    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    if let Ok(idx) = VaultIndex::build_from_vault(&unlocked_vault) {
        if let Ok(mut idx_guard) = state.index.lock() {
            *idx_guard = Some(idx);
        }
    }
    *guard = Some(unlocked_vault);

    Ok(VaultStatusResponse {
        is_open: true,
        is_locked: false,
        vault_id: Some(vault_id),
        project_title: Some(project_title),
        vault_path: Some(canonical_path),
    })
}

/// Resposta de salvamento atômico de cena.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveSceneResponse {
    pub scene_id: String,
    pub saved_at: String,
}

/// Fecha e desvincula o cofre atual, limpando metadados de sessão recente.
#[tauri::command]
pub fn close_vault(state: tauri::State<'_, AppState>) -> Result<(), String> {
    close_vault_internal(&state)
}

/// Lógica interna para fechar e desvincular o cofre.
pub fn close_vault_internal(state: &AppState) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    if let Some(ref mut vault) = *guard {
        vault.lock();
    }
    *guard = None;

    if let Ok(mut idx_guard) = state.index.lock() {
        *idx_guard = None;
    }

    if let Ok(mut snap_guard) = state.refactor_snapshots.lock() {
        snap_guard.clear();
    }

    if let Ok(mut last_path_guard) = state.last_vault_path.lock() {
        *last_path_guard = None;
    }
    if let Ok(mut last_title_guard) = state.last_project_title.lock() {
        *last_title_guard = None;
    }

    Ok(())
}

/// Salva o conteúdo de uma cena de forma estritamente atômica e cifrada no cofre.
#[tauri::command]
pub fn save_scene_content(
    state: tauri::State<'_, AppState>,
    scene_id: String,
    content: String,
) -> Result<SaveSceneResponse, String> {
    save_scene_content_internal(&state, &scene_id, &content)
}

/// Lógica interna para gravação atômica e cifrada de cena.
pub fn save_scene_content_internal(
    state: &AppState,
    scene_id: &str,
    content: &str,
) -> Result<SaveSceneResponse, String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória para salvar o manuscrito.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado. Desbloqueie-o para salvar.".to_string());
    }

    let trimmed_id = if scene_id.trim().is_empty() {
        "default_scene"
    } else {
        scene_id.trim()
    };

    // 1. Gravação atômica cifrada do blob individual
    vault
        .write_blob(trimmed_id, content.as_bytes())
        .map_err(|e| format!("Falha ao gravar blob cifrado: {e}"))?;

    let timestamp_iso = crate::vault::current_timestamp_iso();

    // 2. Atualização atômica do manifesto criptográfico
    let manifest = vault
        .manifest_mut()
        .map_err(|e| format!("Falha ao acessar manifesto: {e}"))?;

    if let Some(item) = manifest.items.iter_mut().find(|it| it.id == trimmed_id) {
        item.blob_id = Some(trimmed_id.to_string());
    } else {
        manifest.items.push(crate::vault::ManifestItem {
            id: trimmed_id.to_string(),
            kind: "scene".to_string(),
            parent_id: None,
            title: "Cena Inicial".to_string(),
            slug: "cena-inicial".to_string(),
            blob_id: Some(trimmed_id.to_string()),
            order: 0,
            tags: Vec::new(),
            status: Some("draft".to_string()),
        });
    }

    // 3. Persistência atômica do manifesto com incremento de versão monotônica
    vault
        .save_manifest(&timestamp_iso)
        .map_err(|e| format!("Falha ao salvar manifesto autenticado: {e}"))?;

    let (item_title, item_kind) = {
        let manifest = vault.manifest().map_err(|e| e.to_string())?;
        manifest
            .items
            .iter()
            .find(|it| it.id == trimmed_id)
            .map(|it| (it.title.clone(), it.kind.clone()))
            .unwrap_or_else(|| (trimmed_id.to_string(), "scene".to_string()))
    };

    drop(guard);

    if let Ok(mut idx_guard) = state.index.lock() {
        if let Some(ref mut idx) = *idx_guard {
            idx.index_document_content(trimmed_id, &item_title, &item_kind, content);
        }
    }

    Ok(SaveSceneResponse {
        scene_id: trimmed_id.to_string(),
        saved_at: timestamp_iso,
    })
}

/// Carrega e decifra o conteúdo em texto de uma cena armazenada no cofre.
#[tauri::command]
pub fn load_scene_content(
    state: tauri::State<'_, AppState>,
    scene_id: String,
) -> Result<String, String> {
    load_scene_content_internal(&state, &scene_id)
}

/// Lógica interna para leitura e decifragem de cena.
pub fn load_scene_content_internal(state: &AppState, scene_id: &str) -> Result<String, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória para carregar o manuscrito.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado. Desbloqueie-o para ler.".to_string());
    }

    let trimmed_id = if scene_id.trim().is_empty() {
        "default_scene"
    } else {
        scene_id.trim()
    };

    let blob_bytes = vault
        .read_blob(trimmed_id)
        .map_err(|e| format!("Falha ao ler blob da cena: {e}"))?;

    String::from_utf8(blob_bytes)
        .map_err(|e| format!("Conteúdo da cena não é uma string UTF-8 válida: {e}"))
}

/// Estrutura de resposta contendo a árvore do manuscrito e a lixeira interna.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManuscriptTreeResponse {
    pub items: Vec<crate::vault::ManifestItem>,
    pub trash: Vec<crate::vault::ManifestItem>,
}

/// Obtém todos os nós da árvore do romance e da lixeira criptografada.
#[tauri::command]
pub fn get_manuscript_tree(
    state: tauri::State<'_, AppState>,
) -> Result<ManuscriptTreeResponse, String> {
    get_manuscript_tree_internal(&state)
}

/// Lógica interna para obtenção da árvore do manuscrito.
pub fn get_manuscript_tree_internal(state: &AppState) -> Result<ManuscriptTreeResponse, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let manifest = vault
        .manifest()
        .map_err(|e| format!("Falha ao acessar manifesto: {e}"))?;

    Ok(ManuscriptTreeResponse {
        items: manifest.items.clone(),
        trash: manifest.trash.clone(),
    })
}

/// Cria um novo nó hierárquico no manuscrito (Parte, Capítulo ou Cena).
#[tauri::command]
pub fn create_manuscript_item(
    state: tauri::State<'_, AppState>,
    kind: String,
    parent_id: Option<String>,
    title: String,
) -> Result<crate::vault::ManifestItem, String> {
    create_manuscript_item_internal(&state, &kind, parent_id.as_deref(), &title)
}

/// Lógica interna para criação de nó no manuscrito.
pub fn create_manuscript_item_internal(
    state: &AppState,
    kind: &str,
    parent_id: Option<&str>,
    title: &str,
) -> Result<crate::vault::ManifestItem, String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let trimmed_title = if title.trim().is_empty() {
        match kind {
            "part" => "Nova Parte",
            "chapter" => "Novo Capítulo",
            _ => "Nova Cena",
        }
    } else {
        title.trim()
    };

    let item_id = format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        rand::random::<u32>(),
        rand::random::<u16>(),
        rand::random::<u16>() & 0x0fff,
        (rand::random::<u16>() & 0x3fff) | 0x8000,
        rand::random::<u64>() & 0xffffffffffff
    );

    let slug = trimmed_title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    let blob_id = if kind == "scene" {
        Some(item_id.clone())
    } else {
        None
    };

    if let Some(ref bid) = blob_id {
        let initial_doc = format!("# {trimmed_title}\n\n");
        vault
            .write_blob(bid, initial_doc.as_bytes())
            .map_err(|e| format!("Falha ao inicializar blob da cena: {e}"))?;
    }

    let timestamp_iso = crate::vault::current_timestamp_iso();
    let manifest = vault
        .manifest_mut()
        .map_err(|e| format!("Falha ao acessar manifesto: {e}"))?;

    let order = manifest
        .items
        .iter()
        .filter(|it| it.parent_id.as_deref() == parent_id)
        .count() as u32;

    let item = crate::vault::ManifestItem {
        id: item_id,
        kind: kind.to_string(),
        parent_id: parent_id.map(|s| s.to_string()),
        title: trimmed_title.to_string(),
        slug,
        blob_id,
        order,
        tags: Vec::new(),
        status: if kind == "scene" {
            Some("draft".to_string())
        } else {
            None
        },
    };

    manifest.items.push(item.clone());
    vault
        .save_manifest(&timestamp_iso)
        .map_err(|e| format!("Falha ao salvar manifesto: {e}"))?;

    let item_clone = item.clone();
    drop(guard);

    if let Ok(mut idx_guard) = state.index.lock() {
        if let Some(ref mut idx) = *idx_guard {
            idx.register_entity(EntityEntry {
                id: item_clone.id.clone(),
                title: item_clone.title.clone(),
                slug: item_clone.slug.clone(),
                kind: item_clone.kind.clone(),
                tags: item_clone.tags.clone(),
                status: item_clone.status.clone(),
                summary: None,
                mention_count: 0,
            });
        }
    }

    Ok(item)
}

/// Renomeia um nó do manuscrito.
#[tauri::command]
pub fn rename_manuscript_item(
    state: tauri::State<'_, AppState>,
    item_id: String,
    new_title: String,
) -> Result<(), String> {
    rename_manuscript_item_internal(&state, &item_id, &new_title)
}

/// Lógica interna para renomeação de nó.
pub fn rename_manuscript_item_internal(
    state: &AppState,
    item_id: &str,
    new_title: &str,
) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let trimmed = new_title.trim();
    if trimmed.is_empty() {
        return Err("O título não pode ser vazio.".to_string());
    }

    let timestamp_iso = crate::vault::current_timestamp_iso();
    let manifest = vault
        .manifest_mut()
        .map_err(|e| format!("Falha ao acessar manifesto: {e}"))?;

    let item = manifest
        .items
        .iter_mut()
        .find(|it| it.id == item_id)
        .ok_or_else(|| format!("Item '{item_id}' não encontrado."))?;

    item.title = trimmed.to_string();
    item.slug = trimmed
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    let item_slug = item.slug.clone();
    let item_kind = item.kind.clone();
    let item_tags = item.tags.clone();
    let item_status = item.status.clone();

    vault
        .save_manifest(&timestamp_iso)
        .map_err(|e| format!("Falha ao salvar manifesto: {e}"))?;

    drop(guard);

    if let Ok(mut idx_guard) = state.index.lock() {
        if let Some(ref mut idx) = *idx_guard {
            idx.remove_entity(item_id);
            idx.register_entity(EntityEntry {
                id: item_id.to_string(),
                title: trimmed.to_string(),
                slug: item_slug,
                kind: item_kind,
                tags: item_tags,
                status: item_status,
                summary: None,
                mention_count: 0,
            });
        }
    }

    Ok(())
}

/// Atualiza o status e etiquetas livres de uma cena.
#[tauri::command]
pub fn update_scene_metadata(
    state: tauri::State<'_, AppState>,
    item_id: String,
    status: Option<String>,
    tags: Vec<String>,
) -> Result<(), String> {
    update_scene_metadata_internal(&state, &item_id, status, tags)
}

/// Lógica interna para atualização de metadados da cena.
pub fn update_scene_metadata_internal(
    state: &AppState,
    item_id: &str,
    status: Option<String>,
    tags: Vec<String>,
) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let timestamp_iso = crate::vault::current_timestamp_iso();
    let manifest = vault
        .manifest_mut()
        .map_err(|e| format!("Falha ao acessar manifesto: {e}"))?;

    let item = manifest
        .items
        .iter_mut()
        .find(|it| it.id == item_id)
        .ok_or_else(|| format!("Item '{item_id}' não encontrado."))?;

    item.status = status;
    item.tags = tags;

    vault
        .save_manifest(&timestamp_iso)
        .map_err(|e| format!("Falha ao salvar manifesto: {e}"))?;

    Ok(())
}

/// Move um item e seus filhos recursivos para a lixeira interna cifrada.
#[tauri::command]
pub fn move_item_to_trash(
    state: tauri::State<'_, AppState>,
    item_id: String,
) -> Result<(), String> {
    move_item_to_trash_internal(&state, &item_id)
}

/// Lógica interna para mover para a lixeira.
pub fn move_item_to_trash_internal(state: &AppState, item_id: &str) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let timestamp_iso = crate::vault::current_timestamp_iso();
    let manifest = vault
        .manifest_mut()
        .map_err(|e| format!("Falha ao acessar manifesto: {e}"))?;

    let mut to_move_ids = std::collections::HashSet::new();
    to_move_ids.insert(item_id.to_string());

    let mut added = true;
    while added {
        added = false;
        for it in &manifest.items {
            if let Some(ref pid) = it.parent_id {
                if to_move_ids.contains(pid) && !to_move_ids.contains(&it.id) {
                    to_move_ids.insert(it.id.clone());
                    added = true;
                }
            }
        }
    }

    let mut moved_items = Vec::new();
    manifest.items.retain(|it| {
        if to_move_ids.contains(&it.id) {
            moved_items.push(it.clone());
            false
        } else {
            true
        }
    });

    manifest.trash.extend(moved_items);

    vault
        .save_manifest(&timestamp_iso)
        .map_err(|e| format!("Falha ao salvar manifesto: {e}"))?;

    drop(guard);

    if let Ok(mut idx_guard) = state.index.lock() {
        if let Some(ref mut idx) = *idx_guard {
            for id in &to_move_ids {
                idx.remove_entity(id);
            }
        }
    }

    Ok(())
}

/// Restaura um item da lixeira interna para o manuscrito ativo.
#[tauri::command]
pub fn restore_item_from_trash(
    state: tauri::State<'_, AppState>,
    item_id: String,
) -> Result<(), String> {
    restore_item_from_trash_internal(&state, &item_id)
}

/// Lógica interna para restauração de item da lixeira.
pub fn restore_item_from_trash_internal(state: &AppState, item_id: &str) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let timestamp_iso = crate::vault::current_timestamp_iso();
    let manifest = vault
        .manifest_mut()
        .map_err(|e| format!("Falha ao acessar manifesto: {e}"))?;

    let pos = manifest
        .trash
        .iter()
        .position(|it| it.id == item_id)
        .ok_or_else(|| format!("Item '{item_id}' não encontrado na lixeira."))?;

    let mut item = manifest.trash.remove(pos);

    if let Some(ref pid) = item.parent_id {
        if !manifest.items.iter().any(|it| it.id == *pid) {
            item.parent_id = None;
        }
    }

    manifest.items.push(item);

    vault
        .save_manifest(&timestamp_iso)
        .map_err(|e| format!("Falha ao salvar manifesto: {e}"))?;

    Ok(())
}

/// Esvazia definitivamente a lixeira interna, apagando os blobs correspondentes no disco.
#[tauri::command]
pub fn empty_trash(state: tauri::State<'_, AppState>) -> Result<(), String> {
    empty_trash_internal(&state)
}

/// Lógica interna para esvaziamento permanente da lixeira.
pub fn empty_trash_internal(state: &AppState) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let timestamp_iso = crate::vault::current_timestamp_iso();
    let manifest = vault
        .manifest_mut()
        .map_err(|e| format!("Falha ao acessar manifesto: {e}"))?;

    let blobs_to_delete: Vec<String> = manifest
        .trash
        .iter()
        .filter_map(|it| it.blob_id.clone())
        .collect();

    manifest.trash.clear();

    for bid in &blobs_to_delete {
        let _ = vault.delete_blob(bid);
    }

    vault
        .save_manifest(&timestamp_iso)
        .map_err(|e| format!("Falha ao salvar manifesto: {e}"))?;

    Ok(())
}

/// Reordena uma lista de itens irmãos sob um mesmo nó pai.
#[tauri::command]
pub fn reorder_manuscript_items(
    state: tauri::State<'_, AppState>,
    parent_id: Option<String>,
    ordered_ids: Vec<String>,
) -> Result<(), String> {
    reorder_manuscript_items_internal(&state, parent_id.as_deref(), &ordered_ids)
}

/// Lógica interna para reordenação de itens.
pub fn reorder_manuscript_items_internal(
    state: &AppState,
    parent_id: Option<&str>,
    ordered_ids: &[String],
) -> Result<(), String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let timestamp_iso = crate::vault::current_timestamp_iso();
    let manifest = vault
        .manifest_mut()
        .map_err(|e| format!("Falha ao acessar manifesto: {e}"))?;

    for (idx, id) in ordered_ids.iter().enumerate() {
        if let Some(it) = manifest.items.iter_mut().find(|it| it.id == *id) {
            if it.parent_id.as_deref() == parent_id {
                it.order = idx as u32;
            }
        }
    }

    vault
        .save_manifest(&timestamp_iso)
        .map_err(|e| format!("Falha ao salvar manifesto: {e}"))?;

    Ok(())
}

/// Altera a frase-senha mestra do cofre ativo sem necessidade de recifrar os blobs de texto.
#[tauri::command]
pub fn change_vault_passphrase(
    state: tauri::State<'_, AppState>,
    current_passphrase: String,
    new_passphrase: String,
) -> Result<(), String> {
    change_vault_passphrase_internal(&state, &current_passphrase, &new_passphrase)
}

/// Lógica interna para alteração de frase-senha mestra.
pub fn change_vault_passphrase_internal(
    state: &AppState,
    current_passphrase: &str,
    new_passphrase: &str,
) -> Result<(), String> {
    if new_passphrase.is_empty() {
        return Err("A nova senha mestra não pode ser vazia.".to_string());
    }

    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let vault_id = vault.header().vault_id.clone();

    // Re-envelopa a DEK no Slot 0 com a nova frase-senha
    vault
        .header_mut()
        .change_passphrase(
            current_passphrase.as_bytes(),
            new_passphrase.as_bytes(),
            None,
        )
        .map_err(|e| format!("Falha ao alterar senha mestra: {e}"))?;

    vault
        .save_header()
        .map_err(|e| format!("Falha ao salvar cabeçalho atualizado: {e}"))?;

    // Se a credencial estava salva no chaveiro do SO, atualiza com a nova senha
    if let Ok(true) = crate::keychain::has_vault_secret(&vault_id) {
        let _ = crate::keychain::save_vault_secret(&vault_id, new_passphrase);
    }

    Ok(())
}

/// Recifra integralmente o cofre (gera nova DEK, recifra todos os blobs e o manifesto, e gera nova chave de recuperação).
#[tauri::command]
pub fn reencrypt_vault(
    state: tauri::State<'_, AppState>,
    passphrase: String,
) -> Result<String, String> {
    reencrypt_vault_internal(&state, &passphrase)
}

/// Lógica interna para recifragem completa do cofre com rotação de DEK e nova chave de recuperação.
pub fn reencrypt_vault_internal(state: &AppState, passphrase: &str) -> Result<String, String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    // 1. Valida que a frase-senha atual abre o Slot 0
    let _ = vault
        .header()
        .unlock_with_passphrase(passphrase.as_bytes())
        .map_err(|_| {
            "Senha incorreta. A recifragem requer autenticação com a senha atual.".to_string()
        })?;

    // 2. Gera nova DEK aleatória e nova chave de recuperação
    let new_dek = crate::crypto::SecretKey::generate();
    let recovery_key = crate::crypto::SecretKey::generate();
    let recovery_token_formatted = recovery_key.to_formatted_token();

    let timestamp_iso = crate::vault::current_timestamp_iso();

    // 3. Recifra todos os blobs e o manifesto em disco com a nova DEK
    vault
        .reencrypt_all(new_dek.clone(), &timestamp_iso)
        .map_err(|e| format!("Falha ao recifrar conteúdos do cofre: {e}"))?;

    // 4. Re-envelopa a nova DEK nos Slots 0 e 1 no cabeçalho
    vault
        .header_mut()
        .rekey(&new_dek, passphrase.as_bytes(), &recovery_key, None)
        .map_err(|e| format!("Falha ao atualizar slots do cabeçalho: {e}"))?;

    vault
        .save_header()
        .map_err(|e| format!("Falha ao gravar novo cabeçalho do cofre: {e}"))?;

    Ok(recovery_token_formatted)
}

/// Consulta se o cofre ativo possui credencial lembrada no chaveiro do SO.
#[tauri::command]
pub fn get_vault_keyring_status(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    get_vault_keyring_status_internal(&state)
}

/// Lógica interna para consulta de status do chaveiro.
pub fn get_vault_keyring_status_internal(state: &AppState) -> Result<bool, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    let vault_id = &vault.header().vault_id;
    match crate::keychain::has_vault_secret(vault_id) {
        Ok(has) => Ok(has),
        Err(_) => Ok(false),
    }
}

/// Salva a credencial do cofre no chaveiro do sistema operacional.
#[tauri::command]
pub fn save_to_keyring(
    state: tauri::State<'_, AppState>,
    passphrase: String,
) -> Result<(), String> {
    save_to_keyring_internal(&state, &passphrase)
}

/// Lógica interna para gravação no chaveiro do SO.
pub fn save_to_keyring_internal(state: &AppState, passphrase: &str) -> Result<(), String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    // Valida a frase-senha antes de persistir no chaveiro
    vault
        .header()
        .unlock_with_passphrase(passphrase.as_bytes())
        .map_err(|_| "Senha incorreta. Não é possível salvar no chaveiro.".to_string())?;

    let vault_id = &vault.header().vault_id;
    crate::keychain::save_vault_secret(vault_id, passphrase)
}

/// Remove a credencial do cofre do chaveiro do sistema operacional.
#[tauri::command]
pub fn remove_from_keyring(state: tauri::State<'_, AppState>) -> Result<(), String> {
    remove_from_keyring_internal(&state)
}

/// Lógica interna para remoção do chaveiro do SO.
pub fn remove_from_keyring_internal(state: &AppState) -> Result<(), String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    let vault_id = &vault.header().vault_id;
    crate::keychain::delete_vault_secret(vault_id)
}

/// Abre e desbloqueia um cofre utilizando a credencial armazenada no chaveiro do SO.
#[tauri::command]
pub fn open_vault_with_keyring(
    state: tauri::State<'_, AppState>,
    vault_path: String,
) -> Result<VaultStatusResponse, String> {
    open_vault_with_keyring_internal(&state, &vault_path)
}

/// Lógica interna para abertura de cofre via chaveiro do SO.
pub fn open_vault_with_keyring_internal(
    state: &AppState,
    vault_path: &str,
) -> Result<VaultStatusResponse, String> {
    let raw_path = PathBuf::from(vault_path.trim());
    if !raw_path.exists() {
        return Err(format!("Caminho do cofre não encontrado: {vault_path}"));
    }

    let header_path = raw_path.join("header");
    let header_bytes =
        fs::read(&header_path).map_err(|e| format!("Falha ao ler cabeçalho do cofre: {e}"))?;
    let header: crate::vault::VaultHeader =
        serde_json::from_slice(&header_bytes).map_err(|e| format!("Cabeçalho corrompido: {e}"))?;

    let secret_opt = crate::keychain::get_vault_secret(&header.vault_id)?;
    let passphrase = secret_opt.ok_or_else(|| {
        "Nenhuma credencial encontrada no chaveiro do SO para este cofre.".to_string()
    })?;

    open_vault_with_passphrase_internal(state, vault_path, &passphrase)
}

/// Constrói o caminho canônico do cofre com a extensão `.magrathea`.
fn resolve_vault_path(base_dir: &Path, title: &str) -> PathBuf {
    let path_str = base_dir.display().to_string();
    if path_str.ends_with(".magrathea") {
        return base_dir.to_path_buf();
    }

    // Sanitiza o nome do título para compor o nome de pasta
    let sanitized_name: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let folder_name = format!("{sanitized_name}.magrathea");
    base_dir.join(folder_name)
}

/// Obtém a visão geral do índice do cofre em memória (entidades, links totais e pendentes).
#[tauri::command]
pub fn get_vault_index(state: tauri::State<'_, AppState>) -> Result<IndexOverviewResponse, String> {
    get_vault_index_internal(&state)
}

/// Lógica interna para obter a visão geral do índice.
pub fn get_vault_index_internal(state: &AppState) -> Result<IndexOverviewResponse, String> {
    let idx_guard = state
        .index
        .lock()
        .map_err(|_| "Falha ao obter lock do índice.".to_string())?;

    let index = idx_guard
        .as_ref()
        .ok_or_else(|| "Índice indisponível. Desbloqueie o cofre.".to_string())?;

    Ok(index.get_overview())
}

/// Obtém todos os backlinks (menções em outros documentos) que apontam para um item.
#[tauri::command]
pub fn get_document_backlinks(
    state: tauri::State<'_, AppState>,
    item_id: String,
) -> Result<Vec<BacklinkEntry>, String> {
    get_document_backlinks_internal(&state, &item_id)
}

/// Lógica interna para obter backlinks de uma entidade ou cena.
pub fn get_document_backlinks_internal(
    state: &AppState,
    item_id: &str,
) -> Result<Vec<BacklinkEntry>, String> {
    let idx_guard = state
        .index
        .lock()
        .map_err(|_| "Falha ao obter lock do índice.".to_string())?;

    let index = idx_guard
        .as_ref()
        .ok_or_else(|| "Índice indisponível. Desbloqueie o cofre.".to_string())?;

    Ok(index.get_backlinks(item_id))
}

/// Obtém metadados de pré-visualização (hover preview) para um termo citado via wikilink.
#[tauri::command]
pub fn get_entity_preview(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Option<EntityEntry>, String> {
    get_entity_preview_internal(&state, &query)
}

/// Lógica interna para resolução de prévia de entidade para hover popover.
pub fn get_entity_preview_internal(
    state: &AppState,
    query: &str,
) -> Result<Option<EntityEntry>, String> {
    let idx_guard = state
        .index
        .lock()
        .map_err(|_| "Falha ao obter lock do índice.".to_string())?;

    let index = idx_guard
        .as_ref()
        .ok_or_else(|| "Índice indisponível. Desbloqueie o cofre.".to_string())?;

    Ok(index.resolve_wikilink(query).cloned())
}

/// Força a reconstrução em memória do índice a partir do cofre ativo.
#[tauri::command]
pub fn rebuild_vault_index(
    state: tauri::State<'_, AppState>,
) -> Result<IndexOverviewResponse, String> {
    rebuild_vault_index_internal(&state)
}

/// Lógica interna para reconstrução do índice em memória.
pub fn rebuild_vault_index_internal(state: &AppState) -> Result<IndexOverviewResponse, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let new_index = VaultIndex::build_from_vault(vault)
        .map_err(|e| format!("Falha ao reconstruir índice: {e}"))?;

    let overview = new_index.get_overview();
    drop(guard);

    if let Ok(mut idx_guard) = state.index.lock() {
        *idx_guard = Some(new_index);
    }

    Ok(overview)
}

/// Gera a prévia de refatoração para renomear uma entidade e todas as suas citações.
#[tauri::command]
pub fn preview_rename_entity(
    state: tauri::State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<RenameRefactorPreviewResponse, String> {
    preview_rename_entity_internal(&state, &old_name, &new_name)
}

/// Lógica interna para pré-visualização de renomeação.
pub fn preview_rename_entity_internal(
    state: &AppState,
    old_name: &str,
    new_name: &str,
) -> Result<RenameRefactorPreviewResponse, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    preview_rename(vault, old_name, new_name)
        .map_err(|e| format!("Falha na prévia de refatoração: {e}"))
}

/// Aplica a refatoração atômica em lote com snapshot de segurança.
#[tauri::command]
pub fn apply_rename_refactor(
    state: tauri::State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<ApplyRenameRefactorResponse, String> {
    apply_rename_refactor_internal(&state, &old_name, &new_name)
}

/// Lógica interna para aplicação da refatoração com snapshot.
pub fn apply_rename_refactor_internal(
    state: &AppState,
    old_name: &str,
    new_name: &str,
) -> Result<ApplyRenameRefactorResponse, String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let mut idx_guard = state
        .index
        .lock()
        .map_err(|_| "Falha ao obter lock do índice.".to_string())?;

    let index = idx_guard
        .as_mut()
        .ok_or_else(|| "Índice indisponível em memória.".to_string())?;

    let (response, snapshot) = apply_rename(vault, index, old_name, new_name)
        .map_err(|e| format!("Falha ao aplicar refatoração: {e}"))?;

    if let Ok(mut snap_guard) = state.refactor_snapshots.lock() {
        snap_guard.insert(response.snapshot_id.clone(), snapshot);
    }

    Ok(response)
}

/// Desfaz uma refatoração anterior restaurando o snapshot do cofre.
#[tauri::command]
pub fn rollback_rename_refactor(
    state: tauri::State<'_, AppState>,
    snapshot_id: String,
) -> Result<(), String> {
    rollback_rename_refactor_internal(&state, &snapshot_id)
}

/// Lógica interna para rollback da refatoração.
pub fn rollback_rename_refactor_internal(
    state: &AppState,
    snapshot_id: &str,
) -> Result<(), String> {
    let snapshot = {
        let mut snap_guard = state
            .refactor_snapshots
            .lock()
            .map_err(|_| "Falha ao obter lock de snapshots.".to_string())?;
        snap_guard
            .remove(snapshot_id)
            .ok_or_else(|| format!("Snapshot '{snapshot_id}' não encontrado para rollback."))?
    };

    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    let mut idx_guard = state
        .index
        .lock()
        .map_err(|_| "Falha ao obter lock do índice.".to_string())?;

    let index = idx_guard
        .as_mut()
        .ok_or_else(|| "Índice indisponível em memória.".to_string())?;

    rollback_rename(vault, index, &snapshot)
        .map_err(|e| format!("Falha no rollback de refatoração: {e}"))?;

    Ok(())
}

/// Obtém visão geral das métricas de produtividade literária.
#[tauri::command]
pub fn get_productivity_overview(
    state: tauri::State<'_, AppState>,
    today_date_iso: String,
) -> Result<crate::productivity::ProductivityOverview, String> {
    get_productivity_overview_internal(&state, &today_date_iso)
}

/// Lógica interna para consulta de visão geral de produtividade.
pub fn get_productivity_overview_internal(
    state: &AppState,
    today_date_iso: &str,
) -> Result<crate::productivity::ProductivityOverview, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    crate::productivity::get_productivity_overview(vault, today_date_iso)
        .map_err(|e| format!("Falha ao obter métricas de produtividade: {e}"))
}

/// Atualiza as metas de escrita literária do autor (projeto, prazo e diária).
#[tauri::command]
pub fn update_productivity_targets(
    state: tauri::State<'_, AppState>,
    project_target: u32,
    deadline: Option<String>,
    daily_target: u32,
    today_date_iso: String,
) -> Result<crate::productivity::ProductivityOverview, String> {
    update_productivity_targets_internal(
        &state,
        project_target,
        deadline,
        daily_target,
        &today_date_iso,
    )
}

/// Lógica interna para atualização das metas de escrita literária.
pub fn update_productivity_targets_internal(
    state: &AppState,
    project_target: u32,
    deadline: Option<String>,
    daily_target: u32,
    today_date_iso: &str,
) -> Result<crate::productivity::ProductivityOverview, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    crate::productivity::update_productivity_targets(
        vault,
        project_target,
        deadline,
        daily_target,
        today_date_iso,
    )
    .map_err(|e| format!("Falha ao atualizar metas de produtividade: {e}"))
}

/// Registra progresso incremental da sessão atual (palavras escritas e tempo de digitação).
#[tauri::command]
pub fn record_productivity_session(
    state: tauri::State<'_, AppState>,
    date_iso: String,
    words_delta: u32,
    session_seconds: u32,
) -> Result<crate::productivity::ProductivityOverview, String> {
    record_productivity_session_internal(&state, &date_iso, words_delta, session_seconds)
}

/// Lógica interna para gravação incremental do log diário de escrita.
pub fn record_productivity_session_internal(
    state: &AppState,
    date_iso: &str,
    words_delta: u32,
    session_seconds: u32,
) -> Result<crate::productivity::ProductivityOverview, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    crate::productivity::record_productivity_session(vault, date_iso, words_delta, session_seconds)
        .map_err(|e| format!("Falha ao registrar sessão de produtividade: {e}"))
}

/// Retorna o template padrão em Markdown + Frontmatter para um tipo de entidade literária.
#[tauri::command]
pub fn get_entity_template(kind: String, title: String) -> String {
    crate::worldbuilding::generate_entity_template(&kind, &title)
}

/// Cria uma nova entidade literária de worldbuilding com seu template gerado no cofre.
#[tauri::command]
pub fn create_world_entity(
    state: tauri::State<'_, AppState>,
    kind: String,
    title: String,
    tags: Vec<String>,
) -> Result<crate::vault::ManifestItem, String> {
    create_world_entity_internal(&state, &kind, &title, tags)
}

/// Lógica interna para criação de entidade de worldbuilding.
pub fn create_world_entity_internal(
    state: &AppState,
    kind: &str,
    title: &str,
    tags: Vec<String>,
) -> Result<crate::vault::ManifestItem, String> {
    let template_body = crate::worldbuilding::generate_entity_template(kind, title);

    // 1. Cria o item no manifesto
    let item = create_manuscript_item_internal(state, kind, None, title)?;

    // 2. Atualiza tags se informadas
    if !tags.is_empty() {
        let _ = update_scene_metadata_internal(state, &item.id, None, tags);
    }

    // 3. Salva o conteúdo inicial gerado pelo template
    save_scene_content_internal(state, &item.id, &template_body)?;

    Ok(item)
}

/// Importa, valida magic bytes e cifra uma imagem anexada a uma entidade literária.
#[tauri::command]
pub fn import_entity_image(
    state: tauri::State<'_, AppState>,
    entity_id: String,
    file_name: String,
    image_bytes: Vec<u8>,
    created_at_iso: String,
) -> Result<crate::worldbuilding::EntityImageMeta, String> {
    import_entity_image_internal(
        &state,
        &entity_id,
        &file_name,
        &image_bytes,
        &created_at_iso,
    )
}

/// Lógica interna para importação e cifragem de imagem no cofre.
pub fn import_entity_image_internal(
    state: &AppState,
    entity_id: &str,
    file_name: &str,
    image_bytes: &[u8],
    created_at_iso: &str,
) -> Result<crate::worldbuilding::EntityImageMeta, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    crate::worldbuilding::import_entity_image(
        vault,
        entity_id,
        file_name,
        image_bytes,
        created_at_iso,
    )
    .map_err(|e| format!("Falha ao importar imagem: {e}"))
}

/// Carrega e decifra uma imagem sob demanda para visualização.
#[tauri::command]
pub fn load_entity_image(
    state: tauri::State<'_, AppState>,
    image_id: String,
) -> Result<crate::worldbuilding::EntityImageData, String> {
    load_entity_image_internal(&state, &image_id)
}

/// Lógica interna para leitura e decifragem de imagem.
pub fn load_entity_image_internal(
    state: &AppState,
    image_id: &str,
) -> Result<crate::worldbuilding::EntityImageData, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    crate::worldbuilding::load_entity_image(vault, image_id)
        .map_err(|e| format!("Falha ao carregar imagem: {e}"))
}

/// Exclui uma imagem do cofre.
#[tauri::command]
pub fn delete_entity_image(
    state: tauri::State<'_, AppState>,
    entity_id: String,
    image_id: String,
) -> Result<(), String> {
    delete_entity_image_internal(&state, &entity_id, &image_id)
}

/// Lógica interna para exclusão de imagem.
pub fn delete_entity_image_internal(
    state: &AppState,
    entity_id: &str,
    image_id: &str,
) -> Result<(), String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    crate::worldbuilding::delete_entity_image(vault, entity_id, image_id)
        .map_err(|e| format!("Falha ao excluir imagem: {e}"))
}

/// Lista todas as imagens anexadas a uma entidade literária.
#[tauri::command]
pub fn list_entity_images(
    state: tauri::State<'_, AppState>,
    entity_id: String,
) -> Result<Vec<crate::worldbuilding::EntityImageMeta>, String> {
    list_entity_images_internal(&state, &entity_id)
}

/// Lógica interna para listagem de imagens.
pub fn list_entity_images_internal(
    state: &AppState,
    entity_id: &str,
) -> Result<Vec<crate::worldbuilding::EntityImageMeta>, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto em memória.".to_string())?;

    if vault.is_locked() {
        return Err("O cofre está bloqueado.".to_string());
    }

    crate::worldbuilding::list_entity_images(vault, entity_id)
        .map_err(|e| format!("Falha ao listar imagens da entidade: {e}"))
}

/// Rastreia e lista as aparições de uma entidade em todas as cenas do manuscrito.
#[tauri::command]
pub fn get_entity_appearances(
    state: tauri::State<'_, AppState>,
    entity_id: String,
) -> Result<Vec<crate::worldbuilding::EntityAppearance>, String> {
    get_entity_appearances_internal(&state, &entity_id)
}

/// Lógica interna para rastreio de aparições no índice.
pub fn get_entity_appearances_internal(
    state: &AppState,
    entity_id: &str,
) -> Result<Vec<crate::worldbuilding::EntityAppearance>, String> {
    let guard = state
        .index
        .lock()
        .map_err(|_| "Falha ao obter lock do índice.".to_string())?;

    let index = guard
        .as_ref()
        .ok_or_else(|| "Índice indisponível em memória.".to_string())?;

    Ok(crate::worldbuilding::get_entity_appearances(
        index, entity_id,
    ))
}

// ============================================================================
// COMANDOS DE SNAPSHOTS E VERSÕES (M5-01, M5-02)
// ============================================================================

/// Resposta da operação de restauração de snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RestoreSnapshotResult {
    pub restored_content: String,
    pub safety_snapshot: crate::snapshots::SnapshotMeta,
}

/// Cria um snapshot da cena ativa no cofre cifrado.
#[tauri::command]
pub fn create_scene_snapshot(
    state: tauri::State<'_, AppState>,
    scene_id: String,
    label: String,
    trigger: String,
    current_content: Option<String>,
) -> Result<crate::snapshots::SnapshotMeta, String> {
    create_scene_snapshot_internal(
        &state,
        &scene_id,
        &label,
        &trigger,
        current_content.as_deref(),
    )
}

pub fn create_scene_snapshot_internal(
    state: &AppState,
    scene_id: &str,
    label: &str,
    trigger: &str,
    current_content: Option<&str>,
) -> Result<crate::snapshots::SnapshotMeta, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    let content_to_snapshot = match current_content {
        Some(c) => c.to_string(),
        None => {
            let bytes = vault.read_blob(scene_id).map_err(|e| e.to_string())?;
            String::from_utf8(bytes).map_err(|e| e.to_string())?
        }
    };

    crate::snapshots::create_scene_snapshot_internal(
        vault,
        scene_id,
        &content_to_snapshot,
        label,
        trigger,
    )
    .map_err(|e| e.to_string())
}

/// Lista todos os snapshots de uma cena no cofre.
#[tauri::command]
pub fn list_scene_snapshots(
    state: tauri::State<'_, AppState>,
    scene_id: String,
) -> Result<Vec<crate::snapshots::SnapshotMeta>, String> {
    list_scene_snapshots_internal(&state, &scene_id)
}

pub fn list_scene_snapshots_internal(
    state: &AppState,
    scene_id: &str,
) -> Result<Vec<crate::snapshots::SnapshotMeta>, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    crate::snapshots::list_scene_snapshots_internal(vault, scene_id).map_err(|e| e.to_string())
}

/// Carrega e decifra o conteúdo de um snapshot específico.
#[tauri::command]
pub fn get_scene_snapshot_content(
    state: tauri::State<'_, AppState>,
    scene_id: String,
    snapshot_id: String,
) -> Result<String, String> {
    get_scene_snapshot_content_internal(&state, &scene_id, &snapshot_id)
}

pub fn get_scene_snapshot_content_internal(
    state: &AppState,
    scene_id: &str,
    snapshot_id: &str,
) -> Result<String, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    crate::snapshots::get_scene_snapshot_content_internal(vault, scene_id, snapshot_id)
        .map_err(|e| e.to_string())
}

/// Compara um snapshot com o conteúdo atual da cena (diff).
#[tauri::command]
pub fn compare_snapshot_with_current(
    state: tauri::State<'_, AppState>,
    scene_id: String,
    snapshot_id: String,
    current_content: String,
) -> Result<crate::snapshots::SnapshotDiff, String> {
    compare_snapshot_with_current_internal(&state, &scene_id, &snapshot_id, &current_content)
}

pub fn compare_snapshot_with_current_internal(
    state: &AppState,
    scene_id: &str,
    snapshot_id: &str,
    current_content: &str,
) -> Result<crate::snapshots::SnapshotDiff, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    crate::snapshots::compare_snapshot_with_current_internal(
        vault,
        scene_id,
        snapshot_id,
        current_content,
    )
    .map_err(|e| e.to_string())
}

/// Restaura o conteúdo de um snapshot para a cena ativa.
#[tauri::command]
pub fn restore_scene_snapshot(
    state: tauri::State<'_, AppState>,
    scene_id: String,
    snapshot_id: String,
    current_content: String,
) -> Result<RestoreSnapshotResult, String> {
    restore_scene_snapshot_internal(&state, &scene_id, &snapshot_id, &current_content)
}

pub fn restore_scene_snapshot_internal(
    state: &AppState,
    scene_id: &str,
    snapshot_id: &str,
    current_content: &str,
) -> Result<RestoreSnapshotResult, String> {
    let mut guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do estado do cofre.".to_string())?;

    let vault = guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    let (restored, safety_meta) = crate::snapshots::restore_scene_snapshot_internal(
        vault,
        scene_id,
        snapshot_id,
        current_content,
    )
    .map_err(|e| e.to_string())?;

    // Salva o conteúdo restaurado no blob principal da cena
    vault
        .write_blob(scene_id, restored.as_bytes())
        .map_err(|e| e.to_string())?;

    // Reindexa no índice em memória
    if let Ok(mut idx_guard) = state.index.lock() {
        if let Some(index) = idx_guard.as_mut() {
            let (title, kind) = if let Ok(manifest) = vault.manifest() {
                manifest
                    .items
                    .iter()
                    .find(|i| i.id == scene_id)
                    .map(|i| (i.title.clone(), i.kind.clone()))
                    .unwrap_or_else(|| ("Cena".to_string(), "scene".to_string()))
            } else {
                ("Cena".to_string(), "scene".to_string())
            };
            index.index_document_content(scene_id, &title, &kind, &restored);
        }
    }

    Ok(RestoreSnapshotResult {
        restored_content: restored,
        safety_snapshot: safety_meta,
    })
}

// ============================================================================
// COMANDOS DE BUSCA E SUBSTITUIÇÃO GLOBAL (M5-03, M5-04)
// ============================================================================

/// Executa busca textual rápida em memória em todo o cofre.
#[tauri::command]
pub fn global_search(
    state: tauri::State<'_, AppState>,
    query: String,
    match_case: bool,
) -> Result<crate::search_replace::GlobalSearchResponse, String> {
    global_search_internal(&state, &query, match_case)
}

pub fn global_search_internal(
    state: &AppState,
    query: &str,
    match_case: bool,
) -> Result<crate::search_replace::GlobalSearchResponse, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    crate::search_replace::execute_global_search(vault, query, match_case)
        .map_err(|e| e.to_string())
}

/// Gera pré-visualização de substituição global antes de ser aplicada.
#[tauri::command]
pub fn preview_global_replace(
    state: tauri::State<'_, AppState>,
    query: String,
    replacement: String,
    match_case: bool,
) -> Result<crate::search_replace::GlobalReplacePreview, String> {
    preview_global_replace_internal(&state, &query, &replacement, match_case)
}

pub fn preview_global_replace_internal(
    state: &AppState,
    query: &str,
    replacement: &str,
    match_case: bool,
) -> Result<crate::search_replace::GlobalReplacePreview, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    crate::search_replace::preview_global_replace_internal(vault, query, replacement, match_case)
        .map_err(|e| e.to_string())
}

/// Aplica substituição global atômica em todo o projeto.
#[tauri::command]
pub fn apply_global_replace(
    state: tauri::State<'_, AppState>,
    query: String,
    replacement: String,
    match_case: bool,
) -> Result<crate::search_replace::ApplyGlobalReplaceResult, String> {
    apply_global_replace_internal(&state, &query, &replacement, match_case)
}

pub fn apply_global_replace_internal(
    state: &AppState,
    query: &str,
    replacement: &str,
    match_case: bool,
) -> Result<crate::search_replace::ApplyGlobalReplaceResult, String> {
    let mut vault_guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = vault_guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    let mut idx_guard = state
        .index
        .lock()
        .map_err(|_| "Falha ao obter lock do índice.".to_string())?;

    let index = idx_guard
        .as_mut()
        .ok_or_else(|| "Índice indisponível em memória.".to_string())?;

    crate::search_replace::apply_global_replace_internal(
        vault,
        index,
        query,
        replacement,
        match_case,
    )
    .map_err(|e| e.to_string())
}

/// Reverte as alterações de uma substituição global a partir do snapshot de rollback.
#[tauri::command]
pub fn rollback_global_replace(
    state: tauri::State<'_, AppState>,
    snapshot_id: String,
) -> Result<(), String> {
    rollback_global_replace_internal(&state, &snapshot_id)
}

pub fn rollback_global_replace_internal(state: &AppState, snapshot_id: &str) -> Result<(), String> {
    let mut vault_guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = vault_guard
        .as_mut()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    let mut idx_guard = state
        .index
        .lock()
        .map_err(|_| "Falha ao obter lock do índice.".to_string())?;

    let index = idx_guard
        .as_mut()
        .ok_or_else(|| "Índice indisponível em memória.".to_string())?;

    crate::search_replace::rollback_global_replace_internal(vault, index, snapshot_id)
        .map_err(|e| e.to_string())
}

// ============================================================================
// COMANDOS DE BACKUP CIFRADO ROTATIVO (M5-05)
// ============================================================================

/// Executa backup integral cifrado do cofre literário com rotação.
#[tauri::command]
pub fn create_vault_backup(
    state: tauri::State<'_, AppState>,
    destination_dir: String,
    max_backups: Option<usize>,
) -> Result<crate::backup::BackupResult, String> {
    create_vault_backup_internal(&state, &destination_dir, max_backups.unwrap_or(5))
}

pub fn create_vault_backup_internal(
    state: &AppState,
    destination_dir: &str,
    max_backups: usize,
) -> Result<crate::backup::BackupResult, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    let dest_path = PathBuf::from(destination_dir.trim());
    let title = vault
        .manifest()
        .map(|m| m.project_title.clone())
        .unwrap_or_else(|_| "Romance".to_string());

    crate::backup::create_encrypted_backup_internal(
        vault.root_path(),
        &dest_path,
        &title,
        max_backups,
    )
}

/// Lista backups existentes na pasta de destino informada.
#[tauri::command]
pub fn list_vault_backups(
    state: tauri::State<'_, AppState>,
    destination_dir: String,
) -> Result<Vec<crate::backup::BackupInfo>, String> {
    list_vault_backups_internal(&state, &destination_dir)
}

pub fn list_vault_backups_internal(
    state: &AppState,
    destination_dir: &str,
) -> Result<Vec<crate::backup::BackupInfo>, String> {
    let guard = state
        .active_vault
        .lock()
        .map_err(|_| "Falha ao obter lock do cofre.".to_string())?;

    let vault = guard
        .as_ref()
        .ok_or_else(|| "Nenhum cofre aberto no momento.".to_string())?;

    let dest_path = PathBuf::from(destination_dir.trim());
    let title = vault
        .manifest()
        .map(|m| m.project_title.clone())
        .unwrap_or_else(|_| "Romance".to_string());

    crate::backup::list_backups_internal(vault.root_path(), &dest_path, &title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_message() {
        let result = greet("Author");
        assert_eq!(result, "Hello, Author! Welcome to Magrathea Writer.");
    }

    #[test]
    fn test_resolve_vault_path() {
        let base = PathBuf::from("/tmp/projects");
        let resolved = resolve_vault_path(&base, "Meu Romance 1");
        assert_eq!(
            resolved,
            PathBuf::from("/tmp/projects/Meu_Romance_1.magrathea")
        );

        let direct = PathBuf::from("/tmp/projects/Obra.magrathea");
        let resolved_direct = resolve_vault_path(&direct, "Qualquer Titulo");
        assert_eq!(resolved_direct, direct);
    }

    #[test]
    fn test_get_default_vault_dir_not_empty() {
        let dir = get_default_vault_dir();
        assert!(!dir.is_empty());
    }

    #[test]
    fn test_vault_commands_lifecycle_and_zeroization() {
        let state = AppState::default();
        let unique_suffix: u64 = rand::random();
        let temp_dir = std::env::temp_dir().join(format!("test_cmd_lifecycle_{unique_suffix}"));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let fixture_key_str = "correto-cavalo-bateria-grampo-romance";

        // 1. Criação do cofre
        let create_res = create_vault_internal(
            &state,
            &temp_dir.display().to_string(),
            "O Guia Galactico",
            fixture_key_str,
        )
        .expect("deve criar cofre");

        assert_eq!(create_res.project_title, "O Guia Galactico");
        assert!(!create_res.recovery_key_token.is_empty());

        // 2. Status após criação (aberto)
        let status = get_vault_status_internal(&state).expect("deve obter status");
        assert!(status.is_open);
        assert!(!status.is_locked);
        assert_eq!(status.project_title.as_deref(), Some("O Guia Galactico"));

        // 3. Bloqueio com descarte de sessão e zeroização em memória
        lock_vault_internal(&state).expect("deve bloquear");

        let locked_status = get_vault_status_internal(&state).expect("deve obter status");
        assert!(!locked_status.is_open);
        assert!(locked_status.is_locked);
        assert_eq!(
            locked_status.vault_path.as_deref(),
            Some(create_res.vault_path.as_str())
        );
        assert_eq!(
            locked_status.project_title.as_deref(),
            Some("O Guia Galactico")
        );

        // 4. Tentativa de desbloqueio com senha errada (deve falhar com segurança)
        let wrong_res = open_vault_with_passphrase_internal(
            &state,
            &create_res.vault_path,
            "senha-errada-incorreta",
        );
        assert!(wrong_res.is_err());

        // 5. Desbloqueio com frase-senha correta
        let unlock_res =
            open_vault_with_passphrase_internal(&state, &create_res.vault_path, fixture_key_str)
                .expect("deve desbloquear com frase-senha");

        assert!(unlock_res.is_open);
        assert!(!unlock_res.is_locked);

        // 6. Novo bloqueio
        lock_vault_internal(&state).expect("deve bloquear");

        // 7. Desbloqueio com chave de recuperação de emergência
        let recovery_res = open_vault_with_recovery_key_internal(
            &state,
            &create_res.vault_path,
            &create_res.recovery_key_token,
        )
        .expect("deve desbloquear com chave de recuperacao");

        assert!(recovery_res.is_open);
        assert!(!recovery_res.is_locked);

        // 7.1. Gravação atômica cifrada de conteúdo de cena
        let save_res = save_scene_content_internal(
            &state,
            "cena_cap1",
            "No princípio, o Universo foi criado. Isso irritou muita gente.",
        )
        .expect("deve salvar cena com sucesso");
        assert_eq!(save_res.scene_id, "cena_cap1");

        // 7.2. Leitura e decifragem de conteúdo de cena
        let loaded =
            load_scene_content_internal(&state, "cena_cap1").expect("deve carregar cena decifrada");
        assert_eq!(
            loaded,
            "No princípio, o Universo foi criado. Isso irritou muita gente."
        );

        // 8. Fechamento total do cofre (close_vault)
        close_vault_internal(&state).expect("deve fechar cofre");
        let closed_status = get_vault_status_internal(&state).expect("deve obter status");
        assert!(!closed_status.is_open);
        assert!(closed_status.is_locked);
        assert!(closed_status.vault_path.is_none());
        assert!(closed_status.project_title.is_none());

        // Limpeza
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
