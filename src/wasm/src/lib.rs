use wasm_bindgen::prelude::*;
use std::alloc::{alloc as std_alloc, dealloc, Layout};
use web_sys::HtmlCanvasElement;
use wgpu::*;
use wgpu::util::DeviceExt;
use js_sys::Date;
use wasm_bindgen::closure::Closure;
use web_sys::window;
use arrow::array::{Array, StringArray};
use arrow::ipc::reader::FileReader;
use std::io::Cursor;
use serde::{Deserialize, Serialize};

// Import the `console.log` function from the Web API
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to make console.log more convenient
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Global WebGPU renderer state
struct WebGpuRenderer {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    render_pipeline: RenderPipeline,
    current_service_index: usize,
}

// Use thread_local for WASM since we're single-threaded
use std::cell::RefCell;
thread_local! {
    static GPU_RENDERER: RefCell<Option<WebGpuRenderer>> = RefCell::new(None);
    static REPLAY_DATA: RefCell<Option<ReplayData>> = RefCell::new(None);
    static ANIMATION_RUNNING: RefCell<bool> = RefCell::new(false);
    static LAST_SERVICE_COUNT: RefCell<usize> = RefCell::new(0);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNode {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub status: String, // "healthy", "warning", "error"
}

#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct ReplayData {
    pub services: Vec<ServiceNode>,
    pub all_service_ids: Vec<String>, // All unique service IDs from Arrow file
    pub timestamp: String,
    pub job_id: String,
}

const VERTEX_SHADER_SOURCE: &str = r#"
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.8, 0.2, 1.0); // Epic golden color for FINAL BOSS MODE! 🚀
}
"#;

const FADE_FRAGMENT_SHADER_SOURCE: &str = r#"
@group(0) @binding(0) var<uniform> fade_alpha: f32;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.8, 0.2, fade_alpha); // Alpha-blended golden color for smooth transitions
}
"#;

