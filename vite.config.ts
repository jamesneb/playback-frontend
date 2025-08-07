import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig(({ command, mode }) => {
	const environment = process.env.PLAYBACK_ENV || 'local';
	const envDir = `environments/${environment}`;
	
	// Load environment variables
	const env = loadEnv(mode, envDir, '');
	
	return {
		envDir,
		plugins: [
			sveltekit(),
			wasm(),
			topLevelAwait()
		],
		server: {
			headers: {
				'Cross-Origin-Embedder-Policy': 'require-corp',
				'Cross-Origin-Opener-Policy': 'same-origin'
			}
		},
		optimizeDeps: {
			exclude: ['@ffmpeg/ffmpeg', '@ffmpeg/util']
		},
		assetsInclude: ['**/*.wasm']
	};
});