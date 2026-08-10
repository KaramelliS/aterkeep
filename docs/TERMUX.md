# aterkeep on Android (Termux)

Run the aterkeep keep-alive daemon on your phone with [Termux](https://termux.dev/). It is a single native `aarch64-linux-android` binary, so it needs no emulator — it runs straight on the ARM CPU.

> **Most people should install the [APK](ANDROID.md) instead.** It carries the same
> daemon, needs no shell and no package installs, and — the part that matters —
> runs it inside a **foreground service**, which is the only reliable way to keep a
> process alive on modern Android. A daemon started from a Termux shell is an
> untracked child process, and since Android 12 the system reaps those; that is
> still true on Android 15 even with `termux-wake-lock` held. Use Termux if you
> specifically want a shell; use the APK if you want it to stay up.

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

## 3. Get the binary

### Option A — download it (what you want)

```bash
cd ~
curl -fsSLO https://github.com/KaramelliS/aterkeep/releases/latest/download/aterkeep-android-arm64
mv aterkeep-android-arm64 aterkeep
chmod +x aterkeep
./aterkeep --version
```

### Option B — build from source

Only possible with read access to the private
[`aterkeep-core`](https://github.com/KaramelliS/aterkeep-core) crate, which the
build pulls as a git dependency. **Without that access this build fails**, so if
you are not the maintainer, use Option A.

```bash
git clone https://github.com/KaramelliS/aterkeep.git
cd aterkeep
cargo build --release --manifest-path rust/Cargo.toml
cp target/release/aterkeep ~/aterkeep
cd ~
```

Building needs the `rust clang make` packages from step 2 and roughly 2 GB of
free space.

---

## 4. First-run setup (onboarding wizard)

On first run aterkeep walks you through entering your Aternos session interactively:

```bash
./aterkeep
```

> **Prefer the web wizard — it is the only one that can renew your session.**
> The terminal wizard below only accepts pasted cookies; it stores no account
> credentials, so when the session expires in ~30 days it cannot log back in and
> you have to repeat this by hand. The web wizard also offers **Aternos account
> login**, after which aterkeep signs in again by itself.
>
> The terminal wizard runs only when stdin is a TTY, so detach stdin to get the
> web one:
>
> ```bash
> ./aterkeep < /dev/null
> ```
>
> Then open **http://127.0.0.1:4041** in the phone's browser and complete setup
> there. Everything after this section still applies.

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

aterkeep binds to `127.0.0.1` only by default, so the panel is reachable just from the phone itself. To expose it on your local network, set `bind` in `config/aterkeep.json` and restart the daemon:

```json
{
  "bind": "0.0.0.0",
  "port": 4041
}
```

The daemon prints a warning on startup whenever `bind` is not loopback, so you can tell at a glance whether the panel is exposed. Then reach it at `http://<phone-ip>:4041` — `ip addr show wlan0` gives you the address.

> **Security note:** binding to `0.0.0.0` exposes the dashboard to anyone on the same Wi-Fi, and the session cookies are decryptable on that machine. The panel password is the only thing in the way. Only do this on a trusted home network, never on public Wi-Fi.

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
