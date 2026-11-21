import { describe, expect, test } from "bun:test";
import { AVAILABLE_LOCALES, catalogs, isRtl, resolveLocale, t } from "./locales";

describe("locale catalogs", () => {
  test("29 locales ship with the full key set", () => {
    expect(AVAILABLE_LOCALES.length).toBe(29);
    const english = catalogs.en!;
    for (const code of AVAILABLE_LOCALES) {
      const catalog = catalogs[code];
      expect(catalog).toBeDefined();
      expect(Object.keys(catalog!)).toHaveLength(Object.keys(english).length);
    }
  });

  test("translations are used and unknown keys fall back to English", () => {
    expect(t("de", "Welcome to Live Linguist")).toBe("Willkommen bei Live Linguist");
    expect(t("fr", "Settings")).toBe("Réglages");
    expect(t("zz", "Welcome to Live Linguist")).toBe("Welcome to Live Linguist");
  });

  test("positional placeholders and percent escapes", () => {
    expect(t("en", "Welcome to Live Linguist")).toBe("Welcome to Live Linguist");
    expect(
      t(
        "en",
        "Live Linguist speaks %lld languages — captions, simplification, and the app itself.",
        29,
      ),
    ).toContain("29");
    expect(t("en", "Downloading models · %lld%%", 42)).toBe("Downloading models · 42%");
  });

  test("resolution follows learning language then device then English", () => {
    expect(resolveLocale(true, "ko", ["en"])).toBe("ko");
    expect(resolveLocale(false, "ko", ["fr"])).toBe("fr");
    expect(resolveLocale(true, null, ["de-DE"])).toBe("de");
    expect(resolveLocale(true, "xx", ["fr"])).toBe("fr");
    expect(resolveLocale(true, null, ["xx-XX"])).toBe("en");
  });

  test("Arabic is RTL", () => {
    expect(isRtl("ar")).toBe(true);
    expect(isRtl("ar-SA")).toBe(true);
    expect(isRtl("ko")).toBe(false);
    expect(isRtl("en")).toBe(false);
  });
});
