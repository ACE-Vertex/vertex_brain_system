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
]

def verify_tree(root: Path, prefix: str) -> bool:
    ok = True
    print(f"{prefix}_ROOT={root}")
    for rel in EXPECTED:
        present = (root / rel).is_file()
        print(f"{prefix}_{rel.upper().replace('/', '_')}={'PASS' if present else 'FAIL'}")
        ok &= present

    if not ok:
        return False

    index = (root / "index.html").read_text(encoding="utf-8")
    styles = (root / "styles.css").read_text(encoding="utf-8")
    app = (root / "app.js").read_text(encoding="utf-8")
    php = (root / "counter.php").read_text(encoding="utf-8")

    checks = {
        "BRAIN_BRAND": "VERTEX BRAIN SYSTEM" in index,
        "JAPANESE_DEFAULT": 'body data-lang="ja"' in index,
        "LANG_JA": 'data-set-lang="ja"' in index,
        "LANG_EN": 'data-set-lang="en"' in index,
        "ACADEMIC_STATE_VECTOR": "x_t ∈ Ω_R" in index,
        "ACADEMIC_BOUNDARY": "∂M" in index,
        "ACADEMIC_OBSERVATION_MAP": "O_v : Ω_R → E" in index,
        "HYPOTHESES": all(token in index for token in ("H1", "H2", "H3", "H4")),
        "ENGLISH_TRANSLATION": "practical theory of observability" in app,
        "READABLE_TYPOGRAPHY": "font-size:18px" in styles and "font-size:17px" in styles,
        "COUNTER_FETCH": "counter.php" in app,
        "COUNTER_KEY": "vertex-brain-system" in php,
        "BLUE_THEME": "#56d9ff" in styles or "#60daff" in styles,
    }
    for name, passed in checks.items():
        print(f"{prefix}_{name}={'PASS' if passed else 'FAIL'}")
        ok &= passed
    return ok

parser = argparse.ArgumentParser()
parser.add_argument("--source-root", required=True)
parser.add_argument("--deploy-root")
args = parser.parse_args()

source = Path(args.source_root)
all_ok = verify_tree(source, "SOURCE")

if args.deploy_root:
    deploy = Path(args.deploy_root)
    all_ok &= verify_tree(deploy, "DEPLOY")
    if all_ok:
        for rel in EXPECTED:
            same = (source / rel).read_bytes() == (deploy / rel).read_bytes()
            print(f"MATCH_{rel.upper().replace('/', '_')}={'PASS' if same else 'FAIL'}")
            all_ok &= same

print("VERTEX_BRAIN_SYSTEM_PUBLIC_SITE_000003_VERIFY=" + ("PASS" if all_ok else "FAIL"))
raise SystemExit(0 if all_ok else 1)
