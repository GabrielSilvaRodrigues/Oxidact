use oxidact_macros::rsx;

pub fn build_login_screen() -> oxidact_core::VNode {
    rsx!(
        <SafeAreaView style="flex: 1; background: #121212">
            <View style="padding: 20; justify-content: center">
                <Text style="font-size: 24; color: #FFFFFF">"Login"</Text>
                <Text style="color: #94a3b8; margin-top: 8">"Entre para continuar"</Text>

                <TextInput
                    placeholder="Seu e-mail"
                    style="border: 1px solid #333; margin-top: 16; padding: 12; radius: 10; color: #e2e8f0"
                />

                <TextInput
                    placeholder="Senha"
                    style="border: 1px solid #333; margin-top: 10; padding: 12; radius: 10; color: #e2e8f0"
                />

                <Pressable
                    onclick="handle_login"
                    style="background: #007AFF; border-radius: 8; margin-top: 20; padding: 12"
                >
                    <Text style="color: #ffffff">"Entrar"</Text>
                </Pressable>
            </View>
        </SafeAreaView>
    )
}

pub fn build_cadastro_screen() -> oxidact_core::VNode {
    rsx!(
        <SafeAreaView style="flex: 1; background: #0b1220">
            <View style="padding: 20; justify-content: center">
                <Text style="font-size: 24; color: #FFFFFF">"Criar conta"</Text>
                <Text style="color: #94a3b8; margin-top: 8">"Leva menos de 1 minuto"</Text>

                <TextInput
                    placeholder="Nome completo"
                    style="border: 1px solid #334155; margin-top: 16; padding: 12; radius: 10; color: #e2e8f0"
                />

                <TextInput
                    placeholder="Seu e-mail"
                    style="border: 1px solid #334155; margin-top: 10; padding: 12; radius: 10; color: #e2e8f0"
                />

                <TextInput
                    placeholder="Senha"
                    style="border: 1px solid #334155; margin-top: 10; padding: 12; radius: 10; color: #e2e8f0"
                />

                <Pressable
                    onclick="handle_signup"
                    style="background: #0ea5e9; border-radius: 8; margin-top: 20; padding: 12"
                >
                    <Text style="color: #ffffff">"Criar conta"</Text>
                </Pressable>
            </View>
        </SafeAreaView>
    )
}
