<script>
  import { getContext } from 'svelte';

  let { comparisonMode = false, benchmarkSeries = {}, benchmarkColors = {} } = $props();
  const { data, xGet, yGet, xScale, yScale, width, height } = getContext('LayerCake');

  let mouseX = $state(null);
  let tooltipData = $state(null);

  function handleMousemove(e) {
    const rect = e.currentTarget.getBoundingClientRect();
    mouseX = e.clientX - rect.left;

    const d = $data;
    if (!d || d.length === 0) return;

    let closest = d[0];
    let minDist = Infinity;
    for (const p of d) {
      const dist = Math.abs($xGet(p) - mouseX);
      if (dist < minDist) {
        minDist = dist;
        closest = p;
      }
    }
    tooltipData = closest;
  }

  function handleMouseleave() {
    tooltipData = null;
    mouseX = null;
  }

  // Find closest benchmark value for the hovered date.
  function getBenchmarkValues(targetDate) {
    if (!comparisonMode || !targetDate) return [];
    const t = targetDate.getTime();
    const results = [];
    for (const [sym, points] of Object.entries(benchmarkSeries)) {
      if (!points || points.length === 0) continue;
      let closest = points[0];
      let minDist = Infinity;
      for (const p of points) {
        const dist = Math.abs(p.date.getTime() - t);
        if (dist < minDist) {
          minDist = dist;
          closest = p;
        }
      }
      results.push({
        symbol: sym,
        value: closest.value,
        color: benchmarkColors[sym] || '#888',
      });
    }
    return results;
  }

  let benchValues = $derived(
    tooltipData ? getBenchmarkValues(tooltipData.date) : [],
  );

  let tooltipHeight = $derived(comparisonMode ? 50 + benchValues.length * 16 : 50);
  let tooltipWidth = $derived(comparisonMode ? 150 : 120);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<rect
  width={$width}
  height={$height}
  fill="transparent"
  onmousemove={handleMousemove}
  onmouseleave={handleMouseleave}
/>

{#if tooltipData}
  {@const x = $xGet(tooltipData)}
  {@const y = $yGet(tooltipData)}
  <circle cx={x} cy={y} r="4" fill="var(--accent)" />
  <line x1={x} y1="0" x2={x} y2={$height} stroke="var(--accent)" stroke-width="1" opacity="0.3" />

  {#if comparisonMode}
    {#each benchValues as bv}
      {@const by = $yScale(bv.value)}
      <circle cx={x} cy={by} r="3" fill={bv.color} />
    {/each}
  {/if}

  <foreignObject
    x={x + 10 > $width - tooltipWidth - 10 ? x - tooltipWidth - 10 : x + 10}
    y={Math.max(0, y - tooltipHeight / 2)}
    width={tooltipWidth}
    height={tooltipHeight}
  >
    <div class="tooltip-box">
      {#if comparisonMode}
        <div class="tooltip-line">
          <span class="tooltip-dot" style="background: var(--accent);"></span>
          AI Index: {tooltipData.value >= 0 ? '+' : ''}{tooltipData.value?.toFixed(2)}%
        </div>
        {#each benchValues as bv}
          <div class="tooltip-line">
            <span class="tooltip-dot" style:background={bv.color}></span>
            {bv.symbol}: {bv.value >= 0 ? '+' : ''}{bv.value.toFixed(2)}%
          </div>
        {/each}
      {:else}
        <div class="tooltip-value">{tooltipData.value?.toFixed(2)}</div>
      {/if}
      <div class="tooltip-date">{tooltipData.dateLabel || ''}</div>
    </div>
  </foreignObject>
{/if}

<style>
  .tooltip-box {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 11px;
    color: var(--text-primary);
  }

  .tooltip-value {
    font-family: var(--font-mono);
    font-weight: 600;
  }

  .tooltip-line {
    display: flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-mono);
    font-size: 10px;
    white-space: nowrap;
  }

  .tooltip-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .tooltip-date {
    color: var(--text-secondary);
    font-size: 10px;
    margin-top: 2px;
  }
</style>
