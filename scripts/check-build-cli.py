#!/usr/bin/env python3
"""Check the actual build script against the installed, pinned cargo-apk CLI.

This only requests help/version. It never builds an APK or needs the Android SDK.
"""
from pathlib import Path
import shlex
import subprocess

root = Path(__file__).resolve().parent.parent
script = (root / "scripts/build-android.sh").read_text(encoding="utf-8")
commands = []
for line in script.splitlines():
    line = line.strip()
    if line.startswith("cargo apk "):
        args = shlex.split(line)
        if args[2] == "build":
            args.append("--help")
        commands.append(args)
assert len(commands) == 2, "Expected exactly the build and version calls"
for args in commands:
    result = subprocess.run(args, cwd=root, text=True, capture_output=True)
    print("$ " + shlex.join(args))
    print(result.stdout, end="")
    print(result.stderr, end="")
    if result.returncode:
        raise SystemExit(result.returncode)
print("All cargo-apk invocations accept their arguments.")
