// aterkeep-bot: Minecraft bot (anti-idle + vanish).
// config/bot.json'dan ayarlari okur, status.json'a canli durum yazar.
// Rust (aterkeep.exe) tarafindan spawn edilir.
const fs = require('fs');
const path = require('path');

const BASE_DIR = process.env.ATERKEEP_BOT_DIR || path.resolve(__dirname, '..');
const CONFIG_PATH = path.join(BASE_DIR, 'config', 'bot.json');
const STATUS_PATH = path.join(BASE_DIR, 'config', 'bot-status.json');

// mineflayer bagimliligi kontrolu — kurulu degilse status yaz ve cik.
// Boylece eksik npm install sessizce crash etmez, panel hatayi gosterir.
let mineflayer;
let mcPing;
try {
  mineflayer = require('mineflayer');
  mcPing = require('minecraft-protocol/src/ping');
} catch (e) {
  const msg = "mineflayer kurulu degil — bot/ dizininde 'npm install' calistir";
  try {
    fs.mkdirSync(path.dirname(STATUS_PATH), { recursive: true });
    fs.writeFileSync(STATUS_PATH, JSON.stringify({
      ts: Date.now(),
      connected: false,
      state: 'error',
      error: msg,
    }, null, 2));
  } catch (_) {}
  console.error(`[aterkeep-bot] ${msg} (${e && e.message ? e.message : e})`);
  process.exit(1);
}

// minecraft-data'nin destekledigi EN YUKSEK protokol (panel uyarisi icin).
// mineflayer 4.x + minecraft-data 3.x -> 1.21.11 = proto 774
const MAX_SUPPORTED_PROTOCOL = 774;
const MAX_SUPPORTED_VERSION = '1.21.11';

function loadConfig() {
  // Her hatayi yutup {} donmek, botun sessizce localhost:25565'e baglanmaya
  // calismasi demekti — kullanici "bot girmiyor" goruyor, sebebi hicbir yerde
  // yazmiyordu. Node, utf8 okumada BOM'u KORUR ve JSON.parse BOM'da patlar;
  // bu bir kez gercekten basimiza geldi (PowerShell ile yazilmis bot.json).
  let raw;
  try {
    raw = fs.readFileSync(CONFIG_PATH, 'utf8').replace(/^﻿/, '');
  } catch (e) {
    fail(`bot.json okunamadi (${CONFIG_PATH}): ${e.message}`);
  }
  try {
    return JSON.parse(raw);
  } catch (e) {
    fail(`bot.json bozuk JSON (${CONFIG_PATH}): ${e.message}`);
  }
}

/// Durumu yaz, sebebi soyle ve CIK. Yanlis sunucuya baglanmaya calismaktansa
/// panelde gorunur bir hatayla durmak dogrudur.
function fail(msg) {
  try {
    fs.mkdirSync(path.dirname(STATUS_PATH), { recursive: true });
    fs.writeFileSync(STATUS_PATH, JSON.stringify({
      ts: Date.now(), connected: false, state: 'error', error: msg,
    }, null, 2));
  } catch (_) {}
  console.error(`[aterkeep-bot] ${msg}`);
  process.exit(1);
}

function writeStatus(patch) {
  try {
    let cur = {};
    try { cur = JSON.parse(fs.readFileSync(STATUS_PATH, 'utf8')); } catch (e) {}
    // ts EN SONA yazilir: onceden {ts} varsayilan olarak basta duruyordu ve
    // dosyadan okunan eski `cur.ts` onu her seferinde eziyordu — durum yillarca
    // "ilk yazim" zamanini gostermeye devam ederdi.
    const next = Object.assign({}, cur, patch, { ts: Date.now() });
    // ATOMIK: daemon bu dosyayi ayni anda okuyor; dogrudan uzerine yazmak
    // yarim JSON okutup "durum bilinmiyor"a dusuruyordu.
    const tmp = `${STATUS_PATH}.tmp`;
    fs.writeFileSync(tmp, JSON.stringify(next, null, 2));
    fs.renameSync(tmp, STATUS_PATH);
  } catch (e) {}
}

