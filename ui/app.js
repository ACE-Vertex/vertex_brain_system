"use strict";

const $ = id => document.getElementById(id);
const TAURI = Boolean(window.__TAURI__?.core?.invoke);
const nativeInvoke = TAURI ? window.__TAURI__.core.invoke : null;
const TYPE_OFFSET_KEY = "vertex.brain.typographyScaleOffsetPt";
const LANGUAGE_KEY = "vertex.brain.language";
const TYPE_OFFSET_MIN = -3;
const TYPE_OFFSET_MAX = 3;
const TYPE_OFFSET_DEFAULT = 0;

let typeOffsetPt = loadNumber(TYPE_OFFSET_KEY, TYPE_OFFSET_DEFAULT, TYPE_OFFSET_MIN, TYPE_OFFSET_MAX);
let language = loadLanguage();
let snapshot = null;
let residentStatus = null;
let residentPassive = null;
let residentProbe = null;
let busy = false;
let autoFocus = true;
let focusLocked = false;
let eventFilter = "ALL";
let evidenceCount = 0;
let lastPhase = null;
let topologyMode = "THRONE";
let focusedCore = 0;
let topologyHitCells = [];
const events = [];
const stateHistory = [];
const energyHistory = [];
const activeHistory = [];

function loadNumber(key, fallback, min, max) {
  try {
    const value = Number(localStorage.getItem(key));
    return Number.isFinite(value) ? Math.max(min, Math.min(max, Math.round(value))) : fallback;
  } catch (_) { return fallback; }
}

function loadLanguage() {
  try { return localStorage.getItem(LANGUAGE_KEY) === "en" ? "en" : "ja"; }
  catch (_) { return "ja"; }
}

function setText(id, value) {
  const node = $(id);
  if (node) node.textContent = value ?? "—";
}

function fmt(value, digits = 3) {
  return Number.isFinite(Number(value)) ? Number(value).toFixed(digits) : "—";
}

function exp(value) {
  return Number.isFinite(Number(value)) ? Number(value).toExponential(2) : "—";
}

function clamp(value, min, max) { return Math.max(min, Math.min(max, Number(value) || 0)); }
function sumAbs(values) { return (values || []).reduce((sum, value) => sum + Math.abs(Number(value) || 0), 0); }
function nowStamp() { return new Date().toLocaleTimeString("ja-JP", { hour12: false, timeZone: "Asia/Tokyo" }); }
function canvasTypePx(basePt) { return (basePt + typeOffsetPt) * 96 / 72; }

function applyLanguage(next, persist = true) {
  language = next === "en" ? "en" : "ja";
  document.documentElement.lang = language;
  document.querySelectorAll("[data-ja][data-en]").forEach(node => {
    node.textContent = node.dataset[language];
  });
  document.querySelectorAll('input[name="language"]').forEach(input => { input.checked = input.value === language; });
  if (persist) {
    try { localStorage.setItem(LANGUAGE_KEY, language); } catch (_) {}
  }
}

function applyTypeOffset(next, persist = true) {
  typeOffsetPt = clamp(Math.round(next), TYPE_OFFSET_MIN, TYPE_OFFSET_MAX);
  document.documentElement.style.setProperty("--type-offset", `${typeOffsetPt}pt`);
  document.documentElement.style.setProperty("--type-growth", `${Math.max(0, typeOffsetPt * 3)}pt`);
  setText("fontDeltaLabel", `${typeOffsetPt > 0 ? "+" : ""}${typeOffsetPt}pt`);
  if (persist) {
    try { localStorage.setItem(TYPE_OFFSET_KEY, String(typeOffsetPt)); } catch (_) {}
  }
  drawAll();
}

async function invoke(name, args) {
  if (!TAURI) throw new Error(language === "ja" ? "Native Core未接続 — 実データのみ表示します" : "Native Core disconnected — real data only");
  return nativeInvoke(name, args);
}

function addEvent(kind, title, detail, level = "normal") {
  const item = { id: ++evidenceCount, kind, title, detail, level, time: nowStamp() };
  events.push(item);
  if (events.length > 80) events.shift();
  setText("metricEvidence", evidenceCount);
  setText("metricEvidenceDelta", `+${Math.min(99, evidenceCount)}`);
  renderEvents();
  renderSnapshots();
  renderAxis();
}

