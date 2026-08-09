const $ = (id) => document.getElementById(id);
const logEl = $("log");
let I18N = {};

// === ONBOARDING / SETUP ===
// Kurulum overlay'i tek yerden yonetilir: checkBoot() (asagida) /api/boot'u
// sorgular ve gerekirse overlay'i acar. Gonderim mantigi setupSubmit'te.

async function loadI18n() {
  let lang = localStorage.getItem("aterkeep_lang") || "tr";
  try {
    const r = await fetch("/api/i18n/" + lang);
    I18N = await r.json();
  } catch (e) { I18N = {}; }
  applyI18n();
  // language selector
  const sel = $("langSelect");
  sel.innerHTML = "";
  (I18N.langs || []).forEach(l => {
    const o = document.createElement("option");
    o.value = l.code; o.textContent = l.name;
    if (l.code === lang) o.selected = true;
    sel.appendChild(o);
  });
  sel.addEventListener("change", async (e) => {
    localStorage.setItem("aterkeep_lang", e.target.value);
    await loadI18n();
  });
}

function t(key) {
  return I18N[key] || key;
}

function applyI18n() {
  document.querySelectorAll("[data-i18n]").forEach(el => {
    const val = t(el.dataset.i18n);
    // Eger elementin child elementi varsa (span vb.), onlari koru:
    // sadece leading text node'u guncelle, child elementleri dokunma.
    if (el.children.length > 0) {
      const firstChild = el.firstChild;
      if (firstChild && firstChild.nodeType === Node.TEXT_NODE) {
        firstChild.nodeValue = val + " ";
      } else {
        el.insertBefore(document.createTextNode(val + " "), el.firstChild);
      }
    } else {
      el.textContent = val;
    }
  });
  $("inspNote").textContent = t("insp_note");
  $("optionsNote").textContent = t("settings_note");
  const footer = document.querySelector("footer");
  if (footer) footer.childNodes[0].textContent = t("footer") + " ";
  renderStatusIfLoaded();
}

let lastStatus = null;
function renderStatusIfLoaded() { if (lastStatus) renderStatus(lastStatus); }

const states = {
  online:    { label: "ONLINE",   cls: "online",   key: "st_already" },
  already:   { label: "ONLINE",   cls: "online",   key: "st_already" },
  starting:  { label: "STARTING", cls: "starting", key: "st_starting" },
  loading:   { label: "LOADING",  cls: "starting", key: "st_starting" },
  queue:     { label: "QUEUE",    cls: "starting", key: "st_queue" },
  inline:    { label: "QUEUE",    cls: "starting", key: "st_queue" },
  eula:      { label: "EULA",     cls: "starting", key: "st_eula" },
  stopping:  { label: "STOPPING", cls: "starting", key: "st_starting" },
  saving:    { label: "SAVING",   cls: "starting", key: "st_starting" },
  waiting:   { label: "WAITING",  cls: "starting", key: "st_starting" },
  pending:   { label: "PENDING",  cls: "starting", key: "st_starting" },
  crashed:   { label: "CRASHED",  cls: "offline",  key: "st_offline" },
  offline:   { label: "OFFLINE",  cls: "offline",  key: "st_offline" },
  boot:      { label: "BOOT",     cls: "starting", key: "st_starting" },
  unknown:   { label: "UNKNOWN",  cls: "",         key: "st_unknown" },
};

// === ANTI-IDLE BOT ===
const botStates = {
  stopped:             { label: "OFFLINE",     cls: "bot-offline",     key: "bot_state_stopped" },
  starting:            { label: "CONNECTING",  cls: "bot-connecting",  key: "bot_state_connecting" },
  connecting:          { label: "CONNECTING",  cls: "bot-connecting",  key: "bot_state_connecting" },
  online:              { label: "ONLINE",      cls: "bot-online",      key: "bot_state_online" },
  waiting:             { label: "WAITING",     cls: "bot-connecting",  key: "bot_state_waiting" },
  kicked:              { label: "KICKED",      cls: "bot-error",       key: "bot_state_kicked" },
  error:               { label: "ERROR",       cls: "bot-error",       key: "bot_state_error" },
  disconnected:        { label: "OFFLINE",     cls: "bot-offline",     key: "bot_state_stopped" },
  unsupported_version: { label: "UNSUPPORTED", cls: "bot-warn",        key: "bot_state_unsupported" },
};
let lastBotState = null;

