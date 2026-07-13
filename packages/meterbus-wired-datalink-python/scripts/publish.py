"""Publish the maturin package using the core crate version."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

PACKAGE_ROOT = Path(__file__).resolve().parent.parent
VERSION_SCRIPT = PACKAGE_ROOT / "scripts/set_package_version.py"


def run(*command: str) -> None:
    subprocess.run(command, cwd=PACKAGE_ROOT, check=True)


def main() -> None:
    run(sys.executable, str(VERSION_SCRIPT))
    try:
        run("maturin", "publish")
    finally:
        run(sys.executable, str(VERSION_SCRIPT), "--reset")


if __name__ == "__main__":
    main()
