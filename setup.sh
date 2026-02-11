#!/usr/bin/env bash
set -euo pipefail

echo "=== Voice Prompt — Fedora Setup ==="

# Install system dependencies
echo "[1/3] Installing system packages..."
sudo dnf install -y \
    gtk4-devel libadwaita-devel gtk4-layer-shell-devel \
    wayland-devel wayland-protocols-devel gobject-introspection-devel \
    alsa-lib-devel pkg-config cmake gcc-c++ \
    wl-clipboard xclip libnotify \
    clang-devel

# Add user to input group (for evdev hotkey access)
echo "[2/3] Adding $USER to 'input' group..."
if groups "$USER" | grep -q '\binput\b'; then
    echo "  Already in input group."
else
    sudo usermod -aG input "$USER"
    echo "  Added. You must log out and back in for this to take effect."
fi

# Build the project
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