function renderBot(d) {
  const st = botStates[d.state] || botStates.stopped;
  const el = $("botState");
  el.textContent = st.label;
  el.className = "badge " + st.cls;
  $("botDesc").innerHTML = esc(t(st.key));
  $("botName").textContent = d.name || "—";
  $("botHost").textContent = d.host ? (d.host + (d.port && d.port !== 25565 ? ":" + d.port : "")) : "—";
  $("botVersion").textContent = d.server_version || "—";
  $("botVanished").textContent = d.vanished ? "✓" : "—";
  // toggle checkbox — only set, never overwrite user interaction mid-flight
  if (document.activeElement !== $("botToggle")) $("botToggle").checked = !!d.enabled;

  // unsupported version / error notice
  const note = $("botError");
  if (d.state === "unsupported_version") {
    let msg = t("bot_unsupported");
    const ver = d.max_supported_version ? "≤ " + d.max_supported_version : "";
    if (msg.includes("{ver}")) msg = msg.replace("{ver}", ver).trim();
    else if (ver) msg += " (" + ver + ")";
    note.textContent = msg;
    note.hidden = false;
  } else if (d.error) {
    note.textContent = d.error;
    note.hidden = false;
  } else {
    note.hidden = true;
  }

  // populate config inputs only when empty (don't clobber typing)
  const fill = (id, val) => { const e = $(id); if (e && !e.value && val != null) e.value = val; };
  fill("botCfgName", d.name);
  fill("botCfgHost", d.host);
  fill("botCfgPort", d.port);
  if (d.vanished) {
    // vanish coords come from config if present; backend may not expose them, leave as-is otherwise
  }

  // log state transitions
  if (lastBotState !== null && lastBotState !== d.state) {
    addLine("bot", `bot → ${st.label.toLowerCase()} (${d.state})`);
  }
  lastBotState = d.state;
}

async function loadBot() {
  try {
    const r = await fetch("/api/bot");
    if (!r.ok) return;
    const d = await r.json();
    renderBot(d);
  } catch (e) { /* bot API not present yet — silent */ }
}

$("botToggle").addEventListener("change", async (e) => {
  const on = e.target.checked;
  addLine("cmd", `$ bot ${on ? "on" : "off"}`);
  try {
    await fetch(`/api/bot/toggle?on=${on}`);
    addLine("bot", t(on ? "bot_enabled" : "bot_disabled"));
  } catch (e) { addLine("err", "bot toggle failed"); }
  loadBot();
});

