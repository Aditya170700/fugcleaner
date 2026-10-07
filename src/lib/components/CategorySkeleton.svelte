<div class="skeleton-grid">
  {#each Array(6) as _, i (i)}
    <div class="card skeleton-card">
      <div class="skeleton-top">
        <div class="skeleton-checkbox shimmer"></div>
        <div class="skeleton-title-group">
          <div class="skeleton-line title shimmer"></div>
          <div class="skeleton-line desc shimmer"></div>
        </div>
        <div class="skeleton-badge shimmer"></div>
      </div>
      <div class="skeleton-bottom">
        <div class="skeleton-line size shimmer"></div>
        <div class="skeleton-line btn shimmer"></div>
      </div>
    </div>
  {/each}
</div>

<style>
  .skeleton-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 16px;
  }

  .skeleton-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 20px;
    padding: 16px;
    height: 120px;
    pointer-events: none;
  }

  .skeleton-top {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .skeleton-checkbox {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .skeleton-title-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }

  .skeleton-line {
    border-radius: 4px;
    background: var(--bg-hover);
  }

  .skeleton-line.title {
    width: 60%;
    height: 16px;
  }

  .skeleton-line.desc {
    width: 90%;
    height: 12px;
  }

  .skeleton-badge {
    width: 50px;
    height: 18px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .skeleton-bottom {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 8px;
    border-top: 1px solid var(--border-color);
  }

  .skeleton-line.size {
    width: 80px;
    height: 16px;
  }

  .skeleton-line.btn {
    width: 50px;
    height: 14px;
  }

  /* Shimmer effect */
  .shimmer {
    background: linear-gradient(
      90deg,
      var(--bg-secondary) 0%,
      var(--bg-hover) 50%,
      var(--bg-secondary) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite linear;
  }

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }
</style>
