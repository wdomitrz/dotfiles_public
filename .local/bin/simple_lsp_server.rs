#!/usr/bin/env rscript.sh
// Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
// AGPL License

//# dependencies:
//# lsp-server = "0.7"
//# lsp-types = "0.97"
//# serde = "1"
//# serde_json = "1"
//# shlex = "1"

//! Expose external commands through a small Language Server Protocol server.

use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let parsed = match args::Args::parse(&argv) {
        Ok(parsed) => parsed,
        Err(message) if message.starts_with("usage:") => {
            println!("{message}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("{}: {message}", env!("CARGO_PKG_NAME"));
            return ExitCode::from(2);
        }
    };
    let config = match parsed.config() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{}: {message}", env!("CARGO_PKG_NAME"));
            return ExitCode::from(2);
        }
    };
    match server::serve(&config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}: {error}", env!("CARGO_PKG_NAME"));
            ExitCode::FAILURE
        }
    }
}

mod actions {
    use lsp_types::{CodeAction, CodeActionKind, Diagnostic, Range};

    use crate::positions;

    pub(crate) fn allows_quick_fix(only: Option<&[CodeActionKind]>) -> bool {
        match only {
            None => true,
            Some(kinds) => kinds.contains(&CodeActionKind::QUICKFIX),
        }
    }

    pub(crate) fn filter_actions(
        actions: Vec<CodeAction>,
        request_range: &Range,
        context_diagnostics: &[Diagnostic],
    ) -> Vec<CodeAction> {
        fn attached(action: &CodeAction) -> &[Diagnostic] {
            action.diagnostics.as_deref().unwrap_or_default()
        }
        if context_diagnostics.is_empty() {
            actions
                .into_iter()
                .filter(|action| {
                    attached(action)
                        .iter()
                        .any(|diagnostic| positions::overlaps(&diagnostic.range, request_range))
                })
                .collect()
        } else {
            actions
                .into_iter()
                .filter(|action| {
                    attached(action).iter().any(|action_diagnostic| {
                        context_diagnostics.iter().any(|context_diagnostic| {
                            same_diagnostic(action_diagnostic, context_diagnostic)
                        })
                    })
                })
                .collect()
        }
    }

    fn same_diagnostic(left: &Diagnostic, right: &Diagnostic) -> bool {
        left.range == right.range
            && left.message == right.message
            && left.code == right.code
            && left.source == right.source
    }
}

mod args {
    use crate::{command, server};

    const USAGE: &str = "\
usage: ./simple_lsp_server.rs [options]

Expose external commands through a small Language Server Protocol server.
Each command receives the document on stdin. Supported placeholders:
{file_path}, {uri}, {line}, {character}, {line1}, {character1}.

options:
  --format-command CMD            write the full formatted document to stdout
  --diagnostics-command CMD       write JSON diagnostics to stdout
  --code-actions-command CMD      write ShellCheck json1 output to stdout
  --hover-command CMD             write markdown/plain text or hover JSON
  --definition-command CMD        write a JSON location, location list, or null
  --references-command CMD        write a JSON location list
  --document-symbols-command CMD  write a JSON list of document symbols
  --diagnostics-on-change         also run diagnostics on every didChange
  --log-level LEVEL               DEBUG, INFO, WARNING, ERROR or CRITICAL
  -h, --help                      show this help";

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) enum LogLevel {
        Critical,
        Error,
        Warning,
        Info,
        Debug,
    }

    impl LogLevel {
        fn parse(name: &str) -> Result<Self, String> {
            match name.to_ascii_uppercase().as_str() {
                "CRITICAL" => Ok(Self::Critical),
                "ERROR" => Ok(Self::Error),
                "WARNING" => Ok(Self::Warning),
                "INFO" => Ok(Self::Info),
                "DEBUG" => Ok(Self::Debug),
                other => Err(format!("argument --log-level: invalid level '{other}'")),
            }
        }

        pub(crate) fn allows_debug(self) -> bool {
            self >= Self::Debug
        }
    }

    #[derive(Debug)]
    pub(crate) struct Args {
        pub(crate) format_command: Option<command::Command>,
        pub(crate) diagnostics_command: Option<command::Command>,
        pub(crate) code_actions_command: Option<command::Command>,
        pub(crate) hover_command: Option<command::Command>,
        pub(crate) definition_command: Option<command::Command>,
        pub(crate) references_command: Option<command::Command>,
        pub(crate) document_symbols_command: Option<command::Command>,
        pub(crate) diagnostics_on_change: bool,
        pub(crate) log_level: LogLevel,
    }

    impl Args {
        // Defaults live here, next to the flag handling.
        pub(crate) fn parse(argv: &[String]) -> Result<Self, String> {
            let mut format_command = None;
            let mut diagnostics_command = None;
            let mut code_actions_command = None;
            let mut hover_command = None;
            let mut definition_command = None;
            let mut references_command = None;
            let mut document_symbols_command = None;
            let mut diagnostics_on_change = false;
            let mut log_level = LogLevel::Warning;
            let mut index = 0;
            while index < argv.len() {
                match argv[index].as_str() {
                    "--format-command" => {
                        format_command = Some(parse_command(argv, &mut index)?);
                    }
                    "--diagnostics-command" => {
                        diagnostics_command = Some(parse_command(argv, &mut index)?);
                    }
                    "--code-actions-command" => {
                        code_actions_command = Some(parse_command(argv, &mut index)?);
                    }
                    "--hover-command" => {
                        hover_command = Some(parse_command(argv, &mut index)?);
                    }
                    "--definition-command" => {
                        definition_command = Some(parse_command(argv, &mut index)?);
                    }
                    "--references-command" => {
                        references_command = Some(parse_command(argv, &mut index)?);
                    }
                    "--document-symbols-command" => {
                        document_symbols_command = Some(parse_command(argv, &mut index)?);
                    }
                    "--diagnostics-on-change" => diagnostics_on_change = true,
                    "--log-level" => {
                        index += 1;
                        let raw = argv
                            .get(index)
                            .ok_or("argument --log-level: expected one argument")?;
                        log_level = LogLevel::parse(raw)?;
                    }
                    "-h" | "--help" => return Err(USAGE.to_string()),
                    other => return Err(format!("unknown argument: {other}")),
                }
                index += 1;
            }
            Ok(Self {
                format_command,
                diagnostics_command,
                code_actions_command,
                hover_command,
                definition_command,
                references_command,
                document_symbols_command,
                diagnostics_on_change,
                log_level,
            })
        }

        pub(crate) fn config(&self) -> Result<server::Config, String> {
            let config = server::Config {
                format: self.format_command.clone(),
                diagnostics: self.diagnostics_command.clone(),
                code_actions: self.code_actions_command.clone(),
                hover: self.hover_command.clone(),
                definition: self.definition_command.clone(),
                references: self.references_command.clone(),
                document_symbols: self.document_symbols_command.clone(),
                diagnostics_on_change: self.diagnostics_on_change,
                log_level: self.log_level,
            };
            config.validate()?;
            Ok(config)
        }
    }

    fn parse_command(argv: &[String], index: &mut usize) -> Result<command::Command, String> {
        *index += 1;
        let flag = argv[*index - 1].as_str();
        let raw = argv
            .get(*index)
            .ok_or_else(|| format!("argument {flag}: expected one argument"))?;
        command::Command::parse(raw)
    }
}

mod command {
    use std::sync::mpsc::Receiver;
    use std::time::{Duration, Instant};

    const TIMEOUT: Duration = Duration::from_secs(10);
    const TIMEOUT_MESSAGE: &str = "timed out after 10 seconds";

    #[derive(Debug)]
    pub(crate) struct CommandResult {
        argv: Vec<String>,
        status: i32,
        stdout: String,
        stderr: String,
    }

    impl CommandResult {
        pub(crate) fn argv(&self) -> &[String] {
            &self.argv
        }
        pub(crate) fn status(&self) -> i32 {
            self.status
        }
        pub(crate) fn stdout(&self) -> &str {
            &self.stdout
        }
        pub(crate) fn stderr(&self) -> &str {
            &self.stderr
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub(crate) struct RenderContext<'a> {
        file_path: &'a str,
        uri: &'a str,
        position: Option<(u32, u32)>,
    }

    impl<'a> RenderContext<'a> {
        pub(crate) fn new(file_path: &'a str, uri: &'a str, position: Option<(u32, u32)>) -> Self {
            Self {
                file_path,
                uri,
                position,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct Command {
        argv: Vec<String>,
    }

    impl Command {
        pub(crate) fn parse(value: &str) -> Result<Self, String> {
            let argv = shlex::split(value)
                .ok_or_else(|| "command contains an unterminated quote".to_string())?;
            if argv.is_empty() {
                return Err("command must not be empty".to_string());
            }
            Ok(Self { argv })
        }

        pub(crate) fn argv(&self) -> &[String] {
            &self.argv
        }

        pub(crate) fn execute(
            &self,
            source: &str,
            context: &RenderContext<'_>,
        ) -> Result<CommandResult, String> {
            exec(&self.render(context), source)
        }

        fn render(&self, context: &RenderContext<'_>) -> Vec<String> {
            self.argv
                .iter()
                .map(|part| render_part(part, context))
                .collect()
        }
    }

    pub(crate) fn render_part(part: &str, context: &RenderContext<'_>) -> String {
        let values = placeholder_values(context);
        let mut rendered = String::with_capacity(part.len());
        let mut rest = part;
        while let Some(open) = rest.find('{') {
            rendered.push_str(&rest[..open]);
            let tail = &rest[open..];
            if let Some((name, value)) = values
                .iter()
                .find(|(name, _)| tail.starts_with(&format!("{{{name}}}")))
            {
                rendered.push_str(value);
                rest = &tail[name.len() + 2..];
            } else {
                rendered.push('{');
                rest = &tail[1..];
            }
        }
        rendered.push_str(rest);
        rendered
    }

    fn placeholder_values(context: &RenderContext<'_>) -> [(&'static str, String); 6] {
        let zero_based = context
            .position
            .map_or((String::new(), String::new()), |(line, character)| {
                (line.to_string(), character.to_string())
            });
        let one_based =
            context
                .position
                .map_or((String::new(), String::new()), |(line, character)| {
                    (
                        line.saturating_add(1).to_string(),
                        character.saturating_add(1).to_string(),
                    )
                });
        [
            ("file_path", context.file_path.to_string()),
            ("uri", context.uri.to_string()),
            ("line", zero_based.0),
            ("character", zero_based.1),
            ("line1", one_based.0),
            ("character1", one_based.1),
        ]
    }

    fn exec(argv: &[String], source: &str) -> Result<CommandResult, String> {
        let program = argv
            .first()
            .ok_or_else(|| "command must not be empty".to_string())?;
        let mut spawned = std::process::Command::new(program);
        spawned
            .args(&argv[1..])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = spawned
            .spawn()
            .map_err(|error| format!("{program}: {error}"))?;

        let mut stdin = child.stdin.take().expect("piped stdin");
        let input = source.to_string();
        std::thread::spawn(move || {
            let _ = std::io::Write::write_all(&mut stdin, input.as_bytes());
        });
        let stdout_receiver = read_to_end(child.stdout.take().expect("piped stdout"));
        let stderr_receiver = read_to_end(child.stderr.take().expect("piped stderr"));

        let deadline = Instant::now() + TIMEOUT;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(TIMEOUT_MESSAGE.to_string());
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(error) => return Err(error.to_string()),
            }
        };
        Ok(CommandResult {
            argv: argv.to_vec(),
            status: status.code().unwrap_or(-1),
            stdout: stdout_receiver.recv().unwrap_or_default(),
            stderr: stderr_receiver.recv().unwrap_or_default(),
        })
    }

