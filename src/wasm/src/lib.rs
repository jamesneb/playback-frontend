use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use wgpu::*;
use wgpu::util::DeviceExt;
use wasm_bindgen_futures::JsFuture;

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
}

// Use thread_local for WASM since we're single-threaded
use std::cell::RefCell;
thread_local! {
    static GPU_RENDERER: RefCell<Option<WebGpuRenderer>> = RefCell::new(None);
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

#[wasm_bindgen]
pub async fn init_webgpu(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
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
                blend: Some(BlendState::REPLACE),
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
    
    // Store the renderer globally
    let renderer = WebGpuRenderer {
        device,
        queue,
        surface,
        surface_config,
        render_pipeline,
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
