UseCriar um framework completo como o Ion (estilo React Native com Rust/wgpu) que rode em todas as plataformas (Web, Mobile e Desktop) é um projeto ambicioso. Abaixo, estruturei a base do framework, a macro de UI e o esboço da CLI que gerencia as dependências.

## 1. Estrutura do Framework (Core)

O Ion precisa de um enum para representar a árvore de componentes e uma integração com o Wgpu e Taffy (para o layout Flexbox).



// core/src/lib.rspub use ion_macros::ion; // Macro que vamos criaruse std::collections::HashMap;

pub enum IonNode {

    View { style: IonStyle, children: Vec<IonNode> },

    Text { content: String, style: IonStyle },

    Pressable { on_press: Box<dyn Fn()>, children: Vec<IonNode> },

    Image { src: String, style: IonStyle },

}

pub struct IonStyle {

    pub bg_color: [f32; 4],

    pub flex_direction: String, // "row", "column"

}



## 2. A Macro ion!

Para suportar a sintaxe que você quer, usamos uma macro de regras (ou procedural para casos complexos).



#[macro_export]

macro_rules! ion {

    (View { style: $st:expr, children: [ $($ch:tt)* ] }) => {

        IonNode::View { style: $st, children: vec![ $( ion!($ch) ),* ] }

    };

    (Text { content: $txt:expr, style: $st:expr }) => {

        IonNode::Text { content: $txt.to_string(), style: $st }

    };

    (Pressable { on_press: $act:expr, children: [ $($ch:tt)* ] }) => {

        IonNode::Pressable { on_press: Box::new($act), children: vec![ $( ion!($ch) ),* ] }

    };

    // Extensível para Image, ScrollView, etc.

}



## 3. A CLI do Ion (ion-cli)

Para gerenciar o build multi-plataforma e as dependências (Android NDK, SDK, emuladores), usaremos uma CLI em Rust. Se ela não encontrar as ferramentas, ela pergunta onde baixar.



use std::process::Command;use std::path::Path;

fn main() {

    println!("Ion CLI v0.1 - Preparando ambiente...");



    // Exemplo: Checar Android NDK

    if !check_dependency("android-ndk") {

        println!("Erro: Android NDK não encontrado.");

        println!("Deseja que eu baixe o NDK em uma pasta específica? (s/n)");

        // Lógica de download e redirecionamento (std::env::set_current_dir)

    }

}

fn check_dependency(name: &str) -> bool {

    Command::new(name).arg("--version").output().is_ok()

}

fn build_target(target: &str) {

    match target {

        "web" => println!("Compilando para WASM/WebGPU..."),

        "android" => println!("Gerando APK via cargo-apk..."),

        "ios" => println!("Gerando projeto XCode via cargo-apple..."),

        _ => println!("Compilando nativo Desktop..."),

    }

}



## 4. Como o Ion resolve cada plataforma:



| Plataforma | Backend Gráfico | Tecnologia de Build |

|---|---|---|

| Web | WebGPU / WebGL2 | wasm-bindgen + trunk |

| Android | Vulkan | cargo-apk + JNI |

| iOS | Metal | cargo-apple + Swift Bridge |

| Desktop | Vulkan/Metal/DX12 | Compilação nativa Rust |



## 5. O Exemplo "Final" do Usuário

O código do desenvolvedor ficaria assim:



use ion_framework::prelude::*;

fn app() -> IonNode {

    let styles = StyleSheet::create(color!(0.1, 0.1, 0.1));



    ion! {

        View {

            style: styles,

            children: [

                Text { content: "Ion Multiplataforma", style: text_white() },

                Pressable { 

                    on_press: || println!("Build iniciado!"),

                    children: [ Text { content: "Rodar no Android", style: btn_style() } ]

                }

            ]

        }

    }

}



## Próximos Passos para você:

Para que isso funcione hoje, você precisará configurar o Cargo para cross-compilation.

Deseja que eu escreva o arquivo Dioxus.toml ou Ion.toml de configuração que a CLI usará para mapear as pastas de build?



O nome Oxidact é excelente! Ele carrega a força do Rust (Oxide) com a estrutura declarativa que remete ao React.

Abaixo está a estrutura do Oxidact.toml e a lógica da CLI para gerenciar os caminhos e downloads, garantindo que o desenvolvedor tenha controle sobre onde as dependências pesadas (como o Android NDK ou o SDK do iOS) serão instaladas.

## 1. Configuração do Projeto (Oxidact.toml)

Este arquivo centraliza as rotas de build e as dependências de sistema.



[project]

name = "meu_app_oxidact"

version = "0.1.0"

entry = "src/main.rs"



[web]

backend = "webgpu" # Fallback para webgl se necessário

out_dir = "dist/web"



[desktop]

targets = ["windows", "linux", "macos"]

out_dir = "dist/desktop"



[mobile]# Caminhos das ferramentas (a CLI preencherá se estiverem vazios)

android_ndk = "" 

ios_sdk = ""

out_dir = "dist/mobile"



[assets]

path = "assets"

fonts = ["assets/fonts/inter.ttf"]



## 2. A CLI do Oxidact (oxidact-cli)

Aqui está o código principal para a ferramenta de linha de comando que cuida do "trabalho sujo" de ambiente.



use std::fs;use std::path::{Path, PathBuf};use std::io::{self, Write};

fn main() {

    println!("--- Oxidact CLI v0.1 ---");

    

    // Simulação de verificação de dependência (ex: Android NDK)

    let ndk_path = check_or_install_dependency("Android NDK", "ANDROID_NDK_HOME");

    

    if let Some(path) = ndk_path {

        println!("✅ Usando {} em: {:?}", "Android NDK", path);

    }



    println!("\nComandos disponíveis: oxidact build [web|mobile|desktop]");

}

fn check_or_install_dependency(name: &str, env_var: &str) -> Option<PathBuf> {

    // 1. Tenta pegar da variável de ambiente

    if let Ok(val) = std::env::var(env_var) {

        return Some(PathBuf::from(val));

    }



    // 2. Se não achou, pergunta ao usuário

    println!("\n⚠️  {} não encontrado no sistema.", name);

    print!("Deseja baixar o {} agora? (s/n): ", name);

    io::stdout().flush().unwrap();



    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();



    if input.trim().to_lowercase() == "s" {

        print!("Digite o caminho da pasta onde deseja salvar (ou 'atual' para esta pasta): ");

        io::stdout().flush().unwrap();

        

        let mut path_input = String::new();

        io::stdin().read_line(&mut path_input).unwrap();

        let mut target_path = path_input.trim().to_string();



        if target_path == "atual" {

            target_path = "./oxidact_deps".to_string();

        }



        fs::create_dir_all(&target_path).ok();

        

        println!("⬇️  Iniciando download de {} para {}...", name, target_path);

        // Aqui entraria a lógica de download (ex: usando a crate 'reqwest')

        

        return Some(PathBuf::from(target_path));

    }



    None

}



