//! Audio sources, WAV decoding, resampling, VAD, chunking, and the 10-second
//! ring buffer (plan §7.3, §13).
//!
//! The audio callback path must not allocate, lock, log, or touch JS. This
//! crate provides the building blocks; the pipeline worker owns the callback
//! handoff.

use ll_core::demo::{DemoConversation, DemoPhase};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::HeapRb;
use std::collections::VecDeque;
use std::f32::consts::PI;
use thiserror::Error;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;
pub const RING_SECONDS: usize = 10;
pub const RING_SAMPLES: usize = TARGET_SAMPLE_RATE as usize * RING_SECONDS;
pub const CHUNK_SAMPLES: usize = 17_600;
pub const CHUNK_DURATION: f64 = 1.1;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("invalid WAV: {0}")]
    InvalidWav(String),
    #[error("unsupported WAV format: {0}")]
    UnsupportedWav(String),
    #[error("audio device error: {0}")]
    Device(String),
    #[error("audio source is exhausted")]
    Exhausted,
}

/// A pull-based audio source. Buffers are 16 kHz mono f32.
pub trait AudioSource: Send {
    fn next_buffer(&mut self) -> Option<Vec<f32>>;
    fn name(&self) -> &'static str;
    /// Total duration in seconds when known (demo/file/silence).
    fn duration_seconds(&self) -> Option<f64> {
        None
    }
}

/// A decoded WAV file (mono f32 at its native rate).
#[derive(Debug, Clone, PartialEq)]
pub struct WavData {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

impl WavData {
    pub fn mono_16k(&self) -> Vec<f32> {
        resample_linear(&self.samples, self.sample_rate, TARGET_SAMPLE_RATE)
    }
}

/// Decode a RIFF/WAVE file: PCM 16-bit, PCM 24-bit, or IEEE float32.
pub fn decode_wav(bytes: &[u8]) -> Result<WavData, AudioError> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(AudioError::InvalidWav("missing RIFF/WAVE header".into()));
    }
    let mut offset = 12usize;
    let mut format = None;
    let mut channels = 0u16;
    let mut sample_rate = 0u32;
    let mut bits = 0u16;
    let mut data: Option<&[u8]> = None;
    while offset + 8 <= bytes.len() {
        let id = &bytes[offset..offset + 4];
        let size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let body_start = offset + 8;
        let body_end = body_start.saturating_add(size).min(bytes.len());
        match id {
            b"fmt " => {
                if body_end - body_start < 16 {
                    return Err(AudioError::InvalidWav("short fmt chunk".into()));
                }
                let body = &bytes[body_start..body_end];
                format = Some(u16::from_le_bytes(body[0..2].try_into().unwrap()));
                channels = u16::from_le_bytes(body[2..4].try_into().unwrap());
                sample_rate = u32::from_le_bytes(body[4..8].try_into().unwrap());
                bits = u16::from_le_bytes(body[14..16].try_into().unwrap());
            }
            b"data" => data = Some(&bytes[body_start..body_end]),
            _ => {}
        }
        offset = body_start + size + (size % 2);
    }
    let format = format.ok_or_else(|| AudioError::InvalidWav("missing fmt chunk".into()))?;
    let data = data.ok_or_else(|| AudioError::InvalidWav("missing data chunk".into()))?;
    if channels == 0 || sample_rate == 0 {
        return Err(AudioError::InvalidWav(
            "zero channels or sample rate".into(),
        ));
    }
    let samples = match (format, bits) {
        (1, 16) => data
            .chunks_exact(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32768.0)
            .collect::<Vec<_>>(),
        (1, 24) => data
            .chunks_exact(3)
            .map(|triple| {
                let value = i32::from_le_bytes([0, triple[0], triple[1], triple[2]]) >> 8;
                value as f32 / 8_388_608.0
            })
            .collect::<Vec<_>>(),
        (3, 32) => data
            .chunks_exact(4)
            .map(|quad| f32::from_le_bytes([quad[0], quad[1], quad[2], quad[3]]))
            .collect::<Vec<_>>(),
        (other, bits) => {
            return Err(AudioError::UnsupportedWav(format!(
                "format {other}, {bits} bits"
            )))
        }
    };
    let samples = if channels > 1 {
        samples
            .chunks(channels as usize)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples
    };
    Ok(WavData {
        sample_rate,
        channels,
        samples,
    })
}

