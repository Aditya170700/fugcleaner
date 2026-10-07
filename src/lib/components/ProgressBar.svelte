<script lang="ts">
  import type { ScanProgress } from '$lib/types';
  import { formatBytes } from '$lib/utils/format';
  import { id } from '$lib/i18n/id';
  import { Loader2, XCircle } from 'lucide-svelte';

  interface Props {
    progress: ScanProgress | null;
    onCancel: () => void;
  }

  let { progress, onCancel }: Props = $props();
</script>

<div class="card scan-progress-card">
  <div class="progress-info-row">
    <div class="phase-group">
      <Loader2 size={18} class="spin text-primary" />
      <div class="phase-texts">
        <span class="phase-title">{progress?.phase || id.dashboard.scanning}</span>
        <span class="current-path" title={progress?.currentPath || ''}>
          {progress?.currentPath || 'Mempersiapkan...'}
        </span>
      </div>
    </div>

    <div class="stats-and-actions">
      <div class="stat-badge">
        <span class="stat-count">{progress?.itemsFound ?? 0} item</span>
        <span class="stat-size">({formatBytes(progress?.bytesFound ?? 0)})</span>
      </div>

      <button class="btn btn-secondary btn-cancel" onclick={onCancel}>
        <XCircle size={15} />
        <span>{id.dashboard.cancelScan}</span>
      </button>
    </div>
  </div>

  <div class="shimmer-bar">
    <div class="shimmer-track"></div>
  </div>
</div>

<style>
  .scan-progress-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    background-color: var(--bg-card);
    border: 1px solid var(--accent-primary);
    box-shadow: var(--shadow-md);
  }

  .progress-info-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .phase-group {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    flex: 1;
  }

  .phase-texts {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .phase-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .current-path {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: var(--font-mono);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 400px;
  }

  .stats-and-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .stat-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.825rem;
    font-weight: 600;
    background-color: var(--bg-subtle);
    padding: 4px 10px;
    border-radius: 6px;
  }

  .stat-size {
    color: var(--accent-primary);
  }

  .btn-cancel {
    padding: 6px 12px;
    font-size: 0.8rem;
    color: var(--accent-danger);
  }

  .btn-cancel:hover {
    background-color: var(--accent-danger-light);
    border-color: var(--accent-danger);
  }

  .shimmer-bar {
    height: 4px;
    border-radius: 2px;
    background-color: var(--bg-subtle);
    overflow: hidden;
    position: relative;
  }

  .shimmer-track {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    width: 40%;
    background: linear-gradient(90deg, transparent, var(--accent-primary), transparent);
    animation: indeterminate 1.5s infinite linear;
  }

  @keyframes indeterminate {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(350%); }
  }

  :global(.spin) {
    animation: rotate 1s linear infinite;
  }

  @keyframes rotate {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  :global(.text-primary) {
    color: var(--accent-primary);
  }
</style>
