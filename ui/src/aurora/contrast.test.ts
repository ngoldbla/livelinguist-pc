import { describe, expect, test } from "bun:test";
import { PRIMARY_FILL, toneStyles } from "./components";
import { colors } from "./theme";

// WCAG 2.x relative luminance and contrast ratio. Colors with alpha are
// composited over the background they sit on.
function channels(hex: string): [number, number, number, number] {
  const value = hex.replace("#", "");
  const at = (index: number) => Number.parseInt(value.slice(index, index + 2), 16);
  const alpha = value.length >= 8 ? at(6) / 255 : 1;
  return [at(0), at(2), at(4), alpha];
}

export function over(top: string, bottom: string): string {
  const [r, g, b, a] = channels(top);
  const [br, bg, bb] = channels(bottom);
  const mix = (f: number, k: number) =>
    Math.round(f * a + k * (1 - a))
      .toString(16)
      .padStart(2, "0");
  return `#${mix(r, br)}${mix(g, bg)}${mix(b, bb)}`;
}

function luminance(hex: string): number {
  const [r, g, b] = channels(hex)
    .slice(0, 3)
    .map((c) => {
      const s = c / 255;
      return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    }) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export function contrast(foreground: string, background: string): number {
  const fg = luminance(foreground);
  const bg = luminance(background);
  return (Math.max(fg, bg) + 0.05) / (Math.min(fg, bg) + 0.05);
}

const BODY = 4.5;
const LARGE = 3;

// Every opaque background text sits on: the window field, glass surfaces
// (composited over the field), and raised cards.
const backgrounds: Record<string, string> = {
  bg: colors.bg,
  surface: over(colors.surface, colors.bg),
  surfaceSolid: colors.surfaceSolid,
  surfaceRaised: colors.surfaceRaised,
};

// Text tokens used at body sizes (< 18 px) somewhere in the UI.
const bodyText: Record<string, string> = {
  text: colors.text,
  muted: colors.muted,
  faint: colors.faint,
  simplified: "#E4DFFF",
  amber: colors.amber,
  mint: colors.mint,
};

describe("WCAG contrast", () => {
  test("helpers match known values", () => {
    expect(contrast("#FFFFFF", "#000000")).toBeCloseTo(21, 5);
    expect(contrast("#777777", "#FFFFFF")).toBeCloseTo(4.48, 1);
  });

  for (const [name, color] of Object.entries(bodyText)) {
    for (const [bgName, bg] of Object.entries(backgrounds)) {
      test(`${name} on ${bgName} >= 4.5:1`, () => {
        expect(contrast(color, bg)).toBeGreaterThanOrEqual(BODY);
      });
    }
  }

  for (const [tone, palette] of Object.entries(toneStyles)) {
    for (const [bgName, bg] of Object.entries(backgrounds)) {
      test(`badge ${tone} text on its fill over ${bgName} >= 4.5:1`, () => {
        expect(contrast(palette.fg, over(palette.bg, bg))).toBeGreaterThanOrEqual(BODY);
      });
    }
  }

  test("primary button label on every fill >= 4.5:1", () => {
    for (const stop of Object.values(PRIMARY_FILL)) {
      expect(contrast("#FFFFFF", stop)).toBeGreaterThanOrEqual(BODY);
    }
  });

  test("danger button label >= 4.5:1", () => {
    for (const bg of Object.values(backgrounds)) {
      expect(contrast("#FFB5C1", over(colors.roseSoft, bg))).toBeGreaterThanOrEqual(BODY);
    }
  });

  test("large text (>= 18 px) and focus ring >= 3:1", () => {
    for (const bg of Object.values(backgrounds)) {
      expect(contrast(colors.accent, bg)).toBeGreaterThanOrEqual(LARGE);
      expect(contrast(colors.rose, bg)).toBeGreaterThanOrEqual(LARGE);
    }
  });
});