/// Linear resampler. Deterministic and dependency-free; the device capture
/// path uses the same function so eval and live share one conversion.
pub fn resample_linear(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || samples.is_empty() || from_rate == 0 || to_rate == 0 {
        return samples.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = ((samples.len() as f64) / ratio).floor() as usize;
    let mut output = Vec::with_capacity(output_len);
    for index in 0..output_len {
        let position = index as f64 * ratio;
        let left = position.floor() as usize;
        let fraction = (position - left as f64) as f32;
        let a = samples.get(left).copied().unwrap_or(0.0);
        let b = samples.get(left + 1).copied().unwrap_or(a);
        output.push(a + (b - a) * fraction);
    }
    output
}

/// Synthetic speech/silence source (demo mode and tests).
#[derive(Debug, Clone)]
pub struct SyntheticSource {
    phases: Vec<DemoPhase>,
    repeats: usize,
    paced: bool,
    speech_amplitude: f32,
    buffer_duration: f64,
    repeat_index: usize,
    phase_index: usize,
    phase_remaining: f64,
    sample_offset: u64,
    delivered: f64,
    total_seconds: f64,
}

impl SyntheticSource {
    pub fn new(phases: Vec<DemoPhase>, repeats: usize, paced: bool) -> Self {
        let total_seconds = phases
            .iter()
            .map(|phase| match phase {
                DemoPhase::Speech(seconds) | DemoPhase::Silence(seconds) => *seconds,
            })
            .sum::<f64>()
            * repeats as f64;
        Self {
            phases,
            repeats,
            paced,
            speech_amplitude: 0.3,
            buffer_duration: 0.1,
            repeat_index: 0,
            phase_index: 0,
            phase_remaining: 0.0,
            sample_offset: 0,
            delivered: 0.0,
            total_seconds,
        }
    }

    pub fn from_conversation(conversation: &DemoConversation, repeats: usize, paced: bool) -> Self {
        Self::new(conversation.audio_pattern.clone(), repeats, paced)
    }

    pub fn silence(seconds: f64) -> Self {
        Self::new(vec![DemoPhase::Silence(seconds)], 1, false)
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.speech_amplitude = amplitude;
        self
    }

    fn samples_per_buffer(&self) -> usize {
        (TARGET_SAMPLE_RATE as f64 * self.buffer_duration) as usize
    }
}

impl AudioSource for SyntheticSource {
    fn next_buffer(&mut self) -> Option<Vec<f32>> {
        if self.phases.is_empty() || self.repeat_index >= self.repeats {
            return None;
        }
        if self.phase_remaining <= 0.0001 {
            if self.phase_index >= self.phases.len() {
                self.repeat_index += 1;
                self.phase_index = 0;
                if self.repeat_index >= self.repeats {
                    return None;
                }
            }
            let phase = self.phases[self.phase_index];
            self.phase_index += 1;
            self.phase_remaining = match phase {
                DemoPhase::Speech(seconds) | DemoPhase::Silence(seconds) => seconds,
            };
        }
        let samples_per_buffer = self.samples_per_buffer();
        let amplitude = matches!(
            self.phases
                .get(self.phase_index.saturating_sub(1))
                .copied()
                .unwrap_or(DemoPhase::Silence(0.0)),
            DemoPhase::Speech(_)
        )
        .then_some(self.speech_amplitude)
        .unwrap_or(0.0);
        let mut buffer = Vec::with_capacity(samples_per_buffer);
        for _ in 0..samples_per_buffer {
            let value = if amplitude > 0.0 {
                amplitude * (self.sample_offset as f32 * 2.0 * PI * 220.0 / 16_000.0).sin()
            } else {
                0.0
            };
            buffer.push(value);
            self.sample_offset = self.sample_offset.wrapping_add(1);
        }
        self.phase_remaining -= self.buffer_duration;
        self.delivered += self.buffer_duration;
        if self.paced {
            std::thread::sleep(std::time::Duration::from_millis(
                (self.buffer_duration * 1000.0) as u64,
            ));
        }
        Some(buffer)
    }

    fn name(&self) -> &'static str {
        "synthetic"
    }

    fn duration_seconds(&self) -> Option<f64> {
        Some(self.total_seconds)
    }
}

