<script>
  let { stocks } = $props();

  let sortKey = $state('symbol');
  let sortDir = $state(1); // 1 = asc, -1 = desc

  function sort(key) {
    if (sortKey === key) {
      sortDir = sortDir * -1;
    } else {
      sortKey = key;
      sortDir = key === 'symbol' || key === 'sector_label' ? 1 : -1;
    }
  }

  let sorted = $derived.by(() => {
    return [...stocks].sort((a, b) => {
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
      {#each sorted as stock (stock.symbol)}
        <tr>
          <td class="symbol">{stock.symbol}</td>
          <td class="sector">{stock.sector_label}</td>
          <td>${fmt(stock.price)}</td>
          <td class:positive={stock.change_pct >= 0} class:negative={stock.change_pct < 0}>
            {fmtPct(stock.change_pct)}
          </td>
          <td>{fmtMcap(stock.market_cap)}</td>
          <td>{fmtWeight(stock.weight)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

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

  .sector {
    color: var(--text-secondary);
    font-family: inherit;
    font-size: 0.8rem;
  }

  tr:hover td {
    background: rgba(0, 212, 255, 0.05);
  }
</style>
