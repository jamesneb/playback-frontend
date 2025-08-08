<script lang="ts">
	import { onMount } from 'svelte';
	import ReplayViewer from '$lib/ReplayViewer.svelte';

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
		<ReplayViewer />
	{:else}
		<div class="error">
			<p>WebAssembly is required for telemetry visualization</p>
		</div>
	{/if}
</main>

<style>
	:global(body) {
		margin: 0;
		padding: 0;
		background: radial-gradient(circle at 20% 80%, rgba(120, 119, 198, 0.1) 0%, transparent 50%),
					radial-gradient(circle at 80% 20%, rgba(16, 185, 129, 0.1) 0%, transparent 50%),
					radial-gradient(circle at 40% 40%, rgba(99, 102, 241, 0.1) 0%, transparent 50%),
					linear-gradient(135deg, #0a0f1c 0%, #0f172a 40%, #1a1b3a 100%);
		min-height: 100vh;
		font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
		overflow-x: hidden;
	}

	:global(*) {
		box-sizing: border-box;
	}

	main {
		position: relative;
		padding: 1rem;
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: flex-start;
	}

	main::before {
		content: '';
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: 
			radial-gradient(600px circle at var(--mouse-x, 50%) var(--mouse-y, 50%), 
				rgba(99, 102, 241, 0.05) 0%, 
				transparent 40%);
		pointer-events: none;
		z-index: -1;
	}

	h1 {
		font-size: clamp(2.5rem, 5vw, 4.5rem);
		font-weight: 800;
		margin: 2rem 0 1rem 0;
		letter-spacing: -0.02em;
		text-align: center;
		background: linear-gradient(135deg, #ffffff 0%, #e2e8f0 30%, #cbd5e1 100%);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
		position: relative;
		z-index: 1;
	}

	h1::after {
		content: '';
		position: absolute;
		top: -10px;
		left: -10px;
		right: -10px;
		bottom: -10px;
		background: linear-gradient(135deg, rgba(99, 102, 241, 0.1), rgba(16, 185, 129, 0.1));
		border-radius: 20px;
		z-index: -1;
		opacity: 0.6;
		filter: blur(20px);
	}

	p {
		color: rgba(148, 163, 184, 0.8);
		font-size: 1rem;
		margin-bottom: 0.5rem;
		font-weight: 400;
		text-align: center;
		letter-spacing: 0.01em;
	}

	.error {
		background: rgba(239, 68, 68, 0.1);
		border: 1px solid rgba(239, 68, 68, 0.2);
		border-radius: 12px;
		padding: 2rem;
		margin-top: 2rem;
		backdrop-filter: blur(10px);
	}

	.error p {
		color: #fca5a5;
		margin: 0;
		font-weight: 500;
	}

	@media (min-width: 768px) {
		main {
			padding: 2rem;
		}
	}

	@media (min-width: 1200px) {
		main {
			max-width: 1400px;
			margin: 0 auto;
		}
	}
</style>