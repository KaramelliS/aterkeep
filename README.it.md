# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="logo aterkeep" width="280"/>
</p>

**Gestore server Aternos & dashboard 24/7.** Un singolo binario Rust (~1.7 MB) tiene online il tuo server Minecraft Aternos gratuito tutto il giorno e ti offre un pannello web moderno — niente automazione del browser, solo HTTP.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a>
</p>

## Funzionalità

- **Ciclo keep-alive** — controlla ogni 90 s e riavvia il server automaticamente (disattivabile)
- **Dashboard web** — stato live, avvio/stop/riavvio, interruttore auto-start
- **Console server** — log del server in diretta dal browser
- **Editor impostazioni** — leggi/modifica `server.properties` dal pannello
- **Lista giocatori** — chi è online
- **Request inspector** — ogni chiamata HTTP con risposta JSON (didattico)
- **14 lingue** — interfaccia cambiabile nell'header
- **Sessione cifrata** — cookie in AES-256-GCM, la chiave non lascia mai la tua macchina

## Requisiti

- Windows 10/11 (usa `curl.exe` integrato)
- Toolchain Rust (solo per compilare)

## Installazione

```powershell
cd rust
cargo build --release
# binario: target/release/aterkeep.exe
```

## Esporta sessione (una volta)

1. Apri **https://aternos.org** e accedi.
2. `F12` → **Console**: `window.AJAX_TOKEN` → `token`; `window.generateAjaxToken()` → parte dopo `:` → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**: copia `ATERNOS_SESSION` e `ATERNOS_SERVER`
4. Crea `http/session.json` (formato: [English README](README.md#setup--export-your-session-once)):

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

5. Importa:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

Crea `session.enc` + `aterkeep.key` — **non perdere la chiave**, è l'unico modo per decifrare la sessione.

## Avvio

```powershell
.\target\release\aterkeep.exe
```

Apri **http://127.0.0.1:4041**.

## Schede del pannello

| Scheda | Funzione |
|---|---|
| **Stato** | badge di stato, controlli, auto-start, log live, inspector |
| **Console** | flusso log server (aggiorna 10 s) |
| **Impostazioni** | modifica `server.properties` e salva |
| **Giocatori** | elenco giocatori online |

**Interruttore auto-start importante:** spento = il server non viene mai riavviato. **Stop** lo spegne automaticamente.

## Durata sessione

I cookie di sessione Aternos durano **~30 giorni**. Quando il pannello mostra `OTURUM BİTTİ`/`LOGGED OUT`, ripeti l'export e reimporta.

## Sicurezza

- Sessione cifrata a riposo (`session.enc`, AES-256-GCM)
- `aterkeep.key` mai committato
- Stringhe API cifrate nel binario, decodificate a runtime con la tua chiave
- Pannello solo su `127.0.0.1`

## Licenza

**aterkeep è software commerciale — non è open source.**

Il codice è pubblicato solo per trasparenza e valutazione. È consentito l'uso
personale e non commerciale. Ridistribuzione, rivendita, opere derivate e uso
commerciale **non** sono consentiti. Termini completi: [LICENSE](LICENSE).

## Acquistare una licenza

Uso commerciale, ridistribuzione, white-labelling e accesso al codice del motore
keep-alive (`aterkeep-core`) richiedono una licenza commerciale a pagamento.

**Contatto:** berlaylc2138@gmail.com

## Avvertenza

Progetto indipendente — non affiliato ad Aternos GmbH o Mojang Studios.
