"""Check a release tag against Cargo.toml and emit GitHub Actions outputs."""

import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def check_version(tag):
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version=1"],
        cwd=ROOT, text=True,
    ))
    package = next(package for package in metadata["packages"] if package["name"] == "ss")
    version = package["version"]
    if tag.removeprefix("v") != version:
        raise SystemExit(f"Release tag {tag!r} does not match Cargo.toml version {version!r}")
    print(f"version={version}")
    print(f"ref=refs/tags/{tag}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag")
    args = parser.parse_args()
    check_version(args.tag)


if __name__ == "__main__":
    main()
