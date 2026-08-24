#!/usr/bin/env rscript.sh
// Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
// AGPL License

use std::fmt::Write as _;
use std::io::{BufRead, Read, Write};
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};

const USAGE: &str = "\
usage: ./qr.rs [OPTIONS]
       ./qr.rs wifi --ssid SSID [OPTIONS]

Generate a QR code from --text (or stdin), or from WiFi credentials.
Run \"./qr.rs wifi --help\" for WiFi-specific options.

options:
  --text <TEXT>                      payload text (reads stdin when absent)
  -l, --error-correction <LEVEL>     L | M | Q | H (default: L)
  -q, --quiet-zone <N>               quiet zone width in modules (default: 2)
  -f, --format <FORMAT>              terminal | bits | ascii | svg | png
                                     (default: terminal)
      --mode <MODE>                  auto | numeric | alphanumeric | byte
                                     (default: auto; kanji is unsupported)
      --version <N>                  pin the QR version, 1-40 (default: the
                                     smallest version that fits)
      --split-mode <HOW>             all | wait | disabled: render every code,
                                     pause between codes, or fail on overflow
                                     (default: all)
      --output <BASE>                write each code to BASE[-N].<ext> files
                                     instead of stdout; multiple png codes
                                     without --output go to qr-N.png files
  -h, --help                         show this help";

const WIFI_USAGE: &str = "\
usage: ./qr.rs wifi --ssid SSID [OPTIONS]

Generate a QR code for WiFi credentials. The password is read interactively
when stdin is a tty, otherwise from stdin.

options:
      --ssid <SSID>                  wifi network name (required)
      --auth <AUTH>                  WPA | WEP | nopass (default: WPA)
      --hidden                       mark the network as hidden
  -l, --error-correction <LEVEL>     L | M | Q | H (default: L)
      --version <N>                  pin the QR version, 1-40 (default: the
                                     smallest version that fits)
  -q, --quiet-zone <N>               quiet zone width in modules (default: 2)
  -f, --format <FORMAT>              terminal | bits | ascii | svg | png
                                     (default: terminal)
      --split-mode <HOW>             all | wait | disabled: render every code,
                                     pause between codes, or fail on overflow
                                     (default: all)
      --output <BASE>                write each code to BASE[-N].<ext> files
                                     instead of stdout; multiple png codes
                                     without --output go to qr-N.png files
  -h, --help                         show this help";

const ANSI_BLACK: &str = "\u{1b}[40m  \u{1b}[0m";
const ANSI_WHITE: &str = "\u{1b}[47m  \u{1b}[0m";
const PNG_BLACK: [u8; 3] = [0x00, 0x00, 0x00];
const PNG_WHITE: [u8; 3] = [0xff, 0xff, 0xff];

#[derive(Clone, Copy)]
enum Format {
    Terminal,
    Bits,
    Ascii,
    Svg,
    Png,
}

impl Format {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "terminal" => Some(Self::Terminal),
            "bits" => Some(Self::Bits),
            "ascii" => Some(Self::Ascii),
            "svg" => Some(Self::Svg),
            "png" => Some(Self::Png),
            _ => None,
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::Bits => "bits",
            Self::Ascii => "ascii",
            Self::Svg => "svg",
            Self::Png => "png",
        }
    }

    fn multi_code_separator(self) -> &'static str {
        match self {
            Self::Svg => "\n",
            Self::Terminal | Self::Bits | Self::Ascii | Self::Png => "\n\n",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SplitMode {
    All,
    Wait,
    Disabled,
}

impl SplitMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "wait" => Some(Self::Wait),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Command {
    Text,
    Wifi,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WifiAuth {
    Wpa,
    Wep,
    Nopass,
}

impl WifiAuth {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "WPA" => Some(Self::Wpa),
            "WEP" => Some(Self::Wep),
            "nopass" => Some(Self::Nopass),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Wpa => "WPA",
            Self::Wep => "WEP",
            Self::Nopass => "nopass",
        }
    }
}

#[derive(Clone, Copy)]
enum Ecc {
    L,
    M,
    Q,
    H,
}

impl Ecc {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "L" => Some(Self::L),
            "M" => Some(Self::M),
            "Q" => Some(Self::Q),
            "H" => Some(Self::H),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::L => "L",
            Self::M => "M",
            Self::Q => "Q",
            Self::H => "H",
        }
    }

    fn table_index(self) -> usize {
        match self {
            Self::L => 0,
            Self::M => 1,
            Self::Q => 2,
            Self::H => 3,
        }
    }

    fn format_bits(self, mask: usize) -> u16 {
        let bits: u16 = match self {
            Self::L => 0b01,
            Self::M => 0b00,
            Self::Q => 0b11,
            Self::H => 0b10,
        };
        let data = (bits << 3) | u16::try_from(mask).expect("mask fits u16");
        let mut remainder = data;
        for _ in 0..10 {
            remainder = (remainder << 1) ^ ((remainder >> 9) * 0x537);
        }
        ((data << 10) | remainder) ^ 0x5412
    }
}

struct Args {
    command: Command,
    text: Option<String>,
    error_correction: Ecc,
    quiet_zone: usize,
    format: Format,
    mode: Option<Mode>,
    version: Option<usize>,
    split_mode: SplitMode,
    output: Option<PathBuf>,
    wifi_ssid: Option<String>,
    wifi_auth: Option<WifiAuth>,
    wifi_hidden: bool,
}

impl Args {
    // Defaults live here, next to the flag handling.
    fn parse(argv: &[String]) -> Result<Self, String> {
        let mut text = None;
        let mut error_correction = Ecc::L;
        let mut quiet_zone = 2;
        let mut format = Format::Terminal;
        let mut mode = None;
        let mut version = None;
        let mut split_mode = SplitMode::All;
        let mut output = None;
        let mut command = Command::Text;
        let mut wifi_ssid = None;
        let mut wifi_auth = None;
        let mut wifi_hidden = false;
        let mut i = 0;
        while i < argv.len() {
            let flag = argv[i].as_str();
            if flag == "wifi" && command == Command::Text {
                command = Command::Wifi;
                i += 1;
                continue;
            }
            match flag {
                "--text" => {
                    i += 1;
                    text = Some(take_value(argv, i, flag)?);
                }
                "-l" | "--error-correction" => {
                    i += 1;
                    let value = take_value(argv, i, flag)?;
                    error_correction = parse_flag_value(flag, &value, Ecc::parse)?;
                }
                "-q" | "--quiet-zone" => {
                    i += 1;
                    let value = take_value(argv, i, flag)?;
                    quiet_zone = value
                        .parse()
                        .map_err(|_| format!("invalid {flag}: {value}"))?;
                }
                "-f" | "--format" => {
                    i += 1;
                    let value = take_value(argv, i, flag)?;
                    format = parse_flag_value(flag, &value, Format::parse)?;
                }
                "--mode" => {
                    i += 1;
                    let value = take_value(argv, i, flag)?;
                    mode = if value == "auto" {
                        None
                    } else {
                        Some(parse_flag_value(flag, &value, Mode::parse)?)
                    };
                }
                "--version" => {
                    i += 1;
                    let value = take_value(argv, i, flag)?;
                    version = Some(parse_version_value(flag, &value)?);
                }
                "--split-mode" => {
                    i += 1;
                    let value = take_value(argv, i, flag)?;
                    split_mode = SplitMode::parse(&value)
                        .ok_or_else(|| format!("invalid {flag}: {value}"))?;
                }
                "--output" => {
                    i += 1;
                    output = Some(PathBuf::from(take_value(argv, i, flag)?));
                }
                "--ssid" => {
                    i += 1;
                    wifi_ssid = Some(take_value(argv, i, flag)?);
                }
                "--auth" => {
                    i += 1;
                    let value = take_value(argv, i, flag)?;
                    wifi_auth = Some(
                        WifiAuth::parse(&value)
                            .ok_or_else(|| format!("invalid {flag}: {value}"))?,
                    );
                }
                "--hidden" => wifi_hidden = true,
                "-h" | "--help" => return Err(help_usage(command).to_string()),
                other => return Err(format!("unknown argument: {other}")),
            }
            i += 1;
        }
        let parsed = Self {
            command,
            text,
            error_correction,
            quiet_zone,
            format,
            mode,
            version,
            split_mode,
            output,
            wifi_ssid,
            wifi_auth,
            wifi_hidden,
        };
        parsed.validate()?;
        Ok(parsed)
    }

