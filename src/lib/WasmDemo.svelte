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
	<h2>🔥 FINAL BOSS MODE: WebGPU + WASM 🔥</h2>
	
	<div class="status">
		WASM: {wasmLoaded ? '✅ Loaded' : '⏳ Loading...'}
		| WebGPU: {webgpuInitialized ? '🔥 ACTIVATED' : '💤 Not initialized'}
	</div>

	<div class="canvas-container">
		<canvas bind:this={canvasElement} width="800" height="400"></canvas>
	</div>

	<div class="buttons">
		<button on:click={initWebGPU} disabled={!wasmLoaded || webgpuInitialized}>
			🚀 Initialize WebGPU
		</button>
		<button on:click={callHello} disabled={!webgpuInitialized}>
			🎨 Render "Hello WASM! 🦀"
		</button>
		<button on:click={callAdd} disabled={!webgpuInitialized}>
			🔢 Render "5 + 3 = 8"
		</button>
		<button on:click={callGreet} disabled={!webgpuInitialized}>
			👋 Render Greeting
		</button>
		<button on:click={clearOutput}>🧹 Clear Output</button>
	</div>

	<div class="output">
		<h3>Console Output:</h3>
		<pre>{output}</pre>
	</div>
</div>

<style>
	.wasm-demo {
		padding: 2rem;
		border: 3px solid #ff3e00;
		border-radius: 12px;
		margin: 1rem 0;
		background: linear-gradient(145deg, #1a1a1a, #2a2a2a);
		color: #fff;
		box-shadow: 0 8px 32px rgba(255, 62, 0, 0.3);
	}

	h2 {
		text-align: center;
		background: linear-gradient(45deg, #ff3e00, #ffd700);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
		font-size: 2rem;
		margin-bottom: 1rem;
		text-shadow: 0 0 20px rgba(255, 62, 0, 0.5);
	}

	.status {
		font-weight: bold;
		margin-bottom: 1rem;
		padding: 1rem;
		background: linear-gradient(135deg, #333, #444);
		border-radius: 8px;
		text-align: center;
		border: 1px solid #555;
	}

	.canvas-container {
		display: flex;
		justify-content: center;
		margin: 2rem 0;
	}

	canvas {
		border: 2px solid #ffd700;
		border-radius: 8px;
		background: #000;
		box-shadow: 0 0 30px rgba(255, 215, 0, 0.5);
	}

	.buttons {
		display: flex;
		gap: 0.75rem;
		margin-bottom: 1.5rem;
		flex-wrap: wrap;
		justify-content: center;
	}

	button {
		padding: 0.75rem 1.5rem;
		border: 2px solid #ff3e00;
		border-radius: 8px;
		background: linear-gradient(145deg, #ff3e00, #ff6600);
		color: white;
		cursor: pointer;
		transition: all 0.3s ease;
		font-weight: bold;
		text-shadow: 0 1px 2px rgba(0,0,0,0.8);
		box-shadow: 0 4px 15px rgba(255, 62, 0, 0.3);
	}

	button:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow: 0 6px 25px rgba(255, 62, 0, 0.5);
		background: linear-gradient(145deg, #ff6600, #ff8800);
	}

	button:disabled {
		opacity: 0.3;
		cursor: not-allowed;
		transform: none;
		box-shadow: none;
		background: #666;
		border-color: #666;
	}

	.output {
		border-top: 2px solid #555;
		padding-top: 1.5rem;
	}

	.output h3 {
		color: #ffd700;
		margin-bottom: 0.5rem;
	}

	pre {
		background-color: #1a1a1a;
		color: #00ff00;
		padding: 1.5rem;
		border-radius: 8px;
		overflow-x: auto;
		white-space: pre-wrap;
		min-height: 120px;
		border: 1px solid #333;
		font-family: 'Courier New', monospace;
		line-height: 1.4;
	}
</style>