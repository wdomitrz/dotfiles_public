#!/usr/bin/env rscript.sh
// Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
// AGPL License

const USAGE: &str = "usage: ./nato.rs WORDS... | ./nato.rs -";

use std::io::BufRead;

struct Args {
    words: Vec<String>,
}

impl Args {
    fn parse(argv: &[String]) -> Result<Self, String> {
        let mut words = Vec::new();
        for arg in argv {
            match arg.as_str() {
                "-h" | "--help" => return Err(USAGE.to_string()),
                other if other == "-" || !other.starts_with('-') => {
                    words.push(other.to_string());
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }
        if words.is_empty() {
            return Err(format!(
                "the following arguments are required: words\n{USAGE}"
            ));
        }
        Ok(Self { words })
    }

    fn run(
        &self,
        input: &mut dyn std::io::BufRead,
        out: &mut dyn std::io::Write,
    ) -> std::io::Result<i32> {
        if self.words == ["-"] {
            for line in input.lines() {
                writeln!(out, "{}", Self::nato_convert(&line?))?;
            }
        } else {
            writeln!(out, "{}", Self::nato_convert(&self.words.join(" ")))?;
        }
        Ok(0)
    }

    fn nato_convert(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| nato_word(c).map_or_else(|| c.to_string(), str::to_string))
            .collect::<Vec<_>>()
            .join("\t")
    }
}

fn nato_word(c: char) -> Option<&'static str> {
    Some(match c {
        '0' => "Zero",
        '1' => "One",
        '2' => "Two",
        '3' => "Three",
        '4' => "Four",
        '5' => "Five",
        '6' => "Six",
        '7' => "Seven",
        '8' => "Eight",
        '9' => "Nine",
        'a' => "Alfa",
        'b' => "Bravo",
        'c' => "Charlie",
        'd' => "Delta",
        'e' => "Echo",
        'f' => "Foxtrot",
        'g' => "Golf",
        'h' => "Hotel",
        'i' => "India",
        'j' => "Juliett",
        'k' => "Kilo",
        'l' => "Lima",
        'm' => "Mike",
        'n' => "November",
        'o' => "Oscar",
        'p' => "Papa",
        'q' => "Quebec",
        'r' => "Romeo",
        's' => "Sierra",
        't' => "Tango",
        'u' => "Uniform",
        'v' => "Victor",
        'w' => "Whiskey",
        'x' => "X-ray",
        'y' => "Yankee",
        'z' => "Zulu",
        _ => return None,
    })
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
            eprintln!("nato.rs: {message}");
            return ExitCode::from(2);
        }
    };
    match parsed.run(&mut std::io::stdin().lock(), &mut std::io::stdout().lock()) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            eprintln!("nato.rs: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Args;

    fn parse(args: &[&str]) -> Args {
        let owned: Vec<String> = args.iter().map(ToString::to_string).collect();
        Args::parse(&owned).expect("parses")
    }

    fn run(args: &Args, input: &str) -> String {
        let mut buffer = Vec::new();
        assert_eq!(
            args.run(&mut input.as_bytes(), &mut buffer).expect("runs"),
            0
        );
        String::from_utf8(buffer).expect("utf8")
    }

    #[test]
    fn converts_words() {
        assert_eq!(run(&parse(&["sos"]), ""), "Sierra\tOscar\tSierra\n");
        assert_eq!(
            run(&parse(&["Hi", "42!"]), ""),
            "Hotel\tIndia\t \tFour\tTwo\t!\n"
        );
    }

    #[test]
    fn reads_stdin_on_dash() {
        assert_eq!(
            run(&parse(&["-"]), "ab\ncd\n"),
            "Alfa\tBravo\nCharlie\tDelta\n"
        );
    }

    #[test]
    fn rejects_bad_arguments() {
        assert!(Args::parse(&[]).is_err());
        let unknown = vec!["-x".to_string()];
        assert!(Args::parse(&unknown).is_err());
    }
}
