//! sherpa-onnx streaming probe for the pinned Nemotron 3.5 ASR model (T12).
//!
//! Usage: `sherpa_probe <asr model dir> <wav> [language]`
//! Prints load time, audio seconds, elapsed decode time, and RTF. The
//! recognized text goes to stdout only when `--print-text` is passed, so
//! evidence runs stay content-free.

use ll_asr::sherpa_recognizer_config;
use sherpa_onnx::OnlineRecognizer;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let print_text = args.iter().any(|arg| arg == "--print-text");
    let positional: Vec<&String> = args.iter().filter(|arg| !arg.starts_with("--")).collect();
    if positional.len() < 2 {
        eprintln!("usage: sherpa_probe <asr model dir> <wav> [language] [--print-text]");
        std::process::exit(64);
    }
    let model_dir = std::path::Path::new(positional[0]);
    let wav_path = std::path::Path::new(positional[1]);
    let language = positional.get(2).map(|value| value.as_str());

    let started = Instant::now();
    let config = sherpa_recognizer_config(model_dir, 2);
    let Some(recognizer) = OnlineRecognizer::create(&config) else {
        eprintln!(
            "error: sherpa-onnx could not create the recognizer from {}",
            model_dir.display()
        );
        std::process::exit(2);
    };
    let load_ms = started.elapsed().as_secs_f64() * 1000.0;

    let bytes = std::fs::read(wav_path).unwrap_or_else(|error| {
        eprintln!("error: read {}: {error}", wav_path.display());
        std::process::exit(2);
    });
    let samples = ll_audio::decode_wav(&bytes)
        .unwrap_or_else(|error| {
            eprintln!("error: decode wav: {error}");
            std::process::exit(2);
        })
        .mono_16k();
    let audio_s = samples.len() as f64 / 16_000.0;

    let stream = recognizer.create_stream();
    if let Some(language) = language {
        stream.set_option("language", language);
    }
    let decode_started = Instant::now();
    let mut endpoints = 0;
    let mut text = String::new();
    for piece in samples.chunks(8_960) {
        stream.accept_waveform(16_000, piece);
        while recognizer.is_ready(&stream) {
            recognizer.decode(&stream);
        }
        if recognizer.is_endpoint(&stream) {
            if let Some(result) = recognizer.get_result(&stream) {
                push(&mut text, &result.text);
            }
            recognizer.reset(&stream);
            endpoints += 1;
        }
    }
    stream.input_finished();
    while recognizer.is_ready(&stream) {
        recognizer.decode(&stream);
    }
    if let Some(result) = recognizer.get_result(&stream) {
        push(&mut text, &result.text);
    }
    let elapsed_s = decode_started.elapsed().as_secs_f64();
    println!("crate=sherpa-onnx 2.0.1");
    println!("load_ms={load_ms:.0}");
    println!("audio_s={audio_s:.2}");
    println!("elapsed_s={elapsed_s:.2}");
    println!("rtf={:.3}", elapsed_s / audio_s.max(1e-9));
    println!("endpoints={endpoints}");
    println!("words={}", text.split_whitespace().count());
    if print_text {
        println!("text={text}");
    }
}

fn push(text: &mut String, piece: &str) {
    let piece = piece.trim();
    if piece.is_empty() {
        return;
    }
    if !text.is_empty() {
        text.push(' ');
    }
    text.push_str(piece);
}
