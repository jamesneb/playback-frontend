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
	:global(body) {
		margin: 0;
		padding: 0;
		background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
		min-height: 100vh;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
	}

	main {
		text-align: center;
		padding: 2rem 1rem;
		max-width: 1200px;
		margin: 0 auto;
		min-height: 100vh;
	}

	h1 {
		color: #f1f5f9;
		font-size: 3rem;
		font-weight: 700;
		margin-bottom: 0.5rem;
		letter-spacing: -0.025em;
		background: linear-gradient(135deg, #f1f5f9 0%, #cbd5e1 100%);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
	}

	p {
		color: #94a3b8;
		font-size: 1.125rem;
		margin-bottom: 2rem;
		font-weight: 400;
	}

	@media (min-width: 768px) {
		main {
			padding: 3rem 2rem;
		}

		h1 {
			font-size: 4rem;
		}
	}

	@media (min-width: 1024px) {
		main {
			max-width: none;
			padding: 4rem 2rem;
		}
	}
</style>