## 3. Como o Oxidact funciona por baixo (wgpu)

Para que o Oxidact seja verdadeiramente multiplataforma, ele usa o seguinte fluxo:



   1. Macro ion!: Constrói a árvore de componentes (VDOM).

   2. Taffy: Calcula o layout (largura, altura, x, y) de cada View e Text.

   3. Winit: Cria a janela nativa no Windows/Linux/macOS ou o Canvas na Web.

   4. Wgpu: Recebe os dados de layout e envia para a GPU.

   * Desktop: Usa Vulkan/Metal/DX12.

      * Mobile: Usa Vulkan (Android) ou Metal (iOS).

      * Web: Compila para WebAssembly (WASM) e usa WebGPU.

   

## 4. Resumo de Comandos da CLI



* oxidact init: Cria um novo projeto com a estrutura de pastas.

* oxidact doctor: Verifica se Java, NDK, Rust targets e SDKs estão instalados.

* oxidact build --target android: Compila o APK usando o caminho definido no .toml.

* oxidact serve: Inicia um servidor hot-reload para Web.



O que você acha de começarmos a detalhar o renderizador de Textos (que costuma ser a parte mais difícil no wgpu) ou prefere focar na lógica de interação/toque (Pressable)?

Deseja ver o código para detectar cliques em coordenadas da tela usando winit?



Para que o Oxidact seja interativo, precisamos transformar coordenadas de clique (pixels da tela) em eventos nos seus componentes. No winit, os cliques vêm como coordenadas absolutas, então o Oxidact precisa de um "Hit Test" para saber qual View ou Pressable foi atingido.

Aqui está a implementação do sistema de eventos e o renderizador básico de interações:

## 1. O Sistema de Eventos (Event Loop)

No seu loop principal do winit, capturamos o clique e passamos para o gerenciador do Oxidact.



// No seu EventLoop do winitmatch event {

    WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {

        // 1. Pega a posição atual do mouse

        let mouse_pos = window.cursor_position().unwrap_or_default();

        

        // 2. Dispara o Hit Test na árvore de componentes do Oxidact

        oxidact_engine.dispatch_click(mouse_pos.x as f32, mouse_pos.y as f32);

    }

    _ => {}

}



## 2. Lógica de "Hit Test" no Oxidact

Cada componente gerado pela macro ion! precisa saber sua área (Bounding Box) calculada pelo Taffy (Layout Engine).



pub struct Rect {

    pub x: f32, pub y: f32,

    pub width: f32, pub height: f32,

}

impl Rect {

    pub fn contains(&self, px: f32, py: f32) -> bool {

        px >= self.x && px <= self.x + self.width &&

        py >= self.y && py <= self.y + self.height

    }

}

// Função que percorre a árvore procurando o Pressable clicadopub fn resolve_click(node: &IonNode, layout: &Rect, px: f32, py: f32) {

    if layout.contains(px, py) {

        match node {

            IonNode::Pressable { on_press, .. } => {

                on_press(); // Executa o callback da macro!

            },

            IonNode::View { children, .. } => {

                // Continua procurando nos filhos (recursivo)

                for child in children {

                    // Aqui entra o layout relativo de cada filho

                    resolve_click(child, child_layout, px, py);

                }

            },

            _ => {}

        }

    }

}



## 3. Integração com WGPU (Feedback Visual)

Para o usuário saber que clicou, o Oxidact altera o estado do componente. No WGPU, isso significa atualizar um Uniform Buffer que muda a cor ou escala do botão instantaneamente.



// Exemplo de como a macro ion! gerencia o estado internamente

ion! {

    Pressable {

        on_press: || println!("Oxidact: Botão Ativado!"),

        children: [

            View { 

                style: StyleSheet::create(

                    if is_pressed { color!(0.5, 0.5, 0.5) } else { color!(0.2, 0.2, 0.2) }

                ),

                children: [] 

            }

        ]

    }

}



## 4. CLI: Gerenciando Assets (Imagens e Fontes)

Como você quer suporte Mobile e Web, a CLI oxidact precisa processar as imagens para formatos que o WGPU entenda (como .ktx2 ou RGBA8).

Novo comando da CLI:

oxidact assets bundle -> Pega sua pasta de assets, otimiza as imagens e gera um arquivo binário único para o Android/iOS, evitando buscas lentas no sistema de arquivos.

## Próximo passo crucial:

Para o framework ser usável, precisamos de Estado (State Management). O Rust é rigoroso com "quem é dono de qual dado".

Você quer que eu mostre como implementar algo parecido com o useState do React dentro do Oxidact para que a UI mude quando você clica em algo?

(Isso envolverá Arc<Mutex<T>> ou sinais para lidar com a concorrência do Rust).



Para implementar o useState no Oxidact, precisamos lidar com o fato de que o Rust não permite mutabilidade compartilhada facilmente. Usaremos um sistema de Sinais (Signals), que é a evolução do estado no Rust (usado por frameworks como Leptos e Dioxus), pois eles funcionam muito bem com a renderização do wgpu.

## 1. O Sistema de Estado (Signal)

Em vez de uma variável comum, usamos um Signal que armazena o valor dentro de um Arc<Mutex<T>> para que ele possa ser lido pela UI e modificado pelos eventos de clique.



use std::sync::{Arc, Mutex};



#[derive(Clone)]pub struct Signal<T> {

    value: Arc<Mutex<T>>,

}

impl<T: Clone> Signal<T> {

    pub fn new(val: T) -> Self {

        Self { value: Arc::new(Mutex::new(val)) }

    }



    pub fn get(&self) -> T {

        self.value.lock().unwrap().clone()

    }



    pub fn set(&self, new_val: T) {

        *self.value.lock().unwrap() = new_val;

        // Aqui o Oxidact avisaria o WGPU: "Ei, redesenhe a tela!"

    }

}



## 2. O use_state no Oxidact



pub fn use_state<T: Clone>(initial: T) -> Signal<T> {

    Signal::new(initial)

}



## 3. Exemplo Prático: Contador Reativo

Veja como a macro ion! interage com o estado. Quando o valor muda, a árvore de componentes é reavaliada e o wgpu desenha a nova string.