    fn validate(&self) -> Result<(), String> {
        match self.command {
            Command::Wifi => {
                if self.text.is_some() {
                    return Err("--text cannot be combined with the wifi subcommand".to_string());
                }
                if self.mode.is_some() {
                    return Err("--mode cannot be combined with the wifi subcommand".to_string());
                }
                if self.wifi_ssid.is_none() {
                    return Err("the wifi subcommand requires --ssid".to_string());
                }
            }
            Command::Text => {
                if self.wifi_ssid.is_some() || self.wifi_auth.is_some() || self.wifi_hidden {
                    return Err(
                        "--ssid, --auth and --hidden require the wifi subcommand".to_string()
                    );
                }
            }
        }
        Ok(())
    }

    fn run(&self, out: &mut dyn Write) -> Result<i32, String> {
        let payload = self.payload_text()?;
        let requested_mode = match self.command {
            Command::Wifi => Some(Mode::Byte),
            Command::Text => self.mode,
        };
        let codes = build_codes(
            &payload,
            self.error_correction,
            requested_mode,
            self.version,
            self.split_mode,
        )?;
        let rendered: Vec<RenderedCode> = codes
            .iter()
            .map(|code| match self.format {
                Format::Terminal => {
                    RenderedCode::Text(render_terminal(code.rows(), self.quiet_zone))
                }
                Format::Bits => RenderedCode::Text(render_bits(code.rows())),
                Format::Ascii => RenderedCode::Text(render_ascii(code.rows(), self.quiet_zone)),
                Format::Svg => RenderedCode::Text(render_svg(code.rows(), self.quiet_zone, 10)),
                Format::Png => RenderedCode::Binary(render_png(code.rows(), self.quiet_zone, 10)),
            })
            .collect();
        self.write_rendered(&rendered, out)
            .map_err(|error| error.to_string())?;
        Ok(0)
    }

    fn payload_text(&self) -> Result<String, String> {
        match self.command {
            Command::Text => {
                if let Some(text) = &self.text {
                    return Ok(text.clone());
                }
                let mut buffer = String::new();
                std::io::stdin()
                    .read_to_string(&mut buffer)
                    .map_err(|error| error.to_string())?;
                Ok(buffer)
            }
            Command::Wifi => {
                let ssid = self.wifi_ssid.as_deref().expect("validated by parse");
                let auth = self.wifi_auth.unwrap_or(WifiAuth::Wpa);
                let password = if auth == WifiAuth::Nopass {
                    String::new()
                } else {
                    read_wifi_password()
                };
                Ok(wifi_payload(ssid, auth, self.wifi_hidden, &password))
            }
        }
    }

    fn write_rendered(
        &self,
        rendered: &[RenderedCode],
        out: &mut dyn Write,
    ) -> std::io::Result<()> {
        if let Some(base) = &self.output {
            let paths = write_to_files(rendered, base, self.format.extension())?;
            for path in paths {
                eprintln!("wrote {}", path.display());
            }
            return Ok(());
        }
        match self.format {
            Format::Png => self.write_pngs(rendered, out),
            _ => self.write_texts(rendered, out),
        }
    }

    fn write_pngs(&self, rendered: &[RenderedCode], out: &mut dyn Write) -> std::io::Result<()> {
        if let (1, [RenderedCode::Binary(bytes)]) = (rendered.len(), rendered) {
            return out.write_all(bytes);
        }
        if self.split_mode == SplitMode::Wait {
            for (index, item) in rendered.iter().enumerate() {
                if index > 0 {
                    wait_for_next_code();
                }
                let RenderedCode::Binary(bytes) = item else {
                    continue;
                };
                out.write_all(bytes)?;
                out.flush()?;
            }
            return Ok(());
        }
        // standalone PNG files cannot share one stdout stream; number them instead
        let paths = write_to_files(rendered, Path::new("qr"), self.format.extension())?;
        for path in paths {
            eprintln!("wrote {}", path.display());
        }
        Ok(())
    }

    fn write_texts(&self, rendered: &[RenderedCode], out: &mut dyn Write) -> std::io::Result<()> {
        let texts: Vec<&str> = rendered
            .iter()
            .map(|item| match item {
                RenderedCode::Text(text) => text.as_str(),
                RenderedCode::Binary(_) => unreachable!("text formats never render binaries"),
            })
            .collect();
        if self.split_mode == SplitMode::Wait && texts.len() > 1 {
            for (index, text) in texts.iter().enumerate() {
                if index > 0 {
                    wait_for_next_code();
                }
                writeln!(out, "{text}")?;
            }
            return Ok(());
        }
        writeln!(out, "{}", texts.join(self.format.multi_code_separator()))
    }
}

enum RenderedCode {
    Text(String),
    Binary(Vec<u8>),
}

fn write_to_files(
    rendered: &[RenderedCode],
    base: &Path,
    extension: &str,
) -> std::io::Result<Vec<PathBuf>> {
    let mut stem = base.to_path_buf();
    if stem
        .extension()
        .is_some_and(|existing| existing == extension)
    {
        stem.set_extension("");
    }
    let stem = stem.into_os_string();
    let mut paths = Vec::new();
    for (index, item) in rendered.iter().enumerate() {
        let mut name = stem.clone();
        if rendered.len() > 1 {
            name.push(format!("-{}", index + 1));
        }
        name.push(format!(".{extension}"));
        let path = PathBuf::from(name);
        match item {
            RenderedCode::Text(text) => std::fs::write(&path, text)?,
            RenderedCode::Binary(bytes) => std::fs::write(&path, bytes)?,
        }
        paths.push(path);
    }
    Ok(paths)
}

fn wait_for_next_code() {
    eprint!("Press Enter for next QR code...");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    match std::fs::File::open("/dev/tty") {
        Ok(file) => {
            let mut reader = std::io::BufReader::new(file);
            let _ = reader.read_line(&mut line);
        }
        Err(_) => {
            let _ = std::io::stdin().read_line(&mut line);
        }
    }
}