/// A WAV file replayed as 0.1 s buffers at 16 kHz.
#[derive(Debug, Clone)]
pub struct FileSource {
    samples: Vec<f32>,
    position: usize,
    buffer_samples: usize,
    paced: bool,
}

impl FileSource {
    pub fn from_bytes(bytes: &[u8], paced: bool) -> Result<Self, AudioError> {
        let wav = decode_wav(bytes)?;
        Ok(Self {
            samples: wav.mono_16k(),
            position: 0,
            buffer_samples: 1_600,
            paced,
        })
    }

    pub fn from_path(path: &std::path::Path, paced: bool) -> Result<Self, AudioError> {
        let bytes = std::fs::read(path)
            .map_err(|error| AudioError::InvalidWav(format!("{}: {error}", path.display())))?;
        Self::from_bytes(&bytes, paced)
    }

    pub fn duration_seconds(&self) -> f64 {
        self.samples.len() as f64 / TARGET_SAMPLE_RATE as f64
    }
}

impl AudioSource for FileSource {
    fn next_buffer(&mut self) -> Option<Vec<f32>> {
        if self.position >= self.samples.len() {
            return None;
        }
        let end = (self.position + self.buffer_samples).min(self.samples.len());
        let buffer = self.samples[self.position..end].to_vec();
        self.position = end;
        if self.paced {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Some(buffer)
    }

    fn name(&self) -> &'static str {
        "file"
    }

    fn duration_seconds(&self) -> Option<f64> {
        Some(self.samples.len() as f64 / TARGET_SAMPLE_RATE as f64)
    }
}

/// Fixed-size chunker for the streaming ASR encoder (1.1 s = 17,600 samples).
#[derive(Debug, Default)]
pub struct AudioChunker {
    buffer: Vec<f32>,
}

impl AudioChunker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, samples: &[f32]) -> Vec<Vec<f32>> {
        self.buffer.extend_from_slice(samples);
        let mut chunks = Vec::new();
        while self.buffer.len() >= CHUNK_SAMPLES {
            chunks.push(self.buffer.drain(..CHUNK_SAMPLES).collect());
        }
        chunks
    }

    /// Zero-pad the trailing partial chunk at end of session.
    pub fn flush(&mut self) -> Option<Vec<f32>> {
        if self.buffer.is_empty() {
            return None;
        }
        let mut chunk = std::mem::take(&mut self.buffer);
        chunk.resize(CHUNK_SAMPLES, 0.0);
        Some(chunk)
    }

    pub fn pending(&self) -> usize {
        self.buffer.len()
    }
}

/// Energy + hangover VAD (v1 heuristic, ported from the Mac reference).
#[derive(Debug, Clone, Copy)]
pub struct VadConfig {
    pub frame_length: usize,
    pub energy_threshold: f32,
    pub hangover_duration: f64,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            frame_length: 320,
            energy_threshold: 0.01,
            hangover_duration: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VadStatus {
    pub is_speech_active: bool,
    pub trailing_silence: f64,
}

#[derive(Debug, Clone)]
pub struct VoiceActivityDetector {
    config: VadConfig,
    samples_since_voice: usize,
    has_heard_speech: bool,
    pending: Vec<f32>,
    voiced_sample_count: usize,
}

impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self::new(VadConfig::default())
    }
}

impl VoiceActivityDetector {
    pub fn new(config: VadConfig) -> Self {
        Self {
            config,
            samples_since_voice: 0,
            has_heard_speech: false,
            pending: Vec::new(),
            voiced_sample_count: 0,
        }
    }

