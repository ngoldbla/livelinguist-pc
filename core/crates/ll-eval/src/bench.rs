//! Eval-only benchmark scenarios (T21 click-bench, T22 simplify-matrix).
//!
//! Summaries are content-free: counts, timings, booleans. Model outputs go
//! only to `--out/outputs/` for local inspection (never committed, never
//! logged). Timings measured on the Linux dev container are labelled as an
//! approximation, not the Windows tier C gate.

use crate::{EvalError, EvalOptions};
use ll_core::pack::FrameworkPack;
use serde_json::{json, Value};

/// Benchmark scenarios that are not runtime `Scenario`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bench {
    /// Click each finalized demo segment in turn: click -> simplified ms.
    ClickBench { clicks: usize },
    /// 29 packs x 2 demo sentences, each simplified twice.
    SimplifyMatrix,
}

impl Bench {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "click-bench" => Some(Bench::ClickBench { clicks: 50 }),
            "simplify-matrix" => Some(Bench::SimplifyMatrix),
            _ => None,
        }
    }
}

/// Label carried by every timing this module writes.
pub fn host_label() -> &'static str {
    if cfg!(windows) {
        "windows host"
    } else {
        "Linux dev approximation, not the Windows tier C gate"
    }
}

/// Peak resident set size of this process in MB, where the OS reports it.
pub fn peak_rss_mb() -> Option<f64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024.0)
}

/// p-th percentile (nearest rank) of `values` in ms.
pub fn percentile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = ((p / 100.0) * sorted.len() as f64).ceil().max(1.0) as usize;
    Some(sorted[rank.min(sorted.len()) - 1])
}

pub fn run(options: &EvalOptions, bench: Bench) -> Result<Value, EvalError> {
    std::fs::create_dir_all(&options.out)?;
    let summary = match bench {
        Bench::ClickBench { clicks } => click_bench(options, clicks)?,
        Bench::SimplifyMatrix => simplify_matrix(options)?,
    };
    let name = match bench {
        Bench::ClickBench { .. } => "bench.json",
        Bench::SimplifyMatrix => "matrix.json",
    };
    let json =
        serde_json::to_string_pretty(&summary).map_err(|error| EvalError::Io(error.to_string()))?;
    std::fs::write(options.out.join(name), json)?;
    Ok(summary)
}

fn click_bench(options: &EvalOptions, clicks: usize) -> Result<Value, EvalError> {
    let pack = FrameworkPack::for_language(options.language)
        .map_err(|error| EvalError::Argument(error.to_string()))?;
    let conversation = ll_core::demo::DemoConversation::from_pack(&pack, 4);
    if conversation.lines.is_empty() {
        return Err(EvalError::Argument("pack has no demo sentences".into()));
    }
    let mut pipeline = crate::bench_pipeline(options, &pack)?;
    pipeline
        .start()
        .map_err(|error| EvalError::Audio(error.to_string()))?;
    // Finalize `clicks` segments (the demo lines, repeated as new segments),
    // then click each once through the UI's path (`simplify_now`).
    let mut ids = Vec::with_capacity(clicks);
    for index in 0..clicks {
        let line = &conversation.lines[index % conversation.lines.len()];
        let start = index as f64 * 4.0;
        let segment =
            ll_core::session::FinalizedSegment::new(line.text.clone(), start, start + 3.0);
        ids.push(segment.id.clone());
        pipeline.push_finalized(segment);
    }
    let _ = pipeline.take_events();
    let mut timings = Vec::with_capacity(clicks);
    let mut simplified = 0usize;
    let mut verbatim = 0usize;
    for id in &ids {
        let started = std::time::Instant::now();
        let result = pipeline
            .simplify_now(id)
            .map_err(|error| EvalError::Audio(error.to_string()))?;
        timings.push(started.elapsed().as_secs_f64() * 1000.0);
        if result.outcome.display_text().is_some() {
            simplified += 1;
        } else {
            verbatim += 1;
        }
        let _ = pipeline.take_events();
    }
    Ok(json!({
        "scenario": "click-bench",
        "language": options.language.locale(),
        "llm": if options.llama { "llama.cpp-cpu" } else { "demo" },
        "threads": options.threads,
        "clicks": clicks,
        "simplified": simplified,
        "verbatim": verbatim,
        "click_p50_ms": percentile(&timings, 50.0),
        "click_p95_ms": percentile(&timings, 95.0),
        "click_max_ms": percentile(&timings, 100.0),
        "peak_rss_mb": peak_rss_mb(),
        "host": host_label(),
        "contentFree": true,
    }))
}