fn build_codes(
    payload: &str,
    ecc: Ecc,
    requested_mode: Option<Mode>,
    version: Option<usize>,
    split_mode: SplitMode,
) -> Result<Vec<QrCode>, String> {
    let segment = Segment::from_text(payload, requested_mode)?;
    if let Ok(code) = QrCode::from_segment(&segment, ecc, version) {
        return Ok(vec![code]);
    }
    if split_mode == SplitMode::Disabled {
        return Err(match version {
            Some(version) => VersionMeta::for_version(version, ecc).capacity_error(&segment),
            None => VersionMeta::for_version(40, ecc).capacity_error(&segment),
        });
    }
    let capacity = VersionMeta::for_version(version.unwrap_or(40), ecc).capacity(segment.mode);
    Ok(split_text(payload, segment.mode, capacity)
        .iter()
        .map(|chunk| {
            let chunk_segment = Segment::from_text(chunk, Some(segment.mode))
                .expect("chunk encodes in the full text's mode");
            QrCode::from_segment(&chunk_segment, ecc, version)
                .expect("chunk fits into the capacity it was split against")
        })
        .collect())
}

fn split_text(text: &str, mode: Mode, capacity: usize) -> Vec<String> {
    if mode != Mode::Byte {
        let characters: Vec<char> = text.chars().collect();
        return characters
            .chunks(capacity)
            .map(|chunk| chunk.iter().collect())
            .collect();
    }
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut size = 0;
    for character in text.chars() {
        let character_size = character.len_utf8();
        if !current.is_empty() && size + character_size > capacity {
            chunks.push(std::mem::take(&mut current));
            size = 0;
        }
        current.push(character);
        size += character_size;
    }
    if !current.is_empty() || chunks.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn help_usage(command: Command) -> &'static str {
    match command {
        Command::Wifi => WIFI_USAGE,
        Command::Text => USAGE,
    }
}

fn take_value(argv: &[String], index: usize, flag: &str) -> Result<String, String> {
    argv.get(index)
        .cloned()
        .ok_or_else(|| format!("argument {flag}: expected one argument"))
}

fn parse_flag_value<T>(
    flag: &str,
    value: &str,
    parser: fn(&str) -> Option<T>,
) -> Result<T, String> {
    parser(value).ok_or_else(|| format!("invalid {flag}: {value}"))
}

fn parse_version_value(flag: &str, value: &str) -> Result<usize, String> {
    let parsed: usize = value
        .parse()
        .map_err(|_| format!("invalid {flag}: {value}"))?;
    if !(1..=40).contains(&parsed) {
        return Err(format!("invalid {flag}: must be between 1 and 40"));
    }
    Ok(parsed)
}

fn escape(text: &str) -> String {
    let mut escaped = String::new();
    for character in text.chars() {
        if matches!(character, '\\' | ';' | ':' | ',') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn wifi_payload(ssid: &str, auth: WifiAuth, hidden: bool, password: &str) -> String {
    let mut payload = format!("WIFI:T:{};S:{};", auth.label(), escape(ssid));
    if auth != WifiAuth::Nopass {
        payload.push_str("P:");
        payload.push_str(&escape(password));
        payload.push(';');
    }
    if hidden {
        payload.push_str("H:true;");
    }
    payload.push(';');
    payload
}

fn stdin_is_tty() -> bool {
    matches!(
        std::fs::metadata("/dev/stdin"),
        Ok(metadata) if metadata.file_type().is_char_device()
    )
}

fn read_wifi_password() -> String {
    eprint!("Password: ");
    let _ = std::io::stderr().flush();
    let mut password = String::new();
    if stdin_is_tty() {
        if let Ok(tty) = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
        {
            let _ = std::process::Command::new("stty")
                .arg("-echo")
                .stdin(std::process::Stdio::from(
                    tty.try_clone().expect("tty cloneable"),
                ))
                .status();
            let mut reader = std::io::BufReader::new(&tty);
            let _ = reader.read_line(&mut password);
            let _ = std::process::Command::new("stty")
                .arg("echo")
                .stdin(std::process::Stdio::from(tty))
                .status();
            eprintln!();
        }
    } else {
        let _ = std::io::stdin().read_to_string(&mut password);
    }
    password.trim_end_matches('\n').to_string()
}

#[derive(Default)]
struct BitBuffer {
    bits: Vec<bool>,
}

impl BitBuffer {
    fn append(&mut self, value: usize, bit_count: usize) {
        for shift in (0..bit_count).rev() {
            self.bits.push((value >> shift) & 1 == 1);
        }
    }

    fn pad_to_byte(&mut self) {
        while !self.bits.len().is_multiple_of(8) {
            self.bits.push(false);
        }
    }

    fn as_codewords(&self) -> Vec<u8> {
        self.bits
            .chunks(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold(0u8, |accumulator, bit| (accumulator << 1) | u8::from(*bit))
            })
            .collect()
    }
}

fn gf_multiply(left: u8, right: u8) -> u8 {
    let mut product: u16 = 0;
    let mut left = u16::from(left);
    let mut right = u16::from(right);
    while right != 0 {
        if right & 1 != 0 {
            product ^= left;
        }
        left <<= 1;
        if left & 0x100 != 0 {
            left ^= 0x11D;
        }
        right >>= 1;
    }
    u8::try_from(product).expect("GF(256) products stay below 256")
}

fn rs_generator(degree: usize) -> Vec<u8> {
    let mut coefficients = vec![0u8; degree];
    coefficients[degree - 1] = 1;
    let mut root = 1u8;
    for _ in 0..degree {
        for index in 0..degree {
            coefficients[index] = gf_multiply(coefficients[index], root);
            if index + 1 < degree {
                coefficients[index] ^= coefficients[index + 1];
            }
        }
        root = gf_multiply(root, 2);
    }
    coefficients
}

fn rs_remainder(data: &[u8], degree: usize) -> Vec<u8> {
    let generator = rs_generator(degree);
    let mut result = vec![0u8; degree];
    for &value in data {
        let factor = value ^ result.remove(0);
        result.push(0);
        for (index, &coefficient) in generator.iter().enumerate() {
            result[index] ^= gf_multiply(coefficient, factor);
        }
    }
    result
}

// Indexed by [level][version - 1]; levels are ordered L, M, Q, H.
const ECC_CODEWORDS_PER_BLOCK: [[usize; 40]; 4] = [
    [
        7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30,
        30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28,
        28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    ],
    [
        13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30,
        30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30,
        30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
];

const ERROR_BLOCKS: [[usize; 40]; 4] = [
    [
        1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13,
        14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25,
    ],
    [
        1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23,
        25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49,
    ],
    [
        1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29,
        34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68,
    ],
    [
        1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35,
        37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81,
    ],
];

const ALIGNMENT_POSITIONS: [&[usize]; 41] = [
    &[],
    &[],
    &[6, 18],
    &[6, 22],
    &[6, 26],
    &[6, 30],
    &[6, 34],
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
    &[6, 30, 54],
    &[6, 32, 58],
    &[6, 34, 62],
    &[6, 26, 46, 66],
    &[6, 26, 48, 70],
    &[6, 26, 50, 74],
    &[6, 30, 54, 78],
    &[6, 30, 56, 82],
    &[6, 30, 58, 86],
    &[6, 34, 62, 90],
    &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98],
    &[6, 30, 54, 78, 102],
    &[6, 28, 54, 80, 106],
    &[6, 32, 58, 84, 110],
    &[6, 30, 58, 86, 114],
    &[6, 34, 62, 90, 118],
    &[6, 26, 50, 74, 98, 122],
    &[6, 30, 54, 78, 102, 126],
    &[6, 26, 52, 78, 104, 130],
    &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138],
    &[6, 30, 58, 86, 114, 142],
    &[6, 34, 62, 90, 118, 146],
    &[6, 30, 54, 78, 102, 126, 150],
    &[6, 24, 50, 76, 102, 128, 154],
    &[6, 28, 54, 80, 106, 132, 158],
    &[6, 32, 58, 84, 110, 136, 162],
    &[6, 26, 54, 82, 110, 138, 166],
    &[6, 30, 58, 86, 114, 142, 170],
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Numeric,
    Alphanumeric,
    Byte,
}

