import { describe, it, expect, beforeEach } from 'vitest';
import { vault } from './vault.svelte';

describe('Vault Security & Keyring Flow (M1-12 & M1-13)', () => {
  beforeEach(async () => {
    vault.isOpen = true;
    vault.isLocked = false;
    vault.vaultPath = '/home/author/Documents/Romance.magrathea';
    vault.projectTitle = 'Romance Teste';
    await vault.checkKeyringStatus();
  });

  it('permite alterar a frase-senha mestra com sucesso', async () => {
    await expect(
      vault.changePassphrase('SenhaAntiga!2026', 'NovaSenhaUltraForte!2026')
    ).resolves.toBeUndefined();
  });

  it('falha ao tentar alterar para senha curta com menos de 10 caracteres', async () => {
    await expect(
      vault.changePassphrase('SenhaAntiga!2026', 'curta')
    ).rejects.toThrow();
  });

  it('recifra o cofre integralmente e retorna novo token de recuperação', async () => {
    const newRecoveryKey = await vault.reencryptVault('SenhaAtual!2026');
    expect(newRecoveryKey).toBeDefined();
    expect(typeof newRecoveryKey).toBe('string');
    expect(vault.newRecoveryKeyAfterReencrypt).toBe(newRecoveryKey);
  });

  it('gerencia ciclo de vida do chaveiro do SO (ativar, consultar, desativar)', async () => {
    expect(vault.hasKeyringEnabled).toBe(false);

    await vault.saveToKeyring('SenhaAtual!2026');
    expect(vault.hasKeyringEnabled).toBe(true);

    const status = await vault.checkKeyringStatus();
    expect(status).toBe(true);

    await vault.removeFromKeyring();
    expect(vault.hasKeyringEnabled).toBe(false);
  });

  it('desbloqueia o cofre automaticamente via chaveiro do SO', async () => {
    await vault.saveToKeyring('SenhaAtual!2026');
    vault.isLocked = true;

    const res = await vault.openWithKeyring();
    expect(res.is_locked).toBe(false);
    expect(vault.isLocked).toBe(false);
  });

  it('alterna o modal de segurança do cofre', () => {
    expect(vault.isSecurityModalOpen).toBe(false);
    vault.toggleSecurityModal(true);
    expect(vault.isSecurityModalOpen).toBe(true);
    vault.toggleSecurityModal(false);
    expect(vault.isSecurityModalOpen).toBe(false);
  });
});
