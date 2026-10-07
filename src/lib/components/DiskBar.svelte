<script lang="ts">
  import type { DiskInfo } from '$lib/types';
  import { formatBytes } from '$lib/utils/format';
  import { id } from '$lib/i18n/id';
  import { HardDrive, RefreshCw } from 'lucide-svelte';

  interface Props {
    diskInfo: DiskInfo | null;
    potentialFreedBytes?: number;
    loading?: boolean;
    onRefresh?: () => void;
  }

  let {
    diskInfo = null,
    potentialFreedBytes = 0,
    loading = false,
    onRefresh,
  }: Props = $props();

  const total = $derived(diskInfo?.total ?? 0);
  const available = $derived(diskInfo?.available ?? 0);
  const used = $derived(total > available ? total - available : 0);
  const cleanable = $derived(Math.min(potentialFreedBytes, used));

  // Calculations for bar percentages
  const usedPercent = $derived(total > 0 ? (used / total) * 100 : 0);
  const cleanablePercent = $derived(total > 0 ? (cleanable / total) * 100 : 0);
  const effectiveUsedPercent = $derived(Math.max(0, usedPercent - cleanablePercent));
  const freePercent = $derived(total > 0 ? (available / total) * 100 : 0);
</script>

<div class="card disk-card">
  <div class="disk-header">
    <div class="disk-title-group">
      <div class="icon-wrapper">
        <HardDrive size={20} class="icon-disk" />
      </div>
      <div>
        <h2 class="title">{id.disk.title}</h2>
        <span class="mount-point">{diskInfo ? diskInfo.mountPoint : '/'}</span>
      </div>
    </div>

    {#if onRefresh}
      <button
        class="btn btn-secondary btn-refresh"
        onclick={onRefresh}
        disabled={loading}
        title={id.disk.refresh}
      >
        <RefreshCw size={15} class={loading ? 'spin' : ''} />
        <span class="refresh-text">{loading ? id.common.loading : id.disk.refresh}</span>
      </button>
    {/if}
  </div>

  {#if loading && !diskInfo}
    <div class="loading-state">
      <div class="skeleton-bar"></div>
      <p class="hint">{id.disk.loading}</p>
    </div>
  {:else if diskInfo}
    <!-- Multi-segment visual progress bar -->
    <div class="progress-track" role="progressbar" aria-valuenow={usedPercent} aria-valuemin={0} aria-valuemax={100}>
      {#if effectiveUsedPercent > 0}
        <div
          class="progress-segment segment-used"
          style="width: {effectiveUsedPercent}%;"
          title="{id.disk.used}: {formatBytes(used - cleanable)}"
        ></div>
      {/if}

      {#if cleanablePercent > 0}
        <div
          class="progress-segment segment-cleanable"
          style="width: {cleanablePercent}%;"
          title="{id.disk.potentialClean}: {formatBytes(cleanable)}"
        ></div>
      {/if}

      {#if freePercent > 0}
        <div
          class="progress-segment segment-free"
          style="width: {freePercent}%;"
          title="{id.disk.available}: {formatBytes(available)}"
        ></div>
      {/if}
    </div>

    <!-- Disk Legend & Stats -->
    <div class="disk-legend">
      <div class="legend-item">
        <span class="dot dot-used"></span>
        <div class="legend-text">
          <span class="label">{id.disk.used}</span>
          <span class="value">{formatBytes(used)} ({usedPercent.toFixed(1)}%)</span>
        </div>
      </div>

      {#if cleanable > 0}
        <div class="legend-item">
          <span class="dot dot-cleanable"></span>
          <div class="legend-text">
            <span class="label">{id.disk.potentialClean}</span>
            <span class="value text-success">+{formatBytes(cleanable)}</span>
          </div>
        </div>
      {/if}

      <div class="legend-item">
        <span class="dot dot-free"></span>
        <div class="legend-text">
          <span class="label">{id.disk.available}</span>
          <span class="value">{formatBytes(available)} ({freePercent.toFixed(1)}%)</span>
        </div>
      </div>

      <div class="legend-item legend-total">
        <div class="legend-text">
          <span class="label">{id.disk.total}</span>
          <span class="value">{formatBytes(total)}</span>
        </div>
      </div>
    </div>
  {:else}
    <div class="error-state">
      <p>{id.disk.error}</p>
    </div>
  {/if}
</div>

<style>
  .disk-card {
    display: flex;
    flex-direction: column;
    gap: 16px;
    background-color: var(--bg-card);
  }

  .disk-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .disk-title-group {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 8px;
    background-color: var(--accent-primary-light);
    color: var(--accent-primary);
  }

  .title {
    font-size: 1.1rem;
    font-weight: 600;
  }

  .mount-point {
    font-size: 0.8rem;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .btn-refresh {
    padding: 6px 12px;
    font-size: 0.8rem;
  }

  .refresh-text {
    display: inline;
  }

  @media (max-width: 600px) {
    .refresh-text {
      display: none;
    }
  }

  .progress-track {
    display: flex;
    height: 14px;
    width: 100%;
    background-color: var(--disk-free);
    border-radius: 7px;
    overflow: hidden;
    gap: 2px;
    padding: 1px;
  }

  .progress-segment {
    height: 100%;
    border-radius: 5px;
    transition: width 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .segment-used {
    background-color: var(--disk-used);
  }

  .segment-cleanable {
    background-color: var(--disk-cleanable);
    animation: pulse-glow 2s infinite ease-in-out;
  }

  .segment-free {
    background-color: transparent;
  }

  @keyframes pulse-glow {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  .disk-legend {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding-top: 4px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .dot-used { background-color: var(--disk-used); }
  .dot-cleanable { background-color: var(--disk-cleanable); }
  .dot-free { background-color: var(--border-hover); }

  .legend-text {
    display: flex;
    flex-direction: column;
    font-size: 0.8rem;
  }

  .legend-text .label {
    color: var(--text-muted);
    font-size: 0.75rem;
  }

  .legend-text .value {
    color: var(--text-primary);
    font-weight: 600;
  }

  .text-success {
    color: var(--accent-success);
  }

  .legend-total {
    margin-left: auto;
    text-align: right;
  }

  .loading-state, .error-state {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 0;
  }

  .skeleton-bar {
    height: 14px;
    border-radius: 7px;
    background: linear-gradient(90deg, var(--bg-subtle) 25%, var(--border-color) 50%, var(--bg-subtle) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  .hint {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  :global(.spin) {
    animation: rotate 1s linear infinite;
  }

  @keyframes rotate {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