function renderEvents() {
  const lane = $("eventStream");
  if (!lane) return;
  lane.replaceChildren();
  const visible = events.filter(item => eventFilter === "ALL" || item.kind === eventFilter).slice(-18);
  if (!visible.length) {
    const empty = document.createElement("div");
    empty.className = "event-empty";
    empty.textContent = language === "ja" ? "観測イベントを待機中" : "Waiting for observed events";
    lane.appendChild(empty);
    return;
  }
  visible.forEach(item => {
    const card = document.createElement("article");
    card.className = `event-card ${item.kind.toLowerCase()} ${item.level}`;
    const icon = document.createElement("div");
    icon.className = "event-icon";
    icon.textContent = item.kind === "TRIGGER" ? "△" : item.kind === "EVIDENCE" ? "▱" : item.kind === "STATE" ? "◉" : "◇";
    const body = document.createElement("div");
    const meta = document.createElement("small"); meta.textContent = `${item.kind} · ${item.time}`;
    const title = document.createElement("b"); title.textContent = item.title;
    const detail = document.createElement("span"); detail.textContent = item.detail;
    body.append(meta, title, detail); card.append(icon, body); lane.appendChild(card);
  });
  if ($("autoScroll")?.checked) lane.scrollLeft = lane.scrollWidth;
}

function renderSnapshots() {
  const box = $("evidenceSnapshots");
  if (!box) return;
  box.replaceChildren();
  const items = events.filter(item => ["EVIDENCE", "STATE", "TRIGGER"].includes(item.kind)).slice(-4);
  while (items.length < 4) items.unshift({ id: "—", time: "—", kind: "IDLE" });
  items.forEach(item => {
    const node = document.createElement("div"); node.className = "snapshot";
    const visual = document.createElement("div"); visual.className = "snapshot-visual";
    const id = document.createElement("b"); id.textContent = item.id === "—" ? "NOT OBSERVED" : `EV-${String(item.id).padStart(5, "0")}`;
    const time = document.createElement("span"); time.textContent = item.time;
    node.append(visual, id, time); box.appendChild(node);
  });
}

function renderAxis() {
  const axis = $("streamAxis");
  if (!axis) return;
  axis.replaceChildren();
  const items = events.slice(-6);
  (items.length ? items : [{ time: "—" }]).forEach(item => {
    const tick = document.createElement("span"); tick.textContent = item.time; axis.appendChild(tick);
  });
}

function pushState(phase) {
  stateHistory.push({ phase, time: nowStamp() });
  if (stateHistory.length > 6) stateHistory.shift();
  renderStateTimeline();
}

function renderStateTimeline() {
  const line = $("stateTimeline");
  if (!line) return;
  line.replaceChildren();
  const items = stateHistory.length ? stateHistory : [{ phase: "NOT OBSERVED", time: "—" }];
  items.forEach((item, index) => {
    const node = document.createElement("div"); node.className = `state-node ${index === items.length - 1 ? "active" : ""}`;
    const dot = document.createElement("i"); const phase = document.createElement("b"); phase.textContent = item.phase;
    const time = document.createElement("span"); time.textContent = item.time;
    node.append(dot, phase, time); line.appendChild(node);
  });
}

function renderMembrane(membrane = {}) {
  const integrity = Number(membrane.integrity_percent);
  setText("metricMembrane", Number.isFinite(integrity) ? `${fmt(integrity, 0)}%` : "—");
  setText("metricMembraneState", membrane.state || "NOT OBSERVED");
  setText("membraneIntegrity", Number.isFinite(integrity) ? `${fmt(integrity, 1)}%` : "—");
  setText("membraneThreat", membrane.state || "NOT OBSERVED");
  setText("nodeMembraneState", membrane.state || "NOT OBSERVED");
  if ($("integrityFill")) $("integrityFill").style.width = Number.isFinite(integrity) ? `${clamp(integrity, 0, 100)}%` : "0%";
  setText("capShell", membrane.arbitrary_shell === false ? "DENIED" : "N/A");
  setText("capFilesystem", membrane.filesystem_write === false ? "DENIED" : "N/A");
  setText("capNetwork", membrane.network_egress === false ? "DENIED" : "N/A");
  renderGates(membrane.capabilities || []);
}

function renderGates(capabilities) {
  const box = $("membraneGates");
  if (!box) return;
  box.replaceChildren();
  if (!capabilities.length) {
    const node = document.createElement("span"); node.className = "gate-empty"; node.textContent = "NOT OBSERVED"; box.appendChild(node); return;
  }
  capabilities.forEach(capability => {
    const node = document.createElement("div");
    node.className = `gate ${capability.authority === "EXPERIMENT" ? "warn" : ""}`;
    const icon = document.createElement("i");
    const label = document.createElement("span"); label.textContent = capability.authority;
    node.title = `${capability.name} · ${capability.direction}`;
    node.append(icon, label); box.appendChild(node);
  });
}

function renderLaa(laa = {}) {
  setText("metricLaaState", (laa.state || "NOT OBSERVED").replaceAll("_", " "));
  setText("metricLaaDepth", (laa.active_depth || "—").replace("_BOX", ""));
  setText("nodeLaa", (laa.state || "OBSERVE ONLY").replaceAll("_", " "));
}