impl Mode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "numeric" => Some(Self::Numeric),
            "alphanumeric" => Some(Self::Alphanumeric),
            "byte" => Some(Self::Byte),
            _ => None,
        }
    }

    fn bits(self) -> usize {
        match self {
            Self::Numeric => 0b0001,
            Self::Alphanumeric => 0b0010,
            Self::Byte => 0b0100,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Numeric => "numeric",
            Self::Alphanumeric => "alphanumeric",
            Self::Byte => "byte",
        }
    }
}

const ALPHANUMERIC_BYTES: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

fn char_count_bits(mode: Mode, version: usize) -> usize {
    let group = if version <= 9 {
        0
    } else if version <= 26 {
        1
    } else {
        2
    };
    match mode {
        Mode::Numeric => [10, 12, 14][group],
        Mode::Alphanumeric => [9, 11, 13][group],
        Mode::Byte => [8, 16, 16][group],
    }
}

struct Segment {
    mode: Mode,
    character_count: usize,
    bits: Vec<bool>,
}

impl Segment {
    fn from_text(text: &str, requested: Option<Mode>) -> Result<Self, String> {
        match requested {
            None => Ok(Self::auto(text)),
            Some(Mode::Numeric) => {
                if !text.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err("text cannot be encoded in numeric mode".to_string());
                }
                Ok(Self::numeric(text))
            }
            Some(Mode::Alphanumeric) => {
                if !text.bytes().all(|byte| ALPHANUMERIC_BYTES.contains(&byte)) {
                    return Err("text cannot be encoded in alphanumeric mode".to_string());
                }
                Ok(Self::alphanumeric(text))
            }
            Some(Mode::Byte) => Ok(Self::byte(text)),
        }
    }

    fn auto(text: &str) -> Self {
        if text.bytes().all(|byte| byte.is_ascii_digit()) {
            Self::numeric(text)
        } else if text.bytes().all(|byte| ALPHANUMERIC_BYTES.contains(&byte)) {
            Self::alphanumeric(text)
        } else {
            Self::byte(text)
        }
    }

    fn numeric(text: &str) -> Self {
        let digits: Vec<usize> = text.bytes().map(|byte| usize::from(byte - b'0')).collect();
        let mut buffer = BitBuffer::default();
        for chunk in digits.chunks(3) {
            let value = chunk
                .iter()
                .fold(0, |accumulator, digit| accumulator * 10 + digit);
            buffer.append(value, chunk.len() * 3 + 1);
        }
        Self {
            mode: Mode::Numeric,
            character_count: digits.len(),
            bits: buffer.bits,
        }
    }

    fn alphanumeric(text: &str) -> Self {
        let values: Vec<usize> = text
            .bytes()
            .map(|byte| {
                ALPHANUMERIC_BYTES
                    .iter()
                    .position(|candidate| *candidate == byte)
                    .expect("caller validates the alphanumeric alphabet")
            })
            .collect();
        let mut buffer = BitBuffer::default();
        for pair in values.chunks(2) {
            if pair.len() == 2 {
                buffer.append(pair[0] * 45 + pair[1], 11);
            } else {
                buffer.append(pair[0], 6);
            }
        }
        Self {
            mode: Mode::Alphanumeric,
            character_count: values.len(),
            bits: buffer.bits,
        }
    }

    fn byte(text: &str) -> Self {
        let data = text.as_bytes();
        let mut buffer = BitBuffer::default();
        for &byte in data {
            buffer.append(usize::from(byte), 8);
        }
        Self {
            mode: Mode::Byte,
            character_count: data.len(),
            bits: buffer.bits,
        }
    }

    fn total_bits(&self, version: usize) -> Option<usize> {
        let count_bits = char_count_bits(self.mode, version);
        if self.character_count >= 1 << count_bits {
            return None;
        }
        Some(4 + count_bits + self.bits.len())
    }
}

fn raw_data_modules(version: usize) -> usize {
    assert!((1..=40).contains(&version));
    let mut result = (16 * version + 128) * version + 64;
    if version >= 2 {
        let alignment_count = version / 7 + 2;
        result -= (25 * alignment_count - 10) * alignment_count - 55;
    }
    if version >= 7 {
        result -= 36;
    }
    result
}

struct VersionMeta {
    version: usize,
    ecc: Ecc,
    error_codewords: usize,
    data_blocks: Vec<usize>,
    alignment_positions: &'static [usize],
}

impl VersionMeta {
    fn size(&self) -> usize {
        17 + self.version * 4
    }

    fn data_codewords(&self) -> usize {
        self.data_blocks.iter().sum()
    }

    fn capacity(&self, mode: Mode) -> usize {
        let available = self.data_codewords() * 8 - 4 - char_count_bits(mode, self.version);
        match mode {
            Mode::Numeric => {
                let (groups, remainder) = (available / 10, available % 10);
                groups * 3
                    + match remainder {
                        value if value >= 7 => 2,
                        value if value >= 4 => 1,
                        _ => 0,
                    }
            }
            Mode::Alphanumeric => {
                let (pairs, remainder) = (available / 11, available % 11);
                pairs * 2 + usize::from(remainder >= 6)
            }
            Mode::Byte => available / 8,
        }
    }

    fn for_version(version: usize, ecc: Ecc) -> Self {
        assert!((1..=40).contains(&version));
        let level_index = ecc.table_index();
        let raw_codewords = raw_data_modules(version) / 8;
        let error_codewords = ECC_CODEWORDS_PER_BLOCK[level_index][version - 1];
        let block_count = ERROR_BLOCKS[level_index][version - 1];
        let short_block_count = block_count - raw_codewords % block_count;
        let short_block_length = raw_codewords / block_count;
        let data_blocks = (0..block_count)
            .map(|block| {
                short_block_length - error_codewords + usize::from(block >= short_block_count)
            })
            .collect();
        Self {
            version,
            ecc,
            error_codewords,
            data_blocks,
            alignment_positions: ALIGNMENT_POSITIONS[version],
        }
    }

    fn for_segment(segment: &Segment, ecc: Ecc) -> Option<Self> {
        (1..=40)
            .map(|version| Self::for_version(version, ecc))
            .find(|metadata| {
                segment
                    .total_bits(metadata.version)
                    .is_some_and(|bits| bits <= metadata.data_codewords() * 8)
            })
    }

    fn capacity_error(&self, segment: &Segment) -> String {
        let unit = if segment.mode == Mode::Byte {
            "UTF-8 bytes"
        } else {
            "characters"
        };
        format!(
            "version {}-{} {} QR codes support at most {} {}",
            self.version,
            self.ecc.label(),
            segment.mode.label(),
            self.capacity(segment.mode),
            unit
        )
    }
}

struct QrCode {
    matrix: Vec<Vec<bool>>,
}

