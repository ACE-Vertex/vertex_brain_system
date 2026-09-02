from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT = Path(r"G:\Vertex_Project\Development\vertex_brain_system")
REMOTE = "https://github.com/ACE-Vertex/vertex_brain_system.git"
BRANCH = "main"

BLOCK_BEGIN = "# >>> VERTEX SOURCE REPOSITORY SAFETY >>>"
BLOCK_END = "# <<< VERTEX SOURCE REPOSITORY SAFETY <<<"

IGNORE_BLOCK = r"""
# >>> VERTEX SOURCE REPOSITORY SAFETY >>>
# Build/cache output
target/
**/target/
node_modules/
**/node_modules/
dist/
build/
.vite/
.tauri/
.cache/
**/.cache/

# Generated/runtime output
versions/
MIGRATION_BACKUPS/
runtime/
logs/
*.pending
*.log
*.tmp
*.bak

# Local model weights / large AI artifacts
models/
**/models/
*.gguf
*.safetensors
*.onnx
*.pt
*.pth

# Native release binaries
*.exe
*.dll
*.pdb
*.msi
*.msix
*.appx
*.dmg
*.pkg

# Secrets / credentials
.env
.env.*
!.env.example
!.env.sample
*.pem
*.key
*.p12
*.pfx
id_rsa
id_ed25519
credentials.json
secrets.json

# OS noise
.DS_Store
Thumbs.db
# <<< VERTEX SOURCE REPOSITORY SAFETY <<<
""".strip() + "\n"

