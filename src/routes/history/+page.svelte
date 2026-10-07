<script lang="ts">
  import { id } from '$lib/i18n/id';
  import { History, Sparkles } from 'lucide-svelte';
</script>

<div class="history-page">
  <header class="page-header">
    <div>
      <h1 class="page-title">{id.history.title}</h1>
      <p class="page-subtitle">{id.history.subtitle}</p>
    </div>
  </header>

  <div class="card empty-card">
    <div class="empty-icon-wrapper">
      <History size={40} />
    </div>
    <h3>{id.history.noHistory}</h3>
    <p>Setelah Anda melakukan pembersihan file cache atau artifact, log riwayat dan total ruang yang dibebaskan akan dicatat di sini.</p>
  </div>
</div>

<style>
  .history-page {
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
</style>
