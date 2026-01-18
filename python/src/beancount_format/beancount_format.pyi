from typing import Optional

__all__ = ["format_text"]

def format_text(
    text: str,
    *,
    path: Optional[str] = ...,
    line_width: Optional[int] = ...,
    indent_width: Optional[int] = ...,
    new_line: Optional[str] = ...,
) -> str: ...
