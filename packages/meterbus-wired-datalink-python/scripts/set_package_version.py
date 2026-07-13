"""Set the binding crate version from the core crate, or reset it to 0.0.0."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

PACKAGE_ROOT = Path(__file__).resolve().parent.parent
CORE_MANIFEST = PACKAGE_ROOT.parents[1] / "crates/meterbus-wired-datalink/Cargo.toml"
BINDING_MANIFEST = (
    PACKAGE_ROOT.parents[1] / "crates/meterbus-wired-datalink-python/Cargo.toml"
)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reset", action="store_true")
    args = parser.parse_args()

    core = CORE_MANIFEST.read_text()
    binding = BINDING_MANIFEST.read_text()
    match = re.search(r'^version\s*=\s*"([^"]+)"', core, flags=re.MULTILINE)
    if match is None:
        raise RuntimeError("could not read the meterbus-wired-datalink crate version")

    version = "0.0.0" if args.reset else match.group(1)
    updated, count = re.subn(
        r'^(version\s*=\s*)"[^"]+"',
        rf'\g<1>"{version}"',
        binding,
        count=1,
        flags=re.MULTILINE,
    )
    if count != 1 or updated == binding:
        raise RuntimeError("could not update the binding crate version")

    BINDING_MANIFEST.write_text(updated)
    print(f"set package version to {version}")


if __name__ == "__main__":
    main()
