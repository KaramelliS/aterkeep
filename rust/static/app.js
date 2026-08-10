const $ = (id) => document.getElementById(id);
const logEl = $("log");
let I18N = {};
// Saat/tarih bicimi de dile baglidir; ceviri yuklenene kadar Ingilizce.
let LOCALE = "en-GB";

// Kurulum/giris kapisi EN ONCE acilir ve dosyanin geri kalanina bagimli
// degildir. Asagidaki herhangi bir ust seviye ifade hata verirse (eksik bir
// element, bozuk bir cevap...) kullanici en azindan kurulum veya giris
// ekranini gorur — bos bir panelle bas basa kalmaz.
checkBoot();

// Kurulum ve giris butonlari da BURADA baglanir. Daha once dosyanin sonuna
// yakin bagleniyorlardi: aradaki herhangi bir ust seviye ifade hata verse
// buton dinleyicisiz kaliyor, kullaniciya "tikliyorum ama hicbir sey olmuyor"
// diye yansiyordu. Kullanicinin ilerlemesini saglayan iki kontrol, uygulamanin
// geri kalanindan once ve ondan bagimsiz olarak hazir olmali.
$("setupSubmit").addEventListener("click", submitSetup);
$("loginForm").addEventListener("submit", submitLogin);

// Yakalanmamis bir hata olursa kullanici bos ekranla kalmasin — ne oldugunu
// gorsun. Sessizce olen bir buton, en kotu hata bicimidir.
// Yakalanmamis soz reddi de goruinsun: sessizce kaybolan bir hata, en kotu
// hata bicimidir.
window.addEventListener("unhandledrejection", (e) => {
  addLine("err", `beklenmeyen hata: ${e.reason && e.reason.message ? e.reason.message : e.reason}`);
});

window.addEventListener("error", (e) => {
  const box = $("setupStatus");
  if (box && !$("setupOverlay").hidden) {
    box.textContent = t("ui_error") + ": " + (e.message || e.error);
    box.className = "setup-status err";
  }
});

// ============================== CEVIRI ==============================

/// Anahtari cevirir. `vars` verilirse metindeki {isim} yer tutucularini doldurur.
/// Anahtar bulunamazsa Ingilizce'ye duser (bunu backend yapar); yine de yoksa
/// anahtarin kendisi doner — ekranda ham anahtar gormek bir hata isaretidir.
function t(key, vars) {
  let s = I18N[key] || key;
  if (vars) {
    // Yer tutucu degerleri KACISLANIR. Ceviri sablonu bizim (icinde bilerek
    // <b>/<code> olabilir) ama degerler bize disaridan gelir: `server_version`
    // dogrudan Minecraft sunucusunun ping yanitindan okunuyor ve dusman bir
    // sunucu oraya istedigini yazabilir. Bu metinler innerHTML'e gittigi icin
    // kacislamamak dogrudan XSS demekti.
    for (const [k, v] of Object.entries(vars)) {
      s = s.split("{" + k + "}").join(esc(String(v ?? "")));
    }
  }
  return s;
}

/// Dil sirasi: (1) bu tarayicida secilen dil, (2) kurulumda config'e yazilan
/// dil, (3) Ingilizce. Kurulum ekrani henuz secim yapilmamis haldedir, yani
/// varsayilan olarak Ingilizce gorunur.
async function currentLang() {
  const stored = localStorage.getItem("aterkeep_lang");
  if (stored) return stored;
  try {
    return (await (await fetch("/api/boot")).json()).lang || "en";
  } catch (e) {
    return "en";
  }
}

async function loadI18n(lang) {
  lang = lang || (await currentLang());
  try {
    I18N = await (await fetch("/api/i18n/" + lang)).json();
  } catch (e) {
    I18N = {};
  }
  LOCALE = I18N.locale || "en-GB";
  document.documentElement.lang = I18N.lang || lang;
  document.documentElement.dir = I18N.dir || "ltr";
  applyI18n();
  fillLangSelect($("langSelect"), I18N.lang || lang);
  fillLangSelect($("setupLang"), I18N.lang || lang);
}

