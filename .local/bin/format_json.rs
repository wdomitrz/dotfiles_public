#!/usr/bin/env rscript.sh
// Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
// AGPL License

//# dependencies:
//# serde_json = { version = "1", features = ["preserve_order"] }

//! Pretty-print JSON from stdin, keeping containers on a single line when
//! they fit within the configured line length.

use serde_json::Value;
use std::io::Write;

const USAGE: &str =
    "usage: ./format_json.rs [--indent N] [--sort-keys | --no-sort-keys] [--line-length N]";

struct Args {
    indent: usize,
    sort_keys: bool,
    line_length: usize,
}

impl Args {
    fn parse(argv: &[String]) -> Result<Self, String> {
        let mut indent = 2;
        let mut sort_keys = false;
        let mut line_length = 80;
        let mut i = 0;
        while i < argv.len() {
            match argv[i].as_str() {
                "--indent" => {
                    i += 1;
                    indent = Self::parse_usize("--indent", argv.get(i))?;
                }
                "--line-length" => {
                    i += 1;
                    line_length = Self::parse_usize("--line-length", argv.get(i))?;
                }
                "--sort-keys" => sort_keys = true,
                "--no-sort-keys" => sort_keys = false,
                "-h" | "--help" => return Err(USAGE.to_string()),
                other => return Err(format!("unknown argument: {other}")),
            }
            i += 1;
        }
        Ok(Self {
            indent,
            sort_keys,
            line_length,
        })
    }

    fn parse_usize(name: &str, raw: Option<&String>) -> Result<usize, String> {
        let raw = raw.ok_or_else(|| format!("argument {name}: expected one argument"))?;
        raw.parse()
            .map_err(|_| format!("argument {name}: invalid integer value: '{raw}'"))
    }

    fn run(&self, input: &str, out: &mut dyn Write) -> Result<i32, String> {
        let mut data: Value =
            serde_json::from_str(input).map_err(|error| format!("invalid JSON: {error}"))?;
        if self.sort_keys {
            sort_keys(&mut data);
        }
        let dumper = JsonDumper {
            indent: " ".repeat(self.indent),
            line_length: self.line_length,
        };
        writeln!(out, "{}", dumper.dump(&data)).map_err(|error| error.to_string())?;
        Ok(0)
    }
}

fn sort_keys(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                sort_keys(item);
            }
        }
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = std::mem::take(map).into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            for (_, value) in &mut entries {
                sort_keys(value);
            }
            *map = entries.into_iter().collect();
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

/// A rendering of one node in two flavors: flattened onto a single line, or
/// fully expanded with one element per line.
struct PartialDump {
    single_lined: String,
    expanded: String,
}

struct JsonDumper {
    indent: String,
    line_length: usize,
}

impl JsonDumper {
    fn dump(&self, value: &Value) -> String {
        let partial = self.dump_node(value, "");
        if partial.single_lined.len() <= self.line_length {
            partial.single_lined
        } else {
            partial.expanded
        }
    }

    fn dump_node(&self, value: &Value, indent: &str) -> PartialDump {
        match value {
            Value::Array(items) => {
                let child_indent = self.child_of(indent);
                let entries: Vec<(String, PartialDump)> = items
                    .iter()
                    .map(|item| (String::new(), self.dump_node(item, &child_indent)))
                    .collect();
                self.combine(("[", "]"), ("[", "]"), &entries, indent)
            }
            Value::Object(entries) => {
                let child_indent = self.child_of(indent);
                let pairs: Vec<(String, PartialDump)> = entries
                    .iter()
                    .map(|(key, value)| {
                        let prefix =
                            format!("{}: ", serde_json::to_string(key).expect("key serializes"));
                        (prefix, self.dump_node(value, &child_indent))
                    })
                    .collect();
                self.combine(("{ ", " }"), ("{", "}"), &pairs, indent)
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                let text = serde_json::to_string(value).expect("scalar serializes");
                PartialDump {
                    single_lined: text.clone(),
                    expanded: text,
                }
            }
        }
    }

    fn child_of(&self, indent: &str) -> String {
        indent.to_string() + &self.indent
    }