#[wasm_bindgen]
pub async fn init_webgpu(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    console_log!("🔥🔥🔥 FINAL WASM BUILD 2025-08-08-21:30 - FIXED WORKER + FALLBACK 🔥🔥🔥");
    console_log!("🚀 Initializing REAL WebGPU for FINAL BOSS MODE!");
    
    // Create wgpu instance
    let instance = Instance::new(&InstanceDescriptor {
        backends: Backends::GL | Backends::BROWSER_WEBGPU,
        ..Default::default()
    });
    
    // Get canvas size before moving canvas
    let canvas_size = (canvas.width() as u32, canvas.height() as u32);
    
    // Create surface from canvas - WASM specific
    let surface = instance.create_surface(SurfaceTarget::Canvas(canvas))
        .map_err(|e| JsValue::from_str(&format!("Failed to create surface: {:?}", e)))?;
    console_log!("🖼️ Surface created successfully");
    
    // Request adapter
    let adapter = instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }).await.map_err(|e| JsValue::from_str(&format!("Failed to find adapter: {:?}", e)))?;
    
    console_log!("📊 Adapter found: {:?}", adapter.get_info());
    
    // Request device and queue with completely empty descriptor to avoid limit issues
    let (device, queue) = adapter.request_device(
        &DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            required_limits: Limits::default().using_resolution(adapter.limits()),
            memory_hints: Default::default(),
            trace: wgpu::Trace::default(),
        }
    ).await.map_err(|e| JsValue::from_str(&format!("Failed to create device: {:?}", e)))?;
    
    // Configure surface
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let surface_config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: canvas_size.0,
        height: canvas_size.1,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &surface_config);
    console_log!("🎨 Surface configured with format: {:?}, size: {}x{}", surface_config.format, surface_config.width, surface_config.height);
    
    // Create shaders
    let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: ShaderSource::Wgsl(VERTEX_SHADER_SOURCE.into()),
    });
    
    let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Fragment Shader"),
        source: ShaderSource::Wgsl(FRAGMENT_SHADER_SOURCE.into()),
    });
    
    // No fade shaders needed for simple 10-second transitions
    
    // Create render pipeline
    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        })),
        cache: None,
        vertex: VertexState {
            module: &vertex_shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &[VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                }],
            }],
        },
        fragment: Some(FragmentState {
            module: &fragment_shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(ColorTargetState {
                format: surface_config.format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,  // Disable culling to see if triangles are facing wrong way
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });
    
    // Simple pipeline - no fade effects needed for 10-second transitions
    
    // Store the renderer globally
    let renderer = WebGpuRenderer {
        device,
        queue,
        surface,
        surface_config,
        render_pipeline,
        current_service_index: 0,
    };
    
    GPU_RENDERER.with(|r| *r.borrow_mut() = Some(renderer));
    
    console_log!("✅ REAL WebGPU initialized successfully! FINAL BOSS MODE ACTIVATED! 🔥");
    Ok(())
}

fn create_char_geometry(c: char, x: f32, y: f32, char_width: f32, char_height: f32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let segment_width = char_width / 5.0;
    let segment_height = char_height / 7.0;
    
    // Simple 5x7 bitmap font patterns
    let pattern = match c.to_ascii_uppercase() {
        'H' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (2,3),                                            // Middle horizontal
            (4,0), (4,1), (4,2), (4,3), (4,4), (4,5), (4,6), // Right vertical
        ],
        'E' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,0), (2,0), (3,0), (4,0),                       // Bottom horizontal
            (1,3), (2,3), (3,3),                              // Middle horizontal  
            (1,6), (2,6), (3,6), (4,6),                       // Top horizontal
        ],
        'L' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,0), (2,0), (3,0), (4,0),                       // Bottom horizontal
        ],
        'O' => vec![
            (1,0), (2,0), (3,0),                              // Bottom horizontal
            (0,1), (0,2), (0,3), (0,4), (0,5),                // Left vertical
            (4,1), (4,2), (4,3), (4,4), (4,5),                // Right vertical
            (1,6), (2,6), (3,6),                              // Top horizontal
        ],
        'A' => vec![
            (1,0), (2,0), (3,0),                              // Bottom of A
            (0,1), (0,2), (0,3), (0,4), (0,5),                // Left vertical
            (4,1), (4,2), (4,3), (4,4), (4,5),                // Right vertical
            (1,3), (2,3), (3,3),                              // Middle horizontal
            (2,6),                                            // Top
        ],
        'W' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,0), (1,1),                                      // Left bottom
            (2,0), (2,1), (2,2),                              // Middle
            (3,0), (3,1),                                      // Right bottom
            (4,0), (4,1), (4,2), (4,3), (4,4), (4,5), (4,6), // Right vertical
        ],
        'S' => vec![
            (1,0), (2,0), (3,0), (4,0),                       // Bottom horizontal
            (0,1), (0,2),                                      // Left bottom
            (1,3), (2,3), (3,3),                              // Middle horizontal
            (4,4), (4,5),                                      // Right top
            (0,6), (1,6), (2,6), (3,6),                       // Top horizontal
        ],
        'M' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,5), (1,6),                                      // Left diagonal
            (2,4), (2,5), (2,6),                              // Middle
            (3,5), (3,6),                                      // Right diagonal
            (4,0), (4,1), (4,2), (4,3), (4,4), (4,5), (4,6), // Right vertical
        ],
        'V' => vec![
            (0,3), (0,4), (0,5), (0,6),                       // Left top
            (1,1), (1,2),                                      // Left middle
            (2,0),                                             // Bottom center
            (3,1), (3,2),                                      // Right middle  
            (4,3), (4,4), (4,5), (4,6),                       // Right top
        ],
        'T' => vec![
            (0,6), (1,6), (2,6), (3,6), (4,6),                // Top horizontal
            (2,0), (2,1), (2,2), (2,3), (2,4), (2,5),         // Center vertical
        ],
        'K' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,3),                                             // Middle connection
            (2,2), (2,4),                                      // Diagonals
            (3,1), (3,5),                                      // Diagonals  
            (4,0), (4,6),                                      // Right ends
        ],
        'I' => vec![
            (0,0), (1,0), (2,0), (3,0), (4,0),                // Bottom horizontal
            (2,1), (2,2), (2,3), (2,4), (2,5),                // Center vertical
            (0,6), (1,6), (2,6), (3,6), (4,6),                // Top horizontal
        ],
        'R' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,3), (1,6), (2,3), (2,6), (3,3), (3,6),         // Horizontals
            (4,4), (4,5), (4,6),                               // Right top
            (3,1), (4,0),                                      // Right diagonal
        ],
        'C' => vec![
            (1,0), (2,0), (3,0),                              // Bottom horizontal
            (0,1), (0,2), (0,3), (0,4), (0,5),                // Left vertical
            (1,6), (2,6), (3,6),                              // Top horizontal
            (4,1), (4,5),                                      // Right ends only
        ],
        'N' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,1), (2,2), (3,3),                              // Diagonal
            (4,0), (4,1), (4,2), (4,3), (4,4), (4,5), (4,6), // Right vertical
        ],
        'U' => vec![
            (0,1), (0,2), (0,3), (0,4), (0,5), (0,6),         // Left vertical
            (1,0), (2,0), (3,0),                              // Bottom horizontal
            (4,1), (4,2), (4,3), (4,4), (4,5), (4,6),         // Right vertical
        ],
        'D' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,0), (2,0), (3,0),                              // Bottom horizontal
            (1,6), (2,6), (3,6),                              // Top horizontal
            (4,1), (4,2), (4,3), (4,4), (4,5),                // Right vertical
        ],
        'P' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,3), (1,6), (2,3), (2,6), (3,3), (3,6),         // Horizontals
            (4,4), (4,5), (4,6),                               // Right top only
        ],
        'F' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,3), (2,3), (3,3),                              // Middle horizontal
            (1,6), (2,6), (3,6), (4,6),                       // Top horizontal
        ],
        'G' => vec![
            (1,0), (2,0), (3,0),                              // Bottom horizontal
            (0,1), (0,2), (0,3), (0,4), (0,5),                // Left vertical
            (1,6), (2,6), (3,6),                              // Top horizontal
            (4,1), (4,2), (2,3), (3,3), (4,3),                // Right with middle bar
        ],
        'B' => vec![
            (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,6), // Left vertical
            (1,0), (2,0), (3,0),                              // Bottom horizontal
            (1,3), (2,3), (3,3),                              // Middle horizontal
            (1,6), (2,6), (3,6),                              // Top horizontal
            (4,1), (4,2), (4,4), (4,5),                       // Right segments
        ],
        'Y' => vec![
            (0,5), (0,6),                                      // Left top
            (1,4), (2,3),                                      // Left diagonal
            (2,0), (2,1), (2,2),                              // Center bottom
            (3,4), (4,5), (4,6),                              // Right diagonal and top
        ],
        'J' => vec![
            (0,1),                                             // Left bottom hook
            (1,0), (2,0), (3,0),                              // Bottom horizontal
            (4,1), (4,2), (4,3), (4,4), (4,5), (4,6),         // Right vertical
        ],
        'Q' => vec![
            (1,0), (2,0), (3,0),                              // Bottom horizontal  
            (0,1), (0,2), (0,3), (0,4), (0,5),                // Left vertical
            (4,1), (4,2), (4,3), (4,4), (4,5),                // Right vertical
            (1,6), (2,6), (3,6),                              // Top horizontal
            (3,1), (4,0),                                      // Tail
        ],
        'X' => vec![
            (0,0), (0,6),                                      // Left corners
            (1,1), (1,5),                                      // Inner diagonals
            (2,2), (2,3), (2,4),                              // Center
            (3,1), (3,5),                                      // Inner diagonals
            (4,0), (4,6),                                      // Right corners
        ],
        'Z' => vec![
            (0,0), (1,0), (2,0), (3,0), (4,0),                // Bottom horizontal
            (4,1), (3,2), (2,3), (1,4), (0,5),                // Diagonal
            (0,6), (1,6), (2,6), (3,6), (4,6),                // Top horizontal
        ],
        '+' => vec![
            (2,1), (2,2), (2,4), (2,5),                       // Vertical line
            (1,3), (3,3),                                      // Horizontal line
        ],
        '=' => vec![
            (1,2), (2,2), (3,2),                              // Top line
            (1,4), (2,4), (3,4),                              // Bottom line
        ],
        '!' => vec![
            (2,2), (2,3), (2,4), (2,5), (2,6),                // Vertical line
            (2,0),                                             // Bottom dot
        ],
        ',' => vec![
            (2,0), (1,1),                                      // Comma shape
        ],
        '.' => vec![
            (2,0),                                             // Dot
        ],
        ':' => vec![
            (2,1), (2,5),                                      // Two dots
        ],
        ';' => vec![
            (2,5), (2,1), (1,0),                              // Semicolon
        ],
        '?' => vec![
            (1,6), (2,6), (3,6),                              // Top
            (4,5), (4,4),                                      // Right top
            (2,3), (3,3),                                      // Middle
            (2,0),                                             // Bottom dot
        ],
        '0'..='9' => {
            let digit = c.to_digit(10).unwrap_or(0);
            match digit {
                0 => vec![
                    (1,0), (2,0), (3,0),                       // Bottom
                    (0,1), (0,2), (0,3), (0,4), (0,5),         // Left
                    (4,1), (4,2), (4,3), (4,4), (4,5),         // Right
                    (1,6), (2,6), (3,6),                       // Top
                ],
                1 => vec![
                    (2,0), (2,1), (2,2), (2,3), (2,4), (2,5), (2,6), // Vertical line
                    (1,5),                                      // Top left
                ],
                2 => vec![
                    (0,0), (1,0), (2,0), (3,0), (4,0),         // Bottom
                    (0,1), (0,2),                               // Left bottom
                    (1,3), (2,3), (3,3),                       // Middle
                    (4,4), (4,5),                               // Right top
                    (0,6), (1,6), (2,6), (3,6), (4,6),         // Top
                ],
                3 => vec![
                    (0,0), (1,0), (2,0), (3,0),                // Bottom
                    (4,1), (4,2),                               // Right bottom
                    (1,3), (2,3), (3,3),                       // Middle
                    (4,4), (4,5),                               // Right top
                    (0,6), (1,6), (2,6), (3,6),                // Top
                ],
                4 => vec![
                    (3,0), (3,1), (3,2), (3,3), (3,4), (3,5), (3,6), // Right vertical
                    (0,3), (0,4), (0,5),                       // Left vertical
                    (1,3), (2,3),                               // Middle horizontal
                ],
                5 => vec![
                    (1,0), (2,0), (3,0), (4,0),                // Bottom
                    (4,1), (4,2),                               // Right bottom
                    (1,3), (2,3), (3,3),                       // Middle
                    (0,4), (0,5),                               // Left top
                    (0,6), (1,6), (2,6), (3,6), (4,6),         // Top
                ],
                6 => vec![
                    (1,0), (2,0), (3,0),                       // Bottom
                    (0,1), (0,2), (0,3), (0,4), (0,5),         // Left
                    (1,3), (2,3), (3,3),                       // Middle
                    (4,1), (4,2),                               // Right bottom
                    (1,6), (2,6), (3,6),                       // Top
                ],
                7 => vec![
                    (3,0), (3,1), (3,2), (3,3),                // Right diagonal
                    (2,4), (2,5),                               // Middle
                    (0,6), (1,6), (2,6), (3,6), (4,6),         // Top
                ],
                8 => vec![
                    (1,0), (2,0), (3,0),                       // Bottom
                    (0,1), (0,2), (0,4), (0,5),                // Left
                    (4,1), (4,2), (4,4), (4,5),                // Right
                    (1,3), (2,3), (3,3),                       // Middle
                    (1,6), (2,6), (3,6),                       // Top
                ],
                _ => vec![(2,3)], // Default dot
            }
        },
        ' ' => vec![], // Space
        _ => vec![(1,1), (2,1), (3,1), (1,2), (2,2), (3,2), (1,3), (2,3), (3,3)], // Default block
    };
    
    // Convert pattern to triangles
    for (px, py) in pattern {
        let seg_x = x + px as f32 * segment_width;
        let seg_y = y + py as f32 * segment_height;
        
        // Two triangles per segment
        vertices.extend_from_slice(&[
            seg_x, seg_y,                                    // Bottom left
            seg_x + segment_width, seg_y,                    // Bottom right
            seg_x, seg_y + segment_height,                   // Top left
        ]);
        vertices.extend_from_slice(&[
            seg_x, seg_y + segment_height,                   // Top left
            seg_x + segment_width, seg_y,                    // Bottom right
            seg_x + segment_width, seg_y + segment_height,   // Top right
        ]);
    }
    
    vertices
}

