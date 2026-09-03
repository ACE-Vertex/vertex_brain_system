from __future__ import annotations

from pathlib import Path
import argparse
import sys

EXPECTED = [
    "index.html",
    "styles.css",
    "app.js",
    "counter.php",
    "data/.gitignore",
    "data/README.txt",
    "assets/vertex-brain-system-neural-observatory.png",
]

def verify_tree(root: Path, prefix: str) -> bool:
    ok = True
    print(f"{prefix}_ROOT={root}")

    for rel in EXPECTED:
        present = (root / rel).is_file()
        print(f"{prefix}_{rel.upper().replace('/', '_').replace('.', '_')}={'PASS' if present else 'FAIL'}")
        ok &= present

    if not ok:
        return False

    index = (root / "index.html").read_text(encoding="utf-8")
    styles = (root / "styles.css").read_text(encoding="utf-8")
    app = (root / "app.js").read_text(encoding="utf-8")
    php = (root / "counter.php").read_text(encoding="utf-8")

    checks = {
        "BRAIN_BRAND": "VERTEX BRAIN SYSTEM" in index,
        "UI_SHOWCASE": 'class="ui-showcase"' in index,
        "UI_IMAGE_LINK": "./assets/vertex-brain-system-neural-observatory.png" in index,
        "PRICE_200M_JA": "2億円" in index,
        "PRICE_200M_EN": "JPY 200,000,000" in index,
        "FORMULA_PSI": "Ψ" in index,
        "FORMULA_PI": "Π" in index,
        "FORMULA_OBSERVATION": "O" in index and "観測写像" in index,
        "LANG_JA": 'data-set-lang="ja"' in index,
        "LANG_EN": 'data-set-lang="en"' in index,
        "BILINGUAL_COPY": "Reference Price JPY 200,000,000" in app,
        "ACADEMIC_COPY": "practical theory of observability" in app,
        "READABLE_TYPE": "font-size:18px" in styles,
        "COUNTER_FETCH": "counter.php" in app,
        "COUNTER_KEY": "vertex-brain-system" in php,
    }

    for name, passed in checks.items():
        print(f"{prefix}_{name}={'PASS' if passed else 'FAIL'}")
        ok &= passed

    return ok

parser = argparse.ArgumentParser()
parser.add_argument("--source-root", required=True)
parser.add_argument("--deploy-root", required=True)
args = parser.parse_args()

source = Path(args.source_root)
deploy = Path(args.deploy_root)

all_ok = verify_tree(source, "SOURCE")
all_ok &= verify_tree(deploy, "DEPLOY")

if all_ok:
    for rel in EXPECTED:
        same = (source / rel).read_bytes() == (deploy / rel).read_bytes()
        print(f"MATCH_{rel.upper().replace('/', '_').replace('.', '_')}={'PASS' if same else 'FAIL'}")
        all_ok &= same

print("VERTEX_BRAIN_SYSTEM_PUBLIC_SITE_DEPLOY_000005_VERIFY=" + ("PASS" if all_ok else "FAIL"))
raise SystemExit(0 if all_ok else 1)
