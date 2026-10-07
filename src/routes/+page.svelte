<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '$lib/api';
  import type { DiskInfo, Category, ScanItem, ScanProgress } from '$lib/types';
  import { id } from '$lib/i18n/id';
  import DiskBar from '$lib/components/DiskBar.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import CategoryCard from '$lib/components/CategoryCard.svelte';
  import ItemListModal from '$lib/components/ItemListModal.svelte';
  import StickyFooter from '$lib/components/StickyFooter.svelte';
  import {
    Sparkles,
    ShieldAlert,
    RefreshCw,
    CheckSquare,
    Square,
    Filter,
  } from 'lucide-svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let diskInfo = $state<DiskInfo | null>(null);
  let loadingDisk = $state(true);
  let hasFullDiskAccess = $state(true);

  let categories = $state<Category[]>([]);
  let scannedItems = $state<ScanItem[]>([]);
  let selectedCategoryIds = $state<Set<string>>(new Set());

  let isScanning = $state(false);
  let currentScanId = $state<string>('');
  let scanProgress = $state<ScanProgress | null>(null);

  let activeModalCategory = $state<Category | null>(null);
  let unlistenProgress: UnlistenFn | null = null;

  // Compute stats per category: size, count, list of items
  const categoryStats = $derived.by(() => {
    const map = new Map<string, { bytes: number; count: number; items: ScanItem[] }>();

    for (const cat of categories) {
      map.set(cat.id, { bytes: 0, count: 0, items: [] });
    }

    for (const item of scannedItems) {
      const stat = map.get(item.categoryId);
      if (stat) {
        stat.bytes += item.bytes;
        stat.count += 1;
        stat.items.push(item);
      }
    }

    return map;
  });

  // Calculate selected total bytes & item count
  const selectedBytes = $derived.by(() => {
    let total = 0;
    for (const catId of selectedCategoryIds) {
      const stat = categoryStats.get(catId);
      if (stat) {
        total += stat.bytes;
      }
    }
    return total;
  });

  const selectedItemCount = $derived.by(() => {
    let count = 0;
    for (const catId of selectedCategoryIds) {
      const stat = categoryStats.get(catId);
      if (stat) {
        count += stat.count;
      }
    }
    return count;
  });

  // Sort categories: available first, then non-zero size, then alphabetical
  const sortedCategories = $derived.by(() => {
    return [...categories].sort((a, b) => {
      if (a.available !== b.available) return a.available ? -1 : 1;
      const bytesA = categoryStats.get(a.id)?.bytes ?? 0;
      const bytesB = categoryStats.get(b.id)?.bytes ?? 0;
      if (bytesA !== bytesB) return bytesB - bytesA;
      return a.name.localeCompare(b.name);
    });
  });

  async function loadDiskInfo() {
    loadingDisk = true;
    try {
      diskInfo = await api.getDiskInfo();
    } catch (e) {
      console.error('Failed to get disk info:', e);
    } finally {
      loadingDisk = false;
    }
  }

  async function loadCategories() {
    try {
      categories = await api.listCategories();
    } catch (e) {
      console.error('Failed to load categories:', e);
    }
  }

  async function startScanGlobal() {
    if (isScanning) return;

    isScanning = true;
    const scanId = 'scan-' + Date.now();
    currentScanId = scanId;
    scanProgress = {
      scanId,
      phase: id.dashboard.scanning,
      currentPath: 'Memulai pemindaian cache...',
      itemsFound: 0,
      bytesFound: 0,
    };

    try {
      const items = await api.scanGlobal(scanId);
      scannedItems = items;

      // Default selection: select all 'safe' categories that have bytes > 0
      const newSelected = new Set<string>();
      for (const cat of categories) {
        const stat = categoryStats.get(cat.id);
        if (cat.risk === 'safe' && cat.available && stat && stat.bytes > 0) {
          newSelected.add(cat.id);
        }
      }
      selectedCategoryIds = newSelected;
    } catch (e) {
      console.error('Scan error:', e);
    } finally {
      isScanning = false;
      scanProgress = null;
      loadDiskInfo();
    }
  }

  function handleCancelScan() {
    if (currentScanId) {
      api.cancelScan(currentScanId);
    }
    isScanning = false;
  }

  function handleToggleCategory(catId: string, selected: boolean) {
    const next = new Set(selectedCategoryIds);
    if (selected) {
      next.add(catId);
    } else {
      next.delete(catId);
    }
    selectedCategoryIds = next;
  }

  function handleSelectAllSafe() {
    const next = new Set<string>();
    for (const cat of categories) {
      const stat = categoryStats.get(cat.id);
      if (cat.available && stat && stat.bytes > 0) {
        next.add(cat.id);
      }
    }
    selectedCategoryIds = next;
  }

  function handleDeselectAll() {
    selectedCategoryIds = new Set();
  }

  onMount(async () => {
    loadDiskInfo();
    await loadCategories();

    try {
      hasFullDiskAccess = await api.checkFullDiskAccess();
    } catch {
      hasFullDiskAccess = true;
    }

    // Subscribe to scan progress events
    try {
      unlistenProgress = await api.onScanProgress((payload) => {
        scanProgress = payload;
      });
    } catch (e) {
      console.error('Failed to listen to scan progress:', e);
    }

    // Auto-scan on launch
    startScanGlobal();
  });

  onDestroy(() => {
    if (unlistenProgress) {
      unlistenProgress();
    }
  });
