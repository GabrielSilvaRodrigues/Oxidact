use std::fmt::Write;
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;

use winit::{
    event::Event,
    event::WindowEvent,
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::{EventLoopExtWebSys, WindowExtWebSys};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::{Event as WebEvent, HtmlInputElement};

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_INPUT_VALUES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

#[cfg(target_arch = "wasm32")]
pub fn web_input_value(key: &str) -> Option<String> {
    WEB_INPUT_VALUES.with(|state| state.borrow().get(key).cloned())
}

#[cfg(target_arch = "wasm32")]
fn set_web_input_value(key: &str, value: &str) {
    WEB_INPUT_VALUES.with(|state| {
        state
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
    });
}

#[cfg(target_arch = "wasm32")]
fn clear_web_input_values() {
    WEB_INPUT_VALUES.with(|state| state.borrow_mut().clear());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    View,
    Text,
    TextInput,
    Pressable,
    SafeAreaView,
    NavigationContainer,
    StackNavigator,
    StackScreen,
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
    pub attributes: Vec<(String, String)>,
}

impl VNode {
    pub fn new(tag: NodeType) -> Self {
        Self {
            tag,
            children: Vec::new(),
            text_content: None,
            style_raw: String::new(),
            attributes: Vec::new(),
        }
    }

    pub fn set_attr(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        if let Some((_, existing)) = self.attributes.iter_mut().find(|(k, _)| *k == key) {
            *existing = value.into();
            return;
        }
        self.attributes.push((key, value.into()));
    }

    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find_map(|(k, v)| if k == key { Some(v.as_str()) } else { None })
    }
}

pub fn tree_text(root: &VNode) -> String {
    let mut out = String::new();
    format_node(root, 0, &mut out);
    out.trim_end().to_string()
}

fn format_node(node: &VNode, indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    let _ = writeln!(out, "{}{:?} (style: {})", pad, node.tag, node.style_raw);

    for (key, value) in &node.attributes {
        let _ = writeln!(out, "{}  attr {}=\"{}\"", pad, key, value);
    }

    if let Some(text) = &node.text_content {
        let _ = writeln!(out, "{}  text: \"{}\"", pad, text);
    }

    for child in &node.children {
        format_node(child, indent + 2, out);
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
    let window = Arc::new(
        WindowBuilder::new()
        .with_title("Oxidact Engine")
        .build(&event_loop)
        .map_err(|e| format!("window build: {e}"))?,
    );

    #[cfg(target_arch = "wasm32")]
    {
        attach_canvas_to_body(&window)?;
        if !browser_supports_webgl2() {
            show_web_fallback(&root.style_raw, "WebGL2 nao disponivel neste navegador.");
            return Ok(());
        }
    }

    #[cfg(target_arch = "wasm32")]
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::GL,
        ..Default::default()
    });

    #[cfg(not(target_arch = "wasm32"))]
    let instance = wgpu::Instance::default();
    let surface = match instance.create_surface(window.clone()) {
        Ok(surface) => surface,
        Err(e) => {
            #[cfg(target_arch = "wasm32")]
            {
                let msg = format!("Renderer indisponivel: {e}");
                show_web_fallback(&root.style_raw, &msg);
                return Ok(());
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                return Err(format!("create surface: {e}"));
            }
        }
    };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| "no suitable GPU adapter found".to_string())?;

    let required_limits = if cfg!(target_arch = "wasm32") {
        // Browsers podem rejeitar alguns limites mais novos; usar baseline WebGL2.
        wgpu::Limits::downlevel_webgl2_defaults()
    } else {
        wgpu::Limits::default()
    };

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("oxidact-device"),
                required_features: wgpu::Features::empty(),
                required_limits,
            },
            None,
        )
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

    let handle_event = move |event, target: &winit::event_loop::EventLoopWindowTarget<()>| {
        match event {
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
        }
    };

    #[cfg(target_arch = "wasm32")]
    {
        event_loop.spawn(handle_event);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        event_loop
            .run(handle_event)
            .map_err(|e| format!("event loop: {e}"))
    }
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

