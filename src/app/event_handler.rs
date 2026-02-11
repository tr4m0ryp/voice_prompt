use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;

use super::model::load_whisper_model;
use super::pipeline::dispatch_refinement;
use super::recording::{start_recording, stop_recording};
use super::state::{AppState, AppStatus, BackendEvent, OverlayPhase, update_status};
use crate::ui::overlay::set_overlay_phase;

/// Handle a backend event. This is the core state machine.
pub fn handle_backend_event(state: &Rc<RefCell<AppState>>, event: BackendEvent) {
    match event {
        BackendEvent::HotkeyTriggered => {
            let current_status = state.borrow().status.clone();
            match current_status {
                AppStatus::Idle => start_recording(state),
                AppStatus::Recording => stop_recording(state),
                _ => {
                    log::info!("Ignoring hotkey while status={current_status:?}");
                }
            }
        }
        BackendEvent::TranscriptionComplete(transcript) => {
            log::info!("Transcript: {transcript}");
            // Transition overlay to Refining
            {
                let mut s = state.borrow_mut();
                s.overlay_phase = Some(OverlayPhase::Refining);
                if let Some(ref overlay) = s.overlay {
                    set_overlay_phase(overlay, &OverlayPhase::Refining);
                }
            }
            update_status(state, AppStatus::Processing, "Refining with Gemini...");
            dispatch_refinement(state, transcript);
        }
        BackendEvent::RefinementComplete(refined) => {
            log::info!("Refined: {refined}");
            on_prompt_ready(state, refined);
        }
        BackendEvent::ProcessingError(err) => {
            log::error!("Processing error: {err}");
            dismiss_overlay(state);
            update_status(state, AppStatus::Idle, &format!("Error: {err}"));
        }
        BackendEvent::ModelDownloadProgress(downloaded, total) => {
            if let Some(ref dash) = state.borrow().dashboard {
                dash.progress_bar.set_visible(true);
                if total > 0 {
                    dash.progress_bar
                        .set_fraction(downloaded as f64 / total as f64);
                    let mb_done = downloaded as f64 / 1_048_576.0;
                    let mb_total = total as f64 / 1_048_576.0;
                    dash.progress_bar.set_text(Some(&format!(
                        "Downloading model: {mb_done:.1} / {mb_total:.1} MB"
                    )));
                } else {
                    dash.progress_bar.pulse();
                }
            }
        }
        BackendEvent::ModelDownloadComplete => {
            if let Some(ref dash) = state.borrow().dashboard {
                dash.progress_bar.set_visible(false);
            }
            load_whisper_model(state);
        }
        BackendEvent::TimerTick => {
            let s = state.borrow();
            if let (Some(start), Some(ref overlay)) = (s.recording_start, &s.overlay) {
                let elapsed = start.elapsed().as_secs();
                let mins = elapsed / 60;
                let secs = elapsed % 60;
                overlay.timer_label.set_text(&format!("{mins:02}:{secs:02}"));
            }
        }
        BackendEvent::AudioLevel(level) => {
            let s = state.borrow();
            if let Some(ref overlay) = s.overlay {
                let mut levels = overlay.audio_levels.borrow_mut();
                if levels.len() >= 24 {
                    levels.pop_front();
                }
                levels.push_back(level);
                overlay.waveform.queue_draw();
            }
        }
        BackendEvent::OverlayClicked => {
            // If Done, re-copy text to clipboard before dismissing
            let phase = state.borrow().overlay_phase.clone();
            if let Some(OverlayPhase::Done(ref text)) = phase {
                let _ = crate::clipboard::copy_to_clipboard(text);
            }
            dismiss_overlay(state);
        }
    }
}

fn on_prompt_ready(state: &Rc<RefCell<AppState>>, text: String) {
    if let Err(e) = crate::clipboard::copy_to_clipboard(&text) {
        log::error!("Clipboard error: {e}");
        dismiss_overlay(state);
        update_status(state, AppStatus::Idle, &format!("Clipboard error: {e}"));
        return;
    }

    {
        let mut s = state.borrow_mut();
        s.stats.record_prompt(&text);
        if let Err(e) = s.stats.save() {
            log::warn!("Failed to save stats: {e}");
        }
    }

    {
        let s = state.borrow();
        if let Some(ref dash) = s.dashboard {
            dash.words_label.set_text(&s.stats.total_words.to_string());
            dash.prompts_label.set_text(&s.stats.total_prompts.to_string());
        }
    }

    // Transition overlay to Done
    {
        let mut s = state.borrow_mut();
        let done_phase = OverlayPhase::Done(text);
        s.overlay_phase = Some(done_phase.clone());
        if let Some(ref overlay) = s.overlay {
            set_overlay_phase(overlay, &done_phase);
        }
    }

    update_status(state, AppStatus::Idle, "Idle — Prompt copied!");

    // Auto-dismiss after 3 seconds
    let state_clone = state.clone();
    let source = glib::timeout_add_local_once(
        std::time::Duration::from_secs(3),
        move || {
            dismiss_overlay(&state_clone);
        },
    );
    state.borrow_mut().overlay_dismiss_source = Some(source);
}

/// Hide overlay, clear phase, cancel dismiss timer.
fn dismiss_overlay(state: &Rc<RefCell<AppState>>) {
    let mut s = state.borrow_mut();
    s.overlay_phase = None;
    if let Some(source) = s.overlay_dismiss_source.take() {
        source.remove();
    }
    if let Some(ref overlay) = s.overlay {
        overlay.window.set_visible(false);
    }
}
