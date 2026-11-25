//! `--scenario asr-wer`: score the streaming ASR on the FLEURS manifest (T31).
//!
//! For each clip of a locale: decode to 16 kHz mono f32, feed 17,600-sample
//! chunks to a fresh utterance stream, take the final hypothesis, and score
//! it (CER for ja/ko/zh, WER otherwise) as a corpus rate. Measures RTF
//! (processing time / audio time), p95 per-chunk latency, and endpoints.
//! `wer.json` is content-free; hypotheses go only to `--out/hyp.txt`.

use crate::wer::{CorpusScore, Metric};
use crate::EvalError;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const CHUNK_SAMPLES: usize = 17_600;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Manifest {
    pub dataset: String,
    pub revision: String,
    pub locales: Vec<ManifestLocale>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ManifestLocale {
    pub locale: String,
    pub fleurs: String,
    pub status: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub clips: Vec<ManifestClip>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ManifestClip {
    pub id: String,
    pub file: String,
    pub seconds: f64,
    pub sha256: String,
    pub reference: String,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self, EvalError> {
        let text = std::fs::read_to_string(path).map_err(|error| {
            EvalError::Argument(format!("manifest {}: {error}", path.display()))
        })?;
        serde_json::from_str(&text)
            .map_err(|error| EvalError::Argument(format!("manifest {}: {error}", path.display())))
    }
}

/// `--scenario asr-wer` arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct AsrWerArgs {
    pub locale: Option<String>,
    pub manifest: Option<PathBuf>,
    pub cache: Option<PathBuf>,
    pub all: bool,
    /// Recognizer CPU threads (T32 compares 2 vs 3).
    pub asr_threads: i32,
    /// Streaming input chunk in milliseconds (autoresearch latency candidates).
    pub chunk_ms: usize,
}

impl Default for AsrWerArgs {
    fn default() -> Self {
        Self {
            locale: None,
            manifest: None,
            cache: None,
            all: false,
            asr_threads: 2,
            chunk_ms: 1100,
        }
    }
}

/// What transcribing one clip produced.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClipRun {
    pub hypothesis: String,
    pub chunk_ms: Vec<f64>,
    pub process_ms: f64,
    pub endpoints: usize,
}

/// Per-locale result (content-free).
#[derive(Debug, Clone, PartialEq)]
pub struct LocaleScore {
    pub score: CorpusScore,
    pub audio_s: f64,
    pub process_ms: f64,
    pub chunk_ms: Vec<f64>,
    pub endpoints: usize,
}

impl LocaleScore {
    pub fn rtf(&self) -> f64 {
        if self.audio_s > 0.0 {
            self.process_ms / 1000.0 / self.audio_s
        } else {
            0.0
        }
    }
}

/// Score clips with any transcriber (the real engine, or a fake in tests).
/// `hypotheses` receives `(clip id, hypothesis)` for the local hyp file.
pub fn score_clips(
    metric: Metric,
    clips: &[(ManifestClip, Vec<f32>)],
    mut transcribe: impl FnMut(&[f32]) -> Result<ClipRun, EvalError>,
    mut hypotheses: impl FnMut(&str, &str),
) -> Result<LocaleScore, EvalError> {
    let mut result = LocaleScore {
        score: CorpusScore::default(),
        audio_s: 0.0,
        process_ms: 0.0,
        chunk_ms: Vec::new(),
        endpoints: 0,
    };
    for (clip, samples) in clips {
        let run = transcribe(samples)?;
        result
            .score
            .add(metric.score(&clip.reference, &run.hypothesis));
        result.audio_s += samples.len() as f64 / 16_000.0;
        result.process_ms += run.process_ms;
        result.chunk_ms.extend(run.chunk_ms);
        result.endpoints += run.endpoints;
        hypotheses(&clip.id, &run.hypothesis);
    }
    Ok(result)
}

