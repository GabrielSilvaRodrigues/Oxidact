use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

// ==============================
// ENTRY POINT
// ==============================
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let command = args[1].as_str();

    let result = match command {
        "doctor" => doctor(),
        "new" => handle_new(&args),
        "web" => run_web(),
        _ => {
            println!("Comando desconhecido: {}", command);
            print_help();
            Ok(())
        }
    };

    if let Err(err) = result {
        eprintln!("Erro: {}", err);
    }
}

// ==============================
// COMMANDS
// ==============================

fn doctor() -> Result<(), String> {
    check_command("cargo", "[ok] cargo");
    check_command("rustc", "[ok] rustc");
    check_command("wasm-pack", "[ok] wasm-pack");
    Ok(())
}

fn handle_new(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        return Err("Uso: oxidact new screen <Nome>".into());
    }

    let sub = args[2].as_str();

    match sub {
        "screen" => {
            let name = &args[3];
            create_screen(name)
        }
        _ => Err("Subcomando desconhecido".into()),
    }
}

fn run_web() -> Result<(), String> {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "scripts\\platforms.bat"])
            .status()
    } else {
        Command::new("bash")
            .arg("scripts/platforms.sh")
            .status()
    };

    match status {
        Ok(s) if s.success() => {
            println!("[ok] Web iniciado");
            Ok(())
        }
        Ok(_) => Err("Script executado com erro".into()),
        Err(e) => Err(format!("Falha ao executar script: {}", e)),
    }
}

// ==============================
// CORE LOGIC
// ==============================

fn create_screen(name: &str) -> Result<(), String> {
    let variant = name.trim();

    if variant.is_empty() {
        return Err("Nome da screen nao pode ser vazio".into());
    }

    insert_in_enum(
        "src/navigation/mod.rs",
        "Screen",
        variant,
    )?;

    println!("[ok] Screen '{}' criada com sucesso!", variant);

    Ok(())
}

fn insert_in_enum(
    file_path: &str,
    enum_name: &str,
    new_variant: &str,
) -> Result<(), String> {
    let path = Path::new(file_path);

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Erro ao ler arquivo: {}", e))?;

    let enum_start = content
        .find(&format!("enum {}", enum_name))
        .ok_or_else(|| format!("Enum '{}' nao encontrada", enum_name))?;

    let enum_body = &content[enum_start..];

    // 🔥 CORREÇÃO: robusto
    let close_rel = enum_body
        .rfind('}')
        .ok_or_else(|| "Nao foi possivel localizar fim da enum".to_string())?;

    let insert_pos = enum_start + close_rel;

    if content.contains(new_variant) {
        return Err("Variant ja existe na enum".into());
    }

    let new_content = format!(
        "{}    {},\n{}",
        &content[..insert_pos],
        new_variant,
        &content[insert_pos..]
    );

    fs::write(path, new_content)
        .map_err(|e| format!("Erro ao escrever arquivo: {}", e))?;

    Ok(())
}

// ==============================
// UTILS
// ==============================

fn check_command(cmd: &str, label: &str) {
    let result = Command::new(cmd).arg("--version").output();

    match result {
        Ok(_) => println!("{}: OK", label),
        Err(_) => println!("{}: NÃO ENCONTRADO", label),
    }
}

fn print_help() {
    println!("Oxidact CLI");
    println!();
    println!("Comandos:");
    println!("  doctor                Verifica dependencias");
    println!("  new screen <Nome>     Cria uma nova screen");
    println!("  web                   Inicia ambiente web");
}