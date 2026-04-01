#!/usr/bin/env -S uv run --script
################################################################
# Copyright (c) 2025 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################
#
# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "lsprotocol",
#     "typer",
#     "pygls==1.3",
# ]
# ///
#
# pyright: reportAny = false
# pyright: reportMissingImports = false
# pyright: reportUnknownArgumentType = false
# pyright: reportUnknownMemberType = false
# pyright: reportUnknownParameterType = false
# pyright: reportUnknownVariableType = false
# pyright: reportUntypedFunctionDecorator = false
# pyright: reportUnusedCallResult = false
# pyright: reportUnusedFunction = false

import logging
import subprocess
from dataclasses import dataclass

import typer
from lsprotocol.types import (
    TEXT_DOCUMENT_FORMATTING,
    DocumentFormattingParams,
    Position,
    Range,
    TextEdit,
)
from pygls.server import LanguageServer


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


@dataclass(kw_only=True, frozen=True)
class Args:
    format_command: list[str]

    def __post_init__(self) -> None:
        return main(self)


def main(args: Args) -> None:
    server = get_server(format_command=args.format_command)
    return server.start_io()


if __name__ == "__main__":
    typer.run(Args)
