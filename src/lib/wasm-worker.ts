// Web Worker for WASM engine with OffscreenCanvas
// Worker owns the entire animation loop and timing

let wasmModule: any = null;
let canvas: OffscreenCanvas | null = null;
let animationRunning = false;
let timerId: number | null = null;
let currentServiceIndex = 0;
let serviceCount = 0;
let busy = false;

const SERVICE_ROTATE_MS = 10000; // 10 seconds per service

// Message types
interface InitMessage {
  type: 'init';
  canvas: OffscreenCanvas;
}

interface AppendDataMessage {
  type: 'append';
  data: Uint8Array;
}

interface StartAnimationMessage {
  type: 'start_animation';
}

interface StopAnimationMessage {
  type: 'stop_animation';
}

type WorkerMessage = InitMessage | AppendDataMessage | StartAnimationMessage | StopAnimationMessage;

// Check if we have services available
function hasServices(): boolean {
  return serviceCount > 0;
}

// Animation step function - using setTimeout (not rAF which doesn't work in workers)
function animationStep() {
  if (!animationRunning || !wasmModule) {
    return;
  }
  
  if (hasServices()) {
    currentServiceIndex = (currentServiceIndex + 1) % serviceCount;
    
    console.log(`🎯 Worker: Rotating to service ${currentServiceIndex + 1}/${serviceCount}`);
    
    // Tell WASM to render current service
    try {
      if (wasmModule.render_service_by_index) {
        wasmModule.render_service_by_index(currentServiceIndex);
      } else {
        // Fallback to general render
        wasmModule.render_replay();
      }
      
      // Notify main thread of current service for UI updates
      self.postMessage({ 
        type: 'service_frame', 
        serviceIndex: currentServiceIndex,
        serviceCount: serviceCount 
      });
      
    } catch (error) {
      console.error('❌ Worker: Animation render error:', error);
    }
  }
  
  // Schedule next tick using setTimeout (works in workers)
  timerId = self.setTimeout(animationStep, SERVICE_ROTATE_MS) as unknown as number;
}

// Start animation once (idempotent)
function startAnimationOnce() {
  if (animationRunning || !hasServices() || !wasmModule) {
    return;
  }
  
  console.log(`🚀 Worker: Starting animation loop for ${serviceCount} services`);
  animationRunning = true;
  currentServiceIndex = 0;
  
  // Start the animation loop with setTimeout
  timerId = self.setTimeout(animationStep, SERVICE_ROTATE_MS) as unknown as number;
  
  self.postMessage({ type: 'animation_started', serviceCount });
}

// Stop animation
function stopAnimationLoop() {
  if (!animationRunning) return;
  
  console.log('🛑 Worker: Stopping animation loop');
  animationRunning = false;
  
  if (timerId !== null) {
    self.clearTimeout(timerId);
    timerId = null;
  }
  
  self.postMessage({ type: 'animation_stopped' });
}

// Initialize WASM once and keep it alive
async function initWasmOnce(offscreenCanvas: OffscreenCanvas) {
  console.log('🔧 Worker: Initializing WASM once with OffscreenCanvas');
  
  try {
    // Import WASM module - it's already initialized on import
    const wasmImport = await import('$lib/wasm-pkg/hello_wasm.js');
    wasmModule = wasmImport;
    canvas = offscreenCanvas;
    
    // Initialize WebGPU with OffscreenCanvas
    await wasmModule.init_webgpu(offscreenCanvas);
    
    console.log('✅ Worker: WASM initialized successfully');
    
    // Post success message back to main thread
    self.postMessage({ type: 'init_success' });
    
  } catch (error) {
    console.error('❌ Worker: Failed to initialize WASM:', error);
    self.postMessage({ 
      type: 'init_error', 
      error: error instanceof Error ? error.message : String(error) 
    });
  }
}

// Append new data using proper memory management - NO STATE REBUILDING
function appendData(data: Uint8Array) {
  if (!wasmModule) {
    console.error('❌ Worker: WASM not initialized');
    return;
  }
  
  // Prevent concurrent appends
  if (busy) {
    console.log('🔄 Worker: Busy with previous append, skipping');
    return;
  }
  
  busy = true;
  
  try {
    // Check if we have proper memory access
    if (!wasmModule.alloc || !wasmModule.append_chunk || !wasmModule.memory) {
      console.log('🔄 Worker: Using fallback parse_arrow_replay (no memory access)');
      // Fallback to parse_arrow_replay if memory management isn't available
      const result = wasmModule.parse_arrow_replay(data);
      
      // Try to extract service count from result
      if (result && result.services) {
        serviceCount = result.services.length;
        console.log(`📊 Worker: Found ${serviceCount} services via fallback`);
      }
      
      console.log('✅ Worker: Data processed via fallback method');
    } else {
      // Use proper append pattern with direct memory writes
      const ptr = wasmModule.alloc(data.length);
      
      try {
        // Write directly to WASM linear memory
        const memoryView = new Uint8Array(wasmModule.memory.buffer, ptr, data.length);
        memoryView.set(data);
        
        // Call append_chunk - no state rebuilding
        wasmModule.append_chunk(ptr, data.length);
        
        console.log('✅ Worker: Appended', data.length, 'bytes via direct memory');
        
        // Get updated service count
        try {
          serviceCount = wasmModule.get_service_count();
          console.log(`📊 Worker: Service count updated to ${serviceCount}`);
        } catch (countError) {
          console.log('🔄 Worker: Could not get service count from WASM');
        }
        
      } finally {
        // Always free allocated memory
        wasmModule.free(ptr, data.length);
      }
    }
    
    // Start animation automatically if we have services (idempotent)
    startAnimationOnce();
    
    // Notify main thread
    self.postMessage({ type: 'data_updated', serviceCount });
    
  } catch (error) {
    console.error('❌ Worker: Failed to append data:', error);
    self.postMessage({ 
      type: 'data_error', 
      error: error instanceof Error ? error.message : String(error) 
    });
  } finally {
    busy = false;
  }
}

// Legacy startAnimation - redirect to new function
function startAnimation() {
  startAnimationOnce();
}

// Legacy stopAnimation - redirect to new function
function stopAnimation() {
  stopAnimationLoop();
}

// Handle messages from main thread
self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;
  
  console.log('📨 Worker: Received message:', message.type);
  
  switch (message.type) {
    case 'init':
      await initWasmOnce(message.canvas);
      break;
      
    case 'append':
      appendData(message.data);
      break;
      
    case 'start_animation':
      startAnimation();
      break;
      
    case 'stop_animation':
      stopAnimation();
      break;
      
    default:
      console.warn('⚠️ Worker: Unknown message type:', (message as any).type);
  }
};

// Handle worker errors
self.onerror = (error) => {
  console.error('❌ Worker: Global error:', error);
  self.postMessage({ 
    type: 'worker_error', 
    error: error.message 
  });
};

console.log('🟢 Worker: WASM worker initialized and ready for messages');