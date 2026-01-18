"""Python bindings for the beancount formatter."""

import sys

from .beancount_format import (  # pyright: ignore[reportMissingModuleSource]
    format_text,
    main as _main,
)

__all__ = ["format_text", "main"]


def main(argv: list[str] | None = None) -> None:
    """Run the CLI via the Rust backend.

    Returns an exit code (0 when everything is formatted, 1 when changes are needed).
    """

    sys.exit(_main(list(argv or sys.argv)))
