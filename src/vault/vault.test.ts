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

  it('desbloqueia cofre com frase-senha com sucesso', async () => {
    vault.vaultPath = '/home/author/Documents/Romance.magrathea';
    vault.isLocked = true;

    const res = await vault.openWithPassphrase('frase-senha-correta');
    expect(res.is_open).toBe(true);
    expect(vault.isOpen).toBe(true);
    expect(vault.isLocked).toBe(false);
    expect(vault.projectTitle).toBe('Romance Teste');
  });

  it('falha ao tentar desbloquear com senha errada', async () => {
    vault.vaultPath = '/home/author/Documents/Romance.magrathea';
    vault.isLocked = true;

    await expect(vault.openWithPassphrase('wrong-passphrase')).rejects.toThrow();
    expect(vault.isLocked).toBe(true);
    expect(vault.errorMessage).toContain('Senha incorreta');
  });

  it('desbloqueia cofre com chave de recuperação', async () => {
    vault.vaultPath = '/home/author/Documents/Romance.magrathea';
    vault.isLocked = true;

    const res = await vault.openWithRecoveryKey('valid-recovery-token');
    expect(res.is_open).toBe(true);
    expect(vault.isOpen).toBe(true);
    expect(vault.isLocked).toBe(false);
  });

  it('desvincula e fecha o cofre ao chamar closeVault', async () => {
    vault.isOpen = true;
    vault.isLocked = false;
    vault.vaultPath = '/home/author/Documents/Romance.magrathea';
    vault.projectTitle = 'Romance Teste';

    await vault.closeVault();
    expect(vault.isOpen).toBe(false);
    expect(vault.isLocked).toBe(true);
    expect(vault.vaultPath).toBeNull();
    expect(vault.projectTitle).toBeNull();
  });

  it('grava e carrega cena de forma atômica e cifrada quando o cofre está aberto', async () => {
    vault.isOpen = true;
    vault.isLocked = false;

    const saveRes = await vault.saveScene(
      'cena_01',
      'Era uma vez um autor que confiava na criptografia offline.'
    );
    expect(saveRes.scene_id).toBe('cena_01');
    expect(vault.lastSavedAt).toBeDefined();

    const loaded = await vault.loadScene('cena_01');
    expect(loaded).toBe('# Mock Scene Content');
  });

  it('impede gravação de cena quando o cofre está bloqueado', async () => {
    vault.isOpen = true;
    vault.isLocked = true;

    await expect(vault.saveScene('cena_01', 'Texto')).rejects.toThrow(
      'Cofre não está aberto ou está bloqueado.'
    );
  });
});