impl QrCode {
    fn from_segment(segment: &Segment, ecc: Ecc, version: Option<usize>) -> Result<Self, String> {
        let meta = match version {
            Some(version) => {
                let meta = VersionMeta::for_version(version, ecc);
                let fits = segment
                    .total_bits(meta.version)
                    .is_some_and(|bits| bits <= meta.data_codewords() * 8);
                if !fits {
                    return Err(meta.capacity_error(segment));
                }
                meta
            }
            None => VersionMeta::for_segment(segment, ecc)
                .ok_or_else(|| VersionMeta::for_version(40, ecc).capacity_error(segment))?,
        };
        let data_codewords = encode_segment(segment, &meta);
        let blocks = make_blocks(&data_codewords, &meta);
        let codewords = interleave_blocks(&blocks);
        Ok(Self {
            matrix: Matrix::build(&codeword_bits(&codewords), &meta),
        })
    }

    fn rows(&self) -> &[Vec<bool>] {
        &self.matrix
    }
}

fn encode_segment(segment: &Segment, meta: &VersionMeta) -> Vec<u8> {
    let mut buffer = BitBuffer::default();
    buffer.append(segment.mode.bits(), 4);
    buffer.append(
        segment.character_count,
        char_count_bits(segment.mode, meta.version),
    );
    for &bit in &segment.bits {
        buffer.append(usize::from(bit), 1);
    }
    let remaining = meta.data_codewords() * 8 - buffer.bits.len();
    buffer.append(0, remaining.min(4));
    buffer.pad_to_byte();

    let mut codewords = buffer.as_codewords();
    let pads = [0xEC, 0x11];
    let mut index = 0;
    while codewords.len() < meta.data_codewords() {
        codewords.push(pads[index % 2]);
        index += 1;
    }
    codewords
}

fn make_blocks(data: &[u8], meta: &VersionMeta) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut blocks = Vec::new();
    let mut offset = 0;
    for &block_size in &meta.data_blocks {
        let block = &data[offset..offset + block_size];
        blocks.push((block.to_vec(), rs_remainder(block, meta.error_codewords)));
        offset += block_size;
    }
    blocks
}

fn interleave(lists: &[Vec<u8>]) -> Vec<u8> {
    let max_length = lists.iter().map(Vec::len).max().unwrap_or(0);
    let mut result = Vec::new();
    for index in 0..max_length {
        for list in lists {
            if index < list.len() {
                result.push(list[index]);
            }
        }
    }
    result
}

fn interleave_blocks(blocks: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
    let data: Vec<Vec<u8>> = blocks.iter().map(|(data, _)| data.clone()).collect();
    let error: Vec<Vec<u8>> = blocks.iter().map(|(_, error)| error.clone()).collect();
    let mut result = interleave(&data);
    result.extend(interleave(&error));
    result
}

fn codeword_bits(codewords: &[u8]) -> Vec<bool> {
    codewords
        .iter()
        .flat_map(|codeword| (0..8).rev().map(move |shift| (codeword >> shift) & 1 == 1))
        .collect()
}

fn version_bits(version: usize) -> u32 {
    let base = u32::from(u8::try_from(version).expect("version fits u8")) << 12;
    let mut result = base;
    for shift in (12..18).rev() {
        if result & (1 << shift) != 0 {
            result ^= 0x1F25 << (shift - 12);
        }
    }
    base | result
}

const TIMING_ROW_COL: usize = 6;

const FORMAT_POSITIONS: [(usize, usize); 15] = [
    (0, 8),
    (1, 8),
    (2, 8),
    (3, 8),
    (4, 8),
    (5, 8),
    (7, 8),
    (8, 8),
    (8, 7),
    (8, 5),
    (8, 4),
    (8, 3),
    (8, 2),
    (8, 1),
    (8, 0),
];

fn mask(mask_id: usize, row: usize, col: usize) -> bool {
    match mask_id {
        0 => (row + col).is_multiple_of(2),
        _ => unreachable!("only mask 0 is used"),
    }
}

struct Matrix {
    modules: Vec<Vec<Option<bool>>>,
    reserved: Vec<Vec<bool>>,
}

impl Matrix {
    fn build(bits: &[bool], meta: &VersionMeta) -> Vec<Vec<bool>> {
        let size = meta.size();
        let mut matrix = Self {
            modules: vec![vec![None; size]; size],
            reserved: vec![vec![false; size]; size],
        };
        matrix.add_patterns(meta);
        matrix.add_data(bits);
        matrix.add_format(meta.ecc);
        matrix
            .modules
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|cell| cell.unwrap_or(false))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn set_reserved(&mut self, row: usize, col: usize, value: bool) {
        self.modules[row][col] = Some(value);
        self.reserved[row][col] = true;
    }

    fn add_patterns(&mut self, meta: &VersionMeta) {
        let size = meta.size();
        self.add_finder(0, 0);
        self.add_finder(0, size - 7);
        self.add_finder(size - 7, 0);
        for index in 8..size - 8 {
            self.set_reserved(TIMING_ROW_COL, index, index % 2 == 0);
            self.set_reserved(index, TIMING_ROW_COL, index % 2 == 0);
        }
        if let Some(&last) = meta.alignment_positions.last() {
            for &row in meta.alignment_positions {
                for &col in meta.alignment_positions {
                    let overlaps_finder = (row == TIMING_ROW_COL
                        && (col == TIMING_ROW_COL || col == last))
                        || (row == last && col == TIMING_ROW_COL);
                    if !overlaps_finder {
                        self.add_alignment(row, col);
                    }
                }
            }
        }
        self.reserve_format(size);
        if meta.version >= 7 {
            self.reserve_version(meta.version, size);
        }
        self.set_reserved(size - 8, 8, true);
    }

    fn add_finder(&mut self, row: usize, col: usize) {
        let size = self.modules.len();
        for dy in -1..8isize {
            let Some(current_row) = row.checked_add_signed(dy) else {
                continue;
            };
            if current_row >= size {
                continue;
            }
            for dx in -1..8isize {
                let Some(current_col) = col.checked_add_signed(dx) else {
                    continue;
                };
                if current_col >= size {
                    continue;
                }
                let inside = (0..=6).contains(&dx) && (0..=6).contains(&dy);
                let is_border = inside && (dx == 0 || dx == 6 || dy == 0 || dy == 6);
                let is_center = inside && (2..=4).contains(&dx) && (2..=4).contains(&dy);
                self.set_reserved(current_row, current_col, is_border || is_center);
            }
        }
    }

    fn add_alignment(&mut self, row: usize, col: usize) {
        let size = self.modules.len();
        for dy in -2..=2isize {
            for dx in -2..=2isize {
                let Some(current_row) = row.checked_add_signed(dy) else {
                    continue;
                };
                let Some(current_col) = col.checked_add_signed(dx) else {
                    continue;
                };
                if current_row >= size || current_col >= size {
                    continue;
                }
                let is_border = dx.abs() == 2 || dy.abs() == 2;
                let is_center = dx == 0 && dy == 0;
                self.set_reserved(current_row, current_col, is_border || is_center);
            }
        }
    }

    fn add_data(&mut self, bits: &[bool]) {
        let size = self.modules.len();
        let mut bit_index = 0;
        let mut upward = true;
        for base_col in (2..size).step_by(2).rev() {
            let right_col = if base_col <= TIMING_ROW_COL {
                base_col - 1
            } else {
                base_col
            };
            let rows: Vec<usize> = if upward {
                (0..size).rev().collect()
            } else {
                (0..size).collect()
            };
            for row in rows {
                for col in [right_col, right_col - 1] {
                    if self.reserved[row][col] {
                        continue;
                    }
                    let bit = bits.get(bit_index).copied().unwrap_or(false);
                    self.modules[row][col] = Some(bit ^ mask(0, row, col));
                    self.reserved[row][col] = true;
                    bit_index += 1;
                }
            }
            upward = !upward;
        }
    }

