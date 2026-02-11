use std::cell::RefCell;
use std::rc::Rc;

use super::state::{AppState, AppStatus, BackendEvent, update_status};

/// Dispatch whisper transcription on the tokio runtime.
pub fn dispatch_transcription(state: &Rc<RefCell<AppState>>, samples: Vec<f32>) {
    let s = state.borrow();
    let ctx = match &s.whisper_ctx {
        Some(ctx) => ctx.clone(),
        None => {
            drop(s);
            update_status(state, AppStatus::Idle, "Whisper model not loaded");
            return;
        }
    };
    let sender = s.backend_sender.clone();

    s.tokio_rt.spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            crate::transcriber::transcribe(&ctx, &samples)
        })
        .await;

        match result {
            Ok(Ok(text)) => {
                let _ = sender.send(BackendEvent::TranscriptionComplete(text)).await;
            }
            Ok(Err(e)) => {
                let _ = sender
                    .send(BackendEvent::ProcessingError(format!(
                        "Transcription failed: {e}"
                    )))
                    .await;
            }
            Err(e) => {
                let _ = sender
                    .send(BackendEvent::ProcessingError(format!(
                        "Transcription task panicked: {e}"
                    )))
                    .await;
            }
        }
    });
}

/// Dispatch Gemini refinement on the tokio runtime.
pub fn dispatch_refinement(state: &Rc<RefCell<AppState>>, transcript: String) {
    let s = state.borrow();
    let api_key = s.config.gemini_api_key.clone();
    let sender = s.backend_sender.clone();

    s.tokio_rt.spawn(async move {
        match crate::refiner::refine(&api_key, &transcript).await {
            Ok(refined) => {
                let _ = sender.send(BackendEvent::RefinementComplete(refined)).await;
            }
            Err(e) => {
                log::warn!("Refinement failed, using raw transcript: {e}");
                let _ = sender
                    .send(BackendEvent::RefinementComplete(transcript))
                    .await;
            }
        }
    });
}
