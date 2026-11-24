import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { LANGUAGES } from "./languages";

const coverageRs = readFileSync(
  join(import.meta.dir, "../../../core/crates/ll-core/src/coverage.rs"),
  "utf8",
);

/// Rust variant (`EnUs`) -> locale (`en-US`).
function locale(variant: string): string {
  return `${variant.slice(0, 2).toLowerCase()}-${variant.slice(2).toUpperCase()}`;
}

/// The `MEASURED` rows of `CoverageMatrix::v1()`, as locale -> UI status.
function rustMatrix(): Map<string, string> {
  const rows = new Map<string, string>();
  const pattern = /\(Language::(\w+), (Ok|SimplifyOnly|Degraded|Tbd), "/g;
  for (const match of coverageRs.matchAll(pattern)) {
    const status = { Ok: "ok", SimplifyOnly: "simplify_only", Degraded: "degraded", Tbd: "tbd" }[
      match[2] as "Ok"
    ];
    rows.set(locale(match[1] as string), status);
  }
  return rows;
}

describe("ASR coverage", () => {
  test("the UI table matches CoverageMatrix::v1() row for row", () => {
    const rust = rustMatrix();
    expect(rust.size).toBe(29);
    expect(LANGUAGES).toHaveLength(29);
    for (const language of LANGUAGES) {
      expect([language.code, language.asrStatus]).toEqual([
        language.code,
        rust.get(language.code) as typeof language.asrStatus,
      ]);
    }
  });

  test("no language is left unmeasured", () => {
    expect(LANGUAGES.filter((language) => language.asrStatus === "tbd")).toEqual([]);
  });

  test("the minimum live set can listen", () => {
    for (const code of ["en-US", "fr-FR", "de-DE", "es-ES", "ar-SA"]) {
      expect(LANGUAGES.find((language) => language.code === code)?.asrStatus).toBe("ok");
    }
  });
});