    /// Combines per-entry renderings into the container's two forms. Each
    /// entry carries its line prefix (`""` for array elements, `"\"key\": "`
    /// for members), which counts toward its fit budget together with one
    /// character reserved after every entry but the last for the comma.
    fn combine(
        &self,
        single: (&str, &str),
        expanded: (&str, &str),
        entries: &[(String, PartialDump)],
        indent: &str,
    ) -> PartialDump {
        if entries.is_empty() {
            let text = format!("{}{}", expanded.0, expanded.1);
            return PartialDump {
                single_lined: text.clone(),
                expanded: text,
            };
        }
        let child_indent = self.child_of(indent);
        let single_lined = format!(
            "{}{}{}",
            single.0,
            entries
                .iter()
                .map(|(prefix, entry)| format!("{prefix}{}", entry.single_lined))
                .collect::<Vec<String>>()
                .join(", "),
            single.1,
        );
        let lines = entries
            .iter()
            .enumerate()
            .map(|(i, (prefix, entry))| {
                let last = i + 1 == entries.len();
                let allowed = self
                    .line_length
                    .saturating_sub(child_indent.len() + prefix.len() + usize::from(!last));
                format!("{child_indent}{prefix}{}", Self::choose(entry, allowed))
            })
            .collect::<Vec<String>>()
            .join(",\n");
        PartialDump {
            single_lined,
            expanded: format!("{}\n{lines}\n{indent}{}", expanded.0, expanded.1),
        }
    }

    fn choose(partial: &PartialDump, allowed: usize) -> &str {
        if partial.single_lined.len() <= allowed {
            &partial.single_lined
        } else {
            &partial.expanded
        }
    }
}

