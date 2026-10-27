//! Comandos IPC expostos para o frontend via ponte Tauri.

use crate::crypto::SecretKey;
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

    if passphrase.len() < 10 {
        return Err("A senha mestra deve conter no mínimo 10 caracteres.".to_string());
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

    vault
        .save_manifest(&timestamp_iso)
        .map_err(|e| format!("Falha ao salvar manifesto: {e}"))?;

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
    if new_passphrase.len() < 10 {
        return Err("A nova senha mestra deve conter no mínimo 10 caracteres.".to_string());
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
