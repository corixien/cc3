#!/usr/bin/env python3
"""Pixel comparison of a native render against a Chromium golden image.
Usage: compare.py native.png golden.png [diff.png]
Prints: share of pixels that differ by more than 8/255 in any channel, and mean abs difference."""
import sys
from PIL import Image, ImageChops
a = Image.open(sys.argv[1]).convert("RGB")
b = Image.open(sys.argv[2]).convert("RGB")
if a.size != b.size:
    a = a.resize(b.size)
d = ImageChops.difference(a, b)
px = d.getdata()
n = len(px)
bad = sum(1 for p in px if max(p) > 8)
mean = sum(sum(p) for p in px) / (3 * n)
print(f"size={b.size} differing={bad / n * 100:.2f}% mean_abs_diff={mean:.2f}/255")
if len(sys.argv) > 3:
    d.point(lambda v: min(255, v * 4)).save(sys.argv[3])
