use winit::{
    event::Event,
    event::WindowEvent,
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    View,
    Text,
    TextInput,
    Pressable,
    SafeAreaView,
}

impl Default for NodeType {
    fn default() -> Self {
        Self::View
    }
}

#[derive(Debug, Clone, Default)]
pub struct VNode {
    pub tag: NodeType,
    pub children: Vec<VNode>,
    pub text_content: Option<String>,
    pub style_raw: String,
}

impl VNode {
    pub fn new(tag: NodeType) -> Self {
        Self {
            tag,
            children: Vec::new(),
            text_content: None,
            style_raw: String::new(),
        }
    }
}

pub fn run(root: VNode) {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(err) = run_async(root).await {
                web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&err));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(err) = pollster::block_on(run_async(root)) {
            eprintln!("Oxidact render error: {err}");
        }
    }
}

async fn run_async(root: VNode) -> Result<(), String> {
    let event_loop = EventLoop::new().map_err(|e| format!("event loop: {e}"))?;
    let window = WindowBuilder::new()
        .with_title("Oxidact Engine")
        .build(&event_loop)
        .map_err(|e| format!("window build: {e}"))?;

    #[cfg(target_arch = "wasm32")]
    {
        attach_canvas_to_body(&window)?;
    }

    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(&window)
        .map_err(|e| format!("create surface: {e}"))?;

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| "no suitable GPU adapter found".to_string())?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .map_err(|e| format!("request device: {e}"))?;

    let size = window.inner_size();
    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .first()
        .copied()
        .ok_or_else(|| "surface has no supported formats".to_string())?;

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let clear = parse_background_color(&root.style_raw);

    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => target.exit(),
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    config.width = new_size.width;
                    config.height = new_size.height;
                    surface.configure(&device, &config);
                }
            }
            Event::AboutToWait => {
                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        surface.configure(&device, &config);
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        target.exit();
                        return;
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        return;
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("oxidact-clear-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => {}
        })
        .map_err(|e| format!("event loop: {e}"))
}

fn parse_background_color(style: &str) -> wgpu::Color {
    let default = wgpu::Color {
        r: 0.06,
        g: 0.06,
        b: 0.08,
        a: 1.0,
    };

    let value = style
        .split(';')
        .map(str::trim)
        .find_map(|entry| {
            entry
                .strip_prefix("bg:")
                .or_else(|| entry.strip_prefix("background:"))
                .map(str::trim)
        });

    let Some(value) = value else {
        return default;
    };

    let Some(hex) = value.strip_prefix('#') else {
        return default;
    };

    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok();
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok();
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok();
            match (r, g, b) {
                (Some(r), Some(g), Some(b)) => wgpu::Color {
                    r: (r as f64) / 255.0,
                    g: (g as f64) / 255.0,
                    b: (b as f64) / 255.0,
                    a: 1.0,
                },
                _ => default,
            }
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok();
            let g = u8::from_str_radix(&hex[2..4], 16).ok();
            let b = u8::from_str_radix(&hex[4..6], 16).ok();
            match (r, g, b) {
                (Some(r), Some(g), Some(b)) => wgpu::Color {
                    r: (r as f64) / 255.0,
                    g: (g as f64) / 255.0,
                    b: (b as f64) / 255.0,
                    a: 1.0,
                },
                _ => default,
            }
        }
        _ => default,
    }
}

#[cfg(target_arch = "wasm32")]
fn attach_canvas_to_body(window: &winit::window::Window) -> Result<(), String> {
    let document = web_sys::window()
        .ok_or_else(|| "window not available".to_string())?
        .document()
        .ok_or_else(|| "document not available".to_string())?;

    let body = document
        .body()
        .ok_or_else(|| "document.body not available".to_string())?;

    let canvas = window
        .canvas()
        .ok_or_else(|| "window canvas not available".to_string())?;
    body.append_child(&canvas)
        .map_err(|_| "failed to append canvas to body".to_string())?;

    Ok(())
}
