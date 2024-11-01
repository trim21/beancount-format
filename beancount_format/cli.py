import io
import pathlib
import sys
from collections.abc import Generator, Sequence

import click
from beancount_parser.parser import make_parser

from beancount_format.format import Formatter


def __input_files(paths: Sequence[pathlib.Path]) -> Generator[pathlib.Path, None, None]:
    for p in paths:
        if p.is_file():
            yield p
        else:
            yield from __input_files(p.iterdir())


@click.command
@click.argument("path", nargs=-1, type=click.Path(path_type=pathlib.Path))
@click.option("--indent", default=4, type=int)
def main(path, indent: int):
    parser = make_parser()
    formatter = Formatter(indent_width=indent)

    exit_code = 0

    for file in __input_files(path):
        if file.suffix.lower() not in {".bean"}:
            continue
        try:
            input_content = file.read_text(encoding="utf-8")
            tree = parser.parse(input_content)
            output_file = io.StringIO()
            with output_file:
                formatter.format(tree, output_file)
                formatted = output_file.getvalue()
            if input_content == formatted:
                continue
            print("formatting", file)
            file.write_bytes(formatted.encode("utf-8"))
            exit_code = 1
        except Exception as e:
            print("failed to format file", file, e)
            return 1

    return sys.exit(exit_code)
