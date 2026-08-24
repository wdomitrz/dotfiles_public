#!/usr/bin/env rscript.sh
// Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
// AGPL License

//# dependencies:
//# serde_json = { version = "1", features = ["preserve_order"] }

//! Wrapper around `i3status` speaking the i3bar protocol.
//!
//! Runs `i3status`, decorates every status line with an extra media block
//! (via `playerctl`) and a no-internet marker, appends a few clickable
//! launcher blocks, and reacts to i3bar click events by executing the
//! matching commands directly (no shell involved).
//!
//! Protocol gotcha: both i3status updates and i3bar click events are
//! elements of an outer JSON array split across lines, so continuation lines
//! carry a *leading* comma which must be stripped before parsing.

use serde_json::{json, Value};
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, ExitCode, Stdio};
use std::thread;

// Configuration

const USAGE: &str = "usage: ./i3status_wrapper.rs [-h | --help]";
const PROGRAM_NAME: &str = "i3status_wrapper";
const I3STATUS_ARGV: [&str; 1] = ["i3status"];

/// Sent after blocking click actions so i3status repaints immediately.
const REFRESH_STATUS_ARGV: [&str; 3] = ["killall", "-SIGUSR1", "i3status"];
const HEADER_LINE: &str = r#"{"version":1,"click_events":true}"#;

/// Launcher blocks appended to every status line.
const TRAILING_BLOCKS: &[(&str, &str)] = &[("terminal", "📄"), ("menu", "🔍"), ("close", "❌")];

/// Block name -> (argv, refresh).
const CLICK_COMMANDS: &[(&str, &[&str], bool)] = &[
    ("close", &["i3-msg", "kill"], false),
    ("editor", &["code"], false),
    ("menu", &["run_menu.sh"], false),
    ("terminal", &["i3-sensible-terminal"], false),
    ("next_track", &["playerctl", "next"], true),
    ("pause", &["playerctl", "pause"], true),
    ("play_pause", &["playerctl", "play-pause"], true),
    ("play", &["playerctl", "play"], true),
    ("previous_track", &["playerctl", "previous"], true),
    ("stop", &["playerctl", "stop"], true),
    ("media_info", &["playerctl", "play-pause"], true),
    ("battery", &["xfce4-power-manager-settings"], false),
    ("cpu_temperature", &["xfce4-taskmanager"], false),
    ("disk_info", &["nautilus"], false),
    ("ethernet", &["nm-connection-editor"], false),
    ("ipv6", &["nm-connection-editor"], false),
    ("load", &["xfce4-taskmanager"], false),
    ("time", &["xdg-open", "https://calendar.google.com/"], false),
    (
        "tztime",
        &["xdg-open", "https://calendar.google.com/"],
        false,
    ),
    ("wireless", &["nm-connection-editor"], false),
];

// Process helpers

/// Spawns `argv` with no inherited descriptors; `stdout` decides whether the
/// child's output is captured or discarded.
fn spawn_child<I, S>(argv: I, stdout: Stdio) -> std::io::Result<Child>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut argv = argv.into_iter();
    let program = argv.next().expect("non-empty argv");
    Command::new(program)
        .args(argv)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(Stdio::null())
        .spawn()
}

/// Runs `argv` to completion, reporting whether it exited successfully.
fn run_to_completion<S: AsRef<OsStr> + Clone>(argv: &[S]) -> bool {
    spawn_child(argv.iter().cloned(), Stdio::null())
        .and_then(|mut child| child.wait())
        .is_ok_and(|status| status.success())
}