#[cfg(target_arch = "wasm32")]
fn browser_supports_webgl2() -> bool {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return false;
    };

    let Ok(temp_canvas) = document.create_element("canvas") else {
        return false;
    };

    let Ok(temp_canvas) = temp_canvas.dyn_into::<web_sys::HtmlCanvasElement>() else {
        return false;
    };

    temp_canvas
        .get_context("webgl2")
        .ok()
        .flatten()
        .is_some()
}

#[cfg(target_arch = "wasm32")]
fn show_web_fallback(style_raw: &str, message: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };

    let bg = extract_bg_hex(style_raw).unwrap_or_else(|| "#111111".to_string());
    let _ = body.set_attribute(
        "style",
        &format!("margin:0;background:{bg};color:#e5e7eb;font-family:monospace;"),
    );

    let Ok(div) = document.create_element("div") else {
        return;
    };
    div.set_text_content(Some(message));
    let _ = div.set_attribute(
        "style",
        "position:fixed;left:12px;bottom:12px;z-index:9999;padding:10px 12px;border-radius:8px;background:rgba(0,0,0,0.65);color:#f5f5f5;font:12px/1.4 monospace;",
    );
    let _ = body.append_child(&div);
}

#[cfg(target_arch = "wasm32")]
fn extract_bg_hex(style: &str) -> Option<String> {
    style
        .split(';')
        .map(str::trim)
        .find_map(|entry| {
            entry
                .strip_prefix("bg:")
                .or_else(|| entry.strip_prefix("background:"))
                .map(str::trim)
        })
        .map(|s| s.to_string())
}

#[cfg(target_arch = "wasm32")]
pub struct WebPreviewTab<'a> {
    pub label: &'a str,
    pub href: &'a str,
    pub active: bool,
    pub active_color: &'a str,
}

#[cfg(target_arch = "wasm32")]
pub struct WebPreviewOptions<'a> {
    pub tabs: &'a [WebPreviewTab<'a>],
    pub show_tree: bool,
}

#[cfg(target_arch = "wasm32")]
pub fn render_web_preview(root: &VNode, options: WebPreviewOptions<'_>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };

    let tree = tree_text(root);
    clear_web_input_values();

    body.set_inner_html("");
    let _ = body.set_attribute(
        "style",
        "margin:0;min-height:100vh;background:radial-gradient(1200px 800px at 10% 10%, #1f2937 0%, #0b1220 60%, #030712 100%);color:#e2e8f0;font-family:'Segoe UI',sans-serif;",
    );

    let Ok(shell) = document.create_element("div") else {
        return;
    };
    let _ = shell.set_attribute("style", "max-width:760px;margin:28px auto;padding:20px;");

    if !options.tabs.is_empty() {
        let Ok(tabs) = document.create_element("div") else {
            return;
        };
        let _ = tabs.set_attribute("style", "display:flex;gap:10px;margin-bottom:14px;");

        for tab in options.tabs {
            let Ok(link) = document.create_element("a") else {
                continue;
            };
            let bg = if tab.active { tab.active_color } else { "#1f2937" };
            link.set_text_content(Some(tab.label));
            let _ = link.set_attribute("href", tab.href);
            let _ = link.set_attribute(
                "style",
                &format!("text-decoration:none;color:#fff;background:{};padding:8px 12px;border-radius:999px;font-weight:600;font-size:13px;", bg),
            );
            let _ = tabs.append_child(&link);
        }

        let _ = shell.append_child(&tabs);
    }

    let Ok(card) = document.create_element("div") else {
        return;
    };
    let _ = card.set_attribute(
        "style",
        "background:rgba(15,23,42,0.88);border:1px solid rgba(148,163,184,0.22);box-shadow:0 18px 40px rgba(2,6,23,0.5);backdrop-filter:blur(8px);border-radius:18px;padding:14px;",
    );

    if let Some(root_el) = vnode_to_dom(&document, root) {
        let _ = card.append_child(&root_el);
    }
    let _ = shell.append_child(&card);

    if options.show_tree {
        let Ok(debug_pre) = document.create_element("pre") else {
            return;
        };
        debug_pre.set_text_content(Some(&tree));
        let _ = debug_pre.set_attribute(
            "style",
            "margin:14px 0 0 0;white-space:pre;line-height:1.35;font-size:12px;color:#cbd5e1;background:rgba(2,6,23,0.65);padding:12px;border-radius:12px;overflow:auto;",
        );
        let _ = shell.append_child(&debug_pre);
    }

    let _ = body.append_child(&shell);
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&tree));
}