    fn add_format(&mut self, ecc: Ecc) {
        let size = self.modules.len();
        let format_bits = ecc.format_bits(0);
        let mut mirror_positions: Vec<(usize, usize)> =
            (0..8).map(|index| (8, size - 1 - index)).collect();
        mirror_positions.extend((8..15).map(|index| (size - 15 + index, 8)));
        for (index, &(row, col)) in FORMAT_POSITIONS.iter().enumerate() {
            let bit = format_bits & (1 << index) != 0;
            self.set_reserved(row, col, bit);
            let (mirror_row, mirror_col) = mirror_positions[index];
            self.set_reserved(mirror_row, mirror_col, bit);
        }
    }

    fn reserve_format(&mut self, size: usize) {
        let mut positions: Vec<(usize, usize)> = Vec::new();
        positions.extend((0..9).map(|col| (8, col)));
        positions.extend((0..9).map(|row| (row, 8)));
        positions.extend((0..7).map(|index| (size - 1 - index, 8)));
        positions.extend((0..8).map(|index| (8, size - 8 + index)));
        for (row, col) in positions {
            if !self.reserved[row][col] {
                self.set_reserved(row, col, false);
            }
        }
    }

    fn reserve_version(&mut self, version: usize, size: usize) {
        let bits = version_bits(version);
        for index in 0..18 {
            let bit = bits & (1 << index) != 0;
            let row = index / 3;
            let col = size - 11 + index % 3;
            self.set_reserved(row, col, bit);
            self.set_reserved(col, row, bit);
        }
    }
}

fn with_quiet_zone(rows: &[Vec<bool>], quiet_zone: usize) -> Vec<Vec<bool>> {
    let width = rows[0].len() + quiet_zone * 2;
    let blank = vec![false; width];
    let mut padded: Vec<Vec<bool>> = Vec::new();
    padded.extend(std::iter::repeat_n(blank.clone(), quiet_zone));
    for row in rows {
        let mut padded_row = vec![false; quiet_zone];
        padded_row.extend_from_slice(row);
        padded_row.extend(std::iter::repeat_n(false, quiet_zone));
        padded.push(padded_row);
    }
    padded.extend(std::iter::repeat_n(blank, quiet_zone));
    padded
}