    fn read_to_end<T>(pipe: T) -> Receiver<String>
    where
        T: std::io::Read + Send + 'static,
    {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut text = String::new();
            let _ = std::io::Read::read_to_string(&mut { pipe }, &mut text);
            let _ = sender.send(text);
        });
        receiver
    }
}

mod diagnostics {
    use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString};
    use serde_json::Value;

    use crate::{json_util, json_util::JsonObject, positions, shellcheck};

    const DEFAULT_SOURCE: &str = "simple-lsp-server";

    pub(crate) fn parse_output(stdout: &str) -> Result<Vec<Diagnostic>, String> {
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }
        match serde_json::from_str::<Value>(stdout) {
            Ok(value) => from_json(&value),
            Err(_) => parse_lines(stdout),
        }
    }

    fn from_json(value: &Value) -> Result<Vec<Diagnostic>, String> {
        match value {
            Value::Object(object) => {
                if object.contains_key("comments") {
                    return shellcheck::parse_diagnostics(value);
                }
                match object.get("diagnostics") {
                    None => Ok(Vec::new()),
                    Some(list) => parse_list(list),
                }
            }
            value => parse_list(value),
        }
    }

    fn parse_list(value: &Value) -> Result<Vec<Diagnostic>, String> {
        json_util::as_array_slice(value, "diagnostics output")?
            .iter()
            .map(parse_item)
            .collect()
    }

    fn parse_item(item: &Value) -> Result<Diagnostic, String> {
        let object = json_util::as_object(item, "each diagnostic")?;
        if object.contains_key("location") {
            return parse_ruff_item(object);
        }
        if object.contains_key("rule") && object.contains_key("description") {
            return parse_markdownlint_json_item(object);
        }
        parse_custom_json_item(object)
    }

    fn parse_custom_json_item(object: &JsonObject) -> Result<Diagnostic, String> {
        let line = json_util::optional_uint(object, "line")?.unwrap_or(0);
        let character = json_util::optional_uint(object, "character")?.unwrap_or(0);
        let end_line = json_util::optional_uint(object, "end_line")?.unwrap_or(line);
        let end_character = json_util::optional_uint(object, "end_character")?
            .unwrap_or(character.saturating_add(1));
        let message = json_util::required_string(object, "message")?;
        let source = match object.get("source") {
            None => DEFAULT_SOURCE.to_string(),
            Some(Value::String(text)) => text.clone(),
            Some(_) => return Err("diagnostic source must be a string".to_string()),
        };
        let code = json_util::optional_number_or_string(object, "code")?;
        let severity = match object.get("severity") {
            None => DiagnosticSeverity::ERROR,
            Some(value) => severity(value.clone())?,
        };
        let range = positions::create_range(line, character, Some(end_line), Some(end_character))?;
        Ok(build(range, severity, code, &source, &message))
    }

    fn parse_ruff_item(object: &JsonObject) -> Result<Diagnostic, String> {
        let location = json_util::as_object(
            json_util::required_field(object, "location")?,
            "ruff location",
        )?;
        let end_location = match object.get("end_location") {
            None => location,
            Some(value) => json_util::as_object(value, "ruff end_location")?,
        };
        let line = one_based_in(location, "row", "location.row")?;
        let character = one_based_in(location, "column", "location.column")?;
        let end_line = one_based_in(end_location, "row", "end_location.row")?;
        let end_character = one_based_in(end_location, "column", "end_location.column")?;
        let message = json_util::required_string(object, "message")?;
        let code = match object.get("code") {
            None | Some(Value::Null) => None,
            Some(Value::String(text)) => Some(NumberOrString::String(text.clone())),
            Some(_) => return Err("ruff diagnostic code must be a string or null".to_string()),
        };
        let severity = match object.get("severity") {
            None => DiagnosticSeverity::ERROR,
            Some(value) => severity(value.clone())?,
        };
        let range = positions::create_range(line, character, Some(end_line), Some(end_character))?;
        Ok(build(range, severity, code, "ruff", &message))
    }

    fn parse_markdownlint_json_item(object: &JsonObject) -> Result<Diagnostic, String> {
        let line = one_based_in(object, "line", "line")?;
        let filename = json_util::string_field(object, "filename")?;
        let rule = json_util::string_field(object, "rule")?;
        let description = json_util::string_field(object, "description")?;
        let range = positions::create_range(line, 0, None, None)?;
        Ok(build(
            range,
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(rule)),
            &filename,
            &description,
        ))
    }

    fn parse_lines(stdout: &str) -> Result<Vec<Diagnostic>, String> {
        stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(parse_line)
            .collect()
    }

    fn parse_line(line: &str) -> Result<Diagnostic, String> {
        if let Some(parsed) = parse_markdownlint_line(line) {
            return parsed;
        }
        parse_gcc_line(line)
    }

    fn parse_markdownlint_line(line: &str) -> Option<Result<Diagnostic, String>> {
        let (head, rest) = split_before_markdownlint_token(line)?;
        let token = rest.split_whitespace().next()?;
        let code = markdownlint_code(token)?;
        let message = rest[token.len()..].trim();
        let (path, row, column) = trailing_position(head.trim_end())?;
        if row == 0 {
            return Some(Err(format!(
                "cannot parse diagnostic line: {line}: line must be one-based"
            )));
        }
        let character = match column {
            None => 0,
            Some(0) => {
                return Some(Err(format!(
                    "cannot parse diagnostic line: {line}: column must be one-based"
                )));
            }
            Some(column) => column - 1,
        };
        let range = match positions::create_range(row - 1, character, None, None) {
            Ok(range) => range,
            Err(error) => return Some(Err(error)),
        };
        Some(Ok(build(
            range,
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(code)),
            path,
            message,
        )))
    }

    fn split_before_markdownlint_token(line: &str) -> Option<(&str, &str)> {
        let bytes = line.as_bytes();
        for (index, _) in line.char_indices() {
            if !line[index..].starts_with("MD") {
                continue;
            }
            let preceded_by_space = index == 0 || bytes[index - 1].is_ascii_whitespace();
            let digits = bytes.get(index + 2..index + 5)?;
            if preceded_by_space && digits.iter().all(u8::is_ascii_digit) {
                return Some((&line[..index], &line[index..]));
            }
        }
        None
    }

    fn markdownlint_code(token: &str) -> Option<String> {
        let base = match token.split_once('/') {
            Some((base, _alias)) => base,
            None => token,
        };
        if base.len() != 5 || !base.starts_with("MD") || !is_digits(&base[2..]) {
            return None;
        }
        Some(base.to_string())
    }

    fn trailing_position(head: &str) -> Option<(&str, u32, Option<u32>)> {
        let (path, row_text, column) = match head.rsplit_once(':') {
            Some((parent, last)) if is_digits(last) => match parent.rsplit_once(':') {
                Some((grandparent, row_text)) if is_digits(row_text) => {
                    let column = last.parse::<u32>().ok()?;
                    (grandparent, row_text, Some(column))
                }
                _ => (parent, last, None),
            },
            _ => return None,
        };
        let row = row_text.parse::<u32>().ok()?;
        Some((path, row, column))
    }

    fn parse_gcc_line(line: &str) -> Result<Diagnostic, String> {
        let malformed = || format!("cannot parse diagnostic line: {line}");
        // Anchor on the first `:row:column:` triple, like the lazy regex in
        // the Python original; path and message may themselves contain colons.
        let colons: Vec<usize> = line.match_indices(':').map(|(index, _)| index).collect();
        let anchor = colons.windows(3).find_map(|window| {
            let (a, b, c) = (window[0], window[1], window[2]);
            (is_digits(&line[a + 1..b]) && is_digits(&line[b + 1..c])).then_some((a, b, c))
        });
        let Some((a, b, c)) = anchor else {
            return Err(malformed());
        };
        let source = &line[..a];
        let row = one_based_text(&line[a + 1..b]).ok_or_else(malformed)?;
        let character = one_based_text(&line[b + 1..c]).ok_or_else(malformed)?;
        let raw_message = &line[c + 1..];
        let (prefix_severity, rest) = strip_severity_prefix(raw_message.trim_start());
        let (message, code) = strip_trailing_code(rest);
        let severity = prefix_severity.unwrap_or(DiagnosticSeverity::ERROR);
        let range = positions::create_range(row, character, None, None)?;
        Ok(build(
            range,
            severity,
            code.map(NumberOrString::String),
            source,
            message,
        ))
    }

    fn one_based_text(text: &str) -> Option<u32> {
        if !is_digits(text) {
            return None;
        }
        let raw = text.parse::<u32>().ok()?;
        (raw > 0).then_some(raw - 1)
    }

    fn is_digits(text: &str) -> bool {
        !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
    }

    fn strip_severity_prefix(message: &str) -> (Option<DiagnosticSeverity>, &str) {
        const PREFIXES: [(&str, DiagnosticSeverity); 5] = [
            ("error", DiagnosticSeverity::ERROR),
            ("warning", DiagnosticSeverity::WARNING),
            ("note", DiagnosticSeverity::INFORMATION),
            ("info", DiagnosticSeverity::INFORMATION),
            ("hint", DiagnosticSeverity::HINT),
        ];
        for (word, severity) in PREFIXES {
            let prefix = format!("{word}:");
            if let Some(rest) = message.strip_prefix(prefix.as_str()) {
                return (Some(severity), rest.trim_start());
            }
        }
        (None, message)
    }

    fn strip_trailing_code(message: &str) -> (&str, Option<String>) {
        if let Some(head) = message.strip_suffix(']') {
            if let Some((_before, inner)) = head.rsplit_once('[') {
                if let Some(code) = bracket_code(inner) {
                    let stripped = &head[..head.len() - inner.len() - 1];
                    return (stripped.trim_end(), Some(code));
                }
            }
        }
        (message, None)
    }

    fn bracket_code(inner: &str) -> Option<String> {
        let letters = inner.len()
            - inner
                .trim_start_matches(|c: char| c.is_ascii_alphabetic())
                .len();
        if letters == 0 {
            return None;
        }
        let digits = &inner[letters..];
        if !is_digits(digits) {
            return None;
        }
        Some(inner.to_string())
    }

    pub(crate) fn severity(value: Value) -> Result<DiagnosticSeverity, String> {
        match value {
            Value::Number(number) => {
                let raw = number
                    .as_i64()
                    .filter(|raw| (1..=4).contains(raw))
                    .ok_or_else(|| format!("unsupported diagnostic severity: {number}"))?;
                Ok(match raw {
                    1 => DiagnosticSeverity::ERROR,
                    2 => DiagnosticSeverity::WARNING,
                    3 => DiagnosticSeverity::INFORMATION,
                    _ => DiagnosticSeverity::HINT,
                })
            }
            Value::String(name) => severity_from_name(&name),
            _ => Err("diagnostic severity must be a string or integer".to_string()),
        }
    }

    fn severity_from_name(name: &str) -> Result<DiagnosticSeverity, String> {
        match name.to_lowercase().as_str() {
            "error" => Ok(DiagnosticSeverity::ERROR),
            "warning" => Ok(DiagnosticSeverity::WARNING),
            "information" | "info" | "note" => Ok(DiagnosticSeverity::INFORMATION),
            "hint" | "style" => Ok(DiagnosticSeverity::HINT),
            other => Err(format!("unsupported diagnostic severity: {other}")),
        }
    }

    pub(crate) fn build(
        range: lsp_types::Range,
        severity: DiagnosticSeverity,
        code: Option<NumberOrString>,
        source: &str,
        message: &str,
    ) -> Diagnostic {
        Diagnostic {
            range,
            severity: Some(severity),
            code,
            code_description: None,
            source: Some(source.to_string()),
            message: message.to_string(),
            related_information: None,
            tags: None,
            data: None,
        }
    }

    fn one_based_in(object: &JsonObject, key: &str, label: &str) -> Result<u32, String> {
        json_util::one_based_field(json_util::required_field(object, key)?, label)
    }
}

