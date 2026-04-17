#!/usr/bin/env python3
"""Generate Windows icon.ico — white rounded square with a centered black '#'.

Mirrors the simple macOS look. Each size is drawn independently so small
sizes (16/32) stay crisp instead of being downsampled from a 256px master.
"""

import os
from PIL import Image, ImageDraw, ImageFont

SIZES = [16, 32, 48, 64, 128, 256]
ASSETS_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets")
ICO_PATH = os.path.join(ASSETS_DIR, "icon.ico")


def find_bold_font() -> str | None:
    candidates = [
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/Library/Fonts/Arial Bold.ttf",
        "/System/Library/Fonts/SFNSDisplayCondensed-Bold.otf",
        "C:\\Windows\\Fonts\\arialbd.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVu-Sans-Bold.ttf",
    ]
    return next((p for p in candidates if os.path.exists(p)), None)


def draw(size: int, font_path: str | None) -> Image.Image:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    # Rounded-rect white background with iOS-style corner ratio.
    margin = max(1, round(size * 0.06))
    corner = round(size * 0.22)
    rect = [margin, margin, size - margin - 1, size - margin - 1]
    d.rounded_rectangle(rect, radius=corner, fill=(255, 255, 255, 255))

    # Centered bold '#'.
    font_size = max(8, round(size * 0.62))
    try:
        font = ImageFont.truetype(font_path, font_size) if font_path else ImageFont.load_default()
    except Exception:
        font = ImageFont.load_default()

    text = "#"
    bbox = d.textbbox((0, 0), text, font=font)
    tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
    tx = (size - tw) / 2 - bbox[0]
    ty = (size - th) / 2 - bbox[1]
    d.text((tx, ty), text, fill=(0, 0, 0, 255), font=font)
    return img


def main():
    os.makedirs(ASSETS_DIR, exist_ok=True)
    font_path = find_bold_font()
    images = [draw(s, font_path) for s in SIZES]

    # Pillow ICO writer: put the largest as primary, rest via append_images,
    # preserving per-size independent renderings (no downsample).
    images[-1].save(
        ICO_PATH,
        format="ICO",
        append_images=images[:-1],
        sizes=[(s, s) for s in SIZES],
    )
    print(f"  icon.ico -> {ICO_PATH}")
    for s in SIZES:
        print(f"    {s}x{s}")


if __name__ == "__main__":
    main()
