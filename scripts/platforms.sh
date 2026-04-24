#!/usr/bin/env bash
set -euo pipefail

cmd="${1:-help}"

case "$cmd" in
  desktop)
    cargo run
    ;;
  web-build)
    wasm-pack build --target web --out-dir pkg
    ;;
  web-serve)
    python3 -m http.server 8080
    ;;
  web)
    wasm-pack build --target web --out-dir pkg
    python3 -m http.server 8080
    ;;
  android-check)
    rustup target add aarch64-linux-android
    cargo check --target aarch64-linux-android
    ;;
  ios-check)
    rustup target add aarch64-apple-ios
    if ! command -v xcrun >/dev/null 2>&1; then
      echo "iOS check exige Xcode/xcrun (macOS)."
      echo "No Linux, valide Android aqui e rode iOS check em uma maquina macOS."
      exit 2
    fi
    cargo check --target aarch64-apple-ios
    ;;
  all-check)
    cargo check
    rustup target add wasm32-unknown-unknown
    cargo check --target wasm32-unknown-unknown
    ;;
  *)
    cat <<'EOF'
Uso: scripts/platforms.sh <comando>

Comandos:
  desktop       Executa no desktop
  web-build     Gera bundle web com wasm-pack
  web-serve     Sobe servidor local na porta 8080
  web           Build web + servidor local
  android-check Verifica compilacao para Android (aarch64)
  ios-check     Verifica compilacao para iOS (aarch64)
  all-check     Checa desktop + wasm32
EOF
    ;;
esac
