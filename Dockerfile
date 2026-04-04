# ─────────────────────────────────────────
#  Etapa 1: compilación
# ─────────────────────────────────────────
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libasound2-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiamos solo los manifiestos primero para aprovechar la caché de capas
COPY Cargo.toml Cargo.lock ./
# Trick para cachear dependencias: compilar un src vacío primero
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    echo 'fn main() {}' > src/ui.rs && \
    cargo build --release && \
    rm -rf src

# Ahora copiamos el código real
COPY src/ src/
# Forzamos recompilación del código propio
RUN touch src/main.rs src/ui.rs && cargo build --release

# ─────────────────────────────────────────
#  Etapa 2: imagen final (más ligera)
# ─────────────────────────────────────────
FROM rust:latest

RUN apt-get update && apt-get install -y \
    # Audio
    alsa-utils \
    pulseaudio-utils \
    # Python + gTTS (fallback)
    python3 python3-pip \
    # Utilidades
    wget bash \
    && pip3 install gtts --break-system-packages -q \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Binarios compilados
COPY --from=builder /app/target/release/bot        ./bot
COPY --from=builder /app/target/release/config-ui  ./config-ui

# ── Piper TTS ─────────────────────────────────────────────────────
RUN mkdir -p /app/files/piper

RUN wget -q \
    "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz" \
    -O /tmp/piper.tar.gz \
    && tar -xzf /tmp/piper.tar.gz -C /app/files/piper \
    && rm /tmp/piper.tar.gz \
    && chmod +x /app/files/piper/piper/piper \
    && chmod +x /app/files/piper/piper/piper_phonemize

# ── Modelo de voz español ─────────────────────────────────────────
ENV BASE_URL="https://huggingface.co/rhasspy/piper-voices/resolve/main/es/es_ES/sharvard/medium"

RUN wget -q "${BASE_URL}/es_ES-sharvard-medium.onnx"      -O /app/files/piper/es_ES-sharvard-medium.onnx \
    && wget -q "${BASE_URL}/es_ES-sharvard-medium.onnx.json" -O /app/files/piper/es_ES-sharvard-medium.onnx.json

# Variable de entorno para que main.rs encuentre Piper
ENV PIPER_DIR=/app/files/piper/piper

# ── Voces de referencia para XTTS-v2 ──────────────────────────────
# WAVs de ~10s incluidos en el repo, usados por los perfiles Coqui
COPY voices/ /app/voices/

# La config se guarda en ~/.config/twitch-tts/config.json dentro del contenedor
# Se monta como volumen para que persista entre reinicios

CMD ["./bot"]
