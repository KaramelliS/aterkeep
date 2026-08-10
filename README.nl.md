# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep logo" width="280"/>
</p>

**Aternos-servermanager & 24/7-dashboard.** Eén Rust-binary (~1.7 MB) houdt je gratis Aternos-Minecraftserver dag en nacht online en geeft je een modern webpaneel — geen browserautomatisering, puur HTTP.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a> · <a href="README.ru.md">Русский</a> · <a href="README.zh.md">中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a>
</p>

## Functies

- **Automatische wachtrijbevestiging** — als je aan de beurt bent opent Aternos een venster van ongeveer 30 seconden; zonder antwoord ga je weer achteraan. Deze stap maakt onbeheerd 24/7 draaien mogelijk.
- **Logt voor je in** — met je Aternos-account haalt aterkeep de sessiecookie zelf op en vernieuwt hem als hij verloopt. Geen DevTools, geen maandelijks kopiëren.
- **Anti-idle bot** — een Minecraft-client die aansluit zodra de server draait, zodat hij niet wordt afgesloten omdat hij leeg is
- **Keep-alive-loop** — controleert elke 90 s en herstart de server automatisch (uitschakelbaar)
- **Webdashboard** — live status, start/stop/herstart, auto-startschakelaar
- **Serverconsole** — live serverlog in je browser
- **Instellingeneditor** — lees/wijzig `server.properties` vanuit het paneel
- **Spelerlijst** — wie is online
- **Request-inspector** — elke HTTP-call met JSON-antwoord (educatief)
- **14 talen** — UI wisselbaar in de header
- **Versleutelde sessie** — cookies in AES-256-GCM, de sleutel verlaat je pc nooit

## Vereisten

- Windows 10/11 (gebruikt ingebouwde `curl.exe`)
- Rust-toolchain (alleen om te compileren)

## Installatie

```powershell
cd rust
cargo build --release
# binary: target/release/aterkeep.exe
```

## Installatie (eenmalig)

Start het binaire bestand en open **http://127.0.0.1:4041**. De wizard vraagt om
drie dingen: paneeltaal, paneelwachtwoord en Aternos-sessie.

Het **paneelwachtwoord** beschermt het paneel *en* versleutelt de sessie. De
sleutel wordt **nooit op schijf geschreven**, maar bij elke start uit het
wachtwoord afgeleid (PBKDF2-HMAC-SHA256, 600 000 iteraties). **Geen herstel.**

**Twee manieren voor de sessie:**

**1. Aternos-account (standaard).** Vul je gebruikersnaam en wachtwoord in:
aterkeep logt via gewone HTTP in en haalt de cookie zelf op. Bij meerdere servers
vraagt de wizard welke je wilt draaiend houden. De gegevens staan in
`config/session.enc`, onder dezelfde AES-256-GCM-versleuteling als de cookies, en
gaan alleen naar `aternos.org`.

> Werkt niet met **tweefactorauthenticatie** of wanneer Aternos een **captcha**
> eist; beide worden met een eigen melding gemeld.

**2. Cookies plakken (terugval).** Op `aternos.org`: F12 → **Network** → F5,
kopieer de volledige `cookie:`-regel van een verzoek en voer `window.AJAX_TOKEN`
uit in de **Console**. Zo'n sessie **vernieuwt zichzelf niet**.

## Starten

```powershell
.\target\release\aterkeep.exe
```

Open **http://127.0.0.1:4041**.

## Paneltabbladen

| Tabblad | Functie |
|---|---|
| **Status** | statusbadge, bediening, auto-start, live log, inspector |
| **Console** | serverlog-stream (10 s verversing) |
| **Instellingen** | `server.properties` bewerken en opslaan |
| **Spelers** | online spelerlijst |

**Auto-startschakelaar is belangrijk:** uit = de server wordt nooit herstart. **Stoppen** schakelt hem automatisch uit.

## Levensduur van de sessie

Aternos geeft de cookie mee met `Max-Age=2592000` — **precies 30 dagen**,
gemeten aan het inlogantwoord, niet geraden.

**Met account:** niets te doen. Bij het verlopen logt de daemon opnieuw in en
gaat door — één regel in het logboek.

**Met geplakte cookies:** het paneel toont een `SESSION`-badge en een melding dat
de sessie is verlopen — geen gestopte server — met een knop terug naar de wizard.

Het paneel toont ook de **sessieleeftijd** en, na de eerste keer verlopen, hoe
lang de vorige het volhield.

Automatisch starten na een herstart: **[docs/AUTOSTART.md](docs/AUTOSTART.md)** (Windows-taak met DPAPI, systemd, Termux:Boot).

## Beveiliging

- Sessie versleuteld opgeslagen (`session.enc`, AES-256-GCM)
- **Geen sleutelbestand op schijf** — de sleutel wordt afgeleid uit het wachtwoord (PBKDF2, 600 000 iteraties, willekeurige salt per installatie). Een gekopieerde `config/`-map is nutteloos zonder wachtwoord
- **Het paneel vereist inloggen** — alle endpoints achter een `HttpOnly`-sessiecookie
- API-strings zijn versleuteld in de binary, worden bij runtime met jouw sleutel ontsleuteld
- Paneel bindt alleen aan `127.0.0.1`

## Licentie

**aterkeep is commerciële software — geen open source.**

De broncode is uitsluitend gepubliceerd voor transparantie en evaluatie.
Persoonlijk, niet-commercieel gebruik is toegestaan. Herdistributie, doorverkoop,
afgeleide werken en commercieel gebruik zijn **niet** toegestaan. Volledige
voorwaarden: [LICENSE](LICENSE).

## Een licentie kopen

Commercieel gebruik, herdistributie, white-labelling en broncodetoegang tot de
keep-alive-engine (`aterkeep-core`) vereisen een betaalde commerciële licentie.

**Contact:** berlaylc2138@gmail.com

## Disclaimer

Onafhankelijk project — niet verbonden aan Aternos GmbH of Mojang Studios.