    pub fn process(&mut self, samples: &[f32]) -> VadStatus {
        self.pending.extend_from_slice(samples);
        let mut offset = 0usize;
        while self.pending.len() - offset >= self.config.frame_length {
            let frame = &self.pending[offset..offset + self.config.frame_length];
            offset += self.config.frame_length;
            let sum = frame.iter().map(|sample| sample * sample).sum::<f32>();
            let rms = (sum / self.config.frame_length as f32).sqrt();
            if rms >= self.config.energy_threshold {
                self.samples_since_voice = 0;
                self.has_heard_speech = true;
                self.voiced_sample_count += self.config.frame_length;
            } else {
                self.samples_since_voice += self.config.frame_length;
            }
        }
        self.pending.drain(..offset);
        self.status()
    }

    pub fn status(&self) -> VadStatus {
        let trailing = self.samples_since_voice as f64 / TARGET_SAMPLE_RATE as f64;
        VadStatus {
            is_speech_active: self.has_heard_speech && trailing < self.config.hangover_duration,
            trailing_silence: trailing,
        }
    }

    pub fn total_voiced_duration(&self) -> f64 {
        self.voiced_sample_count as f64 / TARGET_SAMPLE_RATE as f64
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.config);
    }
}

/// Admission + plausibility gate for streaming ASR decode windows.
#[derive(Debug, Clone, Copy)]
pub struct SilenceGateConfig {
    pub vad: VadConfig,
    pub min_voiced_frames_per_chunk: usize,
    pub pre_roll_duration: f64,
    pub min_implausible_word_count: usize,
    pub min_voiced_for_long_tail: f64,
}

impl Default for SilenceGateConfig {
    fn default() -> Self {
        Self {
            vad: VadConfig::default(),
            min_voiced_frames_per_chunk: 5,
            pre_roll_duration: 0.3,
            min_implausible_word_count: 3,
            min_voiced_for_long_tail: 0.3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SilenceDecision {
    Admit(Vec<f32>),
    Reject,
}

#[derive(Debug, Clone)]
pub struct SilenceGate {
    config: SilenceGateConfig,
    pre_roll: Vec<f32>,
    carried_voiced_frames: usize,
    window_voiced_duration: f64,
}

impl Default for SilenceGate {
    fn default() -> Self {
        Self::new(SilenceGateConfig::default())
    }
}

impl SilenceGate {
    pub fn new(config: SilenceGateConfig) -> Self {
        Self {
            config,
            pre_roll: Vec::new(),
            carried_voiced_frames: 0,
            window_voiced_duration: 0.0,
        }
    }

    pub fn voiced_frame_count(samples: &[f32], config: &VadConfig) -> usize {
        let frame_length = config.frame_length;
        let mut count = 0usize;
        let mut offset = 0usize;
        while offset + frame_length <= samples.len() {
            let sum = samples[offset..offset + frame_length]
                .iter()
                .map(|sample| sample * sample)
                .sum::<f32>();
            if (sum / frame_length as f32).sqrt() >= config.energy_threshold {
                count += 1;
            }
            offset += frame_length;
        }
        count
    }

    pub fn evaluate(&mut self, chunk: &[f32]) -> SilenceDecision {
        let voiced_frames = Self::voiced_frame_count(chunk, &self.config.vad);
        let admits_alone = voiced_frames >= self.config.min_voiced_frames_per_chunk;
        let admits_with_carry = voiced_frames >= 1
            && voiced_frames + self.carried_voiced_frames
                >= self.config.min_voiced_frames_per_chunk;
        if !admits_alone && !admits_with_carry {
            self.carried_voiced_frames = voiced_frames;
            let pre_roll_samples =
                (self.config.pre_roll_duration * TARGET_SAMPLE_RATE as f64) as usize;
            let start = chunk.len().saturating_sub(pre_roll_samples);
            self.pre_roll = chunk[start..].to_vec();
            return SilenceDecision::Reject;
        }
        let counted_frames = if admits_alone {
            voiced_frames
        } else {
            voiced_frames + self.carried_voiced_frames
        };
        self.window_voiced_duration +=
            (counted_frames * self.config.vad.frame_length) as f64 / TARGET_SAMPLE_RATE as f64;
        self.carried_voiced_frames = 0;
        let mut admitted = std::mem::take(&mut self.pre_roll);
        admitted.extend_from_slice(chunk);
        SilenceDecision::Admit(admitted)
    }

    pub fn is_plausible_tail(&self, word_count: usize) -> bool {
        word_count < self.config.min_implausible_word_count
            || self.window_voiced_duration >= self.config.min_voiced_for_long_tail
    }

    pub fn clear_window(&mut self) {
        self.window_voiced_duration = 0.0;
    }

    pub fn reset(&mut self) {
        self.window_voiced_duration = 0.0;
        self.pre_roll.clear();
        self.carried_voiced_frames = 0;
    }
}

/// Bounded 10-second ring buffer. Overflow is counted, never silent.
pub struct AudioRingBuffer {
    producer: ringbuf::HeapProd<f32>,
    consumer: ringbuf::HeapCons<f32>,
    overrun_events: u64,
    dropped_samples: u64,
}

impl std::fmt::Debug for AudioRingBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioRingBuffer")
            .field("len", &self.consumer.occupied_len())
            .field("overrun_events", &self.overrun_events)
            .field("dropped_samples", &self.dropped_samples)
            .finish()
    }
}

impl AudioRingBuffer {
    pub fn new(capacity: usize) -> Self {
        let (producer, consumer) = HeapRb::<f32>::new(capacity).split();
        Self {
            producer,
            consumer,
            overrun_events: 0,
            dropped_samples: 0,
        }
    }

