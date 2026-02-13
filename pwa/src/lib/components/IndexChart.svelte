<script>
  import { LayerCake, Svg } from 'layercake';
  import Line from './chart/Line.svelte';
  import AxisX from './chart/AxisX.svelte';
  import AxisY from './chart/AxisY.svelte';
  import Tooltip from './chart/Tooltip.svelte';
  import BenchmarkLines from './chart/BenchmarkLines.svelte';

  let { history, benchmarkHistory = {}, benchmarkSymbols = [] } = $props();

  const periods = [
    { label: '1W', hours: 168 },
    { label: '1M', hours: 730 },
    { label: '3M', hours: 2190 },
    { label: '6M', hours: 4380 },
    { label: '1Y', hours: 8760 },
    { label: 'All', hours: null },
  ];
  let selectedPeriod = $state('All');

  const benchmarkColors = {
    SPY: '#ff9800',
    QQQ: '#e040fb',
    SMH: '#76ff03',
    BOTZ: '#ff5252',
    AIQ: '#40c4ff',
    ARKQ: '#ffd740',
  };

  let enabledBenchmarks = $state(new Set());

  function toggleBenchmark(sym) {
    const next = new Set(enabledBenchmarks);
    if (next.has(sym)) next.delete(sym);
    else next.add(sym);
    enabledBenchmarks = next;
  }

  let comparisonMode = $derived(enabledBenchmarks.size > 0);

  function filterAndDownsample(items, timestampKey, valueKey) {
    const period = periods.find((p) => p.label === selectedPeriod);
    let filtered = [...items].sort(
      (a, b) => new Date(a[timestampKey]) - new Date(b[timestampKey]),
    );

    if (period?.hours) {
      const cutoff = new Date(Date.now() - period.hours * 3600_000);
      filtered = filtered.filter((d) => new Date(d[timestampKey]) >= cutoff);
    }

    if (!period?.hours || period.hours > 168) {
      const byDay = new Map();
      for (const d of filtered) {
        const day = d[timestampKey].slice(0, 10);
        byDay.set(day, d);
      }
      filtered = [...byDay.values()];
    }

    return filtered;
  }

  let chartData = $derived.by(() => {
    if (!history || history.length === 0) return [];

    const filtered = filterAndDownsample(history, 'timestamp', 'value');

    if (comparisonMode && filtered.length > 0) {
      const baseValue = filtered[0].value;
      return filtered.map((d) => ({
        date: new Date(d.timestamp),
        value: ((d.value / baseValue) - 1) * 100,
        rawValue: d.value,
        dateLabel: new Date(d.timestamp).toLocaleString(),
      }));
    }

    return filtered.map((d) => ({
      date: new Date(d.timestamp),
      value: d.value,
      dateLabel: new Date(d.timestamp).toLocaleString(),
    }));
  });

  let benchmarkSeries = $derived.by(() => {
    if (!comparisonMode) return {};
    const result = {};

    for (const sym of enabledBenchmarks) {
      const raw = benchmarkHistory[sym] || [];
      if (raw.length === 0) continue;

      const filtered = filterAndDownsample(raw, 'timestamp', 'price');
      if (filtered.length === 0) continue;

      const base = filtered[0].price;
      result[sym] = filtered.map((d) => ({
        date: new Date(d.timestamp),
        value: ((d.price / base) - 1) * 100,
      }));
    }

    return result;
  });

  let yFormatFn = $derived(
    comparisonMode
      ? (v) => `${v >= 0 ? '+' : ''}${v.toFixed(1)}%`
      : null,
  );
</script>

<div class="controls">
  <div class="period-selector">
    {#each periods as p}
      <button
        class:active={selectedPeriod === p.label}
        onclick={() => (selectedPeriod = p.label)}
      >
        {p.label}
      </button>
    {/each}
  </div>

  {#if benchmarkSymbols.length > 0}
    <div class="benchmark-toggles">
      <span class="compare-label">Compare:</span>
      {#each benchmarkSymbols as sym}
        <button
          class="bench-btn"
          class:bench-active={enabledBenchmarks.has(sym)}
          style:--bench-color={benchmarkColors[sym] || '#888'}
          onclick={() => toggleBenchmark(sym)}
        >
          {sym}
        </button>
      {/each}
    </div>
  {/if}
</div>

{#if chartData.length > 0}
  <div class="chart-container">
    <LayerCake
      data={chartData}
      x="date"
      y="value"
      padding={{ top: 10, right: 10, bottom: 30, left: 55 }}
    >
      <Svg>
        <AxisX />
        <AxisY formatValue={yFormatFn} />
        <Line />
        {#if comparisonMode}
          <BenchmarkLines series={benchmarkSeries} colors={benchmarkColors} />
        {/if}
        <Tooltip {comparisonMode} {benchmarkSeries} {benchmarkColors} />
      </Svg>
    </LayerCake>
  </div>

  {#if comparisonMode}
    <div class="legend">
      <span class="legend-item">
        <span class="legend-line" style="background: var(--accent);"></span>
        AI Index
      </span>
      {#each [...enabledBenchmarks] as sym}
        <span class="legend-item">
          <span class="legend-line legend-dashed" style:--lc={benchmarkColors[sym] || '#888'}></span>
          {sym}
        </span>
      {/each}
    </div>
  {/if}
{:else}
  <div class="no-data">No history data available</div>
{/if}

<style>
  .chart-container {
    width: 100%;
    height: 300px;
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-bottom: 1rem;
    align-items: center;
  }

  .period-selector {
    display: flex;
    gap: 0.5rem;
  }

  .period-selector button {
    padding: 0.25rem 0.75rem;
    background: var(--bg-card);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .period-selector button.active {
    background: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
  }

  .benchmark-toggles {
    display: flex;
    gap: 0.35rem;
    align-items: center;
  }

  .compare-label {
    color: var(--text-secondary);
    font-size: 0.8rem;
    margin-right: 0.15rem;
  }

  .bench-btn {
    padding: 0.2rem 0.5rem;
    background: var(--bg-card);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.7rem;
    font-weight: 500;
    transition: all 0.15s;
  }

  .bench-btn.bench-active {
    background: color-mix(in srgb, var(--bench-color) 20%, transparent);
    color: var(--bench-color);
    border-color: var(--bench-color);
  }

  .legend {
    display: flex;
    gap: 1rem;
    margin-top: 0.5rem;
    flex-wrap: wrap;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .legend-line {
    display: inline-block;
    width: 18px;
    height: 2px;
  }

  .legend-dashed {
    background: repeating-linear-gradient(
      to right,
      var(--lc),
      var(--lc) 4px,
      transparent 4px,
      transparent 7px
    );
  }

  .no-data {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary);
  }
</style>
