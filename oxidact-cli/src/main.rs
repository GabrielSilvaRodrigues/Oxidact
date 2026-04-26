use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        print_help();
        return ExitCode::SUCCESS;
    };

    match cmd.as_str() {
        "check" => run_and_forward("cargo", &["check"]),
        "run" => handle_run(&args[1..]),
        "wasm" => run_and_forward("cargo", &["build", "--target", "wasm32-unknown-unknown"]),
        "web" => run_web(),
        "fmt" => run_and_forward("cargo", &["fmt"]),
        "doctor" => doctor(),
        "new" => handle_new(&args[1..]),
        "create" => handle_create(&args[1..]),
        "help" | "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        unknown => {
            eprintln!("Comando desconhecido: {unknown}");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn handle_run(args: &[String]) -> ExitCode {
    if args.is_empty() {
        return run_and_forward("cargo", &["run"]);
    }

    match args[0].as_str() {
        "web" => run_web(),
        "android" => run_target_check("aarch64-linux-android"),
        "ios" => run_target_check("aarch64-apple-ios"),
        "windows" => run_target_check("x86_64-pc-windows-gnu"),
        "linux" => run_and_forward("cargo", &["run"]),
        _ => {
            eprintln!("Plataforma de run desconhecida: {}", args[0]);
            ExitCode::from(2)
        }
    }
}

fn handle_new(args: &[String]) -> ExitCode {
    if args.len() < 2 {
        eprintln!("Uso: oxidact new <screen|component|style> <Nome>");
        return ExitCode::from(2);
    }

    let kind = args[0].as_str();
    let name = &args[1];

    let result = match kind {
        "page" => new_page(name),
        "screen" => new_screen(name),
        "component" => new_component(name),
        "style" => new_style(name),
        _ => Err(format!("Tipo de geracao desconhecido: {kind}")),
    };

    match result {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Erro: {err}");
            ExitCode::from(1)
        }
    }
}

fn handle_create(args: &[String]) -> ExitCode {
    if args.len() < 2 {
        eprintln!("Uso: oxidact create <project|platform> <Nome>");
        return ExitCode::from(2);
    }

    let kind = args[0].as_str();
    let name = &args[1];

    let result = match kind {
        "project" => create_project(name),
        "platform" => create_platform(name),
        _ => Err(format!("Tipo de create desconhecido: {kind}")),
    };

    match result {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Erro: {err}");
            ExitCode::from(1)
        }
    }
}

fn run_and_forward(bin: &str, args: &[&str]) -> ExitCode {
    let status = Command::new(bin).args(args).status();

    match status {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(err) => {
            eprintln!("Falha ao executar {bin}: {err}");
            ExitCode::from(1)
        }
    }
}

fn run_web() -> ExitCode {
    if Path::new("scripts/platforms.sh").exists() {
        return run_and_forward("bash", &["scripts/platforms.sh", "web"]);
    }

    let build = run_and_forward("wasm-pack", &["build", "--target", "web", "--out-dir", "pkg"]);
    if build != ExitCode::SUCCESS {
        return build;
    }

    run_and_forward("python3", &["-m", "http.server", "8080"])
}

fn run_target_check(target: &str) -> ExitCode {
    let add = run_and_forward("rustup", &["target", "add", target]);
    if add != ExitCode::SUCCESS {
        return add;
    }
    run_and_forward("cargo", &["check", "--target", target])
}