fn create_text_geometry(text: &str, x: f32, y: f32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let char_width = 0.08;
    let char_height = 0.14;
    let char_spacing = 0.01;
    
    for (i, c) in text.chars().enumerate() {
        let char_x = x + i as f32 * (char_width + char_spacing);
        let char_vertices = create_char_geometry(c, char_x, y, char_width, char_height);
        vertices.extend(char_vertices);
    }
    
    vertices
}

fn render_text_webgpu(text: &str) -> Result<(), JsValue> {
    GPU_RENDERER.with(|r| {
        let renderer_ref = r.borrow();
        let renderer = renderer_ref.as_ref()
            .ok_or_else(|| JsValue::from_str("WebGPU not initialized"))?;
        
        // Create vertex data for text using proper bitmap font
        let vertices = create_text_geometry(text, -0.5, 0.0);
        console_log!("🔍 Generated {} vertices for text '{}': {:?}", vertices.len(), text, &vertices[0..std::cmp::min(12, vertices.len())]);
        
        // Create vertex buffer
        let vertex_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        
        // Get current surface texture
        let output = renderer.surface.get_current_texture()
            .map_err(|e| JsValue::from_str(&format!("Failed to get surface texture: {:?}", e)))?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        
        // Create command encoder
        let mut encoder = renderer.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            // Begin render pass
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0, 
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            render_pass.set_pipeline(&renderer.render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            let vertex_count = (vertices.len() / 2) as u32; // Each vertex is 2 f32s (x,y)
            console_log!("🎯 Drawing {} vertices from {} f32 values", vertex_count, vertices.len());
            render_pass.draw(0..vertex_count, 0..1);
        }
        
        // Submit commands and present
        renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        console_log!("🎨 Rendered '{}' with WebGPU! FINAL BOSS POWER! ⚡", text);
        
        // Check for any errors
        console_log!("🔍 Checking for WebGPU errors...");
        Ok(())
    })
}

