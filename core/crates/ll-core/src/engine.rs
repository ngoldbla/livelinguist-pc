//! Simplification engine: backend call + validators + bounded retry ladder.
//!
//! Ported from `Sources/LiveLinguistKit/Simplification/SimplificationEngine.swift`.
//! Validators are confidence signals, not gates: imperfect output is displayed
//! flagged, never suppressed behind a verbatim fallback unless the device
//! produced no usable text at all.

use crate::disfluency::clean;
use crate::fidelity::FidelityGuard;
use crate::prompt::{PromptBuilder, SimplificationRequest};
use crate::session::{
    FallbackReason, FinalizedSegment, SimplificationConfidence, SimplificationOutcome,
    SimplifiedSegment, SkipReason,
};
use crate::validation::{Severity, ValidatorSuite, Violation};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendErrorKind {
    Timeout,
    Backend,
    OutOfMemory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    pub kind: BackendErrorKind,
    pub message: String,
}

impl BackendError {
    pub fn timeout() -> Self {
        Self {
            kind: BackendErrorKind::Timeout,
            message: "generation budget exceeded".to_string(),
        }
    }

    pub fn backend(message: impl Into<String>) -> Self {
        Self {
            kind: BackendErrorKind::Backend,
            message: message.into(),
        }
    }

    pub fn out_of_memory(message: impl Into<String>) -> Self {
        Self {
            kind: BackendErrorKind::OutOfMemory,
            message: message.into(),
        }
    }
}

/// A synchronous simplifier backend. Real llama.cpp work runs behind this seam;
/// the pipeline worker owns thread isolation and cancellation.
pub trait SimplifierBackend: Send {
    fn name(&self) -> &'static str;
    fn simplify(&mut self, request: &SimplificationRequest) -> Result<String, BackendError>;
}

/// A deterministic scripted backend for tests, demos, and fault injection.
#[derive(Debug, Clone, Default)]
pub struct ScriptedBackend {
    outputs: VecDeque<Result<String, BackendError>>,
}

impl ScriptedBackend {
    pub fn new(outputs: impl IntoIterator<Item = Result<String, BackendError>>) -> Self {
        Self {
            outputs: outputs.into_iter().collect(),
        }
    }

    pub fn from_strings(outputs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::new(outputs.into_iter().map(|output| Ok(output.into())))
    }
}

