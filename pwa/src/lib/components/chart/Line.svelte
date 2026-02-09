<script>
  import { getContext } from 'svelte';

  const { data, xGet, yGet } = getContext('LayerCake');

  let path = $derived.by(() => {
    const d = $data;
    if (!d || d.length === 0) return '';
    return 'M' + d.map((p) => `${$xGet(p)},${$yGet(p)}`).join('L');
  });
</script>

<path d={path} fill="none" stroke="var(--accent)" stroke-width="2" />
