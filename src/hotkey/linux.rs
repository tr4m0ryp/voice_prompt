use evdev::{Device, EventType, KeyCode};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::config::HotkeyConfig;

/// Start the hotkey listener on a dedicated OS thread.
/// Sends `()` through the async channel each time the hotkey is triggered.
pub fn start_listener(
    sender: async_channel::Sender<()>,
    shared_hotkey: Arc<Mutex<HotkeyConfig>>,
) {
    std::thread::Builder::new()
        .name("hotkey-listener".into())
        .spawn(move || {
            if let Err(e) = listener_loop(sender, shared_hotkey) {
                log::error!("Hotkey listener exited: {e}");
            }
        })
        .expect("Failed to spawn hotkey thread");
}

fn listener_loop(
    sender: async_channel::Sender<()>,
    shared_hotkey: Arc<Mutex<HotkeyConfig>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut devices = open_keyboard_devices();
    if devices.is_empty() {
        return Err("No keyboard devices found. Is the user in the 'input' group?".into());
    }
    log::info!("Opened {} keyboard device(s)", devices.len());

    // Set non-blocking
    for dev in &devices {
        dev.set_nonblocking(true)?;
    }

    let mut held_keys: HashSet<u16> = HashSet::new();
    let mut last_trigger = Instant::now() - Duration::from_secs(10);
    let debounce = Duration::from_millis(500);

    loop {
        let mut any_event = false;

        for dev in &mut devices {
            if let Ok(events) = dev.fetch_events() {
                for event in events {
                    if event.event_type() == EventType::KEY {
                        any_event = true;
                        let code = event.code();
                        match event.value() {
                            1 => {
                                held_keys.insert(code);
                            }
                            0 => {
                                held_keys.remove(&code);
                            }
                            _ => {} // repeat events
                        }
                    }
                }
            }
        }

        // Check hotkey match
        let hotkey = shared_hotkey.lock().unwrap().clone();
        let mods_held = hotkey.modifiers.iter().all(|m| held_keys.contains(m));
        let trigger_held = held_keys.contains(&hotkey.trigger);

        if mods_held && trigger_held && last_trigger.elapsed() > debounce {
            last_trigger = Instant::now();
            log::info!("Hotkey triggered: {}", hotkey.display_name);
            if sender.try_send(()).is_err() {
                log::info!("GTK channel closed, exiting hotkey listener");
                return Ok(());
            }
        }

        if !any_event {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

/// Open all /dev/input/event* devices that look like keyboards.
fn open_keyboard_devices() -> Vec<Device> {
    let mut devices = Vec::new();
    let Ok(entries) = std::fs::read_dir("/dev/input") else {
        return devices;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if !name.starts_with("event") {
            continue;
        }
        if let Ok(dev) = Device::open(&path) {
            // Check that the device supports EV_KEY and has KEY_A
            let has_key = dev.supported_events().contains(EventType::KEY);
            let has_key_a = dev
                .supported_keys()
                .map(|keys| keys.contains(KeyCode::KEY_A))
                .unwrap_or(false);
            if has_key && has_key_a {
                log::info!(
                    "Opened keyboard: {} ({})",
                    dev.name().unwrap_or("unknown"),
                    path.display()
                );
                devices.push(dev);
            }
        }
    }
    devices
}

/// Capture a single key combination from evdev devices.
/// Returns when a non-modifier key is pressed while modifiers are held.
/// Used by the hotkey dialog to detect the user's desired combo.
pub fn capture_hotkey_combo() -> Option<HotkeyConfig> {
    let mut devices = open_keyboard_devices();
    if devices.is_empty() {
        return None;
    }
    for dev in &devices {
        let _ = dev.set_nonblocking(true);
    }

    let mut held_keys: HashSet<u16> = HashSet::new();
    let start = Instant::now();
    let timeout = Duration::from_secs(10);

    loop {
        if start.elapsed() > timeout {
            return None;
        }

        for dev in &mut devices {
            if let Ok(events) = dev.fetch_events() {
                for event in events {
                    if event.event_type() == EventType::KEY {
                        let code = event.code();
                        match event.value() {
                            1 => {
                                held_keys.insert(code);
                                // If this is a non-modifier and we have at least one modifier, we're done
                                if !is_modifier(code)
                                    && held_keys.iter().any(|k| is_modifier(*k))
                                {
                                    let modifiers: Vec<u16> = held_keys
                                        .iter()
                                        .copied()
                                        .filter(|k| is_modifier(*k))
                                        .collect();
                                    let display = build_display_name(&modifiers, code);
                                    return Some(HotkeyConfig {
                                        modifiers,
                                        trigger: code,
                                        display_name: display,
                                    });
                                }
                            }
                            0 => {
                                held_keys.remove(&code);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn is_modifier(code: u16) -> bool {
    matches!(
        code,
        29 | 97 | 42 | 54 | 56 | 100 | 125 | 126
        // LCTRL | RCTRL | LSHIFT | RSHIFT | LALT | RALT | LMETA | RMETA
    )
}

fn key_name(code: u16) -> &'static str {
    match code {
        29 => "Ctrl",
        97 => "Ctrl",
        42 => "Shift",
        54 => "Shift",
        56 => "Alt",
        100 => "Alt",
        125 => "Super",
        126 => "Super",
        _ => "",
    }
}

fn trigger_name(code: u16) -> String {
    match code {
        1 => "Esc".into(),
        14 => "Backspace".into(),
        15 => "Tab".into(),
        28 => "Enter".into(),
        57 => "Space".into(),
        _ => {
            // Try to get a readable name from the KeyCode
            let key = KeyCode(code);
            let debug = format!("{key:?}");
            // KeyCode debug looks like "KEY_A" — strip "KEY_"
            debug.strip_prefix("KEY_").unwrap_or(&debug).to_string()
        }
    }
}

fn build_display_name(modifiers: &[u16], trigger: u16) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    for &m in modifiers {
        let name = key_name(m);
        if !name.is_empty() && seen.insert(name) {
            parts.push(name.to_string());
        }
    }
    parts.push(trigger_name(trigger));
    parts.join("+")
}
