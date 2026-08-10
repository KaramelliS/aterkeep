# aterkeep on Android (Termux)

Run the aterkeep keep-alive daemon on your phone with [Termux](https://termux.dev/). The daemon builds into a single native binary for `aarch64-linux-android`, so it needs no emulator — it runs straight on the ARM CPU.

> **Prerequisites:** an Android device you control, and your Aternos account login (https://aternos.org) ready. You do **not** need root.

---

## 1. Install Termux

Use the **F-Droid** build — the Google Play version is deprecated and breaks package installs.

1. Install [Termux from F-Droid](https://f-droid.org/packages/com.termux/).
2. Open Termux and update packages:

   ```bash
   pkg update && pkg upgrade -y
   ```

3. Grant storage if you ever want to copy files in/out (optional):

   ```bash
   termux-setup-storage
   ```

---

## 2. Install the Rust toolchain

```bash
pkg install rust git clang make -y
```

Termux's `rustc` compiles **natively for `aarch64-linux-android`** by default — no `rustup target add` needed in most cases. Verify:

```bash
rustc --print cfg | grep target
# expect: target_arch="aarch64", target_os="android"
```

<details>
<summary><b>Low-RAM phone? Use proot-distro + Ubuntu (click to expand)</b></summary>

Some phones run out of memory compiling Rust directly in Termux. Install a Linux distribution and compile there:

```bash
pkg install proot-distro -y
proot-distro install ubuntu
proot-distro login ubuntu
# inside Ubuntu:
apt update && apt install -y rustc cargo git clang make
```

Then follow the build steps below from inside the Ubuntu session. The resulting binary runs fine back in native Termux afterward.

</details>

---

## 3. Build aterkeep

```bash
git clone https://github.com/KaramelliS/aterkeep.git
cd aterkeep
cargo build --release --manifest-path rust/Cargo.toml
```

The binary lands at `target/release/aterkeep`. Run it from anywhere — copy it somewhere convenient:

```bash
cp target/release/aterkeep ~/aterkeep
cd ~
```

---

## 4. First-run setup (onboarding wizard)

On first run aterkeep walks you through entering your Aternos session interactively:

```bash
./aterkeep
```

You'll be prompted for three values:

| Field | Where to get it |
|---|---|
| `ATERNOS_SESSION` | Aternos.org → log in → `F12` → **Application → Cookies → https://aternos.org** → `ATERNOS_SESSION` |
| `ATERNOS_SERVER` | Same cookies list → `ATERNOS_SERVER` (your server id — a random 16-character string) |
| `AJAX_TOKEN` | `F12` → **Console** → run `window.AJAX_TOKEN` → copy the value |

You will also be asked to set a **panel password**. It protects the panel *and*
encrypts the session — the key is derived from it and **never written to disk**, so
there is no key file to lose (and no recovery if you forget the password).

aterkeep **auto-detects your server address** and writes everything into a single
`config/` folder: `aterkeep.json` (language, port, password verifier) and
`session.enc` (AES-256-GCM).

Because the key is not stored, aterkeep asks for the password on every launch. To
start it from a Termux boot script without a prompt, pass it in the environment:

```bash
ATERKEEP_KEY='your-panel-password' ./aterkeep
```

> **Tip:** On Android, Chrome/Firefox devtools ("F12") are available via the desktop layout. Easier route: log in on a laptop, grab the three values there, then paste them into Termux.

<details>
<summary><b>Advanced: manual import (for automation)</b></summary>

If you prefer a file (e.g. you generate the session on another machine), create `session.json`:

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

Then import it:

```bash
./aterkeep import session.json
```

</details>

---

## 5. Run

```bash
./aterkeep
```

Open the panel on your phone's browser: **http://127.0.0.1:4041** (or **http://localhost:4041**).

---

## 6. Keep it running in the background

By default Android kills background processes aggressively, and the daemon stops when your screen sleeps. Use a **wake lock** plus a persistent session:

```bash
# prevent the device from sleeping the process
termux-wake-lock

# run detached so it survives the terminal closing
nohup ./aterkeep > aterkeep.log 2>&1 &

# (later) release the wake lock when you stop
termux-wake-unlock
```

Alternatives for a persistent session:

- **tmux** — `pkg install tmux`, then `tmux`, run `./aterkeep`, detach with `Ctrl-b d`, reattach with `tmux a`.
- **screen** — `pkg install screen`, then `screen -S aterkeep`, run the daemon, detach with `Ctrl-a d`, reattach with `screen -r aterkeep`.

> **Important:** `termux-wake-lock` is what keeps the CPU polling every 30 s while the screen is off. Without it, Aternos will report your server as offline within minutes. Don't forget `termux-wake-unlock` when you stop the daemon, or it drains your battery.

---

## 7. Access from another device (optional, same Wi-Fi)

aterkeep binds to `127.0.0.1` only by default, so the panel is reachable just from the phone itself. A `ATERKEEP_HOST=0.0.0.0` option to expose the panel on your local network is **coming soon** — it is not available yet in the current build.

> **Security note (for when it ships):** binding to `0.0.0.0` exposes the dashboard to anyone on the same Wi-Fi, and the session cookies are decryptable on that machine. Only enable it on a trusted home network, never on public Wi-Fi.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| `port 4041 dinlenemiyor` / port busy | A previous instance is still running: `pkill aterkeep`, then relaunch. |
| Build fails / `cc` linker error | `pkg install clang make` (or build inside `proot-distro` Ubuntu — see step 2). |
| Build killed (no output, phone reboots) | Out of RAM. Use `proot-distro` + Ubuntu as in step 2, or build on a PC and copy the `aarch64` binary over. |
| Daemon dies when screen turns off | You forgot `termux-wake-lock`. Run it before starting the daemon. |
| Panel unreachable at `127.0.0.1:4041` | Confirm the daemon is running (`ps \| grep aterkeep`), and that no other app grabbed 4041. |
| `LOGGED OUT` after ~30 days | Aternos session expired. Re-run the onboarding wizard (or `aterkeep import` a fresh `session.json`). |

---

Back to the [main README](../README.md).