function fillLangSelect(sel, selected) {
  if (!sel) return;
  sel.innerHTML = "";
  (I18N.langs || []).forEach((l) => {
    const o = document.createElement("option");
    o.value = l.code;
    o.textContent = l.name;
    if (l.code === selected) o.selected = true;
    sel.appendChild(o);
  });
}

/// Dil degisimi: secim ANINDA tum arayuze uygulanir (kurulum sihirbazi dahil).
async function switchLang(code) {
  localStorage.setItem("aterkeep_lang", code);
  await loadI18n(code);
}
$("langSelect").addEventListener("change", (e) => switchLang(e.target.value));
$("setupLang").addEventListener("change", (e) => switchLang(e.target.value));

/// Isaretli her elemani cevirir. Metin icinde <b>/<code> gecmesi gerekenler
/// data-i18n-html tasir; geri kalani textContent olarak yazilir (guvenli).
function applyI18n() {
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  document.querySelectorAll("[data-i18n-html]").forEach((el) => {
    el.innerHTML = t(el.dataset.i18nHtml);
  });
  document.querySelectorAll("[data-i18n-ph]").forEach((el) => {
    el.placeholder = t(el.dataset.i18nPh);
  });
  document.querySelectorAll("[data-i18n-title]").forEach((el) => {
    el.title = t(el.dataset.i18nTitle);
  });
  document.querySelectorAll("[data-i18n-aria]").forEach((el) => {
    el.setAttribute("aria-label", t(el.dataset.i18nAria));
  });
  // Dinamik olarak yazilmis metinler data-i18n tasimaz — yeniden cizdir.
  if (lastStatus) renderStatus(lastStatus);
  if (lastBot) renderBot(lastBot);
}

// ============================== DURUM ==============================

let lastStatus = null;

const states = {
  online:   { label: "ONLINE",   cls: "online",   key: "st_already" },
  already:  { label: "ONLINE",   cls: "online",   key: "st_already" },
  starting: { label: "STARTING", cls: "starting", key: "st_starting" },
  loading:  { label: "LOADING",  cls: "starting", key: "st_starting" },
  queue:    { label: "QUEUE",    cls: "starting", key: "st_queue" },
  inline:   { label: "QUEUE",    cls: "starting", key: "st_queue" },
  eula:     { label: "EULA",     cls: "starting", key: "st_eula" },
  stopping: { label: "STOPPING", cls: "starting", key: "st_starting" },
  saving:   { label: "SAVING",   cls: "starting", key: "st_starting" },
  waiting:  { label: "WAITING",  cls: "starting", key: "st_queue" },
  pending:  { label: "PENDING",  cls: "starting", key: "st_queue" },
  crashed:  { label: "CRASHED",  cls: "offline",  key: "st_offline" },
  offline:  { label: "OFFLINE",  cls: "offline",  key: "st_offline" },
  // Dusmus oturum bir sunucu durumu DEGIL, bir hesap durumudur; "kapali" diye
  // gostermek kullaniciyi sunucusunda sorun ariyormus gibi yanlis yone iter.
  session_expired: { label: "SESSION", cls: "error", key: "st_session_expired" },
  boot:     { label: "BOOT",     cls: "starting", key: "st_starting" },
  unknown:  { label: "UNKNOWN",  cls: "",         key: "st_unknown" },
};

function fmtTime(d) {
  return d.toLocaleTimeString(LOCALE, { hour12: false });
}

