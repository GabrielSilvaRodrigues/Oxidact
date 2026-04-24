# 🦀 Oxidact

**Oxidact** é um framework de UI nativo e multiplataforma escrito em Rust, projetado para oferecer a experiência de desenvolvimento do React Native com a performance bruta do metal. 

> "Performance de motor de jogo, sintaxe de desenvolvimento web."

---

## 🚀 Por que Oxidact?

O Oxidact foi criado para resolver os gargalos de performance dos frameworks atuais:


| Característica | React Native | Jetpack Compose | **Oxidact** |
| :--- | :--- | :--- | :--- |
| **Linguagem** | JavaScript / C++ | Kotlin (JVM) | **Rust (Nativo)** |
| **Gerenciamento de Memória** | Garbage Collector | Garbage Collector | **Ownership (Zero GC)** |
| **Renderização** | Bridge Nativia | Skia / Canvas | **WGPU (Direct GPU)** |
| **Velocidade de Layout** | Yoga (C++) | Compose Runtime | **Taffy (Rust Nativo)** |

## 🛠️ Arquitetura

O projeto é estruturado como um Cargo Workspace para garantir modularidade e tempos de compilação otimizados:

- **`oxidact-core`**: O motor. Gerencia a árvore de nós (`VNodes`), integração com o motor de layout **Taffy** e renderização via **WGPU**.
- **`oxidact-macros`**: O compilador de UI. Transforma sintaxe RSX (estilo XML) em código Rust tipado e eficiente em tempo de compilação.
- **`src/main.rs`**: O ponto de entrada do desenvolvedor.

## 💻 Exemplo de Uso (Tela de Login)

```rust
use oxidact_macros::rsx;
use oxidact_core;

fn main() {
    let app = rsx!(
        <SafeAreaView style="flex: 1; background: #121212">
            <View style="padding: 20; justify-content: center">
                <Text style="font-size: 24; color: #FFFFFF">"Login"</Text>
                
                <TextInput 
                    placeholder="Seu e-mail" 
                    style="border: 1px solid #333; margin-top: 10" 
                />
                
                <Pressable 
                    onclick="handle_login" 
                    style="background: #007AFF; border-radius: 8; margin-top: 20"
                >
                    <Text>"Entrar"</Text>
                </Pressable>
            </View>
        </SafeAreaView>
    );

    oxidact_core::run(app);
}
```

## 🏗️ Componentes Suportados

O Oxidact implementa as tags fundamentais para interfaces modernas, eliminando redundâncias:

- **Layout:** `View`, `ScrollView`, `SafeAreaView`, `KeyboardAvoidingView`.
- **Conteúdo:** `Text`, `Image`, `ActivityIndicator`.
- **Entrada:** `TextInput`, `Pressable` (Substitui todos os Touchables).
- **Listas:** `FlatList` (Virtualização nativa de alta performance).

## ⚡ Diferenciais Técnicos

1. **Compilação Estática:** Erros de sintaxe na sua UI são pegos pelo compilador do Rust, não em tempo de execução.
2. **Zero-Bridge:** Diferente do React Native, não existe uma "ponte" entre a lógica e a UI. Tudo reside no mesmo espaço de memória.
3. **GPU-Bound:** Toda a renderização é feita através de shaders customizados no **WGPU**, permitindo animações de 120fps estáveis.

## 🚧 Status do Projeto

Oxidact está em desenvolvimento ativo. 
- [x] Parser RSX Recursivo
- [x] Árvore de VNodes Nativa
- [x] Suporte a Atributos e Estilos
- [ ] Integração completa com Taffy (Layout Engine)
- [ ] Hot Reloading nativo

---
Desenvolvido com ❤️ e 🦀 por entusiastas de performance.

## 🌐 Rodando no Navegador (Wasm)

1. Instale o target web do Rust:

```bash
rustup target add wasm32-unknown-unknown
```

2. Instale o wasm-pack (uma vez):

```bash
cargo install wasm-pack
```

3. Gere o bundle web do Oxidact:

```bash
wasm-pack build --target web --out-dir pkg
```

4. Sirva a pasta do projeto com um servidor estático e abra no navegador:

```bash
python3 -m http.server 8080
```

Depois, acesse `http://localhost:8080/web/`.

Arquivos principais da versão web:

- `src/lib.rs`: entrada Wasm com `#[wasm_bindgen(start)]`.
- `oxidact-core/src/lib.rs`: criação da janela/surface WGPU e anexação do canvas no `document.body` no target web.
- `web/index.html`: bootstrap mínimo que importa o pacote gerado em `pkg/`.

## 🧭 Web, Mobile e Desktop

O mesmo app (`build_app`) agora é compartilhado entre todas as plataformas, com entradas específicas por target:

- Desktop: `src/main.rs` chama `oxidact_app::run_app()`.
- Web (Wasm): `src/lib.rs` usa `#[wasm_bindgen(start)]` para iniciar automaticamente no browser.
- Mobile: usa a mesma base Rust/WGPU; o fluxo abaixo valida compilação para Android e iOS.

### Comandos rápidos

Use o helper script:

```bash
scripts/platforms.sh desktop
scripts/platforms.sh web-build
scripts/platforms.sh web-serve
scripts/platforms.sh android-check
scripts/platforms.sh ios-check
```

Ou em uma linha para validar desktop + web:

```bash
scripts/platforms.sh all-check
```

Observação para mobile:

- Estes comandos fazem `cargo check` cross-target (garantem compatibilidade de compilação).
- Android pode ser validado em Linux com target `aarch64-linux-android`.
- iOS exige ambiente macOS com Xcode (`xcrun`) para compilação nativa do target `aarch64-apple-ios`.
- Para gerar app Android/iOS instalável, o próximo passo é integrar com toolchains de empacotamento (ex.: `cargo-apk`/NDK e pipeline iOS com Xcode).