fn new_screen(name: &str) -> Result<String, String> {
    let raw = name.trim();
    if raw.is_empty() {
        return Err("Nome da screen vazio".to_string());
    }

    let snake = to_snake_case(raw);
    let pascal = to_pascal_case(raw);
    let route = snake.replace('_', "-");

    let screen_file = format!("src/screens/{snake}.rs");
    if Path::new(&screen_file).exists() {
        return Err(format!("Arquivo ja existe: {screen_file}"));
    }

    let fn_name = format!("build_{snake}_screen");
    let screen_source = format!(
        "use oxidact_macros::rsx;\n\n\
pub fn {fn_name}() -> oxidact_core::VNode {{\n\
    rsx!(\n\
        <SafeAreaView style={{\"flex: 1; background: #0f172a\"}}>\n\
            <View style={{\"padding: 20; justify-content: center\"}}>\n\
                <Text style={{\"font-size: 24; color: #ffffff\"}}>\"{pascal}\"</Text>\n\
            </View>\n\
        </SafeAreaView>\n\
    )\n\
}}\n"
    );
    write_file_create(&screen_file, &screen_source)?;

    let mut screens_mod = read_to_string("src/screens/mod.rs")?;
    let mod_line = format!("mod {snake};");
    let use_line = format!("pub use {snake}::{fn_name};");
    if !screens_mod.contains(&mod_line) {
        screens_mod.push_str(&format!("\n{mod_line}"));
    }
    if !screens_mod.contains(&use_line) {
        screens_mod.push_str(&format!("\n{use_line}"));
    }
    write_file_overwrite("src/screens/mod.rs", &screens_mod)?;

    let mut nav_mod = read_to_string("src/navigation/mod.rs")?;
    let variant_line = format!("    {pascal},");
    if !nav_mod.contains(&variant_line) {
        nav_mod = insert_in_enum(&nav_mod, "pub enum Screen", &variant_line)?;
    }

    let stack_screen_block = format!(
        "                <StackScreen name=\"{pascal}\" route=\"{route}\" headerShown=\"false\">\n                    {{crate::screens::{fn_name}()}}\n                </StackScreen>\n"
    );
    if !nav_mod.contains(&format!("name=\"{pascal}\"")) {
        nav_mod = nav_mod.replacen("            </StackNavigator>", &format!("{stack_screen_block}            </StackNavigator>"), 1);
    }

    let search_condition = format!("search.to_lowercase().contains(\"screen={route}\")");
    let else_if_block = format!(
        "    }} else if {} {{\n        Screen::{}\n    }} else {{",
        search_condition, pascal
    );
    if !nav_mod.contains(&search_condition) {
        nav_mod = nav_mod.replacen("    } else {", &else_if_block, 1);
    }

    let query_line = format!("        Screen::{pascal} => \"?screen={route}\",");
    if !nav_mod.contains(&query_line) {
        nav_mod = nav_mod.replacen("    }\n}\n\npub fn initial_route_name", &format!("{query_line}\n    }}\n}}\n\npub fn initial_route_name"), 1);
    }

    let route_line = format!("        Screen::{pascal} => \"{pascal}\",");
    if !nav_mod.contains(&route_line) {
        nav_mod = nav_mod.replacen("    }\n}", &format!("{route_line}\n    }}\n}}"), 1);
    }

    write_file_overwrite("src/navigation/mod.rs", &nav_mod)?;

    Ok(format!("Screen criada: {pascal} ({screen_file}) e rota registrada automaticamente."))
}

fn new_component(name: &str) -> Result<String, String> {
    let raw = name.trim();
    if raw.is_empty() {
        return Err("Nome do component vazio".to_string());
    }

    let snake = to_snake_case(raw);
    let pascal = to_pascal_case(raw);
    let component_file = format!("src/components/{snake}.rs");
    if Path::new(&component_file).exists() {
        return Err(format!("Arquivo ja existe: {component_file}"));
    }

    let fn_name = format!("build_{snake}");
    let source = format!(
        "use oxidact_macros::rsx;\n\n\
pub fn {fn_name}() -> oxidact_core::VNode {{\n\
    rsx!(\n\
        <View style={{\"padding: 12\"}}>\n\
            <Text>\"{pascal}\"</Text>\n\
        </View>\n\
    )\n\
}}\n"
    );
    write_file_create(&component_file, &source)?;

    let mut components_mod = read_to_string("src/components/mod.rs")?;
    let line = format!("pub mod {snake};");
    if !components_mod.contains(&line) {
        components_mod.push_str(&format!("\n{line}"));
        write_file_overwrite("src/components/mod.rs", &components_mod)?;
    }

    Ok(format!("Component criado: {pascal} ({component_file})."))
}

fn new_style(name: &str) -> Result<String, String> {
    let raw = name.trim();
    if raw.is_empty() {
        return Err("Nome do style vazio".to_string());
    }

    let snake = to_snake_case(raw);
    let pascal = to_pascal_case(raw);
    let upper = snake.to_uppercase();
    let style_file = format!("src/styles/{snake}.rs");
    if Path::new(&style_file).exists() {
        return Err(format!("Arquivo ja existe: {style_file}"));
    }

    let source = format!(
        "pub struct {pascal}Styles;\n\
pub const {upper}: {pascal}Styles = {pascal}Styles;\n\n\
impl {pascal}Styles {{\n\
    pub fn container(&self) -> &'static str {{\n\
        \"flex: 1\"\n\
    }}\n\
}}\n"
    );
    write_file_create(&style_file, &source)?;

    let mut styles_mod = read_to_string("src/styles/mod.rs")?;
    let line = format!("pub mod {snake};");
    if !styles_mod.contains(&line) {
        styles_mod.push_str(&format!("\n{line}"));
        write_file_overwrite("src/styles/mod.rs", &styles_mod)?;
    }

    Ok(format!("Style criado: {pascal} ({style_file})."))
}