/// "3g 4sa" gibi kisa sure. Saniye/dakika seviyesi bu baglamda gurultu.
function fmtDuration(sec) {
  const d = Math.floor(sec / 86400);
  const h = Math.floor((sec % 86400) / 3600);
  const m = Math.floor((sec % 3600) / 60);
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

/// Oturum yasi + varsa bir onceki oturumun omru. Ikincisi kullanicinin
/// "ne siklikta cerez yenilemem gerekiyor?" sorusuna kendi verisinden cevap
/// verir — tahmin etmesine gerek kalmaz.
function fmtAge(age, last) {
  if (age == null) return t("session_never");
  const now = fmtDuration(age);
  return last ? `${now} (${t("session_last")}: ${fmtDuration(last)})` : now;
}

function renderStatus(s) {
  lastStatus = s;
  const st = states[s.state] || states.unknown;
  const el = $("statusValue");
  el.textContent = st.label;
  el.className = "badge " + st.cls;
  $("statusDesc").textContent = t(st.key);
  $("serverAddr").textContent = s.server_addr || "—";

  // Durum bandi: bu urunun tek sorusunun cevabi. Renk + sozcuk + konum
  // birlikte tasir; renk tek basina sinyal degildir.
  const band = $("statusBand");
  band.className = "statusband " + st.cls;
  $("statusBandText").textContent = st.label;
  $("statusBandAddr").textContent = s.server_addr || "";

  // Kuyruk karti. Sira onayi arka planda otomatik gonderilir; kart yalnizca
  // bilgi amaclidir (queue_auto notu bunu soyler).
  const q = s.queue;
  const qb = $("queueBox");
  if (q && (q.position !== undefined || q.time)) {
    qb.hidden = false;
    $("queueTime").textContent = q.time || "—";
    $("queuePos").textContent = q.position !== undefined ? `${q.position} / ${q.count}` : "—";
    const pct = q.percent !== undefined ? Math.max(0, Math.min(100, q.percent)) : 0;
    $("queuePct").textContent = pct.toFixed(0) + "%";
    $("queueFill").style.width = pct + "%";
  } else {
    qb.hidden = true;
  }

  $("serverId").textContent = s.server_id || "—";
  $("lastCheck").textContent = s.last_check ? fmtTime(new Date(s.last_check * 1000)) : "—";
  $("loopState").textContent = s.running ? "…" : s.auto ? "24/7" : "off";
  // Cerezlerin kac gundur ayakta oldugu. Aternos cerez omrunu ilan etmiyor;
  // kullanicinin "ne siklikta yenilemem gerekiyor?" sorusuna verebilecegimiz
  // tek durust cevap kendi olctugumuz sayi.
  $("sessionAge").textContent = fmtAge(s.session_age, s.last_session_lifetime);
  $("autoToggle").checked = !!s.auto;

  $("metricPlayers").textContent = `${s.players ?? 0}/${s.slots ?? 20}`;
  $("metricTps").textContent = s.tps ? Number(s.tps).toFixed(1) : "—";
  // RAM esikleri: OOM'a hicbir uyari almadan girmek, kategoride sik gorulen
  // bir hata (BisectHosting bunu forkunda tamamen dusurmus).
  const ramEl = $("metricRam");
  ramEl.textContent = s.heap ? `${s.heap} MB` : "—";
  const ramPct = s.heap && s.heap_max ? s.heap / s.heap_max : 0;
  ramEl.className =
    "metric-value" + (ramPct >= 0.9 ? " err" : ramPct >= 0.8 ? " warn" : "");
  $("metricLink").textContent = s.ws_connected ? t("link_live") : t("link_poll");
  $("metricLink").className = "metric-value " + (s.ws_connected ? "ok" : "dim");

  // Cerezler dustugunde hicbir aksiyon calismaz; kullanicinin bunu boslugu
  // yorumlayarak degil, yazili olarak gormesi gerekiyor.
  const sb = $("sessionBanner");
  sb.hidden = !s.session_expired;
  if (s.session_expired) $("sessionBannerText").innerHTML = t("session_expired_warn");

  if (s.last_request) renderInspector(s.last_request);
  // Yalnizca aksiyon butonlari kilitlenir; form/kaydet butonlari serbest kalir.
  document.querySelectorAll("[data-action]").forEach((b) => (b.disabled = !!s.running));
}

function renderInspector(req) {
  if (!req) return;
  const sid = (lastStatus && lastStatus.server_id) || "?";
  $("inspCurl").textContent = `$ ${t("insp_req")} SERVER=${sid}`;
  $("inspJson").textContent = JSON.stringify(req.response, null, 2);
  const st = req.response && req.response.data && req.response.data.status;
  $("inspNote").textContent = st ? t("insp_state", { st }) : t("insp_nostate");
}

/// Her basarili yoklamada nabiz noktasini yanip sondurur: panelin canli
/// oldugunu ve verinin tazelendigini kullanici gorebilsin.
function pulse() {
  const p = $("updatePulse");
  if (!p) return;
  p.classList.remove("beat");
  // reflow — animasyonu yeniden tetiklemek icin gerekli
  void p.offsetWidth;
  p.classList.add("beat");
  p.title = t("updated") + " " + fmtTime(new Date());
}

async function refresh() {
  try {
    const r = await fetch("/api/status");
    if (r.status === 401) {
      showLogin(true);
      return;
    }
    renderStatus(await r.json());
    pulse();
  } catch (e) {
    addLine("err", t("refresh_fail"));
  }
}

// ============================== BOT ==============================

const botStates = {
  stopped:             { key: "bot_state_stopped",        cls: "offline",  desc: "bot_desc_off" },
  waiting_server:      { key: "bot_state_waiting_server", cls: "starting", desc: "bot_desc_waiting" },
  waiting:             { key: "bot_state_waiting",        cls: "starting", desc: "bot_desc_waiting" },
  starting:            { key: "bot_state_connecting",     cls: "starting", desc: "bot_desc_connecting" },
  connecting:          { key: "bot_state_connecting",     cls: "starting", desc: "bot_desc_connecting" },
  online:              { key: "bot_state_online",         cls: "online",   desc: "bot_desc_online" },
  kicked:              { key: "bot_state_kicked",         cls: "offline",  desc: "bot_desc_waiting" },
  error:               { key: "bot_state_error",          cls: "offline",  desc: "bot_desc_off" },
  disconnected:        { key: "bot_state_waiting_server", cls: "starting", desc: "bot_desc_waiting" },
  unsupported_version: { key: "bot_state_unsupported",    cls: "warn",     desc: "bot_desc_off" },
};

let lastBot = null;
let lastBotState = null;

function renderBot(b) {
  lastBot = b;
  const cfg = b.config || {};
  const st = b.status || {};
  const meta = botStates[st.state] || botStates.stopped;

  const badge = $("botStateBadge");
  badge.textContent = t(meta.key);
  badge.className = "badge " + meta.cls;

  // Cozumu bilinen hatalar icin bot bir kod yollar (online_mode, whitelist...).
  // Kullaniciya ham kick metnini ("multiplayer.disconnect.unverified_username")
  // gostermek hicbir ise yaramaz; ne yapmasi gerektigini soyleyen ceviri var.
  const known = st.error_code ? "bot_err_" + st.error_code : null;

  // "Etkin" ile "sureci calisiyor" ayri seylerdir: bot yalnizca sunucu online
  // oldugunda yasar. Kullanici botu acip sunucu kapaliyken "offline" gorunce
  // bozuk sandi — artik hangi durumda oldugu yaziyor.
  if (known) {
    $("botDetail").innerHTML = t(known);
  } else {
    $("botDetail").textContent =
      st.error && st.state !== "unsupported_version"
        ? t("bot_error_prefix") + ": " + st.error
        : t(meta.desc);
  }
  $("botStateText").textContent = st.state || "stopped";
  $("botRunning").textContent = b.running ? t("bot_run_on") : t("bot_run_off");
  $("botName").textContent = cfg.name || st.name || "—";
  const host = cfg.host || st.host;
  const port = cfg.port || st.port;
  $("botHost").textContent = host ? host + (port && port !== 25565 ? ":" + port : "") : "—";
  $("botServerVer").textContent = st.server_version || "—";
  $("botMaxVer").textContent = st.max_supported_version || "—";
  $("botVanished").textContent = st.vanished ? "✓" : "—";

  if (document.activeElement !== $("botToggle")) $("botToggle").checked = !!cfg.enabled;

  // Kullanici yazarken alanlarin ustune yazma.
  const active = document.activeElement;
  const fill = (id, val) => {
    const el = $(id);
    if (el && el !== active && val != null && val !== "") el.value = val;
  };
  fill("cfgName", cfg.name);
  fill("cfgHost", cfg.host);
  fill("cfgPort", cfg.port);
  fill("cfgVX", cfg.vanish_x);
  fill("cfgVY", cfg.vanish_y);
  fill("cfgVZ", cfg.vanish_z);

  const ebox = $("botErrorBox");
  if (known) {
    ebox.hidden = false;
    ebox.innerHTML = t(known);
  } else if (st.error && st.state !== "unsupported_version") {
    ebox.hidden = false;
    ebox.textContent = t("bot_error_prefix") + ": " + st.error;
  } else {
    ebox.hidden = true;
  }

  // Bot asla baglanamayacak durumdaysa panel tepeden uyarir — kullanici Bot
  // sekmesine bakmasa da sebebini gorsun.
  const banner = $("botVersionBanner");
  if (st.state === "unsupported_version") {
    banner.hidden = false;
    banner.innerHTML = t("bot_version_warn", {
      server: st.server_version || "?",
      max: st.max_supported_version || "?",
    });
  } else if (known) {
    banner.hidden = false;
    banner.innerHTML = t(known);
  } else {
    banner.hidden = true;
  }

  if (lastBotState !== null && lastBotState !== st.state) {
    addLine("bot", `bot → ${st.state}`);
  }
  lastBotState = st.state;
}

/// Tek uc, tek cizim: /api/bot/config config + status + running
/// dondurur. Daha once iki ayri yoklama (loadBot ve refreshBot) ayni alanlari
/// farkli sekillerde yazip birbirini eziyordu.
async function refreshBot() {
  try {
    const r = await fetch("/api/bot/config");
    if (!r.ok) return;
    renderBot(await r.json());
  } catch (e) {}
}

async function setBotEnabled(on) {
  addLine("cmd", `$ bot ${on ? "on" : "off"}`);
  try {
    await fetch("/api/bot/toggle", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ enabled: on }),
    });
    addLine("bot", t(on ? "bot_enabled" : "bot_disabled"));
  } catch (e) {
    addLine("err", "bot toggle failed");
  }
  refreshBot();
}

