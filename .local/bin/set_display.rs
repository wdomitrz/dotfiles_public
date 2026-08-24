#!/usr/bin/env rscript.sh
// Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
// AGPL License

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, ExitCode};

const USAGE: &str =
    "usage: ./set_display.rs [set-display | show-dpi [--dpi DPI] | set-dpi [--dpi DPI]]";

const PRIMARY_ORDER_DEFAULT: [&str; 5] = ["screen", "rdp", "DP", "HDMI", "eDP"];
// A display whose mm dimensions are missing, zero, or coprime with an aspect
// ratio mismatching the pixels reports garbage EDID data; fall back to 96.
const DEFAULT_FAILSAFE_DPI: f64 = 96.0;
const LAPTOP_SCREEN_DIVISOR: f64 = 1.4;
const GNOME_BASE_DPI: f64 = 96.0;
const INCH_TO_MM: f64 = 25.4;

fn dpi_to_f64(dpi: i64) -> f64 {
    // Exact for any realistic magnitude; avoids a lossy-looking int->float cast.
    dpi.to_string().parse().unwrap_or_default()
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn calc_dpi(resolution: Option<(u32, u32)>, size_mm: Option<(u32, u32)>, laptop: bool) -> f64 {
    let base = match (resolution, size_mm) {
        (Some(res), Some(mm))
            if mm.0 > 0 && mm.1 > 0 && !(res.0 * mm.1 == res.1 * mm.0 && gcd(mm.0, mm.1) == 1) =>
        {
            INCH_TO_MM * f64::from(res.0.max(res.1)) / f64::from(mm.0.max(mm.1))
        }
        _ => DEFAULT_FAILSAFE_DPI,
    };
    let divisor = if laptop { LAPTOP_SCREEN_DIVISOR } else { 1.0 };
    (base / divisor).round()
}

fn lid_open() -> io::Result<bool> {
    let mut seen_any = false;
    for entry in fs::read_dir("/proc/acpi/button/lid")? {
        let state = fs::read_to_string(entry?.path().join("state"))?;
        seen_any = true;
        if state.split_whitespace().nth(1) == Some("open") {
            return Ok(true);
        }
    }
    if !seen_any {
        return Err(io::Error::other("no laptop screen"));
    }
    Ok(false)
}

fn parse_resolution_token(token: &str) -> Option<(u32, u32)> {
    let head = token.split('+').next()?;
    let (width, height) = head.split_once('x')?;
    let width = width.strip_suffix('i').unwrap_or(width);
    let height = height.strip_suffix('i').unwrap_or(height);
    Some((width.parse().ok()?, height.parse().ok()?))
}

fn parse_dimensions(tokens: &[&str]) -> Option<(u32, u32)> {
    // Skip bare "x" tokens from e.g. "(normal left inverted right x axis ...".
    tokens.iter().enumerate().find_map(|(i, token)| {
        if *token != "x" {
            return None;
        }
        let width = tokens.get(i.checked_sub(1)?)?.strip_suffix("mm")?;
        let height = tokens.get(i + 1)?.strip_suffix("mm")?;
        Some((width.parse().ok()?, height.parse().ok()?))
    })
}

struct Display {
    name: String,
    connected: bool,
    primary: bool,
    dpi: f64,
}

impl Display {
    fn parse(line: &str) -> io::Result<Option<Self>> {
        Self::parse_with_lid(line, lid_open)
    }

    fn parse_with_lid(
        line: &str,
        lid: impl FnOnce() -> io::Result<bool>,
    ) -> io::Result<Option<Self>> {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let Some(name) = tokens.first() else {
            return Ok(None);
        };
        let Some(state) = tokens.get(1) else {
            return Ok(None);
        };

        let laptop = name.starts_with("eDP");
        let connected = if *state != "connected" {
            false
        } else if !laptop {
            true
        } else {
            lid()?
        };
        let resolution = tokens
            .iter()
            .find_map(|token| parse_resolution_token(token));
        let size_mm = parse_dimensions(&tokens);
        let dpi = calc_dpi(resolution, size_mm, laptop);

        Ok(Some(Self {
            name: (*name).to_string(),
            connected,
            primary: tokens.get(2) == Some(&"primary"),
            dpi,
        }))
    }
}

fn get_all_displays() -> io::Result<Vec<Display>> {
    let output = Command::new("xrandr").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let info_lines = stdout.lines().filter(|line| !line.starts_with(' ')).skip(1);
    let mut displays = Vec::new();
    for line in info_lines {
        if let Some(display) = Display::parse(line)? {
            displays.push(display);
        }
    }
    Ok(displays)
}

fn pick_primary(displays: &[Display]) -> Option<&Display> {
    for prefix in PRIMARY_ORDER_DEFAULT {
        if let Some(display) = displays
            .iter()
            .find(|d| d.connected && d.name.starts_with(prefix))
        {
            return Some(display);
        }
    }
    displays.first()
}

fn current_primary(displays: &[Display]) -> Option<&Display> {
    displays.iter().find(|display| display.primary)
}

fn run_xrandr(display: &Display, mode: &str, extra: &[&str]) -> io::Result<()> {
    let mut cmd = Command::new("xrandr");
    cmd.arg("--output").arg(&display.name).arg(mode);
    cmd.args(extra);
    if mode == "--auto" && display.name.starts_with("DP") {
        cmd.args(["--set", "Broadcast RGB", "Full"]);
    }
    cmd.status().map(|_| ())
}

fn set_gnome_text_scaling_factor(factor: f64) {
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "text-scaling-factor"])
        .arg(format!("{factor:?}"))
        .status();
}

