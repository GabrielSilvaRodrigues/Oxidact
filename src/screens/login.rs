use oxidact_macros::rsx;
use crate::styles::auth::{AUTH, LOGIN};

pub fn build_login_screen() -> oxidact_core::VNode {
    let mut screen = rsx!(
        <SafeAreaView style={LOGIN.container()}>
            <View style={AUTH.content()}>
                <Text style={LOGIN.title()}>"Login"</Text>
                <Text style={LOGIN.subtitle()}>"Entre para continuar"</Text>

                <TextInput
                    testID="login_email"
                    placeholder="Seu e-mail"
                    onChangeText="set_login_email"
                    style={LOGIN.email_input()}
                />

                <TextInput
                    testID="login_password"
                    placeholder="Senha"
                    secureTextEntry="true"
                    onChangeText="set_login_password"
                    style={LOGIN.password_input()}
                />

                <Pressable
                    onclick="handle_login"
                    style={LOGIN.submit_button()}
                >
                    <Text style={LOGIN.submit_label()}>"Entrar"</Text>
                </Pressable>
            </View>
        </SafeAreaView>
    );

    screen
        .children
        .push(crate::components::bottom_nav::build_bottom_nav(crate::Screen::Login));
    screen
}