$("botToggle").addEventListener("change", (e) => setBotEnabled(e.target.checked));
$("botStartBtn").addEventListener("click", () => setBotEnabled(true));
$("botStopBtn").addEventListener("click", () => setBotEnabled(false));

$("cfgSaveBtn").addEventListener("click", async () => {
  const btn = $("cfgSaveBtn");
  const old = btn.textContent;
  btn.disabled = true;
  btn.textContent = "…";
  const body = {
    name: $("cfgName").value.trim(),
    host: $("cfgHost").value.trim(),
    port: parseInt($("cfgPort").value, 10),
    vanish_x: parseFloat($("cfgVX").value),
    vanish_y: parseFloat($("cfgVY").value),
    vanish_z: parseFloat($("cfgVZ").value),
  };
  try {
    const r = await fetch("/api/bot/config", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const j = await r.json();
    addLine(j.ok ? "bot" : "err", j.ok ? `${t("bot_cfg_title")}: OK` : "bot config ERR " + JSON.stringify(j));
  } catch (e) {
    addLine("err", "bot config save failed");
  }
  btn.disabled = false;
  btn.textContent = old;
  refreshBot();
});

// ============================== LOG ==============================

function clock() {
  $("clock").textContent = fmtTime(new Date());
}
setInterval(clock, 1000);
clock();

function addLine(kind, text, container) {
  container = container || logEl;
  // Elemanlar tek tek kuruluyor: `kind` sunucudan geliyor ve sinif adina
  // dogrudan enterpole edilirse ileride bir enjeksiyon yuzeyi olur.
  const div = document.createElement("div");
  div.className = "line";
  const ts = document.createElement("span");
  ts.className = "time";
  ts.textContent = `[${fmtTime(new Date())}]`;
  const body = document.createElement("span");
  body.className = `k-${String(kind).replace(/[^a-z0-9_-]/gi, "")}`;
  body.textContent = text;
  div.append(ts, document.createTextNode(" "), body);
  // Hata satirlari tam genislik hafif bir kirmizi yikama alir: log taranan bir
  // seydir, rengi yalnizca metne koymak yeterince gorunur degil.
  if (kind === "err") div.classList.add("line-err");
  container.appendChild(div);
  while (container.children.length > 500) container.removeChild(container.firstChild);
  container.scrollTop = container.scrollHeight;
}
function esc(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function bootLines() {
  ["boot_1", "boot_2", "boot_3", "boot_4"].forEach((k, i) =>
    setTimeout(() => addLine(["sys", "ok", "ok", "http"][i], t(k)), 250 + i * 220)
  );
}

// ============================== AKSIYONLAR ==============================

// SADECE data-action tasiyan butonlar sunucu aksiyonu tetikler. Onceden bu
// secici tum ".btn" elemanlariniydi — "Kaydet" veya bot butonlarina basmak da
// /api/action/undefined istegine yol aciyordu.
document.querySelectorAll("[data-action]").forEach((btn) => {
  btn.addEventListener("click", async () => {
    const action = btn.dataset.action;
    addLine("cmd", `$ aternos ${action}`);
    const url =
      action === "cancel" || action === "extend" || action === "confirm"
        ? `/api/${action}`
        : `/api/action/${action}`;
    try {
      await fetch(url);
    } catch (e) {}
    refresh();
  });
});

$("autoToggle").addEventListener("change", async (e) => {
  // Basarisizlikta anahtari GERI AL: yoksa arayuz kullaniciya yalan soyler —
  // "acik" gorunur ama daemon kapali bilir.
  const want = e.target.checked;
  try {
    const r = await fetch(`/api/toggle?on=${want}`);
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    addLine("sys", `auto ${want ? "ON" : "OFF"}`);
  } catch (err) {
    e.target.checked = !want;
    addLine("err", `oto-baslat degistirilemedi: ${err.message}`);
  }
});

document.querySelectorAll(".tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".tab").forEach((x) => x.classList.remove("active"));
    document.querySelectorAll(".tab-panel").forEach((x) => x.classList.remove("active"));
    tab.classList.add("active");
    $(`tab-${tab.dataset.tab}`).classList.add("active");
    if (tab.dataset.tab === "console") loadConsole();
    if (tab.dataset.tab === "settings") loadOptions();
    if (tab.dataset.tab === "players") loadPlayers();
    if (tab.dataset.tab === "bot") refreshBot();
  });
});

