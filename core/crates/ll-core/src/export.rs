//! Transcript and diagnostics exporters.
//!
//! Ported from `Sources/LiveLinguistKit/Session/SessionCSVExporter.swift` and
//! `TranscriptExporter.swift`. Exports are explicit user actions only.

use crate::metrics::{LatencyMetrics, Stage};
use crate::session::{SessionStore, SimplificationOutcome, SimplifiedSegment};

const CSV_HEADER: [&str; 14] = [
    "segment_index",
    "audio_start_s",
    "audio_end_s",
    "verbatim",
    "simplified",
    "is_simplified",
    "fallback_reason",
    "is_english_reversion",
    "soft_violations",
    "tier1_latency_ms",
    "tier2_latency_ms",
    "simplify_duration_ms",
    "confidence",
    "flag_reasons",
];

/// RFC-4180 CSV, CRLF, one row per finalized segment.
pub fn session_csv(
    store: &SessionStore,
    metrics: Option<&LatencyMetrics>,
    include_latency_columns: bool,
) -> String {
    let mut rows = vec![CSV_HEADER.join(",")];
    for (index, entry) in store.segments.iter().enumerate() {
        let stamps = metrics.map(|metrics| metrics.stamps(entry.id()));
        let tier1 = if include_latency_columns {
            milliseconds(&stamps, Stage::AudioEnd, Stage::PartialVisible)
        } else {
            String::new()
        };
        let tier2 = if include_latency_columns {
            milliseconds(&stamps, Stage::AudioEnd, Stage::SimplifiedVisible)
        } else {
            String::new()
        };
        let fields = [
            index.to_string(),
            format!("{:.3}", entry.segment.audio_start),
            format!("{:.3}", entry.segment.audio_end),
            entry.segment.text.clone(),
            entry.outcome.display_text().unwrap_or_default().to_string(),
            bool_str(entry.outcome.is_simplified()),
            fallback_reason(&entry.outcome),
            bool_str(store.is_in_reversion_span(&entry.segment)),
            entry
                .soft_violations
                .iter()
                .map(|violation| violation.rule.clone())
                .collect::<Vec<_>>()
                .join(";"),
            tier1,
            tier2,
            milliseconds(&stamps, Stage::SimplifyStart, Stage::SimplifiedVisible),
            confidence(entry),
            entry.confidence.reasons().join(";"),
        ];
        rows.push(fields.map(escape).join(","));
    }
    rows.join("\r\n") + "\r\n"
}

fn bool_str(value: bool) -> String {
    if value { "true" } else { "false" }.to_string()
}

fn confidence(entry: &SimplifiedSegment) -> String {
    if !entry.outcome.is_simplified() {
        return String::new();
    }
    if entry.confidence.is_clean() {
        "clean".to_string()
    } else {
        "low_confidence".to_string()
    }
}

fn fallback_reason(outcome: &SimplificationOutcome) -> String {
    match outcome {
        SimplificationOutcome::Simplified { .. } => String::new(),
        SimplificationOutcome::VerbatimFallback { reason } => reason.as_str().to_string(),
        SimplificationOutcome::Skipped { reason } => reason.as_str().to_string(),
    }
}

fn milliseconds(
    stamps: &Option<std::collections::BTreeMap<Stage, f64>>,
    start: Stage,
    end: Stage,
) -> String {
    let Some(stamps) = stamps else {
        return String::new();
    };
    let (Some(start_time), Some(end_time)) = (stamps.get(&start), stamps.get(&end)) else {
        return String::new();
    };
    format!("{}", ((end_time - start_time) * 1000.0).round() as i64)
}

fn escape(field: String) -> String {
    if field.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field
    }
}

/// Markdown export with metadata and a per-segment table.
pub fn markdown(store: &SessionStore) -> String {
    let mut lines = vec!["# Live Linguist — Transcript".to_string(), String::new()];
    lines.extend(
        metadata_lines(store)
            .into_iter()
            .map(|(key, value)| format!("- **{key}:** {value}")),
    );
    lines.push(String::new());
    lines.push("| Time | Verbatim | Simplified |".to_string());
    lines.push("|---|---|---|".to_string());
    for entry in &store.segments {
        let time = timestamp(entry.segment.audio_start);
        let flag = if store.is_in_reversion_span(&entry.segment) || is_off_target(entry, store) {
            " EN"
        } else {
            ""
        };
        lines.push(format!(
            "| [{time}]{flag} | {} | {} |",
            escape_pipes(&entry.segment.text),
            simplified_column(entry)
        ));
    }
    lines.join("\n") + "\n"
}

