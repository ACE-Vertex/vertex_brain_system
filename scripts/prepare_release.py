from pathlib import Path
root = Path.cwd() / "vertex_brain_system"
built = root / "src-tauri" / "target" / "release" / "vertex-brain-system.exe"
if built.exists():
    built.unlink()
    print(f"BRAIN_SYSTEM_RELEASE_PREP_REMOVED {built}")
else:
    print(f"BRAIN_SYSTEM_RELEASE_PREP_CLEAN {built}")
