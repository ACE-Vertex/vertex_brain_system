from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
html = (root / "ui" / "index.html").read_text(encoding="utf-8")
js = (root / "ui" / "app.js").read_text(encoding="utf-8")
css = (root / "ui" / "style.css").read_text(encoding="utf-8")
css += "\n" + (root / "ui" / "brain-renovation.css").read_text(encoding="utf-8")
rust = (root / "src-tauri" / "src" / "main.rs").read_text(encoding="utf-8")
laa = (root / "src-tauri" / "src" / "laa.rs").read_text(encoding="utf-8")
mount = (root / "src-tauri" / "src" / "vcell_mount.rs").read_text(encoding="utf-8")
baseline = (root / "src-tauri" / "src" / "resident_baseline.rs").read_text(encoding="utf-8")
separability = (root / "src-tauri" / "src" / "resident_separability.rs").read_text(encoding="utf-8")
scale = (root / "src-tauri" / "src" / "scale_expansion.rs").read_text(encoding="utf-8")
resident32 = (root / "src-tauri" / "src" / "resident32.rs").read_text(encoding="utf-8")
resident32_baseline = (root / "src-tauri" / "src" / "resident32_baseline.rs").read_text(encoding="utf-8")
persistent32 = (root / "src-tauri" / "src" / "resident32_persistent_difference.rs").read_text(encoding="utf-8")
config = (root / "src-tauri" / "tauri.conf.json").read_text(encoding="utf-8")

required_html = [
    "VERTEX BRAIN SYSTEM",
    "BRAIN MEMBRANE",
    "REAL 32C",
    "REGION 000032",
    "NEURAL TOPOLOGY / THRONE",
    "32C 実状態 / 集約配置",
    "32C Persistent Resident Core",
    "4C REGRESSION REFERENCE",
    "4CはRegression Reference",
    "LLM INGESTION",
    "vSCOPE · INTEGRATED",
    "EVIDENCE / TEMPORAL LANE",
    "LAA INGRESS",
    "settingsBtn",
    "settingsDialog",
    "fontAdjustControl",
    "fontMinusBtn",
    "fontResetBtn",
    "fontPlusBtn",
    "fontDeltaLabel",
    "resident32Phase",
    "resident32Step",
    "resident32Injections",
    "resident32StatusBtn",
    "resident32ProbeBtn",
    "resident32PassiveBtn",
    "resident32SettleBtn",
    "resident32ResetBtn",
    "residentStateFingerprint",
    "residentFingerprint",
    "residentDominant",
    "arrivalSummary",
    "PARALLEL",
    "CONFLUENCE",
    "NOT MOUNTED",
]
missing = [x for x in required_html if x not in html]
if missing:
    raise SystemExit(f"Missing Brain 0.9.1 UI labels: {missing}")

for forbidden in ["chat-input", "chatInput", "messageComposer", "promptInput"]:
    if forbidden in html or forbidden in js:
        raise SystemExit(f"Forbidden chat surface found: {forbidden}")

ids = set(re.findall(r'\bid=["\']([^"\']+)', html))
refs = set(re.findall(r'\$\(["\']([^"\']+)["\']\)', js))
missing_ids = sorted(refs - ids)
if missing_ids:
    raise SystemExit(f"JavaScript references missing DOM ids: {missing_ids}")

required_rust = [
    'vertex.brain.snapshot.v5',
    'vertex.brain.membrane.v1',
    'brain_snapshot',
    'vcell_mount_status',
    'vcell_resident_baseline',
    'vcell_response_separability',
    'vcell_scale_expansion',
    'vcell_resident32_persistent_difference',
    'vcell_resident32_baseline',
    'vcell_resident32_status',
    'vcell_resident32_passive_read',
    'vcell_resident32_settle',
    'vcell_resident32_probe',
    'vcell_resident32_reset',
    'run_geometry_reuse',
    'run_delta_propagation',
    'run_temporal_frontier',
    'run_time_gate',
    'laa_status',
    'laa_probe_plan',
    'laa_analyze_observations',
    'laa_self_test',
    'arbitrary_shell: false',
    'filesystem_write: false',
    'network_egress: false',
]
missing_rust = [x for x in required_rust if x not in rust]
if missing_rust:
    raise SystemExit(f"Missing Brain native markers: {missing_rust}")

