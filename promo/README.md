# Promo assets

Marketing material for aterkeep. Everything here is built from the panel's own
design tokens (`docs/DESIGN.md`), so it matches the product exactly.

| File | What it is |
|---|---|
| `video/aterkeep-<lang>.mp4` | **Product video, one per language** (14). 1920×1080, ~30s, H.264. |
| `make_videos.py` | Generates those videos. Frames are drawn, then encoded. |
| `promo_strings.py` | The video copy, 14 languages. Edit here to change wording. |
| `index.html` | The same reel as a self-playing web page (handy for a landing page). |
| `logo.svg` | Brand mark, scalable, no text dependency. |
| `social.svg` | 1200×630 social / OG preview card. |

## The videos

Real `.mp4` files — post them to Discord, Twitter, YouTube, a store page,
anywhere. Seven scenes: the problem, the automatic queue confirmation (the
thing a plain keep-alive script cannot do), sign-in, the live panel, the
feature set, and the purchase call to action.

They are **drawn, not screen-recorded**. Every frame is composed by Pillow and
encoded with ffmpeg, so the result is deterministic: the same input always
produces the same video, and changing a sentence never means re-recording
anything.

### Regenerating

```bash
pip install pillow
pip install arabic-reshaper python-bidi     # only needed for the Arabic video

python promo/make_videos.py                 # all 14 languages (~15 min)
python promo/make_videos.py tr en           # just these two
python promo/make_videos.py --fps 30 tr     # smoother, larger file
```

ffmpeg must be on `PATH`, or point at it:

```bash
ATERKEEP_FFMPEG=/path/to/ffmpeg python promo/make_videos.py
```

### Changing the wording

Edit `promo_strings.py`. English is the base; every other language lists only
what it overrides, and any missing key falls back to English — the same rule
the panel's own translations use. Keep lines short: they are sized for 1920px
and read in a few seconds.

### Notes on the rendering

Two things that are easy to break if you touch `make_videos.py`:

- **Font fallback.** Consolas has no Arabic or CJK glyphs, so a translated
  string in the monospace face renders as tofu boxes. `_needs_fallback()`
  checks the script ranges and switches to the language's sans face. The
  Arabic reshaper emits *presentation forms* (U+FB50+, U+FE70+), which is why
  those ranges are in the list too.
- **Vertical centring is computed once per scene**, from the final frame. If
  it were per-frame, the whole composition would jitter as the queue counter
  narrows and the feature cards appear.

The tick mark next to "confirmed automatically" is **drawn with lines**, not
typed: U+2713 is missing from Segoe UI Semibold and rendered as a box.

## Rendering the SVGs to PNG

```bash
rsvg-convert promo/social.svg -o social.png
rsvg-convert -w 512 -h 512 promo/logo.svg -o logo-512.png
# or open the SVG in a browser and screenshot it
```

## Please keep these

The final scene carries the licence line, the no-affiliation line and the
Aternos-terms warning. Those belong in outward-facing material, not only in
the README — anyone who sees the video should know the risk before they buy.
