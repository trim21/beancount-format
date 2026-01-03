# dprint-beancount

Python package wrapping the Rust `dprint-plugin-beancount` formatter.

## Install

```bash
pip install dprint-beancount
```

## Usage

```python
from dprint_beancount import format_text

source = "2010-01-01 open Assets:Cash\n"
formatted = format_text(source)
print(formatted)

# Override formatter options
formatted = format_text(
    source,
    path="ledger.beancount",  # defaults to "<memory>"
    line_width=80,
    indent_width=2,
    new_line_kind="lf",  # "lf" or "crlf"
)
print(formatted)
```
