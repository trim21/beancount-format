# beancount-format

Rust formatter for Beancount files with a reusable core library, a small CLI, and optional Python bindings.

## Crates
- crates/beancount-formatter: core library containing the formatter and configuration.
- crates/beancount-formatter-cli: CLI wrapper around the formatter library.
- crates/beancount-formatter-py: Python bindings built with PyO3/maturin.

## Development
- Build CLI: `cargo build -p beancount-formatter-cli --release`
- Run tests: `cargo test --workspace`
- Format code: `cargo fmt --all`
