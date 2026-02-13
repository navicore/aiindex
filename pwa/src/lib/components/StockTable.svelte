<script>
  let { stocks } = $props();

  let sortKey = $state('symbol');
  let sortDir = $state(1); // 1 = asc, -1 = desc
  let expanded = $state(null); // symbol of expanded row

  function sort(key) {
    if (sortKey === key) {
      sortDir = sortDir * -1;
    } else {
      sortKey = key;
      sortDir = key === 'symbol' || key === 'sector_label' ? 1 : -1;
    }
  }

  function toggle(symbol) {
    expanded = expanded === symbol ? null : symbol;
  }

  // Split into index stocks and benchmarks.
  let indexStocks = $derived(stocks.filter((s) => s.sector !== 'benchmarks'));
  let benchmarkStocks = $derived(stocks.filter((s) => s.sector === 'benchmarks'));

  let sortedIndex = $derived.by(() => {
    return [...indexStocks].sort((a, b) => {
      let av = a[sortKey];
      let bv = b[sortKey];
      if (av == null) av = -Infinity;
      if (bv == null) bv = -Infinity;
      if (typeof av === 'string') return av.localeCompare(bv) * sortDir;
      return (av - bv) * sortDir;
    });
  });

  let sortedBenchmarks = $derived.by(() => {
    return [...benchmarkStocks].sort((a, b) => {
      let av = a[sortKey];
      let bv = b[sortKey];
      if (av == null) av = -Infinity;
      if (bv == null) bv = -Infinity;
      if (typeof av === 'string') return av.localeCompare(bv) * sortDir;
      return (av - bv) * sortDir;
    });
  });

  function fmt(n, decimals = 2) {
    if (n == null) return '--';
    return n.toFixed(decimals);
  }

  function fmtPct(n) {
    if (n == null) return '--';
    return `${n >= 0 ? '+' : ''}${n.toFixed(2)}%`;
  }

  function fmtMcap(n) {
    if (n == null) return '--';
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}T`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}B`;
    return `${n.toFixed(0)}M`;
  }

  function fmtWeight(n) {
    if (n == null) return '--';
    return `${(n * 100).toFixed(2)}%`;
  }
</script>

