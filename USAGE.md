# Voice Prompt Usage Guide

## Starting the Application

### From Application Menu
- Search for "Voice Prompt" in your application launcher
- Click the icon to start the application
- The dashboard window will appear

### From Terminal
```bash
voice-prompt
```

## Application Behavior

### Window Management
- **First Launch**: Dashboard window opens automatically
- **Closing the Window**:
  - Click the X button or press `Ctrl+W` → Window hides, app keeps running in background
  - The app continues to listen for your global hotkey even when hidden
- **Clicking Icon Again**: Re-opens the hidden window

### Menu Options
Click the menu button (⋮) in the header bar:
- **About Voice Prompt** - View app information and version
- **Hide Window** (`Ctrl+H`) - Hide the window while keeping the app running
- **Quit** (`Ctrl+Q`) - Completely quit the application

### Keyboard Shortcuts
- `Ctrl+Q` - Quit the application completely
- `Ctrl+H` - Hide the window (app stays running)
- `Ctrl+Shift+V` - Show the window if hidden

## Background Operation

Voice Prompt is designed to run in the background:

1. **Start the app** - Dashboard window opens
2. **Close or hide the window** - App continues running
3. **Use your hotkey** (default: `Ctrl+Space`) - Works even when window is hidden
4. **Reopen window** - Click the app icon or press `Ctrl+Shift+V`

## Fully Quitting

To completely stop Voice Prompt:
- **Menu → Quit**
- **Keyboard**: `Ctrl+Q`
- **Terminal**: `pkill voice-prompt`
- **System Monitor**: Find "voice-prompt" and kill the process

## Auto-Start (Optional)

To have Voice Prompt start automatically on login:

### Linux (systemd)
```bash
mkdir -p ~/.config/systemd/user
cp voice-prompt.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now voice-prompt
```

Check status:
```bash
systemctl --user status voice-prompt
```

Stop auto-start:
```bash
systemctl --user disable voice-prompt
```

### macOS (launchd)
```bash
mkdir -p ~/Library/LaunchAgents
cp com.voice-prompt.agent.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.voice-prompt.agent.plist
```

## Uninstall

To remove Voice Prompt:
```bash
bash uninstall.sh
```

You'll be prompted whether to keep or delete your configuration and data.
