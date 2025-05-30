#!/usr/bin/env python3
################################################################
# Copyright (c) 2024 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################
import argparse
from typing import Literal

import i3ipc


def container_to_ignore(container: i3ipc.Con | None) -> bool:
    return (
        container is None
        or "_on" in container.floating  # Floating
        or container.fullscreen_mode == 1  # Full screen
        or container.parent.layout == "stacked"
        or container.parent.layout == "tabbed"
    )


def n_column_layout(i3: i3ipc.Connection, *, n: int | float) -> None:
    def resize_to_nth(i3: i3ipc.Connection, _event: i3ipc.WindowEvent) -> None:
        if container_to_ignore(container := i3.get_tree().find_focused()) or (
            container is not None
            and container.parent is not None
            and container.ipc_data["rect"]["y"]
            != container.parent.ipc_data["rect"]["y"]
        ):
            return

        workspace_width = container.workspace().rect.width
        container_width = container.rect.width
        size_delta = container_width % (workspace_width // n)
        if -n <= (size_delta - workspace_width // n):
            size_delta -= workspace_width // n

        _ = i3.command(f"resize set width {container_width - int(size_delta)}")

    def up_to_n_colums(i3: i3ipc.Connection, _event: i3ipc.WindowEvent) -> None:
        if container_to_ignore(container := i3.get_tree().find_focused()):
            return

        how_to_split: Literal["horizontal", "vertical"] = (
            "horizontal"
            if container.rect.width > 2 * container.workspace().rect.width // n - n
            else "vertical"
        )

        _ = i3.command(f"split {how_to_split}")

    i3.on(event=i3ipc.Event.WINDOW_CLOSE, handler=resize_to_nth)
    i3.on(event=i3ipc.Event.WINDOW_FOCUS, handler=up_to_n_colums)
    i3.on(event=i3ipc.Event.WINDOW_FULLSCREEN_MODE, handler=resize_to_nth)
    i3.on(event=i3ipc.Event.WINDOW_MOVE, handler=resize_to_nth)
    i3.on(event=i3ipc.Event.WINDOW_NEW, handler=resize_to_nth)


def main():
    parser = argparse.ArgumentParser()
    _ = parser.add_argument(
        "-n",
        "--number-of-columns",
        help="number of columns",
        type=float,
        default=2,
    )
    args = parser.parse_args()

    i3 = i3ipc.Connection()
    n_column_layout(i3, n=args.number_of_columns)
    i3.main()


if __name__ == "__main__":
    main()
