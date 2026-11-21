//! Per-segment stage timestamps and content-free diagnostics export.
//!
//! Ported from `Sources/LiveLinguistKit/Metrics/LatencyMetrics.swift`.

use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    AudioEnd,
    PartialVisible,
    Finalized,
    SimplifyStart,
    FirstToken,
    SimplifiedVisible,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::AudioEnd => "audio_end",
            Stage::PartialVisible => "partial_visible",
            Stage::Finalized => "finalized",
            Stage::SimplifyStart => "simplify_start",
            Stage::FirstToken => "first_token",
            Stage::SimplifiedVisible => "simplified_visible",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LatencyReport {
    pub tier1_p50: Option<f64>,
    pub tier1_p95: Option<f64>,
    pub tier2_p50: Option<f64>,
    pub tier2_p95: Option<f64>,
    pub degradation_event_count: usize,
}

#[derive(Debug, Clone)]
struct DegradationEvent {
    kind: String,
    at: f64,
    detail: String,
}

#[derive(Debug, Default)]
struct Inner {
    segment_stamps: BTreeMap<String, BTreeMap<Stage, f64>>,
    segment_order: Vec<String>,
    degradation_events: Vec<DegradationEvent>,
}

/// Built into every session; the JSON export is timings-only, no content.
#[derive(Debug, Default)]
pub struct LatencyMetrics {
    inner: Mutex<Inner>,
}

impl LatencyMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Record a stage stamp. The first stamp for a stage wins ("visible" means
    /// first appearance).
    pub fn record(&self, segment: &str, stage: Stage, at: f64) {
        let mut inner = self.lock();
        if !inner.segment_stamps.contains_key(segment) {
            inner.segment_order.push(segment.to_string());
        }
        inner
            .segment_stamps
            .entry(segment.to_string())
            .or_default()
            .entry(stage)
            .or_insert(at);
    }

    pub fn stamps(&self, segment: &str) -> BTreeMap<Stage, f64> {
        self.lock()
            .segment_stamps
            .get(segment)
            .cloned()
            .unwrap_or_default()
    }

    pub fn record_degradation(&self, kind: &str, at: f64, detail: &str) {
        self.lock().degradation_events.push(DegradationEvent {
            kind: kind.to_string(),
            at,
            detail: detail.to_string(),
        });
    }

    pub fn report(&self) -> LatencyReport {
        let inner = self.lock();
        let tier1 = deltas(&inner, Stage::AudioEnd, Stage::PartialVisible);
        let tier2 = deltas(&inner, Stage::SimplifyStart, Stage::SimplifiedVisible);
        LatencyReport {
            tier1_p50: percentile(&tier1, 50.0),
            tier1_p95: percentile(&tier1, 95.0),
            tier2_p50: percentile(&tier2, 50.0),
            tier2_p95: percentile(&tier2, 95.0),
            degradation_event_count: inner.degradation_events.len(),
        }
    }

    /// Timings-only JSON export. Deliberately contains no transcript content.
    pub fn export_json(&self) -> String {
        let inner = self.lock();
        let segments = inner
            .segment_order
            .iter()
            .filter_map(|id| {
                inner.segment_stamps.get(id).map(|stamps| {
                    let mut object = Map::new();
                    for (stage, value) in stamps {
                        object.insert(stage.as_str().to_string(), json!(value));
                    }
                    Value::Object(object)
                })
            })
            .collect::<Vec<_>>();
        let events = inner
            .degradation_events
            .iter()
            .map(|event| {
                json!({
                    "kind": event.kind,
                    "at": event.at,
                    "detail": event.detail,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "segments": segments,
            "degradationEvents": events,
        })
        .to_string()
    }
}

fn deltas(inner: &Inner, start: Stage, end: Stage) -> Vec<f64> {
    inner
        .segment_order
        .iter()
        .filter_map(|id| {
            let stamps = inner.segment_stamps.get(id)?;
            let start_time = stamps.get(&start)?;
            let end_time = stamps.get(&end)?;
            Some(end_time - start_time)
        })
        .collect()
}

/// Nearest-rank percentile.
pub fn percentile(values: &[f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((percentile / 100.0 * sorted.len() as f64).ceil() as usize).max(1);
    let index = (rank - 1).min(sorted.len() - 1);
    sorted.get(index).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_nearest_rank() {
        let values = (1..=10).map(f64::from).collect::<Vec<_>>();
        assert_eq!(percentile(&values, 50.0), Some(5.0));
        assert_eq!(percentile(&values, 95.0), Some(10.0));
        assert_eq!(percentile(&[], 50.0), None);
        assert_eq!(percentile(&[42.0], 95.0), Some(42.0));
    }

    #[test]
    fn tier2_latency_is_simplify_start_to_glass() {
        let metrics = LatencyMetrics::new();
        metrics.record("a", Stage::AudioEnd, 10.0);
        metrics.record("a", Stage::Finalized, 10.4);
        metrics.record("a", Stage::SimplifyStart, 10.5);
        metrics.record("a", Stage::SimplifiedVisible, 12.1);
        let report = metrics.report();
        assert!((report.tier2_p50.unwrap() - 1.6).abs() < 0.0001);
    }

    #[test]
    fn tier1_latency_is_word_to_glass() {
        let metrics = LatencyMetrics::new();
        metrics.record("a", Stage::AudioEnd, 5.0);
        metrics.record("a", Stage::PartialVisible, 6.3);
        let report = metrics.report();
        assert!((report.tier1_p50.unwrap() - 1.3).abs() < 0.0001);
    }

    #[test]
    fn degradation_events_are_counted() {
        let metrics = LatencyMetrics::new();
        metrics.record_degradation("batch_queue", 100.0, "queue depth 3");
        metrics.record_degradation("skip_simplification", 200.0, "thermal serious");
        assert_eq!(metrics.report().degradation_event_count, 2);
    }

    #[test]
    fn json_export_contains_timings_but_no_content() {
        let metrics = LatencyMetrics::new();
        metrics.record("a", Stage::AudioEnd, 1.0);
        metrics.record("a", Stage::SimplifiedVisible, 3.0);
        metrics.record_degradation("skip_simplification", 5.0, "thermal");
        let json = metrics.export_json();
        assert!(json.contains("\"segments\""));
        assert!(json.contains("\"degradationEvents\""));
        assert!(!json.contains("\"text\""));
        assert!(!json.contains("verbatim"));
    }

    #[test]
    fn incomplete_segments_are_excluded() {
        let metrics = LatencyMetrics::new();
        metrics.record("a", Stage::AudioEnd, 1.0);
        assert!(metrics.report().tier2_p50.is_none());
    }

    #[test]
    fn first_stamp_wins_and_unknown_is_empty() {
        let metrics = LatencyMetrics::new();
        metrics.record("a", Stage::AudioEnd, 1.0);
        metrics.record("a", Stage::AudioEnd, 9.0);
        assert_eq!(metrics.stamps("a").get(&Stage::AudioEnd), Some(&1.0));
        assert!(metrics.stamps("missing").is_empty());
    }
}
