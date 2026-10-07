<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { HistoryEntry } from '$lib/types';
  import { id } from '$lib/i18n/id';
  import { formatBytes, formatDateTime, formatRelativeTime } from '$lib/utils/format';
  import {
    History,
    Sparkles,
    Trash2,
    Clock,
    Shield,
    ChevronDown,
    ChevronUp,
    RefreshCw,
    HardDrive,
    CheckCircle2,
    Filter,
    ArrowRight,
  } from 'lucide-svelte';

  let historyEntries = $state<HistoryEntry[]>([]);
  let loading = $state(true);
  let expandedEntryIds = $state<Set<string>>(new Set());
  let activeFilter = $state<'all' | 'live' | 'dry_run'>('all');
  let showClearConfirm = $state(false);
  let clearing = $state(false);

  // Filtered entries
  const filteredEntries = $derived.by(() => {
    if (activeFilter === 'live') {
      return historyEntries.filter((e) => !e.dryRun);
    }
    if (activeFilter === 'dry_run') {
      return historyEntries.filter((e) => e.dryRun);
    }
    return historyEntries;
  });

  // Cumulative metrics
  const cumulativeStats = $derived.by(() => {
    let totalFreedLive = 0;
    let totalFreedDryRun = 0;
    let totalItemsLive = 0;
    let liveSessionCount = 0;

    for (const entry of historyEntries) {
      if (!entry.dryRun) {
        totalFreedLive += entry.freedBytes;
        totalItemsLive += entry.itemCount;
        liveSessionCount += 1;
      } else {
        totalFreedDryRun += entry.freedBytes;
      }
    }

    return {
      totalFreedLive,
      totalFreedDryRun,
      totalItemsLive,
      liveSessionCount,
      totalSessions: historyEntries.length,
    };
  });

  async function loadHistory() {
    loading = true;
    try {
      historyEntries = await api.getHistory();
    } catch (e) {
      console.error('Failed to load history:', e);
    } finally {
      loading = false;
    }
  }

  function toggleExpand(id: string) {
    const next = new Set(expandedEntryIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expandedEntryIds = next;
  }

  function toggleExpandAll() {
    if (expandedEntryIds.size === filteredEntries.length) {
      expandedEntryIds = new Set();
    } else {
      expandedEntryIds = new Set(filteredEntries.map((e) => e.id));
    }
  }

  async function handleClearHistory() {
    clearing = true;
    try {
      await api.clearHistory();
      historyEntries = [];
      expandedEntryIds = new Set();
      showClearConfirm = false;
    } catch (e) {
      console.error('Failed to clear history:', e);
    } finally {
      clearing = false;
    }
  }

  onMount(() => {
    loadHistory();
  });
</script>

<div class="history-page">
  <header class="page-header">
    <div>
      <h1 class="page-title">{id.history.title}</h1>
      <p class="page-subtitle">{id.history.subtitle}</p>
    </div>

    <div class="header-actions">
      <button class="btn btn-secondary" onclick={loadHistory} disabled={loading} title="Perbarui">
        <RefreshCw size={14} class={loading ? 'spin' : ''} />
        <span>Segarkan</span>
      </button>

      {#if historyEntries.length > 0}
        <button
          class="btn btn-danger-outline"
          onclick={() => (showClearConfirm = true)}
          disabled={loading || clearing}
        >
          <Trash2 size={14} />
          <span>Hapus Riwayat</span>
        </button>
      {/if}
    </div>
  </header>

  <!-- Cumulative Stats Cards -->
  <div class="stats-grid">
    <div class="card stat-card primary-stat">
      <div class="stat-icon-wrapper primary-icon">
        <HardDrive size={22} />
      </div>
      <div class="stat-content">
        <span class="stat-label">{id.history.totalFreed}</span>
        <span class="stat-value">{formatBytes(cumulativeStats.totalFreedLive)}</span>
        <span class="stat-meta">dari {cumulativeStats.liveSessionCount} sesi pembersihan nyata</span>
      </div>
    </div>

    <div class="card stat-card">
      <div class="stat-icon-wrapper success-icon">
        <CheckCircle2 size={22} />
      </div>
      <div class="stat-content">
        <span class="stat-label">Total Item Dibersihkan</span>
        <span class="stat-value">{cumulativeStats.totalItemsLive}</span>
        <span class="stat-meta">file cache & artifact terhapus</span>
      </div>
    </div>

    <div class="card stat-card">
      <div class="stat-icon-wrapper neutral-icon">
        <Clock size={22} />
      </div>
      <div class="stat-content">
        <span class="stat-label">Total Sesi Tercatat</span>
        <span class="stat-value">{cumulativeStats.totalSessions}</span>
        <span class="stat-meta">
          {cumulativeStats.totalFreedDryRun > 0
            ? `${formatBytes(cumulativeStats.totalFreedDryRun)} simulasi dry-run`
            : 'semua mode'}
        </span>
      </div>
    </div>
  </div>

  {#if historyEntries.length === 0}
    <!-- Empty State -->
    <div class="card empty-card">
      <div class="empty-icon-wrapper">
        <History size={48} />
      </div>
      <h3>{id.history.noHistory}</h3>
      <p>
        Setelah Anda melakukan pembersihan file cache atau artifact, log riwayat dan total ruang yang
        dibebaskan akan dicatat di sini.
      </p>
      <a href="/" class="btn btn-primary start-scan-btn">
        <span>Mulai Pindai Sekarang</span>
        <ArrowRight size={14} />
      </a>
    </div>
  {:else}
    <!-- History Table Section -->
    <div class="history-list-section">
      <div class="list-controls">
        <!-- Filter Tabs -->
        <div class="filter-tabs">
          <button
            class="tab-btn"
            class:active={activeFilter === 'all'}
            onclick={() => (activeFilter = 'all')}
          >
            Semua ({historyEntries.length})
          </button>
          <button
            class="tab-btn"
            class:active={activeFilter === 'live'}
            onclick={() => (activeFilter = 'live')}
          >
            Live ({historyEntries.filter((e) => !e.dryRun).length})
          </button>
          <button
            class="tab-btn"
            class:active={activeFilter === 'dry_run'}
            onclick={() => (activeFilter = 'dry_run')}
          >
            Dry Run ({historyEntries.filter((e) => e.dryRun).length})
          </button>
        </div>

        {#if filteredEntries.length > 0}
          <button class="btn-text toggle-all-btn" onclick={toggleExpandAll}>
            <span>
              {expandedEntryIds.size === filteredEntries.length
                ? 'Tutup Semua Rincian'
                : 'Buka Semua Rincian'}
            </span>
          </button>
        {/if}
      </div>

      <!-- Entries List -->
      <div class="entries-container">
        {#each filteredEntries as entry (entry.id)}
          {@const isExpanded = expandedEntryIds.has(entry.id)}
          <div class="card entry-card" class:entry-dryrun={entry.dryRun}>
            <!-- Entry Header -->
            <button
              class="entry-header"
              onclick={() => toggleExpand(entry.id)}
              aria-expanded={isExpanded}
            >
              <div class="entry-header-left">
                <div class="mode-badge-wrapper">
                  {#if entry.dryRun}
                    <span class="badge badge-warning">DRY RUN</span>
                  {:else}
                    <span class="badge badge-success">LIVE</span>
                  {/if}
                </div>

                <div class="entry-time-group">
                  <span class="entry-date">{formatDateTime(entry.timestamp)}</span>
                  <span class="entry-relative">({formatRelativeTime(entry.timestamp)})</span>
                </div>
              </div>

              <div class="entry-header-right">
                <div class="entry-stats">
                  <span class="entry-freed">{formatBytes(entry.freedBytes)}</span>
                  <span class="entry-items-count">({entry.itemCount} item)</span>
                </div>

                <div class="expand-icon-wrapper">
                  {#if isExpanded}
                    <ChevronUp size={16} />
                  {:else}
                    <ChevronDown size={16} />
                  {/if}
                </div>
              </div>
            </button>

            <!-- Expanded Details (Item Breakdown) -->
            {#if isExpanded}
              <div class="entry-details">
                <div class="details-table-wrapper">
                  <table class="details-table">
                    <thead>
                      <tr>
                        <th>Kategori</th>
                        <th>Path Target</th>
                        <th class="text-right">Ukuran</th>
                      </tr>
                    </thead>
                    <tbody>
                      {#each entry.items as item, idx (`${entry.id}-${idx}`)}
                        <tr>
                          <td>
                            <span class="category-chip">{item.categoryId}</span>
                          </td>
                          <td>
                            <span class="path-cell" title={item.path}>{item.path}</span>
                          </td>
                          <td class="text-right font-mono">
                            {formatBytes(item.bytes)}
                          </td>
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Clear Confirmation Dialog -->
  {#if showClearConfirm}
    <div class="modal-backdrop" onclick={() => (showClearConfirm = false)} role="presentation">
      <div
        class="modal-content card"
        onclick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
      >
        <div class="modal-header">
          <div class="modal-icon-danger">
            <Trash2 size={24} />
          </div>
          <div>
            <h3>Hapus Seluruh Riwayat?</h3>
            <p>Tindakan ini akan menghapus semua log catatan pembersihan secara permanen.</p>
          </div>
        </div>

        <div class="modal-actions">
          <button
            class="btn btn-secondary"
            onclick={() => (showClearConfirm = false)}
            disabled={clearing}
          >
            Batal
          </button>
          <button class="btn btn-danger" onclick={handleClearHistory} disabled={clearing}>
            <span>{clearing ? 'Menghapus...' : 'Ya, Hapus Riwayat'}</span>
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .history-page {
    display: flex;
    flex-direction: column;
    gap: 24px;
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;
    padding-bottom: 32px;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .page-title {
    font-size: 1.5rem;
    font-weight: 700;
  }

  .page-subtitle {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-top: 2px;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  /* Stats Grid */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px;
  }

  .stat-icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    border-radius: var(--radius-md);
    flex-shrink: 0;
  }

  .primary-icon {
    background-color: var(--accent-primary-light);
    color: var(--accent-primary);
  }

  .success-icon {
    background-color: var(--accent-success-light);
    color: var(--accent-success);
  }

  .neutral-icon {
    background-color: var(--bg-hover);
    color: var(--text-secondary);
  }

  .stat-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-label {
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1.2;
  }

  .stat-meta {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  /* Empty Card */
  .empty-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 64px 24px;
    gap: 14px;
  }

  .empty-icon-wrapper {
    color: var(--text-muted);
    margin-bottom: 4px;
  }

  .empty-card h3 {
    font-size: 1.15rem;
    font-weight: 600;
  }

  .empty-card p {
    max-width: 480px;
    font-size: 0.875rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .start-scan-btn {
    margin-top: 8px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    text-decoration: none;
  }

  /* History List Section */
  .history-list-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .list-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .filter-tabs {
    display: flex;
    align-items: center;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 2px;
    gap: 2px;
  }

  .tab-btn {
    background: none;
    border: none;
    padding: 6px 14px;
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab-btn:hover {
    color: var(--text-primary);
  }

  .tab-btn.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    font-weight: 600;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .toggle-all-btn {
    font-size: 0.8rem;
    color: var(--accent-primary);
    cursor: pointer;
    background: none;
    border: none;
    padding: 4px 8px;
  }

  .toggle-all-btn:hover {
    text-decoration: underline;
  }

  /* Entries Container */
  .entries-container {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .entry-card {
    padding: 0;
    overflow: hidden;
    transition: border-color 0.15s ease;
  }

  .entry-card:hover {
    border-color: var(--border-hover);
  }

  .entry-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 16px 20px;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    gap: 16px;
    color: var(--text-primary);
  }

  .entry-header-left {
    display: flex;
    align-items: center;
    gap: 14px;
    flex-wrap: wrap;
  }

  .badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.05em;
  }

  .badge-success {
    background-color: var(--accent-success-light);
    color: var(--accent-success);
  }

  .badge-warning {
    background-color: var(--accent-warning-light);
    color: var(--accent-warning);
  }

  .entry-time-group {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }

  .entry-date {
    font-size: 0.9rem;
    font-weight: 600;
  }

  .entry-relative {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .entry-header-right {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .entry-stats {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }

  .entry-freed {
    font-size: 1rem;
    font-weight: 700;
    color: var(--accent-primary);
    font-family: monospace;
  }

  .entry-items-count {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .expand-icon-wrapper {
    color: var(--text-muted);
    display: flex;
    align-items: center;
  }

  /* Details Table */
  .entry-details {
    padding: 0 20px 16px;
    border-top: 1px solid var(--border-color);
    background-color: rgba(0, 0, 0, 0.02);
  }

  .details-table-wrapper {
    overflow-x: auto;
    margin-top: 12px;
  }

  .details-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.825rem;
  }

  .details-table th {
    text-align: left;
    padding: 8px 10px;
    color: var(--text-muted);
    font-weight: 600;
    font-size: 0.75rem;
    text-transform: uppercase;
    border-bottom: 1px solid var(--border-color);
  }

  .details-table td {
    padding: 8px 10px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  }

  .category-chip {
    display: inline-block;
    padding: 2px 6px;
    background: var(--bg-hover);
    border-radius: 4px;
    font-size: 0.75rem;
    font-family: monospace;
    color: var(--text-secondary);
  }

  .path-cell {
    font-family: monospace;
    font-size: 0.775rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
    max-width: 500px;
  }

  .text-right {
    text-align: right;
  }

  .font-mono {
    font-family: monospace;
  }

  .btn-danger-outline {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(220, 38, 38, 0.3);
    background: transparent;
    color: var(--color-danger);
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-danger-outline:hover {
    background-color: rgba(220, 38, 38, 0.08);
    border-color: var(--color-danger);
  }

  /* Modal Backdrop & Content */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 20px;
  }

  .modal-content {
    width: 100%;
    max-width: 440px;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .modal-header {
    display: flex;
    gap: 16px;
  }

  .modal-icon-danger {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border-radius: 10px;
    background: rgba(220, 38, 38, 0.1);
    color: var(--color-danger);
    flex-shrink: 0;
  }

  .modal-header h3 {
    font-size: 1.1rem;
    font-weight: 600;
  }

  .modal-header p {
    font-size: 0.85rem;
    color: var(--text-secondary);
    margin-top: 4px;
    line-height: 1.4;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  :global(.spin) {
    animation: rotate 1s linear infinite;
  }

  @keyframes rotate {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
