<script lang="ts">
  import { formatBytes } from '$lib/utils/format';
  import { id } from '$lib/i18n/id';
  import type { Method, Risk } from '$lib/types';
  import {
    AlertTriangle,
    Trash2,
    ShieldCheck,
    X,
    Info,
    CheckCircle2,
    Sparkles,
  } from 'lucide-svelte';

  export interface ItemSummary {
    id: string;
    name: string;
    count: number;
    bytes: number;
    method: Method;
    risk: Risk;
  }

  interface Props {
    open: boolean;
    title?: string;
    itemsSummary: ItemSummary[];
    totalBytes: number;
    dryRun?: boolean;
    onConfirm: () => void;
    onClose: () => void;
  }

  let {
    open = false,
    title = id.confirmModal.title,
    itemsSummary = [],
    totalBytes = 0,
    dryRun = true,
    onConfirm,
    onClose,
  }: Props = $props();

  const hasDeleteMethod = $derived(
    itemsSummary.some((item) => item.method === 'delete')
  );

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && open) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={onClose}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="modal-card"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      aria-labelledby="confirm-modal-title"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="modal-header">
        <div class="header-left">
          <div class="icon-avatar" class:danger={!dryRun && hasDeleteMethod}>
            {#if dryRun}
              <ShieldCheck size={22} class="text-accent" />
            {:else if hasDeleteMethod}
              <AlertTriangle size={22} class="text-caution" />
            {:else}
              <Trash2 size={22} class="text-accent" />
            {/if}
          </div>
          <div>
            <h3 id="confirm-modal-title" class="modal-title">{title}</h3>
            <p class="modal-subtitle">
              {#if dryRun}
                Mode Simulasi — Aman untuk dijalankan
              {:else}
                Pastikan Anda memeriksa daftar item di bawah
              {/if}
            </p>
          </div>
        </div>
        <button class="btn btn-ghost btn-icon" onclick={onClose} aria-label="Tutup">
          <X size={18} />
        </button>
      </div>

      <div class="modal-body">
        {#if dryRun}
          <div class="banner banner-info">
            <ShieldCheck size={18} />
            <div class="banner-text">
              <strong>Mode DRY RUN Aktif:</strong> Tidak ada file yang akan dihapus dari disk. Operasi ini hanya akan mensimulasikan pembersihan dan menghitung ruang disk yang dibebaskan.
            </div>
          </div>
        {:else if hasDeleteMethod}
          <div class="banner banner-caution">
            <AlertTriangle size={18} />
            <div class="banner-text">
              <strong>Peringatan Penghapusan Permanen:</strong> Sebagian item yang dipilih (mis. cache/build target) akan dihapus secara <em>permanen</em> dan tidak dapat dikembalikan dari Trash.
            </div>
          </div>
        {/if}

        <div class="summary-section">
          <div class="section-heading">
            <span>Rincian Item yang Akan Dibersihkan</span>
            <span class="total-items-badge">{itemsSummary.reduce((acc, curr) => acc + curr.count, 0)} item</span>
          </div>

          <div class="summary-list">
            {#each itemsSummary as item (item.id)}
              <div class="summary-row">
                <div class="item-name-group">
                  <span class="item-name">{item.name}</span>
                  <span class="item-count">({item.count} item)</span>
                </div>

                <div class="item-meta-group">
                  {#if item.method === 'trash'}
                    <span class="method-tag tag-trash" title="Akan dipindahkan ke Trash (dapat di-restore)">
                      Trash
                    </span>
                  {:else if item.method === 'delete'}
                    <span class="method-tag tag-delete" title="Akan dihapus permanen dari disk">
                      Permanen
                    </span>
                  {:else if item.method === 'command'}
                    <span class="method-tag tag-cmd" title="Dijalankan via perintah CLI">
                      CLI
                    </span>
                  {/if}

                  <span class="item-bytes">{formatBytes(item.bytes)}</span>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <div class="total-bar">
          <span class="total-label">Total Ruang yang Dibebaskan:</span>
          <span class="total-value">{formatBytes(totalBytes)}</span>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={onClose}>
          {id.confirmModal.cancelButton}
        </button>
        <button
          class="btn {dryRun ? 'btn-primary' : 'btn-danger'}"
          onclick={onConfirm}
        >
          {#if dryRun}
            <ShieldCheck size={16} />
            <span>Jalankan Simulasi</span>
          {:else}
            <Trash2 size={16} />
            <span>{id.confirmModal.confirmButton}</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 16px;
    animation: fadeIn 0.15s ease-out;
  }

  .modal-card {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    width: 100%;
    max-width: 540px;
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.22);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slideUp 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border-color);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .icon-avatar {
    width: 44px;
    height: 44px;
    border-radius: var(--radius-md);
    background: rgba(37, 99, 235, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-avatar.danger {
    background: rgba(220, 38, 38, 0.1);
  }

  .modal-title {
    font-size: 1.15rem;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
  }

  .modal-subtitle {
    font-size: 0.8rem;
    color: var(--text-muted);
    margin: 2px 0 0 0;
  }

  .modal-body {
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: 65vh;
    overflow-y: auto;
  }

  .banner {
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    line-height: 1.45;
  }

  .banner-info {
    background: rgba(37, 99, 235, 0.08);
    border: 1px solid rgba(37, 99, 235, 0.2);
    color: var(--text-primary);
  }

  .banner-caution {
    background: rgba(217, 119, 6, 0.08);
    border: 1px solid rgba(217, 119, 6, 0.25);
    color: var(--text-primary);
  }

  .banner-text {
    flex: 1;
  }

  .summary-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .section-heading {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .total-items-badge {
    background: var(--bg-hover);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .summary-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 8px 12px;
    max-height: 220px;
    overflow-y: auto;
  }

  .summary-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 4px;
    border-bottom: 1px solid var(--border-color);
    font-size: 0.875rem;
  }

  .summary-row:last-child {
    border-bottom: none;
  }

  .item-name-group {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
  }

  .item-name {
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-count {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .item-meta-group {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .method-tag {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    text-transform: uppercase;
  }

  .tag-trash {
    background: rgba(37, 99, 235, 0.1);
    color: var(--accent-primary);
  }

  .tag-delete {
    background: rgba(220, 38, 38, 0.1);
    color: var(--color-danger);
  }

  .tag-cmd {
    background: rgba(147, 51, 234, 0.1);
    color: #9333ea;
  }

  .item-bytes {
    font-weight: 600;
    color: var(--text-primary);
    min-width: 60px;
    text-align: right;
  }

  .total-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--bg-hover);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
  }

  .total-label {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .total-value {
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--accent-primary);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 12px;
    padding: 16px 24px;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(8px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }
</style>
