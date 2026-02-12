#!/usr/bin/env bash
set -euo pipefail

echo "=== Voice Prompt — Uninstaller ==="

uninstall_linux() {
    echo "Removing Voice Prompt from your system..."

    # Stop systemd service if running
    if systemctl --user is-active voice-prompt &>/dev/null; then
        echo "  Stopping systemd service..."
        systemctl --user stop voice-prompt
    fi

    if systemctl --user is-enabled voice-prompt &>/dev/null; then
        echo "  Disabling systemd service..."
        systemctl --user disable voice-prompt
    fi

    # Remove files
    rm -f "$HOME/.local/bin/voice-prompt"
    rm -f "$HOME/.local/share/applications/voice-prompt.desktop"
    rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/voice-prompt.svg"
    rm -f "$HOME/.config/systemd/user/voice-prompt.service"

    # Remove configuration and data (optional)
    if [ -d "$HOME/.config/voice-prompt" ]; then
        read -p "Remove configuration and data? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$HOME/.config/voice-prompt"
            rm -rf "$HOME/.local/share/voice-prompt"
            echo "  Configuration and data removed."
        fi
    fi

    # Update desktop database
    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

    echo ""
    echo "=== Voice Prompt uninstalled ==="
}

uninstall_macos() {
    echo "Removing Voice Prompt from your system..."

    # Stop launchd agent if running
    if launchctl list | grep -q "com.voice-prompt.agent"; then
        echo "  Stopping launchd agent..."
        launchctl unload "$HOME/Library/LaunchAgents/com.voice-prompt.agent.plist" 2>/dev/null || true
    fi

    # Remove files
    rm -f "$HOME/.cargo/bin/voice-prompt"
    rm -f /usr/local/bin/voice-prompt
    rm -f "$HOME/Library/LaunchAgents/com.voice-prompt.agent.plist"

    # Remove configuration and data (optional)
    if [ -d "$HOME/.config/voice-prompt" ]; then
        read -p "Remove configuration and data? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$HOME/.config/voice-prompt"
            rm -rf "$HOME/.local/share/voice-prompt"
            echo "  Configuration and data removed."
        fi
    fi

    echo ""
    echo "=== Voice Prompt uninstalled ==="
}

# Detect platform
case "$(uname -s)" in
    Darwin)
        uninstall_macos
        ;;
    Linux)
        uninstall_linux
        ;;
    *)
        echo "Error: Unsupported operating system: $(uname -s)"
        exit 1
        ;;
esac
