# Twitch TTS Bot

A **Text-to-Speech (TTS) bot for Twitch**, written in **Rust**, with a terminal-based configuration interface built using **Ratatui**.

Reads chat messages aloud through your system audio and lets the streamer control the volume via chat commands.

![preview](preview.png)

---

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
  - [Docker (recommended)](#docker-recommended)
  - [Manual install — Arch Linux](#manual-install--arch-linux)
- [Configuration](#configuration)
- [Running the Bot](#running-the-bot)
- [Chat Commands](#chat-commands)
- [Technologies](#technologies)

---

## Features

- Text-to-Speech playback of Twitch chat messages
- Spanish voice powered by [Piper TTS](https://github.com/rhasspy/piper) (offline, no API key needed)
- Fallback to gTTS if Piper fails
- Volume control via chat commands
- Configurable message length limit and audio output device
- Terminal-based configuration UI (no config file editing needed)

---

## Requirements

### Docker
- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/)
- PulseAudio running on the host (standard on most Linux desktops)

### Manual (Arch Linux)
- Rust / Cargo
- `alsa-lib`, `pkg-config`, `base-devel`
- Python 3 + pip (for gTTS fallback)

---

## Installation

### Docker (recommended)

No need to install Rust or any dependencies. Everything is handled inside the container.

**1. Clone the repository**

```bash
git clone https://github.com/CroquetaDeArroz/Bot-TTS-Rust.git
cd Bot-TTS-Rust
```

**2. Get a Twitch token**

1. Go to: https://twitchtokengenerator.com/
2. Log in with your **bot account**
3. Copy the `ACCESS TOKEN`

**3. Configure the bot**

```bash
docker compose --profile config run --rm config-ui
```

Fill in all fields and press `Ctrl+S` to save. See [Configuration](#configuration) for details.

**4. Run the bot**

```bash
docker compose up --build
```

On the first run, Docker will automatically download and install Piper TTS and the Spanish voice model — no manual steps needed. Subsequent builds are fast thanks to Docker layer caching.

To run in the background:

```bash
docker compose up --build -d
```

To stop:

```bash
docker compose down
```

---

### Manual install — Arch Linux

**1. Clone the repository**

```bash
git clone https://github.com/CroquetaDeArroz/Bot-TTS-Rust.git
cd Bot-TTS-Rust
```

**2. Run the installer**

```bash
chmod +x instalar.sh
./instalar.sh
```

The script will:
- Install system dependencies (`rust`, `alsa-lib`, `python`, etc.)
- Download Piper TTS binary
- Download the Spanish voice model (`es_ES-sharvard-medium`)
- Compile the bot and config UI

**3. Get a Twitch token**

1. Go to: https://twitchtokengenerator.com/
2. Log in with your **bot account**
3. Copy the `ACCESS TOKEN`

**4. Configure and run** — see sections below.

---

## Configuration

Run the configuration UI:

```bash
# Docker
docker compose --profile config run --rm config-ui

# Local
./files/target/release/config-ui
```

Navigate with `↑↓`, adjust values with `←→`, edit text fields with `Enter`. Save with `Ctrl+S`.

| Field | Description |
|---|---|
| Usuario | Your Twitch bot's username |
| Token OAuth | `oauth:xxxxxxxx` — get it at twitchtokengenerator.com |
| Canal | Channel to join, with `#` (e.g. `#mychannel`) |
| Volumen | Playback volume, 0%–200% |
| Motor TTS | `piper` (offline) or `gtts` (requires internet) |
| Modelo Piper | Path to the `.onnx` voice model file |
| Long. máxima | Max characters per message to read aloud |
| Tarjeta audio | ALSA audio output device (use `←→` to cycle) |

The configuration is saved to `~/.config/twitch-tts/config.json` and persists between runs.

---

## Running the Bot

```bash
# Docker
docker compose up

# Local
./files/target/release/bot
```

The bot will connect to Twitch IRC and start reading chat messages aloud.

---

## Chat Commands

| Command | Effect |
|---|---|
| `!volumen 80` | Sets volume to 80% |
| `!volumen +10` | Increases volume by 10% |
| `!volumen -10` | Decreases volume by 10% |

> Only the streamer (the configured username) can use these commands.

---

## Technologies

- [Rust](https://www.rust-lang.org/)
- [Ratatui](https://ratatui.rs/) — terminal UI
- [Piper TTS](https://github.com/rhasspy/piper) — offline Spanish TTS engine
- [gTTS](https://gtts.readthedocs.io/) — online TTS fallback
- [PulseAudio](https://www.freedesktop.org/wiki/Software/PulseAudio/) — audio output
- Twitch IRC API

---

## Author

CroquetaDeArroz