#[cfg(target_arch = "wasm32")]
fn vnode_to_dom(document: &web_sys::Document, node: &VNode) -> Option<web_sys::Element> {
    let tag = match node.tag {
        NodeType::SafeAreaView
        | NodeType::View
        | NodeType::NavigationContainer
        | NodeType::StackNavigator
        | NodeType::StackScreen => "div",
        NodeType::Text => "div",
        NodeType::Pressable => "button",
        NodeType::TextInput => "input",
    };

    let Ok(el) = document.create_element(tag) else {
        return None;
    };

    let mut style = default_style_for_node(&node.tag);
    style.push_str(&style_to_css(&node.style_raw));
    let _ = el.set_attribute("style", &style);

    if let Some(placeholder) = node.attr("placeholder") {
        if node.tag == NodeType::TextInput {
            let _ = el.set_attribute("placeholder", placeholder);
        } else {
            let _ = el.set_attribute("data-placeholder", placeholder);
        }
    }

    if let Some(action) = node
        .attr("onclick")
        .map(|s| s.to_string())
        .or_else(|| node.attr("to").map(|route| format!("navigate:{route}")))
    {
        let _ = el.set_attribute("data-onclick", &action);
        let click_cb = Closure::<dyn FnMut(WebEvent)>::wrap(Box::new(move |_event: WebEvent| {
            handle_web_action(&action);
        }));
        let _ = el.add_event_listener_with_callback("click", click_cb.as_ref().unchecked_ref());
        click_cb.forget();
    }

    if node.tag == NodeType::TextInput {
        let Ok(input) = el.clone().dyn_into::<HtmlInputElement>() else {
            return Some(el);
        };

        let input_type = if matches!(node.attr("secureTextEntry"), Some("true")) {
            "password"
        } else {
            "text"
        };
        input.set_type(input_type);

        if let Some(value) = node.attr("value") {
            input.set_value(value);
        }

        if let Some(key) = input_state_key(node) {
            let initial = input.value();
            set_web_input_value(&key, &initial);

            let change_handler = node.attr("onChangeText").map(|s| s.to_string());
            let key_for_cb = key.clone();
            let input_cb = Closure::<dyn FnMut(WebEvent)>::wrap(Box::new(move |event: WebEvent| {
                let Some(target) = event.target() else {
                    return;
                };
                let Ok(target_input) = target.dyn_into::<HtmlInputElement>() else {
                    return;
                };
                let value = target_input.value();
                set_web_input_value(&key_for_cb, &value);

                if let Some(handler) = &change_handler {
                    let msg = format!("{} => {}={}", handler, key_for_cb, value);
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
                }
            }));
            let _ = input.add_event_listener_with_callback("input", input_cb.as_ref().unchecked_ref());
            input_cb.forget();
        }
    }

    if let Some(text) = &node.text_content {
        el.set_text_content(Some(text));
    }

    if node.tag == NodeType::StackNavigator {
        let selected = node
            .attr("activeRoute")
            .or_else(|| node.attr("initialRouteName"));

        if let Some(route_name) = selected {
            let active_screen = node.children.iter().find(|child| {
                child.tag == NodeType::StackScreen && child.attr("name") == Some(route_name)
            });

            if let Some(active_screen) = active_screen {
                for child in &active_screen.children {
                    if let Some(child_el) = vnode_to_dom(document, child) {
                        let _ = el.append_child(&child_el);
                    }
                }

                return Some(el);
            }
        }
    }

    for child in &node.children {
        if let Some(child_el) = vnode_to_dom(document, child) {
            let _ = el.append_child(&child_el);
        }
    }

    Some(el)
}