fn set_global_base_dpi(dpi: f64) -> io::Result<()> {
    let path = env::temp_dir().join(format!("set_display_xrdb_{}", std::process::id()));
    fs::write(&path, format!("Xft.dpi: {dpi:.0}\n"))?;
    let _ = Command::new("xrdb").arg("-override").arg(&path).status();
    drop(fs::remove_file(&path));
    Ok(())
}

fn set_rofi_dpi(dpi: f64) -> io::Result<()> {
    let Some(home) = env::var_os("HOME") else {
        return Ok(());
    };
    let config_dir = std::path::PathBuf::from(home).join(".config").join("rofi");
    if !config_dir.is_dir() {
        return Ok(());
    }

    let config_path = config_dir.join("config.rasi");
    let needs_import = match fs::read_to_string(&config_path) {
        Ok(contents) => !contents.lines().any(|line| line == "@import \"dpi\""),
        Err(_) => true,
    };
    if needs_import {
        let contents = fs::read_to_string(&config_path).unwrap_or_default();
        fs::write(&config_path, format!("@import \"dpi\"\n{contents}\n"))?;
    }

    fs::write(
        config_dir.join("dpi.rasi"),
        format!("configuration {{\n    dpi: {dpi:.0};\n}}\n"),
    )
}

fn set_dpi(dpi: f64) -> io::Result<()> {
    set_global_base_dpi(dpi)?;
    set_rofi_dpi(dpi)?;
    set_gnome_text_scaling_factor(dpi / GNOME_BASE_DPI);
    Ok(())
}

fn forced_dpi(explicit: Option<i64>) -> io::Result<Option<f64>> {
    if let Some(dpi) = explicit {
        return Ok(Some(dpi_to_f64(dpi)));
    }
    match env::var("DPI") {
        Ok(value) if !value.is_empty() => value
            .trim()
            .parse::<i64>()
            .map(|dpi| Some(dpi_to_f64(dpi)))
            .map_err(|_| io::Error::other(format!("invalid DPI environment variable: {value}"))),
        _ => Ok(None),
    }
}

fn default_dpi(forced: Option<f64>) -> io::Result<f64> {
    if let Some(dpi) = forced {
        return Ok(dpi);
    }
    let displays = get_all_displays()?;
    let primary = current_primary(&displays)
        .or_else(|| pick_primary(&displays))
        .ok_or_else(|| io::Error::other("no displays found"))?;
    Ok(primary.dpi)
}

