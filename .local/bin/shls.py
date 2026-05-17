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
#   "typing_extensions",
# ]
# ///
#
# pyright: reportAny = false
# pyright: reportMissingImports = false
# pyright: reportMissingTypeArgument = false
# pyright: reportUnknownArgumentType = false
# pyright: reportUnknownMemberType = false
# pyright: reportUnknownParameterType = false
# pyright: reportUnknownVariableType = false
# pyright: reportUntypedFunctionDecorator = false
# pyright: reportUnusedFunction = false
# pyright: reportUnusedParameter = false

from __future__ import annotations

import argparse
import json
import logging
import subprocess
from dataclasses import dataclass

from lsprotocol.types import (
    TEXT_DOCUMENT_CODE_ACTION,
    TEXT_DOCUMENT_DIAGNOSTIC,
    AnnotatedTextEdit,
    CodeAction,
    CodeActionKind,
    CodeActionOptions,
    CodeActionParams,
    CodeDescription,
    Diagnostic,
    DiagnosticOptions,
    DiagnosticSeverity,
    DocumentDiagnosticParams,
    FullDocumentDiagnosticReport,
    OptionalVersionedTextDocumentIdentifier,
    Position,
    Range,
    TextDocumentEdit,
    TextEdit,
    WorkspaceEdit,
)
from pygls.server import LanguageServer
from pygls.workspace import TextDocument
from typing_extensions import Self

SEVERITY_MAPPING: dict[str, DiagnosticSeverity] = dict(
    error=DiagnosticSeverity.Error,
    warning=DiagnosticSeverity.Warning,
    info=DiagnosticSeverity.Information,
    style=DiagnosticSeverity.Hint,
)


def parse_replacement(
    *,
    column: int,
    end_column: int,
    end_line: int,
    insertion_point: str,
    line: int,
    precedence: int,
    replacement: str,
) -> TextEdit:
    del insertion_point, precedence
    return TextEdit(
        range=Range(
            start=Position(line=line - 1, character=column - 1),
            end=Position(line=end_line - 1, character=end_column - 1),
        ),
        new_text=replacement,
    )


def parse_replacements(
    *,
    file_uri: str,
    message: str,
    diagnostic: Diagnostic,
    fix: None | dict[str, list],
) -> CodeAction | None:
    if fix is None:
        return None

    replacements: list[TextEdit | AnnotatedTextEdit] = [
        parse_replacement(
            column=r["column"],
            end_column=r["endColumn"],
            end_line=r["endLine"],
            insertion_point=r["insertionPoint"],
            line=r["line"],
            precedence=r["precedence"],
            replacement=r["replacement"],
        )
        for r in fix.get("replacements", [])
    ]
    if len(replacements) == 0:
        return None

    text_document_edit = TextDocumentEdit(
        text_document=OptionalVersionedTextDocumentIdentifier(uri=file_uri),
        edits=replacements,
    )
    return CodeAction(
        title=message,
        diagnostics=[diagnostic],
        kind=CodeActionKind.QuickFix,
        is_preferred=True,
        edit=WorkspaceEdit(document_changes=[text_document_edit]),
    )


@dataclass(frozen=True, kw_only=True)
class DiagnosticAndCodeAction:
    diagnostic: Diagnostic
    code_action: CodeAction | None


def parse_diagnostic(
    *,
    file_uri: str,
    file: str,
    line: int,
    end_line: int,
    column: int,
    end_column: int,
    level: str,
    code: int,
    message: str,
    fix: None | dict[str, list],
) -> DiagnosticAndCodeAction:
    del file
    diagnostic = Diagnostic(
        range=Range(
            start=Position(line - 1, column - 1),
            end=Position(end_line - 1, end_column - 1),
        ),
        message=message,
        severity=SEVERITY_MAPPING.get(level),
        code=code,
        code_description=CodeDescription(
            href=f"https://www.shellcheck.net/wiki/SC{code}"
        ),
        source="shls",
    )
    code_action = parse_replacements(
        file_uri=file_uri, message=message, fix=fix, diagnostic=diagnostic
    )
    return DiagnosticAndCodeAction(diagnostic=diagnostic, code_action=code_action)


def lint(
    *, ls: LanguageServer, doc: TextDocument, file_uri: str
) -> list[DiagnosticAndCodeAction]:
    diagnostics_command = [
        "shellcheck",
        "-",
        "--exclude=SC1091,SC2312",
        "--enable=all",
        "--format=json1",
    ]
    try:
        diagnostics_process = subprocess.run(
            diagnostics_command,
            input=doc.source,
            capture_output=True,
            text=True,
            check=False,
        )
        diagnostics = json.loads(diagnostics_process.stdout)
        return [
            parse_diagnostic(
                file_uri=file_uri,
                file=d["file"],
                line=d["line"],
                end_line=d["endLine"],
                column=d["column"],
                end_column=d["endColumn"],
                level=d["level"],
                code=d["code"],
                message=d["message"],
                fix=d.get("fix"),
            )
            for d in diagnostics["comments"]
        ]
    except (OSError, json.JSONDecodeError) as e:
        error_info = dict(
            diagnostics_command=diagnostics_command,
            formated_file=doc.path,
            error=str(e),
        )
        logging.exception("diagnostics error: %s", error_info)
        ls.show_message_log(f"diagnostics error: {error_info}")
        return []


def get_server() -> LanguageServer:
    server = LanguageServer("shls", "0.1")

    @server.feature(
        TEXT_DOCUMENT_DIAGNOSTIC,
        DiagnosticOptions(inter_file_dependencies=False, workspace_diagnostics=False),
    )
    def document_diagnostic(
        ls: LanguageServer, params: DocumentDiagnosticParams
    ) -> FullDocumentDiagnosticReport:
        """Return diagnostics for the requested document"""
        logging.debug("%s", params)
        file_uri = params.text_document.uri
        return FullDocumentDiagnosticReport(
            items=[
                x.diagnostic
                for x in lint(
                    ls=ls,
                    doc=ls.workspace.get_text_document(file_uri),
                    file_uri=file_uri,
                )
            ]
        )

    @server.feature(
        TEXT_DOCUMENT_CODE_ACTION,
        CodeActionOptions(code_action_kinds=[CodeActionKind.QuickFix]),
    )
    def code_actions(ls: LanguageServer, params: CodeActionParams) -> list[CodeAction]:
        logging.debug("%s", params)
        file_uri = params.text_document.uri
        return [
            x.code_action
            for x in lint(
                ls=ls, doc=ls.workspace.get_text_document(file_uri), file_uri=file_uri
            )
            if x.code_action is not None
        ]

    return server


@dataclass(kw_only=True, frozen=True)
class Args:
    @classmethod
    def from_args(cls, argv: list[str] | None = None) -> Self:
        parser = argparse.ArgumentParser()
        _ = parser.parse_args(argv)
        return cls()

    def run(self) -> int:
        server = get_server()
        server.start_io()
        return 0


if __name__ == "__main__":
    raise SystemExit(Args.from_args().run())