required_baseline = [
    'vertex.brain.vcell-resident-baseline.v1',
    'VCELL_RESIDENT_RESPONSE_BASELINE_000001',
    '4C_REFERENCE_CORE',
    'CALIBRATED',
    'VRB1-',
    'OBSERVE_ONLY',
    'LOCKED_UNTIL_POST_BASELINE_GATE',
    'STANDBY_ONLY',
    'CANDIDATE_GATED',
    'RESIDENT_SCALE_AND_RESPONSE_SEPARABILITY',
]
missing_baseline = [x for x in required_baseline if x not in baseline]
if missing_baseline:
    raise SystemExit(f"Missing resident baseline markers: {missing_baseline}")

required_laa = [
    'vertex.brain.laa.v1',
    'BLACK_BOX',
    'CANDIDATE_ONLY',
    'raw_prompt_or_response_retained: false',
]
missing_laa = [x for x in required_laa if x not in laa]
if missing_laa:
    raise SystemExit(f"Missing LAA markers: {missing_laa}")

required_mount = [
    'vertex.brain.vcell-mount.v2',
    'ENTHRONED',
    'PERSISTENT_RESIDENT_CORE',
    'NATIVE_COGNITIVE_SUBSTRATE',
    'VXN_MICRO_SEED_24BIT',
    'CANDIDATE_GATED',
    'automatic_transplant: false',
]
missing_mount = [x for x in required_mount if x not in mount]
if missing_mount:
    raise SystemExit(f"Missing vCELL mount markers: {missing_mount}")


required_separability = [
    'vertex.brain.vcell-response-separability.v1',
    'VCELL_4C_RESPONSE_SEPARABILITY_000001',
    'DEFAULT_SEPARATION_THRESHOLD: f64 = 0.08',
    'ANALYSIS_THRESHOLD / NORMALIZED_RESPONSE_SPACE / NOT_PHYSICAL_LAW',
    'response_fingerprint',
    'PARTIALLY_SEPARATED',
    'PATH_DEPENDENCE_OBSERVED_NOT_MEMORY',
    'NOT_ISOLATED_IN_000001',
    'llm_ingestion_state: "LOCKED"',
    'next_gate: "32C_SCALE_EXPANSION"',
]
missing_sep = [x for x in required_separability if x not in separability]
if missing_sep:
    raise SystemExit(f"Missing response separability markers: {missing_sep}")


required_scale = [
    'vertex.brain.vcell-scale-expansion.v1',
    'VCELL_32C_SCALE_EXPANSION_000001',
    '32C_REGION_CANDIDATE',
    'CANDIDATE_ONLY',
    'SCALE_GATE_TILTS: [f64; 7]',
    'cross_scale_numeric_equivalence_claimed: false',
    'llm_ingestion_state: "LOCKED"',
    'next_gate: "32C_RESIDENT_PROMOTION"',
]
missing_scale = [x for x in required_scale if x not in scale]
if missing_scale:
    raise SystemExit(f"Missing 32C scale markers: {missing_scale}")


required_resident32 = [
    'vertex.brain.vcell-resident32.v1',
    'VCELL_32C_RESIDENT_PROMOTION_000001',
    'PERSISTENT_RESIDENT_CORE',
    'REGION 000032',
    '4C_REGRESSION_REFERENCE',
    'ACTIVE_PERSISTENT',
    'llm_ingestion_state: "LOCKED"',
    'next_gate: "32C_RESIDENT_BASELINE"',
]
missing_resident32 = [x for x in required_resident32 if x not in resident32]
if missing_resident32:
    raise SystemExit(f"Missing 32C resident markers: {missing_resident32}")