async function loadConsole() {
  try {
    const data = await (await fetch("/api/console")).json();
    // API ya {"lines":[...], "source":"ws"|"http"} ya da eski usul [...] dizi doner.
    const lines = Array.isArray(data) ? data : data.lines || [];
    const el = $("serverConsole");
    el.innerHTML = "";
    lines.forEach((l) => {
      const div = document.createElement("div");
      div.className = "line";
      const time = Array.isArray(l) ? l[0] : l.t || l.time || "";
      const level = Array.isArray(l) ? l[1] : l.level || l.kind || "";
      const msg = Array.isArray(l) ? l[2] : l.text || l.line || "";
      div.innerHTML = `<span class="time">[${esc(time)}]</span> <span class="k-dim">[${esc(level)}]</span> ${esc(msg)}`;
      el.appendChild(div);
    });
    el.scrollTop = el.scrollHeight;
  } catch (e) {}
}

async function loadOptions() {
  try {
    const opts = await (await fetch("/api/options")).json();
    const tb = document.querySelector("#optionsTable tbody");
    tb.innerHTML = "";
    Object.entries(opts).forEach(([k, v]) => {
      const tr = document.createElement("tr");
      const name = document.createElement("td");
      name.textContent = k;
      const val = document.createElement("td");
      const input = document.createElement("input");
      input.className = "input mono";
      input.value = v;
      val.appendChild(input);
      const act = document.createElement("td");
      const btn = document.createElement("button");
      btn.className = "btn btn-xs";
      btn.textContent = t("save");
      btn.onclick = async () => {
        btn.textContent = "…";
        try {
          const res = await fetch(
            `/api/options/set?name=${encodeURIComponent(k)}&value=${encodeURIComponent(input.value)}`
          );
          const j = await res.json();
          addLine(j.success ? "ok" : "err", `${k}=${input.value} ${j.success ? "saved" : "ERR " + JSON.stringify(j)}`);
        } catch (e) {
          addLine("err", "save failed");
        }
        btn.textContent = t("save");
      };
      act.appendChild(btn);
      tr.append(name, val, act);
      tb.appendChild(tr);
    });
  } catch (e) {}
}

