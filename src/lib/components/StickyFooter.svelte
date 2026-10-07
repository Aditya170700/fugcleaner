<script lang="ts">
  import { formatBytes } from '$lib/utils/format';
  import { id } from '$lib/i18n/id';
  import { Trash2, ShieldCheck, AlertCircle } from 'lucide-svelte';

  interface Props {
    selectedBytes: number;
    selectedCount: number;
    dryRun?: boolean;
    onClean: () => void;
  }

  let {
    selectedBytes = 0,
    selectedCount = 0,
    dryRun = true,
    onClean,
  }: Props = $props();

  const isCleanDisabled = $derived(selectedCount === 0 || selectedBytes === 0);
</script>

<div class="sticky-footer" class:has-selection={!isCleanDisabled}>
  <div class="footer-inner">
    <div class="selection-info">
      <div class="selected-amount">
        <span class="bytes-highlight">{formatBytes(selectedBytes)}</span>
        <span class="items-highlight">({selectedCount} item)</span>
      </div>
      <span class="selection-label">{id.dashboard.selectedToClean}</span>

      {#if dryRun}
        <div class="badge badge-caution footer-dryrun-badge">
          <ShieldCheck size={12} />
          <span>DRY RUN</span>
        </div>
      {/if}
    </div>

    <div class="action-group">
      <button
        class="btn btn-primary btn-clean"
        disabled={isCleanDisabled}
        onclick={onClean}
        title={isCleanDisabled ? 'Pilih minimal satu kategori untuk dibersihkan' : 'Eksekusi pembersihan'}
      >
        <Trash2 size={16} />
        <span>{id.dashboard.cleanButton}</span>
      </button>
    </div>
  </div>
</div>

<style>
  .sticky-footer {
    position: fixed;
    bottom: 0;
    left: 240px;
    right: 0;
    padding: 14px 36px;
    background-color: var(--bg-card);
    border-top: 1px solid var(--border-color);
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.08);
    z-index: 40;
    transition: all 0.2s ease;
  }

  .sticky-footer.has-selection {
    border-top-color: var(--accent-primary);
  }

  .footer-inner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;
    gap: 16px;
    flex-wrap: wrap;
  }

  .selection-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .selected-amount {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }

  .bytes-highlight {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--accent-primary);
  }

  .items-highlight {
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .selection-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .footer-dryrun-badge {
    font-size: 0.7rem;
  }

  .btn-clean {
    padding: 10px 24px;
    font-size: 0.95rem;
    font-weight: 600;
    box-shadow: 0 2px 8px rgba(37, 99, 235, 0.25);
  }

  .btn-clean:disabled {
    box-shadow: none;
  }
</style>
