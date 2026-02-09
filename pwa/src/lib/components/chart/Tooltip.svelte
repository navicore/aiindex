<script>
  import { getContext } from 'svelte';

  const { data, xGet, yGet, xScale, yScale, width, height } = getContext('LayerCake');

  let mouseX = $state(null);
  let tooltipData = $state(null);

  function handleMousemove(e) {
    const rect = e.currentTarget.getBoundingClientRect();
    mouseX = e.clientX - rect.left;

    const d = $data;
    if (!d || d.length === 0) return;

    // Find closest point.
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

  <foreignObject
    x={x + 10 > $width - 120 ? x - 130 : x + 10}
    y={Math.max(0, y - 40)}
    width="120"
    height="50"
  >
    <div class="tooltip-box">
      <div class="tooltip-value">{tooltipData.value?.toFixed(2)}</div>
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
    font-size: 12px;
    color: var(--text-primary);
  }

  .tooltip-value {
    font-family: var(--font-mono);
    font-weight: 600;
  }

  .tooltip-date {
    color: var(--text-secondary);
    font-size: 10px;
  }
</style>
