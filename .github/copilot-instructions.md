This project is a cargo workspace that contains multiple creates.

- `./crates/beancount-formatter/` is the core formatting crate, it provide a function that take raw content, filename and config, then return the formatted result.
- `./crates/beancount-formatter-cli/` is a cli bin crate that users can use to format beancount files.
- `./crates/beancount-formatter-py/` is crate to build a python binding using pyo3, which export formatting library to python users.
- `./crates/dprint-plugin-beancount/` is a dprint plugin that compile the formatter into wasm.
