<script>
  import { getIndex, getStocks, getSectors, getIndexHistory } from '$lib/api.js';
  import IndexBanner from '$lib/components/IndexBanner.svelte';
  import IndexChart from '$lib/components/IndexChart.svelte';
  import SectorBreakdown from '$lib/components/SectorBreakdown.svelte';
  import StockTable from '$lib/components/StockTable.svelte';

  let indexData = $state(null);
  let stocks = $state([]);
  let sectors = $state([]);
  let history = $state([]);
  let loading = $state(true);
  let error = $state(null);

  const REFRESH_INTERVAL = 60_000; // 1 minute

  async function loadData() {
    try {
      const [idx, stk, sec, hist] = await Promise.all([
        getIndex(),
        getStocks(),
        getSectors(),
        getIndexHistory(500),
      ]);
      indexData = idx;
      stocks = stk;
      sectors = sec;
      history = hist;
      error = null;
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadData();
    const interval = setInterval(loadData, REFRESH_INTERVAL);
    return () => clearInterval(interval);
  });
</script>

<header>
  <h1>AI Index Tracker</h1>
  <p class="subtitle">Experimental AI Stock Composite</p>
</header>

{#if loading}
  <div class="loading">Loading market data...</div>
{:else if error}
  <div class="card error-card">
    <p>Failed to load data: {error}</p>
    <button onclick={loadData}>Retry</button>
  </div>
{:else}
  <div class="dashboard">
    <IndexBanner data={indexData} />

    <div class="chart-section card">
      <h2>Index History</h2>
      <IndexChart {history} />
    </div>

    <div class="sectors-section card">
      <h2>Sector Breakdown</h2>
      <SectorBreakdown {sectors} />
    </div>

    <div class="table-section card">
      <h2>Stocks</h2>
      <StockTable {stocks} />
    </div>
  </div>
{/if}

<style>
  header {
    text-align: center;
    padding: 1.5rem 0 1rem;
  }

  header h1 {
    font-size: 1.75rem;
    color: var(--accent);
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin-top: 0.25rem;
  }

  .dashboard {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .chart-section h2,
  .sectors-section h2,
  .table-section h2 {
    font-size: 1rem;
    margin-bottom: 1rem;
    color: var(--text-secondary);
  }

  .error-card {
    text-align: center;
    color: var(--red);
  }

  .error-card button {
    margin-top: 0.75rem;
    padding: 0.5rem 1.5rem;
    background: var(--bg-card);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
  }
</style>
