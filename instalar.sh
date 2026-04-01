#!/bin/bash

# ║           Twitch TTS Bot — Instalador automático            ║
# ║                    Arch Linux / Garuda                      ║
# ╚══════════════════════════════════════════════════════════════╝

set -e  # Para si algo falla

VERDE="\e[32m"
ROJO="\e[31m"
AMARILLO="\e[33m"
MORADO="\e[35m"
RESET="\e[0m"
NEGRITA="\e[1m"

ok()   { echo -e "${VERDE}${NEGRITA}  ✓  $1${RESET}"; }
info() { echo -e "${MORADO}  ▶  $1${RESET}"; }
warn() { echo -e "${AMARILLO}  ⚠  $1${RESET}"; }
err()  { echo -e "${ROJO}${NEGRITA}  ✗  $1${RESET}"; exit 1; }

echo -e "${MORADO}${NEGRITA}"
echo "  ████████╗████████╗███████╗    ██████╗  ██████╗ ████████╗"
echo "     ██╔══╝╚══██╔══╝██╔════╝    ██╔══██╗██╔═══██╗╚══██╔══╝"
echo "     ██║      ██║   ███████╗    ██████╔╝██║   ██║   ██║   "
echo "     ██║      ██║   ╚════██║    ██╔══██╗██║   ██║   ██║   "
echo "     ██║      ██║   ███████║    ██████╔╝╚██████╔╝   ██║   "
echo "     ╚═╝      ╚═╝   ╚══════╝    ╚═════╝  ╚═════╝    ╚═╝   "
echo -e "${RESET}"
echo -e "${MORADO}           Twitch TTS Bot — Instalador v1.0${RESET}"
echo -e "${MORADO}           Arch Linux / Garuda${RESET}"
echo ""

PROYECTO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PIPER_DIR="$PROYECTO_DIR/files/piper"
SRC_DIR="$PROYECTO_DIR/files"

info "Directorio del proyecto: $PROYECTO_DIR"
echo ""

echo -e "${NEGRITA}── Paso 1/6  Dependencias del sistema ──${RESET}"

PAQUETES=(rust alsa-lib pkg-config base-devel python python-pip wget)
FALTANTES=()

for paq in "${PAQUETES[@]}"; do
    if ! pacman -Qi "$paq" &>/dev/null; then
        FALTANTES+=("$paq")
    fi
done

if [ ${#FALTANTES[@]} -eq 0 ]; then
    ok "Todas las dependencias ya están instaladas"
else
    info "Instalando: ${FALTANTES[*]}"
    sudo pacman -S --needed --noconfirm "${FALTANTES[@]}"
    ok "Dependencias instaladas"
fi
echo ""

echo -e "${NEGRITA}── Paso 2/6  Rust ──${RESET}"

if command -v cargo &>/dev/null; then
    VERSION=$(cargo --version)
    ok "Rust ya instalado: $VERSION"
else
    info "Instalando Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
    ok "Rust instalado: $(cargo --version)"
fi
echo ""

echo -e "${NEGRITA}── Paso 3/6  gTTS (fallback de voz) ──${RESET}"

if python3 -c "import gtts" &>/dev/null; then
    ok "gTTS ya instalado"
else
    info "Instalando gTTS..."
    pip install gtts --break-system-packages -q
    ok "gTTS instalado"
fi
echo ""

echo -e "${NEGRITA}── Paso 4/6  Piper TTS ──${RESET}"

mkdir -p "$PIPER_DIR"

if [ -f "$PIPER_DIR/piper/piper" ]; then
    ok "Piper ya descargado"
else
    info "Descargando Piper TTS..."
    wget -q --show-progress \
        "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz" \
        -O "$PIPER_DIR/piper_linux_x86_64.tar.gz"

    info "Extrayendo..."
    tar -xzf "$PIPER_DIR/piper_linux_x86_64.tar.gz" -C "$PIPER_DIR"
    rm "$PIPER_DIR/piper_linux_x86_64.tar.gz"
    ok "Piper extraído"
fi

info "Aplicando permisos de ejecución..."

chmod a+x "$PIPER_DIR/piper/piper"
ok "  piper  →  +x"

chmod a+x "$PIPER_DIR/piper/piper_phonemize"
ok "  piper_phonemize  →  +x"

for lib in "$PIPER_DIR/piper/"*.so*; do
    [ -f "$lib" ] || continue
    chmod a+x "$lib"
    ok "  $(basename "$lib")  →  +x"
done

echo ""

echo -e "${NEGRITA}── Paso 5/6  Modelo de voz español ──${RESET}"

MODELO_ONNX="$PIPER_DIR/es_ES-sharvard-medium.onnx"
MODELO_JSON="$PIPER_DIR/es_ES-sharvard-medium.onnx.json"
BASE_URL="https://huggingface.co/rhasspy/piper-voices/resolve/main/es/es_ES/sharvard/medium"

if [ -f "$MODELO_ONNX" ] && [ -f "$MODELO_JSON" ]; then
    ok "Modelo de voz ya descargado"
else
    info "Descargando modelo es_ES-sharvard-medium..."
    wget -q --show-progress "$BASE_URL/es_ES-sharvard-medium.onnx"      -O "$MODELO_ONNX"
    wget -q --show-progress "$BASE_URL/es_ES-sharvard-medium.onnx.json" -O "$MODELO_JSON"
    ok "Modelo descargado"
fi

info "Verificando Piper con el modelo..."
TEST_RESULTADO=$(LD_LIBRARY_PATH="$PIPER_DIR/piper" \
    bash -c "echo 'hola' | '$PIPER_DIR/piper/piper' \
    --model '$MODELO_ONNX' \
    --output_file /tmp/tts_test.wav 2>&1")

if [ -f "/tmp/tts_test.wav" ]; then
    ok "Piper funciona correctamente"
    rm /tmp/tts_test.wav
else
    warn "Piper generó un aviso (puede ser normal): $TEST_RESULTADO"
fi
echo ""

echo -e "${NEGRITA}── Paso 6/6  Compilando el bot ──${RESET}"

info "Compilando (puede tardar unos minutos la primera vez)..."
cd "$SRC_DIR"
cargo build --release 2>&1 | grep -E "Compiling|Finished|error" || true

if [ -f "$SRC_DIR/target/release/bot" ] && [ -f "$SRC_DIR/target/release/config-ui" ]; then
    ok "Compilación exitosa"
else
    err "La compilación falló. Ejecuta 'cargo build --release' manualmente para ver el error completo."
fi
echo ""

echo -e "${MORADO}${NEGRITA}══════════════════════════════════════════${RESET}"
echo -e "${VERDE}${NEGRITA}  ✓  Instalación completada${RESET}"
echo -e "${MORADO}${NEGRITA}══════════════════════════════════════════${RESET}"
echo ""
echo -e "  ${NEGRITA}Modelo de voz:${RESET}"
echo -e "  ${AMARILLO}$MODELO_ONNX${RESET}"
echo ""
echo -e "  ${NEGRITA}Pasos siguientes:${RESET}"
echo ""
echo -e "  ${MORADO}1.${RESET} Abre la configuración:"
echo -e "     ${AMARILLO}$SRC_DIR/target/release/config-ui${RESET}"
echo ""
echo -e "  ${MORADO}2.${RESET} En el campo ${NEGRITA}Modelo Piper${RESET} escribe:"
echo -e "     ${AMARILLO}$MODELO_ONNX${RESET}"
echo ""
echo -e "  ${MORADO}3.${RESET} Arranca el bot:"
echo -e "     ${AMARILLO}$SRC_DIR/target/release/bot${RESET}"
echo ""
