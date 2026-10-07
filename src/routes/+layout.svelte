<script lang="ts">
  import '../app.css';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { api } from '$lib/api';
  import { onMount } from 'svelte';

  let { children } = $props();

  let hasFullDiskAccess = $state(true);
  let dryRun = $state(true);

  onMount(async () => {
    try {
      hasFullDiskAccess = await api.checkFullDiskAccess();
    } catch {
      hasFullDiskAccess = true;
    }
  });
</script>

<div class="app-container">
  <Sidebar {dryRun} {hasFullDiskAccess} />
  <main class="main-content">
    {@render children()}
  </main>
</div>
