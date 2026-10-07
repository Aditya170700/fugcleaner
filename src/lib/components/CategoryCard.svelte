<script lang="ts">
  import type { Category } from '$lib/types';
  import { formatBytes } from '$lib/utils/format';
  import { id } from '$lib/i18n/id';
  import {
    ChevronRight,
    Terminal,
    Trash,
    Trash2,
    Info,
    CheckCircle2,
    AlertTriangle,
  } from 'lucide-svelte';

  interface Props {
    category: Category;
    bytes: number;
    itemCount: number;
    selected: boolean;
    onToggleSelect: (selected: boolean) => void;
    onOpenDetails: () => void;
  }

  let {
    category,
    bytes,
    itemCount,
    selected,
    onToggleSelect,
    onOpenDetails,
  }: Props = $props();

  const isZero = $derived(bytes === 0 && itemCount === 0);
  const isDisabled = $derived(!category.available);

  function handleCheckboxClick(e: MouseEvent) {
    e.stopPropagation();
    if (!isDisabled && !isZero) {
      onToggleSelect(!selected);
    }
  }

  function handleCardClick(e: MouseEvent) {
    if (itemCount > 0) {
      onOpenDetails();
    }
  }
</script>

<div
  class="card category-card"
  class:selected={selected && !isZero && !isDisabled}
  class:is-zero={isZero && category.available}
  class:is-disabled={isDisabled}
  onclick={handleCardClick}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === 'Enter' && handleCardClick(e as any)}
>
  <div class="card-top">
    <div class="checkbox-and-title">
      <!-- Custom Checkbox -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <label class="checkbox-container" onclick={(e) => e.stopPropagation()}>
        <input
          type="checkbox"
          checked={selected && !isZero && !isDisabled}
          disabled={isDisabled || isZero}
          onchange={(e) => onToggleSelect((e.currentTarget as HTMLInputElement).checked)}
        />
        <span class="checkmark"></span>
      </label>

      <div class="title-group">
        <h3 class="category-name">{category.name}</h3>
        <p class="category-desc">{category.description}</p>
      </div>
    </div>

    <!-- Badges (Risk & Method) -->
    <div class="badge-group">
      {#if category.risk === 'safe'}
        <span class="badge badge-safe">{id.dashboard.riskSafe}</span>
      {:else}
        <span class="badge badge-caution">{id.dashboard.riskCaution}</span>
      {/if}

      {#if category.method === 'command'}
        <span class="badge badge-muted" title="Menjalankan CLI">CLI</span>
      {:else if category.method === 'trash'}
        <span class="badge badge-muted" title="Pindah ke Trash">Trash</span>
      {/if}
    </div>
  </div>

  <div class="card-bottom">
    {#if !category.available}
      <div class="unavailable-note">
        <Info size={14} />
        <span>{category.unavailableReason || 'Tidak tersedia'}</span>
      </div>
    {:else}
      <div class="size-stats">
        <span class="bytes-value" class:text-muted={isZero}>{formatBytes(bytes)}</span>
        {#if itemCount > 0}
          <span class="items-count">({itemCount} item)</span>
        {/if}
      </div>

      {#if itemCount > 0}
        <button
          class="btn-details"
          onclick={(e) => {
            e.stopPropagation();
            onOpenDetails();
          }}
          title="Lihat rincian item"
        >
          <span>Detail</span>
          <ChevronRight size={14} />
        </button>
      {/if}
    {/if}
  </div>
</div>

<style>
  .category-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 16px;
    cursor: pointer;
    position: relative;
    user-select: none;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .category-card:hover:not(.is-disabled) {
    border-color: var(--border-hover);
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  .category-card.selected {
    border-color: var(--accent-primary);
    background-color: var(--bg-card);
    box-shadow: 0 0 0 1px var(--accent-primary), var(--shadow-sm);
  }

  .category-card.is-zero {
    opacity: 0.75;
  }

  .category-card.is-disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background-color: var(--bg-subtle);
  }

  .card-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .checkbox-and-title {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  /* Custom Checkbox */
  .checkbox-container {
    position: relative;
    padding-top: 2px;
    cursor: pointer;
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
    height: 18px;
    width: 18px;
    background-color: var(--bg-card);
    border: 1.5px solid var(--border-hover);
    border-radius: 5px;
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
    left: 6px;
    top: 5px;
    width: 4px;
    height: 8px;
    border: solid white;
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }

  .checkbox-container input:checked ~ .checkmark:after {
    display: block;
  }

  .checkbox-container input:disabled ~ .checkmark {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .title-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .category-name {
    font-size: 0.95rem;
    font-weight: 600;
  }

  .category-desc {
    font-size: 0.785rem;
    color: var(--text-secondary);
    line-height: 1.35;
  }

  .badge-group {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .card-bottom {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 8px;
    border-top: 1px solid var(--border-color);
  }

  .size-stats {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }

  .bytes-value {
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .items-count {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .unavailable-note {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .btn-details {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: none;
    border: none;
    font-size: 0.785rem;
    font-weight: 600;
    color: var(--accent-primary);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
    transition: background-color 0.15s ease;
  }

  .btn-details:hover {
    background-color: var(--accent-primary-light);
  }

  .text-muted {
    color: var(--text-muted) !important;
  }
</style>
