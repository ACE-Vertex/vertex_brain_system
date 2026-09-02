from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(r"G:\Vertex_Project\Development\vertex_brain_system")
REMOTE = "https://github.com/ACE-Vertex/vertex_brain_system.git"
BRANCH = "main"

EXPECTED_REMOTE_INITIAL = "a24d11cd557709411230e62a22edcf4ebfe1dd7c"

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

def ensure_remote():
    remotes = run(["git", "remote"]).stdout.split()
    if "origin" in remotes:
        run(["git", "remote", "set-url", "origin", REMOTE])
    else:
        run(["git", "remote", "add", "origin", REMOTE])
    run(["git", "branch", "-M", BRANCH])
    print("REMOTE_ORIGIN_ACE_VERTEX=PASS")

def remote_head():
    result = run(["git", "ls-remote", "origin", f"refs/heads/{BRANCH}"], check=False)
    if result.returncode != 0:
        raise SystemExit(result.returncode)
    line = result.stdout.strip()
    return line.split()[0] if line else ""

def tracked_paths():
    raw = run(["git", "ls-files", "-z"]).stdout
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

    print(f"SOURCE_SAFETY_SCAN=PASS ({len(paths)} tracked paths)")

def commit_hotfix_files_if_needed():
    run(["git", "add", "-A"])
    status = run(["git", "status", "--short"]).stdout.strip()
    if not status:
        print("HOTFIX_COMMIT=NO_CHANGES")
        return

    commit = run(
        ["git", "commit", "-m", "Add guarded ACE-Vertex GitHub publish hotfix"],
        check=False,
    )
    if commit.returncode != 0:
        raise SystemExit(commit.returncode)
    print("HOTFIX_COMMIT=PASS")

def publish():
    local = run(["git", "rev-parse", "HEAD"]).stdout.strip()
    remote = remote_head()

    print(f"LOCAL_HEAD_BEFORE_PUSH={local}")
    print(f"REMOTE_HEAD_BEFORE_PUSH={remote or 'ABSENT'}")

    if remote == local:
        print("REMOTE_ALREADY_CURRENT=PASS")
        return

    if not remote:
        run(["git", "push", "-u", "origin", BRANCH])
        print("NORMAL_PUSH=PASS")
        return

    if remote != EXPECTED_REMOTE_INITIAL:
        print("REMOTE_HEAD_CHANGED_UNEXPECTEDLY=BLOCK")
        print(f"EXPECTED_INITIAL={EXPECTED_REMOTE_INITIAL}")
        print(f"ACTUAL_REMOTE={remote}")
        raise SystemExit(23)

    lease = f"refs/heads/{BRANCH}:{EXPECTED_REMOTE_INITIAL}"
    run([
        "git", "push",
        "--force-with-lease=" + lease,
        "-u", "origin", BRANCH
    ])
    print("FORCE_WITH_LEASE_INITIAL_COMMIT_REPLACEMENT=PASS")

def verify_remote():
    local = run(["git", "rev-parse", "HEAD"]).stdout.strip()
    remote = remote_head()

    print(f"LOCAL_HEAD={local}")
    print(f"REMOTE_HEAD={remote}")
    ok = bool(local and remote and local == remote)
    print(f"HEAD_MATCH={'PASS' if ok else 'FAIL'}")
    if not ok:
        raise SystemExit(29)

    print("ACE_VERTEX_NAMESPACE=PASS")
    print("VERTEX_BRAIN_SYSTEM_GITHUB_PUBLISH_HOTFIX_000002=PASS")

def main():
    if not ROOT.exists() or not (ROOT / ".git").exists():
        print("LOCAL_GIT_REPOSITORY_MISSING")
        raise SystemExit(2)

    ensure_remote()
    commit_hotfix_files_if_needed()
    safety_scan(tracked_paths())
    publish()
    verify_remote()

if __name__ == "__main__":
    main()
