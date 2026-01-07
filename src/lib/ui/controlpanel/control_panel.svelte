<script lang="ts">
  import {onMount} from 'svelte';
  import init, { compute_layout } from '$lib/wasm/playback_core';

  onMount(async () => {
    await init();
    const layout = compute_layout(1920, 1080);
    console.log(layout);
    // [tm.x, tm.y, tm.w, tm.h, tl.x, tl.y, tl.w, tl.h]
  })
</script>
