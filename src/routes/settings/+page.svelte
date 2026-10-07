<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Settings } from '$lib/types';
  import { id } from '$lib/i18n/id';
  import { Shield, Sliders, Save, Check, Folder } from 'lucide-svelte';

  let currentSettings = $state<Settings>({
    projectRoots: [],
    staleThresholdDays: 30,
    enabledCategories: [],
    excludePaths: [],
    maxScanDepth: 6,
    dryRun: true,
  });

  let saved = $state(false);
  let saving = $state(false);

  async function loadSettings() {
    try {
      currentSettings = await api.getSettings();
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async function handleSave() {
    saving = true;
    try {
      await api.saveSettings(currentSettings);
      saved = true;
      setTimeout(() => {
        saved = false;
      }, 2000);
    } catch (e) {
      console.error('Failed to save settings:', e);
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

  <div class="settings-sections">
    <!-- Safety & Dry-run Section -->
    <section class="card settings-card">
      <div class="card-header">
        <div class="section-icon-wrapper">
          <Shield size={20} />
        </div>
        <div>
          <h3>Keamanan & Simulasi</h3>
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
</style>
