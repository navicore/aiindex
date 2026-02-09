<script>
  import { getContext } from 'svelte';

  const { width, height, xScale, padding } = getContext('LayerCake');

  let ticks = $derived.by(() => {
    const scale = $xScale;
    if (scale.ticks) return scale.ticks(5);
    const domain = scale.domain();
    const step = Math.max(1, Math.floor(domain.length / 5));
    return domain.filter((_, i) => i % step === 0);
  });

  function formatTick(t) {
    if (t instanceof Date) {
      return t.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    }
    return String(t);
  }
</script>

<g class="axis x-axis" transform="translate(0, {$height})">
  {#each ticks as tick}
    <g transform="translate({$xScale(tick)}, 0)">
      <line y1="0" y2="6" stroke="var(--border)" />
      <text y="20" text-anchor="middle" fill="var(--text-secondary)" font-size="11">
        {formatTick(tick)}
      </text>
    </g>
  {/each}
</g>
