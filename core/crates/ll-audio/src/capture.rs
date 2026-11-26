//! WASAPI capture for system audio (loopback) and microphone input.
//!
//! cpal turns an input stream on an *output* device into loopback capture on
//! WASAPI, and an input stream on an *input* device into microphone capture.
//! The data callback only pushes into a lock-free ring; all downmixing and
//! resampling happens on the pull side.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
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

/// What a capture stream listens to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    /// The mix of an output (render) endpoint, via WASAPI loopback.
    Loopback,
    /// A microphone or headset input endpoint.
    Microphone,
}

impl CaptureKind {
    /// Human-readable name used in diagnostics and error messages.
    pub fn label(self) -> &'static str {
        match self {
            Self::Loopback => "system audio",
            Self::Microphone => "microphone",
        }
    }

    fn default_device(self) -> Option<cpal::Device> {
        let host = cpal::default_host();
        match self {
            Self::Loopback => host.default_output_device(),
            Self::Microphone => host.default_input_device(),
        }
    }

    fn default_config(
        self,
        device: &cpal::Device,
    ) -> Result<cpal::SupportedStreamConfig, cpal::Error> {
        match self {
            Self::Loopback => device.default_output_config(),
            Self::Microphone => device.default_input_config(),
        }
    }
}

/// One selectable Windows audio endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureDevice {
    /// Stable cpal device id (`wasapi:{endpoint}`), persisted by the UI.
    pub id: String,
    /// Friendly endpoint name for the picker.
    pub name: String,
    /// True for the Windows default endpoint for this direction.
    pub is_default: bool,
}

/// A running capture. Dropping it stops capture.
pub struct DeviceCapture {
    // Held only to keep the stream alive.
    _stream: cpal::Stream,
    consumer: HeapCons<f32>,
    device_rate: u32,
    channels: usize,
    overruns: Arc<AtomicU64>,
    device_lost: Arc<AtomicBool>,
    device_name: String,
    device_id: String,
    kind: CaptureKind,
    /// True when the endpoint is followed as Windows' default changes.
    follows_default: bool,
    scratch: Vec<f32>,
    resampler: rubato::Fft<f32>,
    /// Mono device-rate samples waiting for a full resampler input chunk.
    pending: Vec<f32>,
}

/// Backwards-compatible name for system-audio capture.
pub type LoopbackCapture = DeviceCapture;
/// Microphone capture uses the same resampling and buffering path.
pub type MicrophoneCapture = DeviceCapture;

impl DeviceCapture {
    /// Open the given endpoint (or the default for the kind) and start streaming.
    ///
    /// An explicit `device_id` that is missing or malformed is an error: capture
    /// never silently falls back to a different device.
    pub fn start(kind: CaptureKind, device_id: Option<&str>) -> Result<Self, AudioError> {
        let requested = device_id.filter(|id| !id.is_empty());
        let device = select_device(kind, requested)?;
        let follows_default = requested.is_none();
        let supported = kind.default_config(&device).map_err(device_error)?;
        let sample_format = supported.sample_format();
        let config = supported.config();
        let device_rate = config.sample_rate;
        let channels = usize::from(config.channels);
        if channels == 0 {
            return Err(AudioError::Device(format!(
                "{} device reports zero channels",
                kind.label()
            )));
        }

        let capacity = device_rate as usize * channels * CAPTURE_RING_SECONDS;
        let (producer, consumer) = HeapRb::<f32>::new(capacity).split();
        let overruns = Arc::new(AtomicU64::new(0));
        let device_lost = Arc::new(AtomicBool::new(false));
        let device_name = endpoint_name(&device);
        let device_id = endpoint_id(&device).ok_or_else(|| {
            AudioError::Device(format!("{} device has no stable endpoint id", kind.label()))
        })?;
        let resampler = build_resampler(device_rate)?;

        let stream = build_stream(
            &device,
            config,
            sample_format,
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
            device_id,
            kind,
            follows_default,
            scratch: Vec::new(),
            resampler,
            pending: Vec::new(),
        })
    }

    /// Open the default output device in loopback mode.
    pub fn start_loopback(device_id: Option<&str>) -> Result<Self, AudioError> {
        Self::start(CaptureKind::Loopback, device_id)
    }

    /// Open the default microphone (or a selected input endpoint).
    pub fn start_microphone(device_id: Option<&str>) -> Result<Self, AudioError> {
        Self::start(CaptureKind::Microphone, device_id)
    }

    /// Keep the historical default (system audio) for existing callers.
    pub fn start_default() -> Result<Self, AudioError> {
        Self::start_loopback(None)
    }

    /// Reopen the same endpoint, or the current default when following it.
    pub fn restart(&self) -> Result<Self, AudioError> {
        Self::start(self.kind, self.requested_device_id())
    }

    /// What this capture listens to.
    pub fn kind(&self) -> CaptureKind {
        self.kind
    }

    /// True when the capture follows Windows' default endpoint for its kind.
    pub fn follows_default(&self) -> bool {
        self.follows_default
    }

    /// The selected endpoint when one was requested; `None` follows the default.
    pub fn requested_device_id(&self) -> Option<&str> {
        if self.follows_default {
            None
        } else {
            Some(&self.device_id)
        }
    }

    /// Native sample rate of the endpoint.
    pub fn device_rate(&self) -> u32 {
        self.device_rate
    }

    /// Name of the endpoint this capture opened.
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Stable id of the endpoint this capture opened.
    pub fn device_id(&self) -> &str {
        &self.device_id
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

impl AudioSource for DeviceCapture {
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
        match self.kind {
            CaptureKind::Loopback => "loopback",
            CaptureKind::Microphone => "microphone",
        }
    }
}

