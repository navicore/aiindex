<script>
  import { LayerCake, Svg } from 'layercake';
  import Line from './chart/Line.svelte';
  import AxisX from './chart/AxisX.svelte';
  import AxisY from './chart/AxisY.svelte';
  import Tooltip from './chart/Tooltip.svelte';

  let { history } = $props();

  const periods = [
    { label: '1W', hours: 168 },
    { label: '1M', hours: 730 },
    { label: '3M', hours: 2190 },
    { label: '6M', hours: 4380 },
    { label: '1Y', hours: 8760 },
    { label: 'All', hours: null },
  ];
  let selectedPeriod = $state('All');

  let chartData = $derived.by(() => {
    if (!history || history.length === 0) return [];

    const period = periods.find((p) => p.label === selectedPeriod);
    let filtered = [...history].reverse(); // oldest first

    if (period?.hours) {
      const cutoff = new Date(Date.now() - period.hours * 3600_000);
      filtered = filtered.filter((d) => new Date(d.timestamp) >= cutoff);
    }

    // For periods longer than 1 week, downsample to one point per day (last of each day).
    if (!period?.hours || period.hours > 168) {
      const byDay = new Map();
      for (const d of filtered) {
        const day = d.timestamp.slice(0, 10); // YYYY-MM-DD
        byDay.set(day, d); // last entry for each day wins
      }
      filtered = [...byDay.values()];
    }

    return filtered.map((d) => ({
      date: new Date(d.timestamp),
      value: d.value,
      dateLabel: new Date(d.timestamp).toLocaleString(),
    }));
  });
</script>

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

{#if chartData.length > 0}
  <div class="chart-container">
    <LayerCake
      data={chartData}
      x="date"
      y="value"
      padding={{ top: 10, right: 10, bottom: 30, left: 50 }}
    >
      <Svg>
        <AxisX />
        <AxisY />
        <Line />
        <Tooltip />
      </Svg>
    </LayerCake>
  </div>
{:else}
  <div class="no-data">No history data available</div>
{/if}

<style>
  .chart-container {
    width: 100%;
    height: 300px;
  }

  .period-selector {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
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

  .no-data {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary);
  }
</style>
