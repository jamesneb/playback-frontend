<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	let wasmModule: any = null;
	let wasmLoaded = false;
	let webgpuInitialized = false;
	let output = '';
	let canvasElement: HTMLCanvasElement;

	onMount(async () => {
		if (!browser) return;
		
		try {
			// Import the WASM module
			const wasm = await import('$lib/wasm-pkg/hello_wasm.js');
			await wasm.default();
			wasmModule = wasm;
			wasmLoaded = true;
			output += '✅ WASM module loaded successfully!\n';
		} catch (error) {
			console.error('Failed to load WASM module:', error);
			output += '❌ Failed to load WASM module: ' + error + '\n';
		}
	});

	async function initWebGPU() {
		if (!wasmModule || !canvasElement) return;
		
		try {
			// Check if WebGPU is supported first
			if (!navigator.gpu) {
				throw new Error('WebGPU not supported in this browser');
			}
			
			console.log('Canvas element:', canvasElement);
			console.log('Canvas size:', canvasElement.width, 'x', canvasElement.height);
			
			await wasmModule.init_webgpu(canvasElement);
			webgpuInitialized = true;
			output += '🔥 WebGPU FINAL BOSS MODE ACTIVATED!\n';
		} catch (error) {
			console.error('Failed to initialize WebGPU:', error);
			output += '❌ WebGPU initialization failed: ' + error + '\n';
		}
	}

	function callHello() {
		if (wasmModule && webgpuInitialized) {
			try {
				wasmModule.hello();
				output += '🎨 hello() rendered with WebGPU! (check canvas)\n';
			} catch (error) {
				output += '❌ hello() failed: ' + error + '\n';
			}
		} else {
			output += '⚠️ WebGPU not initialized yet!\n';
		}
	}

	function callAdd() {
		if (wasmModule && webgpuInitialized) {
			try {
				const result = wasmModule.add(5, 3);
				output += `🎨 add(5, 3) = ${result} rendered with WebGPU!\n`;
			} catch (error) {
				output += '❌ add() failed: ' + error + '\n';
			}
		} else {
			output += '⚠️ WebGPU not initialized yet!\n';
		}
	}

	function callGreet() {
		if (wasmModule && webgpuInitialized) {
			try {
				const greeting = wasmModule.greet('SvelteKit');
				output += `🎨 "${greeting}" rendered with WebGPU!\n`;
			} catch (error) {
				output += '❌ greet() failed: ' + error + '\n';
			}
		} else {
			output += '⚠️ WebGPU not initialized yet!\n';
		}
	}

	function clearOutput() {
		output = '';
	}
</script>

<div class="wasm-demo">
	<h2>WebGPU Rendering Engine</h2>
	
	<div class="status">
		WASM: {wasmLoaded ? 'Ready' : 'Loading...'}
		| WebGPU: {webgpuInitialized ? 'Initialized' : 'Standby'}
	</div>

	<div class="canvas-container">
		<canvas bind:this={canvasElement} width="800" height="400"></canvas>
	</div>

	<div class="buttons">
		<button on:click={initWebGPU} disabled={!wasmLoaded || webgpuInitialized}>
			Initialize Renderer
		</button>
		<button on:click={callHello} disabled={!webgpuInitialized}>
			Render Text Sample
		</button>
		<button on:click={callAdd} disabled={!webgpuInitialized}>
			Render Mathematics
		</button>
		<button on:click={callGreet} disabled={!webgpuInitialized}>
			Render Message
		</button>
		<button on:click={clearOutput}>Clear Console</button>
	</div>

	<div class="output">
		<h3>System Log</h3>
		<pre>{output}</pre>
	</div>
</div>

