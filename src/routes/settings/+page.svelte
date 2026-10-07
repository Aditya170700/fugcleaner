<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/api';
  import type { Settings } from '$lib/types';
  import { id } from '$lib/i18n/id';
  import {
    Shield,
    Sliders,
    Save,
    Check,
    Folder,
    FolderPlus,
    X,
    FolderLock,
    FolderGit2,
    AlertCircle,
  } from 'lucide-svelte';

  let currentSettings = $state<Settings>({
    projectRoots: [],
    staleThresholdDays: 30,
    enabledCategories: [],
    excludePaths: [],
    maxScanDepth: 6,
    dryRun: true,
  });

  let newExcludeInput = $state('');
  let saved = $state(false);
  let saving = $state(false);
  let validationError = $state<string | null>(null);

  async function loadSettings() {
    try {
      currentSettings = await api.getSettings();
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async function handleAddRootFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Pilih Folder Project',
      });

      if (selected && typeof selected === 'string') {
        if (!currentSettings.projectRoots.includes(selected)) {
          currentSettings.projectRoots = [...currentSettings.projectRoots, selected];
        }
      }
    } catch (e) {
      console.error('Failed to open directory picker:', e);
    }
  }

  function handleRemoveRootFolder(folderToRemove: string) {
    currentSettings.projectRoots = currentSettings.projectRoots.filter(
      (r) => r !== folderToRemove
    );
  }

  async function handleAddExcludeFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Pilih Folder yang Dikecualikan',
      });

      if (selected && typeof selected === 'string') {
        if (!currentSettings.excludePaths.includes(selected)) {
          currentSettings.excludePaths = [...currentSettings.excludePaths, selected];
        }
      }
    } catch (e) {
      console.error('Failed to open directory picker:', e);
    }
  }

  function handleAddManualExclude() {
    const trimmed = newExcludeInput.trim();
    if (trimmed && !currentSettings.excludePaths.includes(trimmed)) {
      currentSettings.excludePaths = [...currentSettings.excludePaths, trimmed];
      newExcludeInput = '';
    }
  }

  function handleRemoveExcludePath(pathToRemove: string) {
    currentSettings.excludePaths = currentSettings.excludePaths.filter(
      (p) => p !== pathToRemove
    );
  }

  async function handleSave() {
    validationError = null;

    if (currentSettings.staleThresholdDays < 1 || currentSettings.staleThresholdDays > 365) {
      validationError = 'Batas waktu stale harus antara 1 dan 365 hari.';
      return;
    }

    if (currentSettings.maxScanDepth < 1 || currentSettings.maxScanDepth > 20) {
      validationError = 'Kedalaman scan harus antara 1 dan 20 level.';
      return;
    }

    saving = true;
    try {
      await api.saveSettings(currentSettings);
      saved = true;
      setTimeout(() => {
        saved = false;
      }, 2500);
    } catch (e) {
      console.error('Failed to save settings:', e);
      validationError = 'Gagal menyimpan pengaturan ke sistem.';
    } finally {
      saving = false;
    }
  }

  onMount(() => {
    loadSettings();
  });
</script>

