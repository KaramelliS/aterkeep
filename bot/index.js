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
  try {
    const raw = fs.readFileSync(CONFIG_PATH, 'utf8');
    return JSON.parse(raw);
  } catch (e) {
    return {};
  }
}

function writeStatus(patch) {
  try {
    let cur = {};
    try { cur = JSON.parse(fs.readFileSync(STATUS_PATH, 'utf8')); } catch (e) {}
    const next = Object.assign({ ts: Date.now() }, cur, patch);
    fs.writeFileSync(STATUS_PATH, JSON.stringify(next, null, 2));
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
const PING_TIMEOUT_MS = parseInt(process.env.ATERKEEP_BOT_PINGTO || '8000', 10);

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

function scheduleReconnect() {
  if (reconnectPending) return;
  reconnectPending = true;
  cleanup();
  setTimeout(() => { reconnectPending = false; connect(); }, RETRY_MS);
}

function scheduleDead() {
  if (deadTimer) return;
  cleanup();
  deadTimer = setTimeout(() => { deadTimer = null; connect(); }, DEAD_RETRY_MS);
}

function pingServer() {
  return Promise.race([
    mcPing({ host: HOST, port: PORT }),
    new Promise((_, reject) => setTimeout(() => reject(new Error('ping-timeout')), PING_TIMEOUT_MS)),
  ]).catch(() => null);
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
      bot.look(Math.random() * Math.PI * 2, (Math.random() - 0.5) * 1.2, true);
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
    bot.look(Math.random() * Math.PI * 2, (Math.random() - 0.5) * 1.2, true);
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
  pingServer().then((info) => {
    const proto = info && info.version && info.version.protocol;
    if (!proto || proto < 1) {
      log(`sunucu cevap vermiyor (kapali/sirada) — ${DEAD_RETRY_MS / 1000}sn sonra tekrar`);
      writeStatus({ connected: false, state: 'waiting', server_protocol: null });
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
      setTimeout(() => connect(), 60000);
      return;
    }
    spawnBot(info);
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
    writeStatus({ connected: true, state: 'online', name: currentName, vanished: false, error: null });
    setTimeout(() => writeStatus({ vanished: true }), 3000);
    startActivity();
  });

  bot.on('kicked', (reason) => { log(`KICK: ${reason}`); writeStatus({ connected: false, state: 'kicked', error: String(reason) }); scheduleReconnect(); });
  bot.on('error', (err) => { log(`hata: ${err.message || err}`); writeStatus({ connected: false, state: 'error', error: err.message || String(err) }); scheduleReconnect(); });
  bot.on('end', () => { log('baglanti koptu'); writeStatus({ connected: false, state: 'disconnected' }); scheduleReconnect(); });

  armSpawnTimeout();
}

function cleanup() {
  stopActivity();
  clearTimeout(spawnTimer);
  if (deadTimer) { clearTimeout(deadTimer); deadTimer = null; }
  if (bot) { try { bot.quit(); } catch (e) {} bot.removeAllListeners(); bot = null; }
}

process.on('SIGINT', () => { log('kapatiliyor'); cleanup(); writeStatus({ connected: false, state: 'stopped' }); process.exit(0); });
process.on('SIGTERM', () => { log('SIGTERM'); cleanup(); writeStatus({ connected: false, state: 'stopped' }); process.exit(0); });
process.on('uncaughtException', (err) => { log(`beklenmeyen hata: ${err && err.message ? err.message : err}`); writeStatus({ connected: false, state: 'error', error: err && err.message ? err.message : String(err) }); scheduleReconnect(); });
process.on('unhandledRejection', (err) => { log(`beklenmeyen rejection: ${err && err.message ? err.message : err}`); writeStatus({ connected: false, state: 'error', error: err && err.message ? err.message : String(err) }); scheduleReconnect(); });

writeStatus({ connected: false, state: 'starting', host: HOST, port: PORT, name: BOT_NAME, max_supported_protocol: MAX_SUPPORTED_PROTOCOL, max_supported_version: MAX_SUPPORTED_VERSION });
log(`aterkeep-bot basladi — hedef ${HOST}:${PORT}, isim ${BOT_NAME}, max v${MAX_SUPPORTED_VERSION}`);
connect();
