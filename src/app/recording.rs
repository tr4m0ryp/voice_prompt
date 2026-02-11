use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use gtk4::glib;
use gtk4::prelude::*;

use super::pipeline::dispatch_transcription;
use super::state::{AppState, AppStatus, BackendEvent, OverlayPhase, update_status};
use crate::ui::overlay::set_overlay_phase;

/// Start recording audio from the microphone.
pub fn start_recording(state: &Rc<RefCell<AppState>>) {
    log::info!("Starting recording");

    // Cancel any pending dismiss timer from a previous "Done" phase
    if let Some(source) = state.borrow_mut().overlay_dismiss_source.take() {
        source.remove();
    }

    // Clear audio buffer
    {
        let s = state.borrow();
        s.audio_buffer.lock().unwrap().clear();
    }

    crate::audio_feedback::play_beep(crate::audio_feedback::BeepType::Start);

    // Start cpal capture
    let buffer = state.borrow().audio_buffer.clone();
    match crate::recorder::start_capture(buffer) {
        Ok((stream, sample_rate)) => {
            let mut s = state.borrow_mut();
            s.cpal_stream = Some(stream);
            s.sample_rate = sample_rate;
            s.recording_start = Some(std::time::Instant::now());
            s.status = AppStatus::Recording;
            s.overlay_phase = Some(OverlayPhase::Recording);

            if let Some(ref overlay) = s.overlay {
                overlay.timer_label.set_text("00:00");
                set_overlay_phase(overlay, &OverlayPhase::Recording);
                overlay.window.set_visible(true);
            }
            if let Some(ref dash) = s.dashboard {
                dash.status_label.set_text("Recording...");
            }
        }
        Err(e) => {
            log::error!("Failed to start recording: {e}");
            update_status(state, AppStatus::Idle, &format!("Mic error: {e}"));
            return;
        }
    }

    // Start 80ms tick for waveform updates (~12fps).
    let sender = state.borrow().backend_sender.clone();
    let audio_buf = state.borrow().audio_buffer.clone();
    let tick_counter = Arc::new(AtomicUsize::new(0));

    let source = glib::timeout_add_local(
        std::time::Duration::from_millis(80),
        move || {
            let rms = compute_rms(&audio_buf);
            let _ = sender.try_send(BackendEvent::AudioLevel(rms));

            let count = tick_counter.fetch_add(1, Ordering::Relaxed);
            if count % 12 == 0 {
                let _ = sender.try_send(BackendEvent::TimerTick);
            }

            glib::ControlFlow::Continue
        },
    );
    state.borrow_mut().timer_source = Some(source);
}

/// Compute RMS of the last ~1280 samples in the audio buffer.
fn compute_rms(buffer: &Arc<std::sync::Mutex<Vec<f32>>>) -> f32 {
    let buf = buffer.lock().unwrap();
    let n = buf.len().min(1280);
    if n == 0 {
        return 0.0;
    }
    let start = buf.len() - n;
    let sum_sq: f32 = buf[start..].iter().map(|&s| s * s).sum();
    (sum_sq / n as f32).sqrt()
}

/// Stop recording and dispatch transcription.
pub fn stop_recording(state: &Rc<RefCell<AppState>>) {
    log::info!("Stopping recording");

    if let Some(source) = state.borrow_mut().timer_source.take() {
        source.remove();
    }

    state.borrow_mut().cpal_stream = None;

    crate::audio_feedback::play_beep(crate::audio_feedback::BeepType::Stop);

    // Transition overlay to Transcribing instead of hiding
    {
        let mut s = state.borrow_mut();
        s.overlay_phase = Some(OverlayPhase::Transcribing);
        if let Some(ref overlay) = s.overlay {
            set_overlay_phase(overlay, &OverlayPhase::Transcribing);
        }
    }

    update_status(state, AppStatus::Processing, "Transcribing...");

    let samples: Vec<f32> = state.borrow().audio_buffer.lock().unwrap().clone();
    let sample_rate = state.borrow().sample_rate;

    if samples.is_empty() {
        // Nothing captured — dismiss overlay and go idle
        let mut s = state.borrow_mut();
        s.overlay_phase = None;
        if let Some(ref overlay) = s.overlay {
            overlay.window.set_visible(false);
        }
        drop(s);
        update_status(state, AppStatus::Idle, "No audio captured");
        return;
    }

    log::info!(
        "Captured {} samples ({:.1}s at {}Hz)",
        samples.len(),
        samples.len() as f32 / sample_rate as f32,
        sample_rate
    );

    dispatch_transcription(state, samples);
}