fn render_terminal(rows: &[Vec<bool>], quiet_zone: usize) -> String {
    with_quiet_zone(rows, quiet_zone)
        .iter()
        .map(|row| {
            row.iter()
                .map(|&cell| if cell { ANSI_BLACK } else { ANSI_WHITE })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_bits(rows: &[Vec<bool>]) -> String {
    rows.iter()
        .map(|row| {
            row.iter()
                .map(|cell| u8::from(*cell).to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_ascii(rows: &[Vec<bool>], quiet_zone: usize) -> String {
    with_quiet_zone(rows, quiet_zone)
        .iter()
        .map(|row| {
            row.iter()
                .map(|&cell| if cell { '#' } else { ' ' })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_svg(rows: &[Vec<bool>], quiet_zone: usize, scale: usize) -> String {
    let modules = with_quiet_zone(rows, quiet_zone);
    let pixel_size = modules.len() * scale;
    let mut rects = String::new();
    for (row, line) in modules.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            if cell {
                write!(
                    rects,
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
                    col * scale,
                    row * scale,
                    scale,
                    scale
                )
                .expect("writing to a String is infallible");
            }
        }
    }
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{pixel_size}\" height=\"\
         {pixel_size}\" viewBox=\"0 0 {pixel_size} {pixel_size}\" shape-rendering=\"crispEdges\">\
         <rect width=\"{pixel_size}\" height=\"{pixel_size}\" fill=\"#fff\"/>\
         <g fill=\"#000\">{rects}</g></svg>"
    )
}

fn scaled_pixels(rows: &[Vec<bool>], quiet_zone: usize, scale: usize) -> Vec<Vec<bool>> {
    with_quiet_zone(rows, quiet_zone)
        .into_iter()
        .flat_map(|line| std::iter::repeat_n(line, scale))
        .map(|line| {
            line.into_iter()
                .flat_map(|cell| std::iter::repeat_n(cell, scale))
                .collect()
        })
        .collect()
}

fn crc32(kind: &[u8], data: &[u8]) -> u32 {
    fn update(crc: u32, byte: u8) -> u32 {
        let mut crc = crc ^ u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
        crc
    }
    kind.iter()
        .chain(data)
        .fold(0xFFFF_FFFF, |crc, byte| update(crc, *byte))
        ^ 0xFFFF_FFFF
}

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn zlib_compress_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let push_block = |out: &mut Vec<u8>, block: &[u8], final_block: bool| {
        out.push(u8::from(final_block));
        let length = u16::try_from(block.len()).expect("stored blocks cap at 65535 bytes");
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&(!length).to_le_bytes());
        out.extend_from_slice(block);
    };
    if data.is_empty() {
        push_block(&mut out, &[], true);
    }
    for offset in (0..data.len()).step_by(65535) {
        let end = (offset + 65535).min(data.len());
        push_block(&mut out, &data[offset..end], end == data.len());
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn png_chunk(kind: &[u8], data: &[u8]) -> Vec<u8> {
    let length = u32::try_from(data.len()).expect("chunk lengths fit u32");
    let mut chunk = Vec::with_capacity(12 + data.len());
    chunk.extend_from_slice(&length.to_be_bytes());
    chunk.extend_from_slice(kind);
    chunk.extend_from_slice(data);
    chunk.extend_from_slice(&crc32(kind, data).to_be_bytes());
    chunk
}

fn render_png(rows: &[Vec<bool>], quiet_zone: usize, scale: usize) -> Vec<u8> {
    let pixels = scaled_pixels(rows, quiet_zone, scale);
    let height = pixels.len();
    let width = pixels[0].len();
    let mut scanlines = Vec::new();
    for row in &pixels {
        scanlines.push(0);
        for &cell in row {
            scanlines.extend_from_slice(if cell { &PNG_BLACK } else { &PNG_WHITE });
        }
    }
    let mut header = [0u8; 13];
    header[0..4].copy_from_slice(
        &u32::try_from(width)
            .expect("image width fits u32")
            .to_be_bytes(),
    );
    header[4..8].copy_from_slice(
        &u32::try_from(height)
            .expect("image height fits u32")
            .to_be_bytes(),
    );
    header[8] = 8;
    header[9] = 2;
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend(png_chunk(b"IHDR", &header));
    png.extend(png_chunk(b"IDAT", &zlib_compress_stored(&scanlines)));
    png.extend(png_chunk(b"IEND", &[]));
    png
}

fn main() -> std::process::ExitCode {
    use std::process::ExitCode;

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let parsed = match Args::parse(&argv) {
        Ok(parsed) => parsed,
        Err(message) if message.starts_with("usage:") => {
            println!("{message}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("qr: {message}");
            return ExitCode::from(2);
        }
    };
    match parsed.run(&mut std::io::stdout().lock()) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            eprintln!("qr: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        escape, gf_multiply, interleave_blocks, rs_generator, rs_remainder, version_bits,
        wifi_payload, write_to_files, Args, Command, Ecc, Format, Mode, QrCode, Segment, SplitMode,
        WifiAuth,
    };

    fn run(args: &Args) -> String {
        let mut buffer = Vec::new();
        assert_eq!(args.run(&mut buffer).expect("runs"), 0);
        String::from_utf8(buffer).expect("utf8")
    }

    #[test]
    fn builds_wifi_payloads() {
        assert_eq!(
            escape(r"semi;colon\back:slash,comma"),
            r"semi\;colon\\back\:slash\,comma"
        );
        assert_eq!(
            wifi_payload("Cafe;Net", WifiAuth::Wpa, false, "secret"),
            r"WIFI:T:WPA;S:Cafe\;Net;P:secret;;"
        );
        assert_eq!(
            wifi_payload("Cafe", WifiAuth::Nopass, false, "ignored"),
            "WIFI:T:nopass;S:Cafe;;"
        );
        assert_eq!(
            wifi_payload("Net", WifiAuth::Wep, true, "pw"),
            "WIFI:T:WEP;S:Net;P:pw;H:true;;"
        );
    }

    #[test]
    fn parses_the_wifi_subcommand() {
        let argv: Vec<String> = ["wifi", "--ssid", "Cafe"]
            .iter()
            .map(ToString::to_string)
            .collect();
        let parsed = Args::parse(&argv).expect("parses");
        assert_eq!(parsed.command, Command::Wifi);
        assert_eq!(parsed.wifi_ssid.as_deref(), Some("Cafe"));
        assert_eq!(parsed.wifi_auth, None);
        assert!(!parsed.wifi_hidden);
        let argv: Vec<String> = [
            "wifi",
            "--ssid",
            "S",
            "--auth",
            "nopass",
            "--hidden",
            "-l",
            "H",
            "--version",
            "3",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        let parsed = Args::parse(&argv).expect("parses");
        assert_eq!(parsed.command, Command::Wifi);
        assert_eq!(parsed.wifi_auth, Some(WifiAuth::Nopass));
        assert!(parsed.wifi_hidden);
        assert_eq!(parsed.version, Some(3));
        assert!(Args::parse(&["wifi".to_string()]).is_err());
        assert!(
            Args::parse(&["wifi".to_string(), "--text".to_string(), "hi".to_string()]).is_err()
        );
        assert!(Args::parse(&["--ssid".to_string(), "Cafe".to_string()]).is_err());
        assert!(Args::parse(&[
            "--text".to_string(),
            "A".to_string(),
            "--hidden".to_string()
        ])
        .is_err());
        assert!(
            Args::parse(&["wifi".to_string(), "--auth".to_string(), "wpa2".to_string()]).is_err()
        );
    }

    #[test]
    fn multiplies_in_galois_field() {
        assert_eq!(gf_multiply(0x57, 0x83), 49);
    }

    #[test]
    fn builds_reed_solomon_generators() {
        assert_eq!(rs_generator(3), vec![7, 14, 8]);
    }

    #[test]
    fn computes_reed_solomon_remainders() {
        assert_eq!(
            rs_remainder(&[32, 91, 11, 120, 209, 114, 220], 7),
            vec![255, 198, 226, 122, 164, 250, 136]
        );
        assert_eq!(
            rs_remainder(
                &[
                    64, 198, 23, 54, 70, 102, 23, 54, 70, 102, 23, 54, 70, 96, 236, 17, 236, 17,
                    236
                ],
                7
            ),
            vec![221, 150, 217, 99, 43, 41, 158]
        );
    }

    #[test]
    fn interleaves_blocks_column_wise() {
        let blocks = vec![(vec![1, 2], vec![10, 11]), (vec![3], vec![12, 13])];
        assert_eq!(interleave_blocks(&blocks), vec![1, 3, 2, 10, 12, 11, 13]);
    }

    #[test]
    fn computes_format_and_version_bits() {
        let expectations = [
            (Ecc::L, 30660),
            (Ecc::M, 21522),
            (Ecc::Q, 13663),
            (Ecc::H, 5769),
        ];
        for (level, expected) in expectations {
            assert_eq!(level.format_bits(0), expected);
        }
        assert_eq!(version_bits(7), 31_892);
    }

    #[test]
    fn picks_smallest_fitting_version() {
        assert_eq!(
            QrCode::from_segment(&Segment::auto("A"), Ecc::L, None)
                .expect("fits")
                .rows()
                .len(),
            21
        );
        assert_eq!(
            QrCode::from_segment(&Segment::auto(&"1".repeat(42)), Ecc::L, None)
                .expect("fits")
                .rows()
                .len(),
            25
        );
    }

    #[test]
    fn rejects_oversized_payloads() {
        let Err(error) = QrCode::from_segment(&Segment::auto(&"x".repeat(2954)), Ecc::L, None)
        else {
            panic!("oversized payload must fail")
        };
        assert_eq!(
            error,
            "version 40-L byte QR codes support at most 2953 UTF-8 bytes"
        );
    }

    #[test]
    fn renders_bit_matrix() {
        let args = Args {
            text: Some("A".to_string()),
            error_correction: Ecc::L,
            quiet_zone: 0,
            format: Format::Bits,
            mode: None,
            version: None,
            split_mode: SplitMode::All,
            output: None,
            command: Command::Text,
            wifi_ssid: None,
            wifi_auth: None,
            wifi_hidden: false,
        };
        let first_line = run(&args).lines().next().expect("non-empty").to_string();
        assert_eq!(first_line, "111111100101101111111");
    }

    #[test]
    fn renders_ascii() {
        let args = Args {
            text: Some("A".to_string()),
            error_correction: Ecc::L,
            quiet_zone: 0,
            format: Format::Ascii,
            mode: None,
            version: None,
            split_mode: SplitMode::All,
            output: None,
            command: Command::Text,
            wifi_ssid: None,
            wifi_auth: None,
            wifi_hidden: false,
        };
        let first_line = run(&args).lines().next().expect("non-empty").to_string();
        assert_eq!(first_line, "#######  # ## #######");
    }

    #[test]
    fn renders_svg_and_png_headers() {
        let svg_args = Args {
            text: Some("A".to_string()),
            error_correction: Ecc::L,
            quiet_zone: 0,
            format: Format::Svg,
            mode: None,
            version: None,
            split_mode: SplitMode::All,
            output: None,
            command: Command::Text,
            wifi_ssid: None,
            wifi_auth: None,
            wifi_hidden: false,
        };
        assert!(run(&svg_args).starts_with("<svg "));
        let png_args = Args {
            text: Some("A".to_string()),
            error_correction: Ecc::L,
            quiet_zone: 0,
            format: Format::Png,
            mode: None,
            version: None,
            split_mode: SplitMode::All,
            output: None,
            command: Command::Text,
            wifi_ssid: None,
            wifi_auth: None,
            wifi_hidden: false,
        };
        let mut buffer = Vec::new();
        assert_eq!(png_args.run(&mut buffer).expect("runs"), 0);
        assert_eq!(&buffer[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn splits_oversized_payloads_into_multiple_codes() {
        let args = Args {
            text: Some("x".repeat(3000)),
            error_correction: Ecc::L,
            quiet_zone: 0,
            format: Format::Bits,
            mode: None,
            version: None,
            split_mode: SplitMode::All,
            output: None,
            command: Command::Text,
            wifi_ssid: None,
            wifi_auth: None,
            wifi_hidden: false,
        };
        let output = run(&args);
        assert_eq!(output.trim_end().split("\n\n").count(), 2);
    }

    #[test]
    fn parses_arguments() {
        let argv: Vec<String> = [
            "--text",
            "hi",
            "-l",
            "H",
            "-q",
            "3",
            "--format",
            "bits",
            "--mode",
            "byte",
            "--version",
            "10",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        let parsed = Args::parse(&argv).expect("parses");
        assert_eq!(parsed.text.as_deref(), Some("hi"));
        assert!(matches!(parsed.error_correction, Ecc::H));
        assert_eq!(parsed.quiet_zone, 3);
        assert!(matches!(parsed.format, Format::Bits));
        assert_eq!(parsed.mode, Some(Mode::Byte));
        assert_eq!(parsed.version, Some(10));
        assert!(Args::parse(&["--bogus".to_string()]).is_err());
        assert!(Args::parse(&["--format".to_string(), "nope".to_string()]).is_err());
        assert!(Args::parse(&["-q".to_string(), "-1".to_string()]).is_err());
        assert!(Args::parse(&["--mode".to_string(), "kanji".to_string()]).is_err());
        assert!(Args::parse(&["--version".to_string(), "0".to_string()]).is_err());
        assert!(Args::parse(&["--version".to_string(), "41".to_string()]).is_err());
        let argv: Vec<String> = ["--split-mode", "wait", "--output", "/tmp/qr-out"]
            .iter()
            .map(ToString::to_string)
            .collect();
        let parsed = Args::parse(&argv).expect("parses");
        assert_eq!(parsed.split_mode, SplitMode::Wait);
        assert_eq!(
            parsed.output.as_deref(),
            Some(std::path::Path::new("/tmp/qr-out"))
        );
        assert!(Args::parse(&["--split-mode".to_string(), "sometimes".to_string()]).is_err());
    }

    #[test]
    fn shows_subcommand_specific_help() {
        let Err(general) = Args::parse(&["--help".to_string()]) else {
            panic!("help must fail parsing")
        };
        assert!(general.starts_with("usage: ./qr.rs [OPTIONS]"));
        assert!(!general.contains("--auth"));
        assert!(!general.contains("--hidden"));
        let Err(wifi) = Args::parse(&["wifi".to_string(), "--help".to_string()]) else {
            panic!("help must fail parsing")
        };
        assert!(wifi.starts_with("usage: ./qr.rs wifi --ssid SSID"));
        assert!(wifi.contains("--auth <AUTH>                  WPA | WEP | nopass"));
        // help requested after the subcommand token wins over the general one
        let argv: Vec<String> = ["-l", "H", "wifi", "--hidden", "--help"]
            .iter()
            .map(ToString::to_string)
            .collect();
        let Err(wifi) = Args::parse(&argv) else {
            panic!("help must fail parsing")
        };
        assert!(wifi.starts_with("usage: ./qr.rs wifi"));
    }

    #[test]
    fn rejects_split_disabled_when_oversized() {
        let args = Args {
            text: Some("x".repeat(35)),
            error_correction: Ecc::L,
            quiet_zone: 0,
            format: Format::Bits,
            mode: Some(Mode::Byte),
            version: Some(1),
            split_mode: SplitMode::Disabled,
            output: None,
            command: Command::Text,
            wifi_ssid: None,
            wifi_auth: None,
            wifi_hidden: false,
        };
        let error = args.run(&mut Vec::new()).expect_err("must fail");
        assert!(error.contains("support at most 17 UTF-8 bytes"));
    }

    #[test]
    fn numbers_files_for_multiple_codes() {
        let rendered = vec![
            super::RenderedCode::Binary(b"\x89PNG\r\n\x1a\none".to_vec()),
            super::RenderedCode::Binary(b"\x89PNG\r\n\x1a\ntwo".to_vec()),
        ];
        let base = std::env::temp_dir().join("qr_rs_test_numbered");
        let paths = write_to_files(&rendered, &base, "png").expect("writes");
        assert_eq!(paths.len(), 2);
        assert!(paths[0]
            .to_string_lossy()
            .ends_with("qr_rs_test_numbered-1.png"));
        let first = std::fs::read(&paths[0]).expect("readable");
        let second = std::fs::read(&paths[1]).expect("readable");
        assert_ne!(first, second);
        for path in &paths {
            std::fs::remove_file(path).expect("cleanup");
        }
    }

    #[test]
    fn single_code_with_output_writes_unnumbered_file() {
        let rendered = vec![super::RenderedCode::Text("matrix".to_string())];
        let base = std::env::temp_dir().join("qr_rs_test_single.svg");
        let paths = write_to_files(&rendered, &base, "svg").expect("writes");
        assert_eq!(paths.len(), 1);
        assert!(paths[0]
            .to_string_lossy()
            .ends_with("qr_rs_test_single.svg"));
        std::fs::remove_file(&paths[0]).expect("cleanup");
    }

    #[test]
    fn pins_the_requested_version() {
        let segment = Segment::auto("A");
        let code = QrCode::from_segment(&segment, Ecc::L, Some(10)).expect("fits into version 10");
        assert_eq!(code.rows().len(), 57);
        let Err(error) = QrCode::from_segment(&Segment::auto(&"x".repeat(18)), Ecc::L, Some(1))
        else {
            panic!("payload does not fit into version 1")
        };
        assert_eq!(
            error,
            "version 1-L byte QR codes support at most 17 UTF-8 bytes"
        );
    }

    #[test]
    fn validates_forced_modes() {
        assert!(Segment::from_text("123", Some(Mode::Numeric)).is_ok());
        let Err(error) = Segment::from_text("abc", Some(Mode::Numeric)) else {
            panic!("not digits")
        };
        assert_eq!(error, "text cannot be encoded in numeric mode");
        let Err(error) = Segment::from_text("héllo", Some(Mode::Alphanumeric)) else {
            panic!("not alphanumeric")
        };
        assert_eq!(error, "text cannot be encoded in alphanumeric mode");
        assert!(Segment::from_text("anything \n ünicode", Some(Mode::Byte)).is_ok());
    }

    #[test]
    fn forced_mode_changes_the_encoding() {
        let digits = "1234567890";
        let auto = QrCode::from_segment(
            &Segment::from_text(digits, None).expect("auto"),
            Ecc::L,
            None,
        )
        .expect("fits");
        let forced = QrCode::from_segment(
            &Segment::from_text(digits, Some(Mode::Byte)).expect("byte"),
            Ecc::L,
            None,
        )
        .expect("fits");
        assert_ne!(auto.rows(), forced.rows());
    }

    #[test]
    fn splits_against_the_pinned_version_capacity() {
        let args = Args {
            text: Some("x".repeat(35)),
            error_correction: Ecc::L,
            quiet_zone: 0,
            format: Format::Bits,
            mode: Some(Mode::Byte),
            version: Some(1),
            split_mode: SplitMode::All,
            output: None,
            command: Command::Text,
            wifi_ssid: None,
            wifi_auth: None,
            wifi_hidden: false,
        };
        // version 1-L byte capacity is 17; 35 bytes split into 17 + 17 + 1
        let output = run(&args);
        assert_eq!(output.trim_end().split("\n\n").count(), 3);
    }
}