$("botCfgSave").addEventListener("click", async () => {
  const btn = $("botCfgSave");
  const oldText = btn.textContent;
  btn.disabled = true;
  btn.textContent = "…";
  const body = {
    name: $("botCfgName").value.trim(),
    host: $("botCfgHost").value.trim(),
    port: parseInt($("botCfgPort").value, 10),
    vanish_x: parseFloat($("botCfgVX").value),
    vanish_y: parseFloat($("botCfgVY").value),
    vanish_z: parseFloat($("botCfgVZ").value),
  };
  try {
    const r = await fetch("/api/bot/config", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const j = await r.json();
    addLine(j.ok ? "bot" : "err", j.ok ? `${t("bot_cfg_title")}: OK` : "bot config ERR " + JSON.stringify(j));
  } catch (e) { addLine("err", "bot config save failed"); }
  btn.disabled = false;
  btn.textContent = oldText;
  loadBot();
});

function clock() { $("clock").textContent = new Date().toLocaleTimeString("tr-TR"); }
setInterval(clock, 1000); clock();

function addLine(kind, text, container) {
  container = container || logEl;
  const ts = new Date().toLocaleTimeString("tr-TR", { hour12: false });
  const div = document.createElement("div");
  div.className = "line";
  div.innerHTML = `<span class="time">[${ts}]</span> <span class="k-${kind}">${esc(text)}</span>`;
  container.appendChild(div);
  while (container.children.length > 500) container.removeChild(container.firstChild);
  container.scrollTop = container.scrollHeight;
}
function esc(s) { return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;"); }

function bootLines() {
  [
    ["sys", "aterkeep (rust)"],
    ["ok", "session.enc cozuldu"],
    ["ok", "canli akis aktif"],
    ["http", "sifreli HTTP"],
  ].forEach(([k, t], i) => setTimeout(() => addLine(k, t), 250 + i * 220));
}

function renderStatus(s) {
  lastStatus = s;
  const st = states[s.state] || states.unknown;
  const el = $("statusValue");
  el.textContent = st.label;
  el.className = "badge " + st.cls;
  $("statusDesc").innerHTML = esc(t(st.key));
  $("serverAddr").textContent = s.server_addr || "—";
  // kuyruk karti (sirada beklerken ws/poll besler)
  const q = s.queue;
  const qb = $("queueBox");
  if (q && (q.position !== undefined || q.time)) {
    qb.hidden = false;
    $("queueTime").textContent = q.time || "—";
    $("queuePos").textContent = q.position !== undefined ? `${q.position} / ${q.count}` : "—";
    const pct = q.percent !== undefined ? Math.max(0, Math.min(100, q.percent)) : 0;
    $("queuePct").textContent = `%${pct.toFixed(0)}`;
    $("queueFill").style.width = pct + "%";
  } else {
    qb.hidden = true;
  }
  $("serverId").textContent = s.server_id || "—";
  $("lastCheck").textContent = s.last_check ? new Date(s.last_check * 1000).toLocaleTimeString("tr-TR", { hour12: false }) : "—";
  $("loopState").textContent = s.running ? "…" : (s.auto ? "7/24" : "off");
  $("autoToggle").checked = !!s.auto;
  if (s.last_request) renderInspector(s.last_request);
  document.querySelectorAll(".btn").forEach(b => (b.disabled = !!s.running));
}

function renderInspector(req) {
  if (!req) return;
  const endpointMap = {
    start: "server/start",
    stop: "server/stop",
    restart: "server/restart",
    "start+eula": "server/start",
    cancel: "server/cancel",
    extend: "server/extend",
  };
  const action = req.action;
  const ep = (action && endpointMap[action]) || action || "?";
  const sid = (lastStatus && lastStatus.server_id) || "?";
  $("inspCurl").textContent = `$ (sifreli istek) SERVER=${sid}`;
  $("inspJson").textContent = JSON.stringify(req.response, null, 2);
  const st = req.response && req.response.data && req.response.data.status;
  $("inspNote").textContent = st ? `Cevaptaki state: "${st}".` : "Cevapta state yok — hata mesajına bak.";
}

async function refresh() {
  try {
    const r = await fetch("/api/status");
    renderStatus(await r.json());
  } catch (e) {
    console.warn("refresh failed:", e);
    addLine("err", "durum yenilenemedi");
  }
}

document.querySelectorAll(".btn").forEach((btn) => {
  btn.addEventListener("click", async () => {
    const action = btn.dataset.action;
    addLine("cmd", `$ aternos ${action}`);
    // cancel/extend ayri endpoint'lerde (kuyruk iptali / idle uzatma).
    const url = (action === "cancel" || action === "extend")
      ? `/api/${action}`
      : `/api/action/${action}`;
    try { await fetch(url); } catch (e) {}
    refresh();
  });
});

$("autoToggle").addEventListener("change", async (e) => {
  await fetch(`/api/toggle?on=${e.target.checked}`);
  addLine("sys", `auto ${e.target.checked ? "ON" : "OFF"}`);
});

document.querySelectorAll(".tab").forEach((t) => {
  t.addEventListener("click", () => {
    document.querySelectorAll(".tab").forEach(x => x.classList.remove("active"));
    document.querySelectorAll(".tab-panel").forEach(x => x.classList.remove("active"));
    t.classList.add("active");
    $(`tab-${t.dataset.tab}`).classList.add("active");
    if (t.dataset.tab === "console") loadConsole();
    if (t.dataset.tab === "settings") loadOptions();
    if (t.dataset.tab === "players") loadPlayers();
    if (t.dataset.tab === "bot") refreshBot();
  });
});

async function loadConsole() {
  try {
    const r = await fetch("/api/console");
    const data = await r.json();
    // API ya {"lines":[...], "source":"ws"|"http"} ya da eski usul [...] dizi doner.
    const lines = Array.isArray(data) ? data : (data.lines || []);
    const el = $("serverConsole");
    el.innerHTML = "";
    lines.forEach(l => {
      const div = document.createElement("div");
      div.className = "line";
      // Her eleman ya [time, level, msg] dizisi ya da {t, level, text} nesnesi.
      const time = Array.isArray(l) ? l[0] : (l.t || l.time || "");
      const level = Array.isArray(l) ? l[1] : (l.level || l.kind || "");
      const msg = Array.isArray(l) ? l[2] : (l.text || l.line || "");
      div.innerHTML = `<span class="time">[${esc(time)}]</span> <span class="k-dim">[${esc(level)}]</span> ${esc(msg)}`;
      el.appendChild(div);
    });
    el.scrollTop = el.scrollHeight;
  } catch (e) { console.warn("loadConsole failed:", e); }
}

async function loadOptions() {
  try {
    const r = await fetch("/api/options");
    const opts = await r.json();
    const tb = document.querySelector("#optionsTable tbody");
    tb.innerHTML = "";
    Object.entries(opts).forEach(([k, v]) => {
      const tr = document.createElement("tr");
      const name = document.createElement("td");
      name.textContent = k;
      const val = document.createElement("td");
      const input = document.createElement("input");
      input.value = v;
      val.appendChild(input);
      const act = document.createElement("td");
      const btn = document.createElement("button");
      btn.className = "save-btn";
      btn.textContent = t("save");
      btn.onclick = async () => {
        btn.textContent = "…";
        try {
          const res = await fetch(`/api/options/set?name=${encodeURIComponent(k)}&value=${encodeURIComponent(input.value)}`);
          const j = await res.json();
          addLine(j.success ? "ok" : "err", `${k}=${input.value} ${j.success ? "saved" : "ERR " + JSON.stringify(j)}`);
        } catch (e) { addLine("err", "save failed"); }
        btn.textContent = t("save");
      };
      act.appendChild(btn);
      tr.append(name, val, act);
      tb.appendChild(tr);
    });
  } catch (e) { console.warn("loadOptions failed:", e); }
}

async function loadPlayers() {
  try {
    const r = await fetch("/api/players");
    const p = await r.json();
    $("playersCount").textContent = `${p.online || 0}`;
    const el = $("playerList");
    el.innerHTML = "";
    if (p.names && p.names.length) {
      p.names.forEach(n => {
        const chip = document.createElement("div");
        chip.className = "player-chip";
        chip.textContent = n;
        el.appendChild(chip);
      });
    } else {
      const empty = document.createElement("div");
      empty.className = "player-empty";
      empty.textContent = t("no_players");
      el.appendChild(empty);
    }
  } catch (e) { console.warn("loadPlayers failed:", e); }
}

loadI18n();
bootLines();
refresh();
checkNeedsSetup();
loadBot();
setInterval(refresh, 3000);
setInterval(loadBot, 5000);
setInterval(() => { if ($("tab-console").classList.contains("active")) loadConsole(); }, 10000);
setInterval(refreshBot, 4000);

// ---------- SETUP ----------
async function checkBoot() {
  try {
    const r = await fetch("/api/boot");
    const b = await r.json();
    if (b.setup_mode) {
      $("setupOverlay").hidden = false;
      document.querySelectorAll(".tab").forEach(t => t.disabled = true);
    }
  } catch (e) {}
}
checkBoot();

$("resetSessionBtn").addEventListener("click", async () => {
  if (!confirm("Mevcut oturum silinecek ve setup ekranına dönülecek. Devam?")) return;
  await fetch("/api/setup/reset", { method: "POST" });
  location.reload();
});

$("setupSubmit").addEventListener("click", async () => {
  const cookie = $("setupCookie").value.trim();
  let token = $("setupToken").value.trim();
  const st = $("setupStatus");
  if (!cookie) { st.textContent = "Cookie boş"; st.className = "setup-status err"; return; }
  if (!cookie.includes("ATERNOS_SESSION")) {
    st.innerHTML = "Cookie'de ATERNOS_SESSION yok — Network sekmesinden tüm cookie header'ını kopyaladığından emin ol"; st.className = "setup-status err"; return;
  }
  if (!token) { st.textContent = "Token gerekli (F12 → Console → window.AJAX_TOKEN + '|' + window.generateAjaxToken())"; st.className = "setup-status err"; return; }
  st.textContent = "kuruluyor..."; st.className = "setup-status";
  $("setupSubmit").disabled = true;
  try {
    const r = await fetch("/api/setup", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ cookie, token }),
    });
    const j = await r.json();
    if (j.ok) {
      st.textContent = "✓ " + (j.msg || "başarılı"); st.className = "setup-status ok";
      // boot polling: setup_mode false olunca direkt gizle + reload (race condition onle)
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
            // backend hala setup diyor — yine de reload, belki duzelir
            location.reload();
          }
        } catch (e) { if (tries < 8) setTimeout(poll, 400); }
      };
      setTimeout(poll, 600);
    } else {
      st.textContent = "✗ " + (j.error || "hata"); st.className = "setup-status err";
    }
  } catch (e) { st.textContent = "✗ ağ hatası"; st.className = "setup-status err"; }
  $("setupSubmit").disabled = false;
});