#[cfg(target_arch = "wasm32")]
fn input_state_key(node: &VNode) -> Option<String> {
    node.attr("testID")
        .or_else(|| node.attr("name"))
        .or_else(|| node.attr("id"))
        .map(|s| s.to_string())
}

#[cfg(target_arch = "wasm32")]
fn handle_web_action(action: &str) {
    if let Some(screen) = action.strip_prefix("navigate:") {
        if let Some(window) = web_sys::window() {
            let target = format!("?screen={screen}");
            if let Ok(history) = window.history() {
                let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&target));
                if let Ok(event) = web_sys::Event::new("oxidact:navigate") {
                    let _ = window.dispatch_event(&event);
                }
                return;
            }

            let _ = window.location().set_search(&target);
        }
        return;
    }

    if action == "handle_login" {
        let email = web_input_value("login_email").unwrap_or_default();
        let password = web_input_value("login_password").unwrap_or_default();
        let msg = format!("handle_login => email={}, senha_chars={}", email, password.chars().count());
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
        return;
    }

    if action == "handle_signup" {
        let name = web_input_value("signup_name").unwrap_or_default();
        let email = web_input_value("signup_email").unwrap_or_default();
        let password = web_input_value("signup_password").unwrap_or_default();
        let msg = format!(
            "handle_signup => nome={}, email={}, senha_chars={}",
            name,
            email,
            password.chars().count()
        );
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
    }
}

