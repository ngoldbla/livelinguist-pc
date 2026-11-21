//! Pure text shaping for streamed LLM caption decoding.
//!
//! Ported from `Sources/LiveLinguistKit/Simplification/CaptionDecoding.swift`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodingStyle {
    /// Stop at the first complete sentence (stock few-shot model).
    FirstSentence,
    /// Take the whole line (fine-tuned model splits run-ons into sentences).
    SingleLine,
}

fn is_terminal(character: char) -> bool {
    ".!?…".contains(character)
}

/// True once two complete sentences in a row are identical (case-insensitive).
fn has_consecutive_sentence_repeat(text: &str) -> bool {
    let completed = sentence_chunks(text)
        .into_iter()
        .filter(|chunk| {
            chunk
                .chars()
                .rev()
                .find(|c| !c.is_whitespace())
                .is_some_and(is_terminal)
        })
        .collect::<Vec<_>>();
    if completed.len() < 2 {
        return false;
    }
    let a = completed[completed.len() - 1].trim();
    let b = completed[completed.len() - 2].trim();
    !a.is_empty() && a.to_lowercase() == b.to_lowercase()
}

fn sentence_chunks(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        current.push(character);
        if is_terminal(character) {
            chunks.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// True once the streamed output is a finished caption under `style`.
pub fn reached_stop(streamed: &str, style: DecodingStyle) -> bool {
    let mut visible = streamed.trim_start();
    if visible.starts_with("<think>") {
        match visible.find("</think>") {
            Some(index) => {
                visible = visible[index + "</think>".len()..].trim_start();
            }
            None => return false,
        }
    }
    if visible.contains('\n') {
        return true;
    }
    if has_consecutive_sentence_repeat(visible) {
        return true;
    }
    if style == DecodingStyle::SingleLine {
        return false;
    }
    let Some(last) = visible.chars().last() else {
        return false;
    };
    if !is_terminal(last) {
        return false;
    }
    let words = visible.split_whitespace().collect::<Vec<_>>();
    if words.len() < 3 {
        return false;
    }
    let Some(last_word) = words.last() else {
        return false;
    };
    if last == '.' {
        let stem = &last_word[..last_word.len().saturating_sub(1)];
        let mut stem_chars = stem.chars();
        if let (Some(only), None) = (stem_chars.next(), stem_chars.next()) {
            if only.is_uppercase() {
                return false;
            }
        }
        if stem.chars().last().is_some_and(|c| c.is_numeric()) {
            return false;
        }
    }
    true
}

/// Back-compat alias for the stock first-sentence path.
pub fn reached_sentence_stop(streamed: &str) -> bool {
    reached_stop(streamed, DecodingStyle::FirstSentence)
}

/// One caption line: anything past a newline is run-on output.
pub fn first_line(text: &str) -> String {
    text.split('\n')
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Collapse runs of identical consecutive sentences ("A. A. A." -> "A.").
pub fn collapse_repeated_sentences(text: &str) -> String {
    let mut kept: Vec<String> = Vec::new();
    let mut last_key: Option<String> = None;
    for chunk in sentence_chunks(text) {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_lowercase();
        if last_key.as_deref() == Some(key.as_str()) {
            continue;
        }
        kept.push(trimmed.to_string());
        last_key = Some(key);
    }
    kept.join(" ")
}

/// Drop a leading `<think>…</think>` block. An unterminated block means the
/// whole output was reasoning, so return empty.
pub fn strip_think_block(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with("<think>") {
        return trimmed.to_string();
    }
    match trimmed.find("</think>") {
        Some(index) => trimmed[index + "</think>".len()..].trim().to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_on_sentence_punctuation_after_three_words() {
        assert!(reached_sentence_stop("On peut attendre."));
        assert!(reached_sentence_stop("Quel est le sujet ?"));
        assert!(reached_sentence_stop("On va commencer maintenant…"));
    }

    #[test]
    fn keeps_decoding_short_or_unfinished_output() {
        assert!(!reached_sentence_stop(""));
        assert!(!reached_sentence_stop("Attendons."));
        assert!(!reached_sentence_stop("On peut attendre"));
    }

    #[test]
    fn stops_on_newline() {
        assert!(reached_sentence_stop("Viens voir\nOriginal:"));
        assert!(reached_sentence_stop("Viens\n"));
    }

    #[test]
    fn never_stops_inside_unterminated_think_block() {
        assert!(!reached_sentence_stop("<think>\nLet me think.\n"));
        assert!(reached_sentence_stop(
            "<think>court</think>\nOn peut attendre."
        ));
    }

    #[test]
    fn abbreviation_and_number_periods_do_not_stop() {
        assert!(!reached_sentence_stop("Il parle avec M."));
        assert!(!reached_sentence_stop("Le prix est 3."));
        assert!(reached_sentence_stop("Le prix est 3.50 euros."));
    }

    #[test]
    fn first_line_cuts_run_on_output() {
        assert_eq!(first_line("Une ligne.\nOriginal: deux"), "Une ligne.");
        assert_eq!(first_line("Une ligne.  "), "Une ligne.");
        assert_eq!(first_line(""), "");
    }

    #[test]
    fn single_line_keeps_decoding_past_periods() {
        assert!(!reached_stop(
            "Satz eins. Satz zwei.",
            DecodingStyle::SingleLine
        ));
        assert!(!reached_stop(
            "On peut attendre. On va commencer.",
            DecodingStyle::SingleLine
        ));
        assert!(reached_stop(
            "Satz eins. Satz zwei.\nOriginal:",
            DecodingStyle::SingleLine
        ));
    }

    #[test]
    fn stops_on_consecutive_repeat() {
        let loop_text =
            "Wir haben eine Frage über unsere Vernunft. Wir haben eine Frage über unsere Vernunft.";
        assert!(reached_stop(loop_text, DecodingStyle::SingleLine));
        assert!(reached_stop(loop_text, DecodingStyle::FirstSentence));
        assert!(reached_stop(
            "Hallo Welt. hallo welt.",
            DecodingStyle::SingleLine
        ));
    }

    #[test]
    fn distinct_sentences_still_decode() {
        assert!(!reached_stop(
            "Satz eins. Satz zwei.",
            DecodingStyle::SingleLine
        ));
        assert!(!reached_stop(
            "Satz eins. Satz zwei. Satz drei.",
            DecodingStyle::SingleLine
        ));
        assert!(!reached_stop(
            "Wir reden über die Vorsorge. Wir reden",
            DecodingStyle::SingleLine
        ));
    }

    #[test]
    fn collapses_consecutive_duplicate_sentences() {
        let loop_text = "Wir haben eine Frage über unsere Vernunft. Wir haben eine Frage über unsere Vernunft. Wir haben eine Frage über unsere Vernunft.";
        assert_eq!(
            collapse_repeated_sentences(loop_text),
            "Wir haben eine Frage über unsere Vernunft."
        );
        assert_eq!(
            collapse_repeated_sentences("Satz eins. Satz zwei."),
            "Satz eins. Satz zwei."
        );
        assert_eq!(
            collapse_repeated_sentences("A ist gut. B ist da. A ist gut."),
            "A ist gut. B ist da. A ist gut."
        );
        assert_eq!(collapse_repeated_sentences("Attendons"), "Attendons");
        assert_eq!(collapse_repeated_sentences(""), "");
    }

    #[test]
    fn strips_think_block() {
        assert_eq!(
            strip_think_block("<think>\n\n</think>\n\nViens voir."),
            "Viens voir."
        );
        assert_eq!(strip_think_block("Viens voir."), "Viens voir.");
        assert_eq!(strip_think_block("<think>only reasoning"), "");
    }
}
