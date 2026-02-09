<script>
  let { data } = $props();

  let value = $derived(data?.value);
  let change = $derived(data?.daily_change);
  let changePct = $derived(data?.daily_change_pct);
  let isPositive = $derived(change != null && change >= 0);
</script>

<div class="banner">
  {#if value != null}
    <div class="value">{value.toFixed(2)}</div>
    <div class="change" class:positive={isPositive} class:negative={!isPositive}>
      {#if change != null}
        <span>{change >= 0 ? '+' : ''}{change.toFixed(2)}</span>
        <span class="pct">({changePct >= 0 ? '+' : ''}{changePct?.toFixed(2)}%)</span>
      {:else}
        <span class="no-data">--</span>
      {/if}
    </div>
  {:else}
    <div class="value no-data">Waiting for data...</div>
  {/if}
</div>

<style>
  .banner {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
    text-align: center;
  }

  .value {
    font-size: 3rem;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--accent);
  }

  .change {
    font-size: 1.25rem;
    font-family: var(--font-mono);
    margin-top: 0.25rem;
  }

  .pct {
    margin-left: 0.5rem;
    font-size: 1rem;
  }

  .no-data {
    color: var(--text-secondary);
    font-size: 1.25rem;
  }
</style>
