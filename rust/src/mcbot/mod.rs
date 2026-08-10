//! Yerel (Rust) anti-idle Minecraft botu.
//!
//! Node.js + mineflayer botunun yerini alir. NEDEN: mineflayer'i calistirmak
//! Node kurulumu ve genis bir npm agaci gerektiriyordu; Node bir APK'nin icine
//! sigmadigi icin bot telefonda HIC calismiyordu. Bu surum daemon binary'sinin
//! icinde yasiyor, dolayisiyla her platformda — Android dahil — calisiyor ve
//! "tek binary, kurulacak runtime yok" iddiasi gercekten dogru oluyor.
//!
//! DAVRANIS, Node surumuyle ayni tutuldu (zamanlamalar dahil), cunku panel
//! `config/bot-status.json`'u okuyor ve o sozlesme degismedi.
//!
//! KAPSAM SINIRI, acikca: bu bot yalnizca **offline (cracked)** sunucuya ve
//! yalnizca protokol 774 (1.21.11) ile uyumlu sunuculara baglanir. mineflayer
//! `minecraft-data` sayesinde bir suru surumu bedavaya destekliyordu; burada
//! paket ID'leri elle yazildigi icin o kapsam yok. Sunucu daha yeni ya da
//! uyumsuzsa panel `unsupported_version` gosterir ve bot baglanmayi denemez.

mod proto;

use proto::{
    offline_uuid, put_f32, put_i64, put_str, put_u16, put_uuid, put_varint, read_f64, read_i64,
    read_str, read_varint, Conn,
};
use serde_json::{json, Value};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub use aterkeep_core::BotConfig;

/// Desteklenen en yuksek protokol / surum. Panel bunu gosteriyor.
pub const MAX_SUPPORTED_PROTOCOL: i32 = 774;
pub const MAX_SUPPORTED_VERSION: &str = "1.21.11";

// ─── Paket ID'leri ───
//
// Kaynak: PrismarineJS/minecraft-data, pc/1.21.11/protocol.json (proto 774).
// Elle yazilmadi, o dosyadan cikarildi — tahmin edilen bir ID, tesadufen baska
// bir paketi tetikleyip teshisi imkansiz hatalar uretir.

const HS_SET_PROTOCOL: i32 = 0x00;

const ST_REQUEST: i32 = 0x00;
const ST_RESPONSE: i32 = 0x00;

const LG_LOGIN_START: i32 = 0x00;
const LG_ACKNOWLEDGED: i32 = 0x03;
const LG_S_DISCONNECT: i32 = 0x00;
const LG_S_ENCRYPTION: i32 = 0x01;
const LG_S_SUCCESS: i32 = 0x02;
const LG_S_COMPRESS: i32 = 0x03;

const CF_C_DISCONNECT: i32 = 0x02;
const CF_C_FINISH: i32 = 0x03;
const CF_C_KEEP_ALIVE: i32 = 0x04;
const CF_C_PING: i32 = 0x05;
const CF_C_KNOWN_PACKS: i32 = 0x0e;
const CF_S_FINISH: i32 = 0x03;
const CF_S_KEEP_ALIVE: i32 = 0x04;
const CF_S_PONG: i32 = 0x05;
const CF_S_KNOWN_PACKS: i32 = 0x07;

const PL_C_LOGIN: i32 = 0x30;
const PL_C_KICK: i32 = 0x20;
const PL_C_KEEP_ALIVE: i32 = 0x2b;
const PL_C_POSITION: i32 = 0x46;
const PL_S_TELEPORT_CONFIRM: i32 = 0x00;
const PL_S_CHAT_COMMAND: i32 = 0x06;
const PL_S_KEEP_ALIVE: i32 = 0x1b;
const PL_S_LOOK: i32 = 0x1f;

// ─── Zamanlamalar (Node surumuyle ayni) ───

