# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="logo aterkeep" width="280"/>
</p>

**Menedżer serwera Aternos i panel 24/7.** Pojedynczy plik binarny Rust (~1.7 MB) utrzymuje Twój darmowy serwer Minecraft Aternos online przez całą dobę i daje nowoczesny panel webowy — bez automatyzacji przeglądarki, czysty HTTP.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a> · <a href="README.ru.md">Русский</a> · <a href="README.zh.md">中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.nl.md">Nederlands</a>
</p>

## Funkcje

- **Automatyczne potwierdzanie kolejki** — gdy przychodzi twoja kolej, Aternos otwiera okno na około 30 sekund; bez odpowiedzi wracasz na koniec. To ten krok umożliwia pracę 24/7 bez nadzoru.
- **Loguje się za ciebie** — z kontem Aternos aterkeep sam pobiera ciasteczko sesji i odnawia je po wygaśnięciu. Bez DevTools i comiesięcznego kopiowania.
- **Bot anti-idle** — klient Minecrafta, który dołącza, gdy serwer działa, żeby nie został wyłączony jako pusty
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

## Instalacja (jednorazowo)

Uruchom plik binarny i otwórz **http://127.0.0.1:4041**. Kreator pyta o trzy
rzeczy: język panelu, hasło panelu i sesję Aternos.

**Hasło panelu** chroni panel *i* szyfruje sesję. Klucz **nigdy nie trafia na
dysk** — jest wyprowadzany z hasła przy każdym starcie (PBKDF2-HMAC-SHA256,
600 000 iteracji). **Nie ma odzyskiwania.**

**Dwa sposoby na sesję:**

**1. Konto Aternos (domyślnie).** Podaj nazwę użytkownika i hasło: aterkeep
loguje się czystym HTTP i sam pobiera ciasteczko. Jeśli masz kilka serwerów,
kreator zapyta, który utrzymywać. Dane logowania leżą w `config/session.enc`, pod
tym samym szyfrowaniem AES-256-GCM co ciasteczka, i trafiają wyłącznie do
`aternos.org`.

> Nie zadziała przy **uwierzytelnianiu dwuskładnikowym** ani gdy Aternos zażąda
> **captcha**; oba przypadki mają własny komunikat.

**2. Wklejenie ciasteczek (zapasowo).** Na `aternos.org`: F12 → **Network** → F5,
skopiuj całą linię `cookie:` dowolnego żądania i wykonaj `window.AJAX_TOKEN` w
**Console**. Tak utworzona sesja **nie odnawia się sama**.

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

Aternos wydaje ciasteczko z `Max-Age=2592000` — **dokładnie 30 dni**; zmierzone
w odpowiedzi logowania, nie zgadywane.

**Z kontem:** nic nie trzeba robić. Po wygaśnięciu demon loguje się ponownie i
działa dalej — jedna linia w dzienniku.

**Z wklejonymi ciasteczkami:** panel pokazuje odznakę `SESSION` i komunikat o
wygaśnięciu sesji — a nie zatrzymany serwer — z przyciskiem powrotu do kreatora.

Panel pokazuje też **wiek sesji**, a po pierwszym wygaśnięciu — jak długo
wytrzymała poprzednia.

Automatyczny start po restarcie: **[docs/AUTOSTART.md](docs/AUTOSTART.md)** (zadanie Windows z DPAPI, systemd, Termux:Boot).

## Bezpieczeństwo

- Sesja szyfrowana w spoczynku (`session.enc`, AES-256-GCM)
- **Brak pliku klucza na dysku** — klucz jest wyprowadzany z hasła (PBKDF2, 600 000 iteracji, losowa sól na instalację). Skopiowany folder `config/` jest bezużyteczny bez hasła
- **Panel wymaga logowania** — wszystkie endpointy za sesyjnym ciasteczkiem `HttpOnly`
- Ciągi API zaszyfrowane w binarnym pliku, odszyfrowywane w czasie działania Twoim kluczem
- Panel tylko na `127.0.0.1`

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
