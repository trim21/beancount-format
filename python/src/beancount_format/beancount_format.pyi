from typing import Optional, Sequence

__all__ = ["format_text", "main"]

def format_text(
    text: str,
    *,
    path: Optional[str] = ...,
    line_width: Optional[int] = ...,
    indent_width: Optional[int] = ...,
    new_line: Optional[str] = ...,
) -> str: ...
def main(argv: Sequence[str]) -> int: ...
