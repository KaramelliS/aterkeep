# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep logo" width="280"/>
</p>

**Self-hosted Aternos server manager & keep-alive dashboard.** A single Rust binary (~1.7 MB) that keeps your free Aternos Minecraft server online 24/7 and gives you a modern web panel to control it — no browser automation, pure HTTP.

<p align="center">
  <a href="README.tr.md">Türkçe</a> ·
  <a href="README.de.md">Deutsch</a> ·
  <a href="README.fr.md">Français</a> ·
  <a href="README.es.md">Español</a> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.pt.md">Português</a> ·
  <a href="README.ru.md">Русский</a> ·
  <a href="README.ar.md">العربية</a> ·
  <a href="README.zh.md">中文</a> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.ko.md">한국어</a> ·
  <a href="README.nl.md">Nederlands</a> ·
  <a href="README.pl.md">Polski</a>
</p>

---

## Features

- **Keep-alive loop** — polls every 90 s, restarts the server automatically when it goes offline (toggleable)
- **Web dashboard** — live status, start/stop/restart controls, auto-start switch
- **Server console** — watch the live server log from your browser
- **Settings editor** — read & change `server.properties` directly from the panel
- **Player list** — see who is online in real time
- **Request inspector** — every HTTP call with its JSON response (educational)
- **14 languages** — picked during setup, switchable any time from the header
- **Password-protected panel** — login required, every endpoint behind a session cookie
- **Encrypted session** — AES-256-GCM; the key is derived from your password and **never stored on disk**

## Platforms

aterkeep is a single self-contained binary — one file, no runtime, no dependencies.

| Platform | Notes |
|---|---|
| **Windows** | 10/11 — `aterkeep.exe`, double-click or run from PowerShell/cmd. |
| **Linux** | x86_64 & aarch64 — `./aterkeep`. |
| **macOS** | Intel & Apple Silicon — `./aterkeep`. |
| **Android** | Via [Termux](docs/TERMUX.md) — runs natively on `aarch64`, keep-alive on your phone. |

## Requirements

- **Windows, Linux, macOS, or Android** (the latter via Termux — see the [Termux guide](docs/TERMUX.md))
- Rust toolchain (only needed to build from source; release binaries ship without it)

## Install

### Option A — Release binary (recommended)

Download the right binary for your OS from the [Releases](../../releases) page:

| OS | Binary |
|---|---|
| Windows | `aterkeep.exe` (or `aterkeep-windows-x86_64.exe`) |
| Linux x86_64 | `aterkeep-linux-x86_64` |
| Linux aarch64 | `aterkeep-linux-aarch64` |
| macOS Intel | `aterkeep-macos-x86_64` |
| macOS Apple Silicon | `aterkeep-macos-aarch64` |
| Android (Termux) | `aterkeep-android-aarch64` — see [docs/TERMUX.md](docs/TERMUX.md) |

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

1. Open **https://aternos.org**, log in, and open your server's panel.
2. Press **F12** → **Network** → **F5** to reload.
3. Click any `aternos.org` request → Headers → Request Headers → copy the whole
   **`cookie:`** line.
4. Switch to **Console** and run, then copy the output:

   ```js
   window.AJAX_TOKEN + "|" + window.generateAjaxToken()
   ```

5. Paste both into the wizard and press **Kur ve başlat**.

`ATERNOS_SESSION` is an **HttpOnly** cookie, so no JavaScript (including
`document.cookie`) can read it — copying it from the Network tab is the only way.
Your **server id is detected automatically** from the `ATERNOS_SERVER` cookie in
what you paste; you never type it.

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

Aternos session cookies last **~30 days**. When the panel shows `OTURUM BİTTİ` / `LOGGED OUT`, repeat the export steps above (re-run the wizard, or re-import a fresh `session.json`). That's it — once a month, two minutes.

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
| Panel shows `LOGGED OUT` | Session expired → press **↻ Oturum** in the panel header, or delete `config/session.enc` and set up again. |
| Server keeps stopping | That's normal: Aternos pauses empty servers after ~60 s idle. Keep the daemon running; it restarts within 90 s. For truly uninterrupted uptime, keep a Minecraft client/bot connected. |
| Forgot the panel password | Cannot be recovered — it is never stored. Delete the `config/` folder and run setup again with a fresh session. |
| `decrypt failed (yanlis key?)` on start | Wrong password, or `ATERKEEP_KEY` is set to the wrong value. |
| Won't start in the background | The password cannot be prompted for without a terminal — set `ATERKEEP_KEY` in the service's environment. |
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
