/// Módulo interno de WebAssembly - não editar
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
pub fn setup_web_preview(app: &oxidact_core::VNode, screen: crate::Screen) {
    let tabs = web_tabs(screen);

    oxidact_core::render_web_preview(
        app,
        oxidact_core::WebPreviewOptions {
            tabs: &tabs,
            show_tree: true,
        },
    );
}

#[cfg(target_arch = "wasm32")]
fn web_tabs(screen: crate::Screen) -> [oxidact_core::WebPreviewTab<'static>; 2] {
    [
        oxidact_core::WebPreviewTab {
            label: "Tela Login",
            href: crate::navigation::screen_query(crate::Screen::Login),
            active: screen == crate::Screen::Login,
            active_color: "#2563eb",
        },
        oxidact_core::WebPreviewTab {
            label: "Tela Cadastro",
            href: crate::navigation::screen_query(crate::Screen::Cadastro),
            active: screen == crate::Screen::Cadastro,
            active_color: "#0ea5e9",
        },
    ]
}

#[cfg(target_arch = "wasm32")]
pub fn get_selected_screen() -> crate::Screen {
    if let Some(window) = web_sys::window() {
        if let Ok(search) = window.location().search() {
            return crate::navigation::screen_from_search(&search);
        }
    }

    crate::Screen::Login
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_selected_screen() -> crate::Screen {
    crate::Screen::Login
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    if let Some(window) = web_sys::window() {
        let rerender = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
            |_event: web_sys::Event| {
                crate::run_app();
            },
        ));

        let _ = window.add_event_listener_with_callback(
            "oxidact:navigate",
            rerender.as_ref().unchecked_ref(),
        );
        let _ = window.add_event_listener_with_callback(
            "popstate",
            rerender.as_ref().unchecked_ref(),
        );
        rerender.forget();
    }

    crate::run_app();
}