</script>

<div class="dashboard-page">
  <!-- Page Header -->
  <header class="page-header">
    <div>
      <h1 class="page-title">{id.dashboard.title}</h1>
      <p class="page-subtitle">{id.dashboard.subtitle}</p>
    </div>

    <div class="header-actions">
      <button
        class="btn btn-primary"
        onclick={startScanGlobal}
        disabled={isScanning}
      >
        <RefreshCw size={15} class={isScanning ? 'spin' : ''} />
        <span>{isScanning ? id.dashboard.scanning : 'Pindai Ulang'}</span>
      </button>
    </div>
  </header>

  <!-- Full Disk Access Warning Banner -->
  {#if !hasFullDiskAccess}
    <div class="fda-banner">
      <div class="fda-banner-icon">
        <ShieldAlert size={22} />
      </div>
      <div class="fda-banner-content">
        <h4>{id.fullDiskAccess.bannerTitle}</h4>
        <p>{id.fullDiskAccess.bannerDesc}</p>
      </div>
    </div>
  {/if}

  <!-- Active Scan Progress -->
  {#if isScanning}
    <section class="section">
      <ProgressBar progress={scanProgress} onCancel={handleCancelScan} />
    </section>
  {/if}

  <!-- Disk Capacity Bar -->
  <section class="section">
    <DiskBar
      {diskInfo}
      potentialFreedBytes={selectedBytes}
      loading={loadingDisk}
      onRefresh={loadDiskInfo}
    />
  </section>

  <!-- Categories Section -->
  <section class="section categories-section">
    <div class="section-header">
      <div class="section-title-group">
        <h2>{id.dashboard.categoriesTitle}</h2>
        <span class="category-count">({categories.length} kategori)</span>
      </div>

      <div class="selection-shortcuts">
        <button class="btn-text" onclick={handleSelectAllSafe}>
          <CheckSquare size={14} />
          <span>Pilih Semua</span>
        </button>
        <span class="divider">•</span>
        <button class="btn-text" onclick={handleDeselectAll}>
          <Square size={14} />
          <span>Batal Pilih</span>
        </button>
      </div>
    </div>

    <!-- Category Cards Grid -->
    <div class="categories-grid">
      {#each sortedCategories as category (category.id)}
        {@const stats = categoryStats.get(category.id) ?? { bytes: 0, count: 0, items: [] }}
        <CategoryCard
          {category}
          bytes={stats.bytes}
          itemCount={stats.count}
          selected={selectedCategoryIds.has(category.id)}
          onToggleSelect={(sel) => handleToggleCategory(category.id, sel)}
          onOpenDetails={() => (activeModalCategory = category)}
        />
      {/each}
    </div>
  </section>

  <!-- Sticky Footer for Cleaning -->
  <StickyFooter
    {selectedBytes}
    selectedCount={selectedItemCount}
    dryRun={true}
    onClean={() => {}}
  />

  <!-- Category Item Details Modal -->
  {#if activeModalCategory}
    {@const items = categoryStats.get(activeModalCategory.id)?.items ?? []}
    <ItemListModal
      category={activeModalCategory}
      {items}
      onClose={() => (activeModalCategory = null)}
    />
  {/if}
</div>

<style>
  .dashboard-page {
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 1100px;
    padding-bottom: 24px;
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

  .fda-banner {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 14px 18px;
    border-radius: 10px;
    background-color: var(--accent-warning-light);
    border: 1px solid rgba(245, 158, 11, 0.3);
    color: var(--text-primary);
  }

  .fda-banner-icon {
    color: var(--accent-warning);
    flex-shrink: 0;
  }

  .fda-banner-content h4 {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--accent-warning);
  }

  .fda-banner-content p {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .section-title-group {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .section-title-group h2 {
    font-size: 1.15rem;
    font-weight: 600;
  }

  .category-count {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .selection-shortcuts {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-text {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: none;
    border: none;
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--accent-primary);
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
    transition: color 0.15s ease;
  }

  .btn-text:hover {
    text-decoration: underline;
  }

  .divider {
    color: var(--border-hover);
    font-size: 0.8rem;
  }

  .categories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 16px;
  }

  :global(.spin) {
    animation: rotate 1s linear infinite;
  }

  @keyframes rotate {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
