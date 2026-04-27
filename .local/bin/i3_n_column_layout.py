#!/usr/bin/env python3
################################################################
# Copyright (c) 2024 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################

from dataclasses import dataclass
from typing import Literal, cast

import i3ipc  # pyright: ignore[reportMissingTypeStubs]
import typer


def container_to_ignore(container: i3ipc.Con) -> bool:
    return (
        "_on" in container.floating  # pyright: ignore[reportUnknownMemberType, reportAttributeAccessIssue]
        or getattr(container, "fullscreen_mode", 0) == 1
        or container.parent.layout in ["stacked", "tabbed"]  # pyright: ignore[reportUnknownMemberType]
    )


def get_container_width(container: i3ipc.Con) -> int:
    return cast(int, container.rect.width)


def get_workspace_width(container: i3ipc.Con) -> int:
    workspace = container.workspace()
    assert workspace is not None
    return cast(int, workspace.rect.width)


def n_column_layout(i3: i3ipc.Connection, *, n: int | float) -> None:
    def resize_to_nth(i3: i3ipc.Connection, event: i3ipc.events.IpcBaseEvent) -> None:
        del event
        container = i3.get_tree().find_focused()
        if (
            container is None
            or container_to_ignore(container)
            or (
                container.parent is not None
                and (
                    cast(i3ipc.Con, container.parent).ipc_data["rect"]["y"]
                    != container.ipc_data["rect"]["y"]
                )
            )
        ):
            return

        workspace_width = get_workspace_width(container)
        container_width = get_container_width(container)
        size_delta = container_width % (workspace_width // n)
        if -n <= (size_delta - workspace_width // n):
            size_delta -= workspace_width // n

        _ = i3.command(f"resize set width {container_width - int(size_delta)}")

    def up_to_n_colums(i3: i3ipc.Connection, event: i3ipc.events.IpcBaseEvent) -> None:
        del event
        container = i3.get_tree().find_focused()
        if container is None or container_to_ignore(container):
            return

        if get_container_width(container) > 2 * get_workspace_width(container) // n - n:
            how_to_split: Literal["horizontal", "vertical"] = "horizontal"
        else:
            how_to_split = "vertical"

        _ = i3.command(f"split {how_to_split}")

    i3.on(event=i3ipc.Event.WINDOW_CLOSE, handler=resize_to_nth)
    i3.on(event=i3ipc.Event.WINDOW_FOCUS, handler=up_to_n_colums)
    i3.on(event=i3ipc.Event.WINDOW_FULLSCREEN_MODE, handler=resize_to_nth)
    i3.on(event=i3ipc.Event.WINDOW_MOVE, handler=resize_to_nth)
    i3.on(event=i3ipc.Event.WINDOW_NEW, handler=resize_to_nth)


@dataclass(kw_only=True, frozen=True)
class Args:
    number_of_columns: float = 2.0

    def __post_init__(self) -> None:
        return main(self)


def main(args: Args):
    i3 = i3ipc.Connection()
    n_column_layout(i3, n=args.number_of_columns)
    i3.main()


if __name__ == "__main__":
    typer.run(Args)
