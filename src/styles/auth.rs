pub struct AuthStyles;
pub const AUTH: AuthStyles = AuthStyles;

pub struct LoginStyles;
pub const LOGIN: LoginStyles = LoginStyles;

pub struct CadastroStyles;
pub const CADASTRO: CadastroStyles = CadastroStyles;

impl AuthStyles {
	pub fn content(&self) -> &'static str {
		"padding: 20; justify-content: center; padding-bottom: 96"
	}
}

impl LoginStyles {
	pub fn container(&self) -> &'static str {
		"flex: 1; background: #121212; position: relative"
	}

	pub fn title(&self) -> &'static str {
		"font-size: 24; color: #FFFFFF"
	}

	pub fn subtitle(&self) -> &'static str {
		"color: #94a3b8; margin-top: 8"
	}

	pub fn email_input(&self) -> &'static str {
		"border: 1px solid #333; margin-top: 16; padding: 12; radius: 10; color: #e2e8f0"
	}

	pub fn password_input(&self) -> &'static str {
		"border: 1px solid #333; margin-top: 10; padding: 12; radius: 10; color: #e2e8f0"
	}

	pub fn submit_button(&self) -> &'static str {
		"background: #007AFF; border-radius: 8; margin-top: 20; padding: 12"
	}

	pub fn submit_label(&self) -> &'static str {
		"color: #ffffff"
	}
}

impl CadastroStyles {
	pub fn container(&self) -> &'static str {
		"flex: 1; background: #0b1220; position: relative"
	}

	pub fn title(&self) -> &'static str {
		"font-size: 24; color: #FFFFFF"
	}

	pub fn subtitle(&self) -> &'static str {
		"color: #94a3b8; margin-top: 8"
	}

	pub fn name_input(&self) -> &'static str {
		"border: 1px solid #334155; margin-top: 16; padding: 12; radius: 10; color: #e2e8f0"
	}

	pub fn email_input(&self) -> &'static str {
		"border: 1px solid #334155; margin-top: 10; padding: 12; radius: 10; color: #e2e8f0"
	}

	pub fn password_input(&self) -> &'static str {
		"border: 1px solid #334155; margin-top: 10; padding: 12; radius: 10; color: #e2e8f0"
	}

	pub fn submit_button(&self) -> &'static str {
		"background: #0ea5e9; border-radius: 8; margin-top: 20; padding: 12"
	}

	pub fn submit_label(&self) -> &'static str {
		"color: #ffffff"
	}
}
