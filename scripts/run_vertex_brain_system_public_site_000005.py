from __future__ import annotations

from pathlib import Path
import subprocess

BRAIN_ROOT = Path(r"G:\Vertex_Project\Development\vertex_brain_system")
VERTEX_STUDIO_ROOT = Path(r"G:\Vertex_Project\Development\vertex_studio_ai")
BUILDER = BRAIN_ROOT / "scripts" / "build_vertex_brain_system_public_site_000001.ps1"

if not BUILDER.exists():
    print(f"BUILDER_EXISTS=FAIL {BUILDER}")
    raise SystemExit(1)

print(f"BUILDER_EXISTS=PASS {BUILDER}")
print("RUNNER_MODE=POWERSHELL_BUILDER")
print("RUNNER_DEPLOYMENT=VERTEX_HUB_PUBLIC_ROOT")
print("RUNNER_SITE_MODE=BILINGUAL_ACADEMIC_UI_HERO_200M")
print("RUNNER_TARGET=https://vertex.a-portal.net/vertex-brain-system/")
print("RUNNER_GIT_POLICY=VERIFY_DEPLOY_THEN_COMMIT_PUSH")

cmd = [
    "pwsh",
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    str(BUILDER),
    "-BrainRoot",
    str(BRAIN_ROOT),
    "-VertexStudioRoot",
    str(VERTEX_STUDIO_ROOT),
    "-HostName",
    "vertex.a-portal.net",
    "-Route",
    "vertex-brain-system",
]

proc = subprocess.run(cmd, cwd=str(BRAIN_ROOT))
print(f"BUILDER_EXIT_CODE={proc.returncode}")

if proc.returncode != 0:
    print("VERTEX_BRAIN_SYSTEM_PUBLIC_SITE_000005=FAIL")
    raise SystemExit(proc.returncode)

print("VERTEX_BRAIN_SYSTEM_PUBLIC_SITE_000005=PASS")
