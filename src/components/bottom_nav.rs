use oxidact_macros::rsx;
use crate::styles::bottom_nav::BOTTOM_NAV;
use crate::Screen;

pub fn build_bottom_nav(active_screen: Screen) -> oxidact_core::VNode {
    let login_styles = BOTTOM_NAV.entry_for(active_screen, Screen::Login);
    let cadastro_styles = BOTTOM_NAV.entry_for(active_screen, Screen::Cadastro);

    rsx!(
        <View style={BOTTOM_NAV.container()}>
            <Pressable
                to="login"
                style={login_styles.item}
            >
                <Text style={login_styles.label}>"Login"</Text>
            </Pressable>

            <Pressable
                to="cadastro"
                style={cadastro_styles.item}
            >
                <Text style={cadastro_styles.label}>"Cadastro"</Text>
            </Pressable>
        </View>
    )
}
