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
    const d = t instanceof Date ? t : new Date(t);
    if (!isNaN(d.getTime())) {
      // If data spans less than 2 days, show time; otherwise show date.
      const domain = $xScale.domain();
      const rangeMs = (domain[1] instanceof Date ? domain[1] : new Date(domain[1])) -
                      (domain[0] instanceof Date ? domain[0] : new Date(domain[0]));
      if (rangeMs < 2 * 86_400_000) {
        return d.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
      }
      return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
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
