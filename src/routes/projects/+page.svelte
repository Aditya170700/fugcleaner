<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/api';
  import type { ScannedProject, ProjectArtifact, Settings, ScanProgress } from '$lib/types';
  import { formatBytes, formatRelativeTime } from '$lib/utils/format';
  import { id } from '$lib/i18n/id';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import StickyFooter from '$lib/components/StickyFooter.svelte';
  import {
    FolderGit2,
    Plus,
    FolderSearch,
    Trash2,
    RefreshCw,
    X,
    Folder,
    ExternalLink,
    Clock,
    ChevronDown,
    ChevronRight,
    ArrowUpDown,
    CheckSquare,
    Square,
    AlertCircle,
  } from 'lucide-svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let settings = $state<Settings | null>(null);
  let projects = $state<ScannedProject[]>([]);
  let isScanning = $state(false);
  let currentScanId = $state<string>('');
  let scanProgress = $state<ScanProgress | null>(null);

  let staleOnly = $state(false);
  let selectedArtifactIds = $state<Set<string>>(new Set());
  let expandedProjectIds = $state<Set<string>>(new Set());

  type SortColumn = 'name' | 'activity' | 'size';
  let sortCol = $state<SortColumn>('size');
  let sortAsc = $state(false);

  let unlistenProgress: UnlistenFn | null = null;

  // Filter and sort projects
  const filteredProjects = $derived.by(() => {
    let list = projects;
    if (staleOnly) {
      list = list.filter((p) => p.stale);
    }

    return [...list].sort((a, b) => {
      let cmp = 0;
      if (sortCol === 'name') {
        cmp = a.name.localeCompare(b.name);
      } else if (sortCol === 'activity') {
        cmp = (a.lastActivity || 0) - (b.lastActivity || 0);
      } else if (sortCol === 'size') {
        cmp = a.totalBytes - b.totalBytes;
      }
      return sortAsc ? cmp : -cmp;
    });
  });

  // Calculate selected total bytes and count
  const selectedStats = $derived.by(() => {
    let bytes = 0;
    let count = 0;

    for (const proj of projects) {
      for (const art of proj.artifacts) {
        if (selectedArtifactIds.has(art.id)) {
          bytes += art.bytes;
          count += 1;
        }
      }
    }

    return { bytes, count };
  });

  async function loadSettingsAndScan() {
    try {
      const s = await api.getSettings();
      settings = s;
      if (s.projectRoots.length > 0) {
        await startScanProjects(s.projectRoots);
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async function startScanProjects(roots?: string[]) {
    if (isScanning) return;

    const scanRoots = roots || settings?.projectRoots || [];
    if (scanRoots.length === 0) return;

    isScanning = true;
    const scanId = 'proj-scan-' + Date.now();
    currentScanId = scanId;
    scanProgress = {
      scanId,
      phase: 'Memindai artifact project...',
      currentPath: 'Memulai penelusuran folder...',
      itemsFound: 0,
      bytesFound: 0,
    };

    try {
      const results = await api.scanProjects(scanId, scanRoots);
      projects = results;

      // Select all artifacts of stale projects by default
      const initialSelected = new Set<string>();
      for (const p of results) {
        if (p.stale) {
          for (const a of p.artifacts) {
            if (a.risk === 'safe') {
              initialSelected.add(a.id);
            }
          }
        }
      }
      selectedArtifactIds = initialSelected;
    } catch (e) {
      console.error('Project scan failed:', e);
    } finally {
      isScanning = false;
      scanProgress = null;
    }
  }

  async function handleAddRootFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Pilih Folder Project',
      });

      if (selected && typeof selected === 'string' && settings) {
        if (!settings.projectRoots.includes(selected)) {
          const updatedRoots = [...settings.projectRoots, selected];
          settings.projectRoots = updatedRoots;
          await api.saveSettings(settings);
          await startScanProjects(updatedRoots);
        }
      }
    } catch (e) {
      console.error('Failed to open directory picker:', e);
    }
  }

  async function handleRemoveRootFolder(folderToRemove: string) {
    if (!settings) return;
    const updatedRoots = settings.projectRoots.filter((r) => r !== folderToRemove);
    settings.projectRoots = updatedRoots;
    await api.saveSettings(settings);
    if (updatedRoots.length > 0) {
      await startScanProjects(updatedRoots);
    } else {
      projects = [];
      selectedArtifactIds = new Set();
    }
  }

  function handleCancelScan() {
    if (currentScanId) {
      api.cancelScan(currentScanId);
    }
    isScanning = false;
  }

  function toggleProjectSelection(proj: ScannedProject, select: boolean) {
    const next = new Set(selectedArtifactIds);
    for (const art of proj.artifacts) {
      if (select) {
        next.add(art.id);
      } else {
        next.delete(art.id);
      }
    }
    selectedArtifactIds = next;
  }

  function isProjectFullySelected(proj: ScannedProject): boolean {
    return proj.artifacts.length > 0 && proj.artifacts.every((a) => selectedArtifactIds.has(a.id));
  }

  function isProjectPartiallySelected(proj: ScannedProject): boolean {
    const some = proj.artifacts.some((a) => selectedArtifactIds.has(a.id));
    return some && !isProjectFullySelected(proj);
  }

  function toggleArtifactSelection(artId: string, select: boolean) {
    const next = new Set(selectedArtifactIds);
    if (select) {
      next.add(artId);
    } else {
      next.delete(artId);
    }
    selectedArtifactIds = next;
  }

  function toggleExpandProject(projId: string) {
    const next = new Set(expandedProjectIds);
    if (next.has(projId)) {
      next.delete(projId);
    } else {
      next.add(projId);
    }
    expandedProjectIds = next;
  }

  function handleSelectAllStale() {
    const next = new Set<string>();
    for (const p of projects) {
      if (p.stale) {
        for (const a of p.artifacts) {
          next.add(a.id);
        }
      }
    }
    selectedArtifactIds = next;
  }

  function handleDeselectAll() {
    selectedArtifactIds = new Set();
  }

  function toggleSort(col: SortColumn) {
    if (sortCol === col) {
      sortAsc = !sortAsc;
    } else {
      sortCol = col;
      sortAsc = false;
    }
  }

  async function openInFinder(itemId: string) {
    try {
      await api.openInFileManager(itemId);
    } catch (e) {
      console.error('Failed to open in file manager:', e);
    }
  }

  onMount(async () => {
    try {
      unlistenProgress = await api.onScanProgress((payload) => {
        scanProgress = payload;
      });
    } catch (e) {
      console.error('Failed to subscribe scan progress:', e);
    }

    loadSettingsAndScan();
  });

  onDestroy(() => {
    if (unlistenProgress) {
      unlistenProgress();
    }
  });
