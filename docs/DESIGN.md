# aterkeep — design rules

Every rule here exists because something was measured or read out of a real
product's source. Where a rule came from another panel, that panel is named and
the value is quoted. Where it came from measuring our own CSS, the number is
given. Nothing is here because it sounded nice.

The panel is an **operations tool**. It is looked at while something is wrong,
and the single question it answers is *is the server up*. That is the brief.

Fourteen panels were read at source level for this: Pterodactyl, Pelican,
Aternos, exaroton, Minehut, Shockbyte, Apex, BisectHosting, Multicraft, AMP,
Portainer, Proxmox VE, Crafty Controller, PufferPanel, MCSManager.

---

## 0. The one reference that matters most

Our panel sits **next to Aternos' own panel** — a buyer will have both open. So
their tokens are the adjacency we are designing against, not an abstract ideal.
From [their stylesheet](https://assets.aternos.org/assets/panel/css/aternos-c705dbc3f071973b.css)
(667 bytes, the whole design system):

```css
--font-main: Play, sans-serif;   --font-mono: "Roboto Mono", monospace;  --bold: 700;
--color-main:    #2b87d3;   /* the actual brand: blue */
--color-success: #1fd78d;   --color-danger:  #F62451;
--color-warn:    #EB7B59;   --color-loading: #A4A4A4;
--color-dark:    #2D3943;   --color-light:   #f0f0f0;
```

Two things to take from it:

**Their status is not a badge — it is a full-bleed 48px bar** welded under the
address card, sharing only its bottom radius. The screen's dominant colour *is*
the server state. For a product whose entire value is "is it up", that is the
correct primitive, and a pill is not.

**Their brand blue is subordinate to a functional green.** `#1fd78d` owns the
Start button, the online bar and the metric tiles, so users read green as
"Aternos". We should not fight that association — green meaning "running" is
already installed in our users' heads.

Worth knowing for calibration: the same team's paid product **exaroton** is an
exact inversion — `--color-main: #19ba19`, Poppins 600, `--button-border-radius:
1000px`, dark. Free reads as light + 2px radii + neon; paid reads as dark +
pills + muted. We are a paid product used next to the free one.

---

## 1. Colour

### 1.1 Colour means state. Nothing else.

**Rule: the interactive accent is neutral. Chromatic colour is reserved
entirely for state.**

This was our worst structural mistake. `--ok` green was carrying, at once:
primary buttons, the active sidebar indicator, links, the queue progress bar,
the heartbeat dot, the ONLINE badge, and the "good" metric value. When green is
everywhere, green stops meaning *the server is up*.

The obvious fix — move the accent to blue — is the wrong one. **Nine of eleven
panels surveyed use blue or cyan**: Pterodactyl `#0891b2`, Minehut `#488AFF`,
Shockbyte `#3384ff`, Bisect `#03ddff`, Multicraft `#2CB2FF`, AMP `#5EA8DE`,
Portainer `#0ba5ec`, Proxmox `#3892d4`, Pelican Blue. Joining them buys nothing.

**Portainer** shows the better answer: its primary buttons are
`bg-graphite-700 text-mist-100` — **not blue**. Chrome is neutral; colour is
saved for meaning. We take that position, which also makes the status bar the
loudest thing on the screen, which is exactly what this product should do.

So:

- **Buttons, nav, focus, links** — neutral. `--panel-3` surfaces, `--fg` text,
  `--line-strong` borders. The primary button is brighter, not coloured.
- **`--ok` green appears only where the server is running.** Nowhere else.
- **Progress and the heartbeat dot** are state, so they may use state colour.

Two panels demonstrate what happens otherwise. **PufferPanel** sets online to
`var(--primary)`, which is **user-chosen** — pick pink and "running" is pink —
and `installing` gets the same colour, so only the glyph differs; meanwhile
green is used on the *Start button*, so green means "click to start" and blue
means "started". **MCSManager** uses green for the running tag but its progress
bars run blue → orange → red, skipping green: two contradictory "good" colours.

### 1.2 Offline is grey. Transitional gets its own colour.

The most common failure in the category, in both directions:

- **Crafty** renders offline as amber `#ffaf00` — the same as its warning state.
  A server that is simply switched off looks like a fault.
- **AMP** collapses Stopped, Stopping, Unable-to-run, Suspended and Maintenance
  all into `#B70000`. "I turned it off" and "it died" are the same colour — the
  most important distinction in an ops tool, not encoded.
- **PufferPanel** has no transitional state at all
  (`installing | online | offline | unknown`).
- **MCSManager** renders `STOPPING`, `BUSY` and `STOPPED` identically.
- **Aternos** reuses grey `#A4A4A4` for both `loading` and `pending`, so
  "starting" has no colour of its own.

Ours, which is correct and must stay correct:

| state | colour | meaning |
|---|---|---|
| online | `--ok` | running |
| starting / queue / waiting | `--warn` | in motion, will resolve itself |
| offline | `--fg-mute` | off, nothing wrong |
| crashed / session expired | `--err` | needs a human |

**BisectHosting** is the only panel that gets transitional state genuinely
right, and it does it with shape rather than hue: an `animate-spin` ring
(`border border-current/30 border-t-current`) with `motion-reduce:animate-none`.
Copy the technique, not the colour.

### 1.3 Surfaces: a ramp with defined jobs

Steps follow [Radix's 12-step semantics](https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale),
the clearest published statement of what each step in a dark ramp is *for*:

| step | job | our token |
|---|---|---|
| 1 | app background | `--bg` |
| 2 | secondary surface (cards, sidebar, bars) | `--panel` |
| 3–5 | component background: rest / hover / active | `--panel-2`, `--panel-3` |
| 6 | border on non-interactive elements | `--line-2` |
| 7 | border on interactive elements | `--line` |
| 8 | strong border, focus ring | `--line-strong` |
| 11 | low-contrast text | `--fg-mute` |
| 12 | high-contrast text | `--fg` |

**Borders carry the structure here.** We use no shadows and no gradients, so a
weak border means no structure at all. Measured, background→border:

```
ours (before)  1.31:1
Radix          1.54:1
```

Our surface *steps* were fine (1.06 vs Radix's 1.07) — the flatness came from
the borders. Borders must land at **≥1.5:1** against the surface behind them.

### 1.4 Text contrast is measured, not eyeballed

WCAG AA is 4.5:1 for normal text, 3:1 for large text and UI components.

```
--fg       15.24:1
--fg-dim    7.56:1
--fg-mute   5.53:1     (was #6b7480 = 3.87:1 — below AA, on the smallest text)
```

`--fg-mute` was failing at exactly the 10.5px labels and 12.5px help text where
reading is hardest. **BisectHosting ships the same bug** — `--color-light-500
#4f545a` on `#020525` is ≈2.8:1, and it is a *text* token. Do not reintroduce a
"subtle" grey below 4.5:1.

### 1.5 No hex outside the token system

**MCSManager's** ECharts config alone contains `#e0e6f0`, `#94b8e0`,
`rgba(67,145,250,0.95)` — none of them tokens, so its charts do not follow its
own theme. **AMP** has *zero* custom properties in 3,520 lines of CSS
(`#347EB3` appears 17 times, `#5EA8DE` 19 times), which is why its 36 community
themes each reverse-engineer ~100 undocumented selectors and range from 2 KB to
52 KB for the same job.

We had one: `.stream .time { color: #3f4854 }`, measured at **2.10:1** — log
timestamps were effectively invisible. Any literal colour in a rule is a bug.

---

## 2. Type

Nine distinct font sizes were in use — `10.5 / 11 / 11.5 / 12 / 12.5 / 13 / 14 /
15 / 19` — including three half-pixel values, none derived from any other. This
is the most legible symptom of "generic dashboard": every number chosen by eye,
in isolation.

Six steps:

| token | px | use |
|---|---|---|
| `--t-xs` | 11 | uppercase labels, badges, timestamps |
| `--t-sm` | 12 | log lines, dense cells, help text |
| `--t-md` | 13 | secondary body, key/value values |
| `--t-base` | 14 | body, buttons, inputs |
| `--t-lg` | 16 | panel headings |
| `--t-xl` | 20 | metric values |

Weights: **500** default, **600** headings and labels, **700** brand only.

Do not set `font-weight: lighter` on headings. **AMP** does, on every heading,
button, sidebar link and `<th>` over dark navy — its table headers are lighter
than its table body. **Proxmox** sets weight 300 with no webfont, so on Windows
it falls back to Arial and the 300 does nothing at all.

**Data is monospace and tabular.** Anything that updates in place — TPS, RAM,
players, clock, queue percentage — uses `--mono` *and*
`font-variant-numeric: tabular-nums`. In a proportional face `1` is narrower
than `8`, so live numbers change width on every refresh and drag the layout with
them. We poll every three seconds.

Uppercase micro-labels get `letter-spacing: .06em`. Body text never does.

---

## 3. Space and shape

Nineteen distinct spacing values were in use, **thirteen not multiples of 4**
(`1, 2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 15, 18`). There was no system; there were
opinions.

```
--s-1: 4px    --s-2: 8px    --s-3: 12px   --s-4: 16px
--s-5: 20px   --s-6: 24px   --s-8: 32px   --s-10: 40px
```

Radii were `2, 3, 4, 5, 6, 10px, 50%` — seven values. Now three:

```
--r-sm: 4px    badges, pills, inputs, small buttons
--r:    6px    cards, panels, buttons
--r-full: 50%  dots only
```

The category spans **Aternos and Proxmox at 2px** through **Pterodactyl and
Multicraft at 4px**, **Portainer 8/12px**, **Bisect 12px**, to **PufferPanel at
16px everywhere** with 50% circular icon buttons. 16px plus rounded geometric
type reads friendly-consumer; 0px (Crafty) reads like a 2019 Bootstrap admin
template. 6px is deliberate middle ground: built, not toy-like.

---

## 4. Components

**Status.** The loudest element on the page. Full-width, its own band, colour +
word + shape. Not a chip in a row of chips — **Shockbyte's `● Offline` is
pixel-identical to its `Forge 1.20.1` metadata chip**, which is the cautionary
tale for a product whose entire value is state.

**BisectHosting** has the most sophisticated version: the console frame's 1px
border is state-reactive (`border-warning-600/40` starting,
`border-error-500/40` stopping) with the status pill notched into its top-left
corner. The frame *is* the indicator.

**Top bar.** Fixed, `--panel`, 1px bottom border. Never accent-filled —
**PufferPanel** fills its 64px topbar with `--primary` and it dominates every
screen.

**Sidebar.** Fixed 208px, `--panel`, 1px right border. Active item: `--panel-2`
plus a 2px inset bar (Pterodactyl uses `inset 0 -2px` cyan; AMP uses `inset 4px
0`). No background photograph — **MCSManager** parallax-slides a JPEG behind its
nav and hard-codes fifteen white alphas over it, which are wrong the moment that
photo is light.

**Metric tile.** Uppercase `--t-xs` label, value in `--t-xl` mono-tabular.
**Show the number.** `used / limit`, with `∞` when unlimited — this is a real
convention across Pterodactyl, Pelican and Shockbyte, and it costs nothing to
adopt. Threshold-colour at **0.8 warning / 0.9 danger** (Pterodactyl 0.8/0.9,
Pelican 0.7/0.9, Proxmox and Portainer 0.75/0.9). BisectHosting **dropped**
threshold colouring in its fork, so `9.89 GB / 10 GB` renders identically to
`0.2 GB / 10 GB` and nothing warns before an OOM.

Charts are the second tier, not the first. **Shockbyte's** exact-numbers-by-
default with a `Show Advanced` toggle to time-series is the smartest pattern
found; **Apex** shows one chart with no numeric readout at all, and **Minehut's**
dashboard shows no live RAM/CPU anywhere. Never ship axis-less decorative
sparklines — Bisect's "gridlines" are three `h-[0.5px]` divs at 32/67/97%
corresponding to no value.

**Queue card.** Position, ETA, a track, percentage, and one sentence saying
confirmation is automatic. Aternos has a `getPercentage()` and still shows
`53 / 824` in a **14px pill — the smallest text on the screen**. The queue is
why this product exists; give it the space.

**Log stream.** `--bg` (darker than the card around it), 1px border, `--mono`,
`--t-sm`, **`line-height: 1.6`**, `white-space: pre-wrap`, and an explicit
scrollback cap.

Set every terminal default explicitly. **1000 lines is nobody's decision** — it
is xterm.js's default, unexamined, in Pterodactyl, Pelican, Bisect, Portainer
and Proxmox. **Portainer instantiates `new XTerm()` with no options at all**;
**Proxmox** ships carefully-written console CSS that its WebGL renderer never
reads. Crafty declares no `line-height` (≈1.2), MCSManager passes no
`fontFamily` (Courier New on Windows) and no `lineHeight` (1.0, lines touch).

**Colour log levels — this is the category's biggest gap and our clearest win.**
Pterodactyl, Pelican, Bisect and Aternos are ANSI-passthrough only. AMP uses
last-match-wins substring regex, so a chat message containing "error" turns red.
Apex paints timestamps white and *everything else* one bright green. Portainer
*has* an ANSI-256 parser and deliberately flattens its six semantic log tokens
to monochrome. Multicraft's §-code renderer — the one genuinely Minecraft-native
feature in the category — is **broken in its current default theme**: the CSS
classes were dropped, the JS still emits them.

The strongest pattern found is **Shockbyte's**: muted timestamp, bracketed level
tag in its own hue, near-white message, and a **full-width translucent row wash
on error lines** (`#ff5c5c14`). One rule, and errors become scannable. Filter
tabs with counts (`Errors 3 | Chat 16 | Debug 24`) on top of that.

**Buttons.** One primary per view; make it **state-driven** rather than showing
four buttons three of which are disabled — Shockbyte's single button becomes
Start → Stop → Restart, Bisect's mutates Start → **Wake** and Stop → **Kill**.
Destructive is `--err`, outlined not filled, and never unlabelled next to a safe
one: **PufferPanel** renders Start/Restart/Stop/Kill as four unlabelled 2em
circles inside an `<h1>`.

**Settings.** Echo the underlying key in dim mono under the control —
`white-list=true` beneath a Whitelist toggle, `max-players=20` beneath Slots.
Aternos does this and it is the single cheapest way in the whole survey to make
a consumer product read as a real admin tool. Minehut's uppercase mono
micro-label under a big value (`113 days ago` / `LAST ONLINE`) is the same idea.

**Banners.** Full-width, above content, one per condition, always carrying the
action that resolves it.

---

## 5. Motion

Curves from [IBM Carbon](https://carbondesignsystem.com/elements/motion/overview/):

```css
--ease:     cubic-bezier(0.5, 0, 0.1, 1);   /* default */
--ease-in:  cubic-bezier(0, 0, 0.25, 1);    /* entering */
--ease-out: cubic-bezier(0.25, 0, 1, 1);    /* leaving */
--dur-1: 100ms   /* hover, focus, colour */
--dur-2: 200ms   /* panels, disclosure */
--dur-3: 300ms   /* ceiling */
```

Nothing animates longer than 300ms. **Nothing loops** except the heartbeat dot,
which fires once per successful poll and is information, not decoration.
`prefers-reduced-motion: reduce` disables all of it.

Crafty ships a 2-second continuous spin. MCSManager wobbles its logo every ten
seconds, forever, in an admin tool. AMP pulses card saturation `1 → 2` on a
0.5s infinite loop for every busy instance.

At the other extreme, **Proxmox has zero motion and zero optimistic feedback** —
clicking Start produces a new row in a log panel ~900px away from the button.
Feedback belongs where the click was.

---

## 6. Do not

- **Do not tint the whole UI with a hue.** Chrome is neutral; colour is state.
- **Do not use colour as the only signal.** Every state gets a word.
- **Do not add a second icon set.** Crafty loads Material Design Icons,
  flag-icon, Themify, Typicons, FontAwesome 6 *and six separate Phosphor weight
  stylesheets*, then mixes `mdi-*` with `ph-*` in one navbar. AMP ships Material
  Symbols *and* 54 raster Glyphicons. We ship a single binary: inline SVG or
  nothing.
- **Do not use emoji as icons.**
- **Do not add gradients.** Crafty's one gradient is
  `linear-gradient(to top, var(--dropdown-bg), var(--dropdown-bg))` — a no-op.
- **Do not add decorative imagery.** No hero art, no wallpaper mode, no
  background photographs behind navigation.
- **Do not fake depth with shadows.** Structure comes from borders and surface
  steps. A shadow is for something genuinely floating.
- **Do not centre body text.**
- **Do not `!important` your way out.** MCSManager's wallpaper mode is 136 lines
  of `!important` against Ant internals and breaks on every minor upgrade.
- **Do not let admin and user screens diverge.** Pterodactyl ships AdminLTE 2 +
  Bootstrap 3 + FontAwesome 4.7 + **IE9 conditional comments** in `/admin`
  alongside React/Tailwind for users. Pelican is the only surveyed panel that
  avoided this, by running all three panels through one base provider.
- **Do not reserve ad space in the stylesheet.** Both ad-supported panels here
  do (`.responsive-sky-scraper { min-width:300px }`), and both take their
  loudest complaints for it.
- **Do not ship half-pixel sizes or off-scale spacing.** If a value is not on
  the scale, the layout is wrong, not the scale.

---

## 7. What we already do better than the category

Worth writing down so it does not get "improved" away:

- **Four distinct server states with correct semantics** — grey off, amber
  transitional, green running, red needs-you. Crafty makes offline amber; AMP
  makes stopped and crashed the same red; PufferPanel has no transitional state;
  MCSManager renders stopping, busy and stopped identically.
- **Every console default set explicitly** (font, size, `line-height: 1.6`), and
  the console uses theme tokens rather than hard-coded black. Five surveyed
  panels leave xterm.js on its defaults.
- **A green accent in a category that is 9-of-11 blue.** Differentiating, and
  aligned with what Aternos users already read as "running".
- **One icon approach, one font pair, no vendor admin template.** Crafty carries
  a 30,962-line forked Bootstrap 4.3.1 theme plus a 28,347-line near-duplicate.
- **Design tokens from the start.** AMP has none and pays for it in every theme.
- **Pure HTTP, no framework.** One HTML file, one CSS file, one JS file,
  embedded in the binary.

---

## Sources

- [Radix Colors — the scale](https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale)
  · [values](https://unpkg.com/@radix-ui/colors@3.0.0/slate-dark.css)
- [IBM Carbon — motion](https://carbondesignsystem.com/elements/motion/overview/)
- [Aternos panel tokens](https://assets.aternos.org/assets/panel/css/aternos-c705dbc3f071973b.css)
  · [exaroton tokens](https://assets.exaroton.com/assets/panel/css/dist/front/exaroton-aa9f48c0d62740a0.css)
- [Pterodactyl `tailwind.config.js`](https://github.com/pterodactyl/panel/blob/1.0-develop/tailwind.config.js)
  · [Pelican](https://github.com/pelican-dev/panel) · [Portainer](https://github.com/portainer/portainer)
- [Multicraft live demo](https://demo.multicraft.org/site/login) (`admin`/`admin`)
- Source reads of [Crafty](https://gitlab.com/crafty-controller/crafty-4),
  [PufferPanel](https://github.com/PufferPanel/PufferPanel),
  [MCSManager](https://github.com/MCSManager/MCSManager)