fn new_page(name: &str) -> Result<String, String> {
    let raw = name.trim();
    if raw.is_empty() {
        return Err("Nome da page vazio".to_string());
    }

    let snake = to_snake_case(raw);
    let pascal = to_pascal_case(raw);
    let route = snake.replace('_', "-");

    let style_file = format!("src/styles/{snake}.rs");
    if Path::new(&style_file).exists() {
        return Err(format!("Arquivo ja existe: {style_file}"));
    }

    let component_file = format!("src/components/{snake}.rs");
    if Path::new(&component_file).exists() {
        return Err(format!("Arquivo ja existe: {component_file}"));
    }

    let screen_file = format!("src/screens/{snake}.rs");
    if Path::new(&screen_file).exists() {
        return Err(format!("Arquivo ja existe: {screen_file}"));
    }

    let style_mod = format!("pub struct {pascal}Styles;\npub const {pascal_upper}: {pascal}Styles = {pascal}Styles;\n\nimpl {pascal}Styles {{\n    pub fn container(&self) -> &'static str {{\n        \"flex: 1; background: #0f172a\"\n    }}\n\n    pub fn title(&self) -> &'static str {{\n        \"font-size: 24; color: #ffffff\"\n    }}\n\n    pub fn subtitle(&self) -> &'static str {{\n        \"color: #94a3b8; margin-top: 8\"\n    }}\n}}\n", pascal_upper = snake.to_uppercase());
    write_file_create(&style_file, &style_mod)?;

    let component_fn = format!("build_{snake}");
    let component_mod = format!(
        "use oxidact_macros::rsx;\nuse crate::styles::{snake}::{pascal_upper};\n\npub fn {component_fn}() -> oxidact_core::VNode {{\n    rsx!(\n        <View style={{ {pascal_upper}.container() }}>\n            <Text style={{ {pascal_upper}.title() }}>\"{pascal}\"</Text>\n            <Text style={{ {pascal_upper}.subtitle() }}>\"Tela criada com oxidact new page\"</Text>\n        </View>\n    )\n}}\n",
        pascal_upper = snake.to_uppercase()
    );
    write_file_create(&component_file, &component_mod)?;

    let screen_fn = format!("build_{snake}_screen");
    let screen_mod = format!(
        "use oxidact_macros::rsx;\nuse crate::components::{snake}::{component_fn};\nuse crate::styles::{snake}::{pascal_upper};\n\npub fn {screen_fn}() -> oxidact_core::VNode {{\n    rsx!(\n        <SafeAreaView style={{ {pascal_upper}.container() }}>\n            <View style={{\"padding: 20\" }}>\n                {{ {component_fn}() }}\n            </View>\n        </SafeAreaView>\n    )\n}}\n",
        pascal_upper = snake.to_uppercase()
    );
    write_file_create(&screen_file, &screen_mod)?;

    let mut screens_mod = read_to_string("src/screens/mod.rs")?;
    screens_mod = append_unique_line(&screens_mod, &format!("mod {snake};"));
    screens_mod = append_unique_line(&screens_mod, &format!("pub use {snake}::{screen_fn};"));
    write_file_overwrite("src/screens/mod.rs", &screens_mod)?;

    let mut components_mod = read_to_string("src/components/mod.rs")?;
    components_mod = append_unique_line(&components_mod, &format!("pub mod {snake};"));
    write_file_overwrite("src/components/mod.rs", &components_mod)?;

    let mut styles_mod = read_to_string("src/styles/mod.rs")?;
    styles_mod = append_unique_line(&styles_mod, &format!("pub mod {snake};"));
    write_file_overwrite("src/styles/mod.rs", &styles_mod)?;

    let mut nav_mod = read_to_string("src/navigation/mod.rs")?;
    nav_mod = insert_enum_variant(&nav_mod, "pub enum Screen", &format!("    {pascal},"))?;
    nav_mod = insert_before_marker(
        &nav_mod,
        "            </StackNavigator>",
        &format!(
            "                <StackScreen name=\"{pascal}\" route=\"{route}\" headerShown=\"false\">\n                    {{crate::screens::{screen_fn}()}}\n                </StackScreen>\n"
        ),
    )?;
    nav_mod = insert_before_marker(
        &nav_mod,
        "    } else {\n        Screen::Login\n    }",
        &format!(
            "    }} else if search.to_lowercase().contains(\"screen={route}\") {{\n        Screen::{pascal}\n    }} else {{"
        ),
    )?;
    nav_mod = insert_before_marker(
        &nav_mod,
        "    }\n}\n\npub fn initial_route_name()",
        &format!(
            "        Screen::{pascal} => \"?screen={route}\",\n"
        ),
    )?;
    nav_mod = insert_before_marker(
        &nav_mod,
        "    }\n}\n",
        &format!(
            "        Screen::{pascal} => \"{pascal}\",\n"
        ),
    )?;
    write_file_overwrite("src/navigation/mod.rs", &nav_mod)?;

    Ok(format!("Page criada: {pascal} ({screen_file}, {component_file}, {style_file}) e rota registrada."))
}