STRONG_SECRET_PATTERNS = [
    re.compile(rb"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    re.compile(rb"\bsk-[A-Za-z0-9_-]{20,}\b"),
    re.compile(rb"\bghp_[A-Za-z0-9]{20,}\b"),
    re.compile(rb"\bgithub_pat_[A-Za-z0-9_]{20,}\b"),
    re.compile(rb"\bAKIA[0-9A-Z]{16}\b"),
]

BLOCKED_NAMES = {
    ".env", "credentials.json", "secrets.json",
    "id_rsa", "id_ed25519",
}
BLOCKED_SUFFIXES = {
    ".pem", ".key", ".p12", ".pfx",
    ".exe", ".dll", ".pdb", ".msi", ".msix", ".appx",
    ".dmg", ".pkg", ".zip", ".7z", ".rar",
    ".gguf", ".safetensors", ".onnx", ".pt", ".pth",
}
MAX_BYTES = 95 * 1024 * 1024

def run(args, check=True):
    result = subprocess.run(
        [str(a) for a in args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.stdout.strip():
        print(result.stdout.rstrip())
    if result.stderr.strip():
        print(result.stderr.rstrip())
    if check and result.returncode != 0:
        raise SystemExit(result.returncode)
    return result

def ensure_ignore():
    path = ROOT / ".gitignore"
    existing = path.read_text(encoding="utf-8", errors="replace") if path.exists() else ""
    if BLOCK_BEGIN in existing and BLOCK_END in existing:
        print("GITIGNORE_VERTEX_BLOCK=ALREADY_PRESENT")
        return
    if existing and not existing.endswith("\n"):
        existing += "\n"
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        handle.write(existing)
        if existing:
            handle.write("\n")
        handle.write(IGNORE_BLOCK)
    print("GITIGNORE_VERTEX_BLOCK=ADDED")

def ensure_git():
    if shutil.which("git") is None:
        print("GIT_NOT_FOUND")
        raise SystemExit(3)

    if not (ROOT / ".git").exists():
        run(["git", "init", "-b", BRANCH])
        print("GIT_INIT=PASS")
    else:
        print("GIT_INIT=EXISTING")
        run(["git", "branch", "-M", BRANCH])

    remotes = run(["git", "remote"]).stdout.split()
    if "origin" in remotes:
        run(["git", "remote", "set-url", "origin", REMOTE])
        print("REMOTE_ORIGIN=UPDATED")
    else:
        run(["git", "remote", "add", "origin", REMOTE])
        print("REMOTE_ORIGIN=ADDED")

    if not run(["git", "config", "--get", "user.name"], check=False).stdout.strip():
        run(["git", "config", "user.name", "ACE-Vertex"])
        print("GIT_USER_NAME=LOCAL_FALLBACK")
    if not run(["git", "config", "--get", "user.email"], check=False).stdout.strip():
        run(["git", "config", "user.email", "ACE-Vertex@users.noreply.github.com"])
        print("GIT_USER_EMAIL=LOCAL_FALLBACK")

def staged_paths():
    raw = run(["git", "diff", "--cached", "--name-only", "-z"]).stdout
    return [p for p in raw.split("\0") if p]

def safety_scan(paths):
    problems = []
    for rel in paths:
        p = ROOT / rel
        if not p.exists() or not p.is_file():
            continue
        name = p.name.lower()
        suffix = p.suffix.lower()

        if name in BLOCKED_NAMES:
            problems.append(f"SENSITIVE_PATH {rel}")
            continue
        if suffix in BLOCKED_SUFFIXES:
            problems.append(f"BLOCKED_BINARY_MODEL_OR_SECRET {rel}")
            continue

        size = p.stat().st_size
        if size > MAX_BYTES:
            problems.append(f"TOO_LARGE {size} {rel}")
            continue

        if size <= 4 * 1024 * 1024:
            try:
                data = p.read_bytes()
            except OSError:
                continue
            if any(pattern.search(data) for pattern in STRONG_SECRET_PATTERNS):
                problems.append(f"STRONG_SECRET_PATTERN {rel}")

    if problems:
        print("SOURCE_SAFETY_SCAN=BLOCKED")
        for item in problems:
            print(item)
        raise SystemExit(17)
    print(f"SOURCE_SAFETY_SCAN=PASS ({len(paths)} staged paths)")

def remote_head():
    result = run(["git", "ls-remote", "origin", f"refs/heads/{BRANCH}"], check=False)
    if result.returncode != 0:
        raise SystemExit(result.returncode)
    line = result.stdout.strip()
    return line.split()[0] if line else ""

def commit_source():
    run(["git", "add", "-A"])
    files = staged_paths()
    safety_scan(files)

    status = run(["git", "status", "--short"]).stdout.strip()
    if not status:
        print("GIT_COMMIT=NO_CHANGES")
        return

    commit = run(
        ["git", "commit", "-m", "Publish Vertex Brain System source"],
        check=False,
    )
    if commit.returncode != 0:
        raise SystemExit(commit.returncode)
    print("GIT_COMMIT=PASS")

def push_source():
    remote = remote_head()
    local = run(["git", "rev-parse", "HEAD"]).stdout.strip()

    print(f"LOCAL_HEAD_BEFORE_PUSH={local}")
    print(f"REMOTE_HEAD_BEFORE_PUSH={remote or 'ABSENT'}")

    if remote == local:
        print("REMOTE_ALREADY_CURRENT=PASS")
        return

    # This repository was created specifically for this source publication.
    # If it is no longer empty, do not guess and do not overwrite history.
    if remote:
        print("REMOTE_MAIN_NOT_EMPTY=BLOCK")
        print("BrainSystem publisher will not force-push an unexpected remote history.")
        raise SystemExit(23)

    run(["git", "push", "-u", "origin", BRANCH])
    print("GIT_PUSH=PASS")

def verify_remote():
    local = run(["git", "rev-parse", "HEAD"]).stdout.strip()
    remote = remote_head()
    print(f"LOCAL_HEAD={local}")
    print(f"REMOTE_HEAD={remote}")
    ok = bool(local and remote and local == remote)
    print(f"HEAD_MATCH={'PASS' if ok else 'FAIL'}")
    if not ok:
        raise SystemExit(29)
    print("VERTEX_BRAIN_SYSTEM_GITHUB_PUBLISH_000001=PASS")

def main():
    if not ROOT.exists():
        print(f"ROOT_MISSING={ROOT}")
        raise SystemExit(2)

    ensure_ignore()
    ensure_git()
    commit_source()
    push_source()
    verify_remote()

if __name__ == "__main__":
    main()