<div class="settings-page">
  <header class="page-header">
    <div>
      <h1 class="page-title">{id.settings.title}</h1>
      <p class="page-subtitle">{id.settings.subtitle}</p>
    </div>

    <button class="btn btn-primary" onclick={handleSave} disabled={saving}>
      {#if saved}
        <Check size={16} />
        <span>{id.settings.savedToast}</span>
      {:else}
        <Save size={16} />
        <span>{saving ? 'Menyimpan...' : id.settings.saveButton}</span>
      {/if}
    </button>
  </header>

  {#if validationError}
    <div class="banner banner-error">
      <AlertCircle size={18} />
      <span>{validationError}</span>
    </div>
  {/if}

  <div class="settings-sections">
    <!-- Safety & Dry-run Section -->
    <section class="card settings-card">
      <div class="card-header">
        <div class="section-icon-wrapper">
          <Shield size={20} />
        </div>
        <div>
          <h3>Keamanan & Mode Simulasi</h3>
          <p>Kontrol perilaku eksekusi pembersihan</p>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label for="dry-run-toggle" class="setting-label">{id.settings.dryRunTitle}</label>
          <span class="setting-desc">{id.settings.dryRunDesc}</span>
        </div>
        <input
          id="dry-run-toggle"
          type="checkbox"
          class="toggle-input"
          bind:checked={currentSettings.dryRun}
        />
      </div>
    </section>

    <!-- Scan Configuration -->
    <section class="card settings-card">
      <div class="card-header">
        <div class="section-icon-wrapper">
          <Sliders size={20} />
        </div>
        <div>
          <h3>Konfigurasi Pemindaian</h3>
          <p>Parameter threshold dan batasan scanner</p>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label for="stale-days" class="setting-label">{id.settings.staleDaysTitle}</label>
          <span class="setting-desc">{id.settings.staleDaysDesc}</span>
        </div>
        <div class="input-with-unit">
          <input
            id="stale-days"
            type="number"
            min="1"
            max="365"
            class="number-input"
            bind:value={currentSettings.staleThresholdDays}
          />
          <span class="unit-label">hari</span>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label for="scan-depth" class="setting-label">{id.settings.scanDepthTitle}</label>
          <span class="setting-desc">{id.settings.scanDepthDesc}</span>
        </div>
        <div class="input-with-unit">
          <input
            id="scan-depth"
            type="number"
            min="1"
            max="20"
            class="number-input"
            bind:value={currentSettings.maxScanDepth}
          />
          <span class="unit-label">level</span>
        </div>
      </div>
    </section>

    <!-- Exclude Paths Section -->
    <section class="card settings-card">
      <div class="card-header">
        <div class="section-icon-wrapper">
          <FolderLock size={20} />
        </div>
        <div>
          <h3>{id.settings.excludePathsTitle}</h3>
          <p>Folder yang tidak akan pernah disentuh atau dipindai oleh Fug Cleaner</p>
        </div>
      </div>

      <div class="path-management">
        <div class="add-path-bar">
          <input
            type="text"
            placeholder="Ketik path atau gunakan tombol pilih..."
            class="text-input"
            bind:value={newExcludeInput}
            onkeydown={(e) => e.key === 'Enter' && handleAddManualExclude()}
          />
          <button class="btn btn-secondary" onclick={handleAddManualExclude} disabled={!newExcludeInput.trim()}>
            Tambah
          </button>
          <button class="btn btn-secondary" onclick={handleAddExcludeFolder}>
            <FolderPlus size={16} />
            <span>Pilih Folder</span>
          </button>
        </div>

        {#if currentSettings.excludePaths.length === 0}
          <div class="empty-paths-note">
            Belum ada path yang dikecualikan.
          </div>
        {:else}
          <div class="chips-container">
            {#each currentSettings.excludePaths as excPath (excPath)}
              <div class="path-chip">
                <FolderLock size={14} class="chip-icon text-muted" />
                <span class="chip-text" title={excPath}>{excPath}</span>
                <button
                  class="btn-remove-chip"
                  onclick={() => handleRemoveExcludePath(excPath)}
                  title="Hapus dari pengecualian"
                >
                  <X size={12} />
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </section>

    <!-- Project Roots Section -->
    <section class="card settings-card">
      <div class="card-header">
        <div class="section-icon-wrapper">
          <FolderGit2 size={20} />
        </div>
        <div>
          <h3>{id.settings.projectRootsTitle}</h3>
          <p>Direktori induk tempat scanner mencari project dan artifact build</p>
        </div>
      </div>

      <div class="path-management">
        <div class="add-path-bar">
          <button class="btn btn-secondary" onclick={handleAddRootFolder}>
            <FolderPlus size={16} />
            <span>Tambah Folder Project</span>
          </button>
        </div>

        {#if currentSettings.projectRoots.length === 0}
          <div class="empty-paths-note">
            Belum ada folder project yang ditambahkan.
          </div>
        {:else}
          <div class="chips-container">
            {#each currentSettings.projectRoots as root (root)}
              <div class="path-chip">
                <Folder size={14} class="chip-icon text-accent" />
                <span class="chip-text" title={root}>{root}</span>
                <button
                  class="btn-remove-chip"
                  onclick={() => handleRemoveRootFolder(root)}
                  title="Hapus folder project"
                >
                  <X size={12} />
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </section>
  </div>
</div>

<style>
  .settings-page {
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

  .banner-error {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    background: rgba(220, 38, 38, 0.08);
    border: 1px solid rgba(220, 38, 38, 0.25);
    border-radius: var(--radius-md);
    color: var(--color-danger);
    font-size: 0.875rem;
  }

  .settings-sections {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .settings-card {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border-color);
  }

  .section-icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 8px;
    background-color: var(--accent-primary-light);
    color: var(--accent-primary);
  }

  .card-header h3 {
    font-size: 1rem;
    font-weight: 600;
  }

  .card-header p {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-label {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
  }

  .setting-desc {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .toggle-input {
    width: 20px;
    height: 20px;
    cursor: pointer;
    accent-color: var(--accent-primary);
  }

  .input-with-unit {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .number-input {
    width: 80px;
    padding: 6px 10px;
    border-radius: 6px;
    border: 1px solid var(--border-color);
    background-color: var(--bg-input);
    color: var(--text-primary);
    font-family: inherit;
    font-size: 0.875rem;
    text-align: right;
  }

  .number-input:focus {
    outline: none;
    border-color: var(--accent-primary);
  }

  .unit-label {
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .path-management {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .add-path-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .text-input {
    flex: 1;
    min-width: 240px;
    padding: 8px 12px;
    border-radius: 6px;
    border: 1px solid var(--border-color);
    background-color: var(--bg-input);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .text-input:focus {
    outline: none;
    border-color: var(--accent-primary);
  }

  .empty-paths-note {
    font-size: 0.85rem;
    color: var(--text-muted);
    font-style: italic;
    padding: 4px 0;
  }

  .chips-container {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .path-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-family: monospace;
    max-width: 100%;
  }

  .chip-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 400px;
    color: var(--text-primary);
  }

  .btn-remove-chip {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: none;
    background: var(--bg-hover);
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-remove-chip:hover {
    background: rgba(220, 38, 38, 0.15);
    color: var(--color-danger);
  }
</style>
