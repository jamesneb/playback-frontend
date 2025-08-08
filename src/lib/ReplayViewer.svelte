<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { s3Service } from '$lib/s3Service';
	import { wasmWorkerService } from '$lib/wasmWorkerService';

	// No longer need direct wasmModule - using worker service
	let status: 'loading' | 'initializing' | 'downloading' | 'rendering' | 'ready' | 'error' = 'loading';
	let statusMessage = 'Loading WASM module...';
	let canvasElement: HTMLCanvasElement;
	let replayFiles: Array<{ key: string; jobId?: string; size: number; lastModified: string }> = [];
	let activeServices: string[] = [];
	let errorDetails = '';
	let processedFiles = new Map<string, string>(); // Track file key -> lastModified to prevent duplicate processing

	onMount(async () => {
		if (!browser) return;
		await initializeViewer();
	});

	async function initializeViewer() {
		try {
			// Initialize WASM Worker with OffscreenCanvas  
			status = 'loading';
			statusMessage = 'Loading WebAssembly worker...';
			
			status = 'initializing';
			statusMessage = 'Initializing WebGPU renderer in worker...';
			
			if (!navigator.gpu) {
				throw new Error('WebGPU not supported in this browser');
			}
			
			// Wait for canvas to be ready
			await new Promise(resolve => setTimeout(resolve, 100));
			
			// Initialize worker with canvas (transfers control to worker)
			await wasmWorkerService.initialize(canvasElement);

			// Download and render all replay files
			await loadAndRenderAllReplays();

		} catch (error) {
			console.error('Failed to initialize viewer:', error);
			status = 'error';
			statusMessage = 'Failed to initialize telemetry viewer';
			errorDetails = error instanceof Error ? error.message : String(error);
		}
	}

	async function loadAndRenderAllReplays() {
		try {
			// Only show downloading status on first load, not during polls
			if (status !== 'ready') {
				status = 'downloading';
				statusMessage = 'Discovering replay files...';
			}
			
			const files = await s3Service.listReplayFiles();
			replayFiles = files || [];
			
			if (!files || files.length === 0) {
				status = 'ready';
				statusMessage = 'No replay files found. Waiting for telemetry data...';
				activeServices = [];
				// Worker will handle canvas clearing automatically
				return;
			}

			// Only show detailed messages on first load
			if (status !== 'ready') {
				statusMessage = `Found ${files.length} replay file${files.length === 1 ? '' : 's'}. Downloading...`;
			}
			
			// Canvas clearing is handled in worker automatically
			
			// Process all replay files
			const allServices: string[] = [];
			let totalDataSize = 0;
			
			for (const file of files) {
				try {
					// Check if we've already processed this version of the file
					const lastProcessedVersion = processedFiles.get(file.key);
					if (lastProcessedVersion === file.lastModified) {
						console.log(`⏭️ Skipping unchanged file: ${file.key} (${file.lastModified})`);
						continue;
					}
					
					const arrowData = await s3Service.downloadReplayFile(file.key);
					totalDataSize += arrowData.byteLength;
					
					// Skip empty files
					if (arrowData.byteLength === 0) {
						console.log(`⏭️ Skipping empty file: ${file.key}`);
						continue;
					}
					
					// Mark file as processed
					processedFiles.set(file.key, file.lastModified);
					
					// Create copies for both worker and local use to avoid detached buffer issues
					const workerData = new Uint8Array(arrowData.slice());
					const localData = new Uint8Array(arrowData.slice());
					
					// Send data to worker 
					wasmWorkerService.appendData(workerData);
					
					// Debug logging
					console.log(`📦 File: ${file.key}, Size: ${arrowData.byteLength} bytes, LastModified: ${file.lastModified}`);
					console.log('✅ Data sent to worker via append_chunk');
					
					// Extract service IDs locally for UI
					try {
						const tempWasm = await import('$lib/wasm-pkg/hello_wasm.js');
						await tempWasm.default();
						const replayInfo = await tempWasm.parse_arrow_replay(localData);
						if (replayInfo.services) {
							allServices.push(...replayInfo.services);
							console.log(`🎯 Services from ${file.key}:`, replayInfo.services);
						}
					} catch (tempError) {
						console.warn('Failed to parse services locally for UI:', tempError);
					}
				} catch (fileError) {
					console.warn(`Failed to process file ${file.key}:`, fileError);
				}
			}

			// Update active services list (remove duplicates)
			const newActiveServices = [...new Set(allServices)];
			const servicesChanged = JSON.stringify([...activeServices].sort()) !== JSON.stringify([...newActiveServices].sort());
			activeServices = newActiveServices;

			// Worker now handles animation automatically when services are available
			console.log('✅ Data sent to worker - animation handled independently by worker');
			
			status = 'ready';
			
			// Show real-time update info with file timestamp
			const currentTime = new Date().toLocaleTimeString();
			const latestFile = files.length > 0 ? files[files.length - 1] : null;
			const fileTime = latestFile ? new Date(latestFile.lastModified).toLocaleTimeString() : 'N/A';
			
			if (servicesChanged) {
				statusMessage = `🔄 Updated at ${currentTime} - ${activeServices.length} services cycling (file: ${fileTime})`;
			} else {
				statusMessage = `✨ Live at ${currentTime} - ${activeServices.length} services cycling (file: ${fileTime})`;
			}
			
			console.log('All services found:', allServices);
			console.log('Unique services:', activeServices);
			console.log('Files:', files.map(f => ({ key: f.key, modified: f.lastModified, size: f.size })));

		} catch (error) {
			console.error('Failed to load and render replays:', error);
			status = 'error';
			statusMessage = 'Failed to load replay data';
			errorDetails = error instanceof Error ? error.message : String(error);
		}
	}

	// Continuous polling for real-time updates
	let refreshInterval: NodeJS.Timeout;
	let isPolling = false;
	
	onMount(() => {
		if (browser) {
			// Fast polling for real-time updates - WASM animation timing is now protected
			refreshInterval = setInterval(async () => {
				if ((status === 'ready' || status === 'rendering') && !isPolling) {
					isPolling = true;
					try {
						await loadAndRenderAllReplays();
					} finally {
						isPolling = false;
					}
				}
			}, 100); // Poll every 100ms for real-time data updates
		}
		
		return () => {
			if (refreshInterval) {
				clearInterval(refreshInterval);
			}
			// Terminate worker on component cleanup
			wasmWorkerService.terminate();
		};
	});

	$: statusColor = {
		loading: '#3b82f6',
		initializing: '#6366f1', 
		downloading: '#8b5cf6',
		rendering: '#a855f7',
		ready: '#10b981',
		error: '#ef4444'
	}[status];

	$: statusIcon = {
		loading: '⚡',
		initializing: '🔧',
		downloading: '📡',
		rendering: '🎨',
		ready: '✨',
		error: '❌'
	}[status];
