<script lang="ts">
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<svelte:head>
  <title>Playback</title>
</svelte:head>

{@render children()}

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(html, body) {
    height: 100%;
    font-family: 'Satoshi', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #faf9f7;
    color: #1a1a1c;
  }

  :global(body) {
    overflow: hidden;
  }
</style>