<div class="table-wrap">
  <table>
    <thead>
      <tr>
        <th class="sortable" onclick={() => sort('symbol')}>
          Symbol {sortKey === 'symbol' ? (sortDir === 1 ? '▲' : '▼') : ''}
        </th>
        <th class="sortable" onclick={() => sort('sector_label')}>
          Sector {sortKey === 'sector_label' ? (sortDir === 1 ? '▲' : '▼') : ''}
        </th>
        <th class="sortable" onclick={() => sort('price')}>
          Price {sortKey === 'price' ? (sortDir === 1 ? '▲' : '▼') : ''}
        </th>
        <th class="sortable" onclick={() => sort('change_pct')}>
          Change {sortKey === 'change_pct' ? (sortDir === 1 ? '▲' : '▼') : ''}
        </th>
        <th class="sortable" onclick={() => sort('market_cap')}>
          Mkt Cap {sortKey === 'market_cap' ? (sortDir === 1 ? '▲' : '▼') : ''}
        </th>
        <th class="sortable" onclick={() => sort('weight')}>
          Weight {sortKey === 'weight' ? (sortDir === 1 ? '▲' : '▼') : ''}
        </th>
      </tr>
    </thead>
    <tbody>
      {#each sortedIndex as stock (stock.symbol)}
        <tr class="stock-row" class:expanded-row={expanded === stock.symbol} onclick={() => toggle(stock.symbol)}>
          <td class="symbol">{stock.symbol}</td>
          <td class="sector">{stock.sector_label}</td>
          <td>${fmt(stock.price)}</td>
          <td class:positive={stock.change_pct >= 0} class:negative={stock.change_pct < 0}>
            {fmtPct(stock.change_pct)}
          </td>
          <td>{fmtMcap(stock.market_cap)}</td>
          <td>{fmtWeight(stock.weight)}</td>
        </tr>
        {#if expanded === stock.symbol}
          <tr class="detail-row">
            <td colspan="6">
              <div class="detail-panel">
                <div class="detail-header">
                  {#if stock.logo}
                    <img src={stock.logo} alt="" class="detail-logo" />
                  {/if}
                  <div>
                    <div class="detail-name">{stock.name || stock.symbol}</div>
                    <div class="detail-meta">
                      {#if stock.exchange}{stock.exchange}{/if}
                      {#if stock.exchange && stock.industry} &middot; {/if}
                      {#if stock.industry}{stock.industry}{/if}
                      {#if stock.country} &middot; {stock.country}{/if}
                    </div>
                  </div>
                </div>
                <div class="detail-links">
                  {#if stock.weburl}
                    <a href={stock.weburl} target="_blank" rel="noopener">Company Website</a>
                  {/if}
                  <a href="https://finance.yahoo.com/quote/{stock.symbol}" target="_blank" rel="noopener">Yahoo Finance</a>
                  <a href="https://www.google.com/finance/quote/{stock.symbol}:NASDAQ" target="_blank" rel="noopener">Google Finance</a>
                </div>
              </div>
            </td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>

{#if benchmarkStocks.length > 0}
  <div class="benchmark-section">
    <div class="benchmark-header">
      <h3>Benchmarks</h3>
      <span class="benchmark-note">Not included in index calculation</span>
    </div>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Symbol</th>
            <th>Price</th>
            <th>Change</th>
            <th>Mkt Cap</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedBenchmarks as stock (stock.symbol)}
            <tr class="stock-row" class:expanded-row={expanded === stock.symbol} onclick={() => toggle(stock.symbol)}>
              <td class="symbol benchmark-symbol">{stock.symbol}</td>
              <td>${fmt(stock.price)}</td>
              <td class:positive={stock.change_pct >= 0} class:negative={stock.change_pct < 0}>
                {fmtPct(stock.change_pct)}
              </td>
              <td>{fmtMcap(stock.market_cap)}</td>
            </tr>
            {#if expanded === stock.symbol}
              <tr class="detail-row">
                <td colspan="4">
                  <div class="detail-panel">
                    <div class="detail-header">
                      {#if stock.logo}
                        <img src={stock.logo} alt="" class="detail-logo" />
                      {/if}
                      <div>
                        <div class="detail-name">{stock.name || stock.symbol}</div>
                        <div class="detail-meta">
                          {#if stock.exchange}{stock.exchange}{/if}
                          {#if stock.exchange && stock.industry} &middot; {/if}
                          {#if stock.industry}{stock.industry}{/if}
                          {#if stock.country} &middot; {stock.country}{/if}
                        </div>
                      </div>
                    </div>
                    <div class="detail-links">
                      {#if stock.weburl}
                        <a href={stock.weburl} target="_blank" rel="noopener">Company Website</a>
                      {/if}
                      <a href="https://finance.yahoo.com/quote/{stock.symbol}" target="_blank" rel="noopener">Yahoo Finance</a>
                      <a href="https://www.google.com/finance/quote/{stock.symbol}:NASDAQ" target="_blank" rel="noopener">Google Finance</a>
                    </div>
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>
  </div>
{/if}

<style>
  .table-wrap {
    overflow-x: auto;
  }

  .sortable {
    cursor: pointer;
    user-select: none;
  }

  .sortable:hover {
    color: var(--accent);
  }

  .symbol {
    font-weight: 600;
    color: var(--accent);
  }

  .benchmark-symbol {
    color: var(--text-secondary);
  }

  .sector {
    color: var(--text-secondary);
    font-family: inherit;
    font-size: 0.8rem;
  }

  .stock-row {
    cursor: pointer;
  }

  .stock-row:hover td {
    background: rgba(0, 212, 255, 0.05);
  }

  .expanded-row td {
    border-bottom: none;
  }

  .detail-row td {
    padding: 0;
    border-bottom: 1px solid var(--border);
  }

  .detail-panel {
    background: var(--bg-primary);
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .detail-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .detail-logo {
    width: 32px;
    height: 32px;
    border-radius: 4px;
    object-fit: contain;
    background: #fff;
  }

  .detail-name {
    font-weight: 600;
    font-size: 0.95rem;
  }

  .detail-meta {
    color: var(--text-secondary);
    font-size: 0.8rem;
    margin-top: 0.1rem;
  }

  .detail-links {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .detail-links a {
    color: var(--accent);
    text-decoration: none;
    font-size: 0.8rem;
  }

  .detail-links a:hover {
    text-decoration: underline;
  }

  .benchmark-section {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 2px solid var(--border);
  }

  .benchmark-header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }

  .benchmark-header h3 {
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .benchmark-note {
    font-size: 0.7rem;
    color: var(--text-secondary);
    opacity: 0.7;
    font-style: italic;
  }
</style>
