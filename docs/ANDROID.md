# aterkeep on Android (APK)

Install one APK, type your panel password, and the keep-alive daemon runs in a
foreground service. No Termux, no `pkg install`, no Rust toolchain, no shell.

> **arm64 only.** The APK carries two prebuilt `aarch64` binaries, so it installs
> on arm64 phones (essentially every phone made in the last decade) and refuses
> 32-bit ones rather than installing and doing nothing.

---

## Install

1. Download `aterkeep-android-arm64.apk` from [Releases](../../releases).
2. Open it and allow installing from an unknown source when prompted.
3. Launch **aterkeep**.

## First run

1. **Type a panel password.** On a fresh install you are choosing it — use the
   same one in the panel's setup wizard on the next screen. It protects the panel
   *and* derives the key that encrypts your Aternos session.
2. Optionally tick **remember the password on this device** — see
   [What the "remember" option costs you](#what-the-remember-option-costs-you).
3. Press **Baslat**. The app waits for the daemon, then shows the normal web
   panel — the same 14-language panel as the desktop build, in a WebView.
4. Complete the panel's setup wizard (Aternos account login, or pasted cookies).

## Grant the battery exemption

The app asks for this on first launch, and you should say yes. Android's Doze
mode throttles CPU for background apps; the keep-alive loop polls every 30
seconds, and if it gets throttled Aternos sees your server as unattended. The
service also holds a partial wake lock, but **a wake lock alone is not enough** —
the exemption is the part that matters.

## What actually keeps it alive

A **foreground service** with a persistent notification. This is the only
reliable mechanism on modern Android, and it is why the APK exists at all:

- Since Android 12, the system kills "phantom processes" — untracked child
  processes of an app, especially ones using CPU. That is exactly what a daemon
  launched from a Termux shell looks like, which is why `termux-wake-lock` is not
  enough and [still isn't on Android 15](https://github.com/termux/termux-app/issues/5150).
- The service declares `foregroundServiceType="specialUse"`, deliberately **not**
  `dataSync`. On Android 15 `dataSync` is capped at 6 hours per 24 and cannot be
  started from `BOOT_COMPLETED` — both fatal for something whose whole job is to
  run continuously.

Tap **Durdur** in the notification to stop it.

## The anti-idle bot does not run on Android

This is the one real gap, and it is worth being precise about.

The anti-idle bot is a Node.js program built on mineflayer. Node cannot ship
inside the APK, so on Android you get the daemon **without** the bot. Practically:

| | Desktop (with bot) | Android APK (no bot) |
|---|---|---|
| Queue confirmation | yes | yes |
| Auto session renewal | yes | yes |
| Restarts a stopped server | yes | yes |
| Keeps an *empty* server from stopping | yes | **no** |

Aternos stops a server once it has been empty for a while. With the bot, a
(hidden, spectator) player is always present so that never triggers. Without it,
your server will be stopped and then restarted by the daemon — you get an
**auto-restart loop**, not seamless uptime. For a server people actually play on
that is usually fine; for one you want reachable at every instant, it is not.

Removing this gap means reimplementing the bot natively in Rust so it ships
inside the binary on every platform. That is tracked as planned work, not
something you can enable today.

## Autostart after reboot

Only happens if you ticked **remember the password**. Without a stored password
the daemon cannot decrypt the session, so starting it on boot would produce a
service that runs and immediately fails — worse than not starting.

## What the "remember" option costs you

aterkeep's normal posture is that the session key is never written to disk and
the password is asked for on every launch. Keeping that exactly as-is on a phone
would mean 24/7 stops every time the device reboots until you open the app by
hand.

So it is your call, and it defaults to **off**:

- **off** — password lives only in memory. Same guarantee as desktop. No autostart.
- **on** — password is stored in `EncryptedSharedPreferences`, whose key lives in
  the Android Keystore (hardware-backed where available) and is not readable by
  other apps. Enables autostart. This does weaken the original guarantee, which
  is why it is opt-in rather than the default.

If the stored password ever stops working — a device lock change or a restore
from backup can invalidate the Keystore key — the app simply asks again.

## Under the hood

Worth knowing if you are debugging it:

- The daemon is a real `aarch64` executable, packaged as `libaterkeep.so`.
  Android 10+ forbids executing anything in an app's data directory (W^X); files
  extracted from the APK into `nativeLibraryDir` are the only executable ones,
  and only `lib*.so` names get extracted there. Hence the name — it is not a
  library.
- A **static curl** (`libcurlx.so`) ships alongside it. Every Aternos request
  goes through curl on purpose: curl's TLS fingerprint gets past Cloudflare,
  where a native Rust HTTP client was answered with `403 Just a moment`. The
  bundled build is static-PIE, so it satisfies Android's PIE requirement and
  needs no bionic.
- Because that curl is static it cannot use Android's DNS resolver — there is no
  `/etc/resolv.conf` on Android — so it is invoked with
  `--doh-url https://1.1.1.1/dns-query` and resolves names itself.
- A `cacert.pem` is bundled and used for both curl (`--cacert`) and the
  WebSocket's OpenSSL (`SSL_CERT_FILE`); neither has a usable built-in
  certificate path on Android.
- Panel logs land in `aterkeep.log` inside the app's private storage, capped at
  512 KB. The gate screen shows the tail if the panel fails to come up.

## Troubleshooting

| Problem | Fix |
|---|---|
| "Bu cihaz arm64 degil" | 32-bit device; this APK cannot support it. |
| Stuck on "Panel baslatiliyor…" | The gate screen prints the daemon's own log tail after ~60 s — read it. Wrong password is the usual cause. |
| Panel opens, then everything fails with a network error | Almost always DNS or certificates. Confirm the device has working internet, then check the log for `curl hatasi`. |
| Service disappears after a while | Battery exemption was not granted, or the vendor ROM (Xiaomi, Huawei, Samsung) has its own aggressive killer — allow "autostart"/"no restrictions" for aterkeep in system settings. |
| Server keeps stopping and restarting | Expected without the anti-idle bot; see the section above. |
| `LOGGED OUT` after ~30 days | Session expired and could not renew. Re-run the panel's setup wizard. |

---

Termux is still supported for people who want a shell instead of an app — see the
[Termux guide](TERMUX.md). Back to the [main README](../README.md).
