use std::sync::{Arc, Mutex};

use gtk4::glib;

use crate::config::{Config, HotkeyConfig};
use crate::stats::Stats;
use crate::ui::dashboard::DashboardWidgets;
use crate::ui::overlay::OverlayWidgets;

/// Events sent from background threads to the GTK main thread.
#[derive(Debug, Clone)]
pub enum BackendEvent {
    HotkeyTriggered,
    TranscriptionComplete(String),
    RefinementComplete(String),
    ProcessingError(String),
    ModelDownloadProgress(u64, u64),
    ModelDownloadComplete,
    TimerTick,
    AudioLevel(f32),
    OverlayClicked,
}

/// Application status.
#[derive(Debug, Clone, PartialEq)]
pub enum AppStatus {
    Idle,
    Recording,
    Processing,
    ModelDownloading,
}

/// Overlay pipeline phase.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayPhase {
    Recording,
    Transcribing,
    Refining,
    Done(String),
}

/// Central application state. Lives on the GTK main thread inside Rc<RefCell<>>.
pub struct AppState {
    pub status: AppStatus,
    pub config: Config,
    pub stats: Stats,
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,
    pub shared_hotkey: Arc<Mutex<HotkeyConfig>>,
    pub tokio_rt: tokio::runtime::Runtime,
    pub whisper_ctx: Option<Arc<whisper_rs::WhisperContext>>,
    pub backend_sender: async_channel::Sender<BackendEvent>,

    // Recording state
    pub cpal_stream: Option<cpal::Stream>,
    pub recording_start: Option<std::time::Instant>,
    pub timer_source: Option<glib::SourceId>,
    pub sample_rate: u32,

    // Overlay phase tracking
    pub overlay_phase: Option<OverlayPhase>,
    pub overlay_dismiss_source: Option<glib::SourceId>,

    // UI handles
    pub dashboard: Option<DashboardWidgets>,
    pub overlay: Option<OverlayWidgets>,
}

impl AppState {
    pub fn new(sender: async_channel::Sender<BackendEvent>) -> Self {
        let config = Config::load();
        let stats = Stats::load();
        let shared_hotkey = Arc::new(Mutex::new(config.hotkey.clone()));
        let tokio_rt = tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime");

        Self {
            status: AppStatus::Idle,
            config,
            stats,
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            shared_hotkey,
            tokio_rt,
            whisper_ctx: None,
            backend_sender: sender,
            cpal_stream: None,
            recording_start: None,
            timer_source: None,
            sample_rate: 16000,
            overlay_phase: None,
            overlay_dismiss_source: None,
            dashboard: None,
            overlay: None,
        }
    }
}

/// Helper to update status label and state.
pub fn update_status(
    state: &std::rc::Rc<std::cell::RefCell<AppState>>,
    status: AppStatus,
    label_text: &str,
) {
    let mut s = state.borrow_mut();
    s.status = status;
    if let Some(ref dash) = s.dashboard {
        dash.status_label.set_text(label_text);
    }
}
