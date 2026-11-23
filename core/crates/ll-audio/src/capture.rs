//! WASAPI loopback capture: system audio in, 16 kHz mono f32 out.
//!
//! cpal turns an input stream on an *output* device into loopback capture on
//! WASAPI. The data callback only pushes into a lock-free ring; all downmixing
//! and resampling happens on the pull side.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};

use crate::{downmix_to_mono, resample_linear, AudioError, AudioSource, TARGET_SAMPLE_RATE};

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
    scratch: Vec<f32>,
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

        let stream = build_stream(&device, config, producer, Arc::clone(&overruns))?;
        stream.play().map_err(device_error)?;

        Ok(Self {
            _stream: stream,
            consumer,
            device_rate,
            channels,
            overruns,
            scratch: Vec::new(),
        })
    }

    /// Native sample rate of the output device.
    pub fn device_rate(&self) -> u32 {
        self.device_rate
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
        let deadline = Instant::now() + EMPTY_WAIT;
        while self.consumer.occupied_len() < self.channels && Instant::now() < deadline {
            std::thread::sleep(POLL_INTERVAL);
        }

        if self.drain_available() == 0 {
            // Capture never ends; an empty buffer means "nothing played yet".
            return Some(Vec::new());
        }

        let mono = downmix_to_mono(&self.scratch, self.channels);
        // TODO(T02): switch to rubato for stateful streaming resampling; the
        // linear resampler restarts at every buffer boundary.
        Some(resample_linear(&mono, self.device_rate, TARGET_SAMPLE_RATE))
    }

    fn name(&self) -> &'static str {
        "loopback"
    }
}

fn build_stream(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut producer: HeapProd<f32>,
    overruns: Arc<AtomicU64>,
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
            // pull side sees the stall as empty buffers and stays alive.
            |_error| {},
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

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
