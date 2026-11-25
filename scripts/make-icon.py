"""Generate packaging/icons/livelinguist.ico (app, installer, shortcuts).

A speech bubble with two caption lines on the Aurora accent gradient. Drawn
from shapes only (no fonts), so the output is reproducible:
    python3 scripts/make-icon.py
"""

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "packaging" / "icons" / "livelinguist.ico"
S = 1024  # master size, downsampled per icon size
ACCENT = (0x8B, 0x7C, 0xFF)
MINT = (0x59, 0xE1, 0xC8)
INK = (0x0B, 0x0B, 0x12)


def master() -> Image.Image:
    # Diagonal accent -> mint gradient.
    grad = Image.new("RGB", (S, S))
    px = grad.load()
    for y in range(S):
        for x in range(S):
            t = (x + y) / (2 * (S - 1))
            px[x, y] = tuple(round(a + (b - a) * t) for a, b in zip(ACCENT, MINT))
    mask = Image.new("L", (S, S), 0)
    ImageDraw.Draw(mask).rounded_rectangle((32, 32, S - 32, S - 32), radius=220, fill=255)
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    img.paste(grad, (0, 0), mask)

    d = ImageDraw.Draw(img)
    # Speech bubble with a tail at the lower left.
    d.rounded_rectangle((180, 230, S - 180, 700), radius=110, fill=(255, 255, 255, 255))
    d.polygon([(300, 660), (290, 860), (470, 690)], fill=(255, 255, 255, 255))
    # Two caption lines: the long verbatim line and the short simplified one.
    d.rounded_rectangle((290, 360, S - 290, 430), radius=35, fill=INK + (255,))
    d.rounded_rectangle((290, 500, 600, 570), radius=35, fill=ACCENT + (255,))
    return img


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    sizes = [(n, n) for n in (16, 20, 24, 32, 40, 48, 64, 128, 256)]
    master().save(OUT, format="ICO", sizes=sizes)
    print(f"wrote {OUT.relative_to(ROOT)} ({OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
