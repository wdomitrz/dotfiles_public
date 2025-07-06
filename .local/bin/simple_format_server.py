#!/usr/bin/env -S uv run --script
################################################################
# Copyright (c) 2025 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################

# /// script
# dependencies = [
#     "lsprotocol",
#     "pygls",
# ]
# ///
import argparse
import logging
import subprocess

from lsprotocol.types import (  # pyright: ignore[reportMissingImports]
    TEXT_DOCUMENT_FORMATTING,
    DocumentFormattingParams,
    Position,
    Range,
    TextEdit,
)
from pygls.server import LanguageServer  # pyright: ignore[reportMissingImports]


def get_server(*, format_command: list[str]) -> LanguageServer:
    server = LanguageServer("format_serverls", "0.1")

    @server.feature(TEXT_DOCUMENT_FORMATTING)
    def formatting(
        ls: LanguageServer, params: DocumentFormattingParams
    ) -> list[TextEdit]:
        logging.debug("%s, %s", TEXT_DOCUMENT_FORMATTING, params)

        doc = ls.workspace.get_text_document(params.text_document.uri)

        try:
            formatted_text = subprocess.run(
                [c.format(file_path=doc.path) for c in format_command],
                input=doc.source,
                capture_output=True,
                text=True,
                check=True,
            ).stdout
            return [
                TextEdit(
                    range=Range(
                        start=Position(line=0, character=0),
                        end=Position(line=len(doc.lines), character=0),
                    ),
                    new_text=formatted_text,
                )
            ]
        except subprocess.CalledProcessError as e:
            error_info = dict(
                format_command=format_command,
                formated_file=doc.path,
                stderr=e.stderr,
            )
            logging.error("formatter error: %s", error_info)
            ls.show_message_log(f"formatter error: {error_info}")
            return []

    return server


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("format_command", type=str, nargs="+")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    server = get_server(format_command=args.format_command)
    return server.start_io()


if __name__ == "__main__":
    main()