/// Read and verify one clip, returning 16 kHz mono samples.
pub fn load_clip(cache: &Path, locale: &str, clip: &ManifestClip) -> Result<Vec<f32>, EvalError> {
    let path = cache.join(locale).join(&clip.file);
    let bytes = std::fs::read(&path).map_err(|error| {
        EvalError::Audio(format!(
            "{}: {error} (run scripts/asr-eval/fetch_fleurs.py first)",
            path.display()
        ))
    })?;
    let digest = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(&bytes))
    };
    if digest != clip.sha256 {
        return Err(EvalError::Audio(format!("{}: SHA-256 mismatch", clip.file)));
    }
    let wav = ll_audio::decode_wav(&bytes).map_err(|error| EvalError::Audio(error.to_string()))?;
    Ok(wav.mono_16k())
}

/// Run the scenario. Returns the content-free summary (one locale, or the
/// matrix with `--all`).
pub fn run(
    out: &Path,
    language_locale: &str,
    args: &AsrWerArgs,
    mut transcriber: impl FnMut() -> Result<
        Box<dyn FnMut(&[f32]) -> Result<ClipRun, EvalError>>,
        EvalError,
    >,
) -> Result<Value, EvalError> {
    let manifest_path = args
        .manifest
        .clone()
        .unwrap_or_else(|| PathBuf::from("tests/asr-eval/manifest.json"));
    let cache = args
        .cache
        .clone()
        .ok_or_else(|| EvalError::Argument("asr-wer needs --cache <dir>".into()))?;
    let manifest = Manifest::load(&manifest_path)?;
    let wanted = args
        .locale
        .clone()
        .unwrap_or_else(|| language_locale.to_string());
    let entries: Vec<&ManifestLocale> = manifest
        .locales
        .iter()
        .filter(|entry| args.all || entry.locale == wanted)
        .collect();
    if entries.is_empty() {
        return Err(EvalError::Argument(format!(
            "locale {wanted} is not in the manifest"
        )));
    }
    std::fs::create_dir_all(out)?;
    let mut rows = Vec::new();
    let mut hyp_text = String::new();
    for entry in entries {
        let dir = if args.all {
            out.join(&entry.locale)
        } else {
            out.to_path_buf()
        };
        std::fs::create_dir_all(&dir)?;
        let metric = Metric::for_locale(&entry.locale);
        if entry.status != "fixture" || entry.clips.is_empty() {
            let row = json!({
                "locale": entry.locale, "fleurs_code": entry.fleurs, "status": entry.status,
                "metric": metric.as_str(), "clips": 0,
            });
            write(&dir.join("wer.json"), &row)?;
            rows.push(row);
            continue;
        }
        let mut clips = Vec::with_capacity(entry.clips.len());
        for clip in &entry.clips {
            clips.push((clip.clone(), load_clip(&cache, &entry.locale, clip)?));
        }
        let mut transcribe = transcriber()?;
        let mut local = String::new();
        let score = score_clips(metric, &clips, &mut transcribe, |id, hypothesis| {
            local.push_str(&format!("{id}\t{hypothesis}\n"));
        })?;
        hyp_text.push_str(&local);
        std::fs::write(dir.join("hyp.txt"), local)?;
        let row = json!({
            "locale": entry.locale,
            "fleurs_code": entry.fleurs,
            "note": entry.note,
            "clips": entry.clips.len(),
            "audio_s": (score.audio_s * 10.0).round() / 10.0,
            "metric": metric.as_str(),
            "score": (score.score.rate() * 10_000.0).round() / 10_000.0,
            "edits": score.score.edits,
            "reference_units": score.score.reference_len,
            "rtf": (score.rtf() * 1000.0).round() / 1000.0,
            "p95_chunk_ms": crate::wer_percentile(&score.chunk_ms, 95.0),
            "endpoints": score.endpoints,
            "asr_threads": args.asr_threads,
            "asr_chunk_ms": args.chunk_ms,
            "model_sha256": crate::asr_encoder_sha256(),
            "host": if cfg!(windows) { "windows host" } else { "Linux dev approximation, not the Windows tier C gate" },
            "contentFree": true,
        });
        write(&dir.join("wer.json"), &row)?;
        rows.push(row);
    }
    if args.all {
        let matrix =
            json!({ "dataset": manifest.dataset, "revision": manifest.revision, "rows": rows });
        write(&out.join("matrix.json"), &matrix)?;
        Ok(matrix)
    } else {
        Ok(rows.into_iter().next().unwrap_or(Value::Null))
    }
}