function renderResidentStatus(status = {}) {
  residentStatus = status;
  const cells = status.cells;
  const vertices = status.vertices;
  setText("metricVcellMountState", status.state || "NOT OBSERVED");
  setText("metricCells", Number.isFinite(Number(cells)) ? `${cells}C` : "—");
  setText("resident32Phase", (status.phase || "—").replaceAll("_", " "));
  setText("resident32Step", status.step_index ?? "—");
  setText("resident32Injections", status.injection_count ?? "—");
  setText("resident32Epoch", status.probe_epoch ?? "—");
  setText("resident32Active", `${status.active_cells ?? "—"} / ${cells ?? "—"}`);
  setText("resident32Contacts", status.contact_points ?? "—");
  setText("resident32Energy", exp(status.total_signal_energy));
  setText("resident32Peak", exp(status.peak_amplitude));
  setText("resident32Reference", status.four_c_reference_retained === true ? "RETAINED" : "NOT OBSERVED");
  setText("resident32Llm", status.llm_ingestion_state || "LOCKED");
  setText("resident32Residency", (status.residency || "—").replace("PERSISTENT_RESIDENT_CORE", "PERSISTENT"));
  setText("vcellResidentId", status.resident_id || "REGION 000032");
  setText("vcellResidency", (status.residency || "—").replaceAll("_", " "));
  setText("vcellPhase", (status.phase || "—").replaceAll("_", " "));
  setText("vcellSeed", status.seed_hex || "—");
  setText("vcellLaaIngress", status.llm_ingestion_state || "LOCKED");
  setText("throneState", status.state || "—");
  setText("topologyCells", cells ?? "—");
  setText("topologyVertices", vertices ?? "—");
  setText("topologyShells", Array.isArray(status.shell_counts) ? status.shell_counts.join(" · ") : "—");
  setText("topologyGate", Number.isFinite(Number(status.gate_tilt_deg)) ? `${fmt(status.gate_tilt_deg, 1)}°` : "—");
  setText("nodeGeometry", Number.isFinite(Number(vertices)) ? `${vertices} VERTICES` : "NOT OBSERVED");
  setText("nodeContacts", Number.isFinite(Number(status.contact_points)) ? `${status.contact_points} contacts` : "NOT OBSERVED");
  setText("nodeTemporal", Number.isFinite(Number(status.step_index)) ? `STEP ${status.step_index}` : "NOT OBSERVED");
  setText("metricMemory", status.probe_epoch ?? "—");
  setText("metricActive", Number.isFinite(Number(status.active_cells)) ? `${status.active_cells}/${cells}` : "—");
  setText("metricEnergy", exp(status.total_signal_energy));
  setText("metricNeuralState", Number(status.active_cells) > 0 ? "OBSERVED" : "QUIESCENT");
  updateFocusedCore(focusedCore);
}

function renderPassive(read = {}) {
  residentPassive = read;
  setText("resident32Phase", (read.phase || residentStatus?.phase || "—").replaceAll("_", " "));
  setText("resident32Step", read.step_index ?? residentStatus?.step_index ?? "—");
  setText("resident32Injections", read.injection_count ?? residentStatus?.injection_count ?? "—");
  setText("resident32Epoch", read.probe_epoch ?? residentStatus?.probe_epoch ?? "—");
  setText("resident32Active", `${read.active_cells ?? "—"} / ${residentStatus?.cells ?? "—"}`);
  setText("resident32Energy", exp(read.total_signal_energy));
  setText("resident32Peak", exp(read.peak_amplitude));
  setText("residentStateFingerprint", read.state_fingerprint || "NOT OBSERVED");
  setText("signalValue", exp(read.total_signal_energy));
  setText("metricEnergy", exp(read.total_signal_energy));
  setText("metricActive", `${read.active_cells ?? "—"}/${residentStatus?.cells ?? "—"}`);
  setText("nodeTemporal", Number.isFinite(Number(read.step_index)) ? `STEP ${read.step_index}` : "NOT OBSERVED");
  setText("nodeFlow", Number(read.total_signal_energy) > 0 ? `ENERGY ${exp(read.total_signal_energy)}` : "QUIESCENT");
  const shells = read.shell_energy || [];
  const shellMax = Math.max(...shells.map(Number), 1e-12);
  for (let index = 0; index < 3; index++) {
    setText(`shellEnergy${index}`, exp(shells[index]));
    const bar = $(`shellEnergyBar${index}`);
    if (bar) bar.style.width = Number.isFinite(Number(shells[index])) ? `${clamp(Number(shells[index]) / shellMax * 100, 0, 100)}%` : "0%";
  }
  energyHistory.push(Number(read.total_signal_energy) || 0); if (energyHistory.length > 80) energyHistory.shift();
  activeHistory.push(Number(read.active_cells) || 0); if (activeHistory.length > 80) activeHistory.shift();
  updateFocusedCore(focusedCore);
  drawAll();
}

