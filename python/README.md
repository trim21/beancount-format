# beancount-format

beancount formatter.

## Install

```bash
pip install beancount-format
```

## Usage

### as CLI

```bash
beancount-format ./
```

### as library

```python
from bean_format import format_text

source = "2010-01-01 open Assets:Cash\n"
formatted = format_text(source)
print(formatted)

# Override formatter options
formatted = format_text(
    source,
    path="ledger.beancount",  # defaults to "<memory>"
    line_width=88,
    indent_width=2,
    new_line="lf",  # "lf" or "crlf"
)
print(formatted)
```
