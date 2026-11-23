//! Throwaway-style probe, not product code: can cpal capture what Windows is
//! playing? On WASAPI, building an input stream on an output device turns the
//! stream into loopback capture.
//!
//! Run it once with sound playing and once in silence, then compare the two
//! result lines.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// How long each probe run listens.
const PROBE_SECONDS: u64 = 10;

fn main() {
    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        eprintln!("loopback_probe: no default output device");
        std::process::exit(1);
    };

    let device_name = match device.description() {
        Ok(description) => description.name().to_string(),
        Err(_) => "<unknown>".to_string(),
    };
    println!("device={device_name}");

    let supported = match device.default_output_config() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("loopback_probe: default_output_config failed: {err}");
            std::process::exit(1);
        }
    };

    let sample_format = supported.sample_format();
    let config = supported.config();
    println!(
        "rate={} channels={} format={sample_format:?}",
        config.sample_rate, config.channels
    );

    if sample_format != cpal::SampleFormat::F32 {
        eprintln!("loopback_probe: expected f32 samples, got {sample_format:?}");
        std::process::exit(2);
    }

    let frames = Arc::new(AtomicU64::new(0));
    let callbacks = Arc::new(AtomicU64::new(0));
    let peak_bits = Arc::new(AtomicU32::new(0f32.to_bits()));
    let channels = u64::from(config.channels);

    // No allocation, printing, or locks in the data callback.
    let callback_frames = Arc::clone(&frames);
    let callback_callbacks = Arc::clone(&callbacks);
    let callback_peak_bits = Arc::clone(&peak_bits);
    let data_callback = move |data: &[f32], _info: &cpal::InputCallbackInfo| {
        callback_callbacks.fetch_add(1, Ordering::Relaxed);
        callback_frames.fetch_add(data.len() as u64 / channels, Ordering::Relaxed);

        let mut local_peak = 0f32;
        for &sample in data {
            let magnitude = sample.abs();
            if magnitude > local_peak {
                local_peak = magnitude;
            }
        }
        // Non-negative floats compare like their bit patterns, so a max on the
        // bits is a max on the values and stays lock-free.
        callback_peak_bits.fetch_max(local_peak.to_bits(), Ordering::Relaxed);
    };

    let error_callback = |err| eprintln!("loopback_probe: stream error: {err}");

    let stream =
        match device.build_input_stream::<f32, _, _>(config, data_callback, error_callback, None) {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("loopback_probe: build_input_stream failed: {err}");
                std::process::exit(1);
            }
        };

    if let Err(err) = stream.play() {
        eprintln!("loopback_probe: play failed: {err}");
        std::process::exit(1);
    }

    std::thread::sleep(Duration::from_secs(PROBE_SECONDS));

    let peak = f32::from_bits(peak_bits.load(Ordering::Relaxed));
    println!(
        "frames={} rate={} channels={} peak={peak:.6} callbacks={}",
        frames.load(Ordering::Relaxed),
        config.sample_rate,
        config.channels,
        callbacks.load(Ordering::Relaxed)
    );
}