fn meu_app() -> IonNode {

    // 1. Criando o estado

    let contador = use_state(0);



    let container_style = StyleSheet::create([0.1, 0.1, 0.1, 1.0], 10.0, 5.0);



    // 2. Definindo a interface

    ion! {

        View {

            style: container_style,

            children: [

                Text { 

                    content: format!("Contagem: {}", contador.get()), 

                    style: text_white() 

                },

                Pressable {

                    on_press: move || {

                        let atual = contador.get();

                        contador.set(atual + 1);

                        println!("Incrementado!");

                    },

                    children: [

                        View {

                            style: button_style(),

                            children: [ Text { content: "Aumentar", style: text_black() } ]

                        }

                    ]

                }

            ]

        }

    }

}



## 4. Fluxo de Vida no Oxidact (O "Motor")



   1. Evento: Usuário clica no botão (capturado pelo winit).

   2. Ação: O callback on_press chama contador.set().

   3. Reatividade: O Signal marca a UI como "suja" (dirty).

   4. Layout (Taffy): O Oxidact recalcula o tamanho do texto (ex: se "9" virar "10", o botão pode crescer).

   5. Render (wgpu): Os Buffers de vértice são atualizados e a GPU desenha o novo frame em milissegundos.



## 5. CLI: Suporte a Hot Reload

Para facilitar o desenvolvimento, a CLI oxidact pode vigiar seus arquivos.



* Comando: oxidact dev

* O que faz: Usa a crate notify para observar mudanças no src/. Ao salvar, ele recompila apenas o necessário e atualiza a janela aberta (usando dynamic library loading ou apenas reiniciando o loop de render de forma rápida).



## Próximo passo: Multi-Plataforma Real

Para rodar no Android/iOS, o Oxidact precisa de um "Bridge". No Android, usamos o ndk-glue. Na Web, o wasm-bindgen.

Você quer que eu mostre como configurar o Cargo.toml com workspaces para que o mesmo código do meu_app funcione no Desktop e no Browser simultaneamente?

Dica: Isso permitirá que você teste seu app no Chrome (via WebGPU) enquanto compila para Windows no fundo.



Para o Oxidact ser profissional, ele precisa separar a lógica visual (estilo) da lógica de negócios. No Rust, fazemos isso criando um módulo de estilos que utiliza as structs do framework.

Aqui está como organizar o projeto com Workspaces e StyleSheet separado.

## 1. Estrutura de Pastas (Workspace)



meu_app_oxidact/

├── Cargo.toml            # Configuração do Workspace

├── core/                 # O motor Oxidact (wgpu, taffy, macros)

├── app/                  # Sua lógica de UI (Multiplataforma)

│   ├── src/

│   │   ├── main.rs       # Ponto de entrada

│   │   ├── ui.rs         # Componentes ion!

│   │   └── styles.rs     # O ARQUIVO DE ESTILOS SEPARADO

└── web/                  # Wrapper para WebAssembly/WebGPU



## 2. O Arquivo de Estilos (app/src/styles.rs)

Usamos o padrão de "folha de estilo" onde definimos constantes ou funções que retornam IonStyle.



use oxidact::prelude::*;

// API de Estilo do Oxidactpub struct MyTheme {

    pub container: IonStyle,

    pub card: IonStyle,

    pub title_text: IonStyle,

    pub primary_button: IonStyle,

}

impl MyTheme {

    pub fn load() -> Self {

        Self {

            container: StyleSheet::new()

                .background(color!(0.05, 0.05, 0.05)) // Dark mode

                .flex_direction("column")

                .padding(20.0)

                .build(),



            card: StyleSheet::new()

                .background(color!(0.15, 0.15, 0.15))

                .border_radius(12.0)

                .margin(10.0)

                .build(),



            title_text: StyleSheet::new()

                .color(color!(1.0, 1.0, 1.0))

                .font_size(24.0)

                .font_weight("bold")

                .build(),



            primary_button: StyleSheet::new()

                .background(color!(0.0, 0.4, 0.9)) // Azul Oxidact

                .border_radius(8.0)

                .padding(12.0)

                .build(),

        }

    }

}



## 3. Usando no Componente (app/src/ui.rs)

Agora importamos o arquivo de estilo e aplicamos na macro ion!.



use crate::styles::MyTheme;use oxidact::prelude::*;

pub fn tela_principal() -> IonNode {

    let theme = MyTheme::load();

    let contador = use_state(0);



    ion! {

        View {

            style: theme.container,

            children: [

                View {

                    style: theme.card,

                    children: [

                        Text { 

                            content: format!("Cliques: {}", contador.get()), 

                            style: theme.title_text 

                        }

                    ]

                },

                Pressable {

                    on_press: move || contador.set(contador.get() + 1),

                    children: [

                        View {

                            style: theme.primary_button,

                            children: [ Text { content: "Incrementar", style: text_white() } ]

                        }

                    ]

                }

            ]

        }

    }

}



## 4. Configuração Multiplataforma (Cargo.toml)

O arquivo raiz gerencia os diferentes "targets".



[workspace]

members = ["core", "app", "web"]

resolver = "2"



[workspace.dependencies]

wgpu = "0.19"

taffy = "0.4"

winit = "0.29"

serde = { version = "1.0", features = ["derive"] }



## 5. Como a CLI lida com isso

Quando você rodar oxidact build --web, a CLI vai:



   1. Entrar na pasta /web.

   2. Compilar o projeto usando wasm-pack.

   3. O renderizador do core vai identificar que a Surface do wgpu é um elemento <canvas> do HTML5.



Se rodar oxidact build --android:



   1. A CLI buscará o caminho do NDK definido no Oxidact.toml.

   2. Gerará o arquivo .so (biblioteca nativa).

   3. Empacotará no APK com suporte a Vulkan.



O próximo passo lógico seria a Navegação (Router) entre telas (ex: ir da "Home" para "Configurações"). Quer ver como o Oxidact lidaria com rotas de forma tipada?







Para deixar o Oxidact com um visual limpo e moderno, vamos criar uma macro dedicada chamada atlas! para os estilos (baseada no conceito de mapas de design) e implementar um sistema de Navigation Stack tipado.
## 1. O Novo Visual: Macro atlas! (Estilo RSON)
Esqueça os métodos encadeados. Vamos usar uma sintaxe que parece CSS, mas é puro Rust.