async function loadPlayers() {
  try {
    const p = await (await fetch("/api/players")).json();
    $("playersCount").textContent = `${p.online || 0}`;
    const el = $("playerList");
    el.innerHTML = "";
    if (p.names && p.names.length) {
      p.names.forEach((n) => {
        const chip = document.createElement("li");
        chip.textContent = n;
        el.appendChild(chip);
      });
    } else {
      const empty = document.createElement("li");
      empty.className = "empty";
      empty.textContent = t("no_players");
      el.appendChild(empty);
    }
  } catch (e) {}
}

// ============================== BOOT / LOGIN / SETUP ==============================

function showLogin(show) {
  $("loginOverlay").hidden = !show;
  if (show) setTimeout(() => $("loginPassword").focus(), 50);
}

/// Panel acilisinda hangi ekranin gosterilecegini belirler:
/// kurulum yapilmamis -> setup, jeton yok/gecersiz -> giris, aksi halde panel.
async function checkBoot() {
  try {
    const b = await (await fetch("/api/boot")).json();
    if (b.setup_mode) {
      $("setupOverlay").hidden = false;
      document.querySelectorAll(".tab").forEach((x) => (x.disabled = true));
      return;
    }
    if (b.auth_enabled) {
      // Korumali bir uca istek at: 401 gelirse giris gerekiyor demektir.
      const probe = await fetch("/api/status");
      if (probe.status === 401) {
        showLogin(true);
        return;
      }
    }
    // Panele disaridan erisilebiliyorsa uyar (localhost disi adres).
    const host = location.hostname;
    if (host !== "127.0.0.1" && host !== "localhost" && host !== "[::1]") {
      const el = $("exposureBanner");
      el.textContent = t("exposure_warn");
      el.hidden = false;
    }
  } catch (e) {}
}

