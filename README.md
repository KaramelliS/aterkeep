# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep logo" width="280"/>
</p>

**Self-hosted Aternos server manager & keep-alive dashboard.** A single Rust binary (~2.3 MB) that keeps your free Aternos Minecraft server online 24/7 and gives you a modern web panel to control it — no browser automation, pure HTTP.

<p align="center">
  <a href="README.tr.md">Türkçe</a> ·
  <a href="README.de.md">Deutsch</a> ·
  <a href="README.fr.md">Français</a> ·
  <a href="README.es.md">Español</a> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.pt.md">Português</a> ·
  <a href="README.ru.md">Русский</a> ·
  <a href="README.zh.md">中文</a> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.ko.md">한국어</a> ·
  <a href="README.nl.md">Nederlands</a> ·
  <a href="README.pl.md">Polski</a>
</p>

---

<p align="center">
  <video src="https://cdn.jsdelivr.net/gh/KaramelliS/aterkeep@main/promo/video/aterkeep-en.mp4" controls width="100%">
    <a href="promo/video/aterkeep-en.mp4">Watch the 30-second overview</a>
  </video>
</p>

<p align="center">
  <sub><b>Watch the 30-second overview.</b> Not playing? <a href="promo/video/aterkeep-en.mp4">Download the video</a>.<br/>
  Other languages: <a href="promo/video/aterkeep-tr.mp4">Türkçe</a> · <a href="promo/video/aterkeep-de.mp4">Deutsch</a> · <a href="promo/video/aterkeep-fr.mp4">Français</a> · <a href="promo/video/aterkeep-es.mp4">Español</a> · <a href="promo/video/aterkeep-it.mp4">Italiano</a> · <a href="promo/video/aterkeep-pt.mp4">Português</a> · <a href="promo/video/aterkeep-ru.mp4">Русский</a> · <a href="promo/video/aterkeep-ar.mp4">العربية</a> · <a href="promo/video/aterkeep-zh.mp4">中文</a> · <a href="promo/video/aterkeep-ja.mp4">日本語</a> · <a href="promo/video/aterkeep-ko.mp4">한국어</a> · <a href="promo/video/aterkeep-nl.mp4">Nederlands</a> · <a href="promo/video/aterkeep-pl.mp4">Polski</a></sub>
</p>

---

## Features

- **Automatic queue confirmation** — when your turn comes up Aternos opens a ~30 second window and drops you to the back of the queue if nobody answers. This is the step that makes unattended 24/7 possible, and the reason keep-alive scripts sit in the queue forever.
- **Signs in for you** — set up with your Aternos account and aterkeep obtains the session cookie itself, then renews it when it expires. No DevTools, no monthly copy-paste.
- **Keep-alive loop** — polls every 30 s, restarts the server automatically when it goes offline (toggleable)
- **Anti-idle bot** — a Minecraft client that joins when the server is up so it isn't shut down for being empty
- **Web dashboard** — live status, queue position, start/stop/restart controls, auto-start switch
- **Server console** — watch the live server log from your browser
- **Settings editor** — read & change `server.properties` directly from the panel
- **Player list** — see who is online in real time
- **Request inspector** — every HTTP call with its JSON response (educational)
- **14 languages** — picked during setup, switchable any time from the header
- **Password-protected panel** — login required, every endpoint behind a session cookie
- **Encrypted session** — AES-256-GCM; the key is derived from your password and **never stored on disk**

## Platforms

aterkeep is a single self-contained binary — one file, no runtime to install.
It does shell out to **`curl`**, which ships with Windows 10+, macOS and most
Linux distributions (`pkg install curl` on Termux). That is the only external
requirement.

| Platform | Notes |
|---|---|
| **Windows** | 10/11 — `aterkeep.exe`, double-click or run from PowerShell/cmd. |
| **Linux** | x86_64 — `./aterkeep`. |
| **macOS** | Apple Silicon — `./aterkeep`. |
| **Android** | Via [Termux](docs/TERMUX.md) — builds from source only; no prebuilt binary. |

## Requirements

- **Windows, Linux, macOS, or Android** (the latter via Termux — see the [Termux guide](docs/TERMUX.md))
- Rust toolchain (only needed to build from source; release binaries ship without it)