#[cfg(feature = "llama")]
fn simplify_matrix(options: &EvalOptions) -> Result<Value, EvalError> {
    use ll_core::decoding::strip_think_block;
    use ll_core::language::Language;
    use ll_core::language_guard::LanguageGuard;
    use ll_core::prompt::render_easylanguage_prompt;
    use ll_core::validation::{Severity, ValidatorSuite};
    use ll_llm::llama::MAX_NEW_TOKENS;

    let path = crate::llm_model_path();
    let mut backend = ll_llm::LlamaBackend::load(&path, options.threads)
        .map_err(|error| EvalError::ScenarioUnavailable("simplify-matrix".into(), error.message))?;
    let outputs_dir = options.out.join("outputs");
    std::fs::create_dir_all(&outputs_dir)?;
    let mut rows = Vec::new();
    let mut all_timings = Vec::new();
    for language in Language::ALL {
        let pack = FrameworkPack::for_language(language)
            .map_err(|error| EvalError::Argument(error.to_string()))?;
        let conversation = ll_core::demo::DemoConversation::from_pack(&pack, 4);
        let sentences: Vec<String> = conversation
            .lines
            .iter()
            .take(2)
            .map(|line| line.text.clone())
            .collect();
        let base = pack.base_code();
        let guard = LanguageGuard::new(&base, pack.stopwords.clone().unwrap_or_default());
        let validators = ValidatorSuite::new(&pack.lean_variant());
        let mut timings = Vec::new();
        let (mut same_language, mut deterministic, mut flagged, mut verbatim) = (0, 0, 0, 0);
        let (mut same_language_no_switch, mut identical_no_switch) = (0, 0);
        let mut ran = !sentences.is_empty();
        let mut local = String::new();
        for sentence in &sentences {
            let generate = |backend: &mut ll_llm::LlamaBackend| {
                let started = std::time::Instant::now();
                let output = backend.simplify_sentence(&pack, sentence);
                (output, started.elapsed().as_secs_f64() * 1000.0)
            };
            let (first, first_ms) = generate(&mut backend);
            let (second, second_ms) = generate(&mut backend);
            timings.extend([first_ms, second_ms]);
            let (Ok(first), Ok(second)) = (first, second) else {
                ran = false;
                verbatim += 1;
                continue;
            };
            if first == second {
                deterministic += 1;
            }
            if first.trim().is_empty() {
                verbatim += 1;
            } else {
                if guard.validate(&first).is_empty() {
                    same_language += 1;
                }
                let hard = validators
                    .validate(&first, sentence, &[])
                    .into_iter()
                    .any(|violation| violation.severity == Severity::Hard);
                if hard {
                    flagged += 1;
                }
            }
            // A/B for the open prompt-contract item: the same prompt without
            // the ` /no_think` soft switch.
            let prompt = render_easylanguage_prompt(&pack, sentence).replacen(
                " /no_think<|im_end|>",
                "<|im_end|>",
                1,
            );
            let budget = MAX_NEW_TOKENS.min(3 * backend.tokenize_len(sentence) + 32);
            if let Ok(raw) = backend.generate(&prompt, budget) {
                let without = strip_think_block(&raw).trim().to_string();
                if guard.validate(&without).is_empty() && !without.is_empty() {
                    same_language_no_switch += 1;
                }
                if without == first {
                    identical_no_switch += 1;
                }
                local.push_str(&format!(
                    "SOURCE: {sentence}\nOUTPUT: {first}\nNO_SWITCH: {without}\n\n"
                ));
            }
        }
        std::fs::write(
            outputs_dir.join(format!("{}.txt", language.locale())),
            local,
        )?;
        all_timings.extend(timings.iter().copied());
        let sentences_n = sentences.len();
        let pass = ran && deterministic == sentences_n && same_language == sentences_n;
        rows.push(json!({
            "locale": language.locale(),
            "sentences": sentences_n,
            "ran": ran,
            "same_language": same_language,
            "deterministic": deterministic == sentences_n,
            "flagged": flagged,
            "verbatim": verbatim,
            "p95_ms": percentile(&timings, 95.0),
            "same_language_without_no_think": same_language_no_switch,
            "identical_without_no_think": identical_no_switch,
            "pass": pass,
        }));
    }
    Ok(json!({
        "scenario": "simplify-matrix",
        "threads": options.threads,
        "languages": rows.len(),
        "passed": rows.iter().filter(|row| row["pass"] == true).count(),
        "p95_ms": percentile(&all_timings, 95.0),
        "peak_rss_mb": peak_rss_mb(),
        "host": host_label(),
        "rows": rows,
        "contentFree": true,
    }))
}

#[cfg(not(feature = "llama"))]
fn simplify_matrix(_options: &EvalOptions) -> Result<Value, EvalError> {
    Err(EvalError::Argument(
        "simplify-matrix needs the real model: rebuild with --features llama".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_uses_nearest_rank() {
        let values: Vec<f64> = (1..=20).map(f64::from).collect();
        assert_eq!(percentile(&values, 50.0), Some(10.0));
        assert_eq!(percentile(&values, 95.0), Some(19.0));
        assert_eq!(percentile(&values, 100.0), Some(20.0));
        assert_eq!(percentile(&[], 95.0), None);
    }

    #[test]
    fn click_bench_parses_clicks_and_runs_on_the_demo_backend() {
        let out = std::env::temp_dir().join(format!("ll-eval-click-{}", std::process::id()));
        let options = EvalOptions::parse(&[
            "--scenario".into(),
            "click-bench".into(),
            "--clicks".into(),
            "12".into(),
            "--language".into(),
            "en-US".into(),
            "--threads".into(),
            "2".into(),
            "--out".into(),
            out.display().to_string(),
        ])
        .unwrap();
        assert_eq!(options.bench, Some(Bench::ClickBench { clicks: 12 }));
        assert_eq!(options.threads, 2);
        let summary = run(&options, options.bench.unwrap()).unwrap();
        assert_eq!(summary["clicks"], 12);
        assert_eq!(summary["threads"], 2);
        assert!(summary["click_p95_ms"].is_number());
        let text = std::fs::read_to_string(out.join("bench.json")).unwrap();
        assert!(text.contains("\"contentFree\": true"));
        let _ = std::fs::remove_dir_all(out);
    }

    #[test]
    fn clicks_needs_click_bench_and_a_positive_number() {
        assert!(EvalOptions::parse(&["--clicks".into(), "0".into()]).is_err());
        assert!(EvalOptions::parse(&[
            "--scenario".into(),
            "mock".into(),
            "--clicks".into(),
            "5".into()
        ])
        .is_err());
    }
}