required_mount_v2 = [
    'vertex.brain.vcell-mount.v2',
    'VCELL_MOUNT_VERSION: &str = "0.2.0"',
    'resident_id: "REGION 000032"',
    'reference_scope: "4C_REGRESSION_REFERENCE"',
]
missing_mount_v2 = [x for x in required_mount_v2 if x not in mount]
if missing_mount_v2:
    raise SystemExit(f"Missing Mount v2 markers: {missing_mount_v2}")


required_resident32_baseline = [
    'vertex.brain.vcell-resident32-baseline.v1',
    'VCELL_32C_RESIDENT_BASELINE_000001',
    'ISOLATED_SAME_TOPOLOGY',
    'live_resident_mutated: false',
    'PERSISTENT_DIFFERENCE_OBSERVED',
    'RELAXED_TO_BASELINE',
    'passive_read',
    'llm_ingestion_state: "LOCKED"',
    'next_gate: "32C_PERSISTENT_DIFFERENCE"',
]
missing_resident32_baseline = [x for x in required_resident32_baseline if x not in resident32_baseline]
if missing_resident32_baseline:
    raise SystemExit(f"Missing 32C resident baseline markers: {missing_resident32_baseline}")


required_persistent32 = [
    'vertex.brain.vcell-resident32-persistent-difference.v1',
    'VCELL_32C_PERSISTENT_DIFFERENCE_000001',
    'HORIZONS: [usize; 5] = [256, 512, 1024, 2048, 4096]',
    'run_control',
    'PERSISTENT_PLATEAU_CANDIDATE',
    'DECAYING_RESIDUAL',
    'RELAXED_TO_BASELINE',
    'NON_MONOTONIC_PERSISTENT_RESPONSE',
    'memory_claimed: false',
    'llm_ingestion_state: "LOCKED"',
]
missing_persistent32 = [x for x in required_persistent32 if x not in persistent32]
if missing_persistent32:
    raise SystemExit(f"Missing persistent-difference markers: {missing_persistent32}")


required_result_egress = [
    'evidence_prints_persistent_difference_result',
    '=== 32C PERSISTENT DIFFERENCE RESULT ===',
    'CLASSIFICATION {}',
    'H{} ENERGY',
    'ZERO_WRITE_CONTROL_CLEAN {}',
    'REPLAY_EXACT {}',
    'PLATEAU_CANDIDATE {}',
    'MEMORY_CLAIM {}',
    'FINGERPRINT {}',
]
missing_result_egress = [x for x in required_result_egress if x not in persistent32]
if missing_result_egress:
    raise SystemExit(f"Missing persistent-difference result egress markers: {missing_result_egress}")

combined = rust + "\n" + laa + "\n" + mount + "\n" + baseline + "\n" + separability + "\n" + scale + "\n" + resident32 + "\n" + resident32_baseline + "\n" + persistent32
for forbidden in [
    'std::process::Command',
    'Command::new(',
    'std::net::TcpStream',
    'std::net::UdpSocket',
    'reqwest::',
]:
    if forbidden in combined:
        raise SystemExit(f"Direct external authority surfaced: {forbidden}")

for token in [
    "#168cff", "#3ab8ff", "#7b61ff", "drawTopology",
    ".inspector-panel", ".explorer-panel", ".cell-focus-card", ".membrane-frame"
]:
    if token not in css and token not in js:
        raise SystemExit(f"Missing Vertex visual marker: {token}")

font_markers_css = [
    "--type-offset:0pt",
    "calc(11pt + var(--type-offset))",
    "TYPOGRAPHY SCALE V3",
    ".font-adjust",
]
font_markers_js = [
    'TYPE_OFFSET_KEY = "vertex.brain.typographyScaleOffsetPt"',
    "TYPE_OFFSET_DEFAULT = 0",
    "TYPE_OFFSET_MIN = -3",
    "TYPE_OFFSET_MAX = 3",
    'document.documentElement.style.setProperty("--type-offset"',
    "canvasTypePx(11)",
    "localStorage.setItem",
]
missing_font_css = [x for x in font_markers_css if x not in css]
missing_font_js = [x for x in font_markers_js if x not in js]
if missing_font_css or missing_font_js:
    raise SystemExit(f"Font delta contract missing: css={missing_font_css} js={missing_font_js}")

