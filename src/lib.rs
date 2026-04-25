mod screens;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Login,
    Cadastro,
}

pub fn build_app(screen: Screen) -> oxidact_core::VNode {
    match screen {
        Screen::Login => screens::build_login_screen(),
        Screen::Cadastro => screens::build_cadastro_screen(),
    }
}

pub fn run_app() {
    let screen = selected_screen();
    let app = build_app(screen);

    #[cfg(target_arch = "wasm32")]
    {
        let tabs = web_tabs(screen);

        oxidact_core::render_web_preview(
            &app,
            oxidact_core::WebPreviewOptions {
                tabs: &tabs,
                show_tree: true,
            },
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let tree_text = oxidact_core::tree_text(&app);
        println!("{tree_text}");
        oxidact_core::run(app);
    }
}

#[cfg(target_arch = "wasm32")]
fn web_tabs(screen: Screen) -> [oxidact_core::WebPreviewTab<'static>; 2] {
    [
        oxidact_core::WebPreviewTab {
            label: "Tela Login",
            href: "?screen=login",
            active: screen == Screen::Login,
            active_color: "#2563eb",
        },
        oxidact_core::WebPreviewTab {
            label: "Tela Cadastro",
            href: "?screen=cadastro",
            active: screen == Screen::Cadastro,
            active_color: "#0ea5e9",
        },
    ]
}

fn selected_screen() -> Screen {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(search) = window.location().search() {
                if search.to_lowercase().contains("screen=cadastro") {
                    return Screen::Cadastro;
                }
            }
        }
    }

    Screen::Login
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    run_app();
}
