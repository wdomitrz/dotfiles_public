#!/usr/bin/env rscript.sh
// Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
// AGPL License

const USAGE: &str = "usage: ./url.rs [-h | --help] <url>";

struct Args {
    url: String,
}

impl Args {
    fn parse(argv: &[String]) -> Result<Self, String> {
        let mut urls = Vec::new();
        for arg in argv {
            match arg.as_str() {
                "-h" | "--help" => return Err(USAGE.to_string()),
                other if other.starts_with('-') => {
                    return Err(format!("unknown argument: {other}"));
                }
                other => urls.push(other.to_string()),
            }
        }
        let [url] = urls.as_slice() else {
            return Err(format!(
                "expected exactly one positional argument, got {}",
                urls.len()
            ));
        };
        Ok(Self { url: url.clone() })
    }

    fn run(&self, out: &mut dyn std::io::Write) -> std::io::Result<i32> {
        writeln!(out, "{}", render(&parse_url(&self.url)))?;
        Ok(0)
    }
}

#[derive(Debug, PartialEq)]
enum QueryValue {
    Single(String),
    Multiple(Vec<String>),
}

struct Url {
    original: String,
    protocol: Option<String>,
    username: Option<String>,
    password: Option<String>,
    hostname: Option<String>,
    port: Option<String>,
    path: Option<String>,
    query: Vec<(String, QueryValue)>,
    hash: Option<String>,
}

fn parse_url(raw: &str) -> Url {
    let mut protocol = None;
    // `urlsplit` ignores leading/trailing C0 controls and spaces, but keeps
    // the original string in the output.
    let url = raw.trim_matches(|c: char| c <= ' ');
    let (rest, hash) = split_at_first(url, '#');
    let (rest, raw_query) = split_at_first(rest, '?');
    // `urlparse` recognises a scheme before the first ':' even without an
    // authority (`mailto:a@b`); an authority only follows a '//'.
    let (authority, path) = match split_scheme(rest) {
        Some((scheme, after_scheme)) => {
            protocol = Some(scheme.to_lowercase());
            match after_scheme.strip_prefix("//") {
                Some(after_authority) => {
                    let (host_portion, path) = split_at_slash(after_authority);
                    (Some(host_portion), path)
                }
                None => (None, Some(after_scheme)),
            }
        }
        // `urlparse` also grants an authority to scheme-relative `//host/path` URLs.
        None if rest.starts_with("//") => {
            let (host_portion, path) = split_at_slash(&rest[2..]);
            (Some(host_portion), path)
        }
        None => (None, Some(rest)),
    };
    let (username, password, host_port) = match authority {
        Some(auth) => {
            let (userinfo, host_port) = auth.rsplit_once('@').unwrap_or(("", auth));
            let (user, pass) = userinfo
                .split_once(':')
                .map_or((userinfo, None), |(user, pass)| (user, Some(pass)));
            (
                non_empty(Some(user.to_string())),
                non_empty(pass.map(str::to_string)),
                host_port,
            )
        }
        None => (None, None, ""),
    };
    let (hostname, port) = match host_port.strip_prefix('[') {
        Some(ipv6_tail) => match ipv6_tail.split_once(']') {
            Some((host, tail)) => (
                host.to_string(),
                tail.strip_prefix(':').filter(|p| !p.is_empty()),
            ),
            None => (host_port.to_string(), None),
        },
        None => match host_port.rsplit_once(':') {
            Some((host, port)) => (host.to_string(), Some(port)),
            None => (host_port.to_string(), None),
        },
    };
    Url {
        original: raw.to_string(),
        protocol: non_empty(protocol),
        username: non_empty(username),
        password: non_empty(password),
        hostname: non_empty(Some(hostname.to_lowercase())),
        port: non_empty(port.map(str::to_string)),
        path: non_empty(path.map(str::to_string)),
        query: parse_query(raw_query.unwrap_or_default()),
        hash: non_empty(hash.map(str::to_string)),
    }
}