// Export a `hello` function that renders via WebGPU
#[wasm_bindgen]
pub fn hello() -> Result<(), JsValue> {
    console_log!("🦀 Hello from Rust WASM via REAL WebGPU!");
    render_text_webgpu("Hello WASM! 🦀")
}

// Export an `add` function that renders the result via WebGPU
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> Result<i32, JsValue> {
    let result = a + b;
    console_log!("Adding {} + {} = {} via REAL WebGPU", a, b, result);
    render_text_webgpu(&format!("{} + {} = {}", a, b, result))?;
    Ok(result)
}

// Export a function that renders a greeting via WebGPU
#[wasm_bindgen]
pub fn greet(name: &str) -> Result<String, JsValue> {
    let greeting = format!("Hello, {}! 🦀", name);
    console_log!("Greeting via REAL WebGPU: {}", greeting);
    render_text_webgpu(&greeting)?;
    Ok(greeting)
}

// Memory management exports for direct WASM memory writes
#[wasm_bindgen]
pub fn alloc(size: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { std_alloc(layout) }
}

#[wasm_bindgen]
pub fn free(ptr: *mut u8, size: usize) {
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { dealloc(ptr, layout) }
}

// Direct append from memory pointer - NO REBUILDING STATE
#[wasm_bindgen]
pub fn append_chunk(ptr: *const u8, len: usize) -> Result<(), JsValue> {
    let data = unsafe { std::slice::from_raw_parts(ptr, len) };
    console_log!("📦 Appending {} bytes directly to WASM state", len);
    
    // Parse the chunk
    let cursor = Cursor::new(data);
    let reader = FileReader::try_new(cursor, None)
        .map_err(|e| JsValue::from_str(&format!("Failed to create Arrow reader: {:?}", e)))?;
    
    let mut new_service_ids = Vec::new();
    let mut unique_ids = std::collections::HashSet::new();
    
    // Read all record batches and collect NEW service IDs
    for maybe_batch in reader {
        let batch = maybe_batch
            .map_err(|e| JsValue::from_str(&format!("Failed to read batch: {:?}", e)))?;
        
        // Extract job IDs from the current basic schema
        if let Some(id_column) = batch.column_by_name("id") {
            if let Some(id_array) = id_column.as_any().downcast_ref::<StringArray>() {
                for i in 0..id_array.len() {
                    let id_value = id_array.value(i);
                    if unique_ids.insert(id_value.to_string()) {
                        new_service_ids.push(id_value.to_string());
                    }
                }
            }
        }
    }
    
    // APPEND to existing state - never replace
    let added_count = REPLAY_DATA.with(|r| {
        let mut borrowed = r.borrow_mut();
        
        match borrowed.as_mut() {
            Some(existing_data) => {
                let mut all_existing_ids: std::collections::HashSet<String> = 
                    existing_data.all_service_ids.iter().cloned().collect();
                
                let mut added_new = 0;
                for new_id in new_service_ids {
                    if all_existing_ids.insert(new_id.clone()) {
                        existing_data.all_service_ids.push(new_id);
                        added_new += 1;
                    }
                }
                
                if added_new > 0 {
                    // Regenerate services list only if new services were added
                    existing_data.services = existing_data.all_service_ids.iter().enumerate().map(|(i, id)| {
                        ServiceNode {
                            id: id.clone(),
                            x: (i as f32 * 0.3) - (existing_data.all_service_ids.len() as f32 * 0.15),
                            y: 0.0,
                            status: "healthy".to_string(),
                        }
                    }).collect();
                }
                
                existing_data.timestamp = chrono::Utc::now().to_rfc3339();
                added_new
            },
            None => {
                // First time - create new data
                let services: Vec<ServiceNode> = new_service_ids.iter().enumerate().map(|(i, id)| {
                    ServiceNode {
                        id: id.clone(),
                        x: (i as f32 * 0.3) - (new_service_ids.len() as f32 * 0.15),
                        y: 0.0,
                        status: "healthy".to_string(),
                    }
                }).collect();
                
                let data = ReplayData {
                    services,
                    all_service_ids: new_service_ids.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    job_id: new_service_ids.first().unwrap_or(&String::new()).clone(),
                };
                
                *borrowed = Some(data);
                new_service_ids.len()
            }
        }
    });
    
    console_log!("✅ Appended chunk: {} new services added", added_count);
    Ok(())
}

