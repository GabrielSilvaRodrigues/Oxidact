use oxidact_macros::rsx;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Login,
    Cadastro,
}

pub fn build_stack_navigator(active_screen: Screen) -> oxidact_core::VNode {
    rsx!(
        <NavigationContainer style={"flex: 1"}>
            <StackNavigator
                initialRouteName={initial_route_name()}
                activeRoute={route_name(active_screen)}
                style={"flex: 1"}
            >
                <StackScreen name="Login" route="login" headerShown="false">
                    {crate::screens::build_login_screen()}
                </StackScreen>
                <StackScreen name="Cadastro" route="cadastro" headerShown="false">
                    {crate::screens::build_cadastro_screen()}
                </StackScreen>
            </StackNavigator>
        </NavigationContainer>
    )
}

#[cfg(target_arch = "wasm32")]
pub fn screen_from_search(search: &str) -> Screen {
    if search.to_lowercase().contains("screen=cadastro") {
        Screen::Cadastro
    } else {
        Screen::Login
    }
}

#[cfg(target_arch = "wasm32")]
pub fn screen_query(screen: Screen) -> &'static str {
    match screen {
        Screen::Login => "?screen=login",
        Screen::Cadastro => "?screen=cadastro",
    }
}

pub fn initial_route_name() -> &'static str {
    "Login"
}

pub fn route_name(screen: Screen) -> &'static str {
    match screen {
        Screen::Login => "Login",
        Screen::Cadastro => "Cadastro",
    }
    Perfil,
}