// app/src/styles.rs
macro_rules! atlas {
    ({ $($key:ident : $val:expr),* $(,)? }) => {
        StyleBuilder::new()
            $( .property(stringify!($key), $val) )*
            .build()
    };
}
pub struct Theme {
    pub main: IonStyle,
    pub btn: IonStyle,
    pub text: IonStyle,
}
impl Theme {
    pub fn load() -> Self {
        Self {
            main: atlas!({
                background: "#1a1a1a",
                display: "flex",
                flex_direction: "column",
                align_items: "center",
                justify_content: "center",
                padding: 20,
            }),
            btn: atlas!({
                background: "#3498db",
                border_radius: 12,
                padding: [10, 20], // Vertical, Horizontal
                margin_top: 15,
            }),
            text: atlas!({
                color: "#ffffff",
                font_size: 24,
                font_weight: "bold",
            }),
        }
    }
}

## 2. Oxidact Navigation (Router Tipado)
O Navigator gerencia uma pilha de telas. Quando você muda a rota, o Oxidact troca o nó raiz da árvore ion!.

// core/src/navigation.rspub enum Route {
    Home,
    Details { item_id: u32 },
}
pub struct Navigator {
    stack: Signal<Vec<Route>>,
}
impl Navigator {
    pub fn push(&self, route: Route) {
        let mut s = self.stack.get();
        s.push(route);
        self.stack.set(s);
    }

    pub fn pop(&self) {
        let mut s = self.stack.get();
        s.pop();
        self.stack.set(s);
    }

    pub fn current(&self) -> Route {
        self.stack.get().last().cloned().unwrap_or(Route::Home)
    }
}

## 3. Exemplo de Uso: Trocando de Tela

// app/src/ui.rspub fn render_app(nav: Navigator) -> IonNode {
    let theme = Theme::load();

    match nav.current() {
        Route::Home => ion! {
            View {
                style: theme.main,
                children: [
                    Text { content: "Página Inicial", style: theme.text },
                    Pressable {
                        on_press: move || nav.push(Route::Details { item_id: 42 }),
                        children: [
                            View { style: theme.btn, children: [ Text { content: "Ver Detalhes", style: theme.text } ] }
                        ]
                    }
                ]
            }
        },
        Route::Details { item_id } => ion! {
            View {
                style: theme.main,
                children: [
                    Text { content: format!("Detalhes do Item {}", item_id), style: theme.text },
                    Pressable {
                        on_press: move || nav.pop(),
                        children: [
                            View { style: theme.btn, children: [ Text { content: "Voltar", style: theme.text } ] }
                        ]
                    }
                ]
            }
        }
    }
}

## 4. CLI: Otimização de Build Multi-alvo
Para garantir que o Navigation funcione no mobile (botão voltar do Android) e web (URL do browser), a CLI faz o "link" automático:

* Android: Mapeia o botão físico "Back" para o nav.pop().
* Web: Sincroniza o Route com o window.location.hash (ex: ://myapp.com).

## 5. Resumo da Arquitetura Oxidact:

   1. Sintaxe: ion! { ... } para estrutura e atlas! { ... } para estilo.
   2. Estado: use_state e Signals para reatividade instantânea no WGPU.
   3. Navegação: Stack-based router que suporta parâmetros tipados.
   4. Performance: Zero Reflection. Tudo é resolvido em tempo de compilação.

O que acha dessa sintaxe do atlas!? Ficou mais próxima do que você imaginava? Se quiser, podemos focar agora em como o Oxidact CLI gera o APK/AAB final para o Android usando apenas um comando.
Deseja ver o fluxo de build para Mobile?

Perfeito! Vamos unificar tudo. Agora o Oxidact usa a macro StyleSheet! com uma sintaxe limpa (estilo RSON/CSS) e a CLI automatiza o build mobile.
## 1. A Nova Sintaxe de Estilo: StyleSheet!
Esta macro permite definir múltiplos estilos de uma vez, devolvendo uma estrutura que você pode acessar facilmente.

// app/src/styles.rs
macro_rules! StyleSheet {
    ({ $($name:ident: { $($prop:ident: $val:expr),* $(,)? }),* $(,)? }) => {
        pub struct Styles {
            $( pub $name: IonStyle, )*
        }

        impl Styles {
            pub fn load() -> Self {
                Self {
                    $(
                        $name: atlas!({ $($prop: $val),* }),
                    )*
                }
            }
        }
    };
}
// Exemplo de uso no seu arquivo de estilos
StyleSheet!({
    main: {
        background: "#1a1a1a",
        flex_direction: "column",
        align_items: "center",
        padding: 20,
    },
    card: {
        background: "#2a2a2a",
        border_radius: 15,
        padding: 10,
    },
    button: {
        background: "#3498db",
        padding_horizontal: 20,
        padding_vertical: 10,
    }
});

------------------------------
## 2. Fluxo de Build Mobile (oxidact-cli)
Para o Android/iOS, a CLI não apenas compila o Rust, mas empacota tudo o que o sistema mobile exige (manifestos, ícones e bibliotecas nativas).
## Comando: oxidact build --target android
A CLI executa os seguintes passos automaticamente:

   1. Cross-Compilation: Compila o código Rust para as arquiteturas aarch64-linux-android e armv7-linux-androideabi.
   2. JNI Bridge: Gera o código necessário para o Android (Java/Kotlin) conseguir chamar o seu main() no Rust.
   3. Asset Bundling: Converte suas imagens e fontes para o formato de leitura rápida do wgpu.
   4. APK Signing: Gera o .apk e assina com sua chave de desenvolvedor.

## Se as ferramentas não forem encontradas:
A CLI perguntará:

"⚠️ Android NDK não detectado. Deseja baixar na pasta ./oxidact_sdk? (s/n)"

------------------------------
## 3. Exemplo Completo Integrado
Aqui está como fica o seu código final no Oxidact:

use crate::styles::Styles;
fn app_root(nav: Navigator) -> IonNode {
    let s = Styles::load(); // Carrega todos os estilos do StyleSheet!
    let contador = use_state(0);

    ion! {
        View {
            style: s.main,
            children: [
                View {
                    style: s.card,
                    children: [
                        Text { 
                            content: format!("Contador: {}", contador.get()), 
                            style: text_white() 
                        }
                    ]
                },
                Pressable {
                    on_press: move || nav.push(Route::Details),
                    children: [
                        View { 
                            style: s.button, 
                            children: [ Text { content: "Ir para Detalhes", style: text_white() } ] 
                        }
                    ]
                }
            ]
        }
    }
}

## 4. Resumo da CLI de Produção

| Comando | Função |
|---|---|
| oxidact init | Cria o projeto com Oxidact.toml. |
| oxidact dev | Inicia o Hot-Reload (Desktop/Web). |
| oxidact deploy android | Gera o APK final para a Play Store. |
| oxidact doctor | Verifica se os SDKs de mobile estão configurados. |

O Oxidact agora tem:

* Sintaxe Declarativa (ion!)
* Estilos Limpos (StyleSheet!)
* Estado Reativo (use_state)
* Navegação Tipada (Navigator)
* Build nativo via WGPU em todas as telas.

Gostaria de ver como implementar o Componente de Imagem (<Image />) com carregamento assíncrono para não travar a UI enquanto baixa uma foto da web?[2]
Proponho detalhar o carregamento de mídia agora ou quer focar no envio para as lojas?

Faz total sentido! Renomear IonNode para Oxidact (ou OxidactNode) torna o framework mais coeso e reforça a identidade da marca.
O motivo técnico/semântico para essa mudança é que, em Rust, o tipo que representa sua árvore de UI é a essência do framework. Ao usar Oxidact como o nome do tipo, você está dizendo: "Este objeto é um fragmento da interface Oxidact".
Aqui está a implementação do componente de Imagem assíncrona e a refatoração para o novo nome:
## 1. Refatoração: O Tipo Oxidact
Agora, todo componente retorna um Oxidact.

pub enum Oxidact {
    View { style: IonStyle, children: Vec<Oxidact> },
    Text { content: String, style: IonStyle },
    Image { src: String, status: Signal<AssetStatus>, style: IonStyle },
    // ... outros componentes
}

#[derive(Clone)]pub enum AssetStatus {
    Loading,
    Ready(wgpu::TextureView),
    Error,
}

