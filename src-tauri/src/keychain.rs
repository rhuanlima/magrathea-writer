//! Módulo de integração opcional com o chaveiro de credenciais do SO (M1-13).
//!
//! Fornece armazenamento seguro e desacoplado de senhas no:
//! - macOS Keychain
//! - Windows Credential Manager
//! - Linux Secret Service / libsecret
//!
//! Em conformidade estrita com `docs/SECURITY.md §3.1`:
//! - Desligado por padrão, exige ativação explícita do usuário.
//! - O item é armazenado por identificador único do cofre (`vault:<vault_id>`).
//! - Ao ser desativado pelo usuário, o item é permanentemente excluído do chaveiro.

const SERVICE_NAME: &str = "magrathea-writer";

/// Salva uma credencial do cofre no chaveiro do sistema operacional.
pub fn save_vault_secret(vault_id: &str, secret: &str) -> Result<(), String> {
    let username = format!("vault:{vault_id}");
    let entry = keyring::Entry::new(SERVICE_NAME, &username)
        .map_err(|e| format!("Falha ao instanciar entrada no chaveiro do SO: {e}"))?;

    entry
        .set_password(secret)
        .map_err(|e| format!("Falha ao gravar no chaveiro do SO: {e}"))?;

    Ok(())
}

/// Recupera o segredo do cofre armazenado no chaveiro do sistema operacional, se existir.
pub fn get_vault_secret(vault_id: &str) -> Result<Option<String>, String> {
    let username = format!("vault:{vault_id}");
    let entry = keyring::Entry::new(SERVICE_NAME, &username)
        .map_err(|e| format!("Falha ao instanciar entrada no chaveiro do SO: {e}"))?;

    match entry.get_password() {
        Ok(pass) => Ok(Some(pass)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Falha ao consultar chaveiro do SO: {e}")),
    }
}

/// Exclui o segredo do cofre armazenado no chaveiro do sistema operacional.
pub fn delete_vault_secret(vault_id: &str) -> Result<(), String> {
    let username = format!("vault:{vault_id}");
    let entry = match keyring::Entry::new(SERVICE_NAME, &username) {
        Ok(entry) => entry,
        Err(_) => return Ok(()),
    };

    match entry.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Falha ao remover item do chaveiro do SO: {e}")),
    }
}

/// Verifica se existe credencial salva no chaveiro para o cofre informado.
pub fn has_vault_secret(vault_id: &str) -> Result<bool, String> {
    let secret_opt = get_vault_secret(vault_id)?;
    Ok(secret_opt.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keychain_crud_lifecycle() {
        let test_vault_id = format!("test-vault-{}", rand::random::<u64>());
        let test_secret = "SenhaSuperSecreta!2026";

        // Em ambientes de integração contínua sem interface gráfica (ex: Linux runner headless
        // sem daemon D-Bus org.freedesktop.secrets ativo), a chamada ao chaveiro do SO retorna
        // um erro gracioso de plataforma em vez de panic.
        match has_vault_secret(&test_vault_id) {
            Ok(false) => {
                match save_vault_secret(&test_vault_id, test_secret) {
                    Ok(()) => {
                        assert!(has_vault_secret(&test_vault_id).unwrap());
                        assert_eq!(
                            get_vault_secret(&test_vault_id).unwrap().as_deref(),
                            Some(test_secret)
                        );
                        delete_vault_secret(&test_vault_id).expect("falha ao deletar");
                        assert!(!has_vault_secret(&test_vault_id).unwrap());
                    }
                    Err(e) => {
                        // Ambientes restritos / sandbox macOS / headless CI
                        assert!(
                            e.contains("authorization was denied")
                                || e.contains("Platform secure storage")
                                || e.contains("DBus"),
                            "Erro inesperado ao salvar no chaveiro: {e}"
                        );
                    }
                }
            }
            Ok(true) => panic!("não deveria existir entrada para ID aleatório"),
            Err(e) => {
                assert!(
                    e.contains("Falha ao consultar chaveiro") || e.contains("DBus"),
                    "Erro inesperado no chaveiro: {e}"
                );
            }
        }
    }
}
