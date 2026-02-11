use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";
const MODEL_FILENAME: &str = "ggml-base.en.bin";

/// Directory for model storage: ~/.local/share/voice-prompt/models/
fn models_dir() -> PathBuf {
    let mut p = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("voice-prompt");
    p.push("models");
    p
}

fn model_path() -> PathBuf {
    models_dir().join(MODEL_FILENAME)
}

/// Check whether the whisper model file exists.
pub fn model_exists() -> bool {
    model_path().exists()
}

/// Download the whisper model, sending progress events via the provided callback.
/// `on_progress(bytes_downloaded, total_bytes)` — total may be 0 if unknown.
pub async fn download_model<F>(
    on_progress: F,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(u64, u64) + Send + 'static,
{
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let dir = models_dir();
    tokio::fs::create_dir_all(&dir).await?;

    let response = reqwest::get(MODEL_URL).await?;
    let total = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let path = model_path();
    let mut file = tokio::fs::File::create(&path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }

    file.flush().await?;
    log::info!("Model downloaded to {}", path.display());
    Ok(())
}

/// Load the whisper model from disk. This is CPU-heavy; call from a blocking context.
pub fn load_model() -> Result<WhisperContext, Box<dyn std::error::Error + Send + Sync>> {
    let path = model_path();
    let ctx = WhisperContext::new_with_params(
        path.to_str().ok_or("Invalid model path")?,
        WhisperContextParameters::default(),
    )
    .map_err(|e| format!("Failed to load whisper model: {e}"))?;
    log::info!("Whisper model loaded");
    Ok(ctx)
}

/// Transcribe audio samples (16kHz mono f32). CPU-heavy — call from `spawn_blocking`.
pub fn transcribe(
    ctx: &WhisperContext,
    samples: &[f32],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut state = ctx
        .create_state()
        .map_err(|e| format!("State error: {e}"))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let cpus = std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4);
    params.set_n_threads(cpus);

    state
        .full(params, samples)
        .map_err(|e| format!("Transcription failed: {e}"))?;

    let mut text = String::new();
    for segment in state.as_iter() {
        // WhisperSegment implements Display
        let seg_text = format!("{segment}");
        text.push_str(&seg_text);
        text.push(' ');
    }

    Ok(text.trim().to_string())
}