    pub fn default_ten_seconds() -> Self {
        Self::new(RING_SAMPLES)
    }

    /// Push samples; returns the number of samples dropped because the ring was full.
    pub fn push(&mut self, samples: &[f32]) -> usize {
        let capacity = self.producer.capacity().get();
        let free = capacity - self.producer.occupied_len();
        let dropped = samples.len().saturating_sub(free);
        for sample in samples.iter().take(free) {
            let _ = self.producer.try_push(*sample);
        }
        if dropped > 0 {
            self.overrun_events += 1;
            self.dropped_samples += dropped as u64;
        }
        dropped
    }

    pub fn pop(&mut self, output: &mut [f32]) -> usize {
        let mut count = 0usize;
        for slot in output.iter_mut() {
            match self.consumer.try_pop() {
                Some(sample) => {
                    *slot = sample;
                    count += 1;
                }
                None => break,
            }
        }
        count
    }

    pub fn len(&self) -> usize {
        self.consumer.occupied_len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn overrun_events(&self) -> u64 {
        self.overrun_events
    }

    pub fn dropped_samples(&self) -> u64 {
        self.dropped_samples
    }

    pub fn dropped_ms(&self) -> u64 {
        (self.dropped_samples as f64 / TARGET_SAMPLE_RATE as f64 * 1000.0) as u64
    }
}

/// A queue-backed source for live capture workers.
#[derive(Debug, Default)]
pub struct ChannelSource {
    queue: VecDeque<Vec<f32>>,
    name: &'static str,
}

impl ChannelSource {
    pub fn new(name: &'static str) -> Self {
        Self {
            queue: VecDeque::new(),
            name,
        }
    }

    pub fn push(&mut self, buffer: Vec<f32>) {
        self.queue.push_back(buffer);
    }
}

impl AudioSource for ChannelSource {
    fn next_buffer(&mut self) -> Option<Vec<f32>> {
        self.queue.pop_front()
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav_bytes(sample_rate: u32, channels: u16, samples: &[f32]) -> Vec<u8> {
        let data = samples
            .iter()
            .flat_map(|sample| {
                let value = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                value.to_le_bytes()
            })
            .collect::<Vec<_>>();
        let byte_rate = sample_rate * channels as u32 * 2;
        let block_align = channels * 2;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&channels.to_le_bytes());
        out.extend_from_slice(&sample_rate.to_le_bytes());
        out.extend_from_slice(&byte_rate.to_le_bytes());
        out.extend_from_slice(&block_align.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&data);
        out
    }

