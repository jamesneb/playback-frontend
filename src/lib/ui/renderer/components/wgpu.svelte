<script lang="ts">
import { getContext, onMount } from 'svelte';

let getCanvas = getContext<() => HTMLCanvasElement>('canvas');


onMount(() => {
  const canvas = getCanvas();

});

</script>