mod hovers {
    use lsp_types::{Hover, MarkupContent, MarkupKind};
    use serde_json::Value;

    use crate::{json_util, locations};

    pub(crate) fn parse_output(stdout: &str) -> Result<Option<Hover>, String> {
        if stdout.trim().is_empty() {
            return Ok(None);
        }
        let Ok(value) = serde_json::from_str::<Value>(stdout) else {
            let text = stdout.trim_end_matches('\n');
            return Ok(Some(Hover {
                contents: lsp_types::HoverContents::Markup(markup(MarkupKind::Markdown, text)),
                range: None,
            }));
        };
        match value {
            Value::String(text) => Ok(Some(Hover {
                contents: lsp_types::HoverContents::Markup(markup(MarkupKind::Markdown, &text)),
                range: None,
            })),
            Value::Object(object) => from_object(&object).map(Some),
            _ => Err("hover must be text, a JSON string, or a JSON object".to_string()),
        }
    }

    fn from_object(object: &json_util::JsonObject) -> Result<Hover, String> {
        let contents = match object.get("contents") {
            Some(Value::String(text)) => text.clone(),
            _ => return Err("hover contents must be a string".to_string()),
        };
        let kind = match object.get("kind") {
            None => MarkupKind::Markdown,
            Some(Value::String(kind)) if kind == "markdown" => MarkupKind::Markdown,
            Some(Value::String(kind)) if kind == "plaintext" => MarkupKind::PlainText,
            Some(_) => return Err("hover kind must be markdown or plaintext".to_string()),
        };
        let range = if object.contains_key("line") || object.contains_key("character") {
            Some(locations::range_from_object(object)?)
        } else {
            None
        };
        Ok(Hover {
            contents: lsp_types::HoverContents::Markup(markup(kind, &contents)),
            range,
        })
    }

    fn markup(kind: MarkupKind, value: &str) -> MarkupContent {
        MarkupContent {
            kind,
            value: value.to_string(),
        }
    }
}

mod json_util {
    use lsp_types::NumberOrString;
    use serde_json::Value;

    pub(crate) type JsonObject = serde_json::Map<String, Value>;

    pub(crate) fn as_object<'value>(
        value: &'value Value,
        context: &str,
    ) -> Result<&'value JsonObject, String> {
        value
            .as_object()
            .ok_or_else(|| format!("{context} must be a JSON object"))
    }

    pub(crate) fn as_array_slice<'value>(
        value: &'value Value,
        context: &str,
    ) -> Result<&'value [Value], String> {
        value
            .as_array()
            .map(Vec::as_slice)
            .ok_or_else(|| format!("{context} must be a JSON list"))
    }

    pub(crate) fn required_field<'a>(
        object: &'a JsonObject,
        key: &str,
    ) -> Result<&'a Value, String> {
        object.get(key).ok_or_else(|| format!("{key} is required"))
    }

    pub(crate) fn optional_uint(object: &JsonObject, key: &str) -> Result<Option<u32>, String> {
        match object.get(key) {
            None => Ok(None),
            Some(value) => uint_field(value, key).map(Some),
        }
    }

    pub(crate) fn uint_field(value: &Value, label: &str) -> Result<u32, String> {
        let raw = match value {
            Value::Number(_) => value
                .as_u64()
                .ok_or_else(|| format!("{label} must be an integer"))?,
            Value::String(text) => text
                .trim()
                .parse::<u64>()
                .map_err(|_| format!("{label} must be an integer"))?,
            _ => return Err(format!("{label} must be an integer")),
        };
        u32::try_from(raw).map_err(|_| format!("{label} is too large"))
    }

    pub(crate) fn one_based_field(value: &Value, label: &str) -> Result<u32, String> {
        let raw = uint_field(value, label)?;
        if raw == 0 {
            return Err(format!("{label} must be one-based"));
        }
        Ok(raw - 1)
    }

    pub(crate) fn required_string(object: &JsonObject, key: &str) -> Result<String, String> {
        match object.get(key) {
            Some(Value::String(text)) if !text.is_empty() => Ok(text.clone()),
            _ => Err(format!("{key} must be a non-empty string")),
        }
    }

    pub(crate) fn string_field(object: &JsonObject, key: &str) -> Result<String, String> {
        match object.get(key) {
            Some(Value::String(text)) => Ok(text.clone()),
            _ => Err(format!("{key} must be a string")),
        }
    }

    pub(crate) fn optional_number_or_string(
        object: &JsonObject,
        key: &str,
    ) -> Result<Option<NumberOrString>, String> {
        match object.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(text)) => Ok(Some(NumberOrString::String(text.clone()))),
            Some(value @ Value::Number(_)) => {
                let raw = uint_field(value, key)?;
                let number = i32::try_from(raw).map_err(|_| format!("{key} is too large"))?;
                Ok(Some(NumberOrString::Number(number)))
            }
            Some(_) => Err(format!("{key} must be a string, integer, or null")),
        }
    }
}

mod locations {
    use lsp_types::{Location, Range, Uri};
    use serde_json::Value;

    use crate::{json_util, positions};

    #[derive(Debug)]
    pub(crate) enum Locations {
        Missing,
        One(Location),
        Many(Vec<Location>),
    }

    pub(crate) fn parse_output(stdout: &str) -> Result<Locations, String> {
        if stdout.trim().is_empty() {
            return Ok(Locations::Missing);
        }
        let value: Value =
            serde_json::from_str(stdout).map_err(|error| format!("invalid JSON: {error}"))?;
        match value {
            Value::Null => Ok(Locations::Missing),
            Value::Array(items) => items
                .iter()
                .map(parse_item)
                .collect::<Result<Vec<_>, String>>()
                .map(Locations::Many),
            value => parse_item(&value).map(Locations::One),
        }
    }

    fn parse_item(value: &Value) -> Result<Location, String> {
        let object = json_util::as_object(value, "location")?;
        let uri = match object.get("uri") {
            Some(Value::String(uri)) => uri.clone(),
            _ => uri_from_file_path(object.get("file_path"))?,
        };
        let parsed_uri = uri
            .parse::<Uri>()
            .map_err(|_| format!("invalid location uri: {uri}"))?;
        Ok(Location {
            uri: parsed_uri,
            range: range_from_object(object)?,
        })
    }

