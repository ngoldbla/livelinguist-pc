//! Shared tokenization and sentence helpers for the deterministic validators.
//!
//! Ported from `Sources/LiveLinguistKit/Validation/TextAnalysis.swift`.

/// Trim a token of surrounding punctuation and symbols, keeping inner
/// apostrophes and hyphens.
pub fn trim_token(token: &str) -> &str {
    token.trim_matches(is_punctuation_like)
}

/// Approximation of `CharacterSet.punctuationCharacters ∪ .symbols`.
pub(crate) fn is_punctuation_like(c: char) -> bool {
    c.is_ascii_punctuation()
        || matches!(
            c,
            '«' | '»'
                | '…'
                | '‘'
                | '’'
                | '“'
                | '”'
                | '–'
                | '—'
                | '¡'
                | '¿'
                | '·'
                | '、'
                | '。'
                | '，'
                | '！'
                | '？'
                | '：'
                | '；'
                | '（'
                | '）'
                | '【'
                | '】'
        )
}

/// Lowercased tokens with surrounding punctuation stripped.
pub fn tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(trim_token)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Split into sentences on terminal punctuation (`. ! ? …`).
pub fn sentences(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        current.push(character);
        if ".!?…".contains(character) {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                result.push(trimmed.to_string());
            }
            current.clear();
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        result.push(trimmed.to_string());
    }
    result
}

pub fn word_count(sentence: &str) -> usize {
    sentence.split_whitespace().count()
}

/// Heuristic syllable count (vowel groups, silent-e adjustment).
pub fn syllables(word: &str) -> usize {
    let lowered: String = word.chars().filter(|c| c.is_alphabetic()).collect();
    if lowered.is_empty() {
        return 0;
    }
    let lowered = lowered.to_lowercase();
    let mut count = 0usize;
    let mut previous_was_vowel = false;
    for character in lowered.chars() {
        let is_vowel = "aeiouy".contains(character);
        if is_vowel && !previous_was_vowel {
            count += 1;
        }
        previous_was_vowel = is_vowel;
    }
    if lowered.ends_with('e') && !lowered.ends_with("le") && count > 1 {
        count -= 1;
    }
    count.max(1)
}

/// First `n` characters of a token (Unicode scalar safe).
pub(crate) fn char_prefix(token: &str, n: usize) -> String {
    token.chars().take(n).collect()
}

/// Character count (not byte length) for compound/length checks.
pub(crate) fn char_len(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_lowercase_and_strip_outer_punctuation() {
        assert_eq!(
            tokens("Bonjour, le monde !"),
            vec!["bonjour", "le", "monde"]
        );
        assert_eq!(tokens("l'occasion"), vec!["l'occasion"]);
        assert_eq!(tokens("« fête »"), vec!["fête"]);
    }

    #[test]
    fn sentences_split_on_terminals() {
        assert_eq!(
            sentences("Bonjour. Comment ça va ? Très bien…"),
            vec!["Bonjour.", "Comment ça va ?", "Très bien…"]
        );
        assert_eq!(sentences("no punctuation"), vec!["no punctuation"]);
        assert!(sentences("").is_empty());
    }

    #[test]
    fn word_count_and_syllables() {
        assert_eq!(word_count("one two three"), 3);
        assert_eq!(syllables("hello"), 2);
        assert_eq!(syllables("a"), 1);
        assert_eq!(syllables(""), 0);
    }
}