</script>

<div class="projects-page">
  <!-- Header -->
  <header class="page-header">
    <div>
      <h1 class="page-title">{id.projects.title}</h1>
      <p class="page-subtitle">{id.projects.subtitle}</p>
    </div>

    <div class="header-actions">
      <button class="btn btn-primary" onclick={handleAddRootFolder} disabled={isScanning}>
        <Plus size={16} />
        <span>{id.projects.addFolder}</span>
      </button>

      {#if settings && settings.projectRoots.length > 0}
        <button
          class="btn btn-secondary"
          onclick={() => startScanProjects()}
          disabled={isScanning}
          title="Pindai Ulang Folder Project"
        >
          <RefreshCw size={15} class={isScanning ? 'spin' : ''} />
          <span>{isScanning ? id.dashboard.scanning : 'Pindai Ulang'}</span>
        </button>
      {/if}
    </div>
  </header>

  <!-- Root Folders Bar -->
  {#if settings && settings.projectRoots.length > 0}
    <div class="card roots-card">
      <div class="roots-header">
        <span class="roots-title">Folder Terpantau:</span>
      </div>
      <div class="roots-list">
        {#each settings.projectRoots as rootPath}
          <div class="root-chip">
            <Folder size={14} class="chip-icon" />
            <span class="chip-path" title={rootPath}>{rootPath}</span>
            <button
              class="chip-remove"
              onclick={() => handleRemoveRootFolder(rootPath)}
              title="Hapus folder dari pantauan"
            >
              <X size={12} />
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Active Scan Progress -->
  {#if isScanning}
    <ProgressBar progress={scanProgress} onCancel={handleCancelScan} />
  {/if}

  <!-- Empty State (No roots added yet) -->
  {#if !isScanning && (!settings || settings.projectRoots.length === 0)}
    <div class="card empty-card">
      <div class="empty-icon-wrapper">
        <FolderSearch size={44} />
      </div>
      <h3>{id.projects.noProjectsFound}</h3>
      <p>
        Tambahkan satu atau lebih folder tempat Anda menyimpan project (misalnya
        <code>~/Projects</code> atau <code>~/Code</code>) untuk memindai artifact
        <code>node_modules</code>, build cache, dan target output.
      </p>
      <button class="btn btn-primary mt-4" onclick={handleAddRootFolder}>
        <Plus size={16} />
        <span>{id.projects.addFolder}</span>
      </button>
    </div>
  {:else if !isScanning && projects.length === 0}
    <!-- Scanned but 0 projects found -->
    <div class="card empty-card">
      <div class="empty-icon-wrapper">
        <FolderGit2 size={44} />
      </div>
      <h3>Tidak ada artifact project yang ditemukan</h3>
      <p>
        Semua project di folder terpantau sudah bersih dari artifact build dan dependency lama.
      </p>
    </div>
  {:else if projects.length > 0}
    <!-- Controls bar: Filter & Shortcuts -->
    <div class="table-controls">
      <div class="filters-group">
        <label class="filter-toggle">
          <input type="checkbox" bind:checked={staleOnly} />
          <span class="toggle-label">
            Hanya project stale (&gt; {settings?.staleThresholdDays ?? 30} hari tidak aktif)
          </span>
        </label>
      </div>

      <div class="actions-group">
        <button class="btn-text" onclick={handleSelectAllStale}>
          <CheckSquare size={14} />
          <span>Pilih Semua yang Stale</span>
        </button>
        <span class="divider">•</span>
        <button class="btn-text" onclick={handleDeselectAll}>
          <Square size={14} />
          <span>Batal Pilih</span>
        </button>
      </div>
    </div>

    <!-- Projects Table -->
    <div class="card table-card">
      <div class="table-wrapper">
        <table class="projects-table">
          <thead>
            <tr>
              <th class="col-check"></th>
              <th class="col-expand"></th>
              <th class="col-name" onclick={() => toggleSort('name')}>
                <div class="th-content">
                  <span>{id.projects.colName}</span>
                  <ArrowUpDown size={12} class="sort-icon" />
                </div>
              </th>
              <th class="col-activity" onclick={() => toggleSort('activity')}>
                <div class="th-content">
                  <span>{id.projects.colLastActive}</span>
                  <ArrowUpDown size={12} class="sort-icon" />
                </div>
              </th>
              <th class="col-artifacts">{id.projects.colArtifacts}</th>
              <th class="col-size text-right" onclick={() => toggleSort('size')}>
                <div class="th-content justify-end">
                  <span>{id.projects.colSize}</span>
                  <ArrowUpDown size={12} class="sort-icon" />
                </div>
              </th>
            </tr>
          </thead>
          <tbody>
            {#each filteredProjects as proj (proj.id)}
              {@const isExpanded = expandedProjectIds.has(proj.id)}
              {@const fullySelected = isProjectFullySelected(proj)}
              {@const partial = isProjectPartiallySelected(proj)}

              <!-- Main Project Row -->
              <tr class="project-row" class:row-selected={fullySelected || partial}>
                <!-- Checkbox -->
                <td class="col-check">
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                  <label class="checkbox-container" onclick={(e) => e.stopPropagation()}>
                    <input
                      type="checkbox"
                      checked={fullySelected}
                      indeterminate={partial}
                      onchange={(e) => toggleProjectSelection(proj, (e.currentTarget as HTMLInputElement).checked)}
                    />
                    <span class="checkmark"></span>
                  </label>
                </td>

                <!-- Expand Toggle Button -->
                <td class="col-expand">
                  <button
                    class="btn-expand"
                    onclick={() => toggleExpandProject(proj.id)}
                    title={isExpanded ? 'Tutup detail' : 'Buka detail artifact'}
                  >
                    {#if isExpanded}
                      <ChevronDown size={15} />
                    {:else}
                      <ChevronRight size={15} />
                    {/if}
                  </button>
                </td>

                <!-- Name & Path -->
                <td class="col-name">
                  <div class="name-container">
                    <div class="name-title-row">
                      <span class="proj-name">{proj.name}</span>
                      <span class="badge badge-muted type-badge">{proj.projectType}</span>
                    </div>
                    <span class="proj-path" title={proj.path}>{proj.path}</span>
                  </div>
                </td>

                <!-- Last Activity -->
                <td class="col-activity">
                  <div class="activity-container">
                    <span class="activity-text">{formatRelativeTime(proj.lastActivity)}</span>
                    {#if proj.stale}
                      <span class="badge badge-caution stale-badge">Stale</span>
                    {/if}
                  </div>
                </td>

                <!-- Artifacts Badges -->
                <td class="col-artifacts">
                  <div class="artifacts-chips">
                    {#each proj.artifacts as art}
                      <span
                        class="art-chip"
                        class:chip-selected={selectedArtifactIds.has(art.id)}
                        title="{art.path} ({formatBytes(art.bytes)})"
                      >
                        {art.name} ({formatBytes(art.bytes)})
                      </span>
                    {/each}
                  </div>
                </td>

                <!-- Size -->
                <td class="col-size text-right">
                  <span class="size-val">{formatBytes(proj.totalBytes)}</span>
                </td>
              </tr>

              <!-- Expanded Artifacts Rows -->
              {#if isExpanded}
                {#each proj.artifacts as art}
                  <tr class="artifact-subrow">
                    <td></td>
                    <td class="col-check">
                      <!-- svelte-ignore a11y_click_events_have_key_events -->
                      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                      <label class="checkbox-container" onclick={(e) => e.stopPropagation()}>
                        <input
                          type="checkbox"
                          checked={selectedArtifactIds.has(art.id)}
                          onchange={(e) => toggleArtifactSelection(art.id, (e.currentTarget as HTMLInputElement).checked)}
                        />
                        <span class="checkmark"></span>
                      </label>
                    </td>
                    <td colspan="3" class="subrow-content">
                      <div class="subrow-info">
                        <span class="subrow-name">{art.name}</span>
                        <span class="subrow-path" title={art.path}>{art.path}</span>
                        {#if art.note}
                          <span class="badge badge-muted subrow-note">{art.note}</span>
                        {/if}
                        {#if art.risk === 'safe'}
                          <span class="badge badge-safe">Aman</span>
                        {:else}
                          <span class="badge badge-caution">Hati-hati</span>
                        {/if}
                      </div>
                    </td>
                    <td class="col-size text-right">
                      <div class="subrow-actions">
                        <span class="subrow-size">{formatBytes(art.bytes)}</span>
                        <button
                          class="btn btn-outline btn-finder-mini"
                          onclick={() => openInFinder(art.id)}
                          title="Buka di Finder"
                        >
                          <ExternalLink size={12} />
                        </button>
                      </div>
                    </td>
                  </tr>
                {/each}
              {/if}
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}

  <!-- Sticky Footer for Cleaning -->
  {#if projects.length > 0}
    <StickyFooter
      selectedBytes={selectedStats.bytes}
      selectedCount={selectedStats.count}
      dryRun={true}
      onClean={() => {}}
    />
  {/if}
</div>

<style>
  .projects-page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-width: 1100px;
    padding-bottom: 24px;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
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

  .roots-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 18px;
    background-color: var(--bg-card);
  }

  .roots-title {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .roots-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .root-chip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background-color: var(--bg-subtle);
    border: 1px solid var(--border-color);
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 0.8rem;
  }

  .chip-path {
    font-family: var(--font-mono);
    color: var(--text-primary);
  }

  .chip-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    transition: color 0.15s ease;
  }

  .chip-remove:hover {
    color: var(--accent-danger);
  }

  .table-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    padding: 4px 0;
  }

  .filter-toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 0.85rem;
    color: var(--text-primary);
  }

  .filter-toggle input {
    accent-color: var(--accent-primary);
    width: 16px;
    height: 16px;
  }

  .actions-group {
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
  }

  .btn-text:hover {
    text-decoration: underline;
  }

  .divider {
    color: var(--border-hover);
    font-size: 0.8rem;
  }

  .table-card {
    padding: 0;
    overflow: hidden;
  }

  .table-wrapper {
    overflow-x: auto;
    max-height: 60vh;
  }

  .projects-table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
    font-size: 0.85rem;
  }

  .projects-table th {
    background-color: var(--bg-subtle);
    color: var(--text-secondary);
    font-weight: 600;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border-color);
    position: sticky;
    top: 0;
    z-index: 10;
    user-select: none;
  }

  .th-content {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }

  .projects-table td {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-color);
  }

  .project-row {
    transition: background-color 0.15s ease;
  }

  .project-row:hover {
    background-color: var(--bg-card-hover);
  }

  .project-row.row-selected {
    background-color: var(--accent-primary-light);
  }

  .col-check {
    width: 36px;
    padding-right: 0 !important;
  }

  .col-expand {
    width: 32px;
    padding-left: 4px !important;
    padding-right: 0 !important;
  }

  .btn-expand {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-expand:hover {
    color: var(--text-primary);
  }

  .name-container {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 180px;
  }

  .name-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .proj-name {
    font-weight: 600;
    color: var(--text-primary);
  }

  .type-badge {
    font-size: 0.68rem;
    padding: 1px 6px;
  }

  .proj-path {
    font-size: 0.75rem;
    font-family: var(--font-mono);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 300px;
  }

  .activity-container {
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
  }

  .stale-badge {
    font-size: 0.68rem;
    padding: 1px 6px;
  }

  .artifacts-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .art-chip {
    font-size: 0.75rem;
    padding: 2px 8px;
    border-radius: 5px;
    background-color: var(--bg-subtle);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-family: var(--font-mono);
  }

  .art-chip.chip-selected {
    border-color: var(--accent-primary);
    background-color: var(--accent-primary-light);
    color: var(--accent-primary);
    font-weight: 600;
  }

  .size-val {
    font-weight: 700;
    color: var(--text-primary);
  }

  .text-right {
    text-align: right;
  }

  .justify-end {
    justify-content: flex-end;
  }

  /* Subrow styles for expanded view */
  .artifact-subrow {
    background-color: var(--bg-subtle);
  }

  .subrow-content {
    padding-left: 28px !important;
  }

  .subrow-info {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .subrow-name {
    font-weight: 600;
    font-size: 0.8rem;
    color: var(--text-primary);
  }

  .subrow-path {
    font-size: 0.72rem;
    font-family: var(--font-mono);
    color: var(--text-muted);
    max-width: 300px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .subrow-note {
    font-size: 0.68rem;
  }

  .subrow-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .subrow-size {
    font-size: 0.8rem;
    font-weight: 600;
  }

  .btn-finder-mini {
    padding: 2px 6px;
    font-size: 0.7rem;
  }

  /* Checkbox styling */
  .checkbox-container {
    position: relative;
    cursor: pointer;
    display: inline-block;
  }

  .checkbox-container input {
    position: absolute;
    opacity: 0;
    cursor: pointer;
    height: 0;
    width: 0;
  }

  .checkmark {
    display: block;
    height: 16px;
    width: 16px;
    background-color: var(--bg-card);
    border: 1.5px solid var(--border-hover);
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .checkbox-container:hover input ~ .checkmark {
    border-color: var(--accent-primary);
  }

  .checkbox-container input:checked ~ .checkmark {
    background-color: var(--accent-primary);
    border-color: var(--accent-primary);
  }

  .checkmark:after {
    content: "";
    position: absolute;
    display: none;
    left: 5px;
    top: 2px;
    width: 4px;
    height: 8px;
    border: solid white;
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }

  .checkbox-container input:checked ~ .checkmark:after {
    display: block;
  }

  .empty-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 60px 24px;
    gap: 12px;
  }

  .empty-icon-wrapper {
    color: var(--text-muted);
    margin-bottom: 8px;
  }

  .empty-card h3 {
    font-size: 1.1rem;
  }

  .empty-card p {
    max-width: 480px;
    font-size: 0.875rem;
  }

  .empty-card code {
    font-family: var(--font-mono);
    background-color: var(--bg-subtle);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.8rem;
  }

  .mt-4 {
    margin-top: 8px;
  }
</style>
