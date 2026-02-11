use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::glib;

use super::state::{AppState, AppStatus, BackendEvent, update_status};

/// Attempt to download and/or load the whisper model.
pub fn ensure_whisper_model(state: &Rc<RefCell<AppState>>) {
    if crate::transcriber::model_exists() {
        load_whisper_model(state);
    } else {
        log::info!("Whisper model not found, starting download");
        update_status(state, AppStatus::ModelDownloading, "Downloading model...");
        let sender = state.borrow().backend_sender.clone();
        let progress_sender = sender.clone();

        state.borrow().tokio_rt.spawn(async move {
            let result =
                crate::transcriber::download_model(move |downloaded, total| {
                    let _ = progress_sender.try_send(
                        BackendEvent::ModelDownloadProgress(downloaded, total),
                    );
                })
                .await;

            match result {
                Ok(()) => {
                    let _ = sender.send(BackendEvent::ModelDownloadComplete).await;
                }
                Err(e) => {
                    let _ = sender
                        .send(BackendEvent::ProcessingError(format!(
                            "Model download failed: {e}"
                        )))
                        .await;
                }
            }
        });
    }
}

/// Load the whisper model in a blocking task, then deliver it to the main thread.
pub fn load_whisper_model(state: &Rc<RefCell<AppState>>) {
    log::info!("Loading whisper model...");
    update_status(state, AppStatus::Processing, "Loading model...");

    let sender = state.borrow().backend_sender.clone();

    // We can't send Rc<RefCell> into tokio, so use a separate channel
    // to pass the loaded context back to the main thread.
    let (ctx_tx, ctx_rx) = async_channel::bounded::<whisper_rs::WhisperContext>(1);

    state.borrow().tokio_rt.spawn(async move {
        let result =
            tokio::task::spawn_blocking(|| crate::transcriber::load_model()).await;

        match result {
            Ok(Ok(ctx)) => {
                let _ = ctx_tx.send(ctx).await;
            }
            Ok(Err(e)) => {
                let _ = sender
                    .send(BackendEvent::ProcessingError(format!(
                        "Failed to load model: {e}"
                    )))
                    .await;
            }
            Err(e) => {
                let _ = sender
                    .send(BackendEvent::ProcessingError(format!(
                        "Model load panicked: {e}"
                    )))
                    .await;
            }
        }
    });

    // Receive the loaded context on the GTK main thread
    let state_clone = state.clone();
    glib::spawn_future_local(async move {
        if let Ok(ctx) = ctx_rx.recv().await {
            state_clone.borrow_mut().whisper_ctx = Some(Arc::new(ctx));
            if let Some(ref dash) = state_clone.borrow().dashboard {
                dash.status_label.set_text("Idle");
            }
            state_clone.borrow_mut().status = AppStatus::Idle;
            log::info!("Whisper model ready");
        }
    });
}
