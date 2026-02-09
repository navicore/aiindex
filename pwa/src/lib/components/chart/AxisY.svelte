<script>
  import { getContext } from 'svelte';

  const { width, yScale, padding } = getContext('LayerCake');

  let ticks = $derived.by(() => {
    const scale = $yScale;
    if (scale.ticks) return scale.ticks(5);
    return [];
  });
</script>

<g class="axis y-axis">
  {#each ticks as tick}
    <g transform="translate(0, {$yScale(tick)})">
      <line x1="0" x2={$width} stroke="var(--border)" stroke-dasharray="2,4" />
      <text x="-8" text-anchor="end" dominant-baseline="middle" fill="var(--text-secondary)" font-size="11">
        {tick.toFixed(0)}
      </text>
    </g>
  {/each}
</g>