function renderProbe(report = {}) {
  residentProbe = report;
  setText("residentFingerprint", report.response_fingerprint || "NOT OBSERVED");
  setText("residentDominant", Number.isFinite(Number(report.dominant_cell)) ? `CELL ${String(report.dominant_cell).padStart(2, "0")} · ${fmt((report.dominant_share || 0) * 100, 2)}%` : "NOT OBSERVED");
  setText("residentReached", Number.isFinite(Number(report.reached_cells)) ? `${report.reached_cells} / ${residentStatus?.cells || 32}` : "NOT OBSERVED");
  const arrivals = report.arrival_step_by_cell || [];
  const reached = arrivals.filter(value => value !== null && value !== undefined).map(Number);
  setText("arrivalSummary", reached.length ? `${reached.length} cells · first ${Math.min(...reached)} / last ${Math.max(...reached)} steps` : "NOT OBSERVED");
  renderTopologyAvailability();
  if (autoFocus && !focusLocked && Number.isFinite(Number(report.dominant_cell))) focusedCore = Math.max(0, Number(report.dominant_cell) - 1);
  updateFocusedCore(focusedCore);
  if (autoFocus && !focusLocked) selectTopologyMode("FLOW"); else drawTopology();
}

function renderFrameReference(frame = {}, frameRole = "4C_REGRESSION_REFERENCE") {
  setText("obsState", `${frameRole === "4C_REGRESSION_REFERENCE" ? "4C" : "REF"} · ${(frame.phase || "NOT OBSERVED").replaceAll("_", " ")}`);
  setText("telemetryError", exp(frame.error));
  setText("telemetryShell", exp(frame.outer_shell_distance));
  setText("telemetryEgress", Number.isFinite(Number(sumAbs(frame.egress))) ? fmt(sumAbs(frame.egress), 4) : "—");
  setText("obsEvals", frame.evaluations ?? "—");
}

function renderSnapshot(next) {
  snapshot = next;
  renderResidentStatus(next.resident32 || {});
  renderMembrane(next.membrane || {});
  renderLaa(next.laa || {});
  renderFrameReference(next.frame || {}, next.frame_role);
  setText("systemStatus", "OPERATIONAL");
  setText("systemSubstatus", language === "ja" ? "Native Core接続 · Passive observation" : "Native Core connected · Passive observation");
  if ($("systemStatus")) $("systemStatus").style.color = "var(--vertex-success)";
  const phase = next.resident32?.phase;
  if (phase && phase !== lastPhase) {
    if (lastPhase !== null) addEvent("STATE", `${lastPhase} → ${phase}`, "REGION 000032 state transition observed");
    pushState(phase.replaceAll("_", " "));
    lastPhase = phase;
  }
}

function sizeCanvas(canvas) {
  const rect = canvas.getBoundingClientRect();
  const ratio = window.devicePixelRatio || 1;
  const width = Math.max(1, Math.floor(rect.width * ratio));
  const height = Math.max(1, Math.floor(rect.height * ratio));
  if (canvas.width !== width || canvas.height !== height) { canvas.width = width; canvas.height = height; }
  const ctx = canvas.getContext("2d");
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  return { ctx, width: rect.width, height: rect.height };
}

function drawLineChart(canvas, data, color = "#35d9ff") {
  if (!canvas) return;
  const { ctx, width, height } = sizeCanvas(canvas);
  ctx.clearRect(0, 0, width, height);
  ctx.strokeStyle = "rgba(67,138,180,.12)"; ctx.lineWidth = 1;
  for (let index = 1; index < 4; index++) { const y = height * index / 4; ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(width, y); ctx.stroke(); }
  if (data.length < 2) return;
  const min = Math.min(...data); const max = Math.max(...data); const span = Math.max(max - min, 1e-12); const pad = 3;
  ctx.strokeStyle = color; ctx.lineWidth = 1.3; ctx.beginPath();
  data.forEach((value, index) => { const x = pad + (width - pad * 2) * index / (data.length - 1); const y = height - pad - (height - pad * 2) * ((value - min) / span); index ? ctx.lineTo(x, y) : ctx.moveTo(x, y); });
  ctx.stroke();
}

function topologyCells(width, height) {
  const counts = residentStatus?.shell_counts;
  const expected = Number(residentStatus?.cells);
  if (!Array.isArray(counts) || counts.reduce((sum, value) => sum + Number(value || 0), 0) !== expected) return [];

  const centerX = width * .43;
  const centerY = height * .51;
  const scaleBase = Math.min(width * .72, height);
  const goldenAngle = Math.PI * (3 - Math.sqrt(5));
  const cells = [{ index: 0, shell: 0, x: centerX, y: centerY, depth: 1, scale: 1.18 }];
  let index = 1;

  function buildShell(shell, count, radiusX, radiusY, phase) {
    for (let slot = 0; slot < count; slot++) {
      const vertical = 1 - 2 * ((slot + .5) / count);
      const radial = Math.sqrt(Math.max(0, 1 - vertical * vertical));
      const angle = slot * goldenAngle + phase;
      const x3 = Math.cos(angle) * radial;
      const depth = Math.sin(angle) * radial;
      const x = centerX + x3 * radiusX + depth * radiusX * .12;
      const y = centerY + vertical * radiusY - depth * radiusY * .16;
      cells.push({ index, shell, x, y, depth, scale: .82 + (depth + 1) * .14 });
      index++;
    }
  }

  buildShell(1, Number(counts[1]), scaleBase * .265, scaleBase * .245, .28);
  buildShell(2, Number(counts[2]), scaleBase * .435, scaleBase * .385, 1.07);
  return cells;
}