    pub(crate) fn range_from_object(object: &json_util::JsonObject) -> Result<Range, String> {
        let line = json_util::optional_uint(object, "line")?.unwrap_or(0);
        let character = json_util::optional_uint(object, "character")?.unwrap_or(0);
        let end_line = json_util::optional_uint(object, "end_line")?.unwrap_or(line);
        let end_character = json_util::optional_uint(object, "end_character")?
            .unwrap_or(character.saturating_add(1));
        positions::create_range(line, character, Some(end_line), Some(end_character))
    }

    fn uri_from_file_path(path: Option<&Value>) -> Result<String, String> {
        let Some(Value::String(path)) = path else {
            return Err("location must contain uri or file_path".to_string());
        };
        Ok(file_path_to_uri(path))
    }

    pub(crate) fn file_path_to_uri(path: &str) -> String {
        let expanded = expand_home(path);
        let absolute = absolutize(std::path::Path::new(&expanded));
        let canonical = std::fs::canonicalize(&absolute).unwrap_or(absolute);
        format!("file://{}", percent_encode(&canonical.to_string_lossy()))
    }

    pub(crate) fn file_path_of_uri(uri: &Uri) -> String {
        let raw = uri.as_str();
        let Some(rest) = raw.strip_prefix("file://") else {
            return raw.to_string();
        };
        let path = if rest.starts_with('/') {
            rest
        } else {
            match rest.find('/') {
                Some(index) => &rest[index..],
                None => "/",
            }
        };
        percent_decode(path)
    }

    fn expand_home(path: &str) -> String {
        if let Some(rest) = path.strip_prefix("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return format!("{home}/{rest}");
            }
        }
        path.to_string()
    }

    fn absolutize(path: &std::path::Path) -> std::path::PathBuf {
        if path.is_absolute() {
            return path.to_path_buf();
        }
        std::env::current_dir().unwrap_or_default().join(path)
    }

    fn percent_encode(path: &str) -> String {
        use std::fmt::Write as _;
        let mut encoded = String::with_capacity(path.len());
        for byte in path.bytes() {
            match byte {
                b'A'..=b'Z'
                | b'a'..=b'z'
                | b'0'..=b'9'
                | b'/'
                | b'.'
                | b'_'
                | b'-'
                | b'~'
                | b'+' => encoded.push(char::from(byte)),
                other => {
                    let _ = write!(encoded, "%{other:02X}");
                }
            }
        }
        encoded
    }

    fn percent_decode(text: &str) -> String {
        let bytes = text.as_bytes();
        let mut decoded = Vec::with_capacity(bytes.len());
        let mut index = 0;
        while index < bytes.len() {
            let hex_pair = if bytes[index] == b'%'
                && index + 2 < bytes.len()
                && bytes[index + 1].is_ascii_hexdigit()
                && bytes[index + 2].is_ascii_hexdigit()
            {
                char::from(bytes[index + 1]).to_digit(16).and_then(|high| {
                    char::from(bytes[index + 2])
                        .to_digit(16)
                        .map(move |low| high * 16 + low)
                })
            } else {
                None
            };
            if let Some(byte) = hex_pair {
                decoded.push(u8::try_from(byte).unwrap_or(b'%'));
                index += 3;
            } else {
                decoded.push(bytes[index]);
                index += 1;
            }
        }
        String::from_utf8(decoded).unwrap_or_else(|_| text.to_string())
    }
}

mod positions {
    pub(crate) use lsp_types::{Position, Range};

    pub(crate) fn utf16_length(value: &str) -> usize {
        value.encode_utf16().count()
    }

    pub(crate) fn full_document_range(source: &str) -> Range {
        let lines: Vec<&str> = source.split('\n').collect();
        let last_line = lines.last().copied().unwrap_or("");
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: to_u32(lines.len().saturating_sub(1)),
                character: to_u32(utf16_length(last_line)),
            },
        }
    }

    pub(crate) fn create_range(
        line: u32,
        character: u32,
        end_line: Option<u32>,
        end_character: Option<u32>,
    ) -> Result<Range, String> {
        let end_line = end_line.unwrap_or(line);
        let end_character = end_character.unwrap_or(character.saturating_add(1));
        if (end_line, end_character) < (line, character) {
            return Err("range end must not be before range start".to_string());
        }
        Ok(Range {
            start: Position { line, character },
            end: Position {
                line: end_line,
                character: end_character,
            },
        })
    }

    pub(crate) fn overlaps(left: &Range, right: &Range) -> bool {
        !is_before(left.end, right.start) && !is_before(right.end, left.start)
    }

    fn is_before(left: Position, right: Position) -> bool {
        (left.line, left.character) < (right.line, right.character)
    }

    fn to_u32(value: usize) -> u32 {
        u32::try_from(value).expect("position within u32")
    }
}

mod shellcheck {
    use std::collections::HashMap;

    use lsp_types::{
        CodeAction, CodeActionKind, CodeDescription, Diagnostic, DiagnosticSeverity,
        NumberOrString, TextEdit, Uri, WorkspaceEdit,
    };
    use serde_json::Value;

    use crate::{diagnostics, json_util, json_util::JsonObject, positions};

    pub(crate) fn parse_diagnostics(value: &Value) -> Result<Vec<Diagnostic>, String> {
        Ok(from_json(value)?
            .into_iter()
            .map(|(diagnostic, _action)| diagnostic)
            .collect())
    }

    pub(crate) fn parse_code_actions(
        value: &Value,
        document_uri: &str,
    ) -> Result<Vec<CodeAction>, String> {
        let actions = from_json_with_actions(value, document_uri)?;
        Ok(actions
            .into_iter()
            .filter_map(|(_diagnostic, action)| action)
            .collect())
    }

    // The document uri is only needed for fix edits, so a placeholder is fine
    // when diagnostics alone are requested.
    pub(crate) fn from_json(
        value: &Value,
    ) -> Result<Vec<(Diagnostic, Option<CodeAction>)>, String> {
        from_json_with_actions(value, "file:///document")
    }

    fn from_json_with_actions(
        value: &Value,
        document_uri: &str,
    ) -> Result<Vec<(Diagnostic, Option<CodeAction>)>, String> {
        comments(value)?
            .iter()
            .map(|comment| parse_comment(comment, document_uri))
            .collect()
    }

    fn comments(value: &Value) -> Result<&[Value], String> {
        let object = json_util::as_object(value, "shellcheck output")?;
        match object.get("comments") {
            None => Ok(&[]),
            Some(list) => json_util::as_array_slice(list, "shellcheck comments"),
        }
    }

    fn parse_comment(
        value: &Value,
        document_uri: &str,
    ) -> Result<(Diagnostic, Option<CodeAction>), String> {
        let object = json_util::as_object(value, "shellcheck comment")?;
        let line = one_based(object, "line")?;
        let column = one_based(object, "column")?;
        let end_line = one_based(object, "endLine")?;
        let end_column = one_based(object, "endColumn")?;
        let message = json_util::required_string(object, "message")?;
        let code_raw = json_util::uint_field(json_util::required_field(object, "code")?, "code")?;
        let code =
            i32::try_from(code_raw).map_err(|_| "shellcheck code is too large".to_string())?;
        let level = json_util::string_field(object, "level")?;
        let severity = match level.as_str() {
            "error" => DiagnosticSeverity::ERROR,
            "warning" => DiagnosticSeverity::WARNING,
            "info" => DiagnosticSeverity::INFORMATION,
            "style" => DiagnosticSeverity::HINT,
            other => return Err(format!("unsupported shellcheck level: {other}")),
        };
        let range = positions::create_range(line, column, Some(end_line), Some(end_column))?;
        let mut diagnostic = diagnostics::build(
            range,
            severity,
            Some(NumberOrString::Number(code)),
            "shellcheck",
            &message,
        );
        diagnostic.code_description = Some(CodeDescription {
            href: wiki_uri(code)?,
        });
        let action = quick_fix(object, document_uri, &message, &diagnostic)?;
        Ok((diagnostic, action))
    }

    fn wiki_uri(code: i32) -> Result<Uri, String> {
        format!("https://www.shellcheck.net/wiki/SC{code}")
            .parse::<Uri>()
            .map_err(|_| "invalid shellcheck wiki URL".to_string())
    }

    fn quick_fix(
        object: &JsonObject,
        document_uri: &str,
        title: &str,
        diagnostic: &Diagnostic,
    ) -> Result<Option<CodeAction>, String> {
        let Some(fix) = object.get("fix") else {
            return Ok(None);
        };
        if fix.is_null() {
            return Ok(None);
        }
        let fix_object = json_util::as_object(fix, "shellcheck fix")?;
        let replacements: &[Value] = match fix_object.get("replacements") {
            None => &[],
            Some(list) => list
                .as_array()
                .map(Vec::as_slice)
                .ok_or_else(|| "shellcheck replacements must be a JSON list".to_string())?,
        };
        if replacements.is_empty() {
            return Ok(None);
        }
        let edits = replacements
            .iter()
            .map(replacement_edit)
            .collect::<Result<Vec<_>, String>>()?;
        let uri = document_uri
            .parse::<Uri>()
            .map_err(|_| format!("invalid document uri: {document_uri}"))?;
        Ok(Some(CodeAction {
            title: title.to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(uri, edits)])),
                ..WorkspaceEdit::default()
            }),
            is_preferred: Some(true),
            ..CodeAction::default()
        }))
    }

    fn replacement_edit(value: &Value) -> Result<TextEdit, String> {
        let object = json_util::as_object(value, "shellcheck replacement")?;
        let line = one_based(object, "line")?;
        let column = one_based(object, "column")?;
        let end_line = one_based(object, "endLine")?;
        let end_column = one_based(object, "endColumn")?;
        let new_text = json_util::string_field(object, "replacement")?;
        Ok(TextEdit {
            range: positions::create_range(line, column, Some(end_line), Some(end_column))?,
            new_text,
        })
    }

    fn one_based(object: &JsonObject, key: &str) -> Result<u32, String> {
        json_util::one_based_field(json_util::required_field(object, key)?, key)
    }
}

mod symbols {
    use lsp_types::{DocumentSymbol, Range, SymbolKind};
    use serde_json::Value;

    use crate::{json_util, locations, positions};

