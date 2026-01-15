#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path
import tomllib as toml


def read_workspace_version(cargo_toml: Path) -> str:
    text = cargo_toml.read_bytes().decode("utf-8")
    data = toml.loads(text)
    try:
        return data["workspace"]["package"]["version"]
    except KeyError as exc:
        raise RuntimeError("Missing version in [workspace.package] section") from exc


def _validate_pyproject(text: str, expected_version: str) -> bool:
    data = toml.loads(text)
    return data.get("project", {}).get("version") == expected_version


def update_pyproject(pyproject: Path, version: str) -> bool:
    text = pyproject.read_bytes().decode("utf-8")
    try:
        old_version = toml.loads(text).get("project", {}).get("version")
    except Exception as exc:  # pragma: no cover - invalid TOML
        raise RuntimeError("pyproject.toml is not valid TOML") from exc

    if not old_version:
        raise RuntimeError("Missing version field in [project] section")
    old_version = str(old_version)
    parts = text.split(old_version)
    if len(parts) < 2:
        raise RuntimeError("Failed to find version string in pyproject.toml")

    candidate = version.join([parts[0], old_version.join(parts[1:])])
    if _validate_pyproject(candidate, version):
        if candidate != text:
            pyproject.write_bytes(candidate.encode("utf-8"))
            return True
        return False

    head, tail = text.rsplit(old_version, 1)
    candidate = head + version + tail
    if _validate_pyproject(candidate, version):
        if candidate != text:
            pyproject.write_bytes(candidate.encode("utf-8"))
            return True
        return False

    candidate = version.join(parts)
    if _validate_pyproject(candidate, version):
        if candidate != text:
            pyproject.write_bytes(candidate.encode("utf-8"))
            return True
        return False

    raise RuntimeError("Failed to update pyproject.toml version safely")


def _validate_package_json(text: str, expected_version: str) -> bool:
    data = json.loads(text)
    return data.get("version") == expected_version


def update_package_json(package_json: Path, version: str) -> bool:
    text = package_json.read_bytes().decode("utf-8")
    try:
        old_version = json.loads(text).get("version")
    except json.JSONDecodeError as exc:
        raise RuntimeError("package.json is not valid JSON") from exc

    if not old_version:
        raise RuntimeError("Missing version field in package.json")
    old_version = str(old_version)
    parts = text.split(old_version)
    if len(parts) < 2:
        raise RuntimeError("Failed to find version string in package.json")

    candidate = version.join([parts[0], old_version.join(parts[1:])])
    if _validate_package_json(candidate, version):
        if candidate != text:
            package_json.write_bytes(candidate.encode("utf-8"))
            return True
        return False

    head, tail = text.rsplit(old_version, 1)
    candidate = head + version + tail
    if _validate_package_json(candidate, version):
        if candidate != text:
            package_json.write_bytes(candidate.encode("utf-8"))
            return True
        return False

    candidate = version.join(parts)
    if _validate_package_json(candidate, version):
        if candidate != text:
            package_json.write_bytes(candidate.encode("utf-8"))
            return True
        return False

    raise RuntimeError("Failed to update package.json version safely")


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    cargo_toml = repo_root / "Cargo.toml"
    pyproject = repo_root / "pyproject.toml"
    package_json = repo_root / "package.json"

    version = read_workspace_version(cargo_toml)

    pyproject_updated = update_pyproject(pyproject, version)
    package_updated = update_package_json(package_json, version)

    updated_files = []
    if pyproject_updated:
        updated_files.append("pyproject.toml")
    if package_updated:
        updated_files.append("package.json")

    if updated_files:
        print(f"Updated to version {version}: {', '.join(updated_files)}")
    else:
        print(f"No changes needed. Version is already {version}.")


if __name__ == "__main__":
    main()
