pub fn build_app(screen: crate::navigation::Screen) -> oxidact_core::VNode {
    crate::navigation::build_stack_navigator(screen)
}