function updateFocusedCore(index = focusedCore) {
  const total = Number(residentStatus?.cells) || 32;
  focusedCore = clamp(Math.round(index), 0, Math.max(0, total - 1));
  const shell0 = Number(residentStatus?.shell_counts?.[0]) || 1;
  const shell1 = Number(residentStatus?.shell_counts?.[1]) || 14;
  const shell = focusedCore < shell0 ? 0 : focusedCore < shell0 + shell1 ? 1 : 2;
  const energy = residentPassive?.cell_energy?.[focusedCore];
  const arrival = residentProbe?.arrival_step_by_cell?.[focusedCore];
  const state = Number.isFinite(Number(energy)) ? (Number(energy) > 0 ? "ACTIVE" : "PASSIVE") : "NOT OBSERVED";
  const name = focusedCore === 0 ? "CELL 000001 · GENESIS" : `CELL ${String(focusedCore + 1).padStart(6, "0")}`;
  setText("topologyFocusName", name);
  setText("topologyFocusShell", shell === 0 ? "ROOT / THRONE" : shell === 1 ? "INNER SHELL" : "OUTER SHELL");
  setText("topologyFocusState", state);
  setText("topologyFocusEnergy", Number.isFinite(Number(energy)) ? exp(energy) : "—");
  setText("topologyFocusArrival", arrival === null || arrival === undefined ? "NOT OBSERVED" : `STEP ${arrival}`);
  setText("focusedCell", `${name} · REGION 000032`);
  drawTopology();
}

function drawCore(ctx, cell, visual, focused) {
  const baseRadius = cell.shell === 0 ? 11 : cell.shell === 1 ? 6.3 : 5.3;
  const radius = (baseRadius + visual * 3.2) * cell.scale;
  const alpha = cell.shell === 2 ? .56 : .72;
  if (focused) {
    ctx.strokeStyle = "rgba(145,242,255,.72)"; ctx.lineWidth = 1.3;
    ctx.beginPath(); ctx.arc(cell.x, cell.y, radius + 7, 0, Math.PI * 2); ctx.stroke();
    ctx.fillStyle = "rgba(53,217,255,.07)"; ctx.beginPath(); ctx.arc(cell.x, cell.y, radius + 12, 0, Math.PI * 2); ctx.fill();
  }
  ctx.fillStyle = visual > 0 ? `rgba(68,218,246,${Math.min(1, alpha + visual * .28)})` : `rgba(78,142,170,${alpha})`;
  ctx.shadowColor = visual > 0 ? "#35d9ff" : "rgba(53,217,255,.28)";
  ctx.shadowBlur = visual > 0 ? 7 + visual * 12 : 3;
  ctx.beginPath(); ctx.arc(cell.x, cell.y, radius, 0, Math.PI * 2); ctx.fill(); ctx.shadowBlur = 0;
  ctx.strokeStyle = cell.shell === 0 ? "rgba(167,245,255,.86)" : cell.shell === 1 ? "rgba(74,211,236,.56)" : "rgba(101,139,181,.48)";
  ctx.lineWidth = cell.shell === 0 ? 1.4 : .8;
  ctx.beginPath(); ctx.arc(cell.x, cell.y, radius + 2.5, 0, Math.PI * 2); ctx.stroke();
  ctx.fillStyle = "rgba(225,250,255,.9)"; ctx.beginPath(); ctx.arc(cell.x - radius * .22, cell.y - radius * .24, Math.max(1, radius * .15), 0, Math.PI * 2); ctx.fill();
  return radius;
}

