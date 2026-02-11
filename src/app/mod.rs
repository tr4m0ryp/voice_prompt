mod event_handler;
mod model;
mod pipeline;
mod recording;
mod state;

pub use event_handler::handle_backend_event;
pub use model::ensure_whisper_model;
pub use state::{AppState, BackendEvent, OverlayPhase};
