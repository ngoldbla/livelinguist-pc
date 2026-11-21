#!/usr/bin/env python3
"""Add every `t(locale, "key")` used by the UI to all 29 locale catalogs.

New desktop keys fall back to their English source; translated keys from the
Mac Localizable.xcstrings are never overwritten.
"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "ui" / "src"
LOCALES = ROOT / "assets" / "locales"

pattern = re.compile(r'\bt\(\s*[^,]+,\s*"((?:[^"\\]|\\.)*)"')
keys = set()
for path in UI.rglob("*"):
    if not path.is_file() or path.suffix not in (".ts", ".tsx") or "generated" in path.parts:
        continue
    text = path.read_text()
    keys.update(pattern.findall(text))

english = json.loads((LOCALES / "en.json").read_text())
added = sorted(key for key in keys if key not in english)
for key in added:
    english[key] = key
(LOCALES / "en.json").write_text(json.dumps(english, ensure_ascii=False, indent=2, sort_keys=True) + "\n")

for path in LOCALES.glob("*.json"):
    if path.name == "en.json":
        continue
    catalog = json.loads(path.read_text())
    changed = False
    for key, value in english.items():
        if key not in catalog:
            catalog[key] = value
            changed = True
    if changed:
        path.write_text(json.dumps(catalog, ensure_ascii=False, indent=2, sort_keys=True) + "\n")

print(f"catalogs: {len(list(LOCALES.glob('*.json')))}; keys: {len(english)}; added: {len(added)}")
for key in added:
    print(" +", key)