    pub(crate) fn parse_output(stdout: &str) -> Result<Vec<DocumentSymbol>, String> {
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }
        let value: Value =
            serde_json::from_str(stdout).map_err(|error| format!("invalid JSON: {error}"))?;
        json_util::as_array_slice(&value, "symbols")?
            .iter()
            .map(parse_item)
            .collect()
    }

    fn parse_item(value: &Value) -> Result<DocumentSymbol, String> {
        let object = json_util::as_object(value, "symbol")?;
        let name = json_util::required_string(object, "name")?;
        let detail = match object.get("detail") {
            None | Some(Value::Null) => None,
            Some(Value::String(text)) => Some(text.clone()),
            Some(_) => return Err("symbol detail must be a string or null".to_string()),
        };
        let range = locations::range_from_object(object)?;
        let selection_range = if object.contains_key("selection_line")
            || object.contains_key("selection_character")
        {
            selection_range(object)?
        } else {
            range
        };
        let children = match object.get("children") {
            None | Some(Value::Null) => None,
            Some(list) => json_util::as_array_slice(list, "symbol children")?
                .iter()
                .map(parse_item)
                .collect::<Result<Vec<_>, String>>()
                .map(Some)?,
        };
        let kind_value = object
            .get("kind")
            .cloned()
            .unwrap_or_else(|| Value::String("function".to_string()));
        #[allow(deprecated)] // field is required by the type, but deprecated
        Ok(DocumentSymbol {
            name,
            detail,
            kind: symbol_kind(&kind_value)?,
            tags: None,
            deprecated: None,
            range,
            selection_range,
            children,
        })
    }

    fn selection_range(object: &json_util::JsonObject) -> Result<Range, String> {
        let line = json_util::optional_uint(object, "line")?.unwrap_or(0);
        let character = json_util::optional_uint(object, "character")?.unwrap_or(0);
        let start_line = json_util::optional_uint(object, "selection_line")?.unwrap_or(line);
        let start_character =
            json_util::optional_uint(object, "selection_character")?.unwrap_or(character);
        let end_line =
            json_util::optional_uint(object, "selection_end_line")?.unwrap_or(start_line);
        let end_character = json_util::optional_uint(object, "selection_end_character")?
            .unwrap_or(start_character.saturating_add(1));
        positions::create_range(
            start_line,
            start_character,
            Some(end_line),
            Some(end_character),
        )
    }

    fn symbol_kind(value: &Value) -> Result<SymbolKind, String> {
        const KINDS: [(i64, SymbolKind); 26] = [
            (1, SymbolKind::FILE),
            (2, SymbolKind::MODULE),
            (3, SymbolKind::NAMESPACE),
            (4, SymbolKind::PACKAGE),
            (5, SymbolKind::CLASS),
            (6, SymbolKind::METHOD),
            (7, SymbolKind::PROPERTY),
            (8, SymbolKind::FIELD),
            (9, SymbolKind::CONSTRUCTOR),
            (10, SymbolKind::ENUM),
            (11, SymbolKind::INTERFACE),
            (12, SymbolKind::FUNCTION),
            (13, SymbolKind::VARIABLE),
            (14, SymbolKind::CONSTANT),
            (15, SymbolKind::STRING),
            (16, SymbolKind::NUMBER),
            (17, SymbolKind::BOOLEAN),
            (18, SymbolKind::ARRAY),
            (19, SymbolKind::OBJECT),
            (20, SymbolKind::KEY),
            (21, SymbolKind::NULL),
            (22, SymbolKind::ENUM_MEMBER),
            (23, SymbolKind::STRUCT),
            (24, SymbolKind::EVENT),
            (25, SymbolKind::OPERATOR),
            (26, SymbolKind::TYPE_PARAMETER),
        ];
        match value {
            Value::Number(number) => {
                let raw = number
                    .as_i64()
                    .ok_or_else(|| "symbol kind must be a string or integer".to_string())?;
                KINDS
                    .iter()
                    .find(|(candidate, _)| *candidate == raw)
                    .map(|(_, kind)| *kind)
                    .ok_or_else(|| format!("unsupported symbol kind: {number}"))
            }
            Value::String(name) => symbol_kind_from_name(name),
            _ => Err("symbol kind must be a string or integer".to_string()),
        }
    }

    fn symbol_kind_from_name(name: &str) -> Result<SymbolKind, String> {
        let normalized = name.to_lowercase().replace('_', "");
        match normalized.as_str() {
            "file" => Ok(SymbolKind::FILE),
            "module" => Ok(SymbolKind::MODULE),
            "namespace" => Ok(SymbolKind::NAMESPACE),
            "package" => Ok(SymbolKind::PACKAGE),
            "class" => Ok(SymbolKind::CLASS),
            "method" => Ok(SymbolKind::METHOD),
            "property" => Ok(SymbolKind::PROPERTY),
            "field" => Ok(SymbolKind::FIELD),
            "constructor" => Ok(SymbolKind::CONSTRUCTOR),
            "enum" => Ok(SymbolKind::ENUM),
            "interface" => Ok(SymbolKind::INTERFACE),
            "function" => Ok(SymbolKind::FUNCTION),
            "variable" => Ok(SymbolKind::VARIABLE),
            "constant" => Ok(SymbolKind::CONSTANT),
            "string" => Ok(SymbolKind::STRING),
            "number" => Ok(SymbolKind::NUMBER),
            "boolean" => Ok(SymbolKind::BOOLEAN),
            "array" => Ok(SymbolKind::ARRAY),
            "object" => Ok(SymbolKind::OBJECT),
            "key" => Ok(SymbolKind::KEY),
            "null" => Ok(SymbolKind::NULL),
            "enummember" => Ok(SymbolKind::ENUM_MEMBER),
            "struct" => Ok(SymbolKind::STRUCT),
            "event" => Ok(SymbolKind::EVENT),
            "operator" => Ok(SymbolKind::OPERATOR),
            "typeparameter" => Ok(SymbolKind::TYPE_PARAMETER),
            other => Err(format!("unsupported symbol kind: {other}")),
        }
    }
}

mod server {
    use std::collections::HashMap;

    use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
    use lsp_types::notification::Notification as _;
    use lsp_types::request::Request as _;
    use lsp_types::{
        notification::Exit,
        notification::{
            DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument, LogMessage,
            PublishDiagnostics,
        },
        request::{
            CodeActionRequest, DocumentSymbolRequest, Formatting, GotoDefinition, HoverRequest,
            References, Shutdown,
        },
        CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
        CodeActionProviderCapability, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, DocumentFormattingParams, DocumentSymbolParams,
        DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, HoverParams,
        HoverProviderCapability, Location, LogMessageParams, MessageType, OneOf,
        PublishDiagnosticsParams, ReferenceParams, ServerCapabilities,
        TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
        Uri,
    };
    use serde::de::DeserializeOwned;

    use crate::{
        actions, args::LogLevel, command, command::Command, diagnostics, hovers, locations,
        positions, shellcheck, symbols,
    };

    const METHOD_NOT_FOUND: i32 = -32601;
    const INVALID_PARAMS: i32 = -32602;

    // Keyed by URI string: lsp_types::Uri would trip mutable-key-type.
    type Documents = HashMap<String, String>;

    struct CommandRun {
        source: String,
        uri: Uri,
        result: command::CommandResult,
    }

    #[derive(Debug)]
    pub(crate) struct Config {
        pub(crate) format: Option<Command>,
        pub(crate) diagnostics: Option<Command>,
        pub(crate) code_actions: Option<Command>,
        pub(crate) hover: Option<Command>,
        pub(crate) definition: Option<Command>,
        pub(crate) references: Option<Command>,
        pub(crate) document_symbols: Option<Command>,
        pub(crate) diagnostics_on_change: bool,
        pub(crate) log_level: LogLevel,
    }

    impl Config {
        pub(crate) fn validate(&self) -> Result<(), String> {
            let has_command = self.format.is_some()
                || self.diagnostics.is_some()
                || self.code_actions.is_some()
                || self.hover.is_some()
                || self.definition.is_some()
                || self.references.is_some()
                || self.document_symbols.is_some();
            if has_command {
                Ok(())
            } else {
                Err("at least one LSP command option is required".to_string())
            }
        }
    }

    pub(crate) fn serve(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let (connection, io_threads) = Connection::stdio();
        connection.initialize(serde_json::to_value(capabilities(config))?)?;

        {
            let mut documents = Documents::new();
            event_loop(config, &mut documents, &connection);
        }

        // The writer thread stops only when every message sender is gone, so
        // the connection must be dropped before joining the IO threads.
        drop(connection);
        io_threads.join()?;
        Ok(())
    }

    fn event_loop(config: &Config, documents: &mut Documents, connection: &Connection) {
        for message in &connection.receiver {
            match message {
                Message::Request(request) => {
                    if request.method == Shutdown::METHOD {
                        let _ = connection
                            .sender
                            .send(Message::Response(Response::new_ok(request.id, ())));
                        continue;
                    }
                    handle_request(config, documents, connection, request);
                }
                Message::Notification(notification) => {
                    if notification.method == Exit::METHOD {
                        // The stdio reader thread only stops after exit; keep
                        // draining messages until then or join() would block.
                        break;
                    }
                    handle_notification(config, documents, connection, notification);
                }
                Message::Response(_) => {}
            }
        }
    }