## Install

### Option A — Release binary (recommended)

Download the right binary for your OS from the [Releases](../../releases) page:

| OS | Asset |
|---|---|
| Windows 10/11 | `aterkeep-windows.exe` |
| Linux x86_64 | `aterkeep-linux-amd64` |
| macOS (Apple Silicon) | `aterkeep-macos-arm64` |
| — | `aterkeep-extras.zip` — the anti-idle bot, autostart installers and docs |

**Download `aterkeep-extras.zip` too** if you want the bot or autostart, and
unpack it next to the binary. The daemon alone cannot run the bot: it needs the
`bot/` folder from that archive.

Linux aarch64, macOS Intel and Android are **not** prebuilt — building them
needs access to the private engine crate, so they are unavailable to buyers
today. Do not plan around them.

On Linux/macOS/Android, make it executable after downloading:

```bash
chmod +x aterkeep
```

### Option B — Build from source

This repo is **source-available**: it contains the binary crate (`rust/`) and pulls the
keep-alive engine from [`aterkeep-core`](https://github.com/KaramelliS/aterkeep-core), a
**private** crate. Building from source requires access to that private crate.

```bash
# requires read access to github.com/KaramelliS/aterkeep-core (git credential helper / PAT)
cargo build --release
# binary: target/release/aterkeep (aterkeep.exe on Windows)
```

For everyone else, use **Option A** (prebuilt binary) — the release build is identical and
needs no source access.

## Setup (once)

Run the binary and open **http://127.0.0.1:4041** — the setup wizard walks you
through three steps: panel language, panel password, Aternos session.

```bash
./aterkeep          # Windows: aterkeep.exe
```

### 1. Panel password

You choose a password during setup. It does double duty:

- it **protects the panel** (anyone reaching the panel can control your server), and
- it **encrypts your Aternos session** — the encryption key is derived from it with
  PBKDF2-HMAC-SHA256 (600 000 iterations) and is **never written to disk**.

> **There is no recovery.** The password is not stored anywhere. If you forget it,
> the encrypted session cannot be opened again — delete `config/` and set up afresh.

### 2. Aternos session

Two ways. The first is the one you want.

#### Aternos account (default)

Type your Aternos username and password. aterkeep signs in over plain HTTP and
obtains the session cookie itself — no browser, no DevTools, no copying. If your
account has more than one server the wizard asks which to keep alive; otherwise
it picks the only one and detects its address.

Those credentials are stored inside `config/session.enc`, under the same
AES-256-GCM encryption as the cookies, keyed off your panel password. They are
sent to `aternos.org` and nowhere else. They exist for one purpose: when the
session expires, **aterkeep logs in again by itself** and you never notice.

Two accounts can't use this:

- **Two-factor authentication enabled** — the login needs a code aterkeep can't produce.
- **Aternos demands a captcha** — it turns this on adaptively.

Both are reported as their own message, not a generic failure, and both mean
using the second method.

#### Paste cookies (fallback)

1. Open **https://aternos.org**, log in, and open your server's panel.
2. Press **F12** → **Network** → **F5** to reload.
3. Click any `aternos.org` request → Headers → Request Headers → copy the whole
   **`cookie:`** line.
4. Switch to **Console**, run this and copy the output:

   ```js
   window.AJAX_TOKEN
   ```

5. Paste both into the wizard and press **Install and start**.

`ATERNOS_SESSION` is an **HttpOnly** cookie, so no JavaScript (including
`document.cookie`) can read it — copying it from the Network tab is the only way.
Your **server id is detected automatically** from the `ATERNOS_SERVER` cookie in
what you paste; you never type it.

Sessions set up this way cannot renew themselves, so you will be doing this
again in 30 days (see below).

### Advanced — manual import (for automation)

If you generate the session elsewhere, write a `session.json`:

```json
{
  "token": "PASTE_AJAX_TOKEN",
  "sec": "PASTE_GENERATE_AJAX_TOKEN_VALUE",
  "cookies": [
    { "name": "ATERNOS_SESSION", "value": "PASTE_SESSION_VALUE" },
    { "name": "ATERNOS_SERVER", "value": "PASTE_SERVER_ID" }
  ]
}
```

Then import it (you will be asked for the panel password):

```bash
./aterkeep import session.json     # Windows: aterkeep.exe import session.json
```

## Run

```bash
./aterkeep          # Windows: aterkeep.exe
```

Because the key is derived from your password and never stored, **aterkeep asks for
the password on every start**. When running unattended (systemd, Docker, Termux
boot script), pass it through the environment instead:

```bash
ATERKEEP_KEY='your-panel-password' ./aterkeep
```

Then open **http://127.0.0.1:4041** and log in.

To have it come back after a reboot, use the installers in
**[docs/AUTOSTART.md](docs/AUTOSTART.md)** — a scheduled task on Windows (with
the password protected by DPAPI, so a copied `config/` folder still won't
decrypt), a systemd unit on Linux, Termux:Boot on Android. Each one states
plainly what storing the password costs you, and how to skip it.

## Files it creates

Everything lives in a single `config/` folder next to the binary — nothing is
scattered around:

| File | Contents |
|---|---|
| `config/aterkeep.json` | Language, port, bind address, password verifier (salted PBKDF2 hash) |
| `config/session.enc` | Your Aternos session, AES-256-GCM encrypted |
| `config/bot.json` | Anti-idle bot settings |
| `config/bot-status.json` | Bot's live status (written by the bot) |

Set `ATERKEEP_DIR` to move this folder elsewhere.

> **Never share the `config/` folder**, and never post screenshots of the setup
> screen. Even encrypted, it contains your session — and your cookies grant access
> to your Aternos account.

## Using the panel

| Tab | What it does |
|---|---|
| **Status** | Server state badge, address, last check, start/stop/restart, auto-start switch, live log, request inspector |
| **Console** | Server log stream (auto-refreshes every 10 s) |
| **Settings** | Edit `server.properties` values and save them to the server |
| **Players** | Online player list |

The **auto-start switch** is important: when it's OFF the daemon never restarts the server, so you can stop it and leave it stopped. Pressing **Stop** turns it off automatically.

## Session lifetime

Aternos issues its session cookie with `Max-Age=2592000` — **exactly 30 days**,
measured from the login response, not guessed. It can end sooner if you sign out
elsewhere or change your password.

**Set up with an account:** nothing to do. When the cookies lapse the daemon logs
in again, writes a fresh session and carries on. You'll see one line in the log.

**Set up by pasting cookies:** the panel shows a `SESSION` badge and a banner
saying the session expired — not a stopped server, which is what it used to look
like — with a button that drops you back into the wizard. Switching to the
account method at that point ends the chore for good.

The panel also shows **session age** next to the loop state, and after the first
expiry, how long the previous session lasted. Aternos doesn't publish a number,
so aterkeep measures your own.

## Security

- **No key file on disk.** The AES-256-GCM key that protects `config/session.enc` is
  derived from your password at every start (PBKDF2-HMAC-SHA256, 600 000 iterations,
  per-install random salt) and only ever lives in memory. Copying the `config/` folder
  gets an attacker nothing without the password.
- **The panel requires a login.** Every API endpoint is behind a session cookie
  (`HttpOnly`, `SameSite=Strict`). The password verifier stored in
  `config/aterkeep.json` uses a *different* salt from the encryption key, so the stored
  hash cannot be turned back into the key.
- The keep-alive engine (`aterkeep-core`) is a **private** crate — its source is not in this
  repo, only compiled into the release binary. Aternos endpoint/cookie/HTML strings are
  compile-time encrypted (litcrypt) so they never appear as plaintext in the binary; the
  release binary is also fully stripped (`strip = true`, LTO, `panic = abort`)
- The dashboard binds to `127.0.0.1` by default — nothing is exposed to the network.
  If you change `bind` in `config/aterkeep.json`, the panel password becomes your only
  defence; use a strong one and prefer a VPN or reverse proxy with TLS.

### What this does *not* protect against

Be realistic about the threat model: the protection holds while the daemon is **not
running under your account**. Once you have entered the password, the decrypted
session lives in the process's memory, and anyone with administrator access to that
machine (or malware running as your user) can read it. Keep the host clean.

## Troubleshooting

| Problem | Fix |
|---|---|
| Panel shows `SESSION` | The Aternos session expired → press **Refresh cookies** in the banner, or switch to account login so it renews itself. |
| Server keeps stopping | That's normal: Aternos pauses empty servers after ~60 s idle. Keep the daemon running; it restarts within about a minute. For truly uninterrupted uptime, keep a Minecraft client/bot connected. |
| Forgot the panel password | Cannot be recovered — it is never stored. Delete the `config/` folder and run setup again with a fresh session. |
| `decrypt failed (yanlis key?)` on start | Wrong password, or `ATERKEEP_KEY` is set to the wrong value. |
| Won't start in the background | The password cannot be prompted for without a terminal — set `ATERKEEP_KEY` in the service's environment, or use the installers in the [autostart guide](docs/AUTOSTART.md). |
| Should start on boot | See [docs/AUTOSTART.md](docs/AUTOSTART.md) — scheduled task (Windows, DPAPI-protected), systemd unit (Linux), Termux:Boot (Android), each with its security tradeoff spelled out. |
| Panel says the Aternos session expired | The cookies aged out — this is not a server fault. Press **Refresh cookies** in the banner and paste a fresh `cookie:` header. |
| Port 4041 busy | A previous instance is still running. Kill it (`pkill aterkeep` / Task Manager), or change `port` in `config/aterkeep.json`. |
| Running on Android? | See the dedicated [Termux guide](docs/TERMUX.md). |

## Anti-Idle Bot (Opsiyonel)

aterkeep, sunucuyu ayakta tutmak için gömülü bir **anti-idle botu** da barındırır — bir Minecraft istemcisi bağlıymış gibi davranır, görünmez (spectator + vanish) modda AFK kalır ve koparsa otomatik yeniden bağlanır. Böylece sunucu boş olduğu için durmaz. Bot [mineflayer](https://github.com/PrismarineJS/mineflayer) tabanlıdır ve daemon tarafından `node bot/index.js` olarak spawn edilir.

### Gereksinimler

- **Node.js** 18+ (bot'un çalıştığı makinede kurulu)
- Sunucu **cracked** olmalı (`online-mode=false` — Aternos ücretsiz sunucuları varsayılan böyledir)
- **Vanilla** yazılımı, **1.21.x** veya daha düşük sürüm

### Kurulum

```bash
cd bot
npm install          # mineflayer'ı kurar (tek seferlik)
```

Ardından bot'ı **panel üzerinden** açıp kapatabilirsiniz — daemon `config/bot.json`'u yazar, bot durumunu `config/bot-status.json`'a raporlar. Ayrıntılı doküman: [`docs/BOT.md`](docs/BOT.md), bot README'si: [`bot/README.md`](bot/README.md).

## ⚠ Before you buy: Aternos' terms

Aternos' own support documentation says:

> *"Trying to bypass Aternos system by using bots, scripts, or other tricks to
> keep your server on 24/7 is against our rules… The system automatically
> checks for artificial activity."*
> — [24/7 Hosting](https://support.aternos.org/hc/en-us/articles/31771896948253-24-7-Hosting)

That describes this product. **Using aterkeep may get your server or your
Aternos account suspended or deleted.** There is no way for us to prevent that,
and the anti-idle bot makes the activity easier to spot, not harder.

This is sold as-is, for use on accounts you control and are willing to risk.
If that is not acceptable to you, do not buy it — a paid Minecraft host is the
supported way to run a server around the clock.

## License

**aterkeep is commercial software — it is not open source.**

The source is published for transparency and evaluation only. Personal,
non-commercial use is permitted. Redistribution, resale, derivative works and
commercial use are **not** permitted. See [LICENSE](LICENSE) for the full terms.

## Buying a licence

Commercial use, redistribution, white-labelling and source access to the
keep-alive engine (`aterkeep-core`) are available under a paid commercial licence.

**Contact:** berlaylc2138@gmail.com

## Disclaimer

Independent project — not affiliated with, endorsed by or connected to Aternos
GmbH or Mojang Studios.
