# Autostart — running aterkeep 24/7

aterkeep derives its session-encryption key from your panel password and never
writes that key to disk. That is what makes a stolen `config/` folder useless:
without the password, `session.enc` cannot be decrypted.

The cost is that a rebooted machine has nobody to type the password. If you
want aterkeep to come back on its own, the password has to be stored somewhere.
Each installer below stores it as safely as the platform allows, and each one
tells you exactly what you are trading away. **If you would rather not store it,
skip this page** and start the daemon by hand:

```
ATERKEEP_KEY='your-panel-password' ./aterkeep        # Linux / macOS / Termux
$env:ATERKEEP_KEY='your-panel-password'; .\aterkeep.exe   # Windows
```

---

## Windows — scheduled task at logon

```powershell
.\scripts\install-autostart.ps1
```

Prompts for the panel password, then stores it with **DPAPI** and registers a
task that starts aterkeep when you log in.

DPAPI binds the encrypted blob to **your user account on this machine**. Copying
`config/` — including `autostart.key` — to another machine or another user
account leaves it undecryptable, so the "a stolen folder is worthless" property
survives. What you give up is protection against someone who is already logged
in as you on this machine; at that point they could read your files anyway.

```powershell
Start-ScheduledTask -TaskName aterkeep      # start now
Get-Content .\autostart.log                 # launcher log
Get-Content .\aterkeep.out.log              # daemon output
.\scripts\install-autostart.ps1 -Remove     # uninstall + delete stored password
```

For unattended provisioning across several machines there is `-Password`, but it
puts the password in your shell history — prefer the prompt.

## Linux — systemd

```bash
sudo ./scripts/install-systemd.sh /path/to/install/dir
```

Writes the password to `/etc/aterkeep/aterkeep.env` with mode `0600`, owned by
the service user, and installs a unit that restarts on failure.

**The tradeoff is real:** anyone who can read that file — the service user, or
root — owns your Aternos session. There is no DPAPI equivalent here. The unit
sets `LimitCORE=0` so the password cannot leak through a core dump, plus
`NoNewPrivileges`, `PrivateTmp`, `ProtectSystem=full` and `ProtectHome=read-only`.

```bash
systemctl status aterkeep           # state
journalctl -u aterkeep -f           # logs
systemctl disable --now aterkeep && sudo rm -f /etc/systemd/system/aterkeep.service /etc/aterkeep/aterkeep.env
```

## Termux (Android) — Termux:Boot

```bash
./scripts/install-termux-boot.sh /path/to/install/dir
```

Requires the **Termux:Boot** app from F-Droid, opened at least once — without
that, Android never runs `~/.termux/boot`.

The password lands in `~/.aterkeep.env` (`0600`). Android's per-app sandboxing
keeps other apps out; physical or rooted access does not.

The boot script calls `termux-wake-lock` first. Without it Android suspends the
process as soon as the screen turns off, the keep-alive loop stops, and the
server shuts down — the wake lock is a precondition for 24/7 on Android, not a
nicety.

---

## Which file holds what

| Path | Contents | Protection |
|---|---|---|
| `config/session.enc` | Aternos cookies | AES-256-GCM, key derived from the panel password |
| `config/aterkeep.json` | port, language, password hash | hash is PBKDF2, 600k iterations, per-install salt |
| `config/autostart.key` (Windows) | panel password | DPAPI, bound to user + machine |
| `/etc/aterkeep/aterkeep.env` (Linux) | panel password | file permissions only (`0600`) |
| `~/.aterkeep.env` (Termux) | panel password | file permissions + Android app sandbox |

Never share `config/`. It is the whole account.