fn create_project(name: &str) -> Result<String, String> {
    let project_name = to_snake_case(name);
    let project_dir = PathBuf::from(name);
    if project_dir.exists() {
        return Err(format!("Diretorio ja existe: {}", project_dir.display()));
    }

    fs::create_dir_all(project_dir.join("src")).map_err(|e| e.to_string())?;
    fs::create_dir_all(project_dir.join("web")).map_err(|e| e.to_string())?;

    let cargo_toml = format!(
        "[package]\nname = \"{project_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\noxidact-core = \"0.1.0\"\noxidact-macros = \"0.1.0\"\n"
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|e| e.to_string())?;

    fs::write(
        project_dir.join("src/main.rs"),
        "use oxidact_macros::rsx;\n\nfn main() {\n    let app = rsx!(\n        <SafeAreaView style=\"flex: 1; background: #0b1220\">\n            <View style=\"padding: 24; justify-content: center\">\n                <Text style=\"font-size: 24; color: #ffffff\">\"Novo app Oxidact\"</Text>\n            </View>\n        </SafeAreaView>\n    );\n\n    oxidact_core::run(app);\n}\n",
    )
    .map_err(|e| e.to_string())?;

    fs::write(
        project_dir.join("web/index.html"),
        "<!doctype html>\n<html lang=\"pt-br\">\n  <head><meta charset=\"utf-8\"/><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"/><title>Oxidact</title></head>\n  <body><h1>Projeto Oxidact criado</h1></body>\n</html>\n",
    )
    .map_err(|e| e.to_string())?;

    fs::write(
        project_dir.join("README.md"),
        "# Novo projeto Oxidact\n\nComandos:\n- cargo run\n- oxidact run web\n",
    )
    .map_err(|e| e.to_string())?;

    Ok(format!("Projeto criado em {}", project_dir.display()))
}

fn create_platform(platform: &str) -> Result<String, String> {
    let platform = platform.to_lowercase();
    let dir = PathBuf::from(&platform);
    if dir.exists() {
        return Err(format!("Pasta da plataforma ja existe: {}", dir.display()));
    }

    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    match platform.as_str() {
        "android" => {
            fs::write(dir.join("README.md"), "# Android app shell\n\nIntegre aqui o bootstrap Android do Oxidact.\n")
                .map_err(|e| e.to_string())?;
            fs::write(dir.join("AndroidManifest.xml"), "<manifest package=\"com.oxidact.app\"></manifest>\n")
                .map_err(|e| e.to_string())?;
        }
        "ios" => {
            fs::write(dir.join("README.md"), "# iOS app shell\n\nIntegre aqui o bootstrap iOS do Oxidact.\n")
                .map_err(|e| e.to_string())?;
            fs::write(dir.join("Info.plist"), "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><dict></dict></plist>\n")
                .map_err(|e| e.to_string())?;
        }
        "windows" => {
            fs::write(dir.join("README.md"), "# Windows app shell\n\nIntegre aqui o bootstrap Windows do Oxidact.\n")
                .map_err(|e| e.to_string())?;
            fs::write(dir.join("app.manifest"), "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<assembly></assembly>\n")
                .map_err(|e| e.to_string())?;
        }
        "linux" => {
            fs::write(dir.join("README.md"), "# Linux app shell\n\nIntegre aqui o bootstrap Linux do Oxidact.\n")
                .map_err(|e| e.to_string())?;
            fs::write(dir.join("app.desktop"), "[Desktop Entry]\nName=Oxidact App\nType=Application\nExec=oxidact-app\n")
                .map_err(|e| e.to_string())?;
        }
        _ => {
            return Err("Plataforma suportada: android, ios, windows, linux".to_string());
        }
    }

    Ok(format!("Scaffold de plataforma criado: {}", dir.display()))
}