/// Endpoint ids, names, and the default for one direction.
pub fn list_devices(kind: CaptureKind) -> Vec<CaptureDevice> {
    let host = cpal::default_host();
    let default_id = default_device_id(kind);
    let devices = match kind {
        CaptureKind::Loopback => host.output_devices(),
        CaptureKind::Microphone => host.input_devices(),
    };
    let Ok(devices) = devices else {
        return Vec::new();
    };
    let mut listed = Vec::new();
    for device in devices {
        let Ok(id) = device.id() else {
            // Without an id the endpoint cannot be selected or persisted.
            continue;
        };
        let id = id.to_string();
        let name = endpoint_name(&device);
        listed.push(CaptureDevice {
            is_default: default_id.as_deref() == Some(id.as_str()),
            id,
            name,
        });
    }
    listed.sort_by(|left, right| {
        right
            .is_default
            .cmp(&left.is_default)
            .then_with(|| left.name.cmp(&right.name))
    });
    listed
}

/// Stable id of the current default endpoint for this direction.
pub fn default_device_id(kind: CaptureKind) -> Option<String> {
    kind.default_device()
        .and_then(|device| endpoint_id(&device))
}

/// Name of the current default endpoint for this direction.
pub fn default_device_name(kind: CaptureKind) -> Option<String> {
    kind.default_device().map(|device| endpoint_name(&device))
}

fn endpoint_name(device: &cpal::Device) -> String {
    device
        .description()
        .map(|description| description.name().to_string())
        .unwrap_or_else(|_| device.to_string())
}

fn endpoint_id(device: &cpal::Device) -> Option<String> {
    device.id().ok().map(|id| id.to_string())
}

/// Resolve an explicit endpoint by id, or the current default.
fn select_device(kind: CaptureKind, requested: Option<&str>) -> Result<cpal::Device, AudioError> {
    let Some(raw_id) = requested else {
        return kind
            .default_device()
            .ok_or_else(|| AudioError::Device(format!("no default {} device", kind.label())));
    };
    let id: cpal::DeviceId = raw_id.parse().map_err(|error| {
        AudioError::Device(format!("invalid {} device id: {error}", kind.label()))
    })?;
    cpal::default_host().device_by_id(&id).ok_or_else(|| {
        AudioError::Device(format!("selected {} device is not available", kind.label()))
    })
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

fn stream_error_handler(device_lost: Arc<AtomicBool>) -> impl FnMut(cpal::Error) + Send + 'static {
    move |error| {
        if ends_capture(&error) {
            device_lost.store(true, Ordering::Relaxed);
        }
    }
}