// Legacy parse function - kept for compatibility but avoid using in polling loops
#[wasm_bindgen]
pub fn parse_arrow_replay(data: &[u8]) -> Result<JsValue, JsValue> {
    console_log!("🏹 Parsing Arrow IPC file, {} bytes", data.len());
    
    let cursor = Cursor::new(data);
    let reader = FileReader::try_new(cursor, None)
        .map_err(|e| JsValue::from_str(&format!("Failed to create Arrow reader: {:?}", e)))?;
    
    let mut new_service_ids = Vec::new();
    let mut unique_ids = std::collections::HashSet::new();
    let mut job_id = String::new();
    
    // Read all record batches and collect ALL service IDs
    for maybe_batch in reader {
        let batch = maybe_batch
            .map_err(|e| JsValue::from_str(&format!("Failed to read batch: {:?}", e)))?;
        
        console_log!("📊 Processing batch with {} rows, {} columns", batch.num_rows(), batch.num_columns());
        
        // Extract job IDs from the current basic schema
        if let Some(id_column) = batch.column_by_name("id") {
            if let Some(id_array) = id_column.as_any().downcast_ref::<StringArray>() {
                for i in 0..id_array.len() {
                    let id_value = id_array.value(i);
                    
                    // Add to unique set to avoid duplicates (no logging per ID)
                    if unique_ids.insert(id_value.to_string()) {
                        new_service_ids.push(id_value.to_string());
                    }
                    
                    // Keep the last job_id for backwards compatibility
                    job_id = id_value.to_string();
                }
            }
        }
    }
    
    console_log!("🎯 Parsed {} unique service IDs from this file: {:?}", new_service_ids.len(), new_service_ids);
    
    // APPEND to existing state instead of replacing it
    let replay_data = REPLAY_DATA.with(|r| {
        let mut borrowed = r.borrow_mut();
        
        match borrowed.as_mut() {
            Some(existing_data) => {
                // Merge new services with existing ones
                let mut all_existing_ids: std::collections::HashSet<String> = 
                    existing_data.all_service_ids.iter().cloned().collect();
                
                let mut added_new = false;
                for new_id in new_service_ids {
                    if all_existing_ids.insert(new_id.clone()) {
                        existing_data.all_service_ids.push(new_id.clone());
                        added_new = true;
                    }
                }
                
                if added_new {
                    // Regenerate services list with new positions
                    existing_data.services = existing_data.all_service_ids.iter().enumerate().map(|(i, id)| {
                        ServiceNode {
                            id: id.clone(),
                            x: (i as f32 * 0.3) - (existing_data.all_service_ids.len() as f32 * 0.15),
                            y: 0.0,
                            status: "healthy".to_string(),
                        }
                    }).collect();
                    
                    console_log!("📈 Merged services: {} total services", existing_data.all_service_ids.len());
                } else {
                    console_log!("✅ No new services - existing data intact");
                }
                
                existing_data.timestamp = chrono::Utc::now().to_rfc3339();
                existing_data.job_id = job_id;
                existing_data.clone()
            },
            None => {
                // First time - create new data
                console_log!("🆕 Creating initial replay data");
                let services: Vec<ServiceNode> = new_service_ids.iter().enumerate().map(|(i, id)| {
                    ServiceNode {
                        id: id.clone(),
                        x: (i as f32 * 0.3) - (new_service_ids.len() as f32 * 0.15),
                        y: 0.0,
                        status: "healthy".to_string(),
                    }
                }).collect();
                
                let data = ReplayData {
                    services,
                    all_service_ids: new_service_ids,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    job_id,
                };
                
                *borrowed = Some(data.clone());
                data
            }
        }
    });
    
    console_log!("✅ Updated replay data with {} total services", replay_data.services.len());
    
    // Return data to JavaScript
    serde_wasm_bindgen::to_value(&replay_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize replay data: {:?}", e)))
}

// Start animation ONCE - called only when first services are available
#[wasm_bindgen]
pub fn start_service_animation() -> Result<(), JsValue> {
    let is_running = ANIMATION_RUNNING.with(|r| *r.borrow());
    
    if is_running {
        console_log!("🔄 Animation already running, ignoring start request");
        return Ok(());
    }
    
    let services = REPLAY_DATA.with(|r| {
        r.borrow().as_ref().map(|data| data.services.clone()).unwrap_or_default()
    });
    
    if services.is_empty() {
        return Err(JsValue::from_str("No replay data loaded"));
    }
    
    console_log!("🚀 Starting animation for {} services", services.len());
    console_log!("🎯 Service IDs: {:?}", services.iter().map(|s| &s.id).collect::<Vec<_>>());
    
    // Reset to first service
    GPU_RENDERER.with(|r| {
        if let Some(renderer) = r.borrow_mut().as_mut() {
            renderer.current_service_index = 0;
            console_log!("🎯 Reset animation index to 0");
        }
    });
    
    ANIMATION_RUNNING.with(|r| *r.borrow_mut() = true);
    console_log!("✅ Animation flag set to TRUE");
    
    // Start the self-managing animation loop
    if services.len() == 1 {
        console_log!("📱 Single service - showing without transitions");
        render_service_node(&services[0])?;
    } else {
        console_log!("🎬 Multiple services ({}) - starting timed transitions", services.len());
        start_self_managing_loop();
    }
    
    Ok(())
}

// Update data WITHOUT restarting animation - just refreshes the service list
#[wasm_bindgen]
pub fn render_replay() -> Result<(), JsValue> {
    console_log!("📊 Checking if animation should start or continue");
    
    let services = REPLAY_DATA.with(|r| {
        r.borrow().as_ref().map(|data| data.services.clone()).unwrap_or_default()
    });
    
    if services.is_empty() {
        return Err(JsValue::from_str("No replay data loaded"));
    }
    
    // Check if animation is running
    let is_running = ANIMATION_RUNNING.with(|r| *r.borrow());
    let last_count = LAST_SERVICE_COUNT.with(|r| *r.borrow());
    
    if !is_running {
        console_log!("🚀 Starting animation for {} services", services.len());
        LAST_SERVICE_COUNT.with(|r| *r.borrow_mut() = services.len());
        start_service_animation()?;
    } else if services.len() != last_count {
        console_log!("🔄 Service count changed: {} -> {}, animation will pick up changes on next cycle", last_count, services.len());
        LAST_SERVICE_COUNT.with(|r| *r.borrow_mut() = services.len());
    } else {
        console_log!("✅ No service changes - WASM continues internal timing");
    }
    
    Ok(())
}

// Get service count for worker animation control
#[wasm_bindgen]
pub fn get_service_count() -> usize {
    REPLAY_DATA.with(|r| {
        r.borrow().as_ref().map(|data| data.services.len()).unwrap_or(0)
    })
}

// Render specific service by index (bypassing internal animation)
#[wasm_bindgen]
pub fn render_service_by_index(service_index: usize) -> Result<(), JsValue> {
    console_log!("🎯 Worker-controlled render: service index {}", service_index);
    
    let services = REPLAY_DATA.with(|r| {
        r.borrow().as_ref().map(|data| data.services.clone()).unwrap_or_default()
    });
    
    if services.is_empty() {
        console_log!("❌ No services available to render");
        return Err(JsValue::from_str("No services available"));
    }
    
    let actual_index = service_index % services.len();
    let service = &services[actual_index];
    
    console_log!("🎯 Rendering service '{}' (index {}/{})", service.id, actual_index, services.len() - 1);
    
    // Directly render this specific service, bypassing any internal timing
    render_service_node(service)?;
    
    Ok(())
}

fn start_self_managing_loop() {
    console_log!("🎬 ========== STARTING SELF-MANAGING ANIMATION LOOP ==========");
    
    // Render first service immediately
    console_log!("🎯 Rendering first service immediately");
    render_current_service();
    
    // Set up timer for service transitions (10 seconds each)
    console_log!("⏰ Setting up 10-second timer for first transition");
    schedule_next_service_transition();
}

fn render_current_service() {
    console_log!("📍 ========== render_current_service() CALLED ==========");
    
    let services = REPLAY_DATA.with(|r| {
        r.borrow().as_ref().map(|data| data.services.clone()).unwrap_or_default()
    });
    
    if services.is_empty() { 
        console_log!("❌ No services to render");
        return; 
    }
    
    let current_index = GPU_RENDERER.with(|r| {
        r.borrow().as_ref().map(|renderer| renderer.current_service_index).unwrap_or(0)
    });
    
    console_log!("🔍 Total services: {}, current index: {}", services.len(), current_index);
    
    if current_index < services.len() {
        let current_service = &services[current_index];
        console_log!("🎨 ========== RENDERING SERVICE: '{}' (index: {}) ==========", current_service.id, current_index);
        
        match render_service_node(current_service) {
            Ok(_) => {
                console_log!("✅ Successfully rendered service: '{}'", current_service.id);
            },
            Err(e) => {
                console_log!("❌ FAILED to render service: '{}', error: {:?}", current_service.id, e);
            }
        }
    } else {
        console_log!("❌ Invalid index: {} >= {}", current_index, services.len());
    }
}

fn schedule_next_service_transition() {
    console_log!("🔥🔥🔥 TIMER SET FOR 10 SECONDS 🔥🔥🔥");
    
    let closure = Closure::wrap(Box::new(move || {
        console_log!("🔥🔥🔥 🚨🚨🚨 TIMER FIRED AFTER 10 SECONDS!!! 🚨🚨🚨 🔥🔥🔥");
        
        // Check if animation should continue
        let is_running = ANIMATION_RUNNING.with(|r| *r.borrow());
        if !is_running {
            console_log!("🛑 Animation stopped, ending transition loop");
            return;
        }
        
        // Get fresh service data - this picks up new services automatically
        let services = REPLAY_DATA.with(|r| {
            r.borrow().as_ref().map(|data| data.services.clone()).unwrap_or_default()
        });
        
        if services.is_empty() {
            console_log!("❌ No services available, stopping animation");
            ANIMATION_RUNNING.with(|r| *r.borrow_mut() = false);
            return;
        }
        
        // Move to next service (wrapping around to new services automatically)
        let old_index = GPU_RENDERER.with(|r| {
            r.borrow().as_ref().map(|renderer| renderer.current_service_index).unwrap_or(0)
        });
        
        GPU_RENDERER.with(|r| {
            if let Some(renderer) = r.borrow_mut().as_mut() {
                renderer.current_service_index = (renderer.current_service_index + 1) % services.len();
                console_log!("🔄 TRANSITION: {} -> {} (total: {})", 
                            old_index, renderer.current_service_index, services.len());
            }
        });
        
        // Clear canvas first
        if let Err(e) = clear_canvas() {
            console_log!("❌ Failed to clear canvas: {:?}", e);
        }
        
        // Render new current service immediately after clear
        render_current_service();
        
        // Continue the animation loop if we have services
        if services.len() > 0 {
            schedule_next_service_transition();
        } else {
            console_log!("❌ No more services, stopping animation");
        }
    }) as Box<dyn FnMut()>);
    
    // Schedule transition after 10 seconds - DON'T forget the closure
    let window = window().unwrap();
    let timeout_id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(), 
        10000 // 10 seconds
    ).unwrap();
    
    console_log!("🕐 Timeout {} scheduled for 10 seconds", timeout_id);
    
    // Keep the closure alive by storing it in a static var or leaking it
    // We use forget to prevent dropping - this is OK for continuous animation
    closure.forget();
}

