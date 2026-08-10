# Promo assets

Marketing assets for aterkeep. All self-contained, all built from the panel's
own design tokens (`docs/DESIGN.md`), so they match the product exactly.

| File | What it is |
|---|---|
| `index.html` | Animated ~30s product reel. Open it in a browser; it autoplays and loops. |
| `logo.svg` | The brand mark (green rounded square, `>_` prompt glyph). Scalable, no text dependency. |
| `social.svg` | 1200×630 social/OG preview card. |

## The reel

Open `promo/index.html` in any browser — no server, no build, nothing to
install. It plays seven scenes on a loop: the problem, the automatic queue
confirmation (the differentiator), sign-in, the live panel, the feature set,
and the purchase CTA. `↻ replay` and `⏸ pause` are bottom-right.

It respects `prefers-reduced-motion`: with that on, it stops autoplaying and
becomes a static vertical deck instead.

### Turning it into an actual video file

The reel is HTML, not an `.mp4`. To get a video file:

1. Open `promo/index.html`, press **F** for fullscreen.
2. Screen-record the loop (~30s) with OBS, ShareX, or the built-in recorder
   (Win+G on Windows, Shift+Cmd+5 on macOS).
3. Trim to one clean loop.

A browser-only capture (no extra tools) also works:
`Ctrl+Shift+P` in Chrome DevTools → "Capture screenshot" for stills, or the
Recorder panel for a `.webm`.

### Rendering the SVGs to PNG

If you need raster versions (e.g. for a store listing):

```
# with librsvg
rsvg-convert promo/social.svg -o social.png
rsvg-convert -w 512 -h 512 promo/logo.svg -o logo-512.png

# or just open the SVG in a browser and screenshot it
```

## Note

These sell the product honestly — the CTA scene carries the no-affiliation line
and the Aternos-terms caveat. Keep those in any edit; they belong in
outward-facing material, not just the README.
