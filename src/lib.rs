mod app;
mod components;
mod navigation;
mod screens;
mod styles;
mod wasm_internal;

pub use navigation::Screen;

#[derive(Debug, Clone, Default)]
pub struct LoginForm {
    pub tentativas: i32,
    pub email: String,
    pub senha: String,
}

#[derive(Debug, Clone, Default)]
pub struct CadastroForm {
    pub tentativas: i32,
    pub nome: String,
    pub email: String,
    pub senha: String,
}

#[cfg(target_arch = "wasm32")]
pub fn get_login_form(tentativas: i32) -> LoginForm {
    LoginForm {
        tentativas,
        email: oxidact_core::web_input_value("login_email").unwrap_or_default(),
        senha: oxidact_core::web_input_value("login_password").unwrap_or_default(),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_cadastro_form(tentativas: i32) -> CadastroForm {
    CadastroForm {
        tentativas,
        nome: oxidact_core::web_input_value("signup_name").unwrap_or_default(),
        email: oxidact_core::web_input_value("signup_email").unwrap_or_default(),
        senha: oxidact_core::web_input_value("signup_password").unwrap_or_default(),
    }
}

pub fn build_app(screen: Screen) -> oxidact_core::VNode {
    app::build_app(screen)
}

pub fn run_app() {
    let screen = wasm_internal::get_selected_screen();
    let app = build_app(screen);

    #[cfg(target_arch = "wasm32")]
    {
        wasm_internal::setup_web_preview(&app, screen);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let tree_text = oxidact_core::tree_text(&app);
        println!("{tree_text}");
        oxidact_core::run(app);
    }
}
