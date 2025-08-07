export interface WasmModule {
	instance: WebAssembly.Instance;
	module: WebAssembly.Module;
}

export interface PlaybackData {
	id: string;
	timestamp: number;
	data: Uint8Array;
}

export interface TraceData {
	traceId: string;
	spanId: string;
	operationName: string;
	startTime: number;
	duration: number;
	tags: Record<string, any>;
}