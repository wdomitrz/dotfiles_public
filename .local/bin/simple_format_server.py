#!/usr/bin/env python3
################################################################
# Copyright (c) 2025 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################
#
# /// script
# dependencies = [
#   "lsprotocol",
#   "pygls",
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

import argparse
import logging
import subprocess
from dataclasses import dataclass
from typing import cast

from lsprotocol.types import (
    TEXT_DOCUMENT_FORMATTING,
    DocumentFormattingParams,
    Position,
    Range,
    TextEdit,
)
from pygls.server import LanguageServer
from typing_extensions import Self


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
            logging.exception("formatter error: %s", error_info)
            ls.show_message_log(f"formatter error: {error_info}")
            return []

    return server


@dataclass(kw_only=True, frozen=True)
class Args:
    format_command: list[str]

    @classmethod
    def from_args(cls, argv: list[str] | None = None) -> Self:
        parser = argparse.ArgumentParser()
        _ = parser.add_argument("format_command", nargs="+")
        args = parser.parse_args(argv)
        return cls(format_command=cast(list[str], args.format_command))

    def run(self) -> int:
        server = get_server(format_command=self.format_command)
        server.start_io()
        return 0


if __name__ == "__main__":
    raise SystemExit(Args.from_args().run())
