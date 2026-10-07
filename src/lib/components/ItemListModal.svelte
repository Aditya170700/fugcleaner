<script lang="ts">
  import type { Category, ScanItem } from '$lib/types';
  import { formatBytes } from '$lib/utils/format';
  import { id } from '$lib/i18n/id';
  import { api } from '$lib/api';
  import { X, FolderOpen, ExternalLink, HardDrive } from 'lucide-svelte';

  interface Props {
    category: Category;
    items: ScanItem[];
    onClose: () => void;
  }

  let { category, items, onClose }: Props = $props();

  const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPod|iPad/i.test(navigator.userAgent);
  const fileManagerLabel = isMac ? 'Finder' : 'Folder';

  const totalBytes = $derived(items.reduce((acc, item) => acc + item.bytes, 0));

  async function handleOpenInFinder(itemId: string) {
    try {
      await api.openInFileManager(itemId);
    } catch (e) {
      console.error('Failed to open in file manager:', e);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="modal-backdrop"
  onclick={onClose}
  role="presentation"
>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-card"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    tabindex="-1"
  >
    <div class="modal-header">
      <div class="header-info">
        <h2 id="modal-title">{category.name}</h2>
        <p class="header-desc">{category.description}</p>
      </div>

      <button class="btn-close" onclick={onClose} title="Tutup">
        <X size={20} />
      </button>
    </div>

    <div class="modal-summary">
      <span class="summary-items">{items.length} item ditemukan</span>
      <span class="summary-total">{formatBytes(totalBytes)} total</span>
    </div>

    <div class="item-list-container">
      {#if items.length === 0}
        <div class="empty-items">
          <p>{id.dashboard.noItemsFound}</p>
        </div>
      {:else}
        {#each items as item}
          <div class="item-row">
            <div class="item-info">
              <span class="item-path" title={item.path}>{item.path}</span>
              {#if item.note}
                <span class="badge badge-muted item-note">{item.note}</span>
              {/if}
            </div>

            <div class="item-actions">
              <span class="item-size">{formatBytes(item.bytes)}</span>

              {#if item.path && !item.path.startsWith('Command:')}
                <button
                  class="btn btn-outline btn-finder"
                  onclick={() => handleOpenInFinder(item.id)}
                  title={isMac ? id.dashboard.showInFinder : 'Buka di Folder'}
                >
                  <ExternalLink size={14} />
                  <span>{fileManagerLabel}</span>
                </button>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="modal-footer">
      <button class="btn btn-secondary" onclick={onClose}>
        {id.common.close}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 24px;
    animation: fadeIn 0.15s ease;
  }

  .modal-card {
    background-color: var(--bg-card);
    border: 1px solid var(--border-hover);
    border-radius: 14px;
    width: 100%;
    max-width: 680px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-lg);
    animation: slideUp 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    overflow: hidden;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from { transform: translateY(16px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  .modal-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 20px 24px 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .header-info h2 {
    font-size: 1.25rem;
    font-weight: 700;
  }

  .header-desc {
    font-size: 0.825rem;
    color: var(--text-secondary);
    margin-top: 2px;
  }

  .btn-close {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .btn-close:hover {
    background-color: var(--bg-subtle);
    color: var(--text-primary);
  }

  .modal-summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 24px;
    background-color: var(--bg-subtle);
    font-size: 0.8rem;
    font-weight: 600;
    border-bottom: 1px solid var(--border-color);
  }

  .summary-items {
    color: var(--text-secondary);
  }

  .summary-total {
    color: var(--accent-primary);
  }

  .item-list-container {
    padding: 8px 16px;
    overflow-y: auto;
    flex: 1;
    max-height: 50vh;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .item-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 12px;
    border-radius: 8px;
    background-color: var(--bg-card);
    border: 1px solid var(--border-color);
    transition: background-color 0.15s ease;
  }

  .item-row:hover {
    background-color: var(--bg-card-hover);
  }

  .item-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .item-path {
    font-size: 0.8rem;
    font-family: var(--font-mono);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-note {
    align-self: flex-start;
    font-size: 0.7rem;
  }

  .item-actions {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .item-size {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .btn-finder {
    padding: 4px 10px;
    font-size: 0.75rem;
  }

  .empty-items {
    padding: 40px 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 16px 24px;
    border-top: 1px solid var(--border-color);
    background-color: var(--bg-card);
  }
</style>
