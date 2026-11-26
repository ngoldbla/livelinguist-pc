// Regenerate src/aurora/iconData.ts from the vendored Lucide SVGs.
// Add or remove an SVG in assets/icons, then run: bun scripts/sync-icons.mjs
import fs from "node:fs";
import path from "node:path";

const assets = path.join(import.meta.dir, "..", "assets", "icons");
const output = path.join(import.meta.dir, "..", "src", "aurora", "iconData.ts");

const names = fs
  .readdirSync(assets)
  .filter((name) => name.endsWith(".svg"))
  .map((name) => name.replace(/\.svg$/, ""))
  .sort();

const entries = names.map((name) => {
  const svg = fs
    .readFileSync(path.join(assets, `${name}.svg`), "utf8")
    .replace(/\s+/g, " ")
    .replace(/>\s+</g, "><")
    .trim();
  return `  "${name}": \`${svg}\`,`;
});

const out = `// Lucide icons, ISC licence (https://lucide.dev). Vendored as raw SVG so
// GPUIX can render them with <svg source>. Regenerate with
// \`bun scripts/sync-icons.mjs\`; do not hand-edit the path data.
export const ICON_PATHS = {
${entries.join("\n")}
} as const;

export type IconName = keyof typeof ICON_PATHS;
`;

fs.writeFileSync(output, out, "utf8");
console.log(`wrote ${names.length} icons to ${path.relative(process.cwd(), output)}`);
