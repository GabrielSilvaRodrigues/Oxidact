use oxidact_macros::rsx;

pub fn build_perfil_screen() -> oxidact_core::VNode {
rsx!(
<SafeAreaView style={"flex: 1; background: #0f172a"}>
<View style={"padding: 20; justify-content: center"}>
<Text style={"font-size: 24; color: #ffffff"}>"Perfil"</Text>
</View>
</SafeAreaView>
)
}
