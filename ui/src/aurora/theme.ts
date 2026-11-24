import type { StyleDesc } from "@gpuix/react";

export const colors = {
  bg: "#0B0B12",
  bgDeep: "#07070C",
  surface: "#151522E6",
  surfaceSolid: "#151522",
  surfaceRaised: "#1D1D2E",
  border: "#FFFFFF14",
  borderStrong: "#FFFFFF24",
  text: "#F5F3FF",
  muted: "#A8A3BD",
  // Lightened from #6F6A85 to reach 4.5:1 on every surface (contrast test).
  faint: "#87829D",
  accent: "#8B7CFF",
  accentSoft: "#8B7CFF24",
  mint: "#59E1C8",
  mintSoft: "#59E1C824",
  amber: "#FFC46B",
  amberSoft: "#FFC46B24",
  rose: "#FF7A90",
  roseSoft: "#FF7A9024",
} as const;

export const fonts = {
  sans: "Noto Sans",
  display: "Noto Sans",
  arabic: "Noto Sans Arabic",
  mono: "Noto Sans Mono",
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

export const glass: StyleDesc = {
  backgroundColor: colors.surface,
  borderWidth: 1,
  borderColor: colors.border,
  borderRadius: 18,
  boxShadow: {
    offsetX: 0,
    offsetY: 12,
    blurRadius: 36,
    spreadRadius: 0,
    color: "#00000055",
  },
};

export const card: StyleDesc = {
  backgroundColor: colors.surfaceRaised,
  borderWidth: 1,
  borderColor: colors.border,
  borderRadius: 14,
};

export function textStyle(size = 14, color: string = colors.text, weight?: number): StyleDesc {
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
  borderRadius: 12,
  cursor: "pointer",
  hover: { backgroundColor: "#FFFFFF10" },
  active: { backgroundColor: "#FFFFFF18" },
};
