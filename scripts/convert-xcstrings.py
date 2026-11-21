#!/usr/bin/env python3
"""Convert the Mac iOS Localizable.xcstrings into assets/locales/<code>.json.

Usage:
  python3 scripts/convert-xcstrings.py /path/to/Localizable.xcstrings

The xcstrings keys are the English source strings; the desktop UI uses the same
keys through `t(key, ...)`. Placeholders `%@`/`%lld` are kept verbatim and
formatted positionally by the UI. English is the fallback for every locale.
"""
import json
import sys
from pathlib import Path

DESKTOP_KEYS = [
    "CONSOLE", "CAPTION OVERLAY", "EXPANDED", "Developer", "Runtime health",
    "Window mode", "Console window", "Caption overlay", "Expanded transcript",
    "On-demand", "Live", "ASR status", "Simplify this sentence", "Read latest",
    "Confidence", "Flagged", "Clean", "Verbatim", "Memory-only",
    "No session data", "Demo mode", "Download models", "Search languages",
    "Model", "Sound", "Learning language", "Start", "Paused", "Preparing session",
    "Ready to caption", "Summary", "DURATION", "SENTENCES",
    "IN TARGET LANGUAGE", "ENGLISH REVERSIONS", "Text size", "Caption size",
    "Diagnostics CSV", "Plain text", "Markdown", "Window", "Back", "Continue",
    "Skip", "Done", "Default", "Font", "Open", "Save", "Copy diagnostics",
    "Reset overrides", "Probe backends", "Run calibration", "Reload model",
    "Clear KV cache", "Strict mode", "Fault injection", "None", "Overrun",
    "Timeout", "Error", "OOM", "Mock pipeline", "Tier C", "Stress",
    "ASR only", "LLM only", "Simplify-only", "CPU only", "No GPU",
    "Vulkan only", "CUDA only", "Scenario", "Threads", "Backend", "Device",
    "GPU layers", "Model hash", "TTFT", "Tokens/s", "Event seq", "Dropped",
    "Last event age", "Strict", "Effective", "Requested", "Downloads",
    "Model download pending", "Install the models from setup to enable real captioning.",
    "Everything runs offline after the one-time download.",
    "Audio and transcripts stay on this device.",
    "The app contacts Hugging Face only when you start a model download.",
    "Start speaking, or use the demo conversation.",
    "Click a sentence to simplify it.",
    "Export transcript", "About", "Version", "Built for Windows 11 x64",
    "Linux dev preview", "Hugging Face", "llama.cpp", "sherpa-onnx", "GPUIX",
    "Framework packs", "Qwen3", "Nemotron", "Apache-2.0", "MIT",
    "Same language, not translation", "29 locales",
    "Live listening is not available for this language yet. Paste or type text to simplify it.",
]

def main():
    source = Path(sys.argv[1])
    data = json.loads(source.read_text())
    strings = data.get("strings", {})
    out = Path("assets/locales")
    out.mkdir(parents=True, exist_ok=True)

    english = {key: key for key in strings}
    for key in DESKTOP_KEYS:
        english.setdefault(key, key)
    # A few desktop keys have nicer copy than the key itself.
    overrides = {
        "CONSOLE": "Console",
        "CAPTION OVERLAY": "Caption overlay",
        "EXPANDED": "Expanded transcript",
        "DURATION": "DURATION",
        "SENTENCES": "SENTENCES",
        "IN TARGET LANGUAGE": "IN TARGET LANGUAGE",
        "ENGLISH REVERSIONS": "ENGLISH REVERSIONS",
    }
    english.update(overrides)
    (out / "en.json").write_text(json.dumps(english, ensure_ascii=False, indent=2, sort_keys=True) + "\n")

    codes = set(strings[next(iter(strings))].get("localizations", {}).keys())
    for code in sorted(codes):
        locale = {}
        for key, entry in strings.items():
            value = entry.get("localizations", {}).get(code, {}).get("stringUnit", {}).get("value")
            if value:
                locale[key] = value
        # Desktop-only keys fall back to English.
        for key in DESKTOP_KEYS:
            locale.setdefault(key, english[key])
        (out / f"{code}.json").write_text(json.dumps(locale, ensure_ascii=False, indent=2, sort_keys=True) + "\n")
    print(f"wrote en + {len(codes)} locales, {len(english)} keys each")

if __name__ == "__main__":
    main()