function drawTopology() {
  const canvas = $("networkCanvas");
  if (!canvas) return;
  const { ctx, width, height } = sizeCanvas(canvas);
  ctx.clearRect(0, 0, width, height);
  if (!residentStatus?.cells) {
    ctx.fillStyle = "#7891a0"; ctx.font = `${canvasTypePx(11)}px "Cascadia Mono", monospace`; ctx.textAlign = "center";
    ctx.fillText(language === "ja" ? "NATIVE CORE 接続待機 — 合成表示なし" : "AWAITING NATIVE CORE — NO SYNTHETIC DISPLAY", width / 2, height / 2); return;
  }

  const cells = topologyCells(width, height);
  const energies = residentPassive?.cell_energy || [];
  const shares = residentProbe?.cell_peak_share || [];
  const arrivals = residentProbe?.arrival_step_by_cell || [];
  const maxEnergy = Math.max(...energies.map(Number), 1e-12);
  const maxShare = Math.max(...shares.map(Number), 1e-12);
  const root = cells[0];

  const field = ctx.createRadialGradient(root.x, root.y, 8, root.x, root.y, Math.min(width, height) * .46);
  field.addColorStop(0, "rgba(28,151,191,.10)"); field.addColorStop(.42, "rgba(24,93,130,.055)"); field.addColorStop(1, "rgba(3,9,14,0)");
  ctx.fillStyle = field; ctx.fillRect(0, 0, width, height);

  if (Number(residentStatus.root_degree) === Number(residentStatus.shell_counts?.[1])) {
    cells.filter(cell => cell.shell === 1).forEach(cell => {
      ctx.strokeStyle = `rgba(64,178,211,${.12 + (cell.depth + 1) * .035})`; ctx.lineWidth = .8;
      ctx.beginPath(); ctx.moveTo(root.x, root.y); ctx.quadraticCurveTo((root.x + cell.x) / 2 + cell.depth * 12, (root.y + cell.y) / 2 - 8, cell.x, cell.y); ctx.stroke();
    });
  }

  const ordered = [...cells].sort((a, b) => a.depth - b.depth || a.index - b.index);
  topologyHitCells = [];
  ordered.forEach(cell => {
    const energy = Number(energies[cell.index]) || 0;
    const share = Number(shares[cell.index]) || 0;
    const arrival = arrivals[cell.index];
    const energyRatio = clamp(energy / maxEnergy, 0, 1);
    const shareRatio = clamp(share / maxShare, 0, 1);
    const arrived = arrival !== null && arrival !== undefined;
    const visual = topologyMode === "EVIDENCE" ? shareRatio : topologyMode === "FLOW" && arrived ? 1 - clamp(Number(arrival) / Math.max(1, residentProbe?.steps || 1), 0, 1) : energyRatio;
    const radius = drawCore(ctx, cell, visual, cell.index === focusedCore);
    topologyHitCells.push({ ...cell, hitRadius: Math.max(12, radius + 6) });
  });

  ctx.textAlign = "left"; ctx.fillStyle = "rgba(199,237,246,.92)"; ctx.font = `600 ${canvasTypePx(11)}px "Cascadia Mono", monospace`;
  ctx.fillText("GENESIS", root.x + 19, root.y - 5);
  ctx.fillStyle = "rgba(100,154,175,.92)"; ctx.font = `${canvasTypePx(10)}px "Cascadia Mono", monospace`;
  ctx.fillText("CELL 000001 · ROOT", root.x + 19, root.y + 12);
  ctx.fillStyle = "rgba(88,129,148,.72)";
  ctx.fillText(`INNER SHELL · ${residentStatus.shell_counts?.[1] ?? "—"} CORE`, 22, 34);
  ctx.fillText(`OUTER SHELL · ${residentStatus.shell_counts?.[2] ?? "—"} CORE`, 22, 53);
}

function drawAll() {
  drawTopology();
  drawLineChart($("signalCanvas"), energyHistory, "#35d9ff");
  drawLineChart($("neuralSpark"), activeHistory, "#28dfc6");
  drawLineChart($("memorySpark"), energyHistory, "#7b61ff");
}

function renderTopologyAvailability() {
  document.querySelectorAll(".topology-modes [data-mode]").forEach(button => {
    const needsProbe = ["FLOW", "EVIDENCE"].includes(button.dataset.mode);
    button.disabled = needsProbe && !residentProbe;
    button.title = button.disabled ? "NOT OBSERVED — run PULSE 32C" : "";
  });
}

function selectTopologyMode(mode) {
  const button = document.querySelector(`.topology-modes [data-mode="${mode}"]`);
  if (!button || button.disabled) return;
  topologyMode = mode;
  document.querySelectorAll(".topology-modes [data-mode]").forEach(item => item.classList.toggle("active", item.dataset.mode === mode));
  setText("topologyModeLabel", mode);
  drawTopology();
}

async function pollSnapshot(first = false) {
  if (busy && !first) return;
  try {
    const [nextSnapshot, passive] = await Promise.all([invoke("brain_snapshot"), invoke("vcell_resident32_passive_read")]);
    renderSnapshot(nextSnapshot); renderPassive(passive);
    if (first) addEvent("SYSTEM", "BRAIN SYSTEM ONLINE", "32C Resident + vSCOPE passive observation connected");
  } catch (error) {
    setText("systemStatus", "DISCONNECTED");
    setText("systemSubstatus", String(error));
    if ($("systemStatus")) $("systemStatus").style.color = "var(--vertex-warning)";
    renderTopologyAvailability(); drawAll();
  }
}