const cfg = loadConfig();
// Varsayilanlar notr birakilir: gercek host/port daemon tarafindan config/bot.json'a
// yazilir (Aternos adresi panelden tespit edilir). Buraya sabit bir sunucu adresi
// gomulmez.
const HOST = cfg.host || process.env.ATERKEEP_BOT_HOST || 'localhost';
const PORT = parseInt(String(cfg.port || process.env.ATERKEEP_BOT_PORT || '25565'), 10);
const BOT_NAME = cfg.name || process.env.ATERKEEP_BOT_NAME || 'AterkeepBot';
const VANISH_X = parseFloat(cfg.vanish_x != null ? cfg.vanish_x : (process.env.ATERKEEP_BOT_VX || '5000'));
const VANISH_Y = parseFloat(cfg.vanish_y != null ? cfg.vanish_y : (process.env.ATERKEEP_BOT_VY || '320'));
const VANISH_Z = parseFloat(cfg.vanish_z != null ? cfg.vanish_z : (process.env.ATERKEEP_BOT_VZ || '5000'));

const RETRY_MS = parseInt(process.env.ATERKEEP_BOT_RETRY || '8000', 10);
const DEAD_RETRY_MS = parseInt(process.env.ATERKEEP_BOT_DEAD_RETRY || '20000', 10);
// Aternos'un ucretsiz proxy'si ILK ping'e gec cevap veriyor: olculen sure
// ~12.7 saniye (sunucu tamamen saglikliyken, 1.21.11, 0/20 oyuncu). Eski 8
// saniyelik sinir bu yuzden HER ZAMAN dolup "sunucu cevap vermiyor" diye
// yorumlanmasina yol aciyordu: bot acik bir sunucuya sonsuza kadar hic
// baglanmayi denemeden bekliyordu. Sinir olculen surenin iki katina cikarildi.
const PING_TIMEOUT_MS = parseInt(process.env.ATERKEEP_BOT_PINGTO || '25000', 10);

const NAMES = [
  'Alex', 'Steve', 'Efe', 'Mert', 'Deniz', 'Kaan', 'Arda', 'Emir',
  'Zeynep', 'Elif', 'Defne', 'Ada', 'Yusuf', 'Kerem', 'Baran', 'Doruk',
  'Alp', 'Can', 'Ege', 'Rüzgar', 'Selim', 'Tuna', 'Umut', 'Mira',
];

let VANISHED = false;

function randomName() {
  const base = NAMES[Math.floor(Math.random() * NAMES.length)];
  if (Math.random() < 0.35) return base + Math.floor(Math.random() * 90 + 10);
  return base;
}

function log(...args) {
  console.log(`[${new Date().toISOString()}]`, ...args);
}

let bot = null;
let currentName = null;
let activityTimer = null;
let spawnTimer = null;
let reconnectPending = false;
let deadTimer = null;
let retryTimer = null;
let connecting = false;

/// TUM yeniden baglanma zamanlayicilari burada izlenir ve cleanup() hepsini
/// iptal eder. Izlenmeyen bir setTimeout, kapatilmis bir botu geri getirip
/// AYNI ANDA IKI BOT calistiriyordu; ikisi ayni kullanici adiyla girdigi icin
/// birbirini atiyor ve sonsuz bir gir-cik dongusu olusuyordu.
function scheduleReconnect() {
  if (reconnectPending) return;
  reconnectPending = true;
  cleanup();
  retryTimer = setTimeout(() => {
    retryTimer = null;
    reconnectPending = false;
    connect();
  }, RETRY_MS);
}

function scheduleDead() {
  if (deadTimer) return;
  cleanup();
  deadTimer = setTimeout(() => { deadTimer = null; connect(); }, DEAD_RETRY_MS);
}

function pingServer() {
  const t0 = Date.now();
  // closeTimeout: Promise.race yalnizca SOZU yaristiriyordu; alttaki soket
  // acik kaliyor ve zamanlayici temizlenmiyordu, yani her basarisiz ping bir
  // sizinti birakiyordu. Kutuphanenin kendi zaman asimi soketi de kapatir.
  return mcPing({ host: HOST, port: PORT, closeTimeout: PING_TIMEOUT_MS })
    .then((info) => {
      // Sureyi logla: ping sinirinin dogru olup olmadigini bir daha tahmin
      // etmek zorunda kalmayalim.
      log(`ping ${Date.now() - t0}ms — ${info && info.version ? info.version.name : '?'}`);
      return info;
    })
    .catch((e) => {
      log(`ping basarisiz (${Date.now() - t0}ms): ${e && e.message ? e.message : e}`);
      return null;
    });
}

function extractVersion(info) {
  if (!info || !info.version || !info.version.name) return null;
  const m = String(info.version.name).match(/\b(\d+\.\d+(?:\.\d+)?)\b/);
  return m ? m[1] : null;
}