fn read_to_string(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Falha ao ler {path}: {e}"))
}

fn write_file_overwrite(path: &str, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Falha ao escrever {path}: {e}"))
}

fn write_file_create(path: &str, content: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Falha ao criar pasta {}: {e}", parent.display()))?;
    }
    fs::write(path, content).map_err(|e| format!("Falha ao criar {path}: {e}"))
}

fn insert_in_enum(content: &str, enum_header: &str, variant_line: &str) -> Result<String, String> {
    let enum_start = content
        .find(enum_header)
        .ok_or_else(|| format!("Enum nao encontrada: {enum_header}"))?;
    let enum_body = &content[enum_start..];
    let close_rel = enum_body
        .find("}\n\n")
        .ok_or_else(|| "Nao foi possivel localizar fim da enum".to_string())?;
    let insert_at = enum_start + close_rel;
    let mut out = content.to_string();
    out.insert_str(insert_at, &format!("{variant_line}\n"));
    Ok(out)
}

fn insert_enum_variant(content: &str, enum_header: &str, variant_line: &str) -> Result<String, String> {
    let enum_start = content
        .find(enum_header)
        .ok_or_else(|| format!("Enum nao encontrada: {enum_header}"))?;
    let enum_tail = &content[enum_start..];
    let close = enum_tail
        .find("}\n")
        .ok_or_else(|| "Nao foi possivel localizar fim da enum".to_string())?;
    let insert_at = enum_start + close;
    let mut out = content.to_string();
    out.insert_str(insert_at, &format!("{variant_line}\n"));
    Ok(out)
}

fn insert_before_marker(content: &str, marker: &str, insert_text: &str) -> Result<String, String> {
    let Some(pos) = content.find(marker) else {
        return Err(format!("Marcador nao encontrado: {marker}"));
    };
    let mut out = content.to_string();
    out.insert_str(pos, insert_text);
    Ok(out)
}

fn append_unique_line(content: &str, line: &str) -> String {
    if content.contains(line) {
        content.to_string()
    } else if content.ends_with('\n') {
        format!("{content}{line}\n")
    } else {
        format!("{content}\n{line}\n")
    }
}

fn to_snake_case(input: &str) -> String {
    let mut out = String::new();
    let mut prev_is_sep = true;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() {
                if !prev_is_sep && !out.is_empty() {
                    out.push('_');
                }
                out.push(ch.to_ascii_lowercase());
            } else {
                out.push(ch);
            }
            prev_is_sep = false;
        } else if !prev_is_sep {
            out.push('_');
            prev_is_sep = true;
        }
    }

    out.trim_matches('_').to_string()
}

fn to_pascal_case(input: &str) -> String {
    input
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut s = String::new();
                    s.push(first.to_ascii_uppercase());
                    s.push_str(&chars.as_str().to_ascii_lowercase());
                    s
                }
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn doctor() -> ExitCode {
    let checks = [
        ("cargo", &["--version"] as &[&str]),
        ("rustc", &["--version"]),
        ("wasm-pack", &["--version"]),
    ];

    let mut ok = true;

    for (bin, args) in checks {
        match Command::new(bin).args(args).output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("[ok] {bin}: {}", version.trim());
            }
            Ok(output) => {
                ok = false;
                let err = String::from_utf8_lossy(&output.stderr);
                println!("[erro] {bin}: {}", err.trim());
            }
            Err(_) => {
                ok = false;
                println!("[erro] {bin}: não encontrado no PATH");
            }
        }
    }

    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn print_help() {
    println!("Oxidact CLI");
    println!();
    println!("Uso:");
    println!("  oxidact check                           # roda cargo check");
    println!("  oxidact run [web|android|ios|windows]  # executa fluxo por plataforma");
    println!("  oxidact web                             # build web + servidor local");
    println!("  oxidact wasm                            # build wasm32-unknown-unknown");
    println!("  oxidact fmt                             # formata o workspace");
    println!("  oxidact doctor                          # valida ferramentas instaladas");
    println!("  oxidact new screen <Nome>               # cria screen + registra rota");
    println!("  oxidact new page <Nome>                 # cria screen + component + style + rota");
    println!("  oxidact new component <Nome>            # cria componente");
    println!("  oxidact new style <Nome>                # cria modulo de style");
    println!("  oxidact create project <Nome>           # cria novo projeto");
    println!("  oxidact create platform <plataforma>    # android|ios|windows|linux");
}
