//! WASAPI loopback capture: system audio in, 16 kHz mono f32 out.
//!
//! cpal turns an input stream on an *output* device into loopback capture on
//! WASAPI. The data callback only pushes into a lock-free ring; all downmixing
//! and resampling happens on the pull side.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};

use crate::{downmix_to_mono, AudioError, AudioSource, TARGET_SAMPLE_RATE};

/// Input chunk the FFT resampler consumes per call, in device-rate frames.
const RESAMPLE_CHUNK: usize = 1024;

/// Ring capacity in seconds of device-rate audio.
const CAPTURE_RING_SECONDS: usize = 2;

/// How long `next_buffer` waits for samples before returning an empty buffer.
const EMPTY_WAIT: Duration = Duration::from_millis(50);
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// System-audio capture source. Dropping it stops capture.
pub struct LoopbackCapture {
    // Held only to keep the stream alive.
    _stream: cpal::Stream,
    consumer: HeapCons<f32>,
    device_rate: u32,
    channels: usize,
    overruns: Arc<AtomicU64>,
    device_lost: Arc<AtomicBool>,
    device_name: String,
    scratch: Vec<f32>,
    resampler: rubato::Fft<f32>,
    /// Mono device-rate samples waiting for a full resampler input chunk.
    pending: Vec<f32>,
}

impl LoopbackCapture {
    /// Open the default output device in loopback mode and start streaming.
    pub fn start_default() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| AudioError::Device("no default output device".into()))?;

        let supported = device.default_output_config().map_err(device_error)?;
        let sample_format = supported.sample_format();
        if sample_format != cpal::SampleFormat::F32 {
            return Err(AudioError::Device(format!(
                "default output format {sample_format:?} is not f32"
            )));
        }

        let config = supported.config();
        let device_rate = config.sample_rate;
        let channels = usize::from(config.channels);
        if channels == 0 {
            return Err(AudioError::Device(
                "default output device reports zero channels".into(),
            ));
        }

        let capacity = device_rate as usize * channels * CAPTURE_RING_SECONDS;
        let (producer, consumer) = HeapRb::<f32>::new(capacity).split();
        let overruns = Arc::new(AtomicU64::new(0));
        let device_lost = Arc::new(AtomicBool::new(false));
        let device_name = device
            .description()
            .map(|description| description.name().to_string())
            .unwrap_or_else(|_| "unknown output".to_string());
        let resampler = build_resampler(device_rate)?;

        let stream = build_stream(
            &device,
            config,
            producer,
            Arc::clone(&overruns),
            Arc::clone(&device_lost),
        )?;
        stream.play().map_err(device_error)?;

        Ok(Self {
            _stream: stream,
            consumer,
            device_rate,
            channels,
            overruns,
            device_lost,
            device_name,
            scratch: Vec::new(),
            resampler,
            pending: Vec::new(),
        })
    }

    /// Native sample rate of the output device.
    pub fn device_rate(&self) -> u32 {
        self.device_rate
    }

    /// Name of the output device this capture opened.
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// True once the stream reported a device error; capture will not recover.
    pub fn is_device_lost(&self) -> bool {
        end_capture(&self.device_lost)
    }

    /// Number of callbacks whose samples did not fully fit in the ring.
    pub fn overrun_events(&self) -> u64 {
        self.overruns.load(Ordering::Relaxed)
    }

    /// Move whole interleaved frames from the ring into the scratch buffer.
    fn drain_available(&mut self) -> usize {
        let available = self.consumer.occupied_len();
        let usable = available - available % self.channels;
        self.scratch.clear();
        self.scratch.reserve(usable);
        for _ in 0..usable {
            match self.consumer.try_pop() {
                Some(sample) => self.scratch.push(sample),
                None => break,
            }
        }
        self.scratch.len()
    }
}

impl AudioSource for LoopbackCapture {
    fn next_buffer(&mut self) -> Option<Vec<f32>> {
        // A lost device ends the source; reopening is the caller's decision.
        if end_capture(&self.device_lost) {
            return None;
        }
        let deadline = Instant::now() + EMPTY_WAIT;
        while self.consumer.occupied_len() < self.channels && Instant::now() < deadline {
            std::thread::sleep(POLL_INTERVAL);
        }

        if self.drain_available() == 0 {
            // Capture never ends; an empty buffer means "nothing played yet".
            return Some(Vec::new());
        }

        let mono = downmix_to_mono(&self.scratch, self.channels);
        self.pending.extend_from_slice(&mono);

        let mut resampled = Vec::new();
        drain_resampled(&mut self.resampler, &mut self.pending, &mut resampled);
        Some(resampled)
    }

    fn name(&self) -> &'static str {
        "loopback"
    }
}

/// Stateful resampler from the device rate to [`TARGET_SAMPLE_RATE`].
fn build_resampler(device_rate: u32) -> Result<rubato::Fft<f32>, AudioError> {
    rubato::Fft::<f32>::new(
        device_rate as usize,
        TARGET_SAMPLE_RATE as usize,
        RESAMPLE_CHUNK,
        1,
        rubato::FixedSync::Input,
    )
    .map_err(|error| AudioError::Device(format!("cannot resample {device_rate} Hz: {error}")))
}

/// Resample every whole chunk in `pending` into `out`, keeping the remainder.
///
/// The resampler keeps its state across calls, so buffer boundaries no longer
/// restart the filter. A resampler error stops the loop instead of panicking:
/// capture must stay alive.
fn drain_resampled(resampler: &mut rubato::Fft<f32>, pending: &mut Vec<f32>, out: &mut Vec<f32>) {
    use rubato::audioadapter_buffers::direct::SequentialSliceOfVecs;
    use rubato::Resampler;

    loop {
        let needed = resampler.input_frames_next();
        if pending.len() < needed {
            return;
        }
        let chunk = [pending[..needed].to_vec()];
        let Ok(input) = SequentialSliceOfVecs::new(&chunk, 1, needed) else {
            return;
        };
        let Ok(processed) = resampler.process(&input, None) else {
            return;
        };
        out.extend(processed.take_data());
        pending.drain(..needed);
    }
}