    #[test]
    fn wav_round_trip_and_stereo_downmix() {
        let mono = wav_bytes(16_000, 1, &[0.0, 0.5, -0.5, 1.0]);
        let decoded = decode_wav(&mono).unwrap();
        assert_eq!(decoded.sample_rate, 16_000);
        assert_eq!(decoded.channels, 1);
        assert!((decoded.samples[1] - 0.5).abs() < 0.001);

        let stereo = wav_bytes(16_000, 2, &[1.0, 0.0, 0.0, 1.0]);
        let decoded = decode_wav(&stereo).unwrap();
        assert_eq!(decoded.samples.len(), 2);
        assert!((decoded.samples[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn resample_48k_to_16k_shrinks_by_three() {
        let input = vec![0.0f32; 4800];
        let output = resample_linear(&input, 48_000, 16_000);
        assert_eq!(output.len(), 1600);
        assert_eq!(resample_linear(&input, 16_000, 16_000).len(), 4800);
    }

    #[test]
    fn chunker_emits_fixed_size_and_pads_flush() {
        let mut chunker = AudioChunker::new();
        assert!(chunker.append(&vec![0.1; 17_000]).is_empty());
        let chunks = chunker.append(&vec![0.1; 1_000]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), CHUNK_SAMPLES);
        chunker.append(&vec![0.1; 100]);
        let flush = chunker.flush().unwrap();
        assert_eq!(flush.len(), CHUNK_SAMPLES);
        assert_eq!(flush[500], 0.0);
    }

    #[test]
    fn vad_detects_speech_then_trailing_silence() {
        let mut vad = VoiceActivityDetector::default();
        let speech = vec![0.3f32; 16_000];
        let status = vad.process(&speech);
        assert!(status.is_speech_active);
        assert_eq!(status.trailing_silence, 0.0);
        let silence = vec![0.0f32; 16_000];
        let status = vad.process(&silence);
        assert!(!status.is_speech_active);
        assert!((status.trailing_silence - 1.0).abs() < 0.01);
        assert!((vad.total_voiced_duration() - 1.0).abs() < 0.01);
    }

    #[test]
    fn ring_buffer_counts_overflow() {
        let mut ring = AudioRingBuffer::new(10);
        assert_eq!(ring.push(&[0.0; 8]), 0);
        assert_eq!(ring.push(&[0.0; 5]), 3);
        assert_eq!(ring.len(), 10);
        assert_eq!(ring.overrun_events(), 1);
        assert_eq!(ring.dropped_samples(), 3);
        let mut out = [0.0f32; 4];
        assert_eq!(ring.pop(&mut out), 4);
    }

    #[test]
    fn synthetic_source_emits_speech_and_silence() {
        let mut source = SyntheticSource::new(
            vec![DemoPhase::Speech(0.1), DemoPhase::Silence(0.1)],
            1,
            false,
        );
        let first = source.next_buffer().unwrap();
        assert_eq!(first.len(), 1600);
        assert!(first.iter().any(|sample| sample.abs() > 0.1));
        let second = source.next_buffer().unwrap();
        assert!(second.iter().all(|sample| *sample == 0.0));
        assert!(source.next_buffer().is_none());
        assert!(source.duration_seconds().unwrap() > 0.0);
    }

    #[test]
    fn file_source_replays_and_ends() {
        let bytes = wav_bytes(16_000, 1, &vec![0.1f32; 3200]);
        let mut source = FileSource::from_bytes(&bytes, false).unwrap();
        assert!(source.duration_seconds() >= 0.19);
        assert_eq!(source.next_buffer().unwrap().len(), 1600);
        assert_eq!(source.next_buffer().unwrap().len(), 1600);
        assert!(source.next_buffer().is_none());
        assert_eq!(source.name(), "file");
    }

    #[test]
    fn silence_gate_rejects_pure_silence_and_admits_speech() {
        let mut gate = SilenceGate::default();
        assert_eq!(
            gate.evaluate(&vec![0.0; CHUNK_SAMPLES]),
            SilenceDecision::Reject
        );
        let decision = gate.evaluate(&vec![0.3; CHUNK_SAMPLES]);
        assert!(matches!(decision, SilenceDecision::Admit(_)));
        assert!(gate.is_plausible_tail(2));
        gate.clear_window();
        gate.reset();
    }

    #[test]
    fn invalid_wav_is_an_error() {
        assert!(decode_wav(b"not a wav").is_err());
    }
}
