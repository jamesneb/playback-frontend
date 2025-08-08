// Service to manage WASM Web Worker communication

interface WorkerResponse {
  type: string;
  error?: string;
  [key: string]: any;
}

class WasmWorkerService {
  private worker: Worker | null = null;
  private initialized = false;
  private animationStarted = false;
  
  // Initialize the worker with OffscreenCanvas
  async initialize(canvas: HTMLCanvasElement): Promise<void> {
    if (this.initialized) {
      console.log('✅ WASM Worker already initialized');
      return;
    }
    
    console.log('🔧 Initializing WASM Worker with OffscreenCanvas');
    
    // Create worker
    this.worker = new Worker(
      new URL('./wasm-worker.ts', import.meta.url),
      { type: 'module' }
    );
    
    // Set up message handler
    this.setupMessageHandler();
    
    // Transfer canvas to worker
    const offscreenCanvas = canvas.transferControlToOffscreen();
    
    return new Promise((resolve, reject) => {
      const handleInitResponse = (event: MessageEvent<WorkerResponse>) => {
        const message = event.data;
        
        if (message.type === 'init_success') {
          console.log('✅ WASM Worker initialized successfully');
          this.initialized = true;
          this.worker?.removeEventListener('message', handleInitResponse);
          resolve();
        } else if (message.type === 'init_error') {
          console.error('❌ WASM Worker initialization failed:', message.error);
          this.worker?.removeEventListener('message', handleInitResponse);
          reject(new Error(message.error));
        }
      };
      
      this.worker?.addEventListener('message', handleInitResponse);
      
      // Send init message with transferred canvas
      this.worker?.postMessage({
        type: 'init',
        canvas: offscreenCanvas
      }, [offscreenCanvas]);
      
      // Timeout after 10 seconds
      setTimeout(() => {
        this.worker?.removeEventListener('message', handleInitResponse);
        reject(new Error('WASM Worker initialization timeout'));
      }, 10000);
    });
  }
  
  // Set up message handler for ongoing communication
  private setupMessageHandler(): void {
    if (!this.worker) return;
    
    this.worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
      const message = event.data;
      
      console.log('📨 Main: Received from worker:', message.type);
      
      switch (message.type) {
        case 'data_updated':
          console.log('✅ Main: Data updated in worker');
          if (message.serviceCount) {
            console.log(`📊 Main: Worker has ${message.serviceCount} services`);
          }
          break;
          
        case 'animation_started':
          console.log('✅ Main: Animation started in worker');
          this.animationStarted = true;
          if (message.serviceCount) {
            console.log(`🚀 Main: Animation running for ${message.serviceCount} services`);
          }
          break;
          
        case 'animation_stopped':
          console.log('⏹️ Main: Animation stopped in worker');
          this.animationStarted = false;
          break;
          
        case 'service_frame':
          console.log(`🎯 Main: Service frame ${message.serviceIndex + 1}/${message.serviceCount}`);
          // Main thread no longer needs to do anything for service frames
          // The worker handles all timing and rendering
          break;
          
        case 'data_error':
        case 'animation_error':
        case 'worker_error':
          console.error('❌ Main: Worker error:', message.error);
          break;
          
        default:
          console.log('📝 Main: Worker message:', message);
      }
    };
    
    this.worker.onerror = (error) => {
      console.error('❌ Main: Worker error:', error);
    };
  }
  
  // Send new data to worker (non-blocking)
  appendData(data: Uint8Array): void {
    if (!this.worker || !this.initialized) {
      console.error('❌ Main: Worker not initialized, cannot append data');
      return;
    }
    
    // DON'T transfer ArrayBuffer - copy it instead to avoid detached buffer
    this.worker.postMessage({
      type: 'append',
      data: data  // This will be copied, not transferred
    });
  }
  
  // Start animation (only once)
  startAnimation(): void {
    if (!this.worker || !this.initialized) {
      console.error('❌ Main: Worker not initialized, cannot start animation');
      return;
    }
    
    if (this.animationStarted) {
      console.log('✅ Main: Animation already started');
      return;
    }
    
    console.log('🚀 Main: Starting animation in worker');
    this.worker.postMessage({ type: 'start_animation' });
  }
  
  // Stop animation
  stopAnimation(): void {
    if (!this.worker || !this.initialized || !this.animationStarted) {
      console.log('⏹️ Main: Animation not running or worker not ready');
      return;
    }
    
    console.log('🛑 Main: Stopping animation in worker');
    this.worker.postMessage({ type: 'stop_animation' });
  }
  
  // Clean up worker
  terminate(): void {
    if (this.worker) {
      console.log('🔚 Main: Terminating WASM worker');
      this.stopAnimation();
      this.worker.terminate();
      this.worker = null;
      this.initialized = false;
      this.animationStarted = false;
    }
  }
  
  // Getters
  get isInitialized(): boolean {
    return this.initialized;
  }
  
  get isAnimationStarted(): boolean {
    return this.animationStarted;
  }
}

// Singleton instance
export const wasmWorkerService = new WasmWorkerService();