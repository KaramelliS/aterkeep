# Anti-idle bot

Aternos stops a server once it has been empty for a while. The bot joins it as a
player so that never happens. It lives **inside the aterkeep binary** — there is
nothing to install and nothing to start by hand.

> **This used to be a Node.js program.** It was rewritten as a native Rust client
> because Node cannot ship inside an Android APK, so the bot did not exist at all
> on phones, and on desktop it demanded a Node install plus a large `npm install`.
> Behaviour and the panel's status contract are unchanged.

## What it does

1. **Pings the server first.** Aternos servers are often stopped or queued, and
   dialling straight into one produces a meaningless error. The ping also reads
   the server's version.
2. **Checks the version.** If the server is newer than the bot supports the panel
   shows `unsupported_version` and the bot does not try to connect.
3. **Joins as an offline (cracked) player.**
4. **Tries to hide:** `/gamemode spectator`, then `/tp` to the vanish
   coordinates — far away and out of sight.
5. **Looks around** every few seconds so it never reads as a frozen client.
6. **Reconnects on its own** when kicked or dropped, and waits — pinging — while
   the server is down.
7. Writes live state to `config/bot-status.json`, which the panel reads.

## Requirements

- **A cracked (offline-mode) server.** The bot has no Mojang account, so an
  `online-mode=true` server rejects it — reported as `online_mode`, with the fix
  spelled out in the panel. Aternos free servers are cracked by default.
- **Version 1.21.11 or older.** The bot speaks protocol 774. This is a narrower
  range than the old Node bot, which got wide version coverage for free from
  `minecraft-data`; here the packet layouts are written out explicitly, so each
  version has to be added deliberately.
- **Operator rights, if you want it hidden.** See below.
- No Node.js, no npm, no `bot/` folder. Those are gone.

## Operator rights and the "visible bot" warning

`/gamemode` and `/tp` require operator permission. Without it the server silently
refuses both — the refusal goes to the bot as a chat message, not as a protocol
error — and the bot ends up standing at spawn as an ordinary visible player.

**It still does its job in that state:** it occupies a slot, so the server is not
stopped for being empty. It just isn't invisible.

The bot detects this rather than assuming success: after sending `/tp` it watches
for the server's position update. If that never puts it at the vanish
coordinates, the panel shows `vanish_no_permission` and `vanished: false`. (The
old Node bot reported vanish as active either way, which was simply wrong.)

To fix it, add the bot's name under **Players → Operators** in the Aternos panel.

## Settings (`config/bot.json`)

Written by the daemon at runtime; template in `config/bot.example.json`.

| Field | Type | Meaning |
|---|---|---|
| `enabled` | bool | Whether the bot runs (also togglable from the panel) |
| `name` | string | Player name. Empty means a random one per session. |
| `host` | string | Server address; the daemon fills it from the server id if blank |
| `port` | number | Server port |
| `vanish_x` / `vanish_y` / `vanish_z` | number | Where to teleport to hide |

## Status fields (`config/bot-status.json`)

| Field | Meaning |
|---|---|
| `connected` | Is the bot on the server |
| `state` | `starting` / `connecting` / `online` / `waiting` / `kicked` / `disconnected` / `error` / `unsupported_version` / `stopped` |
| `name` | Active bot name |
| `vanished` | Hiding **confirmed** — not merely attempted |
| `server_version` / `server_protocol` | From the ping |
| `max_supported_version` / `max_supported_protocol` | What the bot speaks |
| `error` | Technical detail, if any |
| `error_code` | Machine-readable cause: `online_mode`, `whitelist`, `vanish_no_permission`, `unsupported_version`. The panel turns this into an instruction in your language. |
| `host` / `port` | Target server |

## Running it by hand

Normally the daemon starts and stops the bot for you. To reproduce a problem, or
to check the protocol against a specific server, run it standalone — no session,
no panel, no password:

```bash
aterkeep bot-probe <host> [port]      # port defaults to 25565
```

It prints the ping result, then the whole join sequence, then keeps going until
you press Ctrl-C. This replaces the old `node bot/index.js`.

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| Stuck in `waiting` | Server is stopped or queued. The bot pings until it comes up. Normal. |
| `unsupported_version` | Server is newer than protocol 774. Pick **Software → Vanilla 1.21.x** in the Aternos panel. |
| Repeated `kicked` with `online_mode` | Server has `online-mode=true`. Set it to `false` in the Settings tab. |
| `vanish_no_permission` | Bot is not an operator — see above. Harmless for uptime. |
| `whitelist` | Whitelist is on and the bot is not on it. |
| Bot never appears at all | Check `host`/`port` in `config/bot.json`, then run `aterkeep bot-probe` against the same address to see the failure directly. |

---

Back to the [main README](../README.md).
