use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rdev::{listen, Event, EventType, Key};

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
            let held_keys: Arc<Mutex<HashSet<u16>>> =
                Arc::new(Mutex::new(HashSet::new()));
            let last_trigger: Arc<Mutex<Instant>> =
                Arc::new(Mutex::new(Instant::now() - Duration::from_secs(10)));
            let debounce = Duration::from_millis(500);

            let keys = held_keys.clone();
            let trigger = last_trigger.clone();
            let hotkey = shared_hotkey.clone();
            let tx = sender.clone();

            let callback = move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        let code = rdev_key_to_code(key);
                        let mut held = keys.lock().unwrap();
                        held.insert(code);

                        let hk = hotkey.lock().unwrap().clone();
                        let mods_held =
                            hk.modifiers.iter().all(|m| held.contains(m));
                        let trigger_held = held.contains(&hk.trigger);

                        let mut last = trigger.lock().unwrap();
                        if mods_held && trigger_held && last.elapsed() > debounce {
                            *last = Instant::now();
                            log::info!("Hotkey triggered: {}", hk.display_name);
                            let _ = tx.try_send(());
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let code = rdev_key_to_code(key);
                        keys.lock().unwrap().remove(&code);
                    }
                    _ => {}
                }
            };

            if let Err(e) = listen(callback) {
                log::error!("rdev listener error: {:?}", e);
            }
        })
        .expect("Failed to spawn hotkey thread");
}

/// Capture a single key combination.
/// Returns when a non-modifier key is pressed while modifiers are held.
/// Used by the hotkey dialog to detect the user's desired combo.
pub fn capture_hotkey_combo() -> Option<HotkeyConfig> {
    let result: Arc<Mutex<Option<HotkeyConfig>>> = Arc::new(Mutex::new(None));
    let done: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let held_keys: Arc<Mutex<HashSet<u16>>> =
        Arc::new(Mutex::new(HashSet::new()));
    let start = Instant::now();
    let timeout = Duration::from_secs(10);

    let res = result.clone();
    let d = done.clone();
    let keys = held_keys.clone();

    let callback = move |event: Event| {
        if *d.lock().unwrap() || start.elapsed() > timeout {
            return;
        }
        match event.event_type {
            EventType::KeyPress(key) => {
                let code = rdev_key_to_code(key);
                let mut held = keys.lock().unwrap();
                held.insert(code);

                if !is_modifier(code)
                    && held.iter().any(|k| is_modifier(*k))
                {
                    let modifiers: Vec<u16> = held
                        .iter()
                        .copied()
                        .filter(|k| is_modifier(*k))
                        .collect();
                    let display = build_display_name(&modifiers, code);
                    *res.lock().unwrap() = Some(HotkeyConfig {
                        modifiers,
                        trigger: code,
                        display_name: display,
                    });
                    *d.lock().unwrap() = true;
                }
            }
            EventType::KeyRelease(key) => {
                let code = rdev_key_to_code(key);
                keys.lock().unwrap().remove(&code);
            }
            _ => {}
        }
    };

    // rdev::listen blocks, so run it in a thread
    let done_check = done.clone();
    let handle = std::thread::spawn(move || {
        let _ = listen(callback);
    });

    // Poll for completion
    loop {
        if *done_check.lock().unwrap() {
            break;
        }
        if start.elapsed() > timeout {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    // We can't cleanly stop rdev::listen, but the callback will
    // stop processing after `done` is set. Detach the thread.
    drop(handle);

    result.lock().unwrap().take()
}

/// Map rdev::Key to a numeric code consistent with evdev codes
/// so that HotkeyConfig serialization is cross-platform.
fn rdev_key_to_code(key: Key) -> u16 {
    match key {
        // Modifiers — use same codes as evdev
        Key::ControlLeft => 29,
        Key::ControlRight => 97,
        Key::ShiftLeft => 42,
        Key::ShiftRight => 54,
        Key::Alt => 56,
        Key::AltGr => 100,
        Key::MetaLeft => 125,
        Key::MetaRight => 126,
        // Common keys
        Key::Escape => 1,
        Key::BackSpace => 14,
        Key::Tab => 15,
        Key::Return => 28,
        Key::Space => 57,
        // Letters
        Key::KeyA => 30,
        Key::KeyB => 48,
        Key::KeyC => 46,
        Key::KeyD => 32,
        Key::KeyE => 18,
        Key::KeyF => 33,
        Key::KeyG => 34,
        Key::KeyH => 35,
        Key::KeyI => 23,
        Key::KeyJ => 36,
        Key::KeyK => 37,
        Key::KeyL => 38,
        Key::KeyM => 50,
        Key::KeyN => 49,
        Key::KeyO => 24,
        Key::KeyP => 25,
        Key::KeyQ => 16,
        Key::KeyR => 19,
        Key::KeyS => 31,
        Key::KeyT => 20,
        Key::KeyU => 22,
        Key::KeyV => 47,
        Key::KeyW => 17,
        Key::KeyX => 45,
        Key::KeyY => 21,
        Key::KeyZ => 44,
        // Numbers
        Key::Num0 => 11,
        Key::Num1 => 2,
        Key::Num2 => 3,
        Key::Num3 => 4,
        Key::Num4 => 5,
        Key::Num5 => 6,
        Key::Num6 => 7,
        Key::Num7 => 8,
        Key::Num8 => 9,
        Key::Num9 => 10,
        // Function keys
        Key::F1 => 59,
        Key::F2 => 60,
        Key::F3 => 61,
        Key::F4 => 62,
        Key::F5 => 63,
        Key::F6 => 64,
        Key::F7 => 65,
        Key::F8 => 66,
        Key::F9 => 67,
        Key::F10 => 68,
        Key::F11 => 87,
        Key::F12 => 88,
        Key::Unknown(code) => code as u16,
        _ => 0,
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
        29 | 97 => "Ctrl",
        42 | 54 => "Shift",
        56 | 100 => "Alt",
        125 | 126 => "Cmd",
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
        30 => "A".into(),
        48 => "B".into(),
        46 => "C".into(),
        32 => "D".into(),
        18 => "E".into(),
        33 => "F".into(),
        34 => "G".into(),
        35 => "H".into(),
        23 => "I".into(),
        36 => "J".into(),
        37 => "K".into(),
        38 => "L".into(),
        50 => "M".into(),
        49 => "N".into(),
        24 => "O".into(),
        25 => "P".into(),
        16 => "Q".into(),
        19 => "R".into(),
        31 => "S".into(),
        20 => "T".into(),
        22 => "U".into(),
        47 => "V".into(),
        17 => "W".into(),
        45 => "X".into(),
        21 => "Y".into(),
        44 => "Z".into(),
        _ => format!("Key{code}"),
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
