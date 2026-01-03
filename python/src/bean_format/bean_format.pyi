from typing import Optional

__all__ = ["format_text"]

def format_text(
    text: str,
    *,
    line_width: Optional[int] = ...,
    use_tabs: Optional[bool] = ...,
    indent_width: Optional[int] = ...,
    new_line_kind: Optional[str] = ...,
) -> str: ...
