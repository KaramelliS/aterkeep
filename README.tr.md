# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep logo" width="280"/>
</p>

**Aternos sunucu yöneticisi & 7/24 bekçi paneli.** Tek bir Rust binary'si (~1.7 MB) ile ücretsiz Aternos Minecraft sunucunu sürekli ayakta tutar ve modern bir web paneliyle yönetmeni sağlar — tarayıcı otomasyonu yok, saf HTTP.

<p align="center">
  <a href="README.md">English</a>
</p>

## Özellikler

- **Keep-alive döngüsü** — 90 saniyede bir kontrol, sunucu kapanınca otomatik başlat (açılıp kapanabilir)
- **Web panel** — canlı durum, başlat/durdur/yeniden başlat, oto-başlat anahtarı
- **Sunucu konsolu** — tarayıcıdan canlı sunucu logu
- **Ayarlar editörü** — `server.properties` değerlerini panelden oku/değiştir
- **Oyuncu listesi** — kim çevrimiçi, anlık gör
- **Request inspector** — her HTTP isteği ve JSON cevabı (eğitici)
- **14 dil** — panel arayüzü header'dan değiştirilebilir
- **Şifreli oturum** — çerezler AES-256-GCM, anahtar makineden çıkmaz

## Gereksinimler

- Windows 10/11 (dahili `curl.exe` kullanır)
- Rust toolchain (sadece kaynaktan derlemek için)

## Kurulum

```powershell
cd rust
cargo build --release
# binary: target/release/aterkeep.exe
```

## Session dışa aktarma (bir kez)

1. **https://aternos.org** aç, giriş yap.
2. `F12` → **Console**: `window.AJAX_TOKEN` → `token`; `window.generateAjaxToken()` → ':' sonrası → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**: `ATERNOS_SESSION` ve `ATERNOS_SERVER` değerlerini kopyala
4. `http/session.json` oluştur (format için [English README](README.md#setup--export-your-session-once)'e bak):

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

5. Şifrele ve içe aktar:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

`session.enc` + `aterkeep.key` oluşur — **key dosyasını kaybetme**, session'ı çözmenin tek yolu o.

## Çalıştırma

```powershell
.\target\release\aterkeep.exe
```

Tarayıcıda **http://127.0.0.1:4041** aç.

## Panel sekmeleri

| Sekme | Ne yapar |
|---|---|
| **Durum** | durum rozeti, kontroller, oto-başlat anahtarı, canlı log, request inspector |
| **Konsol** | sunucu log akışı (10 sn tazeleme) |
| **Ayarlar** | `server.properties` düzenle ve kaydet |
| **Oyuncular** | çevrimiçi oyuncular |

**Oto-başlat anahtarı önemli:** kapalıyken daemon sunucuyu asla yeniden başlatmaz. **Durdur** butonu onu otomatik kapatır.

## Session ömrü

Aternos session çerezleri **~30 gün** yaşar. Panel `OTURUM BİTTİ` gösterince export adımlarını tekrarla ve import et. Ayda 1, 2 dakika.

## Güvenlik

- Session şifreli durur (`session.enc`, AES-256-GCM)
- `aterkeep.key` asla commit edilmez, makineden çıkmaz
- API string'leri binary'de şifreli, runtime'da senin anahtarınla çözülür
- Panel sadece `127.0.0.1`'e bağlanır

## Lisans

**aterkeep ticari bir yazılımdır — açık kaynak değildir.**

Kaynak kodu yalnızca şeffaflık ve inceleme amacıyla yayımlanmıştır. Kişisel ve
ticari olmayan kullanıma izin verilir. Yeniden dağıtım, satış, türev çalışma ve
ticari kullanım **yasaktır**. Tüm koşullar için [LICENSE](LICENSE) dosyasına bakın.

## Lisans satın alma

Ticari kullanım, yeniden dağıtım, markalama ve keep-alive motoruna
(`aterkeep-core`) kaynak erişimi ücretli ticari lisans kapsamındadır.

**İletişim:** berlaylc2138@gmail.com

## Sorumluluk reddi

Bağımsız bir projedir — Aternos GmbH veya Mojang Studios ile bağlantılı değildir.