function armSpawnTimeout() {
  clearTimeout(spawnTimer);
  spawnTimer = setTimeout(() => {
    log('spawn timeout — yeniden deneniyor');
    scheduleReconnect();
  }, 45000);
}

function humanActivity() {
  if (!bot || !bot.entity) return;
  if (VANISHED) {
    if (Math.random() < 0.4) {
      // look() bir soz doner; reddi yakalanmazsa global
      // unhandledRejection'a dusup gereksiz bir yeniden baglanma tetikliyordu.
      bot.look(Math.random() * Math.PI * 2, (Math.random() - 0.5) * 1.2, true).catch(() => {});
    }
    return;
  }
  const roll = Math.random();
  if (roll < 0.30) {
    bot.setControlState('forward', true);
    bot.setControlState('jump', true);
    setTimeout(() => { bot.setControlState('forward', false); bot.setControlState('jump', false); }, 600 + Math.random() * 1200);
  } else if (roll < 0.55) {
    const dir = ['left', 'right'][Math.floor(Math.random() * 2)];
    bot.setControlState(dir, true);
    setTimeout(() => bot.setControlState(dir, false), 500 + Math.random() * 900);
  } else if (roll < 0.75) {
    bot.look(Math.random() * Math.PI * 2, (Math.random() - 0.5) * 1.2, true).catch(() => {});
  } else if (roll < 0.85) {
    bot.setControlState('sneak', true);
    setTimeout(() => bot.setControlState('sneak', false), 400 + Math.random() * 700);
  }
}

function startActivity() {
  stopActivity();
  activityTimer = setInterval(humanActivity, 2500 + Math.random() * 2000);
}

function stopActivity() {
  if (activityTimer) { clearInterval(activityTimer); activityTimer = null; }
  if (bot && typeof bot.setControlState === 'function') {
    ['forward', 'back', 'left', 'right', 'jump', 'sneak'].forEach((c) => bot.setControlState(c, false));
  }
}

function connect() {
  // Ayni anda iki baglanti denemesi olmasin: ping ~12 saniye surebiliyor ve o
  // arada gelen ikinci bir cagri ikinci bir bot uretirdi.
  if (connecting) return;
  connecting = true;
  pingServer().then((info) => {
    const proto = info && info.version && info.version.protocol;
    if (!proto || proto < 1) {
      log(`sunucu cevap vermiyor (kapali/sirada) — ${DEAD_RETRY_MS / 1000}sn sonra tekrar`);
      writeStatus({ connected: false, state: 'waiting', server_protocol: null, error: null, error_code: null });
      connecting = false;
      scheduleDead();
      return;
    }
    // SURUM UYARI KONTROLU: desteklenen max'tan yuksek ise baglanma, uyar
    if (proto > MAX_SUPPORTED_PROTOCOL) {
      const vname = (info.version && info.version.name) || `proto ${proto}`;
      log(`DESTEKLENMEYEN SURUM: server=${vname} (proto ${proto}) > max ${MAX_SUPPORTED_VERSION} (proto ${MAX_SUPPORTED_PROTOCOL})`);
      writeStatus({
        connected: false,
        state: 'unsupported_version',
        server_protocol: proto,
        server_version: vname,
        max_supported_protocol: MAX_SUPPORTED_PROTOCOL,
        max_supported_version: MAX_SUPPORTED_VERSION,
        error: `Server ${vname} desteklenmiyor. Bot en fazla ${MAX_SUPPORTED_VERSION} surumunu destekler. Aternos panelinde Yazılım > Vanilla ${MAX_SUPPORTED_VERSION} secin.`,
      });
      // desteklenmeyen surumde tekrar deneme — bossa bekle
      connecting = false;
      clearTimeout(retryTimer);
      retryTimer = setTimeout(() => { retryTimer = null; connect(); }, 60000);
      return;
    }
    connecting = false;
    spawnBot(info);
  }).catch((e) => {
    connecting = false;
    log(`baglanti hazirligi hatasi: ${e && e.message ? e.message : e}`);
    scheduleDead();
  });
}