<style>
	.wasm-demo {
		padding: 2rem;
		border: 1px solid rgba(99, 102, 241, 0.2);
		border-radius: 12px;
		margin: 1rem 0;
		background: linear-gradient(135deg, #0f0f23 0%, #1a1b3e 100%);
		color: #e2e8f0;
		box-shadow: 
			0 4px 6px -1px rgba(0, 0, 0, 0.1),
			0 2px 4px -1px rgba(0, 0, 0, 0.06),
			0 0 0 1px rgba(99, 102, 241, 0.05);
		backdrop-filter: blur(8px);
	}

	h2 {
		text-align: center;
		color: #f8fafc;
		font-size: 1.5rem;
		font-weight: 600;
		margin-bottom: 1.5rem;
		letter-spacing: -0.025em;
		position: relative;
	}

	h2::after {
		content: '';
		position: absolute;
		bottom: -8px;
		left: 50%;
		transform: translateX(-50%);
		width: 60px;
		height: 2px;
		background: linear-gradient(90deg, #6366f1, #8b5cf6);
		border-radius: 1px;
	}

	.status {
		font-weight: 500;
		margin-bottom: 1.5rem;
		padding: 0.75rem 1rem;
		background: rgba(30, 41, 59, 0.5);
		border-radius: 6px;
		text-align: center;
		border: 1px solid rgba(51, 65, 85, 0.3);
		font-size: 0.875rem;
		color: #cbd5e1;
	}

	.canvas-container {
		display: flex;
		justify-content: center;
		margin: 2rem 0;
	}

	canvas {
		border: 1px solid rgba(71, 85, 105, 0.4);
		border-radius: 8px;
		background: #000011;
		box-shadow: 
			0 10px 15px -3px rgba(0, 0, 0, 0.1),
			0 4px 6px -2px rgba(0, 0, 0, 0.05),
			inset 0 1px 0 rgba(99, 102, 241, 0.1);
	}

	.buttons {
		display: flex;
		gap: 0.75rem;
		margin-bottom: 2rem;
		flex-wrap: wrap;
		justify-content: center;
	}

	button {
		padding: 0.625rem 1.25rem;
		border: 1px solid transparent;
		border-radius: 6px;
		background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
		color: #ffffff;
		cursor: pointer;
		transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
		font-weight: 500;
		font-size: 0.875rem;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
		position: relative;
		overflow: hidden;
	}

	button::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: linear-gradient(135deg, #7c3aed 0%, #a855f7 100%);
		opacity: 0;
		transition: opacity 0.2s ease;
		z-index: -1;
	}

	button:hover:not(:disabled)::before {
		opacity: 1;
	}

	button:hover:not(:disabled) {
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
	}

	button:active:not(:disabled) {
		transform: translateY(0);
		box-shadow: 0 1px 2px rgba(99, 102, 241, 0.4);
	}

	button:disabled {
		opacity: 0.4;
		cursor: not-allowed;
		transform: none;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
		background: #475569;
	}

	.output {
		border-top: 1px solid rgba(51, 65, 85, 0.3);
		padding-top: 1.5rem;
		margin-top: 1rem;
	}

	.output h3 {
		color: #f1f5f9;
		margin-bottom: 0.75rem;
		font-size: 0.875rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		opacity: 0.9;
	}

	pre {
		background: rgba(15, 23, 42, 0.8);
		color: #10b981;
		padding: 1rem;
		border-radius: 8px;
		overflow-x: auto;
		white-space: pre-wrap;
		min-height: 100px;
		border: 1px solid rgba(30, 41, 59, 0.5);
		font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', Consolas, 'Liberation Mono', Menlo, monospace;
		font-size: 0.8125rem;
		line-height: 1.5;
		box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	/* Responsive design */
	@media (max-width: 768px) {
		.wasm-demo {
			padding: 1.5rem;
			margin: 0.5rem 0;
		}

		.buttons {
			gap: 0.5rem;
		}

		button {
			padding: 0.5rem 1rem;
			font-size: 0.8125rem;
		}

		canvas {
			width: 100%;
			max-width: 400px;
			height: auto;
		}
	}
</style>