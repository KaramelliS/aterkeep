# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep logo" width="280"/>
</p>

**Aternos sunucu yöneticisi & 7/24 bekçi paneli.** Tek bir Rust binary'si (~1.7 MB) ile ücretsiz Aternos Minecraft sunucunu sürekli ayakta tutar ve modern bir web paneliyle yönetmeni sağlar — tarayıcı otomasyonu yok, saf HTTP.

<p align="center">
  <a href="README.md">English</a>
</p>

---

<p align="center">
  <video src="https://cdn.jsdelivr.net/gh/KaramelliS/aterkeep@main/promo/video/aterkeep-tr.mp4" controls width="100%">
    <a href="promo/video/aterkeep-tr.mp4">30 saniyelik tanitimi izle</a>
  </video>
</p>

<p align="center">
  <sub><b>30 saniyelik tanitimi izle.</b> Oynatilmiyor mu? <a href="promo/video/aterkeep-tr.mp4">Videoyu indir</a>.<br/>
  Diger diller: <a href="promo/video/aterkeep-en.mp4">English</a> · <a href="promo/video/aterkeep-de.mp4">Deutsch</a> · <a href="promo/video/aterkeep-fr.mp4">Français</a> · <a href="promo/video/aterkeep-es.mp4">Español</a> · <a href="promo/video/aterkeep-it.mp4">Italiano</a> · <a href="promo/video/aterkeep-pt.mp4">Português</a> · <a href="promo/video/aterkeep-ru.mp4">Русский</a> · <a href="promo/video/aterkeep-ar.mp4">العربية</a> · <a href="promo/video/aterkeep-zh.mp4">中文</a> · <a href="promo/video/aterkeep-ja.mp4">日本語</a> · <a href="promo/video/aterkeep-ko.mp4">한국어</a> · <a href="promo/video/aterkeep-nl.mp4">Nederlands</a> · <a href="promo/video/aterkeep-pl.mp4">Polski</a></sub>
</p>

---

## Özellikler

- **Otomatik sıra onayı** — sıra sana geldiğinde Aternos ~30 saniyelik bir onay penceresi açar; cevap gelmezse kuyruğun sonuna atılırsın. 7/24 çalışmanın asıl şartı bu adımdır ve keep-alive scriptlerinin sonsuza kadar kuyrukta dönmesinin sebebi de budur.
- **Senin yerine giriş yapar** — Aternos hesabınla kurulum yaparsan çerezi aterkeep kendisi alır, süresi dolunca da kendisi yeniler. DevTools yok, ayda bir kopyala-yapıştır yok.
- **Anti-idle bot** — sunucu açılınca giren, boş kaldığı için kapatılmasını engelleyen bir Minecraft istemcisi
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

## Kurulum (bir kez)

Binary'yi çalıştır ve **http://127.0.0.1:4041** adresini aç. Sihirbaz üç adım
sorar: panel dili, panel parolası, Aternos oturumu.

**Panel parolası** iki iş yapar: paneli korur *ve* oturumunu şifreler. Anahtar
diske **yazılmaz**, her açılışta bu paroladan türetilir (PBKDF2-HMAC-SHA256,
600 000 tur). **Unutursan kurtarma yoktur** — `config/` silinip yeniden kurulur.

**Aternos oturumu için iki yol var:**

**1. Aternos hesabı (varsayılan).** Kullanıcı adı ve parolanı yaz; aterkeep saf
HTTP ile giriş yapıp çerezi kendisi alır. Tarayıcı, DevTools, kopyalama yok.
Hesabında birden fazla sunucu varsa hangisini ayakta tutacağını sorar. Bilgiler
`config/session.enc` içinde, çerezlerle **aynı** AES-256-GCM şifresi altında
durur ve yalnızca `aternos.org`'a gönderilir. Tek amaçları: oturum düşünce
aterkeep yeniden giriş yapabilsin.

> İki durumda çalışmaz: hesapta **iki adımlı doğrulama** açıksa, ya da Aternos
> **captcha** isterse. İkisi de kendi mesajıyla bildirilir ve ikinci yola geçmen
> gerekir.

**2. Çerez yapıştırma (yedek).** `aternos.org`'da F12 → **Network** → F5 →
herhangi bir isteğin `cookie:` satırının tamamını kopyala; **Console**'da
`window.AJAX_TOKEN` çalıştırıp çıktısını al. İkisini sihirbaza yapıştır.
`ATERNOS_SESSION` **HttpOnly** olduğu için JavaScript onu okuyamaz — Network
sekmesi tek yol. Sunucu kimliği yapıştırdığın çerezlerden otomatik bulunur.
Bu yolla kurulan oturum **kendini yenileyemez**.

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

## Oturum ömrü

Aternos oturum çerezini `Max-Age=2592000` ile veriyor — **tam 30 gün**. Bu sayı
giriş cevabından ölçüldü, tahmin değil. Başka yerden çıkış yaparsan ya da
parolanı değiştirirsen daha erken de bitebilir.

**Hesapla kurduysan** yapacak bir şey yok: çerezler düşünce daemon yeniden giriş
yapar, taze oturumu yazar ve devam eder. Log'da tek satır görürsün.

**Çerez yapıştırdıysan** panel `SESSION` rozeti ve "oturum düştü" uyarısı
gösterir — eskiden bu "sunucun kapalı" gibi görünüyordu — ve seni tek tıkla
sihirbaza götürür. O noktada hesap yöntemine geçmek bu işi tamamen bitirir.

Panel ayrıca **oturum yaşını** gösterir; ilk düşüşten sonra bir öncekinin ne
kadar dayandığını da. Aternos bir sayı ilan etmiyor, aterkeep seninkini ölçüyor.

Makine yeniden başlayınca kendi kalkması için: **[docs/AUTOSTART.md](docs/AUTOSTART.md)** (Windows görevi — parola DPAPI ile korunur, systemd, Termux:Boot).

## Güvenlik

- Session şifreli durur (`config/session.enc`, AES-256-GCM)
- **Diskte anahtar dosyası yok** — anahtar paroladan türetilir (PBKDF2, 600.000 tur, kuruluma özel rastgele salt). `config/` klasörü çalınsa bile parola olmadan açılamaz
- **Panel parola ister** — tüm uçlar `HttpOnly` oturum çerezi arkasında
- API string'leri binary'de şifreli, runtime'da senin anahtarınla çözülür
- Panel sadece `127.0.0.1`'e bağlanır

## ⚠ Satin almadan once: Aternos'un kurallari

Aternos'un kendi destek dokumaninda soyle yaziyor:

> *"Sunucunu 7/24 acik tutmak icin bot, script veya baska hilelerle Aternos
> sistemini asmaya calismak kurallarimiza aykiridir… Sistem yapay aktiviteyi
> otomatik olarak denetler."*
> — [24/7 Hosting](https://support.aternos.org/hc/en-us/articles/31771896948253-24-7-Hosting)

Bu tarif dogrudan bu urunu anlatiyor. **aterkeep kullanmak sunucunun ya da
Aternos hesabinin askiya alinmasina veya silinmesine yol acabilir.** Bunu
engellemek elimizde degil; anti-idle bot da aktiviteyi gizlemez, aksine daha
gorunur kilar.

Urun oldugu gibi satiliyor: yalnizca senin kontrolundeki ve riski goze
aldigin hesaplarda kullan. Bu kabul edilebilir degilse satin alma — 7/24
sunucu icin desteklenen yol ucretli bir Minecraft barindirma hizmetidir.

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