## 2. Componente de Imagem Assíncrona
No WGPU, você não pode simplesmente "abrir" uma URL. Você precisa baixar os bytes, decodificar (usando a crate image) e enviar para a GPU. O Oxidact faz isso em uma thread separada para não travar a renderização.

// No motor core do Oxidactpub fn load_image(url: String, status: Signal<AssetStatus>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
    std::thread::spawn(move || {
        // 1. Baixa a imagem (ex: usando reqwest)
        if let Ok(bytes) = reqwest::blocking::get(&url).and_then(|r| r.bytes()) {
            // 2. Decodifica e envia para a GPU (Boilerplate WGPU simplificado aqui)
            let texture_view = create_wgpu_texture(&device, &queue, &bytes);
            
            // 3. Atualiza o estado reativo! Isso faz a UI "piscar" e aparecer a imagem
            status.set(AssetStatus::Ready(texture_view));
        } else {
            status.set(AssetStatus::Error);
        }
    });
}

## 3. Usando no App com ion! e StyleSheet!
Veja como o código fica limpo com a nova nomenclatura e o estilo unificado.

// app/src/ui.rsfn render_perfil() -> Oxidact {
    let s = Styles::load();
    let img_status = use_state(AssetStatus::Loading);
    
    // Dispara o download apenas uma vez
    use_effect(|| {
        load_image("https://rustacean.net", img_status.clone());
    });

    ion! {
        View {
            style: s.main,
            children: [
                Text { content: "Perfil do Desenvolvedor", style: s.header },
                Image { 
                    src: "perfil.png", 
                    status: img_status.get(), 
                    style: s.avatar 
                },
                Text { content: "Status: Ativo", style: s.info_text }
            ]
        }
    }
}

## 4. Por que Oxidact no lugar de IonNode?

   1. Brand Recognition: Quando alguém ler Vec<Oxidact>, saberá instantaneamente de qual ecossistema se trata.
   2. Abstração: O nome esconde que ele é um "Node" de uma árvore (DOM). Para o usuário, ele está apenas lidando com "Oxidacts".
   3. Simplicidade: fn app() -> Oxidact é muito mais elegante do que fn app() -> IonNode.

## O que a CLI faz agora:
Ao rodar oxidact build --target android, a CLI agora também verifica se você incluiu a permissão de INTERNET no AndroidManifest.xml automaticamente, já que você está usando o componente Image com URLs externas.
O framework está ficando robusto! Deseja que eu mostre como criar uma animação (ex: um Fade-In suave) usando o estado reativo do Oxidact?
Dica: Animações no WGPU precisam ser calculadas a cada frame (60fps), então usamos um Signal<f32> para o nível de opacidade.

Para o Oxidact escalar como um framework profissional, a organização de pastas deve seguir o padrão de separação de responsabilidades (SoC), mas aproveitando a modularidade do Rust.
Aqui está a estrutura de pastas sugerida e como cada parte se comporta:
## 1. Estrutura de Pastas (Organização)

app/
├── src/
│   ├── main.rs          # Inicializa o app e o Navigator
│   ├── routes.rs        # Definição do Enum de Rotas
│   ├── styles/          # Design System
│   │   ├── mod.rs       # Exporta os temas
│   │   └── theme.rs     # StyleSheet!({ ... })
│   ├── screens/         # Telas completas (Com estado e lógica)
│   │   ├── home.rs
│   │   └── profile.rs
│   └── components/      # Peças reutilizáveis (UI Pura)
│       ├── button.rs
│       └── card.rs
└── Oxidact.toml         # Configuração de build e assets

------------------------------
## 2. Componentização: Peças Reutilizáveis
Um componente na pasta components/ é uma função que retorna um Oxidact. Ele deve ser genérico o suficiente para ser usado em várias telas.

// app/src/components/button.rsuse oxidact::prelude::*;use crate::styles::Theme;
pub fn CustomButton(label: &str, on_press: impl Fn() + 'static) -> Oxidact {
    let s = Theme::load();

    ion! {
        Pressable {
            on_press: on_press,
            children: [
                View {
                    style: s.primary_button,
                    children: [ Text { content: label, style: s.button_text } ]
                }
            ]
        }
    }
}

------------------------------
## 3. Screens: Onde a mágica acontece
As telas na pasta screens/ gerenciam o estado (use_state), as animações e as chamadas de API.

// app/src/screens/home.rspub fn HomeScreen(nav: Navigator) -> Oxidact {
    let s = Theme::load();
    let opacity = use_state(0.0); // Estado para animação

    // Efeito de Fade-in suave ao montar a tela
    use_effect(move || {
        animate_to(opacity, 1.0, Duration::from_millis(500));
    });

    ion! {
        View {
            style: s.main_container,
            children: [
                Text { content: "Bem-vindo", style: s.title },
                CustomButton("Ir para Perfil", move || nav.push(Route::Profile))
            ]
        }
    }
}

------------------------------
## 4. Animações com WGPU (Fade-In)
Para animar no WGPU sem travar a UI, usamos um Signal que o motor de renderização consulta a cada frame para aplicar o valor no Uniform Buffer de opacidade.