    fn capabilities(config: &Config) -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            hover_provider: Some(HoverProviderCapability::Simple(config.hover.is_some())),
            definition_provider: Some(OneOf::Left(config.definition.is_some())),
            references_provider: Some(OneOf::Left(config.references.is_some())),
            document_symbol_provider: Some(OneOf::Left(config.document_symbols.is_some())),
            document_formatting_provider: Some(OneOf::Left(config.format.is_some())),
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                ..CodeActionOptions::default()
            })),
            ..ServerCapabilities::default()
        }
    }

    fn trace(config: &Config, message: &str) {
        if config.log_level.allows_debug() {
            eprintln!("{message}");
        }
    }

    fn handle_request(
        config: &Config,
        documents: &Documents,
        connection: &Connection,
        request: Request,
    ) {
        trace(config, &format!("request {}", request.method));
        match request.method.as_str() {
            Formatting::METHOD => formatting_request(config, documents, connection, request),
            CodeActionRequest::METHOD => {
                code_actions_request(config, documents, connection, request);
            }
            HoverRequest::METHOD => hover_request(config, documents, connection, request),
            GotoDefinition::METHOD => definition_request(config, documents, connection, request),
            References::METHOD => references_request(config, documents, connection, request),
            DocumentSymbolRequest::METHOD => {
                document_symbols_request(config, documents, connection, request);
            }
            other => {
                let _ = connection.sender.send(Message::Response(Response::new_err(
                    request.id,
                    METHOD_NOT_FOUND,
                    format!("method not found: {other}"),
                )));
            }
        }
    }

    fn formatting_request(
        config: &Config,
        documents: &Documents,
        connection: &Connection,
        request: Request,
    ) {
        let Some((id, params)) =
            parse_request::<DocumentFormattingParams>(connection, request, Formatting::METHOD)
        else {
            return;
        };
        let edits = match run_document_command(
            config.format.as_ref(),
            documents,
            connection,
            &params.text_document.uri,
            "formatter error",
            false,
            None,
        ) {
            Some(run) => vec![TextEdit {
                range: positions::full_document_range(run.source()),
                new_text: run.result.stdout().to_string(),
            }],
            None => Vec::new(),
        };
        respond(connection, id, &edits);
    }

    fn code_actions_request(
        config: &Config,
        documents: &Documents,
        connection: &Connection,
        request: Request,
    ) {
        let Some((id, params)) =
            parse_request::<CodeActionParams>(connection, request, CodeActionRequest::METHOD)
        else {
            return;
        };
        let reply = code_actions_result(config, documents, connection, &params);
        respond(connection, id, &reply);
    }

    fn code_actions_result(
        config: &Config,
        documents: &Documents,
        connection: &Connection,
        params: &CodeActionParams,
    ) -> Vec<CodeActionOrCommand> {
        let Some(command) = config.code_actions.as_ref() else {
            return Vec::new();
        };
        if !actions::allows_quick_fix(params.context.only.as_deref()) {
            return Vec::new();
        }
        let uri = &params.text_document.uri;
        let Some(run) = run_document_command(
            Some(command),
            documents,
            connection,
            uri,
            "code actions error",
            true,
            None,
        ) else {
            return Vec::new();
        };
        let parsed: serde_json::Value = match serde_json::from_str(run.result.stdout()) {
            Ok(parsed) => parsed,
            Err(error) => {
                log_command_error(
                    connection,
                    "code actions error",
                    run.argv(),
                    uri.as_str(),
                    &format!("invalid JSON: {error}"),
                );
                return Vec::new();
            }
        };
        let parsed_actions = match shellcheck::parse_code_actions(&parsed, uri.as_str()) {
            Ok(parsed_actions) => parsed_actions,
            Err(error) => {
                log_command_error(
                    connection,
                    "code actions error",
                    run.argv(),
                    uri.as_str(),
                    &error,
                );
                return Vec::new();
            }
        };
        actions::filter_actions(parsed_actions, &params.range, &params.context.diagnostics)
            .into_iter()
            .map(CodeActionOrCommand::CodeAction)
            .collect()
    }

    fn hover_request(
        config: &Config,
        documents: &Documents,
        connection: &Connection,
        request: Request,
    ) {
        let Some((id, params)) =
            parse_request::<HoverParams>(connection, request, HoverRequest::METHOD)
        else {
            return;
        };
        let position = params.text_document_position_params.position;
        let reply = match run_document_command(
            config.hover.as_ref(),
            documents,
            connection,
            &params.text_document_position_params.text_document.uri,
            "hover error",
            false,
            Some((position.line, position.character)),
        ) {
            Some(run) => match hovers::parse_output(run.result.stdout()) {
                Ok(hover) => hover,
                Err(error) => {
                    log_command_error(
                        connection,
                        "hover error",
                        run.argv(),
                        uri_display(&run),
                        &error,
                    );
                    None
                }
            },
            None => None,
        };
        respond(connection, id, &reply);
    }

    fn definition_request(
        config: &Config,
        documents: &Documents,
        connection: &Connection,
        request: Request,
    ) {
        let Some((id, params)) =
            parse_request::<GotoDefinitionParams>(connection, request, GotoDefinition::METHOD)
        else {
            return;
        };
        let position = params.text_document_position_params.position;
        let outcome = location_outcome(
            config.definition.as_ref(),
            documents,
            connection,
            &params.text_document_position_params.text_document.uri,
            position,
            "definition error",
        );
        let value = match outcome {
            None | Some(locations::Locations::Missing) => serde_json::Value::Null,
            Some(locations::Locations::One(location)) => {
                serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                    .unwrap_or(serde_json::Value::Null)
            }
            Some(locations::Locations::Many(items)) => {
                serde_json::to_value(GotoDefinitionResponse::Array(items))
                    .unwrap_or(serde_json::Value::Null)
            }
        };
        respond(connection, id, &value);
    }

    fn references_request(
        config: &Config,
        documents: &Documents,
        connection: &Connection,
        request: Request,
    ) {
        let Some((id, params)) =
            parse_request::<ReferenceParams>(connection, request, References::METHOD)
        else {
            return;
        };
        let position = params.text_document_position.position;
        let outcome = location_outcome(
            config.references.as_ref(),
            documents,
            connection,
            &params.text_document_position.text_document.uri,
            position,
            "references error",
        );
        let reply: Vec<Location> = match outcome {
            None | Some(locations::Locations::Missing) => Vec::new(),
            Some(locations::Locations::One(location)) => vec![location],
            Some(locations::Locations::Many(items)) => items,
        };
        respond(connection, id, &reply);
    }

    fn document_symbols_request(
        config: &Config,
        documents: &Documents,
        connection: &Connection,
        request: Request,
    ) {
        let Some((id, params)) = parse_request::<DocumentSymbolParams>(
            connection,
            request,
            DocumentSymbolRequest::METHOD,
        ) else {
            return;
        };
        let reply = match run_document_command(
            config.document_symbols.as_ref(),
            documents,
            connection,
            &params.text_document.uri,
            "document symbols error",
            false,
            None,
        ) {
            Some(run) => match symbols::parse_output(run.result.stdout()) {
                Ok(list) => DocumentSymbolResponse::Nested(list),
                Err(error) => {
                    log_command_error(
                        connection,
                        "document symbols error",
                        run.argv(),
                        uri_display(&run),
                        &error,
                    );
                    DocumentSymbolResponse::Nested(Vec::new())
                }
            },
            None => DocumentSymbolResponse::Nested(Vec::new()),
        };
        respond(connection, id, &reply);
    }

    fn location_outcome(
        command: Option<&Command>,
        documents: &Documents,
        connection: &Connection,
        uri: &Uri,
        position: lsp_types::Position,
        title: &'static str,
    ) -> Option<locations::Locations> {
        let run = run_document_command(
            command,
            documents,
            connection,
            uri,
            title,
            false,
            Some((position.line, position.character)),
        )?;
        match locations::parse_output(run.result.stdout()) {
            Ok(parsed) => Some(parsed),
            Err(error) => {
                log_command_error(connection, title, run.argv(), uri_display(&run), &error);
                None
            }
        }
    }

    fn handle_notification(
        config: &Config,
        documents: &mut Documents,
        connection: &Connection,
        notification: Notification,
    ) {
        trace(config, &format!("notification {}", notification.method));
        let method = notification.method.clone();
        match method.as_str() {
            DidOpenTextDocument::METHOD => {
                if let Ok(params) =
                    notification.extract::<DidOpenTextDocumentParams>(DidOpenTextDocument::METHOD)
                {
                    documents.insert(
                        params.text_document.uri.as_str().to_string(),
                        params.text_document.text.clone(),
                    );
                    publish_diagnostics(config, documents, connection, &params.text_document.uri);
                }
            }
            DidChangeTextDocument::METHOD => {
                if let Ok(params) = notification
                    .extract::<DidChangeTextDocumentParams>(DidChangeTextDocument::METHOD)
                {
                    apply_changes(documents, &params.text_document.uri, params.content_changes);
                    if config.diagnostics_on_change {
                        publish_diagnostics(
                            config,
                            documents,
                            connection,
                            &params.text_document.uri,
                        );
                    }
                }
            }
            DidSaveTextDocument::METHOD => {
                if let Ok(params) =
                    notification.extract::<DidSaveTextDocumentParams>(DidSaveTextDocument::METHOD)
                {
                    publish_diagnostics(config, documents, connection, &params.text_document.uri);
                }
            }
            _ => {}
        }
    }

    // FULL sync is advertised, so clients send whole documents per change.
    fn apply_changes(
        documents: &mut Documents,
        uri: &Uri,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) {
        let entry = documents.entry(uri.as_str().to_string()).or_default();
        for change in changes {
            if change.range.is_none() {
                *entry = change.text;
            }
        }
    }

    fn publish_diagnostics(
        config: &Config,
        documents: &Documents,
        connection: &Connection,
        uri: &Uri,
    ) {
        let Some(command) = config.diagnostics.as_ref() else {
            return;
        };
        let Some(run) = run_document_command(
            Some(command),
            documents,
            connection,
            uri,
            "diagnostics command failed",
            true,
            None,
        ) else {
            return;
        };
        match diagnostics::parse_output(run.result.stdout()) {
            Ok(list) => {
                let _ = connection
                    .sender
                    .send(Message::Notification(Notification::new(
                        PublishDiagnostics::METHOD.to_string(),
                        PublishDiagnosticsParams {
                            uri: run.uri.clone(),
                            diagnostics: list,
                            version: None,
                        },
                    )));
            }
            Err(error) => log_command_error(
                connection,
                "diagnostics error",
                run.argv(),
                uri_display(&run),
                &error,
            ),
        }
    }

    fn run_document_command(
        command: Option<&Command>,
        documents: &Documents,
        connection: &Connection,
        uri: &Uri,
        title: &str,
        allow_failure_with_stdout: bool,
        position: Option<(u32, u32)>,
    ) -> Option<CommandRun> {
        let command = command?;
        let Some(source) = documents.get(uri.as_str()).cloned() else {
            log_command_error(
                connection,
                title,
                command.argv(),
                uri.as_str(),
                "document is not open",
            );
            return None;
        };
        let file_path = locations::file_path_of_uri(uri);
        let context = command::RenderContext::new(&file_path, uri.as_str(), position);
        let result = match command.execute(&source, &context) {
            Ok(result) => result,
            Err(error) => {
                log_command_error(connection, title, command.argv(), &file_path, &error);
                return None;
            }
        };
        let stdout_usable = allow_failure_with_stdout && !result.stdout().trim().is_empty();
        if result.status() != 0 && !stdout_usable {
            let error = format!(
                "exit status {}; stderr: {}",
                result.status(),
                result.stderr()
            );
            log_command_error(connection, title, command.argv(), &file_path, &error);
            return None;
        }
        Some(CommandRun {
            source,
            uri: uri.clone(),
            result,
        })
    }

    fn uri_display(run: &CommandRun) -> &str {
        run.uri.as_str()
    }

    impl CommandRun {
        fn source(&self) -> &str {
            &self.source
        }

        fn argv(&self) -> &[String] {
            self.result.argv()
        }
    }

    fn log_command_error(
        connection: &Connection,
        title: &str,
        argv: &[String],
        target: &str,
        error: &str,
    ) {
        let message = format!("{title}: command {argv:?} on {target}: {error}");
        eprintln!("{message}");
        let _ = connection
            .sender
            .send(Message::Notification(Notification::new(
                LogMessage::METHOD.to_string(),
                LogMessageParams {
                    typ: MessageType::LOG,
                    message,
                },
            )));
    }

    fn parse_request<P: DeserializeOwned>(
        connection: &Connection,
        request: Request,
        method: &str,
    ) -> Option<(RequestId, P)> {
        let id = request.id.clone();
        match request.extract::<P>(method) {
            Ok((_request_id, params)) => Some((id, params)),
            Err(error) => {
                let _ = connection.sender.send(Message::Response(Response::new_err(
                    id,
                    INVALID_PARAMS,
                    format!("invalid params for {method}: {error}"),
                )));
                None
            }
        }
    }

    fn respond<T: serde::Serialize>(connection: &Connection, id: RequestId, result: &T) {
        let response = match serde_json::to_value(result) {
            Ok(value) => Response::new_ok(id, value),
            Err(error) => Response::new_err(id, INVALID_PARAMS, error.to_string()),
        };
        let _ = connection.sender.send(Message::Response(response));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        actions, args, command, diagnostics, hovers, json_util, locations, positions, shellcheck,
        symbols,
    };

    // ---------------------------------------------------------------- command

    #[test]
    fn parses_commands_without_a_shell() {
        assert_eq!(
            command::Command::parse("ruff format --stdin-filename {file_path} -")
                .expect("parses")
                .argv(),
            ["ruff", "format", "--stdin-filename", "{file_path}", "-"]
        );
        assert_eq!(
            command::Command::parse("'a b' \"c d\"")
                .expect("parses")
                .argv(),
            ["a b", "c d"]
        );
        assert_eq!(
            command::Command::parse("").unwrap_err(),
            "command must not be empty"
        );
        assert_eq!(
            command::Command::parse("'unterminated").unwrap_err(),
            "command contains an unterminated quote"
        );
    }

    #[test]
    fn renders_known_placeholders_only() {
        let render = |part: &str, position: Option<(u32, u32)>| {
            let context = command::RenderContext::new("a.py", "file:///a.py", position);
            command::render_part(part, &context)
        };
        assert_eq!(render("{file_path}", None), "a.py");
        assert_eq!(render("--uri={uri}", None), "--uri=file:///a.py");
        assert_eq!(render("--line={line}", Some((0, 3))), "--line=0");
        assert_eq!(
            render("--line={line} --line1={line1}", Some((0, 3))),
            "--line=0 --line1=1"
        );
        assert_eq!(
            render("--character1={character1}", Some((4, 2))),
            "--character1=3"
        );
        assert_eq!(render("--line={line}", None), "--line=");
        assert_eq!(render("awk {print}", None), "awk {print}");
        assert_eq!(render("{file_path}:{line1}", Some((2, 0))), "a.py:3");
        assert_eq!(render("{file_path} {", None), "a.py {");
        assert_eq!(render("{unknown} {file_path}", None), "{unknown} a.py");
    }

    #[test]
    fn executes_commands_with_document_on_stdin() {
        let command = command::Command::parse("cat").expect("parses");
        let context = command::RenderContext::new("a.py", "file:///a.py", None);
        let result = command.execute("hello", &context).expect("runs");
        assert_eq!(result.status(), 0);
        assert_eq!(result.stdout(), "hello");

        let failing =
            command::Command::parse("sh -c 'cat >/dev/null; echo out; echo err >&2; exit 3'")
                .expect("parses");
        let result = failing.execute("", &context).expect("runs");
        assert_eq!(result.status(), 3);
        assert_eq!(result.stdout(), "out\n");
        assert_eq!(result.stderr(), "err\n");

        let missing = command::Command::parse("definitely-missing-binary-xyz").expect("parses");
        assert!(missing.execute("", &context).is_err());
    }

    // -------------------------------------------------------------- positions

    #[test]
    fn measures_utf16_lengths() {
        assert_eq!(positions::utf16_length("a"), 1);
        assert_eq!(positions::utf16_length("\u{1F600}"), 2);
    }

    #[test]
    fn builds_full_document_ranges() {
        let range = positions::full_document_range("one\ntwo");
        assert_eq!((range.start.line, range.start.character), (0, 0));
        assert_eq!((range.end.line, range.end.character), (1, 3));
        let range = positions::full_document_range("one\n");
        assert_eq!((range.end.line, range.end.character), (1, 0));
    }

    #[test]
    fn creates_and_compares_ranges() {
        let range = positions::create_range(1, 2, None, None).expect("creates");
        assert_eq!((range.start.line, range.start.character), (1, 2));
        assert_eq!((range.end.line, range.end.character), (1, 3));

        let closed = positions::create_range(1, 2, Some(1), Some(2)).expect("creates");
        assert_eq!(closed.start, closed.end);

        assert!(positions::create_range(2, 0, Some(1), Some(0)).is_err());

        let range_of = |start, end| lsp_types::Range {
            start: lsp_types::Position {
                line: 0,
                character: start,
            },
            end: lsp_types::Position {
                line: 0,
                character: end,
            },
        };
        assert!(positions::overlaps(&range_of(1, 3), &range_of(2, 4)));
        assert!(!positions::overlaps(&range_of(1, 3), &range_of(5, 6)));
    }

    // ------------------------------------------------------------ diagnostics

    type Summary = (
        u32,
        u32,
        String,
        Option<lsp_types::NumberOrString>,
        Option<lsp_types::DiagnosticSeverity>,
    );

    fn summarize(output: &str) -> Vec<Summary> {
        diagnostics::parse_output(output)
            .expect("parses")
            .into_iter()
            .map(|diagnostic| {
                (
                    diagnostic.range.start.line,
                    diagnostic.range.start.character,
                    diagnostic.message.clone(),
                    diagnostic.code.clone(),
                    diagnostic.severity,
                )
            })
            .collect()
    }

    #[test]
    fn parses_custom_json_diagnostics() {
        assert_eq!(
            summarize(r#"[{"line": 2, "character": 4, "message": "bad", "severity": "warning"}]"#),
            vec![(
                2,
                4,
                "bad".to_string(),
                None,
                Some(lsp_types::DiagnosticSeverity::WARNING)
            )]
        );
        assert_eq!(summarize(""), Vec::new());
    }

    #[test]
    fn parses_ruff_json_diagnostics() {
        assert_eq!(
            summarize(
                r#"[{"code": "F401", "location": {"row": 1, "column": 8}, "end_location": {"row": 1, "column": 10}, "message": "unused", "severity": "error"}]"#
            ),
            vec![(
                0,
                7,
                "unused".to_string(),
                Some(lsp_types::NumberOrString::String("F401".to_string())),
                Some(lsp_types::DiagnosticSeverity::ERROR)
            )]
        );
    }

    #[test]
    fn parses_gcc_style_diagnostics() {
        assert_eq!(
            summarize("-:1:6: warning: bad [SC1234]\n"),
            vec![(
                0,
                5,
                "bad".to_string(),
                Some(lsp_types::NumberOrString::String("SC1234".to_string())),
                Some(lsp_types::DiagnosticSeverity::WARNING)
            )]
        );
        let plain = &summarize("-:1:6: plain message\n")[0];
        assert_eq!(plain.4, Some(lsp_types::DiagnosticSeverity::ERROR));
    }

    #[test]
    fn parses_markdownlint_output() {
        let expected = (
            1u32,
            0u32,
            "First line".to_string(),
            Some(lsp_types::NumberOrString::String("MD041".to_string())),
            Some(lsp_types::DiagnosticSeverity::WARNING),
        );
        assert_eq!(
            summarize("README.md:2 MD041/first-line-heading First line\n"),
            vec![expected.clone()]
        );
        assert_eq!(
            summarize(
                r#"[{"filename": "README.md", "line": 2, "rule": "MD041", "description": "First line"}]"#
            ),
            vec![expected]
        );
    }

    #[test]
    fn parses_shellcheck_diagnostics() {
        assert_eq!(
            summarize(
                r#"{"comments": [{"line": 1, "endLine": 1, "column": 6, "endColumn": 8, "level": "warning", "code": 2154, "message": "x is referenced but not assigned."}]}"#
            ),
            vec![(
                0,
                5,
                "x is referenced but not assigned.".to_string(),
                Some(lsp_types::NumberOrString::Number(2154)),
                Some(lsp_types::DiagnosticSeverity::WARNING)
            )]
        );
    }

    #[test]
    fn rejects_unparseable_diagnostic_lines() {
        assert_eq!(
            diagnostics::parse_output("garbage line").unwrap_err(),
            "cannot parse diagnostic line: garbage line"
        );
    }

    #[test]
    fn parses_severities() {
        let parse = |value: serde_json::Value| diagnostics::severity(value);
        assert_eq!(
            parse(serde_json::json!("error")),
            Ok(lsp_types::DiagnosticSeverity::ERROR)
        );
        assert_eq!(
            parse(serde_json::json!("info")),
            Ok(lsp_types::DiagnosticSeverity::INFORMATION)
        );
        assert_eq!(
            parse(serde_json::json!("style")),
            Ok(lsp_types::DiagnosticSeverity::HINT)
        );
        assert_eq!(
            parse(serde_json::json!(2)),
            Ok(lsp_types::DiagnosticSeverity::WARNING)
        );
        assert_eq!(
            parse(serde_json::json!("bogus")),
            Err("unsupported diagnostic severity: bogus".to_string())
        );
        assert_eq!(
            parse(serde_json::json!(true)),
            Err("diagnostic severity must be a string or integer".to_string())
        );
    }

    // ------------------------------------------------------------- shellcheck

    #[test]
    fn builds_quick_fix_actions_from_shellcheck_fixes() {
        let output = r#"{"comments": [
            {"line": 1, "endLine": 1, "column": 6, "endColumn": 8, "level": "warning",
             "code": 2154, "message": "use double quotes",
             "fix": {"replacements": [{"line": 1, "column": 6, "endLine": 1, "endColumn": 7, "replacement": "\""}]}}
        ]}"#;
        let value: serde_json::Value = serde_json::from_str(output).expect("json");
        let pairs = shellcheck::from_json(&value).expect("parses");
        assert_eq!(pairs.len(), 1);
        let action = pairs[0].1.as_ref().expect("has fix");
        assert_eq!(action.title, "use double quotes");
        let edit = action.edit.as_ref().expect("has edit");
        assert_eq!(edit.changes.as_ref().expect("changes").len(), 1);
        assert_eq!(action.is_preferred, Some(true));

        let without_fix = r#"{"comments": [{"line": 1, "endLine": 1, "column": 1, "endColumn": 2, "level": "error", "code": 1, "message": "no fix here"}]}"#;
        let value: serde_json::Value = serde_json::from_str(without_fix).expect("json");
        let pairs = shellcheck::from_json(&value).expect("parses");
        assert!(pairs[0].1.is_none());
    }

    // -------------------------------------------------------------- locations

    #[test]
    fn parses_locations() {
        let parsed =
            locations::parse_output(r#"{"uri": "file:///a.py", "line": 1, "character": 2}"#)
                .expect("parses");
        match parsed {
            locations::Locations::One(location) => {
                assert_eq!(
                    (location.range.start.line, location.range.start.character),
                    (1, 2)
                );
            }
            other => panic!("expected one location, got {other:?}"),
        }

        assert!(matches!(
            locations::parse_output("null").expect("parses"),
            locations::Locations::Missing
        ));

        let parsed =
            locations::parse_output(r#"[{"uri": "file:///a.py"}, {"uri": "file:///b.py"}]"#)
                .expect("parses");
        match parsed {
            locations::Locations::Many(items) => assert_eq!(items.len(), 2),
            other => panic!("expected many locations, got {other:?}"),
        }

        let parsed = locations::parse_output(r#"{"file_path": "/tmp/a.py"}"#).expect("parses");
        match parsed {
            locations::Locations::One(location) => {
                assert_eq!(location.uri.as_str(), "file:///tmp/a.py");
            }
            other => panic!("expected one location, got {other:?}"),
        }

        assert_eq!(
            locations::parse_output("{}").unwrap_err(),
            "location must contain uri or file_path"
        );
    }

    // ------------------------------------------------------------------ hover

    #[test]
    fn parses_hover_output() {
        let hover = hovers::parse_output("hello")
            .expect("parses")
            .expect("hover");
        assert!(matches!(
            hover.contents,
            lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                ..
            })
        ));
        assert_eq!(hovers::parse_output("").expect("parses"), None);

        let hover = hovers::parse_output(r#"{"contents": "hello", "kind": "plaintext"}"#)
            .expect("parses")
            .expect("hover");
        match hover.contents {
            lsp_types::HoverContents::Markup(content) => {
                assert_eq!(content.kind, lsp_types::MarkupKind::PlainText);
                assert_eq!(content.value, "hello");
            }
            other => panic!("expected markup contents, got {other:?}"),
        }

        let hover = hovers::parse_output(
            r#"{"contents": "hi", "kind": "markdown", "line": 3, "character": 1}"#,
        )
        .expect("parses")
        .expect("hover");
        let range = hover.range.expect("range");
        assert_eq!((range.start.line, range.start.character), (3, 1));

        assert_eq!(
            hovers::parse_output(r#"{"contents": 42}"#).unwrap_err(),
            "hover contents must be a string"
        );
    }

    // ---------------------------------------------------------------- symbols

    #[test]
    fn parses_document_symbols() {
        let parsed = symbols::parse_output(
            r#"[{"name": "main", "kind": "function", "line": 0, "character": 0}]"#,
        )
        .expect("parses");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "main");
        assert_eq!(parsed[0].kind, lsp_types::SymbolKind::FUNCTION);

        let parsed = symbols::parse_output(
            r#"[{"name": "Outer", "kind": 5, "detail": null, "line": 1, "character": 4,
                "selection_line": 1, "selection_character": 10,
                "children": [{"name": "inner", "line": 2, "character": 8}]}]"#,
        )
        .expect("parses");
        assert_eq!(parsed[0].kind, lsp_types::SymbolKind::CLASS);
        assert_eq!(parsed[0].detail, None);
        let selection = parsed[0].selection_range;
        assert_eq!((selection.start.line, selection.start.character), (1, 10));
        let child = parsed[0].children.as_ref().expect("children");
        assert_eq!(child[0].name, "inner");

        assert_eq!(symbols::parse_output("").expect("parses"), Vec::new());
        assert_eq!(
            symbols::parse_output(r#"[{"name": "", "line": 0}]"#).unwrap_err(),
            "name must be a non-empty string"
        );
    }

    // ---------------------------------------------------------------- actions

    #[test]
    fn filters_quick_fix_kinds() {
        assert!(actions::allows_quick_fix(None));
        assert!(actions::allows_quick_fix(Some(&[
            lsp_types::CodeActionKind::QUICKFIX
        ])));
        assert!(!actions::allows_quick_fix(Some(&[
            lsp_types::CodeActionKind::SOURCE
        ])));
    }

    #[test]
    fn filters_actions_by_range_and_context() {
        let range_of = |start: u32, end: u32| lsp_types::Range {
            start: lsp_types::Position {
                line: 0,
                character: start,
            },
            end: lsp_types::Position {
                line: 0,
                character: end,
            },
        };
        let action_with = |start: u32, end: u32| lsp_types::CodeAction {
            title: "t".to_string(),
            diagnostics: Some(vec![lsp_types::Diagnostic {
                range: range_of(start, end),
                ..lsp_types::Diagnostic::default()
            }]),
            ..lsp_types::CodeAction::default()
        };
        let near = action_with(1, 3);
        let far = action_with(50, 60);

        let kept = actions::filter_actions(vec![near.clone(), far.clone()], &range_of(2, 4), &[]);
        assert_eq!(kept.len(), 1);

        let context = lsp_types::Diagnostic {
            range: range_of(50, 60),
            ..lsp_types::Diagnostic::default()
        };
        let kept = actions::filter_actions(
            vec![near.clone(), far.clone()],
            &range_of(2, 4),
            std::slice::from_ref(&context),
        );
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0], far);
    }

    // ------------------------------------------------------------------- args

    fn argv(arguments: &[&str]) -> Vec<String> {
        arguments
            .iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    #[test]
    fn parses_arguments() {
        let parsed = args::Args::parse(&argv(&[
            "--format-command",
            "cat",
            "--diagnostics-on-change",
        ]))
        .expect("parses");
        assert!(parsed.diagnostics_on_change);

        let parsed = args::Args::parse(&argv(&[])).expect("parses");
        assert_eq!(parsed.log_level, args::LogLevel::Warning);

        assert_eq!(
            args::Args::parse(&argv(&["--format-command"])).unwrap_err(),
            "argument --format-command: expected one argument"
        );
        assert_eq!(
            args::Args::parse(&argv(&["--bogus"])).unwrap_err(),
            "unknown argument: --bogus"
        );
        assert_eq!(
            args::Args::parse(&argv(&["--log-level", "LOUD"])).unwrap_err(),
            "argument --log-level: invalid level 'LOUD'"
        );
        assert!(args::Args::parse(&argv(&["--help"])).is_err());
    }

    #[test]
    fn rejects_configurations_without_any_command() {
        let parsed = args::Args::parse(&argv(&[])).expect("parses");
        assert_eq!(
            parsed.config().unwrap_err(),
            "at least one LSP command option is required"
        );
        let parsed =
            args::Args::parse(&argv(&["--document-symbols-command", "echo []"])).expect("parses");
        assert!(parsed.config().is_ok());
    }

    // -------------------------------------------------------------- json util

    #[test]
    fn validates_uint_fields() {
        let value: serde_json::Value =
            serde_json::from_str(r#"{"n": 3, "s": "7", "neg": -1, "f": 1.5, "b": true}"#)
                .expect("json");
        let object = json_util::as_object(&value, "context").expect("object");
        assert_eq!(
            json_util::optional_uint(object, "n").expect("uint"),
            Some(3)
        );
        assert_eq!(
            json_util::optional_uint(object, "s").expect("uint"),
            Some(7)
        );
        assert!(json_util::optional_uint(object, "neg").is_err());
        assert!(json_util::optional_uint(object, "f").is_err());
        assert!(json_util::optional_uint(object, "b").is_err());
        assert_eq!(
            json_util::optional_uint(object, "missing").expect("none"),
            None
        );
    }
}
