mod app;
mod audio_feedback;
mod clipboard;
mod config;
mod hotkey;
mod recorder;
mod refiner;
mod stats;
mod transcriber;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita::prelude::*;

use app::{AppState, BackendEvent};

fn main() {
    env_logger::init();
    log::info!("Voice Prompt starting");

    let application = libadwaita::Application::builder()
        .application_id("com.github.tr4m0ryp.voice-prompt")
        .build();

    application.connect_activate(on_activate);
    application.run();
}

fn on_activate(app: &libadwaita::Application) {
    // Create async channels for backend → UI communication
    let (backend_tx, backend_rx) = async_channel::unbounded::<BackendEvent>();
    let (hotkey_tx, hotkey_rx) = async_channel::unbounded::<()>();

    // Build app state
    let overlay_tx = backend_tx.clone();
    let state = Rc::new(RefCell::new(AppState::new(backend_tx)));

    // Build UI
    let dashboard = ui::dashboard::build_dashboard(
        app,
        "Starting...",
        state.borrow().stats.total_words,
        state.borrow().stats.total_prompts,
        &state.borrow().config.hotkey.display_name,
        &state.borrow().config.gemini_api_key,
    );
    let overlay = ui::overlay::build_overlay(app, overlay_tx);

    // Wire up the "Change Hotkey" button
    {
        let state_clone = state.clone();
        let dash_window = dashboard.window.clone();
        dashboard.change_hotkey_button.connect_clicked(move |_| {
            let state_inner = state_clone.clone();
            ui::hotkey_dialog::show_hotkey_dialog(&dash_window, move |result| {
                if let Some(new_hotkey) = result {
                    log::info!("New hotkey: {}", new_hotkey.display_name);
                    let mut s = state_inner.borrow_mut();
                    *s.shared_hotkey.lock().unwrap() = new_hotkey.clone();
                    s.config.hotkey = new_hotkey.clone();
                    if let Err(e) = s.config.save() {
                        log::warn!("Failed to save config: {e}");
                    }
                    if let Some(ref dash) = s.dashboard {
                        dash.hotkey_label.set_text(&new_hotkey.display_name);
                    }
                }
            });
        });
    }

    // Wire up API key changes
    {
        let state_clone = state.clone();
        dashboard
            .api_key_row
            .connect_changed(move |row: &libadwaita::PasswordEntryRow| {
                let key = row.text().to_string();
                let mut s = state_clone.borrow_mut();
                s.config.gemini_api_key = key;
                if let Err(e) = s.config.save() {
                    log::warn!("Failed to save config: {e}");
                }
            });
    }

    // Wire up prompts row to open history
    {
        let state_clone = state.clone();
        let dash_window = dashboard.window.clone();
        dashboard.prompts_row.connect_activated(move |_| {
            let history = state_clone.borrow().stats.history.clone();
            ui::history::show_history_window(&dash_window, &history);
        });
    }

    // Store UI handles in state
    {
        let mut s = state.borrow_mut();
        s.dashboard = Some(dashboard);
        s.overlay = Some(overlay);
    }

    // Show the dashboard
    state.borrow().dashboard.as_ref().unwrap().window.present();

    // Start hotkey listener
    {
        let shared_hotkey = state.borrow().shared_hotkey.clone();
        hotkey::start_listener(hotkey_tx, shared_hotkey);
    }

    // Forward hotkey triggers to backend event channel
    {
        let sender = state.borrow().backend_sender.clone();
        gtk4::glib::spawn_future_local(async move {
            while hotkey_rx.recv().await.is_ok() {
                let _ = sender.send(BackendEvent::HotkeyTriggered).await;
            }
        });
    }

    // Attach backend event handler
    {
        let state_clone = state.clone();
        gtk4::glib::spawn_future_local(async move {
            while let Ok(event) = backend_rx.recv().await {
                app::handle_backend_event(&state_clone, event);
            }
        });
    }

    // Start whisper model download/load
    app::ensure_whisper_model(&state);
}