async function submitLogin(ev) {
  ev.preventDefault();
  const err = $("loginError");
  const btn = $("loginBtn");
  err.hidden = true;
  btn.disabled = true;
  const old = btn.textContent;
  btn.textContent = "…";
  try {
    const r = await fetch("/api/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ password: $("loginPassword").value }),
    });
    if (r.ok) {
      location.reload();
      return;
    }
    err.textContent = t("login_bad");
    err.hidden = false;
  } catch (e) {
    err.textContent = t("login_neterr");
    err.hidden = false;
  }
  btn.disabled = false;
  btn.textContent = old;
  $("loginPassword").select();
}

// Kurulumun iki yolu. Varsayilan hesapla giris: kullanici DevTools acmaz ve
// cerez 30 gunde dolunca daemon kendi yeniler. Yapistirma yolu 2FA/captcha
// durumlari icin duruyor.
let setupMode = "account";
function selectMode(mode) {
  setupMode = mode;
  $("modeAccount").classList.toggle("active", mode === "account");
  $("modeCookie").classList.toggle("active", mode === "cookie");
  $("paneAccount").hidden = mode !== "account";
  $("paneCookie").hidden = mode !== "cookie";
}
$("modeAccount").addEventListener("click", () => selectMode("account"));
$("modeCookie").addEventListener("click", () => selectMode("cookie"));

