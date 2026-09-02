from pathlib import Path
import json, shutil, sys, time

root = Path.cwd() / "vertex_brain_system"
built = root / "src-tauri" / "target" / "release" / "vertex-brain-system.exe"
version = "0.9.1"
slot = root / "versions" / version
slot.mkdir(parents=True, exist_ok=True)
versioned = slot / "Vertex_Brain_System.exe"
convenience = root / f"Vertex_Brain_System_{version}.exe"
current_json = root / "current.json"
stable_alias = root / "Vertex_Brain_System.exe"

if not built.exists():
    print(f"BRAIN_SYSTEM_BUILD_NOT_FOUND {built}", file=sys.stderr)
    raise SystemExit(2)

shutil.copy2(built, versioned)
shutil.copy2(built, convenience)

tmp = current_json.with_suffix(".json.tmp")
tmp.write_text(json.dumps({
    "product": "Vertex Brain System",
    "version": version,
    "observation": "vSCOPE integrated",
    "connector": "Brain Membrane v1",
    "analysis_adapter": "LAA 0.1 / BLACK BOX ACTIVE",
    "resident_core": "vCELL Mount v1 / ENTHRONED",
    "resident_baseline": "VCELL_RESIDENT_RESPONSE_BASELINE_000001 / PRE-TRANSFER",
    "ui_typography": "BASE 10-12pt / SCALE OFFSET -3pt..+3pt / DEFAULT 0pt",
    "response_separability": "VCELL_4C_RESPONSE_SEPARABILITY_000001 / LLM LOCKED",
    "scale_expansion": "VCELL_32C_SCALE_EXPANSION_000001 / PROMOTION_SOURCE",
    "resident32": "VCELL_32C_RESIDENT_PROMOTION_000001 / ENTHRONED",
    "resident32_baseline": "VCELL_32C_RESIDENT_BASELINE_000001 / PRE-MEMORY",
    "persistent_difference": "VCELL_32C_PERSISTENT_DIFFERENCE_000001 / LONG-HORIZON",
    "ui": "NEURAL_OBSERVATORY_V3 / REAL 32C SPATIAL CORE / JA DEFAULT",
    "resident_observation": "PASSIVE_READ / CELL_ENERGY / SHELL_ENERGY / NO_INJECTION",
    "executable": str(versioned),
    "updated_unix": int(time.time())
}, indent=2), encoding="utf-8")
tmp.replace(current_json)

state = "UPDATED"
try:
    shutil.copy2(built, stable_alias)
except PermissionError:
    state = "LOCKED_DEFERRED"
except OSError as e:
    state = f"DEFERRED:{e.__class__.__name__}"

print(f"BRAIN_SYSTEM_VERSION_SLOT_READY {versioned}")
print(f"BRAIN_SYSTEM_CONVENIENCE_READY {convenience}")
print(f"BRAIN_SYSTEM_CURRENT_POINTER {current_json}")
print(f"BRAIN_SYSTEM_STABLE_ALIAS {state}")
print("FINAL: VERTEX BRAIN SYSTEM 0.9.1 / 32C PERSISTENT DIFFERENCE RESULT EGRESS 000001")
