#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
import tomllib as toml
from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict, Field

REPO = "trim21/beancount-format"
PLUGIN_DIR = "dprint-plugin-beancount"


class NewLineKind(str, Enum):
    LF = "lf"
    CRLF = "crlf"


class DprintPluginSchema(BaseModel):
    model_config = ConfigDict(
        extra="forbid",
        title="beancount dprint plugin config",
    )

    line_width: int | None = Field(
        default=None,
        ge=1,
        description="Override maximum line width. Falls back to dprint global `line_width`.",
    )
    indent_width: int | None = Field(
        default=None,
        ge=1,
        description="Override indentation width. Falls back to dprint global `indent_width`.",
    )
    new_line: NewLineKind | None = Field(
        default=None,
        description="Override newline style. Falls back to dprint global `new_line`.",
    )
    compact_balance_spacing: bool = Field(
        default=False,
        description="When true, removes empty lines between consecutive balance directives.",
    )


def build_schema() -> dict[str, Any]:
    schema = DprintPluginSchema.model_json_schema()
    schema["$schema"] = "http://json-schema.org/draft-07/schema#"

    return schema


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

    schema = build_schema()

    write_json(version_dir / "schema.json", schema)

    latest: dict[str, Any] = {
        "schemaVersion": 1,
        "version": version,
        "url": f"https://cdn.jsdelivr.net/npm/@trim21/dprint-plugin-beancount@{version}/plugin.wasm",
    }

    write_json(plugin_root / "latest.json", latest)


if __name__ == "__main__":
    main()
