"""Python bindings for the beancount formatter."""

from .beancount_format import format_text  # type: ignore

__all__ = ["format_text"]
