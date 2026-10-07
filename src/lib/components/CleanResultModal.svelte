<script lang="ts">
  import { formatBytes } from '$lib/utils/format';
  import { id } from '$lib/i18n/id';
  import type { CleanResult } from '$lib/types';
  import {
    CheckCircle2,
    AlertCircle,
    ShieldCheck,
    X,
    ChevronDown,
    ChevronRight,
    Sparkles,
  } from 'lucide-svelte';

  interface Props {
    open: boolean;
    result: CleanResult | null;
    onClose: () => void;
  }

  let {
    open = false,
    result = null,
    onClose,
  }: Props = $props();

  let showFailedDetails = $state(false);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && open) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open && result}
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
      onclick={(e) => e.stopPropagation()}
    >
      <div class="result-header">
        <div class="icon-avatar" class:dry-run={result.dryRun} class:has-failed={result.failed.length > 0 && result.succeeded.length === 0}>
          {#if result.dryRun}
            <ShieldCheck size={32} class="text-accent" />
          {:else if result.failed.length > 0 && result.succeeded.length === 0}
            <AlertCircle size={32} class="text-danger" />
          {:else}
            <CheckCircle2 size={32} class="text-safe" />
          {/if}
        </div>

        <h3 class="result-title">
          {#if result.dryRun}
            Simulasi Pembersihan Selesai
          {:else if result.failed.length > 0 && result.succeeded.length === 0}
            Pembersihan Gagal
          {:else}
            Pembersihan Berhasil!
          {/if}
        </h3>

        <div class="freed-amount">
          <span class="freed-prefix">
            {result.dryRun ? 'Potensi Ruang Bebas:' : 'Berhasil Membebaskan:'}
          </span>
          <span class="freed-value">{formatBytes(result.freedBytes)}</span>
        </div>
      </div>

      <div class="result-body">
        {#if result.dryRun}
          <div class="banner banner-info">
            <ShieldCheck size={16} class="banner-icon" />
            <div class="banner-text">
              Laporan simulasi: Tidak ada perubahan fisik yang dilakukan pada file Anda di disk.
            </div>
          </div>
        {/if}

        <div class="stats-grid">
          <div class="stat-card">
            <span class="stat-label">Item Berhasil</span>
            <span class="stat-val text-safe">{result.succeeded.length} item</span>
          </div>
          <div class="stat-card">
            <span class="stat-label">Item Gagal</span>
            <span class="stat-val" class:text-danger={result.failed.length > 0}>
              {result.failed.length} item
            </span>
          </div>
        </div>

        {#if result.failed.length > 0}
          <div class="failed-section">
            <button
              class="btn-failed-toggle"
              onclick={() => (showFailedDetails = !showFailedDetails)}
            >
              <div class="toggle-left">
                <AlertCircle size={16} class="text-danger" />
                <span>Lihat {result.failed.length} item yang gagal</span>
              </div>
              {#if showFailedDetails}
                <ChevronDown size={16} />
              {:else}
                <ChevronRight size={16} />
              {/if}
            </button>

            {#if showFailedDetails}
              <div class="failed-list">
                {#each result.failed as item (item.id)}
                  <div class="failed-row">
                    <span class="failed-id">{item.id}</span>
                    <span class="failed-error">{item.error}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <div class="result-footer">
        <button class="btn btn-primary btn-done" onclick={onClose}>
          Selesai
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
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
    max-width: 480px;
    box-shadow: 0 14px 36px rgba(0, 0, 0, 0.24);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slideUp 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .result-header {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 28px 24px 16px 24px;
    gap: 8px;
  }

  .icon-avatar {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: rgba(16, 185, 129, 0.12);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 4px;
  }

  .icon-avatar.dry-run {
    background: rgba(37, 99, 235, 0.12);
  }

  .icon-avatar.has-failed {
    background: rgba(220, 38, 38, 0.12);
  }

  .text-safe {
    color: var(--color-safe);
  }

  .text-danger {
    color: var(--color-danger);
  }

  .result-title {
    font-size: 1.25rem;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
  }

  .freed-amount {
    display: flex;
    align-items: baseline;
    gap: 6px;
    margin-top: 4px;
  }

  .freed-prefix {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .freed-value {
    font-size: 1.35rem;
    font-weight: 700;
    color: var(--accent-primary);
  }

  .result-body {
    padding: 12px 24px 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .banner {
    display: flex;
    gap: 10px;
    padding: 10px 14px;
    border-radius: var(--radius-md);
    font-size: 0.8rem;
    line-height: 1.4;
  }

  .banner-info {
    background: rgba(37, 99, 235, 0.08);
    border: 1px solid rgba(37, 99, 235, 0.2);
    color: var(--text-primary);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .stat-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .stat-val {
    font-size: 1.05rem;
    font-weight: 700;
  }

  .failed-section {
    display: flex;
    flex-direction: column;
    border: 1px solid rgba(220, 38, 38, 0.2);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .btn-failed-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    background: rgba(220, 38, 38, 0.06);
    border: none;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary);
    width: 100%;
    text-align: left;
  }

  .toggle-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .failed-list {
    padding: 8px 12px;
    background: var(--bg-secondary);
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 160px;
    overflow-y: auto;
  }

  .failed-row {
    display: flex;
    flex-direction: column;
    font-size: 0.75rem;
    gap: 2px;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 6px;
  }

  .failed-row:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .failed-id {
    font-family: monospace;
    font-weight: 600;
    color: var(--text-primary);
  }

  .failed-error {
    color: var(--color-danger);
  }

  .result-footer {
    display: flex;
    justify-content: flex-end;
    padding: 16px 24px;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .btn-done {
    width: 100%;
    justify-content: center;
    padding: 10px 0;
    font-size: 0.95rem;
    font-weight: 600;
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
