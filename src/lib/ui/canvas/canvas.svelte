<script lang="ts">
let canvas: HTMLCanvasElement;

setContext('canvas', () => canvas);
</script>
<style>
canvas {
  height: 100%;
  width:  100%;
  display: block;
}
</style>
<canvas bind:this={canvas}></canvas>
