import { NO_GPU_ADAPTER_FOUND, GPU_DEVICE_REQUEST_FAILED, DEFAULT_TEXTURE_FORMAT } from '../constants/errors';
class WebGPUContext {
  readonly adapter: GPUAdapter;
  readonly device:  GPUDevice;
  readonly format:  GPUTextureFormat;

  private constructor(adapter: GPUAdapter, device: GPUDevice, format: GPUTextureFormat) {
    this.adapter = adapter;
    this.device  = device;
    this.format  = format;
  }

  static async create(format?: GPUTextureFormat): Promise<WebGPUContext> {

    const adapter = await navigator.gpu.requestAdapter();

    if (!adapter) throw new Error (NO_GPU_ADAPTER_FOUND);

    const device  = await adapter.requestDevice();

    if (!device) throw new Error (GPU_DEVICE_REQUEST_FAILED);

    return new WebGPUContext(
      adapter,
      device,
      format ?? DEFAULT_TEXTURE_FORMAT
    )

  }
}


