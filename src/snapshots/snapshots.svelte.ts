/**
//! Store reativa para snapshots e histórico de versões da cena (M5-01, M5-02).
//!
//! Controla listagem de revisões, comparação visual de diff e restauração atômica.
*/

import {
  safeInvoke,
  type SnapshotMeta,
  type SnapshotDiff,
  type RestoreSnapshotResult,
} from '../tauri/bridge';

class SnapshotsStore {
  snapshots = $state<SnapshotMeta[]>([]);
  activeDiff = $state<SnapshotDiff | null>(null);
  selectedSnapshot = $state<SnapshotMeta | null>(null);
  isLoading = $state(false);
  isSaving = $state(false);
  isRestoring = $state(false);
  errorMessage = $state<string | null>(null);
  successMessage = $state<string | null>(null);

  /**
   * Carrega todos os snapshots para a cena ativa.
   */
  async loadSnapshots(sceneId: string): Promise<void> {
    if (!sceneId) return;
    this.isLoading = true;
    this.errorMessage = null;

    try {
      this.snapshots = await safeInvoke<SnapshotMeta[]>('list_scene_snapshots', {
        sceneId,
        scene_id: sceneId,
      });
    } catch (err) {
      this.errorMessage = String(err);
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Cria um snapshot manual da cena ativa com rótulo descritivo.
   */
  async createSnapshot(
    sceneId: string,
    label: string,
    currentContent: string
  ): Promise<SnapshotMeta | null> {
    if (!sceneId) return null;
    this.isSaving = true;
    this.errorMessage = null;
    this.successMessage = null;

    try {
      const meta = await safeInvoke<SnapshotMeta>('create_scene_snapshot', {
        sceneId,
        scene_id: sceneId,
        label,
        trigger: 'manual',
        currentContent,
        current_content: currentContent,
      });

      this.snapshots = [meta, ...this.snapshots];
      this.successMessage = 'Snapshot salvo com sucesso.';
      return meta;
    } catch (err) {
      this.errorMessage = String(err);
      return null;
    } finally {
      this.isSaving = false;
    }
  }

  /**
   * Compara um snapshot com o conteúdo atual em edição no editor.
   */
  async compareWithCurrent(
    sceneId: string,
    snapshotId: string,
    currentContent: string
  ): Promise<SnapshotDiff | null> {
    this.isLoading = true;
    this.errorMessage = null;

    try {
      const diff = await safeInvoke<SnapshotDiff>('compare_snapshot_with_current', {
        sceneId,
        scene_id: sceneId,
        snapshotId,
        snapshot_id: snapshotId,
        currentContent,
        current_content: currentContent,
      });

      this.activeDiff = diff;
      this.selectedSnapshot = this.snapshots.find((s) => s.id === snapshotId) || null;
      return diff;
    } catch (err) {
      this.errorMessage = String(err);
      return null;
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Restaura um snapshot para a cena ativa gerando snapshot de segurança antes.
   */
  async restoreSnapshot(
    sceneId: string,
    snapshotId: string,
    currentContent: string
  ): Promise<string | null> {
    this.isRestoring = true;
    this.errorMessage = null;

    try {
      const result = await safeInvoke<RestoreSnapshotResult>('restore_scene_snapshot', {
        sceneId,
        scene_id: sceneId,
        snapshotId,
        snapshot_id: snapshotId,
        currentContent,
        current_content: currentContent,
      });

      // Atualiza a lista com o novo snapshot de segurança criado automaticamente
      this.snapshots = [result.safety_snapshot, ...this.snapshots];
      this.activeDiff = null;
      this.successMessage = 'Versão restaurada com sucesso.';
      return result.restored_content;
    } catch (err) {
      this.errorMessage = String(err);
      return null;
    } finally {
      this.isRestoring = false;
    }
  }

  /**
   * Fecha o visualizador de comparação diff.
   */
  closeDiff(): void {
    this.activeDiff = null;
    this.selectedSnapshot = null;
  }
}

export const snapshots = new SnapshotsStore();