fn main() -> std::process::ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let parsed = match Args::parse(&argv) {
        Ok(parsed) => parsed,
        Err(message) if message.starts_with("usage:") => {
            println!("{message}");
            return std::process::ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("format_json: {message}");
            return std::process::ExitCode::from(2);
        }
    };
    let mut input = String::new();
    if let Err(error) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input) {
        eprintln!("format_json: {error}");
        return std::process::ExitCode::FAILURE;
    }
    let mut out = std::io::stdout().lock();
    match parsed.run(&input, &mut out) {
        Ok(code) => std::process::ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(message) => {
            let _ = out.flush();
            eprintln!("format_json: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, JsonDumper};

    fn run_with(argv: &[&str], input: &str) -> Result<String, String> {
        let argv: Vec<String> = argv.iter().map(|arg| (*arg).to_string()).collect();
        let parsed = Args::parse(&argv)?;
        let mut buffer = Vec::new();
        parsed.run(input, &mut buffer)?;
        Ok(String::from_utf8(buffer).expect("utf8"))
    }

    fn format_ok(argv: &[&str], input: &str) -> String {
        run_with(argv, input).expect("runs")
    }

    #[test]
    fn defaults() {
        let argv: Vec<String> = vec![];
        let parsed = Args::parse(&argv).expect("parses");
        assert_eq!(parsed.indent, 2);
        assert!(!parsed.sort_keys);
        assert_eq!(parsed.line_length, 80);
    }

    #[test]
    fn accepts_flags() {
        let argv: Vec<String> = ["--indent", "4", "--sort-keys", "--line-length", "40"]
            .iter()
            .map(|arg| (*arg).to_string())
            .collect();
        let parsed = Args::parse(&argv).expect("parses");
        assert_eq!(parsed.indent, 4);
        assert!(parsed.sort_keys);
        assert_eq!(parsed.line_length, 40);

        let argv: Vec<String> = ["--no-sort-keys"]
            .iter()
            .map(|arg| (*arg).to_string())
            .collect();
        assert!(!Args::parse(&argv).expect("parses").sort_keys);
    }

    #[test]
    fn rejects_bad_arguments() {
        assert!(Args::parse(&["--bogus".to_string()]).is_err());
        assert!(Args::parse(&["--indent".to_string()]).is_err());
        assert!(Args::parse(&["--indent".to_string(), "x".to_string()]).is_err());
        assert!(Args::parse(&["--indent".to_string(), "-1".to_string()]).is_err());
    }

    #[test]
    fn helps() {
        let result = run_with(&["--help"], "").unwrap_err();
        assert!(result.starts_with("usage:"));
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(run_with(&[], "{").is_err());
        assert!(run_with(&[], "").is_err());
        assert!(run_with(&[], "{} {}").is_err());
    }

    #[test]
    fn keeps_scalars_and_empty_containers_on_one_line() {
        assert_eq!(format_ok(&[], "42"), "42\n");
        assert_eq!(format_ok(&[], "-3.5"), "-3.5\n");
        assert_eq!(format_ok(&[], "\"hi\""), "\"hi\"\n");
        assert_eq!(format_ok(&[], "null"), "null\n");
        assert_eq!(format_ok(&[], "true"), "true\n");
        assert_eq!(format_ok(&[], "[]"), "[]\n");
        assert_eq!(format_ok(&[], "{}"), "{}\n");
    }

    #[test]
    fn keeps_fitting_containers_on_one_line() {
        assert_eq!(
            format_ok(&[], "{\"a\": 1, \"b\": [1, 2]}"),
            "{ \"a\": 1, \"b\": [1, 2] }\n"
        );
        assert_eq!(
            format_ok(&["--line-length", "6"], "[1, 2]"),
            "[1, 2]\n",
            "exactly at the limit still fits"
        );
    }

    #[test]
    fn expands_oversized_containers() {
        assert_eq!(
            format_ok(&["--line-length", "5"], "[1, 2]"),
            "[\n  1,\n  2\n]\n"
        );
        assert_eq!(
            format_ok(&["--line-length", "5"], "{\"a\": 1}"),
            "{\n  \"a\": 1\n}\n"
        );
    }

    #[test]
    fn expands_children_that_do_not_fit_their_budget() {
        // The inner object fits on a line by itself, but not next to its key.
        // Its own children still get their own fit check.
        let input = "{\"key\": {\"inner\": [1]}}";
        assert_eq!(
            format_ok(&["--line-length", "20"], input),
            "{\n  \"key\": {\n    \"inner\": [1]\n  }\n}\n"
        );
    }

    #[test]
    fn honors_indent_option() {
        assert_eq!(
            format_ok(&["--indent", "4", "--line-length", "5"], "[1, 2]"),
            "[\n    1,\n    2\n]\n"
        );
    }

    #[test]
    fn sorts_keys_when_asked() {
        assert_eq!(
            format_ok(&["--sort-keys"], "{\"b\": 1, \"a\": {\"y\": 2, \"x\": 3}}"),
            "{ \"a\": { \"x\": 3, \"y\": 2 }, \"b\": 1 }\n"
        );
        assert_eq!(
            format_ok(&["--no-sort-keys"], "{\"b\": 1, \"a\": 2}"),
            "{ \"b\": 1, \"a\": 2 }\n",
            "insertion order is preserved by default"
        );
    }

    #[test]
    fn dumps_use_json_escaping() {
        assert_eq!(
            format_ok(&[], "{\"a\\nb\": \"c\\\"d\"}"),
            "{ \"a\\nb\": \"c\\\"d\" }\n"
        );
    }

    #[test]
    fn keeps_strings_verbatim() {
        assert_eq!(format_ok(&[], "[\"żć 🎉\"]"), "[\"żć 🎉\"]\n");
    }

    #[test]
    fn dumps_nested_mixed_structures() {
        let input = "{\"users\": [{\"name\": \"Ada\", \"langs\": [\"rs\", \"py\"]}], \"total\": 2}";
        let expected_single = "{ \"users\": [{ \"name\": \"Ada\", \"langs\": [\"rs\", \"py\"] }], \
                               \"total\": 2 }\n";
        assert_eq!(format_ok(&[], input), expected_single);
        assert_ne!(
            format_ok(&["--line-length", "10"], input),
            expected_single,
            "long documents expand"
        );
    }

    #[test]
    fn dumper_picks_expanded_form_for_long_documents() {
        let dumper = JsonDumper {
            indent: "  ".to_string(),
            line_length: 8,
        };
        let value: serde_json::Value = serde_json::from_str("[1, 2, 3]").expect("parses");
        assert_eq!(dumper.dump(&value), "[\n  1,\n  2,\n  3\n]");
    }
}