const RETRY: Duration = Duration::from_millis(8_000);
/// Aternos'un ucretsiz proxy'si ILK ping'e gec cevap veriyor: olculen sure
/// ~12.7 saniye. Bu yuzden sinir olculen surenin iki katinda.
const PING_TIMEOUT: Duration = Duration::from_millis(25_000);
const DEAD_RETRY: Duration = Duration::from_millis(20_000);
/// Bagladi ama oyuna hic girmedi: bu kadar sonra bastan dene.
const JOIN_TIMEOUT: Duration = Duration::from_millis(45_000);
/// Bu sure boyunca sunucudan tek paket gelmezse baglanti olmus sayilir.
/// Vanilla 15 saniyede bir keepalive gonderiyor; 60 saniye rahat bir pay.
const SILENCE_TIMEOUT: Duration = Duration::from_secs(60);

const NAMES: &[&str] = &[
    "Alex", "Steve", "Efe", "Mert", "Deniz", "Kaan", "Arda", "Emir", "Zeynep", "Elif", "Defne",
    "Ada", "Yusuf", "Kerem", "Baran", "Doruk", "Alp", "Can", "Ege", "Selim", "Tuna", "Umut", "Mira",
];

fn random_name() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let base = NAMES[rng.gen_range(0..NAMES.len())];
    if rng.gen_bool(0.35) {
        format!("{base}{}", rng.gen_range(10..100))
    } else {
        base.to_string()
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ─── Durum dosyasi ───

/// `config/bot-status.json`'a KISMI guncelleme yazar.
///
/// Mevcut dosyanin uzerine yalnizca verilen alanlar bindiriliyor (Node surumu de
/// boyle yapiyordu): panel her alani her yazimda gormek istiyor ama her cagri
/// noktasinin tum durumu bilmesi gerekmiyor.
fn write_status(path: &Path, patch: Value) {
    let mut base = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    if let Some(obj) = patch.as_object() {
        for (k, v) in obj {
            base.insert(k.clone(), v.clone());
        }
    }
    base.insert("ts".into(), json!(now_ms()));
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(s) = serde_json::to_string_pretty(&Value::Object(base)) {
        let _ = std::fs::write(path, s);
    }
}

fn log(line: &str) {
    println!("[bot] {line}");
}

// ─── Sunucu ping'i ───

pub struct ServerInfo {
    pub version_name: String,
    pub protocol: i32,
}

/// Server List Ping: sunucu ayakta mi, hangi surum?
///
/// Once ping atmak SART: Aternos sunucusu kapali ya da sirada olabiliyor ve
/// dogrudan login denemek anlamsiz bir hata uretir. Ayrica surum kapisini
/// (protokol 774) burada geciriyoruz.
pub async fn ping(host: &str, port: u16) -> Result<ServerInfo, String> {
    let mut conn = Conn::connect(host, port, PING_TIMEOUT).await?;

    let mut hs = Vec::new();
    put_varint(&mut hs, MAX_SUPPORTED_PROTOCOL);
    put_str(&mut hs, host);
    put_u16(&mut hs, port);
    put_varint(&mut hs, 1); // next state: status
    conn.send(HS_SET_PROTOCOL, &hs).await?;
    conn.send(ST_REQUEST, &[]).await?;

    let (id, body) = tokio::time::timeout(PING_TIMEOUT, conn.recv())
        .await
        .map_err(|_| "ping zaman asimi".to_string())??;
    if id != ST_RESPONSE {
        return Err(format!("beklenmeyen ping yaniti: 0x{id:02x}"));
    }
    let mut cur = Cursor::new(body);
    let json_str = read_str(&mut cur)?;
    let v: Value = serde_json::from_str(&json_str).map_err(|e| format!("ping json: {e}"))?;
    let version_name = v["version"]["name"].as_str().unwrap_or("?").to_string();
    let protocol = v["version"]["protocol"].as_i64().unwrap_or(-1) as i32;
    Ok(ServerInfo {
        version_name,
        protocol,
    })
}

// ─── Oturum ───

enum SessionEnd {
    /// Durdurma istendi.
    Stopped,
    /// Sunucu atti; sebep kodu (varsa) ve ham metin.
    Kicked(Option<&'static str>, String),
    /// Baglanti koptu / hata.
    Lost(String),
}

/// NBT govdesinde ASCII alt dizi arar.
///
/// NEDEN AYRISTIRMIYORUZ: 1.21'de play/configuration disconnect sebebi NBT.
/// Tam bir NBT ayristiricisi yazmanin tek kazanci daha guzel bir log satiri
/// olurdu; bizim ihtiyacimiz ise tek bir karar: bu kick ayar degismeden ASLA
/// duzelmeyecek bir kick mi? NBT string'leri uzunluk onekli UTF-8 oldugu icin
/// ham bayt aramasi bu karar icin yeterli ve yanlis pozitif riski yok denecek
/// kadar az. (Login asamasindaki disconnect zaten duz JSON string geliyor —
/// online-mode reddi orada yakalaniyor.)
fn contains_ascii(hay: &[u8], needle: &str) -> bool {
    let n = needle.as_bytes();
    hay.windows(n.len()).any(|w| w == n)
}

/// Kick sebebinden kalici hata kodu cikarir.
///
/// `online_mode` ve `whitelist`, sunucu ayari degismeden ASLA duzelmeyecek
/// durumlar. Bunlari ayirmak onemli: 8 saniyede bir yeniden denemek sunucuyu
/// bosuna mesgul eder ve panelde sebebi gorunmez.
fn kick_code(raw: &str) -> Option<&'static str> {
    if raw.contains("unverified_username") {
        Some("online_mode")
    } else if raw.contains("not_whitelisted") {
        Some("whitelist")
    } else {
        None
    }
}

struct Session {
    name: String,
    vanish: (f64, f64, f64),
    status_path: PathBuf,
}

/// Oyun icinde yapilacak zamanli isler.
#[derive(Clone, Copy, PartialEq)]
enum Task {
    Spectator,
    Teleport,
    MarkVanished,
    Look,
}

impl Session {
    async fn run(&self, host: &str, port: u16, stop: &Arc<AtomicBool>) -> SessionEnd {
        match self.play(host, port, stop).await {
            Ok(end) => end,
            Err(e) => SessionEnd::Lost(e),
        }
    }

    async fn play(
        &self,
        host: &str,
        port: u16,
        stop: &Arc<AtomicBool>,
    ) -> Result<SessionEnd, String> {
        let mut conn = Conn::connect(host, port, PING_TIMEOUT).await?;

        // ── Handshake + Login ──
        let mut hs = Vec::new();
        put_varint(&mut hs, MAX_SUPPORTED_PROTOCOL);
        put_str(&mut hs, host);
        put_u16(&mut hs, port);
        put_varint(&mut hs, 2); // next state: login
        conn.send(HS_SET_PROTOCOL, &hs).await?;

        let mut ls = Vec::new();
        put_str(&mut ls, &self.name);
        put_uuid(&mut ls, offline_uuid(&self.name));
        conn.send(LG_LOGIN_START, &ls).await?;

        // ── Login asamasi ──
        loop {
            if stop.load(Ordering::Relaxed) {
                return Ok(SessionEnd::Stopped);
            }
            let (id, body) = tokio::time::timeout(JOIN_TIMEOUT, conn.recv())
                .await
                .map_err(|_| "login zaman asimi".to_string())??;
            match id {
                LG_S_COMPRESS => {
                    let mut cur = Cursor::new(body);
                    let t = read_varint(&mut cur)?;
                    conn.set_threshold(t);
                    log(&format!("sikistirma acildi (esik {t})"));
                }
                LG_S_ENCRYPTION => {
                    // Sunucu sifreleme istiyor => online-mode acik. Offline bir
                    // hesapla bunu gecmek MUMKUN DEGIL (Mojang oturumu gerekir),
                    // bu yuzden denemek yerine net sebebi bildiriyoruz.
                    return Ok(SessionEnd::Kicked(
                        Some("online_mode"),
                        "sunucu sifreleme istedi (online-mode=true)".into(),
                    ));
                }
                LG_S_SUCCESS => {
                    conn.send(LG_ACKNOWLEDGED, &[]).await?;
                    log("login tamam — configuration asamasi");
                    break;
                }
                LG_S_DISCONNECT => {
                    let mut cur = Cursor::new(body);
                    let reason = read_str(&mut cur).unwrap_or_else(|_| "?".into());
                    return Ok(SessionEnd::Kicked(kick_code(&reason), reason));
                }
                _ => { /* login asamasinda baska paket ilgilendirmiyor */ }
            }
        }

        // ── Configuration asamasi ──
        loop {
            if stop.load(Ordering::Relaxed) {
                return Ok(SessionEnd::Stopped);
            }
            let (id, body) = tokio::time::timeout(JOIN_TIMEOUT, conn.recv())
                .await
                .map_err(|_| "configuration zaman asimi".to_string())??;
            match id {
                CF_C_KNOWN_PACKS => {
                    // BOS dizi ile cevapliyoruz: "hicbir veri paketini bilmiyorum".
                    // Cevapsiz birakmak olmuyor — sunucu registry verisini
                    // gondermek icin bu yaniti bekliyor ve el sikismasi orada
                    // kilitleniyor. Bos yanit, sunucunun her seyi acikca
                    // gondermesini sagliyor.
                    let mut b = Vec::new();
                    put_varint(&mut b, 0);
                    conn.send(CF_S_KNOWN_PACKS, &b).await?;
                }
                CF_C_KEEP_ALIVE => {
                    let mut cur = Cursor::new(body);
                    let k = read_i64(&mut cur)?;
                    let mut b = Vec::new();
                    put_i64(&mut b, k);
                    conn.send(CF_S_KEEP_ALIVE, &b).await?;
                }
                CF_C_PING => {
                    // Ping'in govdesi i32; pong ayni degeri geri istiyor.
                    conn.send(CF_S_PONG, &body).await?;
                }
                CF_C_FINISH => {
                    conn.send(CF_S_FINISH, &[]).await?;
                    log("configuration tamam — oyuna giriliyor");
                    break;
                }
                CF_C_DISCONNECT => {
                    let raw = String::from_utf8_lossy(&body).to_string();
                    let code = if contains_ascii(&body, "unverified_username") {
                        Some("online_mode")
                    } else if contains_ascii(&body, "not_whitelisted") {
                        Some("whitelist")
                    } else {
                        None
                    };
                    return Ok(SessionEnd::Kicked(code, raw));
                }
                _ => { /* registry_data, tags, feature_flags... hepsi yok sayilir */ }
            }
        }

        // ── Play asamasi ──
        write_status(
            &self.status_path,
            json!({
                "connected": true, "state": "online", "name": self.name,
                "vanished": false, "error": null, "error_code": null
            }),
        );
        log(&format!("GIRDI: {} — vanish bekleniyor", self.name));

        let mut schedule: Vec<(Instant, Task)> = Vec::new();
        let start = Instant::now();
        // Node surumuyle ayni gecikmeler: sunucu oyuncuyu tam olarak yerlestirene
        // kadar komut gondermek ise yaramiyor.
        schedule.push((start + Duration::from_millis(600), Task::Spectator));
        schedule.push((start + Duration::from_millis(1400), Task::Teleport));
        schedule.push((start + Duration::from_millis(3000), Task::MarkVanished));
        schedule.push((start + Duration::from_millis(3000), Task::Look));

        let mut yaw: f32 = 0.0;
        let mut last_recv = Instant::now();
        // Isinlanma GERCEKTEN oldu mu?
        //
        // `/gamemode` ve `/tp` operator yetkisi ister. Yetki yoksa sunucu
        // komutlari sessizce reddeder (hata oyuncuya sohbet mesaji olarak gider,
        // ayri bir pakete donmez) ve bot spawn'da GORUNUR bir oyuncu olarak
        // durur. Eski Node surumu bu durumda da "vanish aktif" diyordu — panelde
        // yanlis bilgi.
        //
        // Dogrulama icin sunucunun geri gonderdigi position paketine bakiyoruz:
        // isinlanma olduysa hedef koordinat bize bildirilir. Bu sinyal dilden ve
        // sunucu yazilimindan bagimsiz.
        let mut vanish_confirmed = false;

        loop {
            if stop.load(Ordering::Relaxed) {
                return Ok(SessionEnd::Stopped);
            }
            if last_recv.elapsed() > SILENCE_TIMEOUT {
                return Err(format!(
                    "sunucudan {} saniye ses gelmedi",
                    SILENCE_TIMEOUT.as_secs()
                ));
            }

            // Sonraki isin zamanina kadar paket bekle. recv'i select! icinde
            // tutmak conn'u odunc alma sorunu cikardigi icin timeout kullaniliyor;
            // zaman asimi bir hata degil, "simdi is yapma sirasi" demek.
            let now = Instant::now();
            let wait = schedule
                .iter()
                .map(|(t, _)| *t)
                .min()
                .map(|t| t.saturating_duration_since(now))
                .unwrap_or(Duration::from_secs(1));

            match tokio::time::timeout(wait, conn.recv()).await {
                Ok(Ok((id, body))) => {
                    last_recv = Instant::now();
                    match id {
                        PL_C_KEEP_ALIVE => {
                            let mut cur = Cursor::new(body);
                            let k = read_i64(&mut cur)?;
                            let mut b = Vec::new();
                            put_i64(&mut b, k);
                            conn.send(PL_S_KEEP_ALIVE, &b).await?;
                        }
                        PL_C_POSITION => {
                            // Isinlanmayi ONAYLAMAK zorunlu: onaylanmayan bir
                            // teleport'ta sunucu oyuncuyu geri cekiyor ve
                            // "moved wrongly" ile atabiliyor.
                            let mut cur = Cursor::new(body);
                            let tp_id = read_varint(&mut cur)?;
                            let mut b = Vec::new();
                            put_varint(&mut b, tp_id);
                            conn.send(PL_S_TELEPORT_CONFIRM, &b).await?;

                            // Hedefe vardik mi? Sunucu bizi vanish noktasina
                            // koyduysa /tp gecmis demektir.
                            if let (Ok(x), Ok(y), Ok(z)) = (
                                read_f64(&mut cur),
                                read_f64(&mut cur),
                                read_f64(&mut cur),
                            ) {
                                let (vx, vy, vz) = self.vanish;
                                // 2 blok tolerans: sunucu koordinati tam olarak
                                // istedigimiz gibi vermeyebiliyor (blok merkezi).
                                if (x - vx).abs() < 2.0 && (y - vy).abs() < 2.0 && (z - vz).abs() < 2.0
                                {
                                    vanish_confirmed = true;
                                }
                            }
                        }
                        PL_C_KICK => {
                            let raw = String::from_utf8_lossy(&body).to_string();
                            let code = if contains_ascii(&body, "unverified_username") {
                                Some("online_mode")
                            } else if contains_ascii(&body, "not_whitelisted") {
                                Some("whitelist")
                            } else {
                                None
                            };
                            return Ok(SessionEnd::Kicked(code, raw));
                        }
                        PL_C_LOGIN => { /* join game — zaten play'deyiz */ }
                        _ => {}
                    }
                }
                Ok(Err(e)) => return Ok(SessionEnd::Lost(e)),
                Err(_) => {
                    // Sirasi gelen isleri yap.
                    let now = Instant::now();
                    let due: Vec<Task> = schedule
                        .iter()
                        .filter(|(t, _)| *t <= now)
                        .map(|(_, task)| *task)
                        .collect();
                    schedule.retain(|(t, _)| *t > now);

                    for task in due {
                        match task {
                            Task::Spectator => {
                                self.command(&mut conn, "gamemode spectator").await?;
                            }
                            Task::Teleport => {
                                let (x, y, z) = self.vanish;
                                self.command(&mut conn, &format!("tp {x} {y} {z}")).await?;
                            }
                            Task::MarkVanished => {
                                if vanish_confirmed {
                                    log("vanish aktif — gorunmez modda AFK");
                                    write_status(
                                        &self.status_path,
                                        json!({ "vanished": true, "error_code": null, "error": null }),
                                    );
                                } else {
                                    // Bot yine de bagli ve slotu tutuyor, yani
                                    // anti-idle isini YAPIYOR. Sadece gorunmez
                                    // degil. Bunu hata gibi degil, uyari gibi
                                    // bildiriyoruz.
                                    log("vanish DOGRULANMADI — bot gorunur duruyor (/gamemode ve /tp operator yetkisi ister)");
                                    // `error` teknik ayrinti; kullaniciya
                                    // gosterilen metin `error_code`dan
                                    // uretiliyor ve panelde kullanicinin
                                    // dilinde cikiyor (bkz. translations:
                                    // bot_err_vanish_no_permission).
                                    write_status(
                                        &self.status_path,
                                        json!({
                                            "vanished": false,
                                            "error_code": "vanish_no_permission",
                                            "error": "vanish not confirmed: /gamemode and /tp require operator"
                                        }),
                                    );
                                }
                            }
                            Task::Look => {
                                // Yalnizca BAKIS degistiriliyor, pozisyon degil.
                                // Vanish'te sunucu bizi bir yere koydu; kendi
                                // pozisyonumuzu bildirmek onunla cakisir.
                                yaw = (yaw + 37.0) % 360.0;
                                let mut b = Vec::new();
                                put_f32(&mut b, yaw);
                                put_f32(&mut b, 0.0);
                                b.push(0x01); // MovementFlags: onGround
                                conn.send(PL_S_LOOK, &b).await?;

                                use rand::Rng;
                                let jitter = rand::thread_rng().gen_range(2500..4500);
                                schedule.push((
                                    Instant::now() + Duration::from_millis(jitter),
                                    Task::Look,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Sohbet komutu gonderir. Onde `/` OLMAZ: chat_command paketi komutu
    /// egik cizgisiz bekliyor.
    async fn command(&self, conn: &mut Conn, cmd: &str) -> Result<(), String> {
        let mut b = Vec::new();
        put_str(&mut b, cmd);
        conn.send(PL_S_CHAT_COMMAND, &b).await
    }
}

/// Bot dongusu: ping → surum kapisi → oturum → yeniden baglan.
///
/// `stop` set edildiginde temiz cikar ve durumu `stopped` yazar.
pub async fn run(cfg: BotConfig, root: PathBuf, stop: Arc<AtomicBool>) {
    let status_path = root.join("config").join("bot-status.json");
    let host = cfg.host.clone();
    let port = cfg.port;

    write_status(
        &status_path,
        json!({
            "connected": false, "state": "starting", "host": host, "port": port,
            "name": cfg.name, "vanished": false,
            "max_supported_protocol": MAX_SUPPORTED_PROTOCOL,
            "max_supported_version": MAX_SUPPORTED_VERSION,
            "error": null, "error_code": null
        }),
    );
    log(&format!(
        "basladi — hedef {host}:{port}, max v{MAX_SUPPORTED_VERSION}"
    ));

    while !stop.load(Ordering::Relaxed) {
        // ── Ping ──
        let info = match ping(&host, port).await {
            Ok(i) if i.protocol > 0 => i,
            _ => {
                log(&format!(
                    "sunucu cevap vermiyor (kapali/sirada) — {}sn sonra tekrar",
                    DEAD_RETRY.as_secs()
                ));
                write_status(
                    &status_path,
                    json!({ "connected": false, "state": "waiting", "server_protocol": null,
                            "error": null, "error_code": null }),
                );
                sleep_or_stop(DEAD_RETRY, &stop).await;
                continue;
            }
        };

        // ── Surum kapisi ──
        if info.protocol > MAX_SUPPORTED_PROTOCOL {
            log(&format!(
                "DESTEKLENMEYEN SURUM: server={} (proto {}) > max {MAX_SUPPORTED_VERSION}",
                info.version_name, info.protocol
            ));
            write_status(
                &status_path,
                json!({
                    "connected": false, "state": "unsupported_version",
                    "server_protocol": info.protocol, "server_version": info.version_name,
                    "error": format!("Server {} desteklenmiyor. Bot en fazla {} surumunu destekler. Aternos panelinde Yazilim > Vanilla {} secin.",
                                     info.version_name, MAX_SUPPORTED_VERSION, MAX_SUPPORTED_VERSION),
                    "error_code": "unsupported_version"
                }),
            );
            // Sunucu surumu degismeden duzelmez; sik denemenin anlami yok.
            sleep_or_stop(Duration::from_secs(60), &stop).await;
            continue;
        }

        // ── Oturum ──
        let name = if cfg.name.trim().is_empty() {
            random_name()
        } else {
            cfg.name.clone()
        };
        log(&format!(
            "baglaniliyor... ({name} @ {host}:{port}) [{}]",
            info.version_name
        ));
        write_status(
            &status_path,
            json!({
                "connected": false, "state": "connecting", "name": name,
                "server_protocol": info.protocol, "server_version": info.version_name,
                "error": null, "error_code": null
            }),
        );

        let session = Session {
            name,
            vanish: (cfg.vanish_x, cfg.vanish_y, cfg.vanish_z),
            status_path: status_path.clone(),
        };

        match session.run(&host, port, &stop).await {
            SessionEnd::Stopped => break,
            SessionEnd::Kicked(code, raw) => {
                log(&format!("KICK: {raw}"));
                write_status(
                    &status_path,
                    json!({ "connected": false, "state": "kicked",
                            "error_code": code, "error": raw }),
                );
                if code.is_some() {
                    // Ayar degismeden duzelmeyecek bir sebep: bir dakika bekle.
                    // 8 saniyede bir denemek sunucuyu bosuna mesgul ediyordu.
                    sleep_or_stop(Duration::from_secs(60), &stop).await;
                } else {
                    sleep_or_stop(RETRY, &stop).await;
                }
            }
            SessionEnd::Lost(e) => {
                log(&format!("baglanti koptu: {e}"));
                write_status(
                    &status_path,
                    json!({ "connected": false, "state": "disconnected", "error": e }),
                );
                sleep_or_stop(RETRY, &stop).await;
            }
        }
    }

    log("durduruldu");
    write_status(
        &status_path,
        json!({ "connected": false, "state": "stopped", "vanished": false }),
    );
}

/// Beklerken durdurma istegine duyarli kalir.
///
/// Duz `sleep` kullanmak, "Durdur"a basildiktan sonra bir dakikaya kadar
/// yasamaya devam eden bir bot demekti.
async fn sleep_or_stop(total: Duration, stop: &Arc<AtomicBool>) {
    let step = Duration::from_millis(250);
    let mut left = total;
    while left > Duration::ZERO {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        let d = step.min(left);
        tokio::time::sleep(d).await;
        left -= d;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kick_code_detects_permanent_failures() {
        assert_eq!(
            kick_code("{\"translate\":\"multiplayer.disconnect.unverified_username\"}"),
            Some("online_mode")
        );
        assert_eq!(
            kick_code("{\"translate\":\"multiplayer.disconnect.not_whitelisted\"}"),
            Some("whitelist")
        );
        // Gecici bir sebep kalici olarak isaretlenmemeli; yoksa bot bir dakika
        // bosuna bekler.
        assert_eq!(kick_code("Server closed"), None);
    }

    #[test]
    fn contains_ascii_finds_needle_in_nbt_like_bytes() {
        let mut nbt = vec![0x08, 0x00, 0x06];
        nbt.extend_from_slice(b"\x00\x13multiplayer.disconnect.unverified_username");
        assert!(contains_ascii(&nbt, "unverified_username"));
        assert!(!contains_ascii(&nbt, "not_whitelisted"));
    }

    #[test]
    fn random_name_is_non_empty_and_reasonable() {
        for _ in 0..50 {
            let n = random_name();
            assert!(!n.is_empty());
            // Minecraft kullanici adi siniri 16 karakter.
            assert!(n.len() <= 16, "isim fazla uzun: {n}");
        }
    }
}
