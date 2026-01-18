# beancount-format

Rust formatter for Beancount files with a reusable core library, a small CLI, and optional Python bindings.

## NPM / jsDelivr
The published package includes plugin.wasm for browser/CDN usage.

- Package: @trim21/dprint-plugin-beancount
- jsDelivr (latest): https://cdn.jsdelivr.net/npm/@trim21/dprint-plugin-beancount@latest/plugin.wasm
- Node usage: require the package and read `wasmPath` or call `getWasmBuffer()`.

## Crates
- crates/beancount-formatter: core library containing the formatter and configuration.
- crates/beancount-formatter-cli: CLI wrapper around the formatter library.
- crates/beancount-formatter-py: Python bindings built with PyO3/maturin.

## CLI usage
- Format files in place (default): `beancount-format path/to/file.beancount`
- Check without modifying (exit 1 if reformat needed): `beancount-format --check path/to/file.beancount`
- Override config: `--line-width 80 --indent-width 4 --new-line lf --prefix-width 60 --num-width 20`

The CLI auto-detects `pyproject.toml` from the working directory or provided paths and merges settings with any CLI overrides.

## Development
- Build CLI: `cargo build -p beancount-formatter-cli --release`
- Run tests: `cargo test --workspace`
- Format code: `cargo fmt --all`
