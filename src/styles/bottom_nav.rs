pub struct BottomNavStyles;
pub const BOTTOM_NAV: BottomNavStyles = BottomNavStyles;

pub struct BottomNavEntryStyles {
	pub item: String,
	pub label: String,
}

impl BottomNavStyles {
	pub fn container(&self) -> &'static str {
		"position: absolute; bottom: 0; left: 0; right: 0; background: #0f172a; flex-direction: row; justify-content: space-around; padding-top: 8; padding-bottom: 12; border-top: 1px solid #334155; shadow: 0 -2 12 rgba(2,6,23,0.30)"
	}

	pub fn item(&self) -> &'static str {
		"align-items: center; padding-vertical: 8; padding-horizontal: 10; border-radius: 8"
	}

	pub fn label(&self) -> &'static str {
		"font-size: 10; color: #94a3b8; margin-top: 4"
	}

	pub fn entry_for(&self, active_screen: crate::Screen, screen: crate::Screen) -> BottomNavEntryStyles {
		BottomNavEntryStyles {
			item: self.item_for(active_screen, screen),
			label: self.label_for(active_screen, screen),
		}
	}

	pub fn item_for(&self, active_screen: crate::Screen, screen: crate::Screen) -> String {
		let base = self.item();
		if active_screen == screen {
			match screen {
				crate::Screen::Login => format!("{}; {}", base, self.login_item_active()),
				crate::Screen::Cadastro => format!("{}; {}", base, self.cadastro_item_active()),
			}
		} else {
			base.to_string()
		}
	}

	pub fn label_for(&self, active_screen: crate::Screen, screen: crate::Screen) -> String {
		let base = self.label();
		if active_screen == screen {
			match screen {
				crate::Screen::Login => format!("{}; {}", base, self.login_label_active()),
				crate::Screen::Cadastro => format!("{}; {}", base, self.cadastro_label_active()),
			}
		} else {
			base.to_string()
		}
	}

	fn login_item_active(&self) -> &'static str {
		"background: #1e3a8a"
	}

	fn login_label_active(&self) -> &'static str {
		"color: #60a5fa; font-weight: 600"
	}

	fn cadastro_item_active(&self) -> &'static str {
		"background: #0c4a6e"
	}

	fn cadastro_label_active(&self) -> &'static str {
		"color: #38bdf8; font-weight: 600"
	}
}
