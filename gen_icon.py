#!/usr/bin/env python3
"""Generate a macOS .icns icon for md-preview."""

import os
import subprocess
import tempfile
from PIL import Image, ImageDraw, ImageFont

SIZES = [16, 32, 64, 128, 256, 512, 1024]
OUTPUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets")
ICONSET_DIR = os.path.join(OUTPUT_DIR, "icon.iconset")


def draw_icon(size: int) -> Image.Image:
    """Draw the md-preview icon at the given size."""
    scale = size / 1024.0
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # --- Rounded rectangle background with gradient ---
    margin = int(60 * scale)
    corner_radius = int(180 * scale)
    rect = [margin, margin, size - margin, size - margin]

    # Draw gradient by layering horizontal lines
    top_color = (74, 144, 217)      # #4A90D9
    bot_color = (53, 122, 189)      # #357ABD
    for y in range(rect[1], rect[3]):
        t = (y - rect[1]) / max(1, rect[3] - rect[1] - 1)
        r = int(top_color[0] + (bot_color[0] - top_color[0]) * t)
        g = int(top_color[1] + (bot_color[1] - top_color[1]) * t)
        b = int(top_color[2] + (bot_color[2] - top_color[2]) * t)
        draw.line([(rect[0], y), (rect[2], y)], fill=(r, g, b, 255))

    # Mask to rounded rectangle
    mask = Image.new("L", (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle(rect, radius=corner_radius, fill=255)
    # Apply mask: keep only pixels inside the rounded rect
    bg = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    img = Image.composite(img, bg, mask)
    draw = ImageDraw.Draw(img)

    # --- Drop shadow (subtle) ---
    # We skip complex shadow for simplicity; the rounded rect looks clean enough.

    # --- Subtle inner border highlight ---
    draw.rounded_rectangle(
        [margin + int(3 * scale), margin + int(3 * scale),
         size - margin - int(3 * scale), size - margin - int(3 * scale)],
        radius=corner_radius - int(3 * scale),
        outline=(255, 255, 255, 25),
        width=int(2 * scale),
    )

    # --- Find a bold font ---
    bold_font_path = None
    regular_font_path = None
    bold_candidates = [
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        "/Library/Fonts/Arial Bold.ttf",
        "/System/Library/Fonts/SFNSDisplayCondensed-Bold.otf",
    ]
    regular_candidates = [
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/SFNSMono.ttf",
        "/System/Library/Fonts/Supplemental/Menlo.ttc",
    ]
    for p in bold_candidates:
        if os.path.exists(p):
            bold_font_path = p
            break
    for p in regular_candidates:
        if os.path.exists(p):
            regular_font_path = p
            break
    fallback_path = bold_font_path or regular_font_path

    # --- "MD" text (large, bold, centered upper area) ---
    font_size = int(380 * scale)
    try:
        font = ImageFont.truetype(fallback_path, font_size)
    except Exception:
        font = ImageFont.load_default()

    text = "MD"
    bbox = draw.textbbox((0, 0), text, font=font)
    tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
    tx = (size - tw) / 2 - bbox[0]
    ty = (size - th) / 2 - bbox[1] - int(60 * scale)
    # Text shadow
    draw.text((tx + 3 * scale, ty + 3 * scale), text, fill=(0, 0, 0, 60), font=font)
    # Main text
    draw.text((tx, ty), text, fill=(255, 255, 255, 255), font=font)

    # --- Decorative "# preview" below MD ---
    hash_size = int(110 * scale)
    try:
        hash_font = ImageFont.truetype(regular_font_path or fallback_path, hash_size)
    except Exception:
        hash_font = ImageFont.load_default()
    hash_text = "# preview"
    hbbox = draw.textbbox((0, 0), hash_text, font=hash_font)
    hw = hbbox[2] - hbbox[0]
    hx = (size - hw) / 2 - hbbox[0]
    hy = ty + th + int(60 * scale)
    draw.text((hx + 2 * scale, hy + 2 * scale), hash_text, fill=(0, 0, 0, 40), font=hash_font)
    draw.text((hx, hy), hash_text, fill=(255, 255, 255, 150), font=hash_font)

    return img


def main():
    os.makedirs(ICONSET_DIR, exist_ok=True)

    # Generate all required sizes for iconutil
    icon_sizes = {
        "icon_16x16.png": 16,
        "icon_16x16@2x.png": 32,
        "icon_32x32.png": 32,
        "icon_32x32@2x.png": 64,
        "icon_128x128.png": 128,
        "icon_128x128@2x.png": 256,
        "icon_256x256.png": 256,
        "icon_256x256@2x.png": 512,
        "icon_512x512.png": 512,
        "icon_512x512@2x.png": 1024,
    }

    # Generate the 1024 master and resize for others
    master = draw_icon(1024)

    for filename, px in icon_sizes.items():
        out_path = os.path.join(ICONSET_DIR, filename)
        if px == 1024:
            master.save(out_path, "PNG")
        else:
            resized = master.resize((px, px), Image.LANCZOS)
            resized.save(out_path, "PNG")
        print(f"  Created {filename} ({px}x{px})")

    # Convert to .icns using iconutil
    icns_path = os.path.join(OUTPUT_DIR, "icon.icns")
    subprocess.run(
        ["iconutil", "-c", "icns", ICONSET_DIR, "-o", icns_path],
        check=True,
    )
    print(f"\n  icon.icns created at: {icns_path}")

    # Also save the 1024 PNG for reference
    master.save(os.path.join(OUTPUT_DIR, "icon_1024.png"), "PNG")
    print(f"  icon_1024.png saved for reference")

    # Windows .ico (multi-size, required for embedding into the .exe)
    ico_sizes = [(s, s) for s in (16, 32, 48, 64, 128, 256)]
    ico_path = os.path.join(OUTPUT_DIR, "icon.ico")
    master.save(ico_path, format="ICO", sizes=ico_sizes)
    print(f"  icon.ico created at: {ico_path}")

    # Clean up iconset directory
    import shutil
    shutil.rmtree(ICONSET_DIR)
    print("  Cleaned up .iconset directory")


if __name__ == "__main__":
    main()
