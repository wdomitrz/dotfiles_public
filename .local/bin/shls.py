#!/usr/bin/env -S uv run --script
################################################################
# Copyright (c) 2025 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################
#
# /// script
# dependencies = [
#     "lsprotocol",
#     "pygls==1.3",
# ]
# ///

import json
import logging
import subprocess
from dataclasses import dataclass

from lsprotocol.types import (  # pyright: ignore[reportMissingImports]
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
from pygls.server import LanguageServer  # pyright: ignore[reportMissingImports]
from pygls.workspace import TextDocument  # pyright: ignore[reportMissingImports]

SEVERITY_MAPPING: dict[str, DiagnosticSeverity] = dict(
    error=DiagnosticSeverity.Error,
    warning=DiagnosticSeverity.Warning,
    info=DiagnosticSeverity.Information,
    style=DiagnosticSeverity.Hint,
)


def parse_replacement(
    *,
    column: int,
    endColumn: int,
    endLine: int,
    insertionPoint: str,
    line: int,
    precedence: int,
    replacement: str,
) -> TextEdit:
    return TextEdit(
        range=Range(
            start=Position(line=line - 1, character=column - 1),
            end=Position(line=endLine - 1, character=endColumn - 1),
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
        parse_replacement(**r) for r in fix.get("replacements", [])
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
    endLine: int,
    column: int,
    endColumn: int,
    level: str,
    code: int,
    message: str,
    fix: None | dict[str, list],
) -> DiagnosticAndCodeAction:
    diagnostic = Diagnostic(
        range=Range(
            start=Position(line - 1, column - 1),
            end=Position(endLine - 1, endColumn - 1),
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
    DIAGNOSTICS_COMMAND = [
        "shellcheck",
        "-",
        "--exclude=SC1091,SC2312",
        "--enable=all",
        "--format=json1",
    ]
    try:
        diagnostics = json.loads(
            subprocess.run(
                DIAGNOSTICS_COMMAND,
                input=doc.source,
                capture_output=True,
                text=True,
                check=False,
            ).stdout
        )
        return [
            parse_diagnostic(file_uri=file_uri, **d) for d in diagnostics["comments"]
        ]
    except subprocess.CalledProcessError as e:
        error_info = dict(
            diagnostics_command=DIAGNOSTICS_COMMAND,
            formated_file=doc.path,
            stderr=e.stderr,
        )
        logging.error("diagnostics error: %s", error_info)
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


def main() -> None:
    server = get_server()
    return server.start_io()


if __name__ == "__main__":
    main()