/// Runs `argv`, returning its stdout if it exited successfully.
fn capture_stdout<const N: usize>(argv: [&str; N]) -> Option<String> {
    let (program, arguments) = argv.split_first()?;
    let output = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

// Status line processing

fn parse_status_line(line: &str) -> Option<Vec<Value>> {
    serde_json::from_str(line.trim().trim_matches(',')).ok()
}

/// Media block first, then the decorated i3status blocks, then the launchers.
fn process_blocks(media_block: Value, old_blocks: Vec<Value>) -> Vec<Value> {
    let mut blocks = vec![media_block];
    blocks.extend(add_no_internet_info(old_blocks));
    blocks.extend(
        TRAILING_BLOCKS
            .iter()
            .map(|(name, full_text)| block(name, full_text)),
    );
    blocks
}

/// Marks the first internet block with ⛔ unless one of them already shows
/// any text (a null `full_text` counts as absent).
fn add_no_internet_info(mut blocks: Vec<Value>) -> Vec<Value> {
    let online = blocks.iter().any(|block| {
        is_internet_block(block)
            && block
                .get("full_text")
                .is_some_and(|full_text| !full_text.is_null())
    });
    if online {
        return blocks;
    }
    if let Some(marked) = blocks.iter_mut().find(|block| is_internet_block(block)) {
        marked["full_text"] = Value::from("⛔");
    }
    blocks
}

fn is_internet_block(block: &Value) -> bool {
    matches!(
        block.get("name").and_then(Value::as_str),
        Some("ipv6" | "wireless" | "ethernet")
    )
}

fn block(name: &str, full_text: &str) -> Value {
    json!({"name": name, "full_text": full_text})
}

/// Queries `playerctl`; falls back to an empty block when it reports nothing.
fn media_block() -> Value {
    let title = capture_stdout(["playerctl", "metadata", "title"]);
    let status = capture_stdout(["playerctl", "status"]);
    render_media_block(title.as_deref(), status.as_deref())
}

fn render_media_block(title: Option<&str>, status: Option<&str>) -> Value {
    let (Some(title), Some(status)) = (title.map(str::trim), status.map(str::trim)) else {
        return block("media_info", "");
    };
    if title.is_empty() && status.is_empty() {
        return block("media_info", "");
    }
    let playing = status == "Playing";
    let icon = if playing { "⏸️" } else { "▶️" };
    let color = if playing { "#BBFFBB" } else { "#BBBBFF" };
    json!({
        "name": "media_info",
        "full_text": format!("{title} {icon}"),
        "color": color,
    })
}

// Click handling

/// What to do on a click: `refresh` actions run to completion and then poke
/// i3status, so the bar reflects their effect immediately; the rest detach.
#[derive(Debug, PartialEq, Eq)]
struct Action {
    argv: Vec<String>,
    refresh: bool,
}

impl Action {
    fn new(argv: &[&str], refresh: bool) -> Self {
        Self {
            argv: argv.iter().map(|arg| (*arg).to_string()).collect(),
            refresh,
        }
    }
}

fn handle_clicks(input: impl Read + 'static) {
    for line in BufReader::new(input).lines() {
        let Ok(line) = line else { break };
        thread::spawn(move || {
            if let Some(action) =
                parse_click_line(&line).and_then(|(name, instance, button, control)| {
                    click_action(&name, instance.as_deref(), button, control)
                })
            {
                perform_action(&action);
            }
        });
    }
}

fn perform_action(action: &Action) {
    if action.refresh {
        run_to_completion(&action.argv);
        drop(spawn_child(REFRESH_STATUS_ARGV, Stdio::null()));
    } else {
        drop(spawn_child(&action.argv, Stdio::null()));
    }
}

fn parse_click_line(line: &str) -> Option<(String, Option<String>, i64, bool)> {
    let click: Value = serde_json::from_str(line.trim().trim_matches(',')).ok()?;
    parse_click(&click)
}

fn parse_click(click: &Value) -> Option<(String, Option<String>, i64, bool)> {
    let name = click.get("name")?.as_str()?.to_string();
    let instance = click
        .get("instance")
        .and_then(Value::as_str)
        .map(String::from);
    let button = click.get("button")?.as_i64()?;
    let control = click
        .get("modifiers")
        .and_then(Value::as_array)
        .is_some_and(|modifiers| {
            modifiers
                .iter()
                .any(|modifier| modifier.as_str() == Some("Control"))
        });
    Some((name, instance, button, control))
}

fn click_action(
    name: &str,
    instance: Option<&str>,
    button: i64,
    with_control: bool,
) -> Option<Action> {
    if let Some((_, argv, refresh)) = CLICK_COMMANDS.iter().find(|(key, ..)| *key == name) {
        return Some(Action::new(argv, *refresh));
    }
    if name != "volume" {
        return None;
    }
    // Instances starting with "default" control the default sink; instances
    // containing "Capture" control a microphone, which only offers mute.
    let instance = instance.unwrap_or_default();
    let sink = instance.starts_with("default");
    let mic = !sink && instance.contains("Capture");
    let step = i64::from(with_control) * 4 + 1;
    let action = match (sink, mic, button) {
        (true, _, 3) => Action::new(&["pavucontrol"], false),
        (true, _, 1 | 2) => Action::new(
            &["pactl", "set-sink-mute", "@DEFAULT_SINK@", "toggle"],
            true,
        ),
        (true, _, 4 | 7) => sink_volume_action('+', step),
        (true, _, 5 | 6) => sink_volume_action('-', step),
        (_, true, 1 | 2) => Action::new(
            &["pactl", "set-source-mute", "@DEFAULT_SOURCE@", "toggle"],
            true,
        ),
        _ => return None,
    };
    Some(action)
}

fn sink_volume_action(sign: char, step: i64) -> Action {
    Action {
        argv: vec![
            "pactl".into(),
            "set-sink-volume".into(),
            "@DEFAULT_SINK@".into(),
            format!("{sign}{step}%"),
        ],
        refresh: true,
    }
}

// Entrypoint

fn parse_args(argv: &[String]) -> Result<(), String> {
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    match argv.as_slice() {
        [] => Ok(()),
        ["-h" | "--help"] => Err(USAGE.to_string()),
        [single] => Err(format!("unknown argument: {single}")),
        [extra, ..] => Err(format!("unexpected argument: {extra}")),
    }
}

fn run(out: &mut dyn Write) -> std::io::Result<i32> {
    let mut i3status = spawn_child(I3STATUS_ARGV, Stdio::piped())?;
    let updates = i3status
        .stdout
        .take()
        .expect("child spawned with piped stdout");
    let clicks = thread::spawn(|| handle_clicks(std::io::stdin().lock()));

    writeln!(out, "{HEADER_LINE}")?;
    writeln!(out, "[")?;
    out.flush()?;
    for line in BufReader::new(updates).lines() {
        let Ok(line) = line else { break };
        let Some(blocks) = parse_status_line(&line) else {
            continue;
        };
        writeln!(
            out,
            "{},",
            serde_json::to_string(&process_blocks(media_block(), blocks))?
        )?;
        out.flush()?;
    }
    drop(i3status.wait());
    writeln!(out, "]")?;
    let _ = clicks.join();
    Ok(0)
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Err(message) = parse_args(&arguments) {
        if message.starts_with("usage:") {
            println!("{message}");
            return ExitCode::SUCCESS;
        }
        eprintln!("{PROGRAM_NAME}: {message}");
        return ExitCode::from(2);
    }
    match run(&mut std::io::stdout().lock()) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            eprintln!("{PROGRAM_NAME}: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        add_no_internet_info, click_action, parse_click, parse_click_line, parse_status_line,
        process_blocks, render_media_block, Action,
    };
    use serde_json::{json, Value};

    #[test]
    fn parses_status_lines() {
        assert_eq!(
            parse_status_line(r#" [{"a":1},{"b":2}], "#).expect("parses"),
            vec![json!({"a": 1}), json!({"b": 2})]
        );
        assert_eq!(
            parse_status_line(",[{\"a\":1}] ").expect("parses"),
            vec![json!({"a": 1})]
        );
        assert!(parse_status_line("[").is_none());
        assert!(parse_status_line("{\"version\":1}").is_none());
        assert!(parse_status_line("not json").is_none());
    }

    #[test]
    fn marks_missing_internet() {
        let down = vec![json!({"name": "ethernet"})];
        let marked = add_no_internet_info(down.clone());
        assert_eq!(marked[0]["full_text"], json!("⛔"));

        let up = vec![json!({"name": "wireless", "full_text": "ssid"})];
        assert_eq!(add_no_internet_info(up.clone()), up);

        let unrelated = vec![json!({"name": "time", "full_text": "12:00"})];
        assert_eq!(add_no_internet_info(unrelated.clone()), unrelated);

        // An explicit empty string still counts as shown text.
        let blank = vec![json!({"name": "ipv6", "full_text": ""})];
        assert_eq!(add_no_internet_info(blank.clone()), blank);
    }

    #[test]
    fn renders_media_block() {
        assert_eq!(
            render_media_block(None, None)["full_text"],
            json!(""),
            "no playerctl at all"
        );

        let playing = render_media_block(Some("Song "), Some("Playing\n"));
        assert_eq!(playing["full_text"], json!("Song ⏸️"));
        assert_eq!(playing["color"], json!("#BBFFBB"));

        let paused = render_media_block(Some("Song"), Some("Paused"));
        assert_eq!(paused["full_text"], json!("Song ▶️"));
        assert_eq!(paused["color"], json!("#BBBBFF"));

        assert_eq!(
            render_media_block(Some(""), Some(""))["full_text"],
            json!(""),
            "playerctl ran but reported nothing"
        );
    }

    #[test]
    fn assembles_blocks() {
        let media = json!({"name": "media_info", "full_text": "x"});
        let blocks = process_blocks(media, vec![json!({"name": "time"})]);
        assert_eq!(
            blocks,
            vec![
                json!({"name": "media_info", "full_text": "x"}),
                json!({"name": "time"}),
                json!({"name": "terminal", "full_text": "📄"}),
                json!({"name": "menu", "full_text": "🔍"}),
                json!({"name": "close", "full_text": "❌"}),
            ]
        );
    }

    #[test]
    fn maps_clicks_to_commands() {
        let action = |name: &str| click_action(name, None, 1, false);
        assert_eq!(
            action("close"),
            Some(Action::new(&["i3-msg", "kill"], false))
        );
        assert_eq!(
            action("media_info"),
            Some(Action::new(&["playerctl", "play-pause"], true))
        );
        assert_eq!(action("unknown"), None);

        let volume = click_action("volume", Some("default.Master.0"), 4, false).expect("up");
        assert_eq!(
            volume,
            Action::new(&["pactl", "set-sink-volume", "@DEFAULT_SINK@", "+1%"], true)
        );

        let boosted = click_action("volume", Some("default.Master.0"), 7, true).expect("boosted");
        assert_eq!(
            boosted,
            Action::new(&["pactl", "set-sink-volume", "@DEFAULT_SINK@", "+5%"], true)
        );

        let lowered = click_action("volume", Some("default.Master.0"), 5, false).expect("lowered");
        assert_eq!(
            lowered,
            Action::new(&["pactl", "set-sink-volume", "@DEFAULT_SINK@", "-1%"], true)
        );

        assert_eq!(
            click_action("volume", Some("default.Master.0"), 3, false).expect("pavucontrol"),
            Action::new(&["pavucontrol"], false)
        );
        assert_eq!(
            click_action("volume", Some("hw:0.Capture.0"), 1, false).expect("mic mute"),
            Action::new(
                &["pactl", "set-source-mute", "@DEFAULT_SOURCE@", "toggle"],
                true
            )
        );
        // Right-click on a microphone does nothing, like the reference.
        assert!(click_action("volume", Some("hw:0.Capture.0"), 3, false).is_none());
        assert!(click_action("volume", Some("other"), 1, false).is_none());
    }

    #[test]
    fn parses_click_events() {
        let click: Value = serde_json::from_str(
            r#"{"name":"volume","instance":"default","button":4,"modifiers":["Control","Shift"]}"#,
        )
        .expect("valid json");
        let (name, instance, button, control) = parse_click(&click).expect("parses");
        assert_eq!((name.as_str(), button, control), ("volume", 4, true));
        assert_eq!(instance.as_deref(), Some("default"));

        let missing_button: Value = json!({"name": "close"});
        assert!(parse_click(&missing_button).is_none());
    }

    #[test]
    fn parses_every_line_of_i3bar_click_stream() {
        // i3bar writes an initial "[" line, a plain object for the first
        // click, and comma-prefixed objects for every following click.
        assert!(parse_click_line("[").is_none());
        let first = parse_click_line("{\"name\":\"close\",\"button\":1}").expect("first");
        assert_eq!(first.0, "close");
        let later = parse_click_line(",{\"name\":\"close\",\"button\":1}").expect("later");
        assert_eq!(later.0, "close");
    }
}