// ---------- BOT ----------
function refreshStatusBanner(s) {
  // surum uyarisi: bot baglanamadi + unsupported_version
  fetch("/api/bot/status").then(r => r.json()).then(b => {
    const st = b.status || {};
    const banner = $("botVersionBanner");
    if (st.state === "unsupported_version") {
      banner.hidden = false;
      banner.className = "banner banner-warn";
      banner.innerHTML = `⚠ <b>Bot desteklenmeyen sürüm:</b> Sunucu ${st.server_version} ancak bot en fazla <b>${st.max_supported_version}</b> destekliyor. Aternos panelinde <b>Yazılım → Vanilla ${st.max_supported_version}</b> sürümüne düşürün, sonra bot otomatik bağlanır.`;
    } else if (st.state === "error" && st.error) {
      banner.hidden = false;
      banner.className = "banner banner-warn";
      banner.textContent = "⚠ Bot hatası: " + st.error;
    } else {
      banner.hidden = true;
    }
  }).catch(() => {});
}

async function refreshBot() {
  const s = lastStatus;
  if (s && s.state) refreshStatusBanner(s);
  if (!$("tab-bot")) return;
  try {
    const r = await fetch("/api/bot/config");
    const b = await r.json();
    const cfg = b.config || {};
    if (document.activeElement?.tagName !== "INPUT" || !$("tab-bot").classList.contains("active")) {
      $("cfgHost").value = cfg.host || "";
      $("cfgPort").value = cfg.port || "";
      $("cfgName").value = cfg.name || "";
      $("cfgVX").value = cfg.vanish_x ?? "";
      $("cfgVY").value = cfg.vanish_y ?? "";
      $("cfgVZ").value = cfg.vanish_z ?? "";
    }
    const st = b.status || {};
    const running = b.running;
    $("botStateBadge").textContent = running ? (st.connected ? "ONLINE" : "ÇALIŞIYOR") : "KAPALI";
    $("botStateBadge").className = "badge " + (st.connected ? "online" : (running ? "starting" : "offline"));
    $("botStateText").textContent = st.state || (running ? "starting" : "stopped");
    $("botServerVer").textContent = st.server_version || "—";
    $("botMaxVer").textContent = st.max_supported_version || "—";
    $("botNodeAvail").textContent = b.node_available ? "var" : "YOK (kurulu değil)";
    $("botDetail").textContent = st.error ? st.error : (running ? "aktif" : "devre dışı");
    const ebox = $("botErrorBox");
    if (st.error && st.state !== "unsupported_version") { ebox.hidden = false; ebox.textContent = st.error; }
    else { ebox.hidden = true; }
  } catch (e) {}
}

$("cfgSaveBtn").addEventListener("click", async () => {
  const body = {
    host: $("cfgHost").value, port: parseInt($("cfgPort").value),
    name: $("cfgName").value,
    vanish_x: parseFloat($("cfgVX").value), vanish_y: parseFloat($("cfgVY").value), vanish_z: parseFloat($("cfgVZ").value),
  };
  $("cfgSaveBtn").textContent = "…";
  await fetch("/api/bot/config", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });
  $("cfgSaveBtn").textContent = "Kaydet";
  refreshBot();
});
$("botStartBtn").addEventListener("click", async () => {
  await fetch("/api/bot/toggle", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ enabled: true }) });
  addLine("cmd", "$ bot start");
  refreshBot();
});
$("botStopBtn").addEventListener("click", async () => {
  await fetch("/api/bot/toggle", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ enabled: false }) });
  addLine("cmd", "$ bot stop");
  refreshBot();
});

const es = new EventSource("/api/stream");
es.onmessage = (ev) => {
  try {
    const d = JSON.parse(ev.data);
    if (d.line) addLine(d.kind, d.line);
  } catch (e) {}
};