fn split_scheme(rest: &str) -> Option<(String, &str)> {
    let (candidate, tail) = rest.split_once(':')?;
    let looks_like_scheme = candidate.starts_with(|c: char| c.is_ascii_alphabetic())
        && candidate
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    looks_like_scheme.then(|| (candidate.to_string(), tail))
}

fn parse_query(raw: &str) -> Vec<(String, QueryValue)> {
    let mut grouped: Vec<(String, Vec<String>)> = Vec::new();
    for pair in raw.split('&').filter(|part| !part.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let (key, value) = (percent_decode_plus(key), percent_decode_plus(value));
        match grouped.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, values)) => values.push(value),
            None => grouped.push((key, vec![value])),
        }
    }
    grouped
        .into_iter()
        .map(|(key, mut values)| {
            if values.len() == 1 {
                (key, QueryValue::Single(values.remove(0)))
            } else {
                (key, QueryValue::Multiple(values))
            }
        })
        .collect()
}

fn percent_decode_plus(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => decoded.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
                match hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                    Some(byte) => {
                        decoded.push(byte);
                        i += 2;
                    }
                    None => decoded.push(b'%'),
                }
            }
            byte => decoded.push(byte),
        }
        i += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn split_at_slash(s: &str) -> (&str, Option<&str>) {
    match s.find('/') {
        Some(index) => (&s[..index], Some(&s[index..])),
        None => (s, None),
    }
}

fn split_at_first(s: &str, separator: char) -> (&str, Option<&str>) {
    match s.split_once(separator) {
        Some((head, tail)) => (head, Some(tail)),
        None => (s, None),
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

fn render(url: &Url) -> String {
    let query = if url.query.is_empty() {
        None
    } else {
        Some(render_query(&url.query))
    };
    // `urlparse.port` yields an `int`, so render valid ports as JSON numbers.
    let port = url.port.as_deref().map(|port| match port.parse::<u16>() {
        Ok(number) => number.to_string(),
        Err(_) => json_string(port),
    });
    let fields: [(&str, Option<String>); 9] = [
        ("original", Some(json_string(&url.original))),
        ("protocol", url.protocol.as_deref().map(json_string)),
        ("username", url.username.as_deref().map(json_string)),
        ("password", url.password.as_deref().map(json_string)),
        ("hostname", url.hostname.as_deref().map(json_string)),
        ("port", port),
        ("path", url.path.as_deref().map(json_string)),
        ("query", query),
        ("hash", url.hash.as_deref().map(json_string)),
    ];
    let entries: Vec<String> = fields
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| format!("  \"{key}\": {value}")))
        .collect();
    format!("{{\n{}\n}}", entries.join(",\n"))
}

fn render_query(query: &[(String, QueryValue)]) -> String {
    let entries: Vec<String> = query
        .iter()
        .map(|(key, value)| {
            let value = match value {
                QueryValue::Single(single) => json_string(single),
                QueryValue::Multiple(multiple) => {
                    let items: Vec<String> = multiple
                        .iter()
                        .map(|item| format!("      {}", json_string(item)))
                        .collect();
                    format!("[\n{}\n    ]", items.join(",\n"))
                }
            };
            format!("    {}: {}", json_string(key), value)
        })
        .collect();
    format!("{{\n{}\n  }}", entries.join(",\n"))
}