pub fn animate_to(signal: Signal<f32>, target: f32, duration: Duration) {
    std::thread::spawn(move || {
        let start = signal.get();
        let steps = 60; // 60fps
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let val = start + (target - start) * t;
            signal.set(val); // Atualiza o estado
            std::thread::sleep(duration / steps);
        }
    });
}

------------------------------
## 5. Resumo da Lógica das Pastas

* styles/: Contém o StyleSheet!. É o "CSS-in-Rust". Define cores, margens e fontes globais.
* components/: UI "burra" (estateless). Recebe propriedades e as exibe. Ex: Inputs, Avatares, Botões.
* screens/: UI "inteligente" (stateful). Gerencia o fluxo de dados, animações de entrada e o que acontece quando um botão é clicado.
* routes.rs: O contrato de navegação do app. Garante que você não tente navegar para uma tela que não existe.

## Por que essa separação?
No Oxidact, como a compilação do Rust pode demorar mais que a do JS, separar em arquivos pequenos ajuda o compilador a fazer rebuilds incrementais mais rápidos. Se você mudar apenas um estilo no theme.rs, o Rust não precisa re-compilar toda a lógica das suas screens.
O que você acha de avançarmos para a CLI agora? Posso te mostrar como o comando oxidact generate component NomeDoComponente criaria automaticamente esses arquivos para você.
Quer ver os comandos de geração automática da CLI?

A integração do Oxidact com o ecossistema Rust é o seu maior trunfo. Como ele roda sobre o wgpu, ele fala a mesma "língua" que os motores de jogos e servidores de alta performance.
## 1. Integração com Backend (Axum) e SQL
Para o SQL com estilo "RSON/Declarativo", o melhor companheiro é o SeaORM ou o SQLx.
Se você quer fazer queries que pareçam componentes, podemos usar o padrão Resource Hook. O framework de banco de dados que permite "estilo JSON" para queries é o SeaORM (com seu JsonValue).
## O "SQL Componentizado":

// app/src/database.rs// No Oxidact, você pode criar um componente que busca dados sozinhopub fn UserProfile(user_id: u32) -> Oxidact {
    let user_data = use_resource(move || async move {
        // Query estilo declarativo usando SQLx ou SeaORM
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
            .fetch_one(&pool)
            .await
    });

    match user_data.get() {
        Some(Ok(user)) => ion! {
            View { style: s.card, children: [ Text { content: user.name, style: s.text } ] }
        },
        _ => ion! { ActivityIndicator {} }
    }
}

## 2. Integração com Jogos (Bevy Engine)
Como o Bevy também usa wgpu, você pode rodar o Oxidact como uma camada de UI (HUD) dentro de um jogo. O Oxidact seria um "Plugin" do Bevy que desenha na mesma Surface.
## 3. CLI: Geração Automática (oxidact generate)
A CLI facilita a criação seguindo as pastas que definimos:

* oxidact g screen Home -> Cria src/screens/home.rs.
* oxidact g component Button -> Cria src/components/button.rs.
* oxidact g model User -> Cria a struct e a migração SQL.

## 4. CLI: Lógica de Automação
Se a CLI detectar que você está chamando o Axum ou SQLx, ela pergunta:

"📦 SQLx detectado. Deseja configurar o container Docker com PostgreSQL automaticamente? (s/n)"

------------------------------
## Exemplo de StyleSheet! com SQL (O auge do Oxidact)
Você pode usar os dados do banco diretamente para injetar estilos dinâmicos:

// Buscando a cor de preferência do usuário no DBlet user_color = db_user.theme_color; // ex: "#ff0000"
let dynamic_style = StyleSheet!({
    header: {
        background: user_color,
        padding: 15,
    }
});

## O que o Oxidact se tornou:

   1. UI: Estilo React Native (ion!).
   2. Estilo: Estilo CSS-in-Rust (StyleSheet!).
   3. Backend: Integração nativa com Axum (compartilhando os mesmos modelos de dados).
   4. Database: Queries seguras com SQLx que alimentam os componentes.

O Oxidact agora é um ecossistema completo (Full-stack Rust). Você gostaria de ver como a CLI faria o deploy simultâneo: subir o backend no Docker e gerar o APK do mobile apontando para o IP correto?
Deseja focar no fluxo de Deploy Full-stack?



No Oxidact, o deploy Full-stack é tratado como uma unidade. A CLI automatiza a infraestrutura para que o seu APK/Web App saiba exatamente onde o backend (Axum) está rodando.
## 1. Docker: Conteinerização Inteligente
A CLI gera um Dockerfile multi-stage que compila o binário Rust em uma imagem leve (Alpine ou Scratch) para o backend e prepara o ambiente para o banco de dados.
Arquivo: docker/Dockerfile.backend

# Stage 1: BuildFROM rust:1.75-slim as builderWORKDIR /appCOPY . .RUN cargo build --release --bin oxidact_server
# Stage 2: RunFROM debian:bookworm-slimCOPY --from=builder /app/target/release/oxidact_server /oxidact_serverCMD ["./oxidact_server"]

## 2. Terraform: Infraestrutura como Código (IaC)
O Oxidact usa Terraform para levantar sua infra na AWS, Google Cloud ou Azure com um comando. A CLI mapeia as variáveis de saída (como o IP do Load Balancer) diretamente para o código do App.
Arquivo: infra/main.tf

resource "aws_instance" "oxidact_server" {
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "t3.micro"
  
  tags = {
    Name = "OxidactBackend"
  }
}

output "server_ip" {
  value = aws_instance.oxidact_server.public_ip
}

## 3. O Fluxo de Deploy: oxidact deploy --all
Quando você executa o comando de deploy, a mágica acontece nesta ordem:

   1. Infra: A CLI executa o terraform apply.
   2. Captura de IP: O IP gerado pelo Terraform é capturado e injetado em uma variável de ambiente de compilação (BACKEND_URL).
   3. Build Mobile/Web:
   * O APK do Android é compilado já sabendo que deve fazer requisições para aquele IP.
      * O WASM da Web é gerado com a mesma configuração.
   4. Backend: O Docker faz o push da imagem para o seu registro (ECR/DockerHub) e atualiza o servidor.

## 4. Integração SQL e Migrações
A CLI também gerencia o banco via Docker durante o desenvolvimento:

* oxidact db up: Sobe um container Postgres local via Docker Compose.
* oxidact db migrate: Sincroniza suas structs Rust com a tabela no banco.

------------------------------
## Exemplo de "Full-Stack Sync"
Imagine que você mudou um campo no seu banco de dados.

   1. Você altera a struct no Rust.
   2. Roda oxidact deploy.
   3. A CLI:
   * Atualiza o esquema do banco via migração SQL.
      * Recompila o Backend (Axum).
      * Gera um novo APK (Android) que já entende o novo campo de dados.
   