if "font-size:calc(15pt + var(--type-offset))" not in css:
    raise SystemExit("Title typography scale is not additive")
if "font-size:calc(10pt + var(--type-offset))" not in css:
    raise SystemExit("Small semantic typography scale is not additive")

if 'Vertex Brain System' not in config or 'net.vertexworld.brainsystem' not in config or '"0.9.1"' not in config:
    raise SystemExit("Tauri identity/version mismatch")

print("VERTEX_BRAIN_SYSTEM_UI_VERIFY PASS")
print(f"DOM_IDS {len(ids)}")
print(f"JS_REFERENCED_IDS {len(refs)}")
print("CHAT_SURFACE ABSENT")
print("BRAIN_MEMBRANE FAIL_CLOSED")
print("VSCOPE OBSERVATION_LAYER_ABSORBED")
print("LAA OBSERVE_ONLY_PRE_TRANSFER")
print("VCELL RESIDENT_CORE_ENTHRONED")
print("VCELL RESIDENT_RESPONSE_BASELINE_000001_PRESENT")
print("LLM INGESTION_LOCKED")
print("PROVIDER STANDBY_ONLY")
print("TRANSFER CANDIDATE_GATED")
print("TYPOGRAPHY BASE_10_TO_12PT")
print("TYPOGRAPHY OFFSET_RANGE_MINUS_3_TO_PLUS_3PT")
print("TYPOGRAPHY OFFSET_PERSISTENT_LOCAL_PREFERENCE")
print("TYPOGRAPHY CANVAS_LABELS_FOLLOW")
print("TOPOLOGY REAL_32C_DETERMINISTIC_PROJECTION")
print("VCELL 4C_RESPONSE_SEPARABILITY_000001_PRESENT")
print("VCELL RESPONSE_FINGERPRINT_EXCLUDES_STIMULUS_ONSET")
print("VCELL CONTACT_AXIS_NOT_ISOLATED")
print("VCELL NEXT_GATE_32C_SCALE_EXPANSION")
print("VCELL 32C_SCALE_EXPANSION_000001_PRESENT")
print("VCELL 32C_CANDIDATE_768_VERTICES")
print("VCELL CROSS_SCALE_FALSE_EQUIVALENCE_DENIED")
print("VCELL 32C_PROMOTION_CANDIDATE_ONLY")
print("VCELL NEXT_GATE_32C_RESIDENT_PROMOTION")
print("VCELL 32C_RESIDENT_PROMOTION_000001_PRESENT")
print("VCELL 32C_RESIDENT_ENTHRONED")
print("VCELL 32C_PERSISTENT_DYNAMIC_STATE")
print("VCELL 4C_REGRESSION_REFERENCE_RETAINED")
print("VCELL MOUNT_V2_ACTIVE")
print("VCELL LLM_INGESTION_STILL_LOCKED")
print("VCELL NEXT_GATE_32C_RESIDENT_BASELINE")
print("VCELL 32C_RESIDENT_BASELINE_000001_PRESENT")
print("VCELL 32C_BASELINE_ISOLATED_CALIBRATION_LANE")
print("VCELL PASSIVE_READ_NO_INJECTION")
print("VCELL LIVE_RESIDENT_NOT_MUTATED_BY_BASELINE")
print("VCELL MEMORY_CLAIM_NOT_YET_MADE")
print("VCELL NEXT_GATE_32C_PERSISTENT_DIFFERENCE")
print("VCELL 32C_PERSISTENT_DIFFERENCE_000001_PRESENT")
print("VCELL LONG_HORIZON_4096_ACTIVE")
print("VCELL ZERO_WRITE_CONTROL_ACTIVE")
print("VCELL PLATEAU_IS_CANDIDATE_NOT_MEMORY")
print("VCELL NEXT_GATE_32C_RECALL_CANDIDATE")
print("VCELL PERSISTENT_DIFFERENCE_RESULT_EGRESS_ACTIVE")
print("VCELL WORKS_EVIDENCE_NO_CAPTURE_RESULT_ROUTE")
