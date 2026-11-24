#!/usr/bin/env python3
"""Fetch the FLEURS test clips listed in tests/asr-eval/manifest.json (T29).

Usage (run with `python -I`):
  python -I scripts/asr-eval/fetch_fleurs.py --cache target/fleurs [--locales en-US,fr-FR]
  python -I scripts/asr-eval/fetch_fleurs.py --cache target/fleurs --build-manifest

FLEURS (google/fleurs) is CC-BY-4.0. Audio goes only into --cache and is never
committed. Tarballs are streamed: only the chosen clips are written, one locale
at a time, and nothing else is kept on disk.

--build-manifest picks test clips in TSV order until 300 s and writes ids,
file names, durations, SHA-256 and the normalized reference transcription.
Without it, the clips in the manifest are fetched (or reused) and every
SHA-256 is verified. Stdlib only.
"""
import argparse
import csv
import hashlib
import json
import os
import sys
import tarfile
import urllib.request

DATASET = "google/fleurs"
REVISION = "70bb2e84b976b7e960aa89f1c648e09c59f894dd"
TARGET_SECONDS = 300.0
SAMPLE_RATE = 16000
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
MANIFEST = os.path.join(ROOT, "tests", "asr-eval", "manifest.json")

# App locale -> FLEURS code, with notes for dialect mismatches (T29 card).
LOCALES = [
    ("de-DE", "de_de", None),
    ("fr-FR", "fr_fr", None),
    ("es-ES", "es_419", "dialect mismatch: Latin-American Spanish recordings"),
    ("en-US", "en_us", None),
    ("ar-SA", "ar_eg", "dialect mismatch: Egyptian speakers reading MSA prompts"),
    ("da-DK", "da_dk", None),
    ("et-EE", "et_ee", None),
    ("fi-FI", "fi_fi", None),
    ("hi-IN", "hi_in", None),
    ("it-IT", "it_it", None),
    ("ja-JP", "ja_jp", "scored with CER"),
    ("ko-KR", "ko_kr", "scored with CER"),
    ("nl-NL", "nl_nl", None),
    ("pt-BR", "pt_br", None),
    ("pt-PT", "pt_br", "mismatch: no pt_pt in FLEURS; Brazilian recordings"),
    ("ru-RU", "ru_ru", None),
    ("sk-SK", "sk_sk", None),
    ("sv-SE", "sv_se", None),
    ("tr-TR", "tr_tr", None),
    ("vi-VN", "vi_vn", None),
    ("zh-CN", "cmn_hans_cn", "scored with CER"),
    ("uk-UA", "uk_ua", None),
    ("ro-RO", "ro_ro", None),
    ("bg-BG", "bg_bg", None),
    ("pl-PL", "pl_pl", None),
    ("hr-HR", "hr_hr", None),
    ("nb-NO", "nb_no", None),
    ("hu-HU", "hu_hu", None),
    ("cs-CZ", "cs_cz", None),
]


def url(path):
    return f"https://huggingface.co/datasets/{DATASET}/resolve/{REVISION}/{path}"


def open_url(path):
    request = urllib.request.Request(url(path), headers={"User-Agent": "livelinguist-asr-eval"})
    return urllib.request.urlopen(request, timeout=120)


def dataset_codes():
    api = f"https://huggingface.co/api/datasets/{DATASET}/tree/{REVISION}/data"
    request = urllib.request.Request(api, headers={"User-Agent": "livelinguist-asr-eval"})
    with urllib.request.urlopen(request, timeout=60) as response:
        return {entry["path"].split("/")[-1] for entry in json.load(response)}


def read_tsv(code):
    with open_url(f"data/{code}/test.tsv") as response:
        text = response.read().decode("utf-8")
    rows = []
    for row in csv.reader(text.splitlines(), delimiter="\t", quoting=csv.QUOTE_NONE):
        # id, file, raw transcription, normalized transcription, chars,
        # num_samples, gender (FLEURS TSVs have no header row).
        if len(row) < 6 or not row[5].isdigit():
            continue
        rows.append({"id": row[0], "file": row[1], "raw": row[2], "normalized": row[3],
                     "samples": int(row[5])})
    return rows


def sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def stream_extract(code, wanted, out_dir):
    """Stream data/<code>/audio/test.tar.gz and write only `wanted` files."""
    os.makedirs(out_dir, exist_ok=True)
    remaining = set(wanted)
    with open_url(f"data/{code}/audio/test.tar.gz") as response:
        with tarfile.open(fileobj=response, mode="r|gz") as tar:
            for member in tar:
                name = member.name.rsplit("/", 1)[-1]
                if not member.isfile() or name not in remaining:
                    continue
                data = tar.extractfile(member).read()
                with open(os.path.join(out_dir, name), "wb") as handle:
                    handle.write(data)
                remaining.discard(name)
                if not remaining:
                    break
    return remaining


def build(cache, locales):
    codes = dataset_codes()
    manifest = {
        "dataset": DATASET,
        "revision": REVISION,
        "split": "test",
        "license": "CC-BY-4.0",
        "attribution": "FLEURS (Conneau et al., 2022), Google, CC-BY-4.0",
        "target_seconds": TARGET_SECONDS,
        "locales": [],
    }
    for locale, code, note in LOCALES:
        if locales and locale not in locales:
            continue
        entry = {"locale": locale, "fleurs": code, "note": note}
        if code not in codes:
            entry.update({"status": "no_fixture", "clips": []})
            manifest["locales"].append(entry)
            print(f"{locale}: no_fixture ({code} missing)", flush=True)
            continue
        chosen, total = [], 0.0
        for row in read_tsv(code):
            if total >= TARGET_SECONDS:
                break
            seconds = row["samples"] / SAMPLE_RATE
            chosen.append({"id": row["id"], "file": row["file"], "seconds": round(seconds, 3),
                           "reference": row["normalized"] or row["raw"]})
            total += seconds
        out_dir = os.path.join(cache, locale)
        missing = stream_extract(code, [clip["file"] for clip in chosen], out_dir)
        if missing:
            raise SystemExit(f"{locale}: {len(missing)} clips not found in the tarball")
        for clip in chosen:
            clip["sha256"] = sha256(os.path.join(out_dir, clip["file"]))
        entry.update({"status": "fixture", "seconds": round(total, 1), "clips": chosen})
        manifest["locales"].append(entry)
        print(f"{locale}: {len(chosen)} clips, {total:.0f} s", flush=True)
    return manifest


def fetch(cache, locales):
    with open(MANIFEST, encoding="utf-8") as handle:
        manifest = json.load(handle)
    if manifest.get("revision") != REVISION:
        raise SystemExit("manifest revision differs from the script pin")
    failures = 0
    for entry in manifest["locales"]:
        if locales and entry["locale"] not in locales:
            continue
        if entry["status"] != "fixture":
            print(f"{entry['locale']}: {entry['status']}")
            continue
        out_dir = os.path.join(cache, entry["locale"])
        need = [clip["file"] for clip in entry["clips"]
                if not os.path.isfile(os.path.join(out_dir, clip["file"]))]
        if need:
            missing = stream_extract(entry["fleurs"], need, out_dir)
            failures += len(missing)
        bad = 0
        for clip in entry["clips"]:
            path = os.path.join(out_dir, clip["file"])
            if not os.path.isfile(path) or sha256(path) != clip["sha256"]:
                bad += 1
        failures += bad
        state = "ok" if bad == 0 else f"{bad} hash mismatches"
        print(f"{entry['locale']}: {len(entry['clips'])} clips verified, {state}", flush=True)
    return failures


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--cache", required=True)
    parser.add_argument("--locales", default="")
    parser.add_argument("--build-manifest", action="store_true")
    args = parser.parse_args()
    locales = {item.strip() for item in args.locales.split(",") if item.strip()}
    if args.build_manifest:
        manifest = build(args.cache, locales)
        if locales and os.path.isfile(MANIFEST):
            with open(MANIFEST, encoding="utf-8") as handle:
                old = json.load(handle)
            keep = [entry for entry in old["locales"] if entry["locale"] not in locales]
            order = [locale for locale, _, _ in LOCALES]
            manifest["locales"] = sorted(keep + manifest["locales"],
                                         key=lambda entry: order.index(entry["locale"]))
        with open(MANIFEST, "w", encoding="utf-8") as handle:
            json.dump(manifest, handle, ensure_ascii=False, indent=1)
            handle.write("\n")
        return 0
    return 1 if fetch(args.cache, locales) else 0


if __name__ == "__main__":
    sys.exit(main())
