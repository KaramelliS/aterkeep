# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="logo aterkeep" width="280"/>
</p>

**Menedżer serwera Aternos i panel 24/7.** Pojedynczy plik binarny Rust (~1.7 MB) utrzymuje Twój darmowy serwer Minecraft Aternos online przez całą dobę i daje nowoczesny panel webowy — bez automatyzacji przeglądarki, czysty HTTP.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a> · <a href="README.ru.md">Русский</a> · <a href="README.ar.md">العربية</a> · <a href="README.zh.md">中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.nl.md">Nederlands</a>
</p>

## Funkcje

- **Pętla keep-alive** — sprawdza co 90 s i automatycznie restartuje serwer (wyłączalna)
- **Panel webowy** — status na żywo, start/stop/restart, przełącznik auto-start
- **Konsola serwera** — log serwera na żywo w przeglądarce
- **Edytor ustawień** — czytaj/zmieniaj `server.properties` z panelu
- **Lista graczy** — kto jest online
- **Request inspector** — każde żądanie HTTP z odpowiedzią JSON (edukacyjne)
- **14 języków** — interfejs przełączany w nagłówku
- **Zaszyfrowana sesja** — cookies w AES-256-GCM, klucz nigdy nie opuszcza Twojego komputera

## Wymagania

- Windows 10/11 (używa wbudowanego `curl.exe`)
- Toolchain Rust (tylko do kompilacji)

## Instalacja

```powershell
cd rust
cargo build --release
# binarny: target/release/aterkeep.exe
```

## Eksport sesji (raz)

1. Otwórz **https://aternos.org** i zaloguj się.
2. `F12` → **Console**: `window.AJAX_TOKEN` → `token`; `window.generateAjaxToken()` → część po `:` → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**: skopiuj `ATERNOS_SESSION` i `ATERNOS_SERVER`
4. Utwórz `http/session.json` (format: [English README](README.md#setup--export-your-session-once)):

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

5. Zaimportuj:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

Podczas instalacji ustawiasz **hasło panelu**: chroni panel *i* szyfruje sesję. Klucz **nigdy nie jest zapisywany na dysku** — jest wyprowadzany z hasła przy każdym uruchomieniu. Wszystko trafia do jednego folderu `config/`. **Zapomniane hasło oznacza brak odzyskania.** Do pracy bez nadzoru: `ATERKEEP_KEY='twoje-haslo' ./aterkeep`

## Uruchomienie

```powershell
.\target\release\aterkeep.exe
```

Otwórz **http://127.0.0.1:4041**.

## Zakładki panelu

| Zakładka | Funkcja |
|---|---|
| **Status** | odznaka statusu, sterowanie, auto-start, log na żywo, inspector |
| **Konsola** | strumień logu serwera (odświeżanie 10 s) |
| **Ustawienia** | edycja `server.properties` i zapis |
| **Gracze** | lista graczy online |

**Przełącznik auto-start jest ważny:** wyłączony = serwer nigdy nie zostanie uruchomiony ponownie. **Stop** wyłącza go automatycznie.

## Czas życia sesji

Cookies sesji Aternos działają **~30 dni**. Gdy panel pokaże `OTURUM BİTTİ`/`LOGGED OUT`, powtórz eksport i zaimportuj ponownie.

## Bezpieczeństwo

- Sesja szyfrowana w spoczynku (`session.enc`, AES-256-GCM)
- **Brak pliku klucza na dysku** — klucz jest wyprowadzany z hasła (PBKDF2, 600 000 iteracji, losowa sól na instalację). Skopiowany folder `config/` jest bezużyteczny bez hasła
- **Panel wymaga logowania** — wszystkie endpointy za sesyjnym ciasteczkiem `HttpOnly`
- Ciągi API zaszyfrowane w binarnym pliku, odszyfrowywane w czasie działania Twoim kluczem
- Panel tylko na `127.0.0.1`

## Licencja

**aterkeep to oprogramowanie komercyjne — nie jest open source.**

Kod źródłowy publikowany jest wyłącznie w celach przejrzystości i oceny. Dozwolone
jest osobiste, niekomercyjne użycie. Redystrybucja, odsprzedaż, dzieła pochodne
i użycie komercyjne są **zabronione**. Pełne warunki: [LICENSE](LICENSE).

## Zakup licencji

Użycie komercyjne, redystrybucja, white-labelling oraz dostęp do kodu silnika
keep-alive (`aterkeep-core`) wymagają płatnej licencji komercyjnej.

**Kontakt:** berlaylc2138@gmail.com

## Zastrzeżenie

Projekt niezależny — niepowiązany z Aternos GmbH ani Mojang Studios.
