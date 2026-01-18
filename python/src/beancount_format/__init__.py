"""Python bindings for the beancount formatter."""

from .beancount_format import format_text, main as _main  # type: ignore

__all__ = ["format_text", "main"]


def main(argv=None) -> int:
    """Run the CLI via the Rust backend.

    Returns an exit code (0 when everything is formatted, 1 when changes are needed).
    """

    import sys

    args = sys.argv if argv is None else [sys.argv[0], *argv]
    changed = _main(list(args))
    return 1 if changed else 0