</script>

<div class="telemetry-viewer">
	<!-- Animated background elements -->
	<div class="bg-mesh"></div>
	<div class="bg-particles"></div>
	<div class="bg-glow"></div>
	
	<div class="viewer-header">
		<div class="title-section">
			<div class="title-badge">
				<div class="badge-glow"></div>
				<span class="badge-icon">⚡</span>
				<span>LIVE TELEMETRY</span>
			</div>
			<h1 class="main-title">
				<span class="title-gradient">Service Topology</span>
				<div class="title-underline"></div>
			</h1>
			<div class="subtitle">
				<span class="subtitle-accent">Real-time</span> distributed systems visualization
			</div>
		</div>
		
		<div class="status-panel">
			<div class="status-card" style="--status-color: {statusColor}">
				<div class="status-glow"></div>
				<div class="status-content">
					<div class="status-main">
						<div class="status-icon-wrapper">
							<span class="status-icon">{statusIcon}</span>
							<div class="icon-pulse"></div>
						</div>
						<div class="status-info">
							<div class="status-text">{statusMessage}</div>
							{#if status === 'ready'}
								<div class="live-badge">
									<div class="live-pulse"></div>
									<span class="live-text">STREAMING</span>
									<div class="live-bars">
										<div class="bar"></div>
										<div class="bar"></div>
										<div class="bar"></div>
									</div>
								</div>
							{/if}
						</div>
					</div>
				</div>
			</div>
			
			{#if status === 'ready' && activeServices.length > 0}
				<div class="metrics-dashboard">
					<div class="metric-card">
						<div class="metric-glow"></div>
						<div class="metric-icon">🔗</div>
						<div class="metric-value">{activeServices.length}</div>
						<div class="metric-label">Active Services</div>
						<div class="metric-trend">+{Math.max(0, activeServices.length - 5)}</div>
					</div>
					<div class="metric-card">
						<div class="metric-glow"></div>
						<div class="metric-icon">📊</div>
						<div class="metric-value">{replayFiles.length}</div>
						<div class="metric-label">Data Streams</div>
						<div class="metric-trend">Real-time</div>
					</div>
					<div class="metric-card">
						<div class="metric-glow"></div>
						<div class="metric-icon">💾</div>
						<div class="metric-value">{Math.round(replayFiles.reduce((sum, f) => sum + f.size, 0) / 1024)}</div>
						<div class="metric-label">KB Processed</div>
						<div class="metric-trend">↗ Live</div>
					</div>
				</div>
			{/if}
		</div>
	</div>

	<div class="canvas-stage">
		<div class="canvas-frame">
			<canvas bind:this={canvasElement} width="1600" height="900"></canvas>
			
			{#if status === 'error'}
				<div class="error-overlay">
					<div class="error-content">
						<div class="error-icon">⚠️</div>
						<h3>Visualization Error</h3>
						<p>{statusMessage}</p>
						{#if errorDetails}
							<details>
								<summary>Technical Details</summary>
								<pre>{errorDetails}</pre>
							</details>
						{/if}
						<button on:click={initializeViewer}>Retry</button>
					</div>
				</div>
			{/if}
			
			{#if status === 'loading' || status === 'initializing' || status === 'downloading' || status === 'rendering'}
				<div class="loading-overlay">
					<div class="loading-spinner">
						<div class="spinner-ring"></div>
						<div class="spinner-ring"></div>
						<div class="spinner-ring"></div>
					</div>
					<p>{statusMessage}</p>
				</div>
			{/if}
		</div>
		
		{#if status === 'ready' && activeServices.length === 0}
			<div class="empty-state">
				<div class="empty-icon">🔍</div>
				<h3>Waiting for Telemetry</h3>
				<p>No service topology data available yet. The viewer will automatically refresh when new replay files are generated.</p>
			</div>
		{/if}
	</div>

	{#if status === 'ready' && activeServices.length > 0}
		<div class="service-panel">
			<h4>Active Services</h4>
			<div class="service-grid">
				{#each activeServices as service}
					<div class="service-node">
						<div class="service-indicator"></div>
						<span>{service}</span>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>

<style>
	.telemetry-viewer {
		position: relative;
		width: 100%;
		max-width: 1920px;
		margin: 0 auto;
		min-height: 100vh;
		background: 
			radial-gradient(ellipse at 20% 50%, rgba(120, 119, 198, 0.1) 0%, transparent 50%),
			radial-gradient(ellipse at 80% 20%, rgba(255, 119, 198, 0.08) 0%, transparent 50%),
			radial-gradient(ellipse at 40% 80%, rgba(16, 185, 129, 0.06) 0%, transparent 50%),
			linear-gradient(135deg, #0a0a23 0%, #1a1a2e 25%, #16213e 50%, #0f172a 100%);
		overflow: hidden;
	}

	/* Animated background elements */
	.bg-mesh {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: 
			linear-gradient(90deg, transparent 24%, rgba(99, 102, 241, 0.03) 25%, rgba(99, 102, 241, 0.03) 26%, transparent 27%, transparent 74%, rgba(99, 102, 241, 0.03) 75%, rgba(99, 102, 241, 0.03) 76%, transparent 77%),
			linear-gradient(0deg, transparent 24%, rgba(16, 185, 129, 0.02) 25%, rgba(16, 185, 129, 0.02) 26%, transparent 27%, transparent 74%, rgba(16, 185, 129, 0.02) 75%, rgba(16, 185, 129, 0.02) 76%, transparent 77%);
		background-size: 50px 50px;
		animation: mesh-move 20s ease-in-out infinite;
		pointer-events: none;
	}

	.bg-particles {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-image: 
			radial-gradient(2px 2px at 20px 30px, rgba(99, 102, 241, 0.4), transparent),
			radial-gradient(2px 2px at 40px 70px, rgba(16, 185, 129, 0.3), transparent),
			radial-gradient(1px 1px at 90px 40px, rgba(255, 119, 198, 0.4), transparent),
			radial-gradient(1px 1px at 130px 80px, rgba(99, 102, 241, 0.3), transparent),
			radial-gradient(2px 2px at 160px 30px, rgba(16, 185, 129, 0.2), transparent);
		background-repeat: repeat;
		background-size: 200px 100px;
		animation: particles-float 15s ease-in-out infinite;
		pointer-events: none;
	}

	.bg-glow {
		position: absolute;
		top: -50%;
		left: -50%;
		width: 200%;
		height: 200%;
		background: 
			radial-gradient(ellipse at 25% 25%, rgba(99, 102, 241, 0.15) 0%, transparent 50%),
			radial-gradient(ellipse at 75% 75%, rgba(16, 185, 129, 0.1) 0%, transparent 50%);
		animation: glow-rotate 30s linear infinite;
		pointer-events: none;
	}

	@keyframes mesh-move {
		0%, 100% { transform: translate(0, 0); }
		50% { transform: translate(-5px, -5px); }
	}

	@keyframes particles-float {
		0%, 100% { transform: translateY(0px) rotate(0deg); }
		33% { transform: translateY(-10px) rotate(120deg); }
		66% { transform: translateY(-5px) rotate(240deg); }
	}

	@keyframes glow-rotate {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.viewer-header {
		position: relative;
		z-index: 10;
		display: flex;
		flex-direction: column;
		gap: 3rem;
		padding: 4rem 3rem 3rem;
		background: 
			linear-gradient(135deg, rgba(15, 23, 42, 0.95) 0%, rgba(30, 41, 59, 0.95) 100%);
		backdrop-filter: blur(40px);
		border-bottom: 1px solid rgba(99, 102, 241, 0.2);
		border-radius: 0 0 40px 40px;
		box-shadow: 
			0 20px 40px rgba(0, 0, 0, 0.3),
			inset 0 1px 0 rgba(255, 255, 255, 0.1);
	}

	.title-section {
		text-align: center;
		position: relative;
	}

	.title-badge {
		position: relative;
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1.25rem;
		margin-bottom: 1.5rem;
		background: rgba(99, 102, 241, 0.1);
		border: 1px solid rgba(99, 102, 241, 0.3);
		border-radius: 50px;
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.1em;
		color: #a5b4fc;
		text-transform: uppercase;
		backdrop-filter: blur(10px);
		transition: all 0.3s ease;
	}

	.title-badge:hover {
		transform: translateY(-2px);
		box-shadow: 0 10px 25px rgba(99, 102, 241, 0.2);
		border-color: rgba(99, 102, 241, 0.5);
	}

	.badge-glow {
		position: absolute;
		inset: -2px;
		background: linear-gradient(45deg, rgba(99, 102, 241, 0.3), rgba(16, 185, 129, 0.3));
		border-radius: 50px;
		opacity: 0;
		transition: opacity 0.3s ease;
		z-index: -1;
		filter: blur(8px);
	}

	.title-badge:hover .badge-glow {
		opacity: 1;
	}

	.badge-icon {
		font-size: 1rem;
		animation: pulse-glow 2s ease-in-out infinite;
	}

	@keyframes pulse-glow {
		0%, 100% { transform: scale(1); }
		50% { transform: scale(1.1); filter: brightness(1.2); }
	}

	.main-title {
		position: relative;
		margin: 0;
		font-size: clamp(3rem, 8vw, 5rem);
		font-weight: 900;
		letter-spacing: -0.05em;
		line-height: 0.9;
		text-align: center;
	}

	.title-gradient {
		background: linear-gradient(135deg, 
			#ffffff 0%, 
			#e2e8f0 25%, 
			#cbd5e1 50%, 
			#94a3b8 100%);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
		filter: drop-shadow(0 4px 8px rgba(99, 102, 241, 0.3));
		animation: title-shimmer 3s ease-in-out infinite;
	}

	@keyframes title-shimmer {
		0%, 100% { filter: drop-shadow(0 4px 8px rgba(99, 102, 241, 0.3)) brightness(1); }
		50% { filter: drop-shadow(0 6px 12px rgba(16, 185, 129, 0.3)) brightness(1.1); }
	}

	.title-underline {
		height: 6px;
		width: 120px;
		margin: 1rem auto;
		background: linear-gradient(90deg, 
			rgba(99, 102, 241, 0.8) 0%, 
			rgba(16, 185, 129, 0.8) 100%);
		border-radius: 3px;
		position: relative;
		overflow: hidden;
	}

	.title-underline::after {
		content: '';
		position: absolute;
		top: 0;
		left: -100%;
		width: 100%;
		height: 100%;
		background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.4), transparent);
		animation: shine 2s ease-in-out infinite;
	}

	@keyframes shine {
		0% { left: -100%; }
		100% { left: 100%; }
	}

	.subtitle {
		font-size: 1.25rem;
		font-weight: 400;
		color: rgba(148, 163, 184, 0.9);
		letter-spacing: 0.025em;
		line-height: 1.5;
	}

	.subtitle-accent {
		color: #10b981;
		font-weight: 600;
		position: relative;
	}

	.subtitle-accent::after {
		content: '';
		position: absolute;
		bottom: -2px;
		left: 0;
		width: 100%;
		height: 2px;
		background: linear-gradient(90deg, #10b981, transparent);
		animation: accent-pulse 2s ease-in-out infinite;
	}

	@keyframes accent-pulse {
		0%, 100% { opacity: 0.5; transform: scaleX(0.8); }
		50% { opacity: 1; transform: scaleX(1); }
	}

	.status-panel {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2rem;
	}

	.status-card {
		position: relative;
		width: 100%;
		max-width: 600px;
		background: rgba(15, 23, 42, 0.8);
		border: 1px solid rgba(99, 102, 241, 0.3);
		border-radius: 20px;
		backdrop-filter: blur(20px);
		overflow: hidden;
		transition: all 0.3s ease;
	}

	.status-card:hover {
		transform: translateY(-4px);
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
		border-color: var(--status-color, rgba(99, 102, 241, 0.5));
	}

	.status-glow {
		position: absolute;
		inset: 0;
		background: linear-gradient(135deg, 
			var(--status-color, rgba(99, 102, 241, 0.1)), 
			transparent 50%, 
			var(--status-color, rgba(99, 102, 241, 0.05)));
		opacity: 0.5;
		transition: opacity 0.3s ease;
	}

	.status-card:hover .status-glow {
		opacity: 0.8;
	}

	.status-content {
		position: relative;
		z-index: 2;
		padding: 1.5rem;
	}

	.status-main {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.status-icon-wrapper {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 60px;
		height: 60px;
		background: rgba(99, 102, 241, 0.1);
		border: 2px solid rgba(99, 102, 241, 0.3);
		border-radius: 50%;
		backdrop-filter: blur(10px);
	}

	.status-icon {
		font-size: 1.5rem;
		animation: status-pulse 2s ease-in-out infinite;
	}

	.icon-pulse {
		position: absolute;
		inset: -4px;
		border: 2px solid var(--status-color, rgba(99, 102, 241, 0.4));
		border-radius: 50%;
		animation: icon-ring-pulse 2s ease-in-out infinite;
	}

	@keyframes status-pulse {
		0%, 100% { transform: scale(1); }
		50% { transform: scale(1.1); }
	}

	@keyframes icon-ring-pulse {
		0% { transform: scale(1); opacity: 1; }
		100% { transform: scale(1.5); opacity: 0; }
	}

	.status-info {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.status-text {
		color: #f1f5f9;
		font-weight: 500;
		font-size: 1.125rem;
		line-height: 1.4;
	}

	.live-badge {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.5rem 1rem;
		background: rgba(16, 185, 129, 0.15);
		border: 1px solid rgba(16, 185, 129, 0.4);
		border-radius: 50px;
		backdrop-filter: blur(10px);
	}

	.live-pulse {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #10b981;
		box-shadow: 0 0 10px rgba(16, 185, 129, 1);
		animation: live-pulse 1.5s ease-in-out infinite;
	}

	.live-text {
		font-size: 0.75rem;
		font-weight: 700;
		color: #10b981;
		letter-spacing: 0.1em;
		text-transform: uppercase;
	}

	.live-bars {
		display: flex;
		gap: 2px;
		height: 12px;
		align-items: flex-end;
	}

	.live-bars .bar {
		width: 3px;
		background: #10b981;
		border-radius: 2px;
		animation: bar-dance 1.2s ease-in-out infinite;
	}

	.live-bars .bar:nth-child(2) {
		animation-delay: 0.1s;
	}

	.live-bars .bar:nth-child(3) {
		animation-delay: 0.2s;
	}

	@keyframes live-pulse {
		0%, 100% { 
			opacity: 1; 
			transform: scale(1);
			box-shadow: 0 0 10px rgba(16, 185, 129, 1);
		}
		50% { 
			opacity: 0.7;
			transform: scale(1.3);
			box-shadow: 0 0 20px rgba(16, 185, 129, 0.6);
		}
	}

	@keyframes bar-dance {
		0%, 100% { height: 6px; }
		50% { height: 12px; }
	}

	.status-icon {
		font-size: 1.125rem;
		animation: pulse 2s infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.7; }
	}

	.metrics-dashboard {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1.5rem;
		width: 100%;
		max-width: 800px;
	}

	.metric-card {
		position: relative;
		padding: 2rem 1.5rem;
		background: rgba(15, 23, 42, 0.6);
		border: 1px solid rgba(99, 102, 241, 0.2);
		border-radius: 16px;
		backdrop-filter: blur(20px);
		text-align: center;
		transition: all 0.3s ease;
		overflow: hidden;
	}

	.metric-card:hover {
		transform: translateY(-4px) scale(1.02);
		border-color: rgba(99, 102, 241, 0.4);
		box-shadow: 0 15px 30px rgba(0, 0, 0, 0.2);
	}

	.metric-glow {
		position: absolute;
		inset: 0;
		background: linear-gradient(135deg, 
			rgba(99, 102, 241, 0.05), 
			rgba(16, 185, 129, 0.05), 
			rgba(255, 119, 198, 0.05));
		opacity: 0;
		transition: opacity 0.3s ease;
	}

	.metric-card:hover .metric-glow {
		opacity: 1;
	}

	.metric-icon {
		font-size: 2rem;
		margin-bottom: 1rem;
		animation: metric-float 3s ease-in-out infinite;
	}

	.metric-card:nth-child(2) .metric-icon {
		animation-delay: 0.5s;
	}

	.metric-card:nth-child(3) .metric-icon {
		animation-delay: 1s;
	}

	@keyframes metric-float {
		0%, 100% { transform: translateY(0px); }
		50% { transform: translateY(-4px); }
	}

	.metric-value {
		font-size: 2.5rem;
		font-weight: 900;
		color: #ffffff;
		line-height: 1;
		margin-bottom: 0.5rem;
		background: linear-gradient(135deg, #ffffff, #e2e8f0);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
		animation: value-glow 2s ease-in-out infinite;
	}

	@keyframes value-glow {
		0%, 100% { filter: brightness(1); }
		50% { filter: brightness(1.2); }
	}

	.metric-label {
		font-size: 0.875rem;
		color: rgba(148, 163, 184, 0.9);
		font-weight: 600;
		letter-spacing: 0.025em;
		margin-bottom: 0.5rem;
		text-transform: uppercase;
	}

	.metric-trend {
		font-size: 0.75rem;
		font-weight: 600;
		color: #10b981;
		padding: 0.25rem 0.5rem;
		background: rgba(16, 185, 129, 0.1);
		border: 1px solid rgba(16, 185, 129, 0.3);
		border-radius: 20px;
		display: inline-block;
		letter-spacing: 0.025em;
	}

	.canvas-stage {
		position: relative;
		z-index: 5;
		padding: 3rem 2rem;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
	}

	.canvas-frame {
		position: relative;
		border-radius: 20px;
		overflow: hidden;
		background: 
			radial-gradient(ellipse at center, rgba(99, 102, 241, 0.05) 0%, transparent 50%),
			linear-gradient(135deg, #000510 0%, #001122 50%, #000a1a 100%);
		box-shadow: 
			0 30px 60px rgba(0, 0, 0, 0.4),
			0 0 0 1px rgba(99, 102, 241, 0.2),
			inset 0 1px 0 rgba(255, 255, 255, 0.1);
		border: 2px solid rgba(99, 102, 241, 0.3);
		transition: all 0.3s ease;
	}

	.canvas-frame:hover {
		transform: translateY(-2px);
		box-shadow: 
			0 40px 80px rgba(0, 0, 0, 0.5),
			0 0 0 1px rgba(99, 102, 241, 0.4),
			inset 0 1px 0 rgba(255, 255, 255, 0.15);
		border-color: rgba(99, 102, 241, 0.5);
	}

	.canvas-frame::before {
		content: '';
		position: absolute;
		inset: -2px;
		background: linear-gradient(45deg, 
			rgba(99, 102, 241, 0.3), 
			rgba(16, 185, 129, 0.3), 
			rgba(255, 119, 198, 0.2));
		border-radius: 22px;
		opacity: 0;
		transition: opacity 0.3s ease;
		z-index: -1;
		filter: blur(10px);
	}

	.canvas-frame:hover::before {
		opacity: 0.6;
	}

	canvas {
		display: block;
		max-width: 100%;
		height: auto;
		aspect-ratio: 16 / 9;
	}

	.loading-overlay, .error-overlay {
		position: absolute;
		inset: 0;
		background: rgba(0, 5, 16, 0.9);
		backdrop-filter: blur(8px);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		border-radius: 16px;
	}

	.loading-spinner {
		position: relative;
		width: 60px;
		height: 60px;
		margin-bottom: 1.5rem;
	}

	.spinner-ring {
		position: absolute;
		width: 100%;
		height: 100%;
		border: 2px solid transparent;
		border-radius: 50%;
		border-top: 2px solid #6366f1;
		animation: spin 1.5s linear infinite;
	}

	.spinner-ring:nth-child(2) {
		animation-delay: -0.5s;
		border-top-color: #8b5cf6;
		width: 75%;
		height: 75%;
		top: 12.5%;
		left: 12.5%;
	}

	.spinner-ring:nth-child(3) {
		animation-delay: -1s;
		border-top-color: #10b981;
		width: 50%;
		height: 50%;
		top: 25%;
		left: 25%;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.loading-overlay p, .error-overlay p {
		color: #e2e8f0;
		font-weight: 500;
		text-align: center;
		max-width: 300px;
		line-height: 1.5;
	}

	.error-overlay {
		text-align: center;
	}

	.error-content {
		max-width: 400px;
		padding: 2rem;
	}

	.error-icon {
		font-size: 3rem;
		margin-bottom: 1rem;
	}

	.error-overlay h3 {
		color: #fca5a5;
		margin: 0 0 1rem 0;
		font-size: 1.25rem;
		font-weight: 600;
	}

	.error-overlay details {
		margin-top: 1rem;
		text-align: left;
	}

	.error-overlay summary {
		cursor: pointer;
		color: #cbd5e1;
		font-size: 0.875rem;
		margin-bottom: 0.5rem;
	}

	.error-overlay pre {
		background: rgba(0, 0, 0, 0.3);
		padding: 1rem;
		border-radius: 8px;
		font-size: 0.75rem;
		color: #fca5a5;
		overflow-x: auto;
		white-space: pre-wrap;
	}

	.error-overlay button {
		background: linear-gradient(135deg, #6366f1, #8b5cf6);
		color: white;
		border: none;
		border-radius: 8px;
		padding: 0.75rem 1.5rem;
		font-weight: 500;
		cursor: pointer;
		margin-top: 1.5rem;
		transition: all 0.2s;
	}

	.error-overlay button:hover {
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
	}

	.empty-state {
		text-align: center;
		padding: 3rem;
		color: rgba(148, 163, 184, 0.8);
	}

	.empty-icon {
		font-size: 4rem;
		margin-bottom: 1rem;
		opacity: 0.6;
	}

	.empty-state h3 {
		color: #e2e8f0;
		margin: 0 0 1rem 0;
		font-size: 1.25rem;
		font-weight: 600;
	}

	.empty-state p {
		max-width: 500px;
		margin: 0 auto;
		line-height: 1.6;
	}

	.service-panel {
		padding: 1.5rem 2.5rem 2.5rem;
		border-top: 1px solid rgba(51, 65, 85, 0.2);
		background: rgba(30, 41, 59, 0.2);
	}

	.service-panel h4 {
		color: #f1f5f9;
		margin: 0 0 1rem 0;
		font-size: 0.875rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		opacity: 0.9;
	}

	.service-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 0.75rem;
	}

	.service-node {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		background: rgba(15, 23, 42, 0.6);
		border: 1px solid rgba(99, 102, 241, 0.2);
		border-radius: 8px;
		color: #e2e8f0;
		font-size: 0.875rem;
		font-weight: 500;
		transition: all 0.2s;
	}

	.service-node:hover {
		border-color: rgba(99, 102, 241, 0.4);
		background: rgba(99, 102, 241, 0.05);
	}

	.service-indicator {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #10b981;
		box-shadow: 0 0 8px rgba(16, 185, 129, 0.5);
		animation: pulse-dot 2s infinite;
	}

	@keyframes pulse-dot {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.6; }
	}

	/* Responsive design */
	@media (max-width: 1200px) {
		.telemetry-viewer {
			margin: 1rem;
			border-radius: 16px;
		}

		.canvas-frame canvas {
			width: 100%;
		}
	}

	@media (max-width: 768px) {
		.viewer-header {
			padding: 1.5rem;
		}

		.title-section h2 {
			font-size: 1.75rem;
		}

		.metrics-grid {
			grid-template-columns: repeat(3, 1fr);
			gap: 1rem;
		}

		.canvas-stage {
			padding: 1rem;
		}

		.service-panel {
			padding: 1rem 1.5rem;
		}

		.service-grid {
			grid-template-columns: 1fr;
		}
	}

	@media (max-width: 480px) {
		.status-indicator {
			flex-direction: column;
			text-align: center;
			gap: 0.5rem;
		}

		.metrics-grid {
			gap: 0.5rem;
		}
	}
</style>