function spawnBot(info) {
  currentName = BOT_NAME;
  const detected = extractVersion(info);
  log(`baglaniliyor... (${currentName} @ ${HOST}:${PORT})${detected ? ` [v${detected}]` : ''}`);
  writeStatus({
    connected: false,
    state: 'connecting',
    server_protocol: info.version.protocol,
    server_version: info.version.name,
    max_supported_protocol: MAX_SUPPORTED_PROTOCOL,
    max_supported_version: MAX_SUPPORTED_VERSION,
    name: currentName,
  });

  const botOpts = {
    host: HOST, port: PORT, username: currentName, auth: 'offline',
    hideErrors: true, viewDistance: 'tiny', chatLengthLimit: 256,
  };
  if (detected) botOpts.version = detected;

  bot = mineflayer.createBot(botOpts);

  bot.once('spawn', () => {
    clearTimeout(spawnTimer);
    VANISHED = false;
    log(`GIRDI: ${currentName} — vanish bekleniyor`);
    try {
      bot._client.write('custom_payload', { channel: 'minecraft:brand', data: Buffer.from('vanilla') });
    } catch (e) {}
    setTimeout(() => { try { bot.chat('/gamemode spectator'); } catch (e) {} }, 600);
    setTimeout(() => { try { bot.chat(`/tp ${VANISH_X} ${VANISH_Y} ${VANISH_Z}`); } catch (e) {} }, 1400);
    setTimeout(() => { VANISHED = true; log('vanish aktif — gorunmez modda AFK'); }, 3000);
    writeStatus({ connected: true, state: 'online', name: currentName, vanished: false, error: null, error_code: null });
    setTimeout(() => writeStatus({ vanished: true }), 3000);
    startActivity();
  });

  bot.on('kicked', (reason) => {
    const raw = typeof reason === 'string' ? reason : JSON.stringify(reason);
    log(`KICK: ${raw}`);
    // online-mode=true olan bir sunucu, offline modda baglanan botu
    // "unverified_username" ile atar. Bu, ayar degismeden ASLA duzelmez;
    // ham cevirii anahtarini gostermek yerine panele ne yapilmasi gerektigini
    // soyleyen bir kod yaz (panel bunu kullanicinin dilinde gosterir).
    if (raw.includes('unverified_username') || raw.includes('multiplayer.disconnect.not_whitelisted')) {
      const code = raw.includes('unverified_username') ? 'online_mode' : 'whitelist';
      writeStatus({ connected: false, state: 'kicked', error_code: code, error: raw });
      // 8 saniyede bir bosuna baglanip sunucuyu mesgul etme; ayar degisirse
      // bir dakika icinde kendiliginden girsin.
      cleanup();
      retryTimer = setTimeout(() => { retryTimer = null; connect(); }, 60000);
      return;
    }
    writeStatus({ connected: false, state: 'kicked', error_code: null, error: raw });
    scheduleReconnect();
  });
  bot.on('error', (err) => { log(`hata: ${err.message || err}`); writeStatus({ connected: false, state: 'error', error: err.message || String(err) }); scheduleReconnect(); });
  bot.on('end', () => { log('baglanti koptu'); writeStatus({ connected: false, state: 'disconnected', error: null, error_code: null }); scheduleReconnect(); });

  armSpawnTimeout();
}

function cleanup() {
  stopActivity();
  clearTimeout(spawnTimer);
  if (deadTimer) { clearTimeout(deadTimer); deadTimer = null; }
  if (retryTimer) { clearTimeout(retryTimer); retryTimer = null; }
  connecting = false;
  if (bot) { try { bot.quit(); } catch (e) {} bot.removeAllListeners(); bot = null; }
}

process.on('SIGINT', () => { log('kapatiliyor'); cleanup(); writeStatus({ connected: false, state: 'stopped' }); process.exit(0); });
process.on('SIGTERM', () => { log('SIGTERM'); cleanup(); writeStatus({ connected: false, state: 'stopped' }); process.exit(0); });
process.on('uncaughtException', (err) => { log(`beklenmeyen hata: ${err && err.message ? err.message : err}`); writeStatus({ connected: false, state: 'error', error: err && err.message ? err.message : String(err) }); scheduleReconnect(); });
process.on('unhandledRejection', (err) => { log(`beklenmeyen rejection: ${err && err.message ? err.message : err}`); writeStatus({ connected: false, state: 'error', error: err && err.message ? err.message : String(err) }); scheduleReconnect(); });

writeStatus({ connected: false, state: 'starting', host: HOST, port: PORT, name: BOT_NAME, max_supported_protocol: MAX_SUPPORTED_PROTOCOL, max_supported_version: MAX_SUPPORTED_VERSION });
log(`aterkeep-bot basladi — hedef ${HOST}:${PORT}, isim ${BOT_NAME}, max v${MAX_SUPPORTED_VERSION}`);
connect();
