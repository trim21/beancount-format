# AGENTS

This file defines repository-specific rules and expectations for automated agents and contributors.

## Repository layout

This is a Cargo workspace with multiple crates and language bindings:

- `crates/beancount-formatter/`: core formatting crate; exposes the main formatting function that takes raw content and config, and returns formatted output.
- `crates/beancount-formatter-cli/`: CLI binary/library that formats `.beancount` and `.bean` files for end users.
- `crates/beancount-formatter-py/`: Python binding crate built with PyO3/maturin that exports formatting APIs to Python users.
- `crates/dprint-plugin-beancount/`: dprint plugin crate that compiles the formatter to WASM.
- `python/src/beancount_format/`: Python package source and type stubs.
- `scripts/`: release/support scripts (including plugin asset generation).

## Ownership and API boundaries

- Keep formatting behavior in `beancount-formatter`.
- Keep argument/config loading behavior in `beancount-formatter-cli`.
- Keep Python-facing signature changes in sync between:
	- `crates/beancount-formatter-py/src/lib.rs`
	- `python/src/beancount_format/beancount_format.pyi`
- Keep dprint-specific mapping logic inside `crates/dprint-plugin-beancount`.

## dprint plugin configuration contract

### Plugin config vs global config

The dprint plugin reads these keys from plugin-specific config (snake_case):

- `line_width`
- `indent_width`
- `new_line`
- `compact_balance_spacing`

When any of these plugin keys are omitted, values fall back to dprint global configuration and then formatter defaults.

Global dprint keys that provide fallback values are:

- `line_width`
- `indent_width`
- `new_line`

The corresponding core formatter fields remain:

- `line_width`
- `indent_width`
- `new_line`

### Schema rule

Generated dprint plugin schema must include all plugin-readable keys and use snake_case naming.

For current behavior, `scripts/generate_plugin_assets.py` must emit schema properties containing:

- `line_width`
- `indent_width`
- `new_line`
- `compact_balance_spacing`

If plugin config behavior changes, update both runtime mapping and schema generation in the same PR.

## Formatting option naming conventions

- Core Rust config uses `snake_case` fields.
- CLI/`pyproject.toml` user options use `kebab-case`.
- dprint plugin config uses `snake_case`.

When introducing a new option, keep naming consistent across all integration points.

## Required sync points when adding/changing options

When adding a formatter option, update all applicable surfaces:

1. Core config struct/defaults (`beancount-formatter`).
2. Formatting logic (`beancount-formatter`).
3. CLI args and pyproject parser (`beancount-formatter-cli`).
4. Python binding signature and `.pyi` stub.
5. dprint plugin mapping and schema generator.
6. Docs in `README.md` and `python/README.md` when user-facing.
7. Tests/fixtures for behavior and config parsing.

## Testing guidance

Preferred commands:

- Core formatter fixtures: `cargo test -p beancount-formatter format_and_check_fixtures`
- CLI config parsing: `cargo test -p beancount-formatter-cli --lib`
- CLI e2e: `cargo test -p beancount-formatter-cli --test cli_e2e`
- dprint plugin crate: `cargo test -p dprint-plugin-beancount`
- Python crate compile/tests: `cargo test -p beancount-formatter-py`

Notes:

- Fixture expected files can be refreshed with `TEST_UPDATE_EXPECTED=1` for fixture-based tests.
- If plugin schema changes, run:
	- `uv run python scripts/generate_plugin_assets.py --out-dir dist`

## Release-sensitive files

Changes in these files usually affect distribution artifacts and should be reviewed carefully:

- `scripts/generate_plugin_assets.py`
- `.github/workflows/release.yaml`
- `plugin.wasm`
- `package.json`, `pyproject.toml`, and workspace version in `Cargo.toml`

## Change discipline

- Keep edits minimal and scoped to the requested behavior.
- Do not introduce config keys in schema that runtime does not read.
- Do not read plugin config keys that schema does not document.
- Preserve backward compatibility by default unless explicitly requested.
