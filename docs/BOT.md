# Anti-idle bot

> **Note:** the detail below is in Turkish. The short version in English:
> the daemon spawns `node bot/index.js` when the server comes online and stops
> it when the server goes down. Settings live in `config/bot.json`, live state
> in `config/bot-status.json`, and the bot needs Node.js plus `npm install`
> inside `bot/`. The `bot/` folder ships in `aterkeep-extras.zip`, not with the
> bare binary.

# aterkeep-bot — Anti-Idle Bot

aterkeep daemon'ı, sunucuyu kalıcı olarak ayakta tutmak için opsiyonel bir **Node.js anti-idle botu** barındırır. Bot, bir gerçek oyuncu gibi bağlanır ama görünmez (spectator + vanish) modda uzakta AFK kalır; koparsa otomatik yeniden bağlanır. Böylece Aternos'un "boş sunucu → durdurma" davranışı tetiklenmez.

## Nasıl çalışır

1. Daemon, `config/bot.json`'da `enabled: true` olduğunda `node bot/index.js` sürecini başlatır ve `ATERKEEP_BOT_DIR=<repo kökü>` env değişkenini geçirir.
2. Bot önce sunucuyu **pingler** (`minecraft-protocol/src/ping`), sürümü tespit eder. Sürüm desteklenen aralıktaysa (`1.21.x` ve altı) bağlanır.
3. Bağlanınca `/gamemode spectator` + `/tp <vanish_x> <vanish_y> <vanish_z>` komutlarıyla görünmez, uzak noktaya ışınlanır.
4. Her ~3 sn **insan-benzeri mikro-etkinlik** yapar (bakış değiştirme, hafif hareket) — vanish'teyken sadece bakınır, tespit riskini düşürür.
5. Kick/kopma/error durumlarında otomatik yeniden bağlanır; sunucu kapalıysa pingleyip açılmasını bekler.
6. Canlı durumunu `config/bot-status.json`'a yazar — daemon paneli buradan okur.

## Gereksinimler

- **Node.js 18+** — bot'un spawn edildiği makinede kurulu olmalı.
- **Cracked (offline-mode) sunucu** — bot `auth: 'offline'` ile bağlanır. Premium (`online-mode=true`) sunucular offline hesabı reddeder. Aternos ücretsiz sunucuları varsayılan olarak cracked'dir.
- **Vanilla yazılımı** — Aternos panelinde *Yazılım > Vanilla* seçili olmalı (Paper/Spigot da çalışabilir ama test edilmemiştir).
- **Sürüm ≤ 1.21.x** — daha yüksek sürümlerde `minecraft-data` henüz protokol verisine sahip olmadığı için bot bağlanmaz (panel `unsupported_version` gösterir).

## Kurulum

```bash
cd bot
npm install          # mineflayer + geçişli bağımlılıklar
```

Bu **tek seferlik**. Kurulumdan sonra bot'ı panelden yönetirsiniz.

## Ayarlar (`config/bot.json`)

Daemon çalışma zamanında yazar; git'e eklenmez. Şablon: `config/bot.example.json`.

| Alan | Tür | Açıklama |
|---|---|---|
| `enabled` | bool | Bot aktif mi (panelden de değişir) |
| `name` | string | Oyuncu adı |
| `host` | string | Sunucu adresi; boşsa daemon server id'den doldurur |
| `port` | number | Sunucu portu (genelde `25565`) |
| `vanish_x` | number | AFK ışınlanma X |
| `vanish_y` | number | AFK ışınlanma Y (yüksek, yükten düşmez) |
| `vanish_z` | number | AFK ışınlanma Z |

### Env değişkenleri (daemon yerine manuel çalıştırma için)

| Değişken | Varsayılan | Açıklama |
|---|---|---|
| `ATERKEEP_BOT_DIR` | `..` (bot dizininin üstü) | config/status için kök |
| `ATERKEEP_BOT_HOST` | config | host override |
| `ATERKEEP_BOT_PORT` | config | port override |
| `ATERKEEP_BOT_NAME` | config | isim override |
| `ATERKEEP_BOT_VX/VY/VZ` | config | vanish koordinatları override |
| `ATERKEEP_BOT_RETRY` | `25000` | normal yeniden bağlanma ms |
| `ATERKEEP_BOT_DEAD_RETRY` | `20000` | sunucu kapalıyken bekleme ms |
| `ATERKEEP_BOT_PINGTO` | `25000` | ping timeout ms |

## Durum alanları (`config/bot-status.json`)

Panelin okuduğu alanlar:

| Alan | Açıklama |
|---|---|
| `connected` | Bot sunucuda mı |
| `state` | `starting` / `connecting` / `online` / `waiting` / `kicked` / `disconnected` / `error` / `unsupported_version` / `stopped` |
| `name` | Aktif bot ismi |
| `vanished` | Vanish modunda mı |
| `server_version` | Ping'den gelen sürüm adı |
| `server_protocol` | Ping'den gelen protokol numarası |
| `max_supported_version` | Botun desteklediği en yüksek sürüm |
| `max_supported_protocol` | Aynı, protokol numarası |
| `error` | Hata mesajı (varsa) |
| `host` / `port` | Hedef sunucu |

## Sorun giderme

| Belirti | Sebep / Çözüm |
|---|---|
| Panel `mineflayer kurulu değil` | `cd bot && npm install` çalıştırılmamş. |
| Bot `waiting` durumunda takılı | Sunucu kapalı/sırada. Daemon açana kadar bekler, normal. |
| `unsupported_version` | Sunucu sürümü çok yeni. Aternos > Yazılım > Vanilla `1.21.x` seç. |
| Sürekli `kicked` | Sunucu online-mode (premium) olabilir → cracked yap, ya da bot yasaklı. |
| Spawn timeout (45 sn) | Bağlandı ama spawn olmuyor — genelde ağır sunucu. Otomatik yeniden dener. |
| `host`/`port` yanlış | `config/bot.json`'u kontrol et (daemon host'u otomatik doldurmalı). |

## Manuel çalıştırma (debug)

Normalde daemon spawn eder. Elle test için:

```bash
cd <repo-kökü>
ATERKEEP_BOT_DIR="$PWD" node bot/index.js
```

(Önce `config/bot.json`'un var olduğundan emin olun.)
