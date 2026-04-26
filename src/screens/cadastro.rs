use oxidact_macros::rsx;
use crate::styles::auth::{AUTH, CADASTRO};

pub fn build_cadastro_screen() -> oxidact_core::VNode {
    let mut screen = rsx!(
        <SafeAreaView style={CADASTRO.container()}>
            <View style={AUTH.content()}>
                <Text style={CADASTRO.title()}>"Criar conta"</Text>
                <Text style={CADASTRO.subtitle()}>"Leva menos de 1 minuto"</Text>

                <TextInput
                    testID="signup_name"
                    placeholder="Nome completo"
                    onChangeText="set_signup_name"
                    style={CADASTRO.name_input()}
                />

                <TextInput
                    testID="signup_email"
                    placeholder="Seu e-mail"
                    onChangeText="set_signup_email"
                    style={CADASTRO.email_input()}
                />

                <TextInput
                    testID="signup_password"
                    placeholder="Senha"
                    secureTextEntry="true"
                    onChangeText="set_signup_password"
                    style={CADASTRO.password_input()}
                />

                <Pressable
                    onclick="handle_signup"
                    style={CADASTRO.submit_button()}
                >
                    <Text style={CADASTRO.submit_label()}>"Criar conta"</Text>
                </Pressable>
            </View>
        </SafeAreaView>
    );

    screen
        .children
        .push(crate::components::bottom_nav::build_bottom_nav(crate::Screen::Cadastro));
    screen
}
