//! Conservative disfluency pre-pass over ASR source text.
//!
//! Ported from `Sources/LiveLinguistKit/Simplification/DisfluencyCleaner.swift`.
//! Removes only unambiguous hesitation fillers and immediate word repetitions;
//! the verbatim transcript shown to the user is untouched.

use crate::text_analysis::trim_token;

fn fillers(language: &str) -> &'static [&'static str] {
    match language {
        "fr" => &["euh", "heu", "hum"],
        "en" => &["um", "uh", "uhm", "er", "erm", "hmm"],
        "de" => &["äh", "ähm", "öhm", "hm"],
        "es" => &["eh", "ehm", "em"],
        _ => &[],
    }
}

fn bare_form(word: &str) -> String {
    // Haskell's CharacterSet.punctuationCharacters ∪ .symbols includes ASCII
    // symbols; trim_token covers both ends.
    trim_token(&word.to_lowercase()).to_string()
}

/// Clean a sentence for the prompt and validators.
pub fn clean(text: &str, language: &str) -> String {
    let fillers = fillers(language);
    let words = text.split_whitespace().collect::<Vec<_>>();
    let mut kept: Vec<&str> = Vec::new();

    for word in &words {
        let bare = bare_form(word);
        if fillers.contains(&bare.as_str()) {
            continue;
        }
        if !bare.is_empty() {
            if let Some(last) = kept.last() {
                if bare_form(last) == bare {
                    kept.pop();
                }
            }
        }
        kept.push(word);
    }

    if kept.len() >= words.len() {
        return text.to_string();
    }
    let cleaned = kept.join(" ");
    if cleaned.is_empty() {
        text.to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unambiguous_fillers_are_removed() {
        assert_eq!(
            clean("On va euh visiter la médina.", "fr"),
            "On va visiter la médina."
        );
        assert_eq!(
            clean("Um. So we go to the market.", "en"),
            "So we go to the market."
        );
    }

    #[test]
    fn immediate_repetitions_collapse() {
        assert_eq!(
            clean("on était partis pour euh pour visiter la médina", "fr"),
            "on était partis pour visiter la médina"
        );
    }

    #[test]
    fn ambiguous_filler_words_are_kept() {
        let english = "I like this plan and we should also start soon.";
        assert_eq!(clean(english, "en"), english);
        let german = "Wir haben also einen Plan für morgen.";
        assert_eq!(clean(german, "de"), german);
    }

    #[test]
    fn clean_text_is_returned_verbatim() {
        let text = "La conférence a été annulée par les organisateurs.";
        assert_eq!(clean(text, "fr"), text);
    }

    #[test]
    fn all_filler_input_falls_back_to_original() {
        assert_eq!(clean("euh euh euh", "fr"), "euh euh euh");
    }
}
