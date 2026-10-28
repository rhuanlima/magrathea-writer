<script lang="ts">
  import { t } from '../i18n';
  import { safeInvoke, type EntityImageData } from '../tauri/bridge';
  import { worldbuilding } from './worldbuilding.svelte';

  interface Props {
    entityId: string;
  }

  let { entityId }: Props = $props();

  let fileInput: HTMLInputElement | null = $state(null);
  let isUploading = $state(false);
  let uploadError = $state<string | null>(null);
  let selectedImage = $state<EntityImageData | null>(null);

  async function handleFileSelected(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    if (file.size > 5 * 1024 * 1024) {
      uploadError = t('worldbuilding.imageTooLarge');
      return;
    }

    isUploading = true;
    uploadError = null;

    try {
      await worldbuilding.importImage(entityId, file);
      if (fileInput) fileInput.value = '';
    } catch (err) {
      uploadError = String(err);
    } finally {
      isUploading = false;
    }
  }

  async function viewImage(imageId: string) {
    try {
      const data = await safeInvoke<EntityImageData>('load_entity_image', {
        image_id: imageId,
      });
      selectedImage = data;
    } catch (err) {
      console.error('Falha ao decifrar imagem:', err);
    }
  }

  async function deleteImage(imageId: string) {
    if (confirm(t('worldbuilding.confirmDeleteImage'))) {
      try {
        await worldbuilding.deleteImage(entityId, imageId);
        if (selectedImage?.meta.id === imageId) {
          selectedImage = null;
        }
      } catch (err) {
        console.error('Falha ao excluir imagem:', err);
      }
    }
  }
</script>

<div class="gallery-container">
  <header class="gallery-header">
    <div class="header-left">
      <span class="gallery-icon">🖼️</span>
      <h3 class="gallery-title">{t('worldbuilding.galleryTitle')}</h3>
      <span class="gallery-count font-mono">{worldbuilding.images.length}</span>
    </div>

    <div class="header-actions">
      <input
        type="file"
        accept="image/png,image/jpeg,image/webp,image/gif"
        bind:this={fileInput}
        onchange={handleFileSelected}
        style="display: none;"
      />
      <button
        type="button"
        class="upload-btn"
        onclick={() => fileInput?.click()}
        disabled={isUploading}
        title={t('worldbuilding.uploadImage')}
      >
        {isUploading ? t('common.saving') : `+ ${t('worldbuilding.uploadImage')}`}
      </button>
    </div>
  </header>

  {#if uploadError}
    <div class="upload-error-msg">{uploadError}</div>
  {/if}

  {#if worldbuilding.images.length === 0}
    <div class="empty-gallery">
      <p>{t('worldbuilding.noImagesYet')}</p>
      <span class="hint">{t('worldbuilding.imagesFormatHint')}</span>
    </div>
  {:else}
    <div class="thumbnails-grid">
      {#each worldbuilding.images as img}
        <div class="thumbnail-card">
          <button
            type="button"
            class="thumb-btn"
            onclick={() => viewImage(img.id)}
            title={img.file_name}
          >
            <span class="file-icon">📷</span>
            <span class="file-name">{img.file_name}</span>
          </button>
          <button
            type="button"
            class="delete-thumb-btn"
            onclick={() => deleteImage(img.id)}
            title={t('common.delete')}
          >
            ✕
          </button>
        </div>
      {/each}
    </div>
  {/if}

  <!-- Modal / visualizador de imagem decifrada em memória -->
  {#if selectedImage}
    <div
      class="image-modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-labelledby="img-preview-title"
    >
      <div class="image-modal-card">
        <header class="preview-header">
          <span id="img-preview-title" class="preview-title">
            🔒 {selectedImage.meta.file_name} ({selectedImage.meta.mime_type})
          </span>
          <button
            type="button"
            class="close-preview-btn"
            onclick={() => (selectedImage = null)}
            aria-label={t('common.close')}
          >
            ✕
          </button>
        </header>
        <div class="preview-content">
          <img
            src="data:{selectedImage.meta.mime_type};base64,{selectedImage.base64_data}"
            alt={selectedImage.meta.file_name}
            class="decrypted-image"
          />
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .gallery-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .gallery-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: var(--space-xs);
    border-bottom: 1px solid var(--color-paper-border);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .gallery-icon {
    font-size: 14px;
  }

  .gallery-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0;
  }

  .gallery-count {
    background-color: var(--color-olive-subtle);
    color: var(--color-olive);
    padding: 1px 6px;
    border-radius: var(--rounded-full);
    font-size: 11px;
    font-weight: 600;
  }

  .upload-btn {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    padding: 2px 8px;
    font-size: 11px;
    font-weight: 500;
    color: var(--color-ink-charcoal);
    cursor: pointer;
  }

  .upload-btn:hover {
    background-color: var(--color-paper-hover);
    border-color: var(--color-olive);
  }

  .upload-error-msg {
    background-color: #fdf2f2;
    color: var(--color-error);
    padding: 6px 8px;
    border-radius: var(--rounded-sm);
    font-size: 12px;
  }

  .empty-gallery {
    padding: var(--space-md) var(--space-sm);
    text-align: center;
    color: var(--color-ink-muted);
    font-size: 13px;
  }

  .hint {
    display: block;
    font-size: 11px;
    color: var(--color-ink-faded);
    margin-top: 4px;
  }

  .thumbnails-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: var(--space-xs);
  }

  .thumbnail-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    padding: 6px 8px;
    overflow: hidden;
  }

  .thumb-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    cursor: pointer;
    overflow: hidden;
    text-align: left;
    padding: 0;
    flex: 1;
  }

  .file-icon {
    font-size: 14px;
  }

  .file-name {
    font-size: 12px;
    color: var(--color-ink-charcoal);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .thumb-btn:hover .file-name {
    color: var(--color-olive);
  }

  .delete-thumb-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    color: var(--color-ink-faded);
    font-size: 12px;
    padding: 2px;
  }

  .delete-thumb-btn:hover {
    color: var(--color-error);
  }

  .image-modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(30, 35, 29, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 110;
    padding: var(--space-md);
  }

  .image-modal-card {
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    box-shadow: var(--shadow-modal);
    max-width: 90vw;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
    background-color: var(--color-paper-surface);
    border-bottom: 1px solid var(--color-paper-border);
  }

  .preview-title {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-ink-charcoal);
  }

  .close-preview-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 14px;
    color: var(--color-ink-muted);
  }

  .preview-content {
    padding: var(--space-md);
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: auto;
    background-color: var(--color-paper-base);
  }

  .decrypted-image {
    max-width: 100%;
    max-height: 70vh;
    object-fit: contain;
    border-radius: var(--rounded-default);
  }

  .font-mono {
    font-family: var(--font-mono);
  }
</style>
