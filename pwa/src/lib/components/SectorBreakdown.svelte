<script>
  let { sectors } = $props();

  const colors = [
    '#00d4ff', '#7c4dff', '#00e676', '#ff9100',
    '#ff5252', '#ffea00', '#18ffff',
  ];

  let maxWeight = $derived(
    Math.max(...(sectors || []).map((s) => s.total_weight), 0.01)
  );
</script>

{#if sectors && sectors.length > 0}
  <div class="breakdown">
    {#each sectors as sector, i (sector.key)}
      <div class="sector-row">
        <div class="sector-info">
          <span class="sector-label">{sector.label}</span>
          <span class="sector-stats">
            <span class="weight">{(sector.total_weight * 100).toFixed(1)}%</span>
            <span class:positive={sector.avg_change_pct >= 0} class:negative={sector.avg_change_pct < 0}>
              {sector.avg_change_pct >= 0 ? '+' : ''}{sector.avg_change_pct.toFixed(2)}%
            </span>
          </span>
        </div>
        <div class="bar-bg">
          <div
            class="bar-fill"
            style="width: {(sector.total_weight / maxWeight) * 100}%; background: {colors[i % colors.length]}"
          ></div>
        </div>
        <div class="symbols">
          {#each sector.symbols as sym}
            <span class="chip">{sym}</span>
          {/each}
        </div>
      </div>
    {/each}
  </div>
{:else}
  <div class="no-data">No sector data available</div>
{/if}

<style>
  .breakdown {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .sector-info {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 0.25rem;
  }

  .sector-label {
    font-weight: 600;
    font-size: 0.875rem;
  }

  .sector-stats {
    display: flex;
    gap: 0.75rem;
    font-family: var(--font-mono);
    font-size: 0.8rem;
  }

  .weight {
    color: var(--text-secondary);
  }

  .bar-bg {
    background: var(--bg-primary);
    border-radius: 4px;
    height: 8px;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.5s ease;
  }

  .symbols {
    display: flex;
    gap: 0.25rem;
    flex-wrap: wrap;
    margin-top: 0.25rem;
  }

  .chip {
    font-size: 0.7rem;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    background: var(--bg-primary);
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
  }

  .no-data {
    text-align: center;
    padding: 1rem;
    color: var(--text-secondary);
  }
</style>