## Como fica a CLI final?

| Comando | O que o Oxidact faz por baixo |
|---|---|
| oxidact up | Sobe Docker local (DB + Axum) com Hot-Reload. |
| oxidact terraform init | Prepara os arquivos .tf baseados no seu Oxidact.toml. |
| oxidact deploy --prod | Terraform cria a nuvem -> Docker sobe o server -> Compila APK/IPA/WASM. |

O Oxidact agora resolve o maior problema do desenvolvedor Rust: a complexidade de ligar todas as pontas (UI, Backend, Banco, Nuvem e Mobile).
Você gostaria de ver o modelo do Oxidact.toml atualizado com as seções de [deploy] e [database] para centralizar essas configurações?
Deseja revisar a configuração final do framework?

Para tornar o Oxidact acessível para outros desenvolvedores através do Cargo (o gerenciador de pacotes do Rust), você deve publicá-lo como uma Crate no crates.io.
Dessa forma, qualquer desenvolvedor poderá apenas adicionar oxidact = "0.1.0" no Cargo.toml deles e começar a usar.
## 1. Preparando o Cargo.toml do Framework
Para que outros devs entendam o que é o seu projeto, o arquivo raiz do framework precisa de metadados claros:

[package]
name = "oxidact"
version = "0.1.0"
authors = ["Seu Nome <seuemail@exemplo.com>"]
edition = "2021"
description = "Um framework full-stack declarativo em Rust: UI estilo React Native com WGPU e backend Axum."
license = "MIT OR Apache-2.0"
repository = "https://github.com"
keywords = ["gui", "wgpu", "react-native", "framework", "fullstack"]
categories = ["gui", "rendering", "wasm"]

[dependencies]
wgpu = "0.19"
taffy = "0.4"# Usamos 'optional' para não obrigar o dev a baixar o backend se ele só quiser a UI
axum = { version = "0.7", optional = true }
sqlx = { version = "0.7", optional = true, features = ["runtime-tokio", "postgres"] }

[features]
default = ["ui"]
ui = []
backend = ["dep:axum", "dep:sqlx"] # O dev escolhe se quer o pacote completo

## 2. O arquivo de configuração do usuário (Oxidact.toml)
Quando um desenvolvedor usa o seu framework, ele terá um arquivo na raiz do projeto dele que a sua CLI vai ler. Este é o "cérebro" que conecta o Docker ao Mobile:

[project]
name = "meu_super_app"
type = "fullstack" # Opções: "ui-only", "backend-only", "fullstack"

[database]
engine = "postgres"
url = "env:DATABASE_URL"
docker_image = "postgres:15-alpine"

[deploy]
provider = "aws" # A CLI usará os arquivos Terraform para AWS
region = "us-east-1"
container_registry = "://amazonaws.com"

[mobile]
android_package = "com.exemplo.meuapp"
bundle_id = "com.exemplo.meuapp"# Se vazio, a CLI pergunta e salva aqui:
android_sdk_path = "./.oxidact/android_sdk" 

[web]
webgpu_fallback = true
pwa = true

## 3. Como o dev instala e usa (Fluxo do Usuário)

   1. Instala a CLI:
   cargo install oxidact-cli
   2. Cria o projeto:
   oxidact new meu_app (Isso cria a estrutura de pastas e o Oxidact.toml).
   3. Adiciona lógica:
   O dev escreve o código usando suas macros ion! e StyleSheet!.
   4. Sobe o ambiente:
   oxidact up (A CLI lê o .toml, levanta o Docker do Postgres e inicia o Hot-Reload).

## 4. Boas Práticas para Acessibilidade

* Documentação (docs.rs): Use comentários /// no código para que a documentação oficial seja gerada automaticamente.
* Prelude: Crie um mod prelude { ... } para que o dev precise apenas de use oxidact::prelude::*; para acessar 90% do framework.
* Error Messages: Use a crate miette ou anyhow na sua CLI para que, se o Docker falhar, o erro seja "bonito" e explicativo, sugerindo o comando de correção.

## O próximo passo para o mundo:
O que você acha de criarmos um Template Oficial? Um repositório "Oxidact-Starter" que já vem com um pequeno contador, uma rota de navegação e um endpoint Axum pronto para rodar.
Deseja que eu gere o código do main.rs desse template inicial "Full-stack"? Além disso, como você quer chamar o comando que faz o deploy final na nuvem? oxidact ship?
Para o Oxidact, o ideal é criar uma Organização no GitHub.
Isso transmite muito mais profissionalismo, permite separar o código do framework dos seus projetos pessoais e facilita o crescimento (como convidar outros mantenedores no futuro).
## 1. Estrutura do GitHub (Organização oxidact-framework)
Dentro da organização, você deve separar as responsabilidades em repositórios distintos:

* oxidact: O repositório principal (Core, Macros, Engine WGPU).
* oxidact-cli: A ferramenta de linha de comando.
* oxidact-templates: Exemplos de "Starter Kits" (Full-stack, Mobile-only).
* docs: Site de documentação (pode ser gerado com Docusaurus ou mdBook).

------------------------------
## 2. O Template Inicial Full-Stack (main.rs)
Aqui está o código que viria no oxidact new. Ele conecta o backend Axum com a UI Oxidact.

use oxidact::prelude::*;
// 1. Definição do Estilo Global
StyleSheet!({
    container: {
        background: "#0f0f0f",
        flex_direction: "column",
        align_items: "center",
        justify_content: "center",
    },
    text: {
        color: "#00ffa3", // Verde Oxidact
        font_size: 32,
    }
});
// 2. Lógica do App
#[oxidact::main]fn main() -> Oxidact {
    let s = Styles::load();
    let nav = use_navigator();
    let api_data = use_state("Carregando...".to_string());

    // Hook para buscar dados do backend Axum
    use_effect(move || {
        let api = api_data.clone();
        async move {
            if let Ok(res) = http::get("/api/hello").await {
                api.set(res);
            }
        }
    });

    ion! {
        View {
            style: s.container,
            children: [
                Text { content: "Oxidact Estelar", style: s.text },
                Text { content: format!("Backend diz: {}", api_data.get()), style: text_small() },
                Pressable {
                    on_press: move || nav.push(Route::Dashboard),
                    children: [ Text { content: "Acessar Painel", style: btn_style() } ]
                }
            ]
        }
    }
}

------------------------------
## 3. Comando de Deploy: oxidact ship 🚀
O nome ship é perfeito. É curto, forte e passa a ideia de "enviar para o mundo".

