<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { DiskInfo } from '$lib/types';
  import { id } from '$lib/i18n/id';
  import DiskBar from '$lib/components/DiskBar.svelte';
  import {
    Sparkles,
    FolderSearch,
    ShieldAlert,
    Trash2,
    Layers,
    Clock,
    ArrowRight,
  } from 'lucide-svelte';

  let diskInfo = $state<DiskInfo | null>(null);
  let loadingDisk = $state(true);
  let hasFullDiskAccess = $state(true);

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

  async function checkPermissions() {
    try {
      hasFullDiskAccess = await api.checkFullDiskAccess();
    } catch {
      hasFullDiskAccess = true;
    }
  }

  onMount(() => {
    loadDiskInfo();
    checkPermissions();
  });
</script>

<div class="dashboard-page">
  <!-- Page Header -->
  <header class="page-header">
    <div>
      <h1 class="page-title">{id.dashboard.title}</h1>
      <p class="page-subtitle">{id.dashboard.subtitle}</p>
    </div>
  </header>

  <!-- Full Disk Access Warning Banner (if macOS & missing permission) -->
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

  <!-- Disk Capacity Bar -->
  <section class="section">
    <DiskBar {diskInfo} loading={loadingDisk} onRefresh={loadDiskInfo} />
  </section>

  <!-- Quick Action & Category Cards Overview -->
  <section class="section">
    <div class="section-header">
      <h2>{id.dashboard.categoriesTitle}</h2>
    </div>

    <div class="feature-grid">
      <div class="card feature-card card-hover">
        <div class="feature-icon-wrapper bg-blue">
          <Sparkles size={22} />
        </div>
        <div class="feature-body">
          <h3>Cache Global Developer</h3>
          <p>npm, pnpm, yarn, bun, cargo registry, xcode derived data, & build tools.</p>
        </div>
        <div class="feature-footer">
          <span class="badge badge-safe">Otomatis Terdeteksi</span>
        </div>
      </div>

      <div class="card feature-card card-hover">
        <div class="feature-icon-wrapper bg-purple">
          <FolderSearch size={22} />
        </div>
        <div class="feature-body">
          <h3>Artifact Project</h3>
          <p>node_modules lama, target Rust, dist, .next, .nuxt, & python venv.</p>
        </div>
        <div class="feature-footer">
          <a href="/projects" class="link-btn">
            <span>Pindai Folder</span>
            <ArrowRight size={14} />
          </a>
        </div>
      </div>

      <div class="card feature-card card-hover">
        <div class="feature-icon-wrapper bg-emerald">
          <Trash2 size={22} />
        </div>
        <div class="feature-body">
          <h3>Pembersihan Aman</h3>
          <p>Default memindahkan ke Trash sehingga file dapat dipulihkan kapan saja.</p>
        </div>
        <div class="feature-footer">
          <span class="badge badge-safe">Bisa di-undo</span>
        </div>
      </div>
    </div>
  </section>
</div>

<style>
  .dashboard-page {
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 1100px;
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
  }

  .section-header h2 {
    font-size: 1.15rem;
    font-weight: 600;
  }

  .feature-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;
  }

  .feature-card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    cursor: default;
  }

  .feature-icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 42px;
    height: 42px;
    border-radius: 10px;
  }

  .bg-blue {
    background-color: var(--accent-primary-light);
    color: var(--accent-primary);
  }

  .bg-purple {
    background-color: rgba(139, 92, 246, 0.15);
    color: #8b5cf6;
  }

  .bg-emerald {
    background-color: var(--accent-success-light);
    color: var(--accent-success);
  }

  .feature-body h3 {
    font-size: 0.95rem;
    margin-bottom: 4px;
  }

  .feature-body p {
    font-size: 0.825rem;
    line-height: 1.4;
  }

  .feature-footer {
    margin-top: auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .link-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 0.825rem;
    font-weight: 600;
    color: var(--accent-primary);
    text-decoration: none;
    transition: gap 0.2s ease;
  }

  .link-btn:hover {
    gap: 10px;
  }
</style>
