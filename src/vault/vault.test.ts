import { describe, it, expect, beforeEach } from 'vitest';
import { vault } from './vault.svelte';
import { assessPasswordStrength } from '../crypto/password-strength';

describe('Vault Creation & Lifecycle Flow (Frontend M1-02)', () => {
  beforeEach(() => {
    vault.isOpen = false;
    vault.isLocked = true;
    vault.vaultId = null;
    vault.projectTitle = null;
    vault.vaultPath = null;
    vault.recoveryKeyToken = null;
    vault.errorMessage = null;
  });

  it('inicia com cofre fechado e bloqueado', () => {
    expect(vault.isOpen).toBe(false);
    expect(vault.isLocked).toBe(true);
    expect(vault.recoveryKeyToken).toBeNull();
  });

  it('avalia critérios de senha forte antes de permitir submissão', () => {
    const weakPass = '12345';
    const weakRes = assessPasswordStrength(weakPass);
    expect(weakRes.isAcceptable).toBe(false);

    const strongPass = 'correto cavalo bateria grampo';
    const strongRes = assessPasswordStrength(strongPass);
    expect(strongRes.isAcceptable).toBe(true);
    expect(strongRes.score).toBeGreaterThanOrEqual(2);
  });

  it('cria o cofre e armazena o token de recuperação para exibição única', async () => {
    const response = await vault.createVault(
      '/home/author/Documents',
      'Meu Romance Especial',
      'frase-senha-super-segura-e-longa-123'
    );

    expect(response.vault_id).toBeDefined();
    expect(response.project_title).toBe('Meu Romance Especial');
    expect(response.recovery_key_token).toBeDefined();
    expect(response.recovery_key_token.length).toBeGreaterThan(32);

    expect(vault.isOpen).toBe(true);
    expect(vault.isLocked).toBe(false);
    expect(vault.projectTitle).toBe('Meu Romance Especial');
    expect(vault.recoveryKeyToken).toBe(response.recovery_key_token);
  });

  it('ao bloquear o cofre, zeroiza e descarta a chave de recuperação', async () => {
    await vault.createVault(
      '/home/author/Documents',
      'Romance Bloqueio',
      'frase-senha-longa-segura'
    );

    expect(vault.isOpen).toBe(true);
    expect(vault.recoveryKeyToken).not.toBeNull();

    await vault.lockVault();
    expect(vault.isLocked).toBe(true);
    expect(vault.recoveryKeyToken).toBeNull();
  });
});