async function submitSetup() {
  const password = $("setupPassword").value;
  const password2 = $("setupPassword2").value;
  const lang = $("setupLang").value;
  const st = $("setupStatus");
  const fail = (key) => {
    st.textContent = t(key);
    st.className = "setup-status err";
  };
  if (password.length < 4) return fail("setup_err_pw_short");
  if (password !== password2) return fail("setup_err_pw_match");

  const payload = { password, lang };
  if (setupMode === "account") {
    const u = $("setupAccUser").value.trim();
    const p = $("setupAccPass").value;
    if (!u || !p) return fail("setup_err_account");
    payload.aternos_user = u;
    payload.aternos_pass = p;
    // Sunucu secimi yalnizca backend birden fazla sunucu bildirdiyse gorunur.
    const picked = $("setupServer").value;
    if (picked) payload.server_id = picked;
  } else {
    const cookie = $("setupCookie").value.trim();
    const token = $("setupToken").value.trim();
    if (!cookie) return fail("setup_err_cookie_empty");
    if (!cookie.includes("ATERNOS_SESSION")) return fail("setup_err_cookie_session");
    if (!token) return fail("setup_err_token");
    payload.cookie = cookie;
    payload.token = token;
  }

  st.textContent = t("setup_busy");
  st.className = "setup-status";
  $("setupSubmit").disabled = true;
  try {
    const r = await fetch("/api/setup", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    const j = await r.json();
    // Hesapta birden fazla sunucu varsa backend secim ister — tahmin edip
    // yanlis sunucuyu yonetmektense soruyoruz.
    if (j.need_server) {
      const sel = $("setupServer");
      sel.innerHTML = "";
      j.need_server.forEach((s) => {
        const o = document.createElement("option");
        o.value = s.id;
        o.textContent = s.name;
        sel.appendChild(o);
      });
      $("serverPickWrap").hidden = false;
      st.textContent = t("setup_pick_server");
      st.className = "setup-status";
      $("setupSubmit").disabled = false;
      return;
    }
    if (j.ok) {
      st.textContent = "✓ " + (j.msg || t("setup_ok"));
      st.className = "setup-status ok";
      // Kurulum kendini yeniden baslatir; setup_mode dusunce sayfayi yenile.
      let tries = 0;
      const poll = async () => {
        tries++;
        try {
          const b = await (await fetch("/api/boot")).json();
          if (!b.setup_mode) {
            $("setupOverlay").hidden = true;
            setTimeout(() => location.reload(), 400);
          } else if (tries < 8) {
            setTimeout(poll, 400);
          } else {
            location.reload();
          }
        } catch (e) {
          if (tries < 8) setTimeout(poll, 400);
        }
      };
      setTimeout(poll, 600);
      return; // yeniden baslarken butonu tekrar etkinlestirme
    } else {
      st.textContent = "✗ " + (j.error || t("setup_fail"));
      st.className = "setup-status err";
    }
  } catch (e) {
    st.textContent = "✗ " + t("setup_neterr");
    st.className = "setup-status err";
  }
  $("setupSubmit").disabled = false;
}

$("copyAddr").addEventListener("click", async () => {
  const addr = $("serverAddr").textContent.trim();
  if (!addr || addr === "—") return;
  try {
    await navigator.clipboard.writeText(addr);
    const b = $("copyAddr");
    b.textContent = t("copied");
    setTimeout(() => (b.textContent = t("copy")), 1200);
  } catch (e) {}
});

$("logoutBtn").addEventListener("click", async () => {
  await fetch("/api/logout", { method: "POST" });
  location.reload();
});

// Bannerdaki dugme ile kenar cubugundaki "oturumu yenile" ayni isi yapar:
// mevcut oturumu silip kurulum sihirbazina donerler.
$("sessionRenewBtn").addEventListener("click", () => $("resetSessionBtn").click());

$("resetSessionBtn").addEventListener("click", async () => {
  if (!confirm(t("reset_confirm"))) return;
  await fetch("/api/setup/reset", { method: "POST" });
  location.reload();
});

// ============================== CANLI AKIS ==============================

const es = new EventSource("/api/stream");
es.onmessage = (ev) => {
  try {
    const d = JSON.parse(ev.data);
    if (d.line) addLine(d.kind, d.line);
  } catch (e) {}
};

// Sekme arka plandayken tarayici zamanlayicilari kisar; one gelince veriyi
// hemen tazele — kullanici bayat sayilara bakmasin.
document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    refresh();
    refreshBot();
  }
});

loadI18n().then(bootLines);
refresh();
refreshBot();
setInterval(refresh, 3000);
setInterval(refreshBot, 3000);
setInterval(() => {
  if ($("tab-console").classList.contains("active")) loadConsole();
}, 10000);
