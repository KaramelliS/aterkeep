# aterkeep-bot

Node.js tabanlı, [mineflayer](https://github.com/PrismarineJS/mineflayer) kullanan anti-idle Minecraft botu. Aterkeep daemon'ı (`aterkeep` / `aterkeep.exe`) tarafından otomatik olarak `node bot/index.js` şeklinde spawn edilir — kendi başına çalıştırmak genellikle gerekmez.

## Ne yapar?

- Sunucuya **offline/cracked** hesap olarak bağlanır (rastgele isimle).
- Bağlanınca kendini **spectator** moduna alır ve `vanish_x/y/z` koordinatlarına ışınlar (uzakta, görünmez).
- Düzenli aralıklarla **insan-benzeri** hareketler yapar (bakış, zıplama, kıkırdama) — böylece sunucu AFK saymaz.
- Kopunca otomatik yeniden bağlanır; sunucu kapalıysa ping atıp açılmasını bekler.
- Canlı durumunu `config/bot-status.json` dosyasına yazar (daemon paneli buradan okur).

## Gereksinimler

- **Node.js** 18+ sunucunuzda kurulu olmalı (daemon bot'u spawn ettiğinde).
- **Offline/cracked mod** — sunucu `online-mode=false` olmalı (Aternos ücretsiz sunucuları varsayılan olarak böyledir). Premium (online-mode) sunucularda offline hesap bağlanamaz.
- **Vanilla yazılımı** ve **1.21.x veya daha düşük** sürüm. Daha yeni sürümler desteklenmez (panel uyarı verir, bot bağlanmaz).

## Kurulum

```bash
cd bot
npm install
```

Bu `mineflayer`'ı kurar. Sonra bot'ı panel üzerinden açıp kapatırsınız — daemon config'i yazar.

## Ayarlar

Gerçek `config/bot.json` daemon tarafından çalışma zamanında yazılır (git'e eklenmez). Şablon için `config/bot.example.json`:

| Alan | Açıklama |
|---|---|
| `enabled` | Bot aktif mi (daemon panelinden de değişir) |
| `name` | Bot oyuncu adı |
| `host` | Sunucu adresi (boşsa daemon doldurur) |
| `port` | Sunucu portu (genelde `25565`) |
| `vanish_x/y/z` | AFK/vanish ışınlanma koordinatı |

## Sorun giderme

| Sorun | Çözüm |
|---|---|
| Panel `mineflayer kurulu değil` hatası | `cd bot && npm install` çalıştırın. |
| Bot bağlanmıyor / `waiting` | Sunucu kapalı veya sırada. Daemon sunucuyu açar, bot ping atıp bekler. |
| `unsupported_version` | Sunucu sürümü çok yeni. Aternos panelinde Yazılım > Vanilla `1.21.x` seçin. |
| Bot kickleniyor | Sunucu `online-mode=true` olabilir (cracked olmalı) veya bot yasaklı. |
| Bot hiç girmiyor | `host`/`port` doğru mu kontrol edin (`config/bot.json`). |