/// Plain-text export.
pub fn plain_text(store: &SessionStore) -> String {
    let mut lines = vec!["Live Linguist — Transcript".to_string(), String::new()];
    lines.extend(
        metadata_lines(store)
            .into_iter()
            .map(|(key, value)| format!("{key}: {value}")),
    );
    lines.push(String::new());
    for entry in &store.segments {
        let time = timestamp(entry.segment.audio_start);
        let flag = if store.is_in_reversion_span(&entry.segment) || is_off_target(entry, store) {
            " [EN]"
        } else {
            ""
        };
        lines.push(format!("[{time}]{flag} V: {}", entry.segment.text));
        match &entry.outcome {
            SimplificationOutcome::Simplified { text } => lines.push(format!("        S: {text}")),
            SimplificationOutcome::VerbatimFallback { .. } => {
                lines.push(format!("        S: ≡ {}", entry.segment.text));
            }
            SimplificationOutcome::Skipped { .. } => lines.push("        S: —".to_string()),
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

fn metadata_lines(store: &SessionStore) -> Vec<(String, String)> {
    let mut metadata = Vec::new();
    if let Some(started) = &store.started_at_iso {
        metadata.push(("Date".to_string(), started.clone()));
    }
    if let Some(duration) = store.duration() {
        let minutes = duration as i64 / 60;
        let seconds = duration as i64 % 60;
        metadata.push(("Duration".to_string(), format!("{minutes} min {seconds} s")));
    }
    metadata.push((
        "Language".to_string(),
        store.configuration.language.display_name().to_string(),
    ));
    metadata.push((
        "Framework".to_string(),
        store.configuration.framework().display_name().to_string(),
    ));
    if let Some(percent) = store.percent_target_language() {
        metadata.push((
            "Target language".to_string(),
            format!("{percent}% of detected segments"),
        ));
    }
    metadata
}

fn simplified_column(entry: &SimplifiedSegment) -> String {
    match &entry.outcome {
        SimplificationOutcome::Simplified { text } => escape_pipes(text),
        SimplificationOutcome::VerbatimFallback { .. } => {
            format!("≡ {}", escape_pipes(&entry.segment.text))
        }
        SimplificationOutcome::Skipped { .. } => "—".to_string(),
    }
}

fn is_off_target(entry: &SimplifiedSegment, store: &SessionStore) -> bool {
    entry.segment.detected_language.as_deref() == Some("en")
        && store.configuration.language.base_code() != "en"
}

fn timestamp(seconds: f64) -> String {
    let total = seconds as i64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn escape_pipes(text: &str) -> String {
    text.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use crate::metrics::LatencyMetrics;
    use crate::session::{
        AudioSourceKind, FallbackReason, FinalizedSegment, SessionConfiguration,
        SimplificationConfidence, SimplificationOutcome, SimplifiedSegment, SkipReason,
    };
    use crate::validation::Violation;

    fn entry(
        verbatim: &str,
        outcome: SimplificationOutcome,
        start: f64,
        end: f64,
    ) -> SimplifiedSegment {
        SimplifiedSegment::new(
            FinalizedSegment::with_language(verbatim, Some("fr".into()), start, end),
            outcome,
        )
    }

    fn store() -> SessionStore {
        SessionStore::new(SessionConfiguration::new(
            Language::FrFr,
            AudioSourceKind::Microphone,
        ))
    }

    fn data_rows(csv: &str) -> Vec<Vec<String>> {
        csv.split("\r\n")
            .skip(1)
            .filter(|row| !row.is_empty())
            .map(|row| row.split(',').map(ToString::to_string).collect())
            .collect()
    }

    #[test]
    fn header_is_exact_and_crlf_terminated() {
        let mut store = store();
        store.append(entry(
            "Bonjour.",
            SimplificationOutcome::Simplified {
                text: "Salut.".into(),
            },
            1.0,
            3.5,
        ));
        let csv = session_csv(&store, None, false);
        let header = CSV_HEADER.join(",");
        assert!(csv.starts_with(&header));
        assert!(csv.ends_with("\r\n"));
        assert!(!csv.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn french_text_with_comma_quote_newline_is_quoted() {
        let mut store = store();
        store.append(entry(
            "Alors, il a dit : « ça va »,\nn'est-ce pas ?",
            SimplificationOutcome::Simplified {
                text: "Il dit \"ça va\".".into(),
            },
            1.0,
            3.5,
        ));
        let csv = session_csv(&store, None, false);
        assert!(csv.contains("\"Alors, il a dit : « ça va »,\nn'est-ce pas ?\""));
        assert!(csv.contains("\"Il dit \"\"ça va\"\".\""));
        assert_eq!(csv.split("\r\n").filter(|row| !row.is_empty()).count(), 2);
    }

    #[test]
    fn simplified_row_has_expected_fields() {
        let mut store = store();
        let mut simplified = entry(
            "Bonjour.",
            SimplificationOutcome::Simplified {
                text: "Salut.".into(),
            },
            1.0,
            3.5,
        );
        simplified.soft_violations = vec![
            Violation::soft("sentence_length", "21 words"),
            Violation::soft("passive_voice", "1 occurrence"),
        ];
        store.append(simplified);
        let fields = &data_rows(&session_csv(&store, None, false))[0];
        assert_eq!(fields[0], "0");
        assert_eq!(fields[1], "1.000");
        assert_eq!(fields[2], "3.500");
        assert_eq!(fields[3], "Bonjour.");
        assert_eq!(fields[4], "Salut.");
        assert_eq!(fields[5], "true");
        assert_eq!(fields[6], "");
        assert_eq!(fields[7], "false");
        assert_eq!(fields[8], "sentence_length;passive_voice");
        assert_eq!(fields[9], "");
        assert_eq!(fields[10], "");
        assert_eq!(fields[11], "");
        assert_eq!(fields[12], "clean");
        assert_eq!(fields[13], "");
    }

    #[test]
    fn low_confidence_row_carries_flags() {
        let mut store = store();
        let mut simplified = entry(
            "C'est compliqué.",
            SimplificationOutcome::Simplified {
                text: "C'est dur.".into(),
            },
            1.0,
            3.5,
        );
        simplified.confidence = SimplificationConfidence::LowConfidence {
            reasons: vec!["unfaithful".into(), "numbers_changed".into()],
        };
        store.append(simplified);
        let fields = &data_rows(&session_csv(&store, None, false))[0];
        assert_eq!(fields[12], "low_confidence");
        assert_eq!(fields[13], "unfaithful;numbers_changed");
    }

    #[test]
    fn fallback_reasons_are_snake_case() {
        let cases = [
            (FallbackReason::Timeout, "timeout"),
            (FallbackReason::RetryFailed, "retry_failed"),
            (FallbackReason::WrongLanguage, "wrong_language"),
            (FallbackReason::Unfaithful, "unfaithful"),
            (FallbackReason::BackendError, "backend_error"),
            (FallbackReason::DegradationSkip, "degradation_skip"),
        ];
        for (reason, expected) in cases {
            let mut store = store();
            store.append(entry(
                "C'est compliqué.",
                SimplificationOutcome::VerbatimFallback { reason },
                1.0,
                3.5,
            ));
            let fields = &data_rows(&session_csv(&store, None, false))[0];
            assert_eq!(fields[4], "");
            assert_eq!(fields[5], "false");
            assert_eq!(fields[6], expected);
            assert_eq!(fields[12], "");
            assert_eq!(fields[13], "");
        }
    }

    #[test]
    fn skipped_rows() {
        for (reason, expected) in [
            (SkipReason::OffTargetLanguage, "skipped_off_target_language"),
            (SkipReason::Fragment, "skipped_fragment"),
            (SkipReason::NotRequested, "not_requested"),
        ] {
            let mut store = store();
            store.append(entry(
                "Okay.",
                SimplificationOutcome::Skipped { reason },
                1.0,
                3.5,
            ));
            let fields = &data_rows(&session_csv(&store, None, false))[0];
            assert_eq!(fields[4], "");
            assert_eq!(fields[5], "false");
            assert_eq!(fields[6], expected);
        }
    }

    #[test]
    fn latency_columns_are_computed_from_stamps() {
        let metrics = LatencyMetrics::new();
        let mut store = store();
        let segment = entry(
            "Bonjour.",
            SimplificationOutcome::Simplified {
                text: "Salut.".into(),
            },
            1.0,
            3.5,
        );
        let id = segment.id().to_string();
        store.append(segment);
        metrics.record(&id, Stage::AudioEnd, 10.0);
        metrics.record(&id, Stage::PartialVisible, 11.234);
        metrics.record(&id, Stage::SimplifyStart, 11.5);
        metrics.record(&id, Stage::SimplifiedVisible, 12.75);
        let fields = &data_rows(&session_csv(&store, Some(&metrics), true))[0];
        assert_eq!(fields[9], "1234");
        assert_eq!(fields[10], "2750");
        assert_eq!(fields[11], "1250");
    }

    #[test]
    fn excluding_latency_keeps_simplify_duration() {
        let metrics = LatencyMetrics::new();
        let mut store = store();
        let segment = entry(
            "Bonjour.",
            SimplificationOutcome::Simplified {
                text: "Salut.".into(),
            },
            1.0,
            3.5,
        );
        let id = segment.id().to_string();
        store.append(segment);
        metrics.record(&id, Stage::AudioEnd, 10.0);
        metrics.record(&id, Stage::PartialVisible, 11.0);
        metrics.record(&id, Stage::SimplifyStart, 11.5);
        metrics.record(&id, Stage::SimplifiedVisible, 12.75);
        let fields = &data_rows(&session_csv(&store, Some(&metrics), false))[0];
        assert_eq!(fields[9], "");
        assert_eq!(fields[10], "");
        assert_eq!(fields[11], "1250");
    }

    #[test]
    fn missing_stamps_leave_cells_empty() {
        let metrics = LatencyMetrics::new();
        let mut store = store();
        let segment = entry(
            "Bonjour.",
            SimplificationOutcome::Simplified {
                text: "Salut.".into(),
            },
            1.0,
            3.5,
        );
        let id = segment.id().to_string();
        store.append(segment);
        metrics.record(&id, Stage::AudioEnd, 10.0);
        let fields = &data_rows(&session_csv(&store, Some(&metrics), true))[0];
        assert_eq!(fields[9], "");
        assert_eq!(fields[10], "");
        assert_eq!(fields[11], "");
    }

    #[test]
    fn reversion_span_flags_rows() {
        let mut store = store();
        store.reversion_spans = vec![crate::session::ReversionSpan {
            start: 5.0,
            end: 10.0,
        }];
        store.append(entry(
            "Switching to English.",
            SimplificationOutcome::Skipped {
                reason: SkipReason::OffTargetLanguage,
            },
            6.0,
            8.0,
        ));
        store.append(entry(
            "On continue en français.",
            SimplificationOutcome::Simplified {
                text: "On continue.".into(),
            },
            12.0,
            14.0,
        ));
        let rows = data_rows(&session_csv(&store, None, false));
        assert_eq!(rows[0][7], "true");
        assert_eq!(rows[1][7], "false");
    }

    #[test]
    fn markdown_and_plain_text_render_both_tiers() {
        let mut store = store();
        store.started_at_iso = Some("2026-11-21T12:00:00Z".to_string());
        store.duration_seconds = Some(125.0);
        store.append(entry(
            "Bonjour.",
            SimplificationOutcome::Simplified {
                text: "Salut.".into(),
            },
            1.0,
            3.5,
        ));
        store.append(entry(
            "Au revoir.",
            SimplificationOutcome::VerbatimFallback {
                reason: FallbackReason::Timeout,
            },
            4.0,
            5.0,
        ));
        store.append(entry(
            "Okay.",
            SimplificationOutcome::Skipped {
                reason: SkipReason::OffTargetLanguage,
            },
            6.0,
            7.0,
        ));
        let markdown = markdown(&store);
        assert!(markdown.contains("# Live Linguist — Transcript"));
        assert!(markdown.contains("- **Duration:** 2 min 5 s"));
        assert!(markdown.contains("| [00:01] | Bonjour. | Salut. |"));
        assert!(markdown.contains("≡ Au revoir."));
        assert!(markdown.contains("| — |"));

        let plain = plain_text(&store);
        assert!(plain.contains("[00:01] V: Bonjour."));
        assert!(plain.contains("        S: Salut."));
        assert!(plain.contains("        S: ≡ Au revoir."));
        assert!(plain.contains("        S: —"));
    }

    #[test]
    fn off_target_english_is_flagged_in_exports() {
        let mut store = store();
        store.append(SimplifiedSegment::new(
            FinalizedSegment::with_language("Hello.", Some("en".into()), 1.0, 2.0),
            SimplificationOutcome::Skipped {
                reason: SkipReason::OffTargetLanguage,
            },
        ));
        assert!(markdown(&store).contains("| [00:01] EN |"));
        assert!(plain_text(&store).contains("[00:01] [EN] V: Hello."));
    }
}