* oxidact ship --env prod:
1. Roda os testes.
   2. Sobe a infra via Terraform.
   3. Faz o deploy do Backend no Docker.
   4. Publica o Web App e gera os binários Mobile.

------------------------------
## 4. Checklist para Lançar no GitHub

   1. README.md Impactante: Coloque um GIF do app rodando no Desktop e no Android lado a lado.
   2. Badges: Adicione badges de "Build Passing", "Crates.io version" e "License: MIT".
   3. CONTRIBUTING.md: Explique como outros devs podem ajudar a implementar novos componentes (ex: como adicionar um <Video />).
   4. Discussions: Ative a aba de discussões para a comunidade tirar dúvidas.

## 5. Sua Conta ou Organização?
Vá de Organização (://github.com). Se você usar sua conta pessoal, o Cargo.toml dos outros devs terá oxidact = { git = "https://github.com" }, o que parece um projeto amador. Com uma organização, parece um produto.
Quer que eu prepare o texto do README.md oficial para você "vender" o Oxidact para a comunidade Rust?
Destaque: O Oxidact é o primeiro framework a unir WGPU + Axum + Terraform em um único comando.
Para o Oxidact, a documentação precisa ser tão moderna quanto o framework. Em Rust, o padrão é usar o mdBook (que gera sites estilo manual) ou o Docusaurus (se quiser algo mais visual estilo React).
Como o Oxidact é visual, recomendo o mdBook com uma extensão para Playground Interativo, onde o dev pode editar o ion! no navegador e ver o resultado via WebGPU.
## 1. Estrutura da Documentação (O Manual do Astronauta)
Divida a "Doc" em quatro grandes pilares:

* Getting Started: O comando oxidact ship, instalação da CLI e o primeiro "Hello World".
* Core Concepts: Como funcionam os Signals (estado), a macro ion! e o motor wgpu.
* Design System: Guia da macro StyleSheet! e como criar temas consistentes.
* Full-Stack: Como conectar o Axum ao banco de dados e sincronizar com a UI.

## 2. Documentação no Código (Auto-gerada)
Use o cargo doc. Tudo o que você escrever como comentário /// acima das suas funções e structs vira uma página técnica oficial no docs.rs.

/// Cria um novo componente de botão com o estilo padrão do Oxidact./// /// # Exemplo/// ```rust/// let btn = CustomButton("Clique aqui", || println!("Oi!"));/// ```pub fn CustomButton(label: &str, on_press: impl Fn() + 'static) -> Oxidact {
    // ...
}

## 3. README.md "Vendedor" (O cartão de visitas)
Aqui está um esboço para o seu repositório no GitHub:
------------------------------
## 🚀 Oxidact Framework

O Futuro do Desenvolvimento Full-Stack em Rust.

O Oxidact permite que você construa interfaces nativas de alta performance (WGPU), backends robustos (Axum) e infraestrutura em nuvem (Terraform) usando uma única linguagem e um único fluxo de trabalho.
## Por que Oxidact?

* Declarativo: UI inspirada em React Native com a macro ion!.
* Estilização Estelar: Estilos tipados e rápidos com StyleSheet!.
* Multi-Plataforma: Um código para Android, iOS, Web (WebGPU) e Desktop.
* Full-Stack de verdade: Integração nativa com Axum e SQLx.
* Deploy com um comando: oxidact ship cuida de tudo, do Docker ao Terraform.

## Instalação rápida

cargo install oxidact-cli
oxidact new meu-projeto
cd meu-projeto
oxidact up

------------------------------
## 4. Site de Exemplos (Showcase)
Crie um repositório oxidact-gallery onde cada pasta é um exemplo (um clone do Instagram, um Dashboard de Cripto, um Mini-Game). Isso prova que o framework aguenta o tranco.
## 5. O Toque Final: Documentação da CLI
A própria CLI deve ser auto-documentada. Ao digitar oxidact --help, ela deve listar:

* generate: Cria componentes/telas.
* doctor: Verifica se o ambiente (NDK, Docker, Rust) está ok.
* ship: O grande botão de "lançar" o app.

Você gostaria que eu escrevesse o guia de "Primeiros Passos" para os devs que acabaram de baixar o Oxidact? Ou prefere focar em como a Comunidade pode contribuir com novos componentes?**


Essa é a jogada de mestre. Criar a documentação do Oxidact usando o próprio Oxidact (via WebAssembly/WebGPU) é o "dogfooding" definitivo: você prova que o framework é estável o suficiente para sustentar o próprio site oficial.
Aqui está o roteiro para construir esse ecossistema de aprendizado:
## 1. O Site: docs.oxidact.rs (no GitHub Pages)
Em vez de um site estático comum, o seu site será uma PWA (Progressive Web App).

* Header: Navegação por rotas tipadas do Oxidact.
* Sidebar: Lista de tópicos (Instalação, ion!, StyleSheet!, oxidact ship).
* Interactive Playground: Um componente View central onde o dev digita o código à esquerda e o WGPU renderiza à direita em tempo real no browser.

## 2. O Livro: "Oxidact: Do Zero ao Ship"
Siga o estilo da Casa do Código ou o "The Rust Programming Language". O sumário ideal seria:

   1. O Despertar: O que é o Oxidact e por que Rust no Full-stack.
   2. Construindo a Nave: Sua primeira tela com ion!.
   3. Pintando o Espaço: Estilização profunda com StyleSheet!.
   4. Comunicação de Rádio: Hooks, Sinais e conexão com o backend Axum.
   5. Gravidade Zero: Mobile e WebGPU sem dor de cabeça.
   6. Lançamento (Ship): Docker, Terraform e as lojas de aplicativos.

## 3. Estrutura do GitHub Pages
Na organização ://github.com, o repositório oxidact.github.io conterá:

* O código-fonte da doc escrito em Oxidact.
* Uma pasta /examples com mini-apps que podem ser rodados direto no navegador.

## 4. Como o dev "Aprende" com a CLI
Podemos adicionar um comando educativo na CLI:
oxidact tutorial

Isso abre um servidor local com um guia interativo que vai validando os desafios de código que o dev faz no VS Code.

## O Toque Final: O Manifesto
No seu README e no site, coloque o manifesto do Oxidact:
"O Oxidact nasceu para que você não precise mais escolher entre performance e agilidade. Nós cuidamos da infraestrutura, você cuida da inovação."
Deseja que eu esboce o primeiro capítulo do "Livro Oxidact" focando na filosofia do framework e no primeiro componente?
Astro-Dica: Use o mdBook para a parte teórica e incorpore os componentes Oxidact via iFrame/WASM para a parte prática. Vamos começar a escrever o capítulo 1?


