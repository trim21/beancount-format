from collections.abc import Sequence

__all__ = ["format_text", "main"]

def format_text(
    text: str,
    *,
    line_width: int | None = ...,
    indent_width: int | None = ...,
    new_line: str | None = ...,
    compact_balance_spacing: bool | None = ...,
) -> str: ...
def main(argv: Sequence[str]) -> int: ...
