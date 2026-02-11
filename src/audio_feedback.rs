use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;

/// Type of beep to play.
#[derive(Debug, Clone, Copy)]
pub enum BeepType {
    /// Start recording: ascending 600→900 Hz
    Start,
    /// Stop recording: descending 900→600 Hz
    Stop,
}

/// Play a short beep. Spawns a thread and returns immediately.
pub fn play_beep(beep: BeepType) {
    std::thread::spawn(move || {
        if let Err(e) = play_beep_blocking(beep) {
            log::warn!("Beep failed: {e}");
        }
    });
}

fn play_beep_blocking(beep: BeepType) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No output device found")?;
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;

    let duration_secs = 0.15_f32;
    let total_samples = (sample_rate * duration_secs) as usize;

    let (freq_start, freq_end) = match beep {
        BeepType::Start => (600.0_f32, 900.0_f32),
        BeepType::Stop => (900.0_f32, 600.0_f32),
    };

    // Pre-generate samples
    let mut samples = Vec::with_capacity(total_samples);
    for i in 0..total_samples {
        let t = i as f32 / sample_rate;
        let progress = i as f32 / total_samples as f32;
        let freq = freq_start + (freq_end - freq_start) * progress;
        // Fade-out envelope
        let envelope = 1.0 - progress;
        let value = (2.0 * PI * freq * t).sin() * envelope * 0.3;
        samples.push(value);
    }

    let sample_idx = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let sample_idx_clone = sample_idx.clone();
    let samples = std::sync::Arc::new(samples);
    let samples_clone = samples.clone();
    let total = total_samples;

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut idx = sample_idx_clone.load(std::sync::atomic::Ordering::Relaxed);
            for frame in data.chunks_mut(channels) {
                let value = if idx < total {
                    samples_clone[idx]
                } else {
                    0.0
                };
                for sample in frame.iter_mut() {
                    *sample = value;
                }
                idx += 1;
            }
            sample_idx_clone.store(idx, std::sync::atomic::Ordering::Relaxed);
        },
        |err| log::error!("Audio output error: {err}"),
        None,
    )?;

    stream.play()?;

    // Wait for playback to finish + small buffer
    std::thread::sleep(std::time::Duration::from_millis(200));

    drop(stream);
    Ok(())
}