function showProbeOutput(title, result) {
  setText("probeOutput", `${title}\n\n${JSON.stringify(result, null, 2).slice(0, 28000)}`);
}

async function runResident32(name) {
  if (busy) return;
  busy = true; setText("probeOutput", `RUNNING ${name}…`);
  try {
    const args = name === "vcell_resident32_probe" || name === "vcell_resident32_settle" ? { steps: 256 } : undefined;
    const result = await invoke(name, args);
    if (name === "vcell_resident32_probe") {
      renderProbe(result); showProbeOutput("32C RESIDENT PROBE", result);
      addEvent("EVIDENCE", "32C RESIDENT PULSE", `${result.response_fingerprint} · reached ${result.reached_cells}/${residentStatus?.cells || 32}`);
      const [status, passive] = await Promise.all([invoke("vcell_resident32_status"), invoke("vcell_resident32_passive_read")]); renderResidentStatus(status); renderPassive(passive);
    } else if (name === "vcell_resident32_passive_read" || name === "vcell_resident32_settle") {
      renderPassive(result); showProbeOutput(name === "vcell_resident32_settle" ? "32C SETTLE" : "32C PASSIVE READ", result);
      addEvent(name.endsWith("settle") ? "TRIGGER" : "SYSTEM", name.endsWith("settle") ? "32C SETTLE" : "PASSIVE READ", `${result.state_fingerprint} · injection=${result.injection_performed}`);
      if (name.endsWith("settle")) renderResidentStatus(await invoke("vcell_resident32_status"));
    } else {
      renderResidentStatus(result); showProbeOutput("32C RESIDENT STATUS", result);
      addEvent("SYSTEM", "32C RESIDENT", `${result.phase} · ${result.cells}C/${result.vertices}V`);
      if (name.endsWith("reset")) { residentProbe = null; setText("residentFingerprint", "NOT OBSERVED"); setText("residentDominant", "NOT OBSERVED"); setText("residentReached", "NOT OBSERVED"); setText("arrivalSummary", "NOT OBSERVED — run PULSE 32C"); renderTopologyAvailability(); }
    }
  } catch (error) { setText("probeOutput", `ERROR\n${error}`); addEvent("SYSTEM", "32C command failed", String(error), "warning"); }
  finally { busy = false; }
}

async function runDiagnostic(name, kind = "EVIDENCE", args) {
  if (busy) return;
  busy = true; setText("probeOutput", `RUNNING ${name}…`);
  try {
    const result = await invoke(name, args);
    showProbeOutput(name.replaceAll("_", " ").toUpperCase(), result);
    addEvent(kind, name.replaceAll("_", " ").toUpperCase(), `${result.state || result.classification || result.schema || "observed"}`);
  } catch (error) { setText("probeOutput", `ERROR\n${error}`); addEvent("SYSTEM", `${name} failed`, String(error), "warning"); }
  finally { busy = false; }
}

function focusPanel(name) {
  document.querySelectorAll("[data-focus]").forEach(item => item.classList.toggle("active", item.dataset.focus === name));
  const targets = [...document.querySelectorAll(`[data-panel="${name}"]`)];
  const target = targets.find(node => node.offsetParent !== null) || targets[0];
  if (!target) return;
  target.classList.remove("is-focus"); void target.offsetWidth; target.classList.add("is-focus");
}

function topologyCellFromPointer(event) {
  const canvas = $("networkCanvas");
  if (!canvas) return null;
  const rect = canvas.getBoundingClientRect();
  const x = event.clientX - rect.left; const y = event.clientY - rect.top;
  return [...topologyHitCells]
    .sort((a, b) => b.depth - a.depth)
    .find(cell => Math.hypot(cell.x - x, cell.y - y) <= cell.hitRadius) || null;
}

