from __future__ import annotations

from pathlib import Path
import sys

root = Path(r"G:\Vertex_Project\Development\vertex_brain_system")
site = root / "site" / "vertex-brain-system-public"

checks = {
    "INDEX": (site / "index.html").is_file(),
    "STYLES": (site / "styles.css").is_file(),
    "APP": (site / "app.js").is_file(),
    "IMAGE": (site / "assets" / "vertex-brain-system-neural-observatory.png").is_file(),
}

index = (site / "index.html").read_text(encoding="utf-8") if checks["INDEX"] else ""
styles = (site / "styles.css").read_text(encoding="utf-8") if checks["STYLES"] else ""
app = (site / "app.js").read_text(encoding="utf-8") if checks["APP"] else ""

checks.update({
    "UI_SHOWCASE": 'class="ui-showcase"' in index,
    "IMAGE_LINK": "./assets/vertex-brain-system-neural-observatory.png" in index,
    "PRICE_200M": "2億円" in index and "JPY 200,000,000" in index,
    "FORMULA": "Π" in index and "O" in index and "Ψ" in index,
    "ACADEMIC_CAPTION": "観測写像" in index,
    "BILINGUAL_UI": "nav_system_ui" in app and "price_title" in app,
    "READABLE_UI_COPY": "font-size:18px" in styles,
})

all_ok = True
for name, ok in checks.items():
    print(f"{name}={'PASS' if ok else 'FAIL'}")
    all_ok &= ok

print("VERTEX_BRAIN_SYSTEM_PUBLIC_SITE_UI_HERO_000004_VERIFY=" + ("PASS" if all_ok else "FAIL"))
raise SystemExit(0 if all_ok else 1)
