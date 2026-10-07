<script lang="ts">
  import { page } from '$app/stores';
  import { id } from '$lib/i18n/id';
  import {
    LayoutDashboard,
    FolderGit2,
    History,
    Settings as SettingsIcon,
    ShieldAlert,
  } from 'lucide-svelte';

  interface Props {
    dryRun?: boolean;
    hasFullDiskAccess?: boolean;
  }

  let { dryRun = true, hasFullDiskAccess = true }: Props = $props();

  const navItems = [
    { href: '/', label: id.nav.dashboard, icon: LayoutDashboard },
    { href: '/projects', label: id.nav.projects, icon: FolderGit2 },
    { href: '/history', label: id.nav.history, icon: History },
    { href: '/settings', label: id.nav.settings, icon: SettingsIcon },
  ];
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="logo-wrapper">
      <img src="/logo.svg" alt="Fug Cleaner" class="logo-img" />
    </div>
    <div class="app-info">
      <h1 class="app-title">{id.app.name}</h1>
      <span class="app-version">v0.1.0</span>
    </div>
  </div>

  <nav class="sidebar-nav">
    {#each navItems as item}
      {@const isActive = $page.url.pathname === item.href}
      <a
        href={item.href}
        class="nav-link"
        class:active={isActive}
        aria-current={isActive ? 'page' : undefined}
      >
        <item.icon size={18} class="nav-icon" />
        <span class="nav-label">{item.label}</span>
      </a>
    {/each}
  </nav>

  <div class="sidebar-footer">
    {#if !hasFullDiskAccess}
      <div class="fda-warning" title={id.fullDiskAccess.bannerTitle}>
        <ShieldAlert size={14} />
        <span>Akses Terbatas</span>
      </div>
    {/if}

    {#if dryRun}
      <div class="badge badge-caution dry-run-badge" title={id.dashboard.dryRunNote}>
        <span>DRY RUN</span>
      </div>
    {/if}

    <span class="copyright">Fug Cleaner © 2026</span>
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
    height: 100vh;
    background-color: var(--bg-sidebar);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    padding: 20px 16px;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 8px 24px;
    border-bottom: 1px solid var(--border-color);
  }

  .logo-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: 8px;
    overflow: hidden;
    flex-shrink: 0;
  }

  .logo-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .app-info {
    display: flex;
    flex-direction: column;
  }

  .app-title {
    font-size: 1.05rem;
    font-weight: 700;
    line-height: 1.2;
    color: var(--text-primary);
  }

  .app-version {
    font-size: 0.7rem;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 16px;
    flex: 1;
  }

  .nav-link {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 8px;
    color: var(--text-secondary);
    text-decoration: none;
    font-size: 0.875rem;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .nav-link:hover {
    background-color: var(--bg-card-hover);
    color: var(--text-primary);
  }

  .nav-link.active {
    background-color: var(--accent-primary-light);
    color: var(--accent-primary);
    font-weight: 600;
  }

  :global(.nav-icon) {
    flex-shrink: 0;
  }

  .sidebar-footer {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color);
  }

  .fda-warning {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: var(--accent-warning);
    background-color: var(--accent-warning-light);
    padding: 6px 10px;
    border-radius: 6px;
    font-weight: 500;
  }

  .dry-run-badge {
    align-self: flex-start;
  }

  .copyright {
    font-size: 0.7rem;
    color: var(--text-muted);
  }
</style>
