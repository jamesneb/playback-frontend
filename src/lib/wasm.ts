import type { WasmModule } from './types.js';

export class WasmLoader {
	private static instance: WasmLoader;
	private modules: Map<string, WasmModule> = new Map();

	static getInstance(): WasmLoader {
		if (!WasmLoader.instance) {
			WasmLoader.instance = new WasmLoader();
		}
		return WasmLoader.instance;
	}

	async loadModule(name: string, wasmUrl: string): Promise<WasmModule> {
		if (this.modules.has(name)) {
			return this.modules.get(name)!;
		}

		try {
			const wasmBytes = await fetch(wasmUrl).then(r => r.arrayBuffer());
			const wasmModule = await WebAssembly.compile(wasmBytes);
			const wasmInstance = await WebAssembly.instantiate(wasmModule);

			const module: WasmModule = {
				instance: wasmInstance,
				module: wasmModule
			};

			this.modules.set(name, module);
			return module;
		} catch (error) {
			console.error(`Failed to load WASM module ${name}:`, error);
			throw error;
		}
	}

	getModule(name: string): WasmModule | null {
		return this.modules.get(name) || null;
	}
}