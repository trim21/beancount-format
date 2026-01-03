"""Python bindings for the beancount formatter."""

from .bean_format import format_text  # type: ignore

__all__ = ["format_text"]