#[cfg(target_arch = "wasm32")]
fn default_style_for_node(tag: &NodeType) -> String {
    match tag {
        NodeType::SafeAreaView => {
            "display:flex;flex-direction:column;min-height:560px;border-radius:14px;".to_string()
        }
        NodeType::View => "display:flex;flex-direction:column;".to_string(),
        NodeType::NavigationContainer => "display:flex;flex-direction:column;".to_string(),
        NodeType::StackNavigator => "display:flex;flex-direction:column;".to_string(),
        NodeType::StackScreen => "display:flex;flex-direction:column;".to_string(),
        NodeType::Text => "margin:0 0 10px 0;".to_string(),
        NodeType::Pressable => {
            "border:none;cursor:pointer;text-align:left;font:inherit;".to_string()
        }
        NodeType::TextInput => {
            "display:block;background:#0f172a;color:#e2e8f0;outline:none;box-sizing:border-box;width:100%;"
                .to_string()
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn style_to_css(style_raw: &str) -> String {
    let mut css = String::new();
    let mut border_top_width: Option<String> = None;
    let mut border_top_color: Option<String> = None;

    for item in style_raw.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let mut parts = item.splitn(2, ':');
        let Some(key) = parts.next().map(str::trim) else {
            continue;
        };
        let Some(value_raw) = parts.next().map(str::trim) else {
            continue;
        };

        let value = css_value(value_raw);
        match key {
            "bg" | "background" => {
                css.push_str("background:");
                css.push_str(&value);
                css.push(';');
            }
            "padding" => {
                css.push_str("padding:");
                css.push_str(&value);
                css.push(';');
            }
            "padding-top" => {
                css.push_str("padding-top:");
                css.push_str(&value);
                css.push(';');
            }
            "padding-bottom" => {
                css.push_str("padding-bottom:");
                css.push_str(&value);
                css.push(';');
            }
            "padding-left" => {
                css.push_str("padding-left:");
                css.push_str(&value);
                css.push(';');
            }
            "padding-right" => {
                css.push_str("padding-right:");
                css.push_str(&value);
                css.push(';');
            }
            "padding-vertical" => {
                css.push_str("padding-top:");
                css.push_str(&value);
                css.push(';');
                css.push_str("padding-bottom:");
                css.push_str(&value);
                css.push(';');
            }
            "padding-horizontal" => {
                css.push_str("padding-left:");
                css.push_str(&value);
                css.push(';');
                css.push_str("padding-right:");
                css.push_str(&value);
                css.push(';');
            }
            "margin" => {
                css.push_str("margin:");
                css.push_str(&value);
                css.push(';');
            }
            "margin-top" => {
                css.push_str("margin-top:");
                css.push_str(&value);
                css.push(';');
            }
            "margin-bottom" => {
                css.push_str("margin-bottom:");
                css.push_str(&value);
                css.push(';');
            }
            "radius" | "border-radius" => {
                css.push_str("border-radius:");
                css.push_str(&value);
                css.push(';');
            }
            "color" => {
                css.push_str("color:");
                css.push_str(&value);
                css.push(';');
            }
            "font-size" => {
                css.push_str("font-size:");
                css.push_str(&value);
                css.push(';');
            }
            "border" => {
                css.push_str("border:");
                css.push_str(value_raw);
                css.push(';');
            }
            "border-top" => {
                css.push_str("border-top:");
                css.push_str(value_raw);
                css.push(';');
            }
            "border-top-width" => {
                border_top_width = Some(value);
            }
            "border-top-color" => {
                border_top_color = Some(value_raw.to_string());
            }
            "flex" => {
                if value_raw == "1" {
                    css.push_str("flex:1;");
                }
            }
            "position" => {
                css.push_str("position:");
                css.push_str(value_raw);
                css.push(';');
            }
            "left" => {
                css.push_str("left:");
                css.push_str(&value);
                css.push(';');
            }
            "right" => {
                css.push_str("right:");
                css.push_str(&value);
                css.push(';');
            }
            "top" => {
                css.push_str("top:");
                css.push_str(&value);
                css.push(';');
            }
            "bottom" => {
                css.push_str("bottom:");
                css.push_str(&value);
                css.push(';');
            }
            "width" => {
                css.push_str("width:");
                css.push_str(&value);
                css.push(';');
            }
            "height" => {
                css.push_str("height:");
                css.push_str(&value);
                css.push(';');
            }
            "flex-direction" => {
                css.push_str("flex-direction:");
                css.push_str(value_raw);
                css.push(';');
            }
            "justify-content" => {
                css.push_str("justify-content:");
                css.push_str(value_raw);
                css.push(';');
            }
            "align-items" => {
                css.push_str("align-items:");
                css.push_str(value_raw);
                css.push(';');
            }
            "font-weight" => {
                css.push_str("font-weight:");
                css.push_str(value_raw);
                css.push(';');
            }
            "shadow" => {
                let shadow_css = value_raw.replace(' ', "px ");
                css.push_str("box-shadow:");
                css.push_str(&shadow_css);
                css.push(';');
            }
            _ => {}
        }
    }

    if border_top_width.is_some() || border_top_color.is_some() {
        css.push_str("border-top-style:solid;");
    }
    if let Some(width) = border_top_width {
        css.push_str("border-top-width:");
        css.push_str(&width);
        css.push(';');
    }
    if let Some(color) = border_top_color {
        css.push_str("border-top-color:");
        css.push_str(&color);
        css.push(';');
    }

    css
}

#[cfg(target_arch = "wasm32")]
fn css_value(value: &str) -> String {
    if value.chars().all(|c| c.is_ascii_digit()) {
        return format!("{}px", value);
    }
    value.to_string()
}
