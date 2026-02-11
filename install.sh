#!/usr/bin/env bash
set -euo pipefail

echo "=== Voice Prompt — Multi-Platform Installer ==="

install_macos() {
    echo "[1/3] Installing macOS dependencies via Homebrew..."
    if ! command -v brew &>/dev/null; then
        echo "Error: Homebrew is required. Install from https://brew.sh"
        exit 1
    fi
    brew install gtk4 libadwaita pkg-config cmake

    echo "[2/3] Building release binary..."
    cargo build --release

    echo "[3/3] Installing binary..."
    install -d "$HOME/.cargo/bin"
    cp target/release/voice-prompt "$HOME/.cargo/bin/"

    echo ""
    echo "=== Setup complete ==="
    echo "Binary: $HOME/.cargo/bin/voice-prompt"
    echo ""
    echo "IMPORTANT: macOS requires Accessibility permissions for global hotkeys."
    echo "  System Settings > Privacy & Security > Input Monitoring"
    echo "  Add 'voice-prompt' (or your terminal) to the list."
    echo ""
    echo "To auto-start on login:"
    echo "  mkdir -p ~/Library/LaunchAgents"
    echo "  cp com.voice-prompt.agent.plist ~/Library/LaunchAgents/"
    echo "  launchctl load ~/Library/LaunchAgents/com.voice-prompt.agent.plist"
}

install_fedora() {
    echo "[1/3] Installing Fedora packages..."
    sudo dnf install -y \
        gtk4-devel libadwaita-devel gtk4-layer-shell-devel \
        wayland-devel wayland-protocols-devel gobject-introspection-devel \
        alsa-lib-devel pkg-config cmake gcc-c++ \
        wl-clipboard xclip libnotify \
        clang-devel

    echo "[2/3] Adding $USER to 'input' group..."
    if groups "$USER" | grep -q '\binput\b'; then
        echo "  Already in input group."
    else
        sudo usermod -aG input "$USER"
        echo "  Added. You must log out and back in for this to take effect."
    fi

    echo "[3/3] Building release binary..."
    cargo build --release

    echo ""
    echo "=== Setup complete ==="
    echo "Binary: target/release/voice-prompt"
    echo ""
    echo "NOTE: If you were just added to the 'input' group, log out and back in first."
    echo "To install as a systemd user service:"
    echo "  cp target/release/voice-prompt ~/.cargo/bin/"
    echo "  mkdir -p ~/.config/systemd/user"
    echo "  cp voice-prompt.service ~/.config/systemd/user/"
    echo "  systemctl --user daemon-reload"
    echo "  systemctl --user enable --now voice-prompt"
}

install_debian() {
    echo "[1/3] Installing Debian/Ubuntu packages..."
    sudo apt update
    sudo apt install -y \
        libgtk-4-dev libadwaita-1-dev libgtk4-layer-shell-dev \
        libasound2-dev pkg-config cmake g++ libclang-dev \
        wl-clipboard xclip libnotify-bin

    echo "[2/3] Adding $USER to 'input' group..."
    if groups "$USER" | grep -q '\binput\b'; then
        echo "  Already in input group."
    else
        sudo usermod -aG input "$USER"
        echo "  Added. You must log out and back in for this to take effect."
    fi

    echo "[3/3] Building release binary..."
    cargo build --release

    echo ""
    echo "=== Setup complete ==="
    echo "Binary: target/release/voice-prompt"
    echo ""
    echo "NOTE: If you were just added to the 'input' group, log out and back in first."
    echo "To install as a systemd user service:"
    echo "  cp target/release/voice-prompt ~/.cargo/bin/"
    echo "  mkdir -p ~/.config/systemd/user"
    echo "  cp voice-prompt.service ~/.config/systemd/user/"
    echo "  systemctl --user daemon-reload"
    echo "  systemctl --user enable --now voice-prompt"
}

install_arch() {
    echo "[1/3] Installing Arch packages..."
    sudo pacman -S --needed --noconfirm \
        gtk4 libadwaita gtk4-layer-shell \
        alsa-lib pkg-config cmake clang \
        wl-clipboard xclip libnotify

    echo "[2/3] Adding $USER to 'input' group..."
    if groups "$USER" | grep -q '\binput\b'; then
        echo "  Already in input group."
    else
        sudo usermod -aG input "$USER"
        echo "  Added. You must log out and back in for this to take effect."
    fi

    echo "[3/3] Building release binary..."
    cargo build --release

    echo ""
    echo "=== Setup complete ==="
    echo "Binary: target/release/voice-prompt"
    echo ""
    echo "NOTE: If you were just added to the 'input' group, log out and back in first."
    echo "To install as a systemd user service:"
    echo "  cp target/release/voice-prompt ~/.cargo/bin/"
    echo "  mkdir -p ~/.config/systemd/user"
    echo "  cp voice-prompt.service ~/.config/systemd/user/"
    echo "  systemctl --user daemon-reload"
    echo "  systemctl --user enable --now voice-prompt"
}

# Detect platform
case "$(uname -s)" in
    Darwin)
        install_macos
        ;;
    Linux)
        if command -v dnf &>/dev/null; then
            install_fedora
        elif command -v apt &>/dev/null; then
            install_debian
        elif command -v pacman &>/dev/null; then
            install_arch
        else
            echo "Error: Unsupported Linux distribution."
            echo "Supported: Fedora (dnf), Debian/Ubuntu (apt), Arch (pacman)"
            exit 1
        fi
        ;;
    *)
        echo "Error: Unsupported operating system: $(uname -s)"
        exit 1
        ;;
esac
