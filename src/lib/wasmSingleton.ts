// WASM Singleton - Initialize once, keep alive, push data incrementally

interface WasmExports {
  alloc(size: number): number;
  free(ptr: number, size: number): void;
  append_chunk(ptr: number, len: number): void;
  render_replay(): void;
  stop_animation(): void;
  init_webgpu(canvas: HTMLCanvasElement): Promise<void>;
  memory: WebAssembly.Memory;
}

class WasmSingleton {
  private wasmReady: Promise<WasmExports> | null = null;
  private exportsRef: WasmExports | null = null;
  private animationStarted = false;

  // Initialize WASM once and keep it alive
  getWasm(): Promise<WasmExports> {
    if (!this.wasmReady) {
      this.wasmReady = (async () => {
        console.log('🔧 Initializing WASM singleton...');
        
        // Import and initialize wasm-pack module
        const wasm = await import('$lib/wasm-pkg/hello_wasm.js');
        await wasm.default();
        
        // Store exports reference
        this.exportsRef = wasm as unknown as WasmExports;
        
        console.log('✅ WASM singleton initialized');
        return this.exportsRef!;
      })();
    }
    return this.wasmReady;
  }

  // Initialize WebGPU (call once after WASM is ready)
  async initWebGPU(canvas: HTMLCanvasElement): Promise<void> {
    const wasm = await this.getWasm();
    await wasm.init_webgpu(canvas);
    console.log('✅ WebGPU initialized with WASM singleton');
  }

  // Append new data using direct memory writes (zero-copy)
  async appendData(bytes: Uint8Array): Promise<void> {
    const wasm = await this.getWasm();
    
    // Allocate memory in WASM
    const ptr = wasm.alloc(bytes.length);
    
    try {
      // Write bytes directly to WASM linear memory
      const memoryView = new Uint8Array(wasm.memory.buffer, ptr, bytes.length);
      memoryView.set(bytes);
      
      // Call append function with pointer and length
      wasm.append_chunk(ptr, bytes.length);
      
      console.log('✅ Appended', bytes.length, 'bytes via direct memory write');
      
    } finally {
      // Always free the allocated memory
      wasm.free(ptr, bytes.length);
    }
  }

  // Start animation (only once)
  async startAnimation(): Promise<void> {
    if (this.animationStarted) {
      console.log('✅ Animation already started');
      return;
    }

    const wasm = await this.getWasm();
    wasm.render_replay();
    this.animationStarted = true;
    console.log('🚀 Animation started via singleton');
  }

  // Stop animation
  async stopAnimation(): Promise<void> {
    if (!this.animationStarted) return;
    
    const wasm = await this.getWasm();
    wasm.stop_animation();
    this.animationStarted = false;
    console.log('🛑 Animation stopped via singleton');
  }

  // Check if initialized
  get isReady(): boolean {
    return this.exportsRef !== null;
  }

  get isAnimationStarted(): boolean {
    return this.animationStarted;
  }
}

// Export singleton instance
export const wasmSingleton = new WasmSingleton();