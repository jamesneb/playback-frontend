<script lang="ts">
	import { onMount } from 'svelte';
	import WasmDemo from '$lib/WasmDemo.svelte';

	let wasmSupported: boolean = false;

	onMount(async () => {
		// Test WASM support
		wasmSupported = typeof WebAssembly === 'object' && typeof WebAssembly.instantiate === 'function';
	});
</script>

<svelte:head>
	<title>Playback Frontend</title>
	<meta name="description" content="Playback telemetry visualization" />
</svelte:head>

<main>
	<h1>Welcome to Playback</h1>
	<p>
		{#if wasmSupported}
			✅ WebAssembly is supported
		{:else}
			❌ WebAssembly not supported
		{/if}
	</p>
	
	{#if wasmSupported}
		<WasmDemo />
	{/if}
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 800px;
		margin: 0 auto;
	}

	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>