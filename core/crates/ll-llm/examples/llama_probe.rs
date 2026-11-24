//! llama.cpp CPU probe for the pinned EasyLanguage GGUF (T17).
//!
//! Usage: `llama_probe <gguf> <locale> [--threads N] [--print-text]`
//! Uses the exact T09 prompt. Prints load ms, prompt tokens, TTFT, generated
//! tokens, tok/s, total ms. Text (raw and think-stripped) only with
//! `--print-text`, for a human at the terminal; never in evidence files.

use ll_core::decoding::strip_think_block;
use ll_core::language::Language;
use ll_core::pack::FrameworkPack;
use ll_core::prompt::render_easylanguage_prompt;
use ll_llm::llama::{LlamaBackend, MAX_NEW_TOKENS};
use std::time::Instant;

const SENTENCE: &str = "Notwithstanding the aforementioned considerations, the committee has \
resolved to postpone the deliberations indefinitely.";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let print_text = args.iter().any(|arg| arg == "--print-text");
    let threads = args
        .iter()
        .position(|arg| arg == "--threads")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse().ok())
        .unwrap_or(3);
    let positional: Vec<&String> = args
        .iter()
        .enumerate()
        .filter(|(index, arg)| {
            !arg.starts_with("--") && (*index == 0 || args[index - 1] != "--threads")
        })
        .map(|(_, arg)| arg)
        .collect();
    if positional.len() < 2 {
        eprintln!("usage: llama_probe <gguf> <locale> [--threads N] [--print-text]");
        std::process::exit(64);
    }
    let language = Language::from_locale(positional[1]).unwrap_or_else(|| {
        eprintln!("unknown locale");
        std::process::exit(64);
    });
    let pack = FrameworkPack::for_language(language).expect("pack");
    let started = Instant::now();
    let mut backend = LlamaBackend::load(std::path::Path::new(positional[0]), threads)
        .unwrap_or_else(|error| {
            eprintln!("error: {}", error.message);
            std::process::exit(2);
        });
    let load_ms = started.elapsed().as_secs_f64() * 1000.0;
    let prompt = render_easylanguage_prompt(&pack, SENTENCE);
    let raw = backend
        .generate(&prompt, MAX_NEW_TOKENS)
        .unwrap_or_else(|error| {
            eprintln!("error: {}", error.message);
            std::process::exit(2);
        });
    let stats = backend.last_stats().clone();
    let stripped = strip_think_block(&raw);
    let decode_s = ((stats.total_ms - stats.ttft_ms) / 1000.0).max(1e-9);
    println!("crate=llama-cpp-2 0.1.161 (CPU, openmp, no common)");
    println!("threads={threads}");
    println!("load_ms={load_ms:.0}");
    println!("prompt_tokens={}", stats.prompt_tokens);
    println!("ttft_ms={:.0}", stats.ttft_ms);
    println!("generated_tokens={}", stats.generated_tokens);
    println!(
        "tokens_per_s={:.1}",
        stats.generated_tokens as f64 / decode_s
    );
    println!("total_ms={:.0}", stats.total_ms);
    println!(
        "think_left_after_strip={}",
        stripped.contains("<think>") || stripped.contains("</think>")
    );
    if print_text {
        println!("raw={raw:?}");
        println!("output={stripped}");
    }
}
