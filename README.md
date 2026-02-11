# Voice Prompt

A cross-platform voice-to-clipboard prompt generator. Press a hotkey, speak your prompt, and it's transcribed, cleaned up, and copied to your clipboard — ready to paste into any AI coding assistant.

**Voice Prompt** combines local [Whisper](https://github.com/openai/whisper) speech-to-text with optional [Gemini](https://ai.google.dev/) refinement to turn spoken dictation into clean, structured prompts. It runs as a background service with a minimal overlay bar and a GTK4/Adwaita settings dashboard.

## How It Works

```
Hotkey → Record Audio → Transcribe (Whisper) → Refine (Gemini) → Clipboard
```

1. **Press your hotkey** (default: `Ctrl+Space`) from anywhere on your desktop
2. A recording overlay appears — speak your prompt
3. **Press the hotkey again** to stop recording
4. Audio is transcribed locally using Whisper (nothing leaves your machine)
5. The transcript is optionally refined by Gemini to remove filler words and fix speech artifacts
6. The final prompt is **copied to your clipboard** automatically

## Features

- **Cross-platform** — native support for Linux (Wayland/X11) and macOS
- **Global hotkey** — works from any application (evdev on Linux, rdev on macOS)
- **Local transcription** — Whisper runs entirely on your machine; no audio is uploaded
- **Smart refinement** — Gemini removes filler words (um, uh, like), fixes speech errors, and preserves technical terms
- **Graceful fallback** — works without an API key (raw transcription only)
- **Recording overlay** — minimal floating bar with live waveform, timer, and status phases
- **Settings dashboard** — GTK4/Adwaita UI to configure hotkey, API key, and view history
- **Prompt history** — browse and re-copy past prompts
- **Usage statistics** — tracks total prompts and word count
- **Audio feedback** — ascending beep on record start, descending beep on stop
- **Auto-start** — systemd service (Linux) or launchd agent (macOS)
- **Clipboard support** — `wl-copy` (Wayland), `xclip` (X11), `pbcopy` (macOS)

## Quick Start

### One-command install

The installer auto-detects your OS and package manager:

```bash
git clone https://github.com/tr4m0ryp/voice-prompt.git
cd voice-prompt
bash install.sh
```

This handles everything — system dependencies, build, and post-install instructions.

| Platform | Package manager | What it does |
|----------|----------------|--------------|
| **macOS** | Homebrew | Installs GTK4/libadwaita, builds, copies binary |
| **Fedora** | dnf | Installs dev packages, adds `input` group, builds |
| **Ubuntu/Debian** | apt | Installs dev packages, adds `input` group, builds |
| **Arch** | pacman | Installs dev packages, adds `input` group, builds |

### Run

```bash
./target/release/voice-prompt
```

On first launch, the Whisper model (~140 MB) downloads automatically. The settings dashboard opens where you can configure your hotkey and optionally add a Gemini API key.

### Auto-start (optional)

<details>
<summary><strong>Linux (systemd)</strong></summary>

```bash
cp target/release/voice-prompt ~/.cargo/bin/
mkdir -p ~/.config/systemd/user
cp voice-prompt.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now voice-prompt
```
</details>

<details>
<summary><strong>macOS (launchd)</strong></summary>

```bash
cp target/release/voice-prompt /usr/local/bin/
mkdir -p ~/Library/LaunchAgents
cp com.voice-prompt.agent.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.voice-prompt.agent.plist
```
</details>

## Platform Notes

### Linux

- Global hotkeys use **evdev**, which requires your user to be in the `input` group. The installer handles this, but you must **log out and back in** afterward.
- The recording overlay uses **gtk4-layer-shell** on Wayland for proper always-on-top positioning. On X11 it falls back to standard GTK windowing.

### macOS

- Global hotkeys use **rdev**. macOS requires you to grant **Input Monitoring** permission:
  **System Settings > Privacy & Security > Input Monitoring** — add `voice-prompt` (or your terminal) to the list.
- The overlay renders as an undecorated floating window using native GTK4 window management.

## Manual Dependency Install

If you prefer to install dependencies yourself instead of using `install.sh`:

<details>
<summary><strong>Fedora</strong></summary>

```bash
sudo dnf install -y \
    gtk4-devel libadwaita-devel gtk4-layer-shell-devel \
    wayland-devel wayland-protocols-devel gobject-introspection-devel \
    alsa-lib-devel pkg-config cmake gcc-c++ \
    wl-clipboard xclip libnotify clang-devel
sudo usermod -aG input $USER   # log out & back in
```
</details>

<details>
<summary><strong>Ubuntu / Debian</strong></summary>

```bash
sudo apt install -y \
    libgtk-4-dev libadwaita-1-dev libgtk4-layer-shell-dev \
    libasound2-dev pkg-config cmake g++ libclang-dev \
    wl-clipboard xclip libnotify-bin
sudo usermod -aG input $USER   # log out & back in
```
</details>

<details>
<summary><strong>Arch</strong></summary>

```bash
sudo pacman -S --needed gtk4 libadwaita gtk4-layer-shell \
    alsa-lib pkg-config cmake clang \
    wl-clipboard xclip libnotify
sudo usermod -aG input $USER   # log out & back in
```
</details>

<details>
<summary><strong>macOS (Homebrew)</strong></summary>

```bash
brew install gtk4 libadwaita pkg-config cmake
```
</details>

Then build:

```bash
cargo build --release
```

## Configuration

All configuration is managed through the settings dashboard and stored in `~/.config/voice-prompt/config.json`.

### Gemini API Key (optional)

To enable smart transcript refinement:

1. Get a free API key from [Google AI Studio](https://aistudio.google.com/apikey)
2. Enter it in the settings dashboard under "Gemini API Key"

Without an API key, Voice Prompt still works — you get raw Whisper transcriptions without cleanup.

### Hotkey

Default: `Ctrl+Space`. Click **Change Hotkey** in the dashboard and press your desired combination to rebind it.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   GTK Main Thread                    │
│  ┌───────────┐  ┌───────────┐  ┌────────────────┐   │
│  │ Dashboard  │  │  Overlay  │  │ Event Handler  │   │
│  │ (Settings) │  │ (Record)  │  │  (Dispatch)    │   │
│  └───────────┘  └───────────┘  └───────┬────────┘   │
│                                        │             │
│                            async_channel│             │
├────────────────────────────────────────┼─────────────┤
│               Background Threads        │             │
│  ┌────────────┐  ┌──────────┐  ┌───────┴──────────┐  │
│  │evdev / rdev│  │   CPAL   │  │     Tokio        │  │
│  │ (Hotkey)   │  │ (Record) │  │ (Whisper + API)  │  │
│  └────────────┘  └──────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────┘
```

- **GTK main thread** — all UI state in `Rc<RefCell<AppState>>`
- **Hotkey thread** — evdev (Linux) or rdev (macOS) for global key monitoring
- **CPAL** — captures microphone audio at 16kHz mono
- **Tokio runtime** — async Gemini API calls + CPU-heavy Whisper transcription via `spawn_blocking`
- **async_channel** — bridges background threads to the GTK main loop

## Data Storage

| Data | Location |
|------|----------|
| Configuration | `~/.config/voice-prompt/config.json` |
| Usage statistics | `~/.local/share/voice-prompt/stats.json` |
| Whisper model | `~/.local/share/voice-prompt/models/ggml-base.en.bin` |

## Privacy

- **Audio never leaves your machine** — Whisper transcription runs entirely locally
- **Transcript text** is sent to Google Gemini only if you provide an API key, and only for refinement
- **No telemetry** — no data is collected or sent anywhere
- All personal data (prompts, statistics) is stored locally in your home directory

## License

[MIT](LICENSE)
