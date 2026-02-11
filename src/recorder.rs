use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Start capturing audio from the default input device.
/// Samples are appended to the shared buffer at ~16kHz mono f32.
/// Drop the returned `Stream` to stop recording.
pub fn start_capture(
    buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<(cpal::Stream, u32), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device found")?;

    log::info!("Input device: {:?}", device.description());

    let supported_configs: Vec<_> = device.supported_input_configs()?.collect();

    // Try to find a config that supports 16kHz mono
    let target_rate: u32 = 16000;
    let desired = supported_configs.iter().find(|c| {
        c.channels() == 1
            && c.min_sample_rate() <= target_rate
            && c.max_sample_rate() >= target_rate
            && c.sample_format() == cpal::SampleFormat::F32
    });

    let (config, native_rate, downsample_factor) = if let Some(cfg) = desired {
        let config = cfg.with_sample_rate(target_rate).config();
        (config, 16000u32, 1usize)
    } else {
        // Fall back to default config, downsample later
        let default_config = device.default_input_config()?;
        let rate = default_config.sample_rate();
        let factor = (rate / 16000).max(1) as usize;
        let actual_rate = rate / factor as u32;
        log::info!(
            "Using native rate {rate}Hz, downsampling by {factor}x to ~{actual_rate}Hz"
        );
        (default_config.config(), actual_rate, factor)
    };

    let channels = config.channels as usize;

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut buf = buffer.lock().unwrap();
            for (i, chunk) in data.chunks(channels).enumerate() {
                if i % downsample_factor == 0 {
                    let mono = chunk.iter().sum::<f32>() / channels as f32;
                    buf.push(mono);
                }
            }
        },
        |err| log::error!("Input stream error: {err}"),
        None,
    )?;

    stream.play()?;
    Ok((stream, native_rate))
}

/// Convert f32 samples to WAV bytes (mono 16-bit PCM).
#[allow(dead_code)]
pub fn samples_to_wav(
    samples: &[f32],
    sample_rate: u32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i16_val = (clamped * i16::MAX as f32) as i16;
        writer.write_sample(i16_val)?;
    }
    writer.finalize()?;
    Ok(cursor.into_inner())
}