function wireUi() {
  $("settingsBtn")?.addEventListener("click", () => $("settingsDialog")?.showModal());
  document.querySelectorAll('input[name="language"]').forEach(input => input.addEventListener("change", () => applyLanguage(input.value)));
  $("fontMinusBtn")?.addEventListener("click", () => applyTypeOffset(typeOffsetPt - 1));
  $("fontPlusBtn")?.addEventListener("click", () => applyTypeOffset(typeOffsetPt + 1));
  $("fontResetBtn")?.addEventListener("click", () => applyTypeOffset(TYPE_OFFSET_DEFAULT));
  $("diagnosticsBtn")?.addEventListener("click", () => { $("probeDrawer")?.classList.add("open"); $("probeDrawer")?.setAttribute("aria-hidden", "false"); });
  $("closeDiagnostics")?.addEventListener("click", () => { $("probeDrawer")?.classList.remove("open"); $("probeDrawer")?.setAttribute("aria-hidden", "true"); });
  $("autoFocusBtn")?.addEventListener("click", () => { autoFocus = !autoFocus; $("autoFocusBtn").classList.toggle("active", autoFocus); addEvent("SYSTEM", "AUTO FOCUS", autoFocus ? "ENABLED" : "DISABLED"); });
  $("focusLockBtn")?.addEventListener("click", () => { focusLocked = !focusLocked; $("focusLockBtn").classList.toggle("active", focusLocked); setText("focusLockBtn", focusLocked ? "● Locked" : "⌾ Lock"); });
  $("networkCanvas")?.addEventListener("pointermove", event => { if (!focusLocked) { const cell = topologyCellFromPointer(event); if (cell && cell.index !== focusedCore) updateFocusedCore(cell.index); } });
  $("networkCanvas")?.addEventListener("pointerleave", () => { if (!focusLocked) updateFocusedCore(autoFocus && residentProbe?.dominant_cell ? Number(residentProbe.dominant_cell) - 1 : 0); });
  $("networkCanvas")?.addEventListener("click", event => { const cell = topologyCellFromPointer(event); if (cell) updateFocusedCore(cell.index); });
  $("networkCanvas")?.addEventListener("keydown", event => {
    if (!["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) return;
    event.preventDefault(); const direction = event.key === "ArrowLeft" || event.key === "ArrowUp" ? -1 : 1; updateFocusedCore(focusedCore + direction);
  });
  $("resident32PassiveBtn")?.addEventListener("click", () => runResident32("vcell_resident32_passive_read"));
  $("resident32ProbeBtn")?.addEventListener("click", () => runResident32("vcell_resident32_probe"));
  $("resident32SettleBtn")?.addEventListener("click", () => runResident32("vcell_resident32_settle"));
  $("resident32ResetBtn")?.addEventListener("click", () => runResident32("vcell_resident32_reset"));
  $("resident32StatusBtn")?.addEventListener("click", () => runResident32("vcell_resident32_status"));
  document.querySelectorAll("[data-resident32]").forEach(button => button.addEventListener("click", () => runResident32(button.dataset.resident32)));
  document.querySelectorAll("[data-probe]").forEach(button => button.addEventListener("click", () => runDiagnostic(button.dataset.probe)));
  document.querySelectorAll("[data-baseline]").forEach(button => button.addEventListener("click", () => runDiagnostic(button.dataset.baseline)));
  document.querySelectorAll("[data-separability]").forEach(button => button.addEventListener("click", () => runDiagnostic(button.dataset.separability)));
  document.querySelectorAll("[data-scale]").forEach(button => button.addEventListener("click", () => runDiagnostic(button.dataset.scale)));
  document.querySelectorAll("[data-persistent32]").forEach(button => button.addEventListener("click", () => runDiagnostic(button.dataset.persistent32)));
  document.querySelectorAll("[data-baseline32]").forEach(button => button.addEventListener("click", () => runDiagnostic(button.dataset.baseline32)));
  document.querySelectorAll("[data-laa]").forEach(button => button.addEventListener("click", () => runDiagnostic(button.dataset.laa, "SYSTEM", button.dataset.laa === "laa_probe_plan" ? { depth: "BLACK_BOX" } : undefined)));
  document.querySelectorAll("[data-focus]").forEach(button => button.addEventListener("click", () => focusPanel(button.dataset.focus)));
  document.querySelectorAll(".topology-modes [data-mode]").forEach(button => button.addEventListener("click", () => selectTopologyMode(button.dataset.mode)));
  document.querySelectorAll("#eventFilters button").forEach(button => button.addEventListener("click", () => { eventFilter = button.dataset.filter; document.querySelectorAll("#eventFilters button").forEach(item => item.classList.toggle("active", item === button)); renderEvents(); }));
  $("topologyFocusBtn")?.addEventListener("click", () => focusPanel("throne"));
  $("topologyExpandBtn")?.addEventListener("click", () => { document.querySelector(".topology-panel")?.classList.toggle("expanded"); setTimeout(drawAll, 50); });
  window.addEventListener("resize", drawAll);
}

function clock() {
  const now = new Date();
  setText("clockDate", new Intl.DateTimeFormat("ja-JP", { timeZone: "Asia/Tokyo", year: "numeric", month: "2-digit", day: "2-digit" }).format(now).replaceAll("/", "."));
  setText("clockTime", new Intl.DateTimeFormat("ja-JP", { timeZone: "Asia/Tokyo", hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit" }).format(now));
}

(async function boot() {
  applyLanguage(language, false); applyTypeOffset(typeOffsetPt, false); renderEvents(); renderSnapshots(); renderAxis(); renderStateTimeline(); renderGates([]); renderTopologyAvailability(); wireUi(); clock(); setInterval(clock, 1000); drawAll();
  await pollSnapshot(true);
  setInterval(() => pollSnapshot(false), 1000);
})();