/// True when the capture stream has reported a device error.
///
/// Kept separate from `next_buffer` so the decision is testable without a
/// device.
fn end_capture(device_lost: &AtomicBool) -> bool {
    device_lost.load(Ordering::Relaxed)
}

/// Whether a stream error ends the capture.
///
/// cpal 0.18 reports routine buffer underruns as `Xrun` and route changes as
/// `DeviceChanged` (the stream stays alive), so only a genuinely gone device
/// or an invalidated stream counts as loss.
fn ends_capture(error: &cpal::Error) -> bool {
    matches!(
        error.kind(),
        cpal::ErrorKind::DeviceNotAvailable
            | cpal::ErrorKind::StreamInvalidated
            | cpal::ErrorKind::HostUnavailable
    )
}

fn build_stream(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut producer: HeapProd<f32>,
    overruns: Arc<AtomicU64>,
    device_lost: Arc<AtomicBool>,
) -> Result<cpal::Stream, AudioError> {
    device
        .build_input_stream::<f32, _, _>(
            config,
            move |data: &[f32], _info| {
                let pushed = producer.push_slice(data);
                if pushed < data.len() {
                    overruns.fetch_add(1, Ordering::Relaxed);
                }
            },
            // A device error cannot be reported through this callback; the
            // pull side sees the flag and ends the source.
            move |error| {
                if ends_capture(&error) {
                    device_lost.store(true, Ordering::Relaxed);
                }
            },
            None,
        )
        .map_err(device_error)
}

fn device_error(error: cpal::Error) -> AudioError {
    AudioError::Device(error.to_string())
}

/// True when cpal sees a default output device that loopback can capture.
pub fn default_output_available() -> bool {
    cpal::default_host().default_output_device().is_some()
}

/// Name of the current default output device, if there is one.
pub fn default_device_name() -> Option<String> {
    cpal::default_host()
        .default_output_device()
        .and_then(|device| device.description().ok())
        .map(|description| description.name().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn device_loss_ends_the_capture_source() {
        // `next_buffer` returns `None` on this flag; the real `LoopbackCapture`
        // cannot be built without a device, so the decision is tested directly.
        let lost = AtomicBool::new(false);
        assert!(!end_capture(&lost));
        lost.store(true, Ordering::Relaxed);
        assert!(end_capture(&lost));
    }

    #[test]
    fn only_a_gone_device_ends_the_capture() {
        let gone = cpal::Error::new(cpal::ErrorKind::DeviceNotAvailable);
        assert!(ends_capture(&gone));
        assert!(ends_capture(&cpal::Error::new(
            cpal::ErrorKind::StreamInvalidated
        )));
        // Routine underruns and reroutes keep the stream alive.
        assert!(!ends_capture(&cpal::Error::new(cpal::ErrorKind::Xrun)));
        assert!(!ends_capture(&cpal::Error::new(
            cpal::ErrorKind::DeviceChanged
        )));
    }

    #[test]
    fn resampler_keeps_sine_length_and_level() {
        fn rms(samples: &[f32]) -> f32 {
            (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
        }

        let mut resampler = build_resampler(48_000).expect("build 48 kHz resampler");
        let sine: Vec<f32> = (0..48_000)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48_000.0).sin() * 0.5)
            .collect();
        let input_rms = rms(&sine);

        // Seven pieces, none a multiple of the 1024-frame chunk size, so chunk
        // boundaries land inside pieces.
        let mut pending = Vec::new();
        let mut output = Vec::new();
        let mut start = 0;
        for len in [6_000, 7_000, 5_000, 8_000, 6_500, 5_500, 10_000] {
            pending.extend_from_slice(&sine[start..start + len]);
            drain_resampled(&mut resampler, &mut pending, &mut output);
            start += len;
        }

        assert_eq!(start, sine.len());
        let expected = 16_000usize;
        let difference = output.len().abs_diff(expected);
        assert!(
            difference <= 1_024,
            "resampled {} frames, expected {expected} ± 1024",
            output.len()
        );
        let output_rms = rms(&output);
        assert!(
            (output_rms - input_rms).abs() <= 0.1 * input_rms,
            "RMS changed from {input_rms} to {output_rms}"
        );
    }

    #[test]
    #[ignore = "needs an output device and plays the fixture wav"]
    fn loopback_captures_playing_audio() {
        let wav = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../tests/fixtures/sample.wav");
        let script = format!(
            "$p = New-Object System.Media.SoundPlayer '{}'; $p.PlayLooping(); Start-Sleep -Seconds 5; $p.Stop()",
            wav.display()
        );
        let mut player = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .spawn()
            .expect("spawn powershell SoundPlayer");

        let mut capture = LoopbackCapture::start_default().expect("start loopback capture");
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut heard = false;
        while Instant::now() < deadline && !heard {
            let buffer = capture.next_buffer().expect("capture never ends");
            if !buffer.is_empty() {
                let rms = (buffer.iter().map(|sample| sample * sample).sum::<f32>()
                    / buffer.len() as f32)
                    .sqrt();
                heard = rms > 0.001;
            }
        }

        let _ = player.kill();
        let _ = player.wait();
        assert!(heard, "no loopback audio above 0.001 RMS in 5 s");
    }
}
