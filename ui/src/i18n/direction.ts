// RTL helpers. GPUIX 0.11 has no CSS direction primitive, so rows are mirrored
// explicitly and text alignment is set per text node.
export type Direction = "ltr" | "rtl";

export function directionFor(locale: string): Direction {
  return /^(ar|he|fa|ur)(-|$)/.test(locale) ? "rtl" : "ltr";
}

export function rowDirection(locale: string): "row" | "row-reverse" {
  return directionFor(locale) === "rtl" ? "row-reverse" : "row";
}

export function textAlign(locale: string): "left" | "right" {
  return directionFor(locale) === "rtl" ? "right" : "left";
}