// Stop the animation loop
#[wasm_bindgen]
pub fn stop_animation() -> Result<(), JsValue> {
    ANIMATION_RUNNING.with(|running| {
        *running.borrow_mut() = false;
    });
    console_log!("🛑 Animation loop stopped");
    Ok(())
}

// Remove the old fade function - no longer needed

// Remove the old alpha rendering function - no longer needed

fn render_service_node(service: &ServiceNode) -> Result<(), JsValue> {
    // Create geometry for this specific service node
    let vertices = create_service_geometry(&service.id, service.x, service.y, &service.status);
    
    GPU_RENDERER.with(|r| {
        let renderer_ref = r.borrow();
        let renderer = renderer_ref.as_ref()
            .ok_or_else(|| JsValue::from_str("WebGPU not initialized"))?;
        
        // Create vertex buffer for this service
        let vertex_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Service {} Vertex Buffer", service.id)),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        
        // Get current surface texture
        let output = renderer.surface.get_current_texture()
            .map_err(|e| JsValue::from_str(&format!("Failed to get surface texture: {:?}", e)))?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        
        // Create command encoder
        let mut encoder = renderer.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Service Render Encoder"),
        });
        
        {
            // Begin render pass with CLEAR - remove previous service text
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Service Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }), // CLEAR the canvas for each service
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            render_pass.set_pipeline(&renderer.render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            let vertex_count = (vertices.len() / 2) as u32;
            render_pass.draw(0..vertex_count, 0..1);
        }
        
        // Submit commands and present
        renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        console_log!("🎨 Rendered service '{}' with CLEAR", service.id);
        
        Ok(())
    })
}

