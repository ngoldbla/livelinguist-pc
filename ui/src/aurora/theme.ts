import type { StyleDesc } from "@gpuix/react";

/**
 * Quiet utility palette. One accent, three semantic tones, no decorative
 * gradients: text sits on flat near-black surfaces and every body token keeps
 * >= 4.5:1 against them (see contrast.test.ts).
 */
export const colors = {
  bg: "#0A0A0D",
  bgDeep: "#070709",
  surface: "#111116",
  surfaceSolid: "#111116",
  surfaceRaised: "#18181F",
  border: "#FFFFFF1A",
  borderStrong: "#FFFFFF2E",
  text: "#F4F4F7",
  muted: "#C6C7D0",
  faint: "#A0A1AC",
  accent: "#AFA2FF",
  accentSoft: "#AFA2FF1F",
  mint: "#5FE3C0",
  mintSoft: "#5FE3C01F",
  amber: "#F5C06A",
  amberSoft: "#F5C06A1F",
  rose: "#FF8A9B",
  roseSoft: "#FF8A9B1F",
} as const;

/**
 * One type scale. Nothing renders below 12px, so metadata stays readable for
 * the people the product is for (easy-language readers, low vision).
 */
export const type = {
  meta: 12,
  label: 13,
  body: 14,
  title: 20,
  page: 28,
  hero: 34,
} as const;

const isWindows = typeof process !== "undefined" && process.platform === "win32";

export const fonts = {
  sans: isWindows ? "Segoe UI" : "Noto Sans",
  display: isWindows ? "Segoe UI" : "Noto Sans",
  arabic: "Noto Sans Arabic",
  mono: isWindows ? "Consolas" : "Noto Sans Mono",
} as const;

export function gradient(from: string, to: string, angle = 120): StyleDesc["background"] {
  return {
    type: "linear-gradient",
    angle,
    stops: [
      { color: from, position: 0 },
      { color: to, position: 1 },
    ],
    colorSpace: "oklab",
  };
}

/** Flat panel. No blur, no drop shadow: the surface is the elevation. */
export const glass: StyleDesc = {
  backgroundColor: colors.surface,
  borderWidth: 1,
  borderColor: colors.border,
  borderRadius: 12,
};

/** A panel nested inside a Surface. */
export const card: StyleDesc = {
  backgroundColor: "#FFFFFF06",
  borderWidth: 1,
  borderColor: colors.border,
  borderRadius: 10,
};

export function textStyle(
  size: number = type.body,
  color: string = colors.text,
  weight?: number,
): StyleDesc {
  return {
    fontSize: size,
    color,
    ...(weight ? { fontWeight: weight } : {}),
    fontFamily: fonts.sans,
  };
}

export const focusRing: StyleDesc = {
  outlineWidth: 2,
  outlineColor: colors.accent,
  outlineOffset: 2,
};

export const pressable: StyleDesc = {
  borderRadius: 8,
  cursor: "pointer",
  hover: { backgroundColor: "#FFFFFF10" },
  active: { backgroundColor: "#FFFFFF18" },
};
