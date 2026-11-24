//! Text normalizer and WER/CER scoring for ASR evaluation (T30).
//!
//! Scores are corpus-level: sum of edits over sum of reference lengths across
//! clips ([`CorpusScore`]), never the mean of per-clip rates. Nothing here
//! logs or stores text; callers decide where hypotheses go.

use unicode_general_category::{get_general_category, GeneralCategory};

fn is_punctuation(c: char) -> bool {
    matches!(
        get_general_category(c),
        GeneralCategory::ConnectorPunctuation
            | GeneralCategory::DashPunctuation
            | GeneralCategory::OpenPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::InitialPunctuation
            | GeneralCategory::FinalPunctuation
            | GeneralCategory::OtherPunctuation
    )
}

/// Lowercase (Unicode), drop punctuation (`P*` categories), collapse
/// whitespace, trim. Digits are kept as-is. With `cjk`, all spaces go too.
pub fn normalize(text: &str, cjk: bool) -> String {
    let lowered = text.to_lowercase();
    let stripped: String = lowered
        .chars()
        .map(|c| if is_punctuation(c) { ' ' } else { c })
        .collect();
    let words: Vec<&str> = stripped.split_whitespace().collect();
    if cjk {
        words.concat()
    } else {
        words.join(" ")
    }
}

fn levenshtein<T: PartialEq>(reference: &[T], hypothesis: &[T]) -> usize {
    if reference.is_empty() {
        return hypothesis.len();
    }
    let mut previous: Vec<usize> = (0..=hypothesis.len()).collect();
    let mut current = vec![0; hypothesis.len() + 1];
    for (i, r) in reference.iter().enumerate() {
        current[0] = i + 1;
        for (j, h) in hypothesis.iter().enumerate() {
            let substitution = previous[j] + usize::from(r != h);
            current[j + 1] = substitution.min(previous[j + 1] + 1).min(current[j] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[hypothesis.len()]
}

/// Word edit distance and reference word count on normalized text.
pub fn wer(reference: &str, hypothesis: &str) -> (usize, usize) {
    let reference = normalize(reference, false);
    let hypothesis = normalize(hypothesis, false);
    let r: Vec<&str> = reference.split_whitespace().collect();
    let h: Vec<&str> = hypothesis.split_whitespace().collect();
    (levenshtein(&r, &h), r.len())
}

/// Character edit distance and reference length (Unicode scalar values) on
/// normalized text. Spaces are removed, so this suits CJK scripts.
pub fn cer(reference: &str, hypothesis: &str) -> (usize, usize) {
    let r: Vec<char> = normalize(reference, true).chars().collect();
    let h: Vec<char> = normalize(hypothesis, true).chars().collect();
    (levenshtein(&r, &h), r.len())
}

/// Which metric scores a locale: CER for ja/ko/zh, WER otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    Wer,
    Cer,
}

impl Metric {
    pub fn for_locale(locale: &str) -> Self {
        let base = locale.split(['-', '_']).next().unwrap_or("");
        match base.to_ascii_lowercase().as_str() {
            "ja" | "ko" | "zh" => Metric::Cer,
            _ => Metric::Wer,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Metric::Wer => "wer",
            Metric::Cer => "cer",
        }
    }

    pub fn score(self, reference: &str, hypothesis: &str) -> (usize, usize) {
        match self {
            Metric::Wer => wer(reference, hypothesis),
            Metric::Cer => cer(reference, hypothesis),
        }
    }
}

/// Corpus accumulator: sum(edits) / sum(reference length).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CorpusScore {
    pub edits: usize,
    pub reference_len: usize,
    pub clips: usize,
}

impl CorpusScore {
    pub fn add(&mut self, (edits, reference_len): (usize, usize)) {
        self.edits += edits;
        self.reference_len += reference_len;
        self.clips += 1;
    }

    pub fn rate(&self) -> f64 {
        if self.reference_len == 0 {
            0.0
        } else {
            self.edits as f64 / self.reference_len as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_zero() {
        assert_eq!(wer("the cat sat", "the cat sat"), (0, 3));
    }

    #[test]
    fn substitution_plus_insertion() {
        assert_eq!(wer("the cat sat", "the bat sat on"), (2, 3));
    }

    #[test]
    fn normalize_strips_punctuation_and_lowercases() {
        assert_eq!(
            normalize("¿Qué? ¡Sí! a.b, c; d: «e» \"f\"", false),
            "qué sí a b c d e f"
        );
        assert_eq!(normalize("ÉCOLE", false), "école");
        assert_eq!(
            normalize("  many   spaces\there ", false),
            "many spaces here"
        );
        assert_eq!(normalize("In 2024, 3 cats", false), "in 2024 3 cats");
        assert_eq!(normalize("今天 天气，好。", true), "今天天气好");
    }

    #[test]
    fn cer_counts_chars() {
        assert_eq!(cer("今天天气好", "今天天汽好"), (1, 5));
    }

    #[test]
    fn empty_hypothesis_costs_reference_length() {
        assert_eq!(wer("one two three four", ""), (4, 4));
        assert_eq!(cer("今天天气好", ""), (5, 5));
    }

    #[test]
    fn corpus_score_is_pooled_not_averaged() {
        let mut corpus = CorpusScore::default();
        corpus.add(wer("a b c d e f g h i j", "a b c d e f g h i j"));
        corpus.add(wer("x", "y"));
        // Pooled: 1 / 11, not mean(0.0, 1.0) = 0.5.
        assert_eq!(corpus.edits, 1);
        assert_eq!(corpus.reference_len, 11);
        assert!((corpus.rate() - 1.0 / 11.0).abs() < 1e-12);
    }

    #[test]
    fn metric_choice() {
        assert_eq!(Metric::for_locale("ja-JP"), Metric::Cer);
        assert_eq!(Metric::for_locale("zh-CN"), Metric::Cer);
        assert_eq!(Metric::for_locale("ko-KR"), Metric::Cer);
        assert_eq!(Metric::for_locale("en-US"), Metric::Wer);
    }
}