fn json_string(value: &str) -> String {
    // Match `json.dumps`' default `ensure_ascii`: everything outside
    // printable ASCII becomes `\uXXXX` (surrogate pairs above the BMP).
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{8}' => escaped.push_str("\\b"),
            '\u{c}' => escaped.push_str("\\f"),
            other if (other as u32) < 0x20 || (other as u32) > 0x7e => {
                use std::fmt::Write as _;
                let code = other as u32;
                if code <= 0xffff {
                    let _ = write!(escaped, "\\u{code:04x}");
                } else {
                    let offset = code - 0x1_0000;
                    let _ = write!(
                        escaped,
                        "\\u{:04x}\\u{:04x}",
                        0xd800 + (offset >> 10),
                        0xdc00 + (offset & 0x3ff)
                    );
                }
            }
            other => escaped.push(other),
        }
    }
    escaped.push('"');
    escaped
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
            eprintln!("url: {message}");
            return ExitCode::from(2);
        }
    };
    match parsed.run(&mut std::io::stdout().lock()) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            eprintln!("url: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_query, parse_url, render, render_query, Args};

    fn run(args: &Args) -> String {
        let mut buffer = Vec::new();
        assert_eq!(args.run(&mut buffer).expect("runs"), 0);
        String::from_utf8(buffer).expect("utf8")
    }

    #[test]
    fn parses_full_url() {
        let args =
            Args::parse(&["https://user:pass@example.com:8080/a/b?x=1&y=2&x=3#frag".to_string()])
                .expect("parses");
        assert_eq!(
            run(&args),
            [
                "{",
                "  \"original\": \"https://user:pass@example.com:8080/a/b?x=1&y=2&x=3#frag\",",
                "  \"protocol\": \"https\",",
                "  \"username\": \"user\",",
                "  \"password\": \"pass\",",
                "  \"hostname\": \"example.com\",",
                "  \"port\": 8080,",
                "  \"path\": \"/a/b\",",
                "  \"query\": {",
                "    \"x\": [",
                "      \"1\",",
                "      \"3\"",
                "    ],",
                "    \"y\": \"2\"",
                "  },",
                "  \"hash\": \"frag\"",
                "}",
            ]
            .join("\n")
                + "\n"
        );
    }

    #[test]
    fn omits_missing_and_empty_fields() {
        let args = Args::parse(&["mailto:someone@example.com".to_string()]).expect("parses");
        assert_eq!(
            run(&args),
            [
                "{",
                "  \"original\": \"mailto:someone@example.com\",",
                "  \"protocol\": \"mailto\",",
                "  \"path\": \"someone@example.com\"",
                "}",
            ]
            .join("\n")
                + "\n"
        );
    }

    #[test]
    fn normalises_like_urlparse() {
        assert_eq!(
            render(&parse_url("  HTTP://EXAMPLE.COM:8080/x \t")),
            [
                "{",
                "  \"original\": \"  HTTP://EXAMPLE.COM:8080/x \\t\",",
                "  \"protocol\": \"http\",",
                "  \"hostname\": \"example.com\",",
                "  \"port\": 8080,",
                "  \"path\": \"/x\"",
                "}",
            ]
            .join("\n")
        );
    }

    #[test]
    fn escapes_non_ascii_like_json_dumps() {
        let query = parse_query("e=%e2%82%ac&emoji=%f0%9f%98%80");
        assert_eq!(
            render_query(&query),
            "{\n    \"e\": \"\\u20ac\",\n    \"emoji\": \"\\ud83d\\ude00\"\n  }"
        );
    }

    #[test]
    fn decodes_and_groups_query_parameters() {
        let query = parse_query("a=1&a=&b=x%20y&c=z+w&=empty-key");
        let [a, b, c, empty]: &[(String, super::QueryValue)] = query.as_slice() else {
            panic!("unexpected query shape")
        };
        assert_eq!(a.0, "a");
        assert_eq!(b.1, super::QueryValue::Single("x y".to_string()));
        assert_eq!(c.1, super::QueryValue::Single("z w".to_string()));
        assert_eq!(empty.0, "");
    }

    #[test]
    fn rejects_bad_arguments() {
        assert!(Args::parse(&[]).is_err());
        assert!(Args::parse(&["a".to_string(), "b".to_string()]).is_err());
        assert!(Args::parse(&["--bogus".to_string()]).is_err());
    }

    #[test]
    fn renders_minimal_url() {
        assert_eq!(
            render(&parse_url("http://h")),
            "{\n  \"original\": \"http://h\",\n  \"protocol\": \"http\",\n  \"hostname\": \"h\"\n}"
        );
    }
}
