#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
import tomllib as toml
from typing import Any

REPO = "trim21/beancount-format"
PLUGIN_DIR = "dprint-plugin-beancount"


def read_workspace_version(cargo_toml: Path) -> str:
    text = cargo_toml.read_bytes().decode("utf-8")
    data = toml.loads(text)
    try:
        return str(data["workspace"]["package"]["version"])
    except KeyError as exc:
        raise RuntimeError("Missing version in [workspace.package] section") from exc


def write_json(path: Path, data: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(data, indent=2, ensure_ascii=False)
    path.write_text(payload + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate dprint plugin assets.")
    parser.add_argument("--out-dir", type=Path, default=None)
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    out_dir = args.out_dir or (repo_root / "dist")

    version = read_workspace_version(repo_root / "Cargo.toml")
    plugin_root = out_dir / PLUGIN_DIR
    version_dir = plugin_root / version

    schema: dict[str, Any] = {}

    write_json(version_dir / "schema.json", schema)

    latest: dict[str, Any] = {
        "schemaVersion": 1,
        "version": version,
        "url": f"https://cdn.jsdelivr.net/gh/{REPO}@gh-pages/{PLUGIN_DIR}/{version}/plugin.wasm",
    }

    write_json(plugin_root / "latest.json", latest)

    del args


if __name__ == "__main__":
    main()
