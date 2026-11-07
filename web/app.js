const statusEl  = document.getElementById('status');
const logEl     = document.getElementById('log');
const canvas    = document.getElementById('canvas');
const ctx       = canvas.getContext('2d');
const screenEl  = document.getElementById('screen');
const modeSel   = document.getElementById('mode');

const log = (line) => { logEl.textContent = line; };
const setState = (s) => { statusEl.textContent = s; statusEl.dataset.state = s; };

// ---- scale mode (persisted) ---------------------------------------------

const savedMode = localStorage.getItem('ghostto.mode') || 'fit';
modeSel.value = savedMode;
applyMode(savedMode);
modeSel.addEventListener('change', () => {
  localStorage.setItem('ghostto.mode', modeSel.value);
  applyMode(modeSel.value);
});

function applyMode(mode) {
  screenEl.dataset.mode = mode;
  paintPlaceholder(mode);
}

function paintPlaceholder(mode) {
  const w = canvas.width, h = canvas.height;
  ctx.fillStyle = '#0b0f14';
  ctx.fillRect(0, 0, w, h);
  ctx.strokeStyle = '#2a3644';
  ctx.lineWidth = 2;
  ctx.strokeRect(8, 8, w - 16, h - 16);
  ctx.fillStyle = '#5b6b7d';
  ctx.font = '16px ui-monospace, monospace';
  ctx.textAlign = 'center';
  ctx.fillText(`ghostto · ${w}×${h} · ${mode}`, w / 2, h / 2);
}

// ---- free resize --------------------------------------------------------

// Canvas stays at guest resolution (backing store unchanged); CSS size
// is independent, so scaling is GPU-accelerated and free.
new ResizeObserver(() => {
  const r = screenEl.getBoundingClientRect();
  screenEl.dataset.size = `${Math.round(r.width)}×${Math.round(r.height)}`;
}).observe(screenEl);

// ---- WebSocket ----------------------------------------------------------

const wsUrl = (() => {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${proto}//${location.host}/ws`;
})();

let ws = null;
let backoff = 500;

function connect() {
  log(`connecting ${wsUrl}`);
  ws = new WebSocket(wsUrl);

  ws.addEventListener('open', () => {
    backoff = 500;
    log('connected');
    ping();
  });

  ws.addEventListener('message', (ev) => handleMessage(ev.data));

  ws.addEventListener('close', () => {
    log('disconnected');
    setState('off');
    setTimeout(connect, backoff);
    backoff = Math.min(backoff * 2, 8000);
  });

  ws.addEventListener('error', () => log('ws error'));
}

function handleMessage(data) {
  let msg;
  try { msg = JSON.parse(data); } catch { return; }
  switch (msg.type) {
    case 'ready':
      setState('ready');
      log(`server ready (v${msg.version})`);
      break;
    case 'guest':
      setState(msg.state);
      break;
    case 'log':
      log(msg.message);
      break;
    case 'error':
      log(`error: ${msg.message}`);
      break;
    case 'resize':
      log(`resize ${msg.w}×${msg.h} accepted=${msg.accepted}` +
          (msg.reason ? ` (${msg.reason})` : ''));
      break;
  }
}

function send(cmd) {
  if (!ws || ws.readyState !== WebSocket.OPEN) return;
  ws.send(JSON.stringify(cmd));
}

function ping() { send({ type: 'ping', t: Date.now() }); }
setInterval(() => {
  if (ws && ws.readyState === WebSocket.OPEN) ping();
}, 5000);

connect();
