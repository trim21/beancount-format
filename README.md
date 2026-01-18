# beancount-format

Rust formatter for Beancount files with a reusable core library, a small CLI, and optional Python bindings.

## Usage

### cli/pypi package

- Format files in place (default): `beancount-format path/to/file.beancount`
- Check without modifying (exit 1 if reformat needed): `beancount-format --check path/to/file.beancount`

## Config

The CLI auto-detects `pyproject.toml` from the working directory or provided paths and merges settings with any CLI overrides.

```toml
[tool.beancount-format]
line-width = 70
indent-width = 2
new-line-kind = "lf"
```

- Can be overridden from args: `--line-width 80 --indent-width 4 --new-line lf`

### dprint

A [dprint](https://github.com/dprint/dprint) plugin is published in npm and can be fetched from jsDelivr directly:

```json
{
  "plugins": [
    "https://cdn.jsdelivr.net/npm/@trim21/dprint-plugin-beancount@latest/plugin.wasm"
  ]
}
```

## Crates

- crates/beancount-formatter: core library containing the formatter and configuration.
- crates/beancount-formatter-cli: CLI wrapper around the formatter library.
- crates/beancount-formatter-py: Python bindings built with PyO3/maturin.

## Development

- Build CLI: `cargo build -p beancount-formatter-cli --release`
- Run tests: `cargo test --workspace`
- Format code: `cargo fmt --all`