fn write(path: &Path, value: &Value) -> Result<(), EvalError> {
    let json =
        serde_json::to_string_pretty(value).map_err(|error| EvalError::Io(error.to_string()))?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Transcribe one clip with the sherpa engine: commit each utterance at its
/// endpoint, then the stream's final text.
#[cfg(feature = "sherpa")]
pub fn sherpa_transcribe(
    asr: &mut ll_asr::SherpaAsr,
    samples: &[f32],
    chunk_ms: usize,
) -> Result<ClipRun, EvalError> {
    use ll_asr::AsrEngine;
    let chunk_samples = (chunk_ms * 16).max(1);
    let mut run = ClipRun::default();
    let mut committed: Vec<String> = Vec::new();
    let mut current = String::new();
    for (index, chunk) in samples.chunks(chunk_samples).enumerate() {
        let started = std::time::Instant::now();
        let event = asr
            .process(chunk, (index + 1) as f64 * chunk_samples as f64 / 16_000.0)
            .map_err(|error| EvalError::Audio(error.to_string()))?;
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        run.chunk_ms.push(elapsed);
        run.process_ms += elapsed;
        if let Some(event) = event {
            current = event.hypothesis;
        }
        if asr.take_endpoint() == Some(true) {
            run.endpoints += 1;
            if !current.is_empty() {
                committed.push(std::mem::take(&mut current));
            }
        }
    }
    let started = std::time::Instant::now();
    let tail = asr.finish();
    run.process_ms += started.elapsed().as_secs_f64() * 1000.0;
    if !tail.is_empty() {
        committed.push(tail);
    } else if !current.is_empty() {
        committed.push(current);
    }
    run.hypothesis = committed.join(" ");
    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(id: &str, reference: &str) -> ManifestClip {
        ManifestClip {
            id: id.into(),
            file: format!("{id}.wav"),
            seconds: 1.0,
            sha256: String::new(),
            reference: reference.into(),
        }
    }

    #[test]
    fn echoing_the_reference_scores_zero() {
        let clips = vec![
            (clip("a", "the cat sat on the mat"), vec![0.0; 16_000]),
            (clip("b", "a second clip"), vec![0.0; 32_000]),
        ];
        let mut index = 0;
        let references: Vec<String> = clips
            .iter()
            .map(|(clip, _)| clip.reference.clone())
            .collect();
        let score = score_clips(
            Metric::Wer,
            &clips,
            |_samples| {
                let run = ClipRun {
                    hypothesis: references[index].clone(),
                    chunk_ms: vec![10.0],
                    process_ms: 300.0,
                    endpoints: 1,
                };
                index += 1;
                Ok(run)
            },
            |_, _| {},
        )
        .unwrap();
        assert_eq!(score.score.edits, 0);
        assert_eq!(score.score.reference_len, 9);
        assert_eq!(score.score.rate(), 0.0);
        assert!((score.audio_s - 3.0).abs() < 1e-9);
        assert!((score.rtf() - 0.2).abs() < 1e-9);
        assert_eq!(score.endpoints, 2);
    }

    #[test]
    fn manifest_parses() {
        let json = r#"{"dataset":"google/fleurs","revision":"abc","locales":[
            {"locale":"en-US","fleurs":"en_us","status":"fixture","note":null,
             "clips":[{"id":"1","file":"x.wav","seconds":1.5,"sha256":"00","reference":"hi"}]},
            {"locale":"xx-XX","fleurs":"xx","status":"no_fixture","clips":[]}]}"#;
        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.locales.len(), 2);
        assert_eq!(manifest.locales[0].clips[0].seconds, 1.5);
        assert_eq!(manifest.locales[1].status, "no_fixture");
    }
}
