# Icons

[Lucide](https://lucide.dev) SVGs, ISC licensed (`LICENSE.txt`). Copied from
`lucide-icons/lucide` (`main`) on 2026-11-25.

`src/aurora/iconData.ts` is generated from these files:

```bash
bun scripts/sync-icons.mjs
```

To add an icon, drop the Lucide SVG in this folder and rerun the script. The
runtime renders the embedded path with GPUIX's `<svg source>` and tints it by
replacing `currentColor`; no icon font or network fetch is involved.
