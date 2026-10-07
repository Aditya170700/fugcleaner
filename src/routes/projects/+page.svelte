<script lang="ts">
  import { id } from '$lib/i18n/id';
  import { FolderGit2, Plus, FolderSearch } from 'lucide-svelte';
</script>

<div class="projects-page">
  <header class="page-header">
    <div>
      <h1 class="page-title">{id.projects.title}</h1>
      <p class="page-subtitle">{id.projects.subtitle}</p>
    </div>

    <button class="btn btn-primary" onclick={() => {}}>
      <Plus size={16} />
      <span>{id.projects.addFolder}</span>
    </button>
  </header>

  <div class="card empty-card">
    <div class="empty-icon-wrapper">
      <FolderSearch size={40} />
    </div>
    <h3>{id.projects.noProjectsFound}</h3>
    <p>Tambahkan folder tempat Anda menyimpan project (misalnya <code>~/Projects</code> atau <code>~/Code</code>) untuk memindai artifact yang memakan disk.</p>
    <button class="btn btn-primary mt-4">
      <Plus size={16} />
      <span>{id.projects.addFolder}</span>
    </button>
  </div>
</div>

<style>
  .projects-page {
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
