# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep logo" width="280"/>
</p>

**Self-hosted Aternos-Serververwaltung & 24/7-Dashboard.** Eine einzelne Rust-Binary (~1.7 MB) hält deinen kostenlosen Aternos-Minecraft-Server rund um die Uhr online und gibt dir ein modernes Webpanel — keine Browser-Automation, reines HTTP.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a>
</p>

## Funktionen

- **Automatische Warteschlangen-Bestätigung** — wenn du an der Reihe bist, öffnet Aternos ein ~30-Sekunden-Fenster; antwortet niemand, landest du hinten. Dieser Schritt macht den unbeaufsichtigten 24/7-Betrieb überhaupt erst möglich.
- **Meldet sich für dich an** — mit deinem Aternos-Konto holt aterkeep das Session-Cookie selbst und erneuert es, wenn es abläuft. Keine DevTools, kein monatliches Kopieren.
- **Anti-Idle-Bot** — ein Minecraft-Client, der beitritt, sobald der Server läuft, damit er nicht wegen Leerlauf abgeschaltet wird
- **Keep-alive-Schleife** — prüft alle 90 s und startet den Server automatisch neu (abschaltbar)
- **Web-Dashboard** — Live-Status, Start/Stopp/Neustart, Auto-Start-Schalter
- **Server-Konsole** — Live-Serverlog im Browser
- **Einstellungs-Editor** — `server.properties` direkt aus dem Panel lesen/ändern
- **Spielerliste** — wer ist online
- **Request Inspector** — jeder HTTP-Aufruf mit JSON-Antwort (lehrreich)
- **14 Sprachen** — UI im Header umschaltbar
- **Verschlüsselte Session** — Cookies als AES-256-GCM, Schlüssel verlässt den PC nie

## Voraussetzungen

- Windows 10/11 (nutzt das integrierte `curl.exe`)
- Rust-Toolchain (nur zum Kompilieren)

## Installation

```powershell
cd rust
cargo build --release
# Binary: target/release/aterkeep.exe
```

## Einrichtung (einmalig)

Binary starten und **http://127.0.0.1:4041** öffnen. Der Assistent fragt drei
Dinge: Panel-Sprache, Panel-Passwort, Aternos-Sitzung.

Das **Panel-Passwort** schützt das Panel *und* verschlüsselt die Sitzung. Der
Schlüssel wird **nie gespeichert**, sondern bei jedem Start daraus abgeleitet
(PBKDF2-HMAC-SHA256, 600 000 Runden). **Es gibt keine Wiederherstellung.**

**Zwei Wege für die Aternos-Sitzung:**

**1. Aternos-Konto (Standard).** Benutzername und Passwort eingeben — aterkeep
meldet sich über reines HTTP an und holt das Cookie selbst. Bei mehreren Servern
fragt der Assistent, welcher wachgehalten werden soll. Die Zugangsdaten liegen in
`config/session.enc` unter derselben AES-256-GCM-Verschlüsselung wie die Cookies
und gehen ausschließlich an `aternos.org`.

> Funktioniert nicht bei **Zwei-Faktor-Authentifizierung** oder wenn Aternos ein
> **Captcha** verlangt. Beides wird als eigene Meldung angezeigt.

**2. Cookies einfügen (Rückfallweg).** Auf `aternos.org`: F12 → **Network** → F5,
die komplette `cookie:`-Zeile einer Anfrage kopieren, in der **Console**
`window.AJAX_TOKEN` ausführen. Beides in den Assistenten einfügen. So eingerichtete
Sitzungen **erneuern sich nicht selbst**.

## Starten

```powershell
.\target\release\aterkeep.exe
```

Dann **http://127.0.0.1:4041** im Browser öffnen.

## Panel-Tabs

| Tab | Funktion |
|---|---|
| **Status** | Status-Badge, Steuerung, Auto-Start-Schalter, Live-Log, Request Inspector |
| **Konsole** | Serverlog-Stream (10 s Aktualisierung) |
| **Einstellungen** | `server.properties` bearbeiten & speichern |
| **Spieler** | Online-Spielerliste |

**Auto-Start-Schalter ist wichtig:** Aus = der Server wird nie wieder gestartet. **Stopp** schaltet ihn automatisch aus.

## Sitzungsdauer

Aternos setzt das Cookie mit `Max-Age=2592000` — **genau 30 Tage**, aus der
Login-Antwort gemessen, nicht geraten.

**Mit Konto eingerichtet:** nichts zu tun. Läuft die Sitzung ab, meldet sich der
Daemon neu an und macht weiter — eine Zeile im Protokoll.

**Mit eingefügten Cookies:** das Panel zeigt ein `SESSION`-Abzeichen und einen
Hinweis, dass die Sitzung abgelaufen ist — früher sah das wie ein gestoppter
Server aus — mit einer Schaltfläche zurück in den Assistenten.

Das Panel zeigt außerdem das **Sitzungsalter** und nach dem ersten Ablauf, wie
lange die vorherige gehalten hat.

Automatischer Start nach dem Neustart: **[docs/AUTOSTART.md](docs/AUTOSTART.md)** (Windows-Aufgabe mit DPAPI, systemd, Termux:Boot).

## Sicherheit

- Session verschlüsselt (`session.enc`, AES-256-GCM)
- **Keine Schlüsseldatei auf der Festplatte** — der Schlüssel wird aus dem Passwort abgeleitet (PBKDF2, 600 000 Iterationen, zufälliges Salt pro Installation). Ein kopierter `config/`-Ordner nützt ohne Passwort nichts
- **Das Panel verlangt eine Anmeldung** — alle Endpunkte hinter einem `HttpOnly`-Sitzungscookie
- API-Strings sind in der Binary verschlüsselt, werden nur zur Laufzeit mit deinem Schlüssel entschlüsselt
- Panel nur auf `127.0.0.1` gebunden

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

## Lizenz

**aterkeep ist kommerzielle Software — kein Open Source.**

Der Quellcode wird ausschließlich zur Transparenz und Evaluierung veröffentlicht.
Private, nicht-kommerzielle Nutzung ist gestattet. Weiterverbreitung, Verkauf,
abgeleitete Werke und kommerzielle Nutzung sind **nicht** gestattet. Vollständige
Bedingungen: [LICENSE](LICENSE).

## Lizenz erwerben

Kommerzielle Nutzung, Weiterverbreitung, White-Labelling und Quellcode-Zugriff auf
die Keep-Alive-Engine (`aterkeep-core`) erfordern eine kostenpflichtige Lizenz.

**Kontakt:** berlaylc2138@gmail.com

## Haftungsausschluss

Unabhängiges Projekt — nicht mit Aternos GmbH oder Mojang Studios verbunden.