fn build_stream(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
    producer: HeapProd<f32>,
    overruns: Arc<AtomicU64>,
    device_lost: Arc<AtomicBool>,
) -> Result<cpal::Stream, AudioError> {
    match sample_format {
        cpal::SampleFormat::F32 => {
            build_typed_stream::<f32>(device, config, producer, overruns, device_lost)
        }
        cpal::SampleFormat::F64 => {
            build_typed_stream::<f64>(device, config, producer, overruns, device_lost)
        }
        cpal::SampleFormat::I16 => {
            build_typed_stream::<i16>(device, config, producer, overruns, device_lost)
        }
        cpal::SampleFormat::U16 => {
            build_typed_stream::<u16>(device, config, producer, overruns, device_lost)
        }
        cpal::SampleFormat::I32 => {
            build_typed_stream::<i32>(device, config, producer, overruns, device_lost)
        }
        cpal::SampleFormat::U32 => {
            build_typed_stream::<u32>(device, config, producer, overruns, device_lost)
        }
        cpal::SampleFormat::I8 => {
            build_typed_stream::<i8>(device, config, producer, overruns, device_lost)
        }
        cpal::SampleFormat::U8 => {
            build_typed_stream::<u8>(device, config, producer, overruns, device_lost)
        }
        other => Err(AudioError::Device(format!(
            "unsupported capture sample format {other:?}"
        ))),
    }
}

fn build_typed_stream<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut producer: HeapProd<f32>,
    overruns: Arc<AtomicU64>,
    device_lost: Arc<AtomicBool>,
) -> Result<cpal::Stream, AudioError>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    device
        .build_input_stream::<T, _, _>(
            config,
            move |data: &[T], _info| {
                let mut dropped = 0usize;
                for &sample in data {
                    if producer.try_push(f32::from_sample(sample)).is_err() {
                        dropped += 1;
                    }
                }
                if dropped > 0 {
                    overruns.fetch_add(1, Ordering::Relaxed);
                }
            },
            stream_error_handler(device_lost),
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

/// True when cpal sees a default microphone or input endpoint.
pub fn default_input_available() -> bool {
    cpal::default_host().default_input_device().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn device_loss_ends_the_capture_source() {
        // `next_buffer` returns `None` on this flag; the real `DeviceCapture`
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
    fn unknown_device_id_is_an_error_not_the_default() {
        let error = select_device(CaptureKind::Microphone, Some("not-a-device-id"))
            .expect_err("a malformed id must not open the default microphone");
        assert!(matches!(error, AudioError::Device(_)));
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

        let mut capture = DeviceCapture::start_loopback(None).expect("start loopback capture");
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

    #[test]
    #[ignore = "needs a microphone endpoint and Windows microphone permission"]
    fn microphone_opens_and_reports_its_endpoint() {
        let capture = DeviceCapture::start_microphone(None).expect("open default microphone");
        assert_eq!(capture.kind(), CaptureKind::Microphone);
        assert!(capture.follows_default());
        assert!(!capture.device_name().is_empty());
        assert!(!capture.device_id().is_empty());
    }

    #[test]
    #[ignore = "needs audio endpoints"]
    fn every_listed_endpoint_opens_by_id() {
        for kind in [CaptureKind::Loopback, CaptureKind::Microphone] {
            for device in list_devices(kind) {
                let capture = DeviceCapture::start(kind, Some(&device.id))
                    .unwrap_or_else(|error| panic!("open {} {}: {error}", kind.label(), device.id));
                assert_eq!(capture.device_id(), device.id);
                assert!(!capture.follows_default());
                assert_eq!(capture.requested_device_id(), Some(device.id.as_str()));
            }
        }
    }

    #[test]
    #[ignore = "prints the host's audio endpoints; no capture is opened"]
    fn print_audio_endpoints() {
        for kind in [CaptureKind::Loopback, CaptureKind::Microphone] {
            let devices = list_devices(kind);
            println!("{}: {} endpoint(s)", kind.label(), devices.len());
            for device in devices {
                println!(
                    "  default={} id={} name={}",
                    device.is_default, device.id, device.name
                );
            }
        }
    }
}