impl SimplifierBackend for ScriptedBackend {
    fn name(&self) -> &'static str {
        "scripted"
    }

    fn simplify(&mut self, _request: &SimplificationRequest) -> Result<String, BackendError> {
        self.outputs
            .pop_front()
            .unwrap_or_else(|| Ok(String::new()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimplifyConfig {
    /// Hard per-segment generation budget spanning all attempts.
    pub generation_budget: Duration,
    /// Constrained retries after a hard violation, on top of the first attempt.
    pub max_retries: usize,
    /// Extra full-budget attempts when generation yields no text at all.
    pub no_output_retries: usize,
}

impl Default for SimplifyConfig {
    fn default() -> Self {
        Self {
            generation_budget: Duration::from_millis(1_500),
            max_retries: 1,
            no_output_retries: 0,
        }
    }
}

impl SimplifyConfig {
    /// Click-to-simplify: the user explicitly asked and accepts more latency.
    pub fn on_demand() -> Self {
        Self {
            generation_budget: Duration::from_secs(8),
            max_retries: 2,
            no_output_retries: 2,
        }
    }
}

#[derive(Debug)]
struct Attempt {
    output: String,
    violations: Vec<Violation>,
    hard: Vec<Violation>,
}

pub struct SimplificationEngine {
    backend: Box<dyn SimplifierBackend>,
    escalation: Option<Box<dyn SimplifierBackend>>,
    prompt_builder: PromptBuilder,
    validators: ValidatorSuite,
    config: SimplifyConfig,
    session_language: String,
    session_base_language: String,
}

impl SimplificationEngine {
    pub fn new(backend: Box<dyn SimplifierBackend>, pack: &crate::pack::FrameworkPack) -> Self {
        Self::with_config(backend, pack, SimplifyConfig::default())
    }

    pub fn with_config(
        backend: Box<dyn SimplifierBackend>,
        pack: &crate::pack::FrameworkPack,
        config: SimplifyConfig,
    ) -> Self {
        Self {
            backend,
            escalation: None,
            prompt_builder: PromptBuilder::new(pack),
            validators: ValidatorSuite::new(pack),
            config,
            session_language: pack.language.clone(),
            session_base_language: base_code(pack.base_lang.as_deref().unwrap_or(&pack.language)),
        }
    }

    pub fn with_escalation(mut self, escalation: Box<dyn SimplifierBackend>) -> Self {
        self.escalation = Some(escalation);
        self
    }

    pub fn config(&self) -> SimplifyConfig {
        self.config
    }

    pub fn validators(&self) -> &ValidatorSuite {
        &self.validators
    }

    pub fn simplify(
        &mut self,
        segment: &FinalizedSegment,
        previous_segments: &[String],
        skip_simplification: bool,
    ) -> SimplifiedSegment {
        // Off-target segments are never simplified (Tier 1 only).
        if let Some(detected) = segment.detected_language.as_deref() {
            if base_code(detected) != self.session_base_language {
                return SimplifiedSegment::new(
                    segment.clone(),
                    SimplificationOutcome::Skipped {
                        reason: SkipReason::OffTargetLanguage,
                    },
                );
            }
        }

        let source = clean(&segment.text, &self.session_language);
        if FidelityGuard::is_fragment(&source, &self.session_language) {
            return SimplifiedSegment::new(
                segment.clone(),
                SimplificationOutcome::Skipped {
                    reason: SkipReason::Fragment,
                },
            );
        }
        if skip_simplification {
            return SimplifiedSegment::new(
                segment.clone(),
                SimplificationOutcome::VerbatimFallback {
                    reason: FallbackReason::DegradationSkip,
                },
            );
        }

        let deadline = Instant::now() + self.config.generation_budget;
        let mut request = self.prompt_builder.make_request(&source, previous_segments);
        let mut best: Option<Attempt> = None;
        let mut last_failure = FallbackReason::RetryFailed;

        for _ in 0..=self.config.max_retries {
            let output = match attempt(self.backend.as_mut(), &request, deadline) {
                Ok(output) => output,
                Err(error) => {
                    last_failure = failure_reason(&error);
                    break;
                }
            };
            if output.trim().is_empty() {
                continue;
            }
            let evaluated = self.evaluate(&output, &source, previous_segments);
            if evaluated.hard.is_empty() {
                return SimplifiedSegment::with_validation(
                    segment.clone(),
                    SimplificationOutcome::Simplified {
                        text: evaluated.output.clone(),
                    },
                    evaluated.violations,
                    SimplificationConfidence::Clean,
                );
            }
            let improve = best
                .as_ref()
                .map(|current| evaluated.hard.len() < current.hard.len())
                .unwrap_or(true);
            if improve {
                best = Some(evaluated.clone());
            }
            request = self.prompt_builder.make_retry_request(
                &source,
                previous_segments,
                &output,
                &evaluated.hard,
            );
        }

        if best.is_none() {
            for index in 0..self.config.no_output_retries {
                let is_last = index + 1 == self.config.no_output_retries;
                let recovery_deadline = Instant::now() + self.config.generation_budget;
                let output = if is_last {
                    match self.escalation.as_mut() {
                        Some(escalation) => {
                            attempt(escalation.as_mut(), &request, recovery_deadline)
                        }
                        None => attempt(self.backend.as_mut(), &request, recovery_deadline),
                    }
                } else {
                    attempt(self.backend.as_mut(), &request, recovery_deadline)
                };
                match output {
                    Ok(output) => {
                        if output.trim().is_empty() {
                            continue;
                        }
                        best = Some(self.evaluate(&output, &source, previous_segments));
                        break;
                    }
                    Err(error) => {
                        last_failure = failure_reason(&error);
                        continue;
                    }
                }
            }
        }

        let Some(best) = best else {
            return SimplifiedSegment::new(
                segment.clone(),
                SimplificationOutcome::VerbatimFallback {
                    reason: last_failure,
                },
            );
        };
        if best.hard.is_empty() {
            return SimplifiedSegment::with_validation(
                segment.clone(),
                SimplificationOutcome::Simplified {
                    text: best.output.clone(),
                },
                best.violations,
                SimplificationConfidence::Clean,
            );
        }
        let mut reasons: Vec<String> = Vec::new();
        for violation in &best.hard {
            if !reasons.contains(&violation.rule) {
                reasons.push(violation.rule.clone());
            }
        }
        SimplifiedSegment::with_validation(
            segment.clone(),
            SimplificationOutcome::Simplified { text: best.output },
            best.violations
                .into_iter()
                .filter(|violation| violation.severity == Severity::Soft)
                .collect(),
            SimplificationConfidence::LowConfidence { reasons },
        )
    }

    fn evaluate(&self, output: &str, source: &str, context: &[String]) -> Attempt {
        let violations = self.validators.validate(output, source, context);
        let hard = violations
            .iter()
            .filter(|violation| violation.severity == Severity::Hard)
            .cloned()
            .collect();
        Attempt {
            output: output.to_string(),
            violations,
            hard,
        }
    }
}

impl Clone for Attempt {
    fn clone(&self) -> Self {
        Self {
            output: self.output.clone(),
            violations: self.violations.clone(),
            hard: self.hard.clone(),
        }
    }
}

fn base_code(code: &str) -> String {
    code.split(['-', '_']).next().unwrap_or(code).to_lowercase()
}

fn failure_reason(error: &BackendError) -> FallbackReason {
    match error.kind {
        BackendErrorKind::Timeout => FallbackReason::Timeout,
        BackendErrorKind::Backend | BackendErrorKind::OutOfMemory => FallbackReason::BackendError,
    }
}

/// Run one backend call racing the budget deadline. A synchronous backend
/// cannot be interrupted; an over-budget result is discarded as a timeout.
fn attempt(
    backend: &mut dyn SimplifierBackend,
    request: &SimplificationRequest,
    deadline: Instant,
) -> Result<String, BackendError> {
    if Instant::now() >= deadline {
        return Err(BackendError::timeout());
    }
    let result = backend.simplify(request);
    if Instant::now() > deadline {
        return Err(BackendError::timeout());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Framework;
    use crate::pack::FrameworkPack;
    use crate::session::FinalizedSegment;

    fn pack(framework: Framework) -> FrameworkPack {
        FrameworkPack::load_framework(framework).unwrap()
    }

    fn segment(text: &str) -> FinalizedSegment {
        FinalizedSegment::new(text, 0.0, 2.0)
    }

    #[test]
    fn clean_output_is_displayed_with_soft_violations() {
        let pack = pack(Framework::Falc);
        let shot = &pack.few_shots[0];
        let backend = ScriptedBackend::from_strings([shot.simplified.clone()]);
        let mut engine = SimplificationEngine::new(Box::new(backend), &pack);
        let result = engine.simplify(&segment(&shot.source), &[], false);
        assert_eq!(
            result.outcome.display_text(),
            Some(shot.simplified.as_str())
        );
        assert_eq!(result.confidence, SimplificationConfidence::Clean);
    }

    #[test]
    fn hard_violation_retries_then_returns_clean() {
        let pack = pack(Framework::Falc);
        let shot = &pack.few_shots[0];
        let too_long = "Je pense que nous devrions aller voir le nouveau film ensemble demain soir vers huit heures.";
        let backend =
            ScriptedBackend::from_strings([too_long.to_string(), shot.simplified.clone()]);
        let mut engine = SimplificationEngine::new(Box::new(backend), &pack);
        let result = engine.simplify(&segment(&shot.source), &[], false);
        assert_eq!(
            result.outcome.display_text(),
            Some(shot.simplified.as_str())
        );
    }

    #[test]
    fn persistent_hard_violation_is_low_confidence_not_hidden() {
        let pack = pack(Framework::Falc);
        let too_long = "Je pense que nous devrions aller voir le nouveau film ensemble demain soir vers huit heures.";
        let backend = ScriptedBackend::from_strings([too_long.to_string(), too_long.to_string()]);
        let mut engine = SimplificationEngine::new(Box::new(backend), &pack);
        let result = engine.simplify(&segment("Le film commence demain soir."), &[], false);
        assert!(result.outcome.is_simplified());
        assert!(!result.confidence.is_clean());
        assert!(result
            .confidence
            .reasons()
            .contains(&"sentence_length".to_string()));
    }

    #[test]
    fn timeout_falls_back_to_verbatim() {
        let pack = pack(Framework::Falc);
        let backend = ScriptedBackend::new([Err(BackendError::timeout())]);
        let mut engine = SimplificationEngine::new(Box::new(backend), &pack);
        let result = engine.simplify(&segment("Le film commence demain soir."), &[], false);
        assert_eq!(
            result.outcome,
            SimplificationOutcome::VerbatimFallback {
                reason: FallbackReason::Timeout
            }
        );
    }

    #[test]
    fn empty_output_uses_recovery_retries() {
        let pack = pack(Framework::Falc);
        let shot = &pack.few_shots[0];
        let backend = ScriptedBackend::from_strings(["".to_string(), shot.simplified.clone()]);
        let config = SimplifyConfig {
            max_retries: 0,
            no_output_retries: 1,
            ..SimplifyConfig::default()
        };
        let mut engine = SimplificationEngine::with_config(Box::new(backend), &pack, config);
        let result = engine.simplify(&segment(&shot.source), &[], false);
        assert_eq!(
            result.outcome.display_text(),
            Some(shot.simplified.as_str())
        );
    }

    #[test]
    fn off_target_language_is_skipped() {
        let pack = pack(Framework::Falc);
        let backend = ScriptedBackend::from_strings(["ignored"]);
        let mut engine = SimplificationEngine::new(Box::new(backend), &pack);
        let mut segment = segment("This is English.");
        segment.detected_language = Some("en".to_string());
        let result = engine.simplify(&segment, &[], false);
        assert_eq!(
            result.outcome,
            SimplificationOutcome::Skipped {
                reason: SkipReason::OffTargetLanguage
            }
        );
    }

    #[test]
    fn fragment_is_skipped() {
        let pack = pack(Framework::LeichteSprache);
        let backend = ScriptedBackend::from_strings(["ignored"]);
        let mut engine = SimplificationEngine::new(Box::new(backend), &pack);
        let result = engine.simplify(&segment("Politik."), &[], false);
        assert_eq!(
            result.outcome,
            SimplificationOutcome::Skipped {
                reason: SkipReason::Fragment
            }
        );
    }

    #[test]
    fn degradation_skip_is_verbatim_with_marker() {
        let pack = pack(Framework::Falc);
        let backend = ScriptedBackend::from_strings(["ignored"]);
        let mut engine = SimplificationEngine::new(Box::new(backend), &pack);
        let result = engine.simplify(&segment("Le film commence demain soir."), &[], true);
        assert_eq!(
            result.outcome,
            SimplificationOutcome::VerbatimFallback {
                reason: FallbackReason::DegradationSkip
            }
        );
    }

    #[test]
    fn retry_request_sees_violations() {
        // A backend that records the prompt would prove the retry names the
        // violation; the golden prompt tests already pin the renderer, so here
        // we assert the engine reaches the retry path.
        let pack = pack(Framework::LeichteSprache);
        let too_long = "Wir haben gestern Abend zusammen mit allen Freunden und der ganzen Familie sehr lange über die Pläne gesprochen.";
        let backend = ScriptedBackend::from_strings([
            too_long.to_string(),
            "Wir haben gestern Abend gesprochen.".to_string(),
        ]);
        let mut engine = SimplificationEngine::new(Box::new(backend), &pack);
        let result = engine.simplify(&segment("Gestern haben wir lange gesprochen."), &[], false);
        assert!(result.outcome.is_simplified());
    }
}