fn set_display() -> io::Result<()> {
    let displays = get_all_displays()?;
    let primary = pick_primary(&displays).ok_or_else(|| io::Error::other("no displays found"))?;
    run_xrandr(primary, "--auto", &["--primary"])?;
    set_dpi(default_dpi(None)?)?;

    // Mirror the reference: judge each display by its reported primary flag
    // from a fresh xrandr query, not by the choice made above.
    let single_display = env::var("SINGLE_DISPLAY").as_deref() == Ok("1");
    for display in get_all_displays()? {
        if display.primary {
            continue;
        }
        if !display.connected || single_display {
            run_xrandr(&display, "--off", &[])?;
        } else {
            run_xrandr(&display, "--auto", &[])?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum Subcommand {
    SetDisplay,
    ShowDpi,
    SetDpi,
}

struct Args {
    command: Subcommand,
    dpi: Option<i64>,
}

impl Args {
    fn parse(argv: &[String]) -> Result<Self, String> {
        let mut command = Subcommand::SetDisplay;
        let mut dpi = None;
        let mut in_subcommand = false;
        let mut i = 0;
        while i < argv.len() {
            match argv[i].as_str() {
                "-h" | "--help" => return Err(USAGE.to_string()),
                "set-display" | "show-dpi" | "set-dpi" => {
                    if in_subcommand {
                        return Err(format!("unknown argument: {}", argv[i]));
                    }
                    command = match argv[i].as_str() {
                        "set-display" => Subcommand::SetDisplay,
                        "show-dpi" => Subcommand::ShowDpi,
                        _ => Subcommand::SetDpi,
                    };
                    in_subcommand = command != Subcommand::SetDisplay;
                }
                "--dpi" => {
                    if !in_subcommand {
                        return Err("unrecognized argument: --dpi".to_string());
                    }
                    i += 1;
                    let value = argv
                        .get(i)
                        .ok_or_else(|| "argument --dpi: expected one argument".to_string())?;
                    dpi =
                        Some(value.parse().map_err(|_| {
                            format!("argument --dpi: invalid integer value: {value}")
                        })?);
                }
                other => return Err(format!("unknown argument: {other}")),
            }
            i += 1;
        }
        Ok(Self { command, dpi })
    }

    fn run(&self, out: &mut dyn Write) -> io::Result<i32> {
        match self.command {
            Subcommand::ShowDpi => writeln!(out, "{:.0}", default_dpi(forced_dpi(self.dpi)?)?)?,
            Subcommand::SetDpi => set_dpi(default_dpi(forced_dpi(self.dpi)?)?)?,
            Subcommand::SetDisplay => set_display()?,
        }
        Ok(0)
    }
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let parsed = match Args::parse(&argv) {
        Ok(parsed) => parsed,
        Err(message) if message.starts_with("usage:") => {
            println!("{message}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("set_display: {message}");
            return ExitCode::from(2);
        }
    };
    match parsed.run(&mut std::io::stdout().lock()) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            eprintln!("set_display: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{calc_dpi, parse_dimensions, parse_resolution_token, Args, Display, Subcommand};

    #[test]
    fn parses_resolution_tokens() {
        assert_eq!(parse_resolution_token("1920x1080+0+0"), Some((1920, 1080)));
        assert_eq!(parse_resolution_token("1920x1080"), Some((1920, 1080)));
        assert_eq!(parse_resolution_token("1280ix720i"), Some((1280, 720)));
        assert_eq!(parse_resolution_token("x"), None);
        assert_eq!(parse_resolution_token("(normal"), None);
    }

    #[test]
    fn parses_dimensions() {
        let tokens: Vec<&str> = "509mm x 286mm".split_whitespace().collect();
        assert_eq!(parse_dimensions(&tokens), Some((509, 286)));
        let tokens: Vec<&str> = "connected primary".split_whitespace().collect();
        assert_eq!(parse_dimensions(&tokens), None);
        // The "x" inside the parenthetical must not abort the scan.
        let tokens: Vec<&str> =
            "2560x1440+0+0 (normal left inverted right x axis y axis) 309mm x 174mm"
                .split_whitespace()
                .collect();
        assert_eq!(parse_dimensions(&tokens), Some((309, 174)));
    }

    fn dpi_of(resolution: Option<(u32, u32)>, size_mm: Option<(u32, u32)>, laptop: bool) -> String {
        format!("{:.0}", calc_dpi(resolution, size_mm, laptop))
    }

    #[test]
    fn calculates_dpi() {
        assert_eq!(dpi_of(Some((1920, 1080)), Some((509, 286)), false), "96");
        assert_eq!(dpi_of(Some((1920, 1080)), Some((509, 286)), true), "68");
        assert_eq!(dpi_of(None, None, false), "96");
        assert_eq!(dpi_of(Some((1920, 1080)), Some((0, 0)), false), "96");
        // Matching pixel/mm aspect with coprime mm means bogus EDID data.
        assert_eq!(dpi_of(Some((1500, 1000)), Some((3, 2)), false), "96");
    }

    #[test]
    fn parses_display_lines() {
        let display = Display::parse("DP1 connected primary 3840x2160+0+0 (normal) 600mm x 340mm")
            .expect("parses")
            .expect("a display");
        assert_eq!(display.name, "DP1");
        assert!(display.connected && display.primary);
        assert_eq!(format!("{:.0}", display.dpi), "163");

        // Real-world laptop panel: 25.4 * 2560 / 309 / 1.4 ~= 150.
        let display = Display::parse_with_lid(
            "eDP-1 connected primary 2560x1440+0+0 (normal left inverted right x axis y axis) 309mm x 174mm",
            || Ok(true),
        )
        .expect("parses")
        .expect("a display");
        assert_eq!(display.name, "eDP-1");
        assert_eq!(format!("{:.0}", display.dpi), "150");

        let display = Display::parse("HDMI1 disconnected (normal)").expect("parses");
        let display = display.expect("a display");
        assert!(!display.connected);
        assert_eq!(format!("{:.0}", display.dpi), "96");
    }

    #[test]
    fn parses_arguments() {
        assert!(matches!(
            Args::parse(&[]).expect("parses").command,
            Subcommand::SetDisplay
        ));
        assert!(matches!(
            Args::parse(&["show-dpi".to_string()])
                .expect("parses")
                .command,
            Subcommand::ShowDpi
        ));
        let args = Args::parse(&[
            "set-dpi".to_string(),
            "--dpi".to_string(),
            "144".to_string(),
        ])
        .expect("parses");
        assert!(matches!(args.command, Subcommand::SetDpi));
        assert_eq!(args.dpi, Some(144));
        assert!(matches!(
            Args::parse(&["set-display".to_string()])
                .expect("parses")
                .command,
            Subcommand::SetDisplay
        ));
        assert_eq!(
            Args::parse(&[
                "show-dpi".to_string(),
                "--dpi".to_string(),
                "-5".to_string()
            ])
            .expect("parses")
            .dpi,
            Some(-5)
        );

        assert!(Args::parse(&["--bogus".to_string()]).is_err());
        // Deliberate divergence: set-display is an explicit subcommand here.
        // Like argparse, a second subcommand token is rejected.
        assert!(Args::parse(&["show-dpi".to_string(), "set-display".to_string()]).is_err());
        assert!(Args::parse(&["show-dpi".to_string(), "set-dpi".to_string()]).is_err());
        // --dpi is a subcommand-only option; the reference rejects it up top.
        assert!(Args::parse(&["--dpi".to_string()]).is_err());
        assert!(Args::parse(&["--dpi".to_string(), "5".to_string()]).is_err());
        assert!(Args::parse(&["show-dpi".to_string(), "--dpi".to_string()]).is_err());
        assert!(Args::parse(&[
            "show-dpi".to_string(),
            "--dpi".to_string(),
            "abc".to_string()
        ])
        .is_err());
    }
}