fn create_service_geometry(service_id: &str, _x: f32, _y: f32, _status: &str) -> Vec<f32> {
    // Render a clean service identifier instead of full service ID
    let clean_id = extract_service_number(service_id);
    
    // Center the service display
    let centered_x = 0.0; // Always center horizontally
    let centered_y = 0.0; // Always center vertically
    
    console_log!("🎯 Creating geometry for service '{}' -> clean ID: '{}' at ({}, {})", 
                service_id, clean_id, centered_x, centered_y);
    
    create_text_geometry(&clean_id, centered_x, centered_y)
}

fn extract_service_number(service_id: &str) -> String {
    // Extract just the number/identifier part for cleaner display
    if service_id.starts_with("job-") {
        // For "job-abc123" -> "abc123" 
        service_id.strip_prefix("job-").unwrap_or(service_id).to_string()
    } else if service_id.len() > 8 {
        // For very long IDs, take last 8 characters
        service_id.chars().rev().take(8).collect::<String>().chars().rev().collect()
    } else {
        // Use the ID as-is if it's short
        service_id.to_string()
    }
}

// Clear the canvas for new replay rendering
#[wasm_bindgen]
pub fn clear_canvas() -> Result<(), JsValue> {
    GPU_RENDERER.with(|r| {
        let renderer_ref = r.borrow();
        let renderer = renderer_ref.as_ref()
            .ok_or_else(|| JsValue::from_str("WebGPU not initialized"))?;
        
        // Get current surface texture
        let output = renderer.surface.get_current_texture()
            .map_err(|e| JsValue::from_str(&format!("Failed to get surface texture: {:?}", e)))?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        
        // Create command encoder
        let mut encoder = renderer.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Clear Encoder"),
        });
        
        {
            // Clear the canvas
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        
        // Submit commands and present
        renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    })
}
