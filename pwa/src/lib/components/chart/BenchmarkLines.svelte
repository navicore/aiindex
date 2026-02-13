<script>
  import { getContext } from 'svelte';

  let { series = {}, colors = {} } = $props();
  const { xScale, yScale } = getContext('LayerCake');

  let paths = $derived.by(() => {
    const result = [];
    for (const [sym, points] of Object.entries(series)) {
      if (!points || points.length === 0) continue;
      const d =
        'M' +
        points
          .map((p) => `${$xScale(p.date)},${$yScale(p.value)}`)
          .join('L');
      result.push({ symbol: sym, d, color: colors[sym] || '#888' });
    }
    return result;
  });
</script>

{#each paths as { symbol, d, color }}
  <path {d} fill="none" stroke={color} stroke-width="1.5" stroke-dasharray="6,3" opacity="0.8" />
{/each}
