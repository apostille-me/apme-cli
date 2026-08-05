#!/usr/bin/env python3
from __future__ import annotations

import pathlib
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
EXPECTED = {
    "apostille-me/apme-interfaces",
    "apostille-me/apme-libs",
    "apostille-me/apme-clients",
}


def load(path: pathlib.Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def main() -> int:
    errors: list[str] = []
    manifest = load(ROOT / ".zpkg.toml")
    lock = load(ROOT / ".zpkg.lock")
    package = manifest.get("package", {})
    dependencies = manifest.get("dependencies", {})
    if package.get("org") != "apostille-me" or package.get("name") != "apme-cli":
        errors.append("package identity must be apostille-me/apme-cli")
    if package.get("repository", {}).get("url") != "https://github.com/apostille-me/apme-cli":
        errors.append("package.repository.url must match the canonical repository")
    if not isinstance(dependencies, dict) or not EXPECTED.issubset(dependencies):
        errors.append("apme-cli must depend on interfaces, libs, and clients")
        dependencies = dependencies if isinstance(dependencies, dict) else {}
    for dependency in dependencies:
        if dependency.rsplit("/", 1)[-1].endswith("-infra"):
            errors.append(f"CLI may not import infrastructure: {dependency}")
    if lock.get("version") != 1:
        errors.append(".zpkg.lock must use version = 1")
    if manifest.get("targets", {}).get("repository", {}).get("dir") != ".":
        errors.append("[targets.repository] must publish the repository root")
    for error in errors:
        print(f"error: {error}", file=sys.stderr)
    if errors:
        return 1
    print("validated apostille-me/apme-cli dependency graph")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
