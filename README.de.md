# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep logo" width="280"/>
</p>

**Self-hosted Aternos-Serververwaltung & 24/7-Dashboard.** Eine einzelne Rust-Binary (~1.7 MB) hält deinen kostenlosen Aternos-Minecraft-Server rund um die Uhr online und gibt dir ein modernes Webpanel — keine Browser-Automation, reines HTTP.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a>
</p>

## Funktionen

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

## Session exportieren (einmalig)

1. **https://aternos.org** öffnen und anmelden.
2. `F12` → **Console**: `window.AJAX_TOKEN` → `token`; `window.generateAjaxToken()` → Teil nach `:` → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**: `ATERNOS_SESSION` und `ATERNOS_SERVER` kopieren
4. `http/session.json` erstellen (Format: [English README](README.md#setup--export-your-session-once)):

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

5. Importieren:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

Bei der Einrichtung legst du ein **Panel-Passwort** fest: Es schützt das Panel *und* verschlüsselt die Session. Der Schlüssel wird **nie auf die Festplatte geschrieben**, sondern bei jedem Start aus dem Passwort abgeleitet. Alles liegt in einem einzigen `config/`-Ordner. **Ohne Passwort gibt es keine Wiederherstellung.** Für unbeaufsichtigten Betrieb: `ATERKEEP_KEY='dein-passwort' ./aterkeep`

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

## Session-Lebensdauer

Aternos-Session-Cookies halten **~30 Tage**. Zeigt das Panel `OTURUM BİTTİ`/`LOGGED OUT`, Export-Schritte wiederholen und neu importieren.

## Sicherheit

- Session verschlüsselt (`session.enc`, AES-256-GCM)
- **Keine Schlüsseldatei auf der Festplatte** — der Schlüssel wird aus dem Passwort abgeleitet (PBKDF2, 600 000 Iterationen, zufälliges Salt pro Installation). Ein kopierter `config/`-Ordner nützt ohne Passwort nichts
- **Das Panel verlangt eine Anmeldung** — alle Endpunkte hinter einem `HttpOnly`-Sitzungscookie
- API-Strings sind in der Binary verschlüsselt, werden nur zur Laufzeit mit deinem Schlüssel entschlüsselt
- Panel nur auf `127.0.0.1` gebunden

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
