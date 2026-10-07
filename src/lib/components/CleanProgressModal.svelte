<script lang="ts">
  import type { CleanProgress } from '$lib/types';
  import { Loader2, ShieldCheck, Sparkles } from 'lucide-svelte';

  interface Props {
    open: boolean;
    progress: CleanProgress | null;
    dryRun?: boolean;
  }

  let {
    open = false,
    progress = null,
    dryRun = false,
  }: Props = $props();

  const percentage = $derived.by(() => {
    if (!progress || progress.total === 0) return 0;
    return Math.min(100, Math.round((progress.done / progress.total) * 100));
  });
</script>

{#if open}
  <div class="modal-backdrop">
    <div class="modal-card" role="dialog" aria-modal="true">
      <div class="progress-body">
        <div class="spinner-container">
          {#if dryRun}
            <div class="shield-pulse">
              <ShieldCheck size={36} class="text-accent" />
            </div>
          {:else}
            <Loader2 size={36} class="spinner text-accent" />
          {/if}
        </div>

        <h3 class="progress-title">
          {#if dryRun}
            Menjalankan Simulasi Pembersihan...
          {:else}
            Membersihkan Sistem...
          {/if}
        </h3>

        <p class="progress-status">
          {#if progress}
            Memproses {progress.done} dari {progress.total} item ({percentage}%)
          {:else}
            Mempersiapkan item...
          {/if}
        </p>

        <div class="progress-track">
          <div
            class="progress-fill"
            style="width: {percentage}%"
          ></div>
        </div>

        {#if progress?.currentPath}
          <div class="current-path" title={progress.currentPath}>
            {progress.currentPath}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(6px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
    padding: 16px;
    animation: fadeIn 0.15s ease-out;
  }

  .modal-card {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    width: 100%;
    max-width: 440px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.28);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: scaleIn 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .progress-body {
    padding: 32px 28px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 12px;
  }

  .spinner-container {
    margin-bottom: 4px;
  }

  :global(.spinner) {
    animation: spin 1s linear infinite;
  }

  .shield-pulse {
    animation: pulse 1.5s ease-in-out infinite;
  }

  .progress-title {
    font-size: 1.15rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .progress-status {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin: 0;
  }

  .progress-track {
    width: 100%;
    height: 8px;
    background: var(--bg-hover);
    border-radius: 999px;
    overflow: hidden;
    margin-top: 8px;
    border: 1px solid var(--border-color);
  }

  .progress-fill {
    height: 100%;
    background: var(--accent-primary);
    border-radius: 999px;
    transition: width 0.15s ease-out;
  }

  .current-path {
    font-family: monospace;
    font-size: 0.75rem;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
    margin-top: 4px;
    direction: rtl;
    text-align: center;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 1; }
    50% { transform: scale(1.1); opacity: 0.8; }
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes scaleIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
</style>
