"""Python bindings for the beancount formatter."""

import sys
from .beancount_format import format_text, main as _main  # type: ignore

__all__ = ["format_text", "main"]


def main(argv: list[str] | None = None) -> int:
    """Run the CLI via the Rust backend.

    Returns an exit code (0 when everything is formatted, 1 when changes are needed).
    """

    return _main(list(argv or sys.argv))
