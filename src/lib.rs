pub fn build_app() -> oxidact_core::VNode {
    let mut root = oxidact_core::VNode::new(oxidact_core::NodeType::SafeAreaView);
    root.style_raw = "flex: 1; bg: #111111".to_string();

    let mut container = oxidact_core::VNode::new(oxidact_core::NodeType::View);
    container.style_raw = "padding: 20".to_string();

    let mut title = oxidact_core::VNode::new(oxidact_core::NodeType::Text);
    title.text_content = Some("Bem-vindo ao Oxidact (Web + Native)".to_string());

    let mut section = oxidact_core::VNode::new(oxidact_core::NodeType::View);
    section.style_raw = "margin-top: 50".to_string();

    let mut email_label = oxidact_core::VNode::new(oxidact_core::NodeType::Text);
    email_label.text_content = Some("Email:".to_string());

    let mut button = oxidact_core::VNode::new(oxidact_core::NodeType::Pressable);
    button.style_raw = "bg: #333; radius: 8".to_string();

    let mut button_text = oxidact_core::VNode::new(oxidact_core::NodeType::Text);
    button_text.text_content = Some("Clique para entrar".to_string());

    button.children.push(button_text);
    section.children.push(email_label);
    section.children.push(button);
    container.children.push(title);
    container.children.push(section);
    root.children.push(container);

    root
}

pub fn run_app() {
    oxidact_core::run(build_app());
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    run_app();
}
