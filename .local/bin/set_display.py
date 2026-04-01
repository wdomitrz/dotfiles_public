#!/usr/bin/env -S uv run --script
################################################################
# Copyright (c) 2022 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################
#
# /// script
# dependencies = [
#     "typer",
# ]
# ///

from __future__ import annotations

import glob
import math
import os
import re
import subprocess
import tempfile
import textwrap
from collections.abc import Sequence
from dataclasses import dataclass

import typer  # pyright: ignore[reportMissingImports]

PRIMARY_ORDER_DEFAULT: tuple[str, ...] = ("screen", "rdp", "DP", "HDMI", "eDP")
DEFAULT_FAILSAFE_DPI = DEFAULT_GNOME_DPI = 96
ADDITIONAL_MULT = 1.0
LAPTOP_SCREEN_DIVISOR = 1.4
INCH_TO_MM = 25.4


@dataclass(kw_only=True)
class Display:
    name: str
    is_connected: bool
    is_primary: bool
    is_laptop: bool
    size: tuple[int, int] | None
    resolution: tuple[int, int] | None
    position: tuple[int, int] | None
    dpi: int | None

    @staticmethod
    def is_laptop_display(name: str) -> bool:
        return "eDP" == name[:3]

    @staticmethod
    def is_laptop_display_open() -> bool:
        state_files = glob.glob("/proc/acpi/button/lid/LID*/state")
        if len(state_files) == 0:
            raise RuntimeError("No laptop screen")

        for fn in state_files:
            with open(fn, encoding="utf-8") as f:
                if f.read().split()[1] == "open":
                    return True
        return False

    @staticmethod
    def get_display_info(display_info_line: str) -> Display:
        name, connected_state, maybe_primary = display_info_line.split()[:3]

        resolution_regex = re.compile(r"(\d+)i?x(\d+)i?(\+(\d+)\+(\d+))?")
        dimension_regex = re.compile(r"(\d+)mm\sx\s(\d+)mm")

        if (res := resolution_regex.search(display_info_line)) is not None:
            resolution = (int(res.group(1)), int(res.group(2)))
            if len(res.groups()) >= 5:
                position = (int(res.group(4)), int(res.group(5)))
            else:
                position = None
        else:
            resolution = position = None

        if (res := dimension_regex.search(display_info_line)) is not None:
            size = (int(res.group(1)), int(res.group(2)))
        else:
            size = None

        is_laptop = Display.is_laptop_display(name=name)

        return Display(
            name=name,
            is_connected=connected_state == "connected"
            and (not is_laptop or Display.is_laptop_display_open()),
            is_primary=maybe_primary == "primary",
            is_laptop=is_laptop,
            resolution=resolution,
            size=size,
            position=position,
            dpi=DpiSettings.get_my_dpi(
                resolution=resolution,
                size=size,
                is_laptop=is_laptop,
            ),
        )

    @staticmethod
    def get_all_displays() -> list[Display]:
        res = subprocess.run(
            "xrandr",
            text=True,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        return [
            Display.get_display_info(display_info_line)
            for display_info_line in res.stdout.splitlines()[1:]
            if not display_info_line.startswith(" ")
        ]

    def set_primary(self, **kwargs: str | None) -> None:
        self.is_primary = True
        return self.set_auto(**kwargs)

    def set_auto(
        self,
        same_as: str | None = None,
        left_of: str | None = None,
        right_of: str | None = None,
        above: str | None = None,
        below: str | None = None,
    ) -> None:
        cmd: list[str] = ["xrandr", "--output", self.name, "--auto"]

        if self.is_primary:
            cmd += ["--primary"]

        if same_as is not None:
            cmd += ["--same-as", same_as]
        elif left_of is not None:
            cmd += ["--left-of", left_of]
        elif right_of is not None:
            cmd += ["--right-of", right_of]
        elif above is not None:
            cmd += ["--above", above]
        elif below is not None:
            cmd += ["--below", below]

        if self.name.startswith("DP"):
            cmd += ["--set", "Broadcast RGB", "Full"]

        _ = subprocess.run(cmd, check=False)

    def set_off(self) -> None:
        _ = subprocess.run(["xrandr", "--output", self.name, "--off"], check=False)

    def set_auto_or_off(self) -> None:
        if os.getenv("SINGLE_DISPLAY") == "1":
            return self.set_off()
        else:
            return self.set_auto()


class GettingPrimaryDisplay:
    @staticmethod
    def new(
        displays: list[Display], primary_order: Sequence[str] = PRIMARY_ORDER_DEFAULT
    ) -> Display:
        for requirement in primary_order:
            for display in displays:
                if display.is_connected and display.name.startswith(requirement):
                    return display
        return displays[0]

    @staticmethod
    def current(displays: list[Display]) -> Display | None:
        for display in displays:
            if display.is_primary:
                return display
        return None

    @staticmethod
    def current_with_failsafe(
        displays: list[Display], primary_order: Sequence[str] = PRIMARY_ORDER_DEFAULT
    ) -> Display:
        primary_display = GettingPrimaryDisplay.current(displays)
        if primary_display is not None:
            return primary_display
        return GettingPrimaryDisplay.new(displays=displays, primary_order=primary_order)


class DpiSettings:
    @staticmethod
    def set_rofi_dpi(dpi: int) -> None:
        config_dir = os.path.expanduser("~/.config/rofi")
        if not os.path.isdir(config_dir):
            return

        add_dpi_import = False
        try:
            with open(
                os.path.join(config_dir, "config.rasi"), "r", encoding="utf-8"
            ) as rofi_config_file:
                for line in rofi_config_file.readlines():
                    if line == '@import "dpi"\n':
                        break
                else:
                    add_dpi_import = True
        except FileNotFoundError:
            add_dpi_import = True
        if add_dpi_import:
            with open(
                os.path.join(config_dir, "config.rasi"), "r+", encoding="utf-8"
            ) as rofi_config_file:
                rofi_config_contents = rofi_config_file.read()
                _ = rofi_config_file.seek(0)
                _ = rofi_config_file.write(
                    textwrap.dedent(
                        f"""\
                        @import "dpi"
                        {rofi_config_contents}
                        """
                    )
                )

        with open(
            os.path.join(config_dir, "dpi.rasi"), "w+", encoding="utf-8"
        ) as rofi_dpi_config_file:
            _ = rofi_dpi_config_file.write(
                textwrap.dedent(
                    f"""\
                    configuration {{
                        dpi: {dpi};
                    }}
                    """
                )
            )

    @staticmethod
    def set_gnome_text_scaling_factor(scaling_factor: float) -> None:
        _ = subprocess.run(
            [
                *"gsettings set org.gnome.desktop.interface text-scaling-factor".split(),
                str(scaling_factor),
            ],
            check=False,
        )

    @staticmethod
    def set_global_base_dpi(dpi: int) -> None:
        with tempfile.NamedTemporaryFile("w+") as config_file:
            _ = config_file.write(
                textwrap.dedent(
                    f"""\
                    Xft.dpi: {dpi}
                    """
                )
            )
            _ = config_file.seek(0)
            _ = subprocess.run(["xrdb", "-override", config_file.name], check=False)

    @classmethod
    def set_dpi(cls, dpi: int) -> None:
        cls.set_global_base_dpi(dpi)
        cls.set_rofi_dpi(dpi)
        cls.set_gnome_text_scaling_factor(dpi / DEFAULT_GNOME_DPI)

    @staticmethod
    def get_my_dpi(
        resolution: tuple[int, int] | None,
        size: tuple[int, int] | None,
        is_laptop: bool = False,
    ) -> int:
        additional_divisor = 1
        if is_laptop:
            additional_divisor *= LAPTOP_SCREEN_DIVISOR

        def get_my_base_dpi() -> float:
            if (
                resolution is None
                or size is None
                or any(s == 0 for s in size)
                or (
                    resolution[0] * size[0] == resolution[1] * size[1]
                    and math.gcd(*size) == 1
                )
            ):
                return DEFAULT_FAILSAFE_DPI

            return INCH_TO_MM * ADDITIONAL_MULT * max(resolution) / max(size)

        return round(get_my_base_dpi() / additional_divisor)

    @staticmethod
    def get_forced_dpi(forced_dpi: int | None = None) -> int | None:
        if forced_dpi is not None:
            return forced_dpi
        env_dpi = os.getenv("DPI")
        if env_dpi is not None and env_dpi != "":
            return int(env_dpi)

        return None

    @staticmethod
    def get_default_dpi(forced_dpi: int | None = None) -> int:
        if (
            forced_dpi := DpiSettings.get_forced_dpi(forced_dpi=forced_dpi)
        ) is not None:
            return forced_dpi

        displays = Display.get_all_displays()
        primary_display = GettingPrimaryDisplay.current_with_failsafe(displays)
        return (
            primary_display.dpi
            if primary_display.dpi is not None
            else DEFAULT_FAILSAFE_DPI
        )


app = typer.Typer()  # pyright: ignore[reportUnknownVariableType, reportUnknownMemberType]


class Commands:
    @app.command()  # pyright: ignore[reportUnknownMemberType, reportUntypedFunctionDecorator]
    @staticmethod
    def show_dpi(dpi: int | None = None) -> None:
        """Display inferred dpi."""
        print(DpiSettings.get_default_dpi(forced_dpi=dpi))

    @app.command()  # pyright: ignore[reportUnknownMemberType, reportUntypedFunctionDecorator]
    @staticmethod
    def set_dpi(dpi: int | None = None) -> None:
        DpiSettings.set_dpi(DpiSettings.get_default_dpi(forced_dpi=dpi))

    @app.callback(invoke_without_command=True)  # pyright: ignore[reportUnknownMemberType, reportUntypedFunctionDecorator]
    @staticmethod
    def set_display(ctx: typer.Context) -> None:  # pyright: ignore[reportUnknownParameterType, reportUnknownMemberType]
        if ctx.invoked_subcommand is not None:  # pyright: ignore[reportUnknownMemberType]
            return

        GettingPrimaryDisplay.new(Display.get_all_displays()).set_primary()
        DpiSettings.set_dpi(DpiSettings.get_default_dpi())

        for d in Display.get_all_displays():
            if not d.is_connected:
                d.set_off()
            elif not d.is_primary:
                d.set_auto_or_off()


if __name__ == "__main__":
    app()
