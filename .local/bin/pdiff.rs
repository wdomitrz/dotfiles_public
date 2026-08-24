#!/usr/bin/env rscript.sh
// Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
// AGPL License

pub(crate) mod text {
    pub(crate) fn split_lines(data: &[u8]) -> Vec<String> {
        let text = String::from_utf8_lossy(data);
        let trimmed = text.trim_end_matches('\n');
        if trimmed.is_empty() {
            Vec::new()
        } else {
            trimmed.split('\n').map(str::to_string).collect()
        }
    }

    pub(crate) fn normalize_lines(lines: &[String]) -> Vec<String> {
        lines
            .iter()
            .map(|line| line.chars().filter(|c| !c.is_whitespace()).collect())
            .collect()
    }

    pub(crate) fn move_key(lines: &[String], ignore_whitespace: bool) -> String {
        let normalized: Vec<String> = if ignore_whitespace {
            normalize_lines(lines)
        } else {
            lines.to_vec()
        };
        normalized.join("\n")
    }

    pub(crate) fn is_binary(data: &[u8]) -> bool {
        data.contains(&0)
    }
}

pub(crate) mod diff {

    const MIN_MOVE_LINES: usize = 3;

    const HUNK_SEPARATOR: &str = " ============================================================";

    const MAX_FUZZY_MOVE_COMPARISONS: usize = 40_000;

    use crate::text;
    use std::collections::HashMap;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum Kind {
        Same,
        Prev,
        Next,
        Replace,
        MoveFrom,
        MoveTo,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(crate) struct Range {
        pub(crate) kind: Kind,
        pub(crate) prev: Vec<String>,
        pub(crate) next: Vec<String>,
    }

    impl Range {
        pub(crate) fn same(lines: Vec<String>) -> Self {
            Self {
                kind: Kind::Same,
                prev: lines,
                next: Vec::new(),
            }
        }

        pub(crate) fn removed(lines: Vec<String>) -> Self {
            Self {
                kind: Kind::Prev,
                prev: lines,
                next: Vec::new(),
            }
        }

        pub(crate) fn added(lines: Vec<String>) -> Self {
            Self {
                kind: Kind::Next,
                prev: Vec::new(),
                next: lines,
            }
        }

        pub(crate) fn changed(prev: Vec<String>, next: Vec<String>) -> Self {
            Self {
                kind: Kind::Replace,
                prev,
                next,
            }
        }

        fn prev_size(&self) -> usize {
            match self.kind {
                Kind::Same | Kind::Prev | Kind::Replace | Kind::MoveFrom => self.prev.len(),
                Kind::Next | Kind::MoveTo => 0,
            }
        }

        fn next_size(&self) -> usize {
            match self.kind {
                Kind::Same => self.prev.len(),
                Kind::Next | Kind::Replace | Kind::MoveTo => self.next.len(),
                Kind::Prev | Kind::MoveFrom => 0,
            }
        }

        fn advance(&self, prev_line: usize, next_line: usize) -> (usize, usize) {
            (prev_line + self.prev_size(), next_line + self.next_size())
        }

        fn is_move_source_candidate(&self) -> bool {
            self.kind == Kind::Prev && self.prev.len() >= MIN_MOVE_LINES
        }

        fn is_move_target_candidate(&self) -> bool {
            self.kind == Kind::Next && self.next.len() >= MIN_MOVE_LINES
        }
    }

    fn append_same(ranges: &mut Vec<Range>, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        match ranges.last_mut() {
            Some(last) if last.kind == Kind::Same => last.prev.extend_from_slice(lines),
            _ => ranges.push(Range::same(lines.to_vec())),
        }
    }

    fn unique_lines(lines: &[String], lo: usize, hi: usize) -> HashMap<&str, usize> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        let mut index: HashMap<&str, usize> = HashMap::new();
        for (offset, line) in lines[lo..hi].iter().enumerate() {
            *counts.entry(line.as_str()).or_default() += 1;
            index.insert(line.as_str(), lo + offset);
        }
        counts.retain(|_, count| *count == 1);
        index
            .into_iter()
            .filter(|(line, _)| counts.contains_key(*line))
            .collect()
    }

    fn longest_increasing_subsequence(values: &[usize]) -> Vec<usize> {
        if values.is_empty() {
            return Vec::new();
        }
        let mut tails: Vec<usize> = Vec::new();
        let mut back = vec![usize::MAX; values.len()];
        for (i, value) in values.iter().enumerate() {
            let mut lo = 0;
            let mut hi = tails.len();
            while lo < hi {
                let mid = usize::midpoint(lo, hi);
                if values[tails[mid]] < *value {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            if lo > 0 {
                back[i] = tails[lo - 1];
            }
            if lo == tails.len() {
                tails.push(i);
            } else {
                tails[lo] = i;
            }
        }
        let mut out = vec![0; tails.len()];
        let mut current = *tails.last().expect("non-empty checked above");
        for slot in (0..tails.len()).rev() {
            out[slot] = current;
            current = back[current];
        }
        out
    }

    pub(crate) struct LineDiff<'a> {
        pub(crate) prev: &'a [String],
        pub(crate) next: &'a [String],
    }

    impl LineDiff<'_> {
        pub(crate) fn ranges(&self) -> Vec<Range> {
            self.patience(0, self.prev.len(), 0, self.next.len())
        }

        fn patience(&self, p0: usize, p1: usize, n0: usize, n1: usize) -> Vec<Range> {
            let mut prefix_len = 0;
            while p0 + prefix_len < p1
                && n0 + prefix_len < n1
                && self.prev[p0 + prefix_len] == self.next[n0 + prefix_len]
            {
                prefix_len += 1;
            }
            let mut suffix_len = 0;
            while p1 > p0 + prefix_len + suffix_len
                && n1 > n0 + prefix_len + suffix_len
                && self.prev[p1 - 1 - suffix_len] == self.next[n1 - 1 - suffix_len]
            {
                suffix_len += 1;
            }

            let mut ranges = Vec::new();
            append_same(&mut ranges, &self.prev[p0..p0 + prefix_len]);
            ranges.extend(self.middle(
                p0 + prefix_len,
                p1 - suffix_len,
                n0 + prefix_len,
                n1 - suffix_len,
            ));
            append_same(&mut ranges, &self.prev[p1 - suffix_len..p1]);
            ranges
        }

        fn middle(&self, p0: usize, p1: usize, n0: usize, n1: usize) -> Vec<Range> {
            if p0 == p1 && n0 == n1 {
                return Vec::new();
            }
            if p0 == p1 {
                return vec![Range::added(self.next[n0..n1].to_vec())];
            }
            if n0 == n1 {
                return vec![Range::removed(self.prev[p0..p1].to_vec())];
            }

            let unique_prev = unique_lines(self.prev, p0, p1);
            let unique_next = unique_lines(self.next, n0, n1);
            let mut matches: Vec<(usize, usize)> = unique_prev
                .iter()
                .filter_map(|(line, pi)| unique_next.get(line).map(|ni| (*pi, *ni)))
                .collect();
            matches.sort_unstable();
            if matches.is_empty() {
                return vec![Range::changed(
                    self.prev[p0..p1].to_vec(),
                    self.next[n0..n1].to_vec(),
                )];
            }

            let values: Vec<usize> = matches.iter().copied().map(|(_, ni)| ni).collect();
            let mut ranges = Vec::new();
            let (mut cur_p, mut cur_n) = (p0, n0);
            for anchor in longest_increasing_subsequence(&values) {
                let (pi, ni) = matches[anchor];
                ranges.extend(self.patience(cur_p, pi, cur_n, ni));
                append_same(&mut ranges, std::slice::from_ref(&self.prev[pi]));
                (cur_p, cur_n) = (pi + 1, ni + 1);
            }
            ranges.extend(self.patience(cur_p, p1, cur_n, n1));
            ranges
        }
    }

    #[derive(Clone, Copy)]
    struct MoveCandidate {
        range_index: usize,
        start_line: usize,
    }

    pub(crate) fn detect_moves(ranges: &mut [Range], ignore_whitespace: bool) {
        let (prev_buckets, next_buckets) = collect_candidates(ranges, ignore_whitespace);
        for (key, prevs) in &prev_buckets {
            if let Some(nexts) = next_buckets.get(key) {
                pair_exact_moves(ranges, prevs, nexts);
            }
        }
        pair_fuzzy_moves(ranges, ignore_whitespace);
    }

    fn collect_candidates(
        ranges: &[Range],
        ignore_whitespace: bool,
    ) -> (
        HashMap<String, Vec<MoveCandidate>>,
        HashMap<String, Vec<MoveCandidate>>,
    ) {
        let mut prev_buckets: HashMap<String, Vec<MoveCandidate>> = HashMap::new();
        let mut next_buckets: HashMap<String, Vec<MoveCandidate>> = HashMap::new();
        let (mut prev_line, mut next_line) = (1_usize, 1_usize);
        for (i, range) in ranges.iter().enumerate() {
            match range.kind {
                Kind::Prev if range.is_move_source_candidate() => {
                    let key = text::move_key(&range.prev, ignore_whitespace);
                    prev_buckets.entry(key).or_default().push(MoveCandidate {
                        range_index: i,
                        start_line: prev_line,
                    });
                }
                Kind::Next if range.is_move_target_candidate() => {
                    let key = text::move_key(&range.next, ignore_whitespace);
                    next_buckets.entry(key).or_default().push(MoveCandidate {
                        range_index: i,
                        start_line: next_line,
                    });
                }
                _ => {}
            }
            (prev_line, next_line) = range.advance(prev_line, next_line);
        }
        (prev_buckets, next_buckets)
    }

    fn pair_exact_moves(ranges: &mut [Range], prevs: &[MoveCandidate], nexts: &[MoveCandidate]) {
        let mut used = vec![false; nexts.len()];
        let mut ordered = prevs.to_vec();
        ordered.sort_by_key(|candidate| candidate.start_line);
        for prev in ordered {
            let best = nearest_unused(nexts, &used, prev.start_line);
            if let Some(j) = best {
                used[j] = true;
                convert_to_move(ranges, prev.range_index, nexts[j].range_index);
            }
        }
    }

    fn nearest_unused(
        candidates: &[MoveCandidate],
        used: &[bool],
        start_line: usize,
    ) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_distance = usize::MAX;
        for (j, candidate) in candidates.iter().enumerate() {
            if used[j] {
                continue;
            }
            let distance = start_line.abs_diff(candidate.start_line);
            if distance < best_distance {
                (best, best_distance) = (Some(j), distance);
            }
        }
        best
    }

    fn pair_fuzzy_moves(ranges: &mut [Range], ignore_whitespace: bool) {
        let (prevs, nexts) = unmatched_candidates(ranges);
        if prevs.len() * nexts.len() > MAX_FUZZY_MOVE_COMPARISONS {
            return;
        }
        for prev in prevs {
            if ranges[prev.range_index].kind != Kind::Prev {
                continue;
            }
            let mut best: Option<(usize, usize, usize)> = None; // (j, same, total)
            for (j, next) in nexts.iter().enumerate() {
                if ranges[next.range_index].kind != Kind::Next {
                    continue;
                }
                let (same, total) = similarity_counts(
                    &ranges[prev.range_index].prev,
                    &ranges[next.range_index].next,
                    ignore_whitespace,
                );
                // `same / total >= 1/2`, compared without floating point.
                if same * 2 < total {
                    continue;
                }
                let better = match best {
                    None => true,
                    Some((_, bs, bt)) => same * bt > bs * total,
                };
                if better {
                    best = Some((j, same, total));
                }
            }
            if let Some((j, _, _)) = best {
                convert_to_move(ranges, prev.range_index, nexts[j].range_index);
            }
        }
    }

    fn unmatched_candidates(ranges: &[Range]) -> (Vec<MoveCandidate>, Vec<MoveCandidate>) {
        let mut prevs = Vec::new();
        let mut nexts = Vec::new();
        let (mut prev_line, mut next_line) = (1_usize, 1_usize);
        for (i, range) in ranges.iter().enumerate() {
            match range.kind {
                Kind::Prev if range.is_move_source_candidate() => prevs.push(MoveCandidate {
                    range_index: i,
                    start_line: prev_line,
                }),
                Kind::Next if range.is_move_target_candidate() => nexts.push(MoveCandidate {
                    range_index: i,
                    start_line: next_line,
                }),
                _ => {}
            }
            (prev_line, next_line) = range.advance(prev_line, next_line);
        }
        (prevs, nexts)
    }

    fn similarity_counts(
        prev: &[String],
        next: &[String],
        ignore_whitespace: bool,
    ) -> (usize, usize) {
        let normalize = |lines: &[String]| -> Vec<String> {
            if ignore_whitespace {
                text::normalize_lines(lines)
            } else {
                lines.to_vec()
            }
        };
        let (prev_norm, next_norm) = (normalize(prev), normalize(next));
        let total = prev_norm.len().max(next_norm.len());
        if total == 0 {
            return (1, 1); // unreachable for real candidates; stay neutral
        }
        let same = LineDiff {
            prev: &prev_norm,
            next: &next_norm,
        }
        .ranges()
        .iter()
        .filter(|range| range.kind == Kind::Same)
        .map(Range::prev_size)
        .sum();
        (same, total)
    }

    fn convert_to_move(ranges: &mut [Range], source: usize, target: usize) {
        let prev = ranges[source].prev.clone();
        let next = ranges[target].next.clone();
        ranges[source] = Range {
            kind: Kind::MoveFrom,
            prev: prev.clone(),
            next: next.clone(),
        };
        ranges[target] = Range {
            kind: Kind::MoveTo,
            prev,
            next,
        };
    }

    pub(crate) struct Hunk {
        prev_start: usize,
        prev_size: usize,
        next_start: usize,
        next_size: usize,
        pub(crate) ranges: Vec<Range>,
    }

    impl Hunk {
        pub(crate) fn header(&self) -> String {
            format!(
                "@@ -{},{} +{},{} @@{}",
                self.prev_start, self.prev_size, self.next_start, self.next_size, HUNK_SEPARATOR
            )
        }

        fn from_ranges(ranges: Vec<Range>, start: (usize, usize)) -> Self {
            Self {
                prev_start: start.0,
                prev_size: ranges.iter().map(Range::prev_size).sum(),
                next_start: start.1,
                next_size: ranges.iter().map(Range::next_size).sum(),
                ranges,
            }
        }

        pub(crate) fn from_flat_ranges(flat_ranges: &[Range], context: usize) -> Vec<Self> {
            struct Open {
                prev_start: usize,
                next_start: usize,
            }
            let mut hunks = Vec::new();
            let mut open: Option<Open> = None;
            let mut current: Vec<Range> = Vec::new();
            let mut prefix: Vec<String> = Vec::new();
            let mut same_after: Vec<String> = Vec::new();
            let (mut prev_line, mut next_line) = (1_usize, 1_usize);

            for range in flat_ranges {
                if range.kind == Kind::Same {
                    if open.is_none() {
                        prefix.extend_from_slice(&range.prev);
                        if prefix.len() > context {
                            prefix = keep_last(&prefix, context);
                        }
                    } else {
                        same_after.extend_from_slice(&range.prev);
                        if same_after.len() > 2 * context {
                            let snapshot = std::mem::take(&mut same_after);
                            if let Some(start) = open.take() {
                                close_hunk(
                                    &mut current,
                                    &snapshot,
                                    context,
                                    (start.prev_start, start.next_start),
                                    &mut hunks,
                                );
                            }
                            prefix = keep_last(&snapshot, context);
                        }
                    }
                } else {
                    match open {
                        None => {
                            let start = Open {
                                prev_start: prev_line - prefix.len(),
                                next_start: next_line - prefix.len(),
                            };
                            if !prefix.is_empty() {
                                current.push(Range::same(std::mem::take(&mut prefix)));
                            }
                            open = Some(start);
                        }
                        Some(_) => {
                            if !same_after.is_empty() {
                                current.push(Range::same(std::mem::take(&mut same_after)));
                            }
                        }
                    }
                    current.push(range.clone());
                }
                (prev_line, next_line) = range.advance(prev_line, next_line);
            }

            if let Some(start) = open.take() {
                close_hunk(
                    &mut current,
                    &same_after,
                    context,
                    (start.prev_start, start.next_start),
                    &mut hunks,
                );
            }
            hunks
        }
    }

    fn close_hunk(
        current: &mut Vec<Range>,
        trailing: &[String],
        context: usize,
        start: (usize, usize),
        hunks: &mut Vec<Hunk>,
    ) {
        let take = trailing.len().min(context);
        if take > 0 {
            current.push(Range::same(trailing[..take].to_vec()));
        }
        hunks.push(Hunk::from_ranges(std::mem::take(current), start));
    }

    fn keep_last(lines: &[String], context: usize) -> Vec<String> {
        match context {
            0 => Vec::new(),
            _ => lines[lines.len().saturating_sub(context)..].to_vec(),
        }
    }
}

pub(crate) mod refine {

    /// Separates lines during token refinement; cannot collide with real tokens
    /// because lines are split on newlines up front.
    const LINE_SENTINEL: &str = "\n";
    use crate::diff::{Kind, LineDiff, Range};

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(crate) struct Segment {
        pub(crate) kind: Kind,
        pub(crate) text: String,
    }

    pub(crate) type RefinedLine = Vec<Segment>;

    pub(crate) struct RefinedReplace {
        pub(crate) prev: Vec<RefinedLine>,
        pub(crate) next: Vec<RefinedLine>,
    }

    impl RefinedReplace {
        pub(crate) fn from_lines(prev_lines: &[String], next_lines: &[String]) -> Self {
            let (prev_tokens, next_tokens) = flatten(prev_lines, next_lines);
            let token_ranges = LineDiff {
                prev: &prev_tokens,
                next: &next_tokens,
            }
            .ranges();
            Self {
                prev: collapse_tokens(&token_ranges, true),
                next: collapse_tokens(&token_ranges, false),
            }
        }

        pub(crate) fn is_whitespace_only(&self) -> bool {
            let clean = |lines: &[RefinedLine], marker: Kind| {
                lines
                    .iter()
                    .flatten()
                    .all(|segment| segment.kind != marker || segment.text.trim().is_empty())
            };
            clean(&self.prev, Kind::Prev) && clean(&self.next, Kind::Next)
        }

        pub(crate) fn unified_lines(&self) -> &[RefinedLine] {
            if self.next.is_empty() {
                &self.prev
            } else {
                &self.next
            }
        }
    }

    fn flatten(prev_lines: &[String], next_lines: &[String]) -> (Vec<String>, Vec<String>) {
        let flatten_one = |lines: &[String]| -> Vec<String> {
            let mut tokens = Vec::new();
            for line in lines {
                tokens.extend(tokenize(line));
                tokens.push(LINE_SENTINEL.to_string());
            }
            tokens
        };
        (flatten_one(prev_lines), flatten_one(next_lines))
    }

    fn collapse_tokens(token_ranges: &[Range], prev_side: bool) -> Vec<RefinedLine> {
        fn emit(lines: &mut Vec<RefinedLine>, current: &mut Vec<Segment>, kind: Kind, text: &str) {
            if text == LINE_SENTINEL {
                lines.push(std::mem::take(current));
            } else if current.last().is_some_and(|segment| segment.kind == kind) {
                if let Some(last) = current.last_mut() {
                    last.text.push_str(text);
                }
            } else {
                current.push(Segment {
                    kind,
                    text: text.to_string(),
                });
            }
        }

        let mut lines = Vec::new();
        let mut current = Vec::new();
        for range in token_ranges {
            match range.kind {
                Kind::Same => {
                    for token in &range.prev {
                        emit(&mut lines, &mut current, Kind::Same, token);
                    }
                }
                Kind::Prev if prev_side => {
                    for token in &range.prev {
                        emit(&mut lines, &mut current, Kind::Prev, token);
                    }
                }
                Kind::Next if !prev_side => {
                    for token in &range.next {
                        emit(&mut lines, &mut current, Kind::Next, token);
                    }
                }
                Kind::Replace => {
                    let (tokens, kind) = if prev_side {
                        (&range.prev, Kind::Prev)
                    } else {
                        (&range.next, Kind::Next)
                    };
                    for token in tokens {
                        emit(&mut lines, &mut current, kind, token);
                    }
                }
                Kind::Prev | Kind::Next | Kind::MoveFrom | Kind::MoveTo => {}
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
        lines
    }

    fn tokenize(line: &str) -> Vec<String> {
        const DELIMITERS: &str = "\"{}[]#,.;()_";
        const PUNCT: &str = "=`+-/!@$%^&*:|<>";
        const NUMERIC_EXTRA: &str = "._,eE+-";
        fn run_end(chars: &[char], start: usize, class: impl Fn(char) -> bool) -> usize {
            let mut j = start;
            while j < chars.len() && class(chars[j]) {
                j += 1;
            }
            j
        }

        let chars: Vec<char> = line.chars().collect();
        let mut tokens = Vec::new();
        let mut i = 0;
        while i < chars.len() {
            let ch = chars[i];
            if ch.is_ascii_digit() {
                let end = run_end(&chars, i, |c| {
                    c.is_ascii_digit() || NUMERIC_EXTRA.contains(c)
                });
                split_numeric_literal(&chars[i..end], &mut tokens);
                i = end;
            } else if DELIMITERS.contains(ch) {
                tokens.push(ch.to_string());
                i += 1;
            } else if PUNCT.contains(ch) {
                let end = run_end(&chars, i, |c| PUNCT.contains(c));
                tokens.push(chars[i..end].iter().collect());
                i = end;
            } else if ch == ' ' || ch == '\t' {
                let end = run_end(&chars, i, |c| c == ' ' || c == '\t');
                tokens.push(chars[i..end].iter().collect());
                i = end;
            } else {
                let end = run_end(&chars, i, |c| {
                    !(DELIMITERS.contains(c) || PUNCT.contains(c) || c == ' ' || c == '\t')
                });
                tokens.push(chars[i..end].iter().collect());
                i = end;
            }
        }
        tokens
    }

    fn split_numeric_literal(chars: &[char], tokens: &mut Vec<String>) {
        let mut i = 0;
        while i < chars.len() {
            tokens.push(chars[i].to_string());
            if matches!(chars[i], 'e' | 'E')
                && i + 1 < chars.len()
                && matches!(chars[i + 1], '+' | '-')
            {
                i += 1;
                tokens.push(chars[i].to_string());
            }
            i += 1;
        }
    }
}

pub(crate) mod render {

    /// Reset sequence appended to every colored output line (followed by the
    /// line break) so the terminal background stays clean past the end of it.
    const CLEAR_EOL_LINE: &str = "\x1b[0m \x1b[0m\x1b[K\n";
    use std::borrow::Cow;

    use crate::diff::{Hunk, Kind, Range};
    use crate::refine::{RefinedLine, RefinedReplace, Segment};

    #[derive(Clone, Copy)]
    enum LineMark {
        Same,
        Prev,
        Next,
        Unified,
        Hunk,
    }

    impl LineMark {
        const fn parts(self) -> (&'static str, &'static str, &'static str) {
            match self {
                Self::Same => (" |", "100", ""),
                Self::Prev => ("-|", "41", "31"),
                Self::Next => ("+|", "42", "32"),
                Self::Unified => ("!|", "43", ""),
                Self::Hunk => ("@|", "100", "1"),
            }
        }
    }

    #[derive(Clone, Copy)]
    enum SideMark {
        Prev,
        Next,
        MoveFrom,
        MoveTo,
    }

    impl SideMark {
        const fn parts(self) -> (&'static str, &'static str) {
            match self {
                Self::Prev => ("-|", "41"),
                Self::Next => ("+|", "42"),
                Self::MoveFrom => ("<|", "45"),
                Self::MoveTo => (">|", "46"),
            }
        }

        const fn segment_style(self, kind: Kind) -> &'static str {
            match (self, kind) {
                (Self::Prev | Self::MoveFrom, Kind::Same) => "90",
                (Self::Next, Kind::Same) => "",
                (Self::MoveTo, Kind::Same) => "33",
                (Self::Prev, _) => "31",
                (Self::Next, _) => "32",
                (Self::MoveFrom, _) => "31;1",
                (Self::MoveTo, _) => "32;1",
            }
        }
    }

    pub(crate) struct Renderer<'a> {
        pub(crate) prev_name: &'a str,
        pub(crate) next_name: &'a str,
        pub(crate) hunks: &'a [Hunk],
        pub(crate) color: bool,
    }

    impl Renderer<'_> {
        pub(crate) fn render(&self) -> String {
            if self.hunks.is_empty() {
                return String::new();
            }
            let mut out = String::new();
            push_line(&mut out, &format!("------ {}", self.prev_name));
            push_line(&mut out, &format!("++++++ {}", self.next_name));
            for hunk in self.hunks {
                self.write_line(&mut out, LineMark::Hunk, &hunk.header());
                for range in &hunk.ranges {
                    match range.kind {
                        Kind::Same => {
                            self.write_plain_lines(&mut out, LineMark::Same, &range.prev);
                        }
                        Kind::Prev => {
                            self.write_plain_lines(&mut out, LineMark::Prev, &range.prev);
                        }
                        Kind::Next => {
                            self.write_plain_lines(&mut out, LineMark::Next, &range.next);
                        }
                        Kind::Replace => self.write_replace(&mut out, range),
                        Kind::MoveFrom => {
                            let refined = RefinedReplace::from_lines(&range.prev, &range.next);
                            self.write_refined_lines(&mut out, SideMark::MoveFrom, &refined.prev);
                        }
                        Kind::MoveTo => {
                            let refined = RefinedReplace::from_lines(&range.prev, &range.next);
                            self.write_refined_lines(&mut out, SideMark::MoveTo, &refined.next);
                        }
                    }
                }
            }
            out
        }

        fn write_replace(&self, out: &mut String, range: &Range) {
            let refined = RefinedReplace::from_lines(&range.prev, &range.next);
            if refined.is_whitespace_only() {
                for line in refined.unified_lines() {
                    self.write_line(out, LineMark::Unified, &refined_plain(line));
                }
            } else {
                self.write_refined_lines(out, SideMark::Prev, &refined.prev);
                self.write_refined_lines(out, SideMark::Next, &refined.next);
            }
        }

        fn write_plain_lines(&self, out: &mut String, mark: LineMark, lines: &[String]) {
            for line in lines {
                self.write_line(out, mark, line);
            }
        }

        fn write_line(&self, out: &mut String, mark: LineMark, text: &str) {
            let (prefix, prefix_style, text_style) = mark.parts();
            out.push_str(&self.paint(prefix_style, prefix));
            let text = self.paint(text_style, text);
            if !text.is_empty() {
                out.push(' ');
                out.push_str(&text);
            }
            out.push_str(self.end_line());
        }

        fn write_refined_lines(&self, out: &mut String, mark: SideMark, lines: &[RefinedLine]) {
            for line in lines {
                let (prefix, prefix_style) = mark.parts();
                let mut rendered = self.paint(prefix_style, prefix).into_owned();
                if line.is_empty() {
                    out.push_str(&rendered);
                    out.push_str(self.end_line());
                    continue;
                }
                rendered.push(' ');
                for segment in line {
                    rendered.push_str(&self.paint(mark.segment_style(segment.kind), &segment.text));
                }
                out.push_str(&rendered);
                out.push_str(self.end_line());
            }
        }

        fn paint<'s>(&self, style: &str, text: &'s str) -> Cow<'s, str> {
            if self.color && !style.is_empty() && !text.is_empty() {
                Cow::Owned(ansi(style, text))
            } else {
                Cow::Borrowed(text)
            }
        }

        const fn end_line(&self) -> &'static str {
            if self.color {
                CLEAR_EOL_LINE
            } else {
                "\n"
            }
        }
    }

    pub(crate) fn push_line(out: &mut String, line: &str) {
        out.push_str(line);
        out.push('\n');
    }

    fn refined_plain(line: &[Segment]) -> String {
        line.iter().map(|segment| segment.text.as_str()).collect()
    }

    pub(crate) fn ansi(style: &str, text: &str) -> String {
        fn wrap(out: &mut String, style: &str, chunk: &str) {
            out.push_str("\x1b[");
            out.push_str(style);
            out.push('m');
            out.push_str(chunk);
            out.push_str("\x1b[0m");
        }

        let mut out = String::new();
        let mut start = 0;
        for (i, ch) in text.char_indices() {
            if ch == '\r' || ch == '\n' {
                if start < i {
                    wrap(&mut out, style, &text[start..i]);
                }
                out.push(ch);
                start = i + ch.len_utf8();
            }
        }
        if start < text.len() {
            wrap(&mut out, style, &text[start..]);
        }
        if out.is_empty() {
            wrap(&mut out, style, text);
        }
        out
    }

    pub(crate) fn colored_line(text: &str) -> String {
        format!("{text}{CLEAR_EOL_LINE}")
    }

    pub(crate) struct StdinRefiner<'a> {
        pub(crate) data: &'a [u8],
        pub(crate) color: bool,
    }

    impl StdinRefiner<'_> {
        pub(crate) fn render(&self) -> String {
            if self.data.iter().all(|&byte| byte == b'\n') {
                return String::new();
            }
            let text = String::from_utf8_lossy(self.data);
            let mut out = String::new();
            let (mut del_run, mut add_run) = (Vec::new(), Vec::new());
            let mut in_hunk = false;
            for line in text.trim_end_matches('\n').split('\n') {
                if line.starts_with("@@") {
                    flush_run(&mut out, &mut del_run, &mut add_run, self.color);
                    in_hunk = true;
                    self.write_meta(&mut out, line);
                } else if in_hunk && line.starts_with('-') && !line.starts_with("---") {
                    del_run.push(line[1..].to_string());
                } else if in_hunk && line.starts_with('+') && !line.starts_with("+++") {
                    add_run.push(line[1..].to_string());
                } else {
                    flush_run(&mut out, &mut del_run, &mut add_run, self.color);
                    self.write_meta(&mut out, line);
                }
            }
            flush_run(&mut out, &mut del_run, &mut add_run, self.color);
            if self.data.ends_with(b"\n") {
                out
            } else {
                out.trim_end_matches('\n').to_string()
            }
        }

        fn write_meta(&self, out: &mut String, line: &str) {
            let line = if self.color && line.starts_with("@@") {
                ansi("1", line)
            } else {
                line.to_string()
            };
            if self.color {
                out.push_str(&colored_line(&line));
            } else {
                push_line(out, &line);
            }
        }
    }

    fn flush_run(
        out: &mut String,
        del_run: &mut Vec<String>,
        add_run: &mut Vec<String>,
        color: bool,
    ) {
        match (del_run.is_empty(), add_run.is_empty()) {
            (true, true) => {}
            (true, false) => {
                for line in add_run.drain(..) {
                    write_plain(out, '+', &line, color);
                }
            }
            (false, true) => {
                for line in del_run.drain(..) {
                    write_plain(out, '-', &line, color);
                }
            }
            (false, false) => {
                let refined = RefinedReplace::from_lines(del_run, add_run);
                for line in &refined.prev {
                    write_refined(out, '-', line, color);
                }
                for line in &refined.next {
                    write_refined(out, '+', line, color);
                }
                del_run.clear();
                add_run.clear();
            }
        }
    }

    fn write_plain(out: &mut String, prefix: char, text: &str, color: bool) {
        let line = format!("{prefix}{text}");
        let line = match (color, prefix) {
            (true, '-') => ansi("31", &line),
            (true, '+') => ansi("32", &line),
            _ => line,
        };
        if color {
            out.push_str(&colored_line(&line));
        } else {
            push_line(out, &line);
        }
    }

    fn write_refined(out: &mut String, prefix: char, line: &[Segment], color: bool) {
        if !color {
            push_line(out, &format!("{prefix}{}", refined_plain(line)));
            return;
        }
        let style = if prefix == '-' { "31" } else { "32" };
        let mut rendered = ansi(style, &prefix.to_string());
        for segment in line {
            let segment_style = match (prefix, segment.kind) {
                ('-', Kind::Same) => "90",
                ('+', Kind::Same) => "",
                _ => style,
            };
            if segment_style.is_empty() {
                rendered.push_str(&segment.text);
            } else {
                rendered.push_str(&ansi(segment_style, &segment.text));
            }
        }
        out.push_str(&colored_line(&rendered));
    }
}

pub(crate) mod sources {

    /// SHA placeholder `git` passes for the empty side of an added/deleted file.
    const NULL_SHA: &str = ".";

    const GIT_DEFAULT_CONTEXT: usize = 3;
    use std::collections::HashSet;
    use std::fs;
    use std::io;
    use std::os::unix::fs::FileTypeExt;
    use std::path::Path;

    use crate::diff::{detect_moves, Hunk, Kind, LineDiff, Range};
    use crate::render::{ansi, colored_line, push_line, Renderer};
    use crate::text::{self, split_lines};

    pub(crate) struct FileDiff<'a> {
        pub(crate) prev_data: &'a [u8],
        pub(crate) next_data: &'a [u8],
        pub(crate) prev_name: &'a str,
        pub(crate) next_name: &'a str,
        pub(crate) context: usize,
        pub(crate) color: bool,
        pub(crate) ignore_whitespace: bool,
        pub(crate) find_moves: bool,
    }

    impl FileDiff<'_> {
        pub(crate) fn output(&self) -> (String, bool) {
            if self.prev_data == self.next_data {
                return (String::new(), false);
            }
            if text::is_binary(self.prev_data) || text::is_binary(self.next_data) {
                return (
                    format!(
                        "Binary files {} and {} differ\n",
                        self.prev_name, self.next_name
                    ),
                    true,
                );
            }
            let prev_lines = split_lines(self.prev_data);
            let next_lines = split_lines(self.next_data);
            let mut ranges = compute_ranges(&prev_lines, &next_lines, self.ignore_whitespace);
            if self.find_moves {
                detect_moves(&mut ranges, self.ignore_whitespace);
            }
            let hunks = Hunk::from_flat_ranges(&ranges, self.context);
            if hunks.is_empty() {
                return (String::new(), false);
            }
            let rendered = Renderer {
                prev_name: self.prev_name,
                next_name: self.next_name,
                hunks: &hunks,
                color: self.color,
            }
            .render();
            (rendered, true)
        }
    }

    fn compute_ranges(prev: &[String], next: &[String], ignore_whitespace: bool) -> Vec<Range> {
        if !ignore_whitespace {
            return LineDiff { prev, next }.ranges();
        }
        let key_diff = LineDiff {
            prev: &text::normalize_lines(prev),
            next: &text::normalize_lines(next),
        }
        .ranges();
        remap_to_original(&key_diff, prev, next)
    }

    fn remap_to_original(
        key_ranges: &[Range],
        prev_orig: &[String],
        next_orig: &[String],
    ) -> Vec<Range> {
        let (mut pi, mut ni) = (0_usize, 0_usize);
        let mut out = Vec::new();
        for range in key_ranges {
            // Same-ranges carry their content on the previous side only;
            // both cursors advance by that shared count.
            match range.kind {
                Kind::Same => {
                    let take = range.prev.len();
                    out.push(Range::same(next_orig[ni..ni + take].to_vec()));
                    pi += take;
                    ni += take;
                }
                Kind::Prev => {
                    let take = range.prev.len();
                    out.push(Range::removed(prev_orig[pi..pi + take].to_vec()));
                    pi += take;
                }
                Kind::Next => {
                    let take = range.next.len();
                    out.push(Range::added(next_orig[ni..ni + take].to_vec()));
                    ni += take;
                }
                Kind::Replace => {
                    let (prev_take, next_take) = (range.prev.len(), range.next.len());
                    out.push(Range::changed(
                        prev_orig[pi..pi + prev_take].to_vec(),
                        next_orig[ni..ni + next_take].to_vec(),
                    ));
                    pi += prev_take;
                    ni += next_take;
                }
                Kind::MoveFrom | Kind::MoveTo => {}
            }
        }
        out
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum PathKind {
        File,
        Dir,
        Other,
        Missing,
    }

    pub(crate) fn path_kind(path: &Path) -> PathKind {
        let Ok(metadata) = fs::metadata(path) else {
            return PathKind::Missing;
        };
        let file_type = metadata.file_type();
        if file_type.is_file() || file_type.is_fifo() {
            PathKind::File
        } else if file_type.is_dir() {
            PathKind::Dir
        } else {
            PathKind::Other
        }
    }

    #[derive(Clone, Copy)]
    pub(crate) struct DiffOptions {
        pub(crate) context: usize,
        pub(crate) color: bool,
        pub(crate) ignore_whitespace: bool,
        pub(crate) find_moves: bool,
    }

    pub(crate) struct PathDiff<'a> {
        pub(crate) prev_path: &'a Path,
        pub(crate) next_path: &'a Path,
        pub(crate) options: DiffOptions,
    }

    impl PathDiff<'_> {
        pub(crate) fn output(&self) -> io::Result<(String, bool)> {
            match (path_kind(self.prev_path), path_kind(self.next_path)) {
                (PathKind::File, PathKind::File) => {
                    let (prev_data, next_data) =
                        (fs::read(self.prev_path)?, fs::read(self.next_path)?);
                    Ok(file_output(
                        self.prev_path,
                        self.next_path,
                        &prev_data,
                        &next_data,
                        self.options,
                    ))
                }
                (PathKind::Dir, PathKind::Dir) => DirectoryDiff {
                    prev_dir: self.prev_path,
                    next_dir: self.next_path,
                    options: self.options,
                }
                .output(),
                (PathKind::Missing, PathKind::File) => {
                    let next_data = fs::read(self.next_path)?;
                    Ok(file_output(
                        Path::new("/dev/null"),
                        self.next_path,
                        &[],
                        &next_data,
                        self.options,
                    ))
                }
                (PathKind::File, PathKind::Missing) => {
                    let prev_data = fs::read(self.prev_path)?;
                    Ok(file_output(
                        self.prev_path,
                        Path::new("/dev/null"),
                        &prev_data,
                        &[],
                        self.options,
                    ))
                }
                _ => Ok((
                    format!(
                        "Files {} and {} are not the same type\n",
                        self.prev_path.display(),
                        self.next_path.display()
                    ),
                    true,
                )),
            }
        }
    }

    fn file_output(
        prev_path: &Path,
        next_path: &Path,
        prev_data: &[u8],
        next_data: &[u8],
        options: DiffOptions,
    ) -> (String, bool) {
        let prev_name = prev_path.display().to_string();
        let next_name = next_path.display().to_string();
        FileDiff {
            prev_data,
            next_data,
            prev_name: &prev_name,
            next_name: &next_name,
            context: options.context,
            color: options.color,
            ignore_whitespace: options.ignore_whitespace,
            find_moves: options.find_moves,
        }
        .output()
    }

    pub(crate) struct DirectoryDiff<'a> {
        pub(crate) prev_dir: &'a Path,
        pub(crate) next_dir: &'a Path,
        pub(crate) options: DiffOptions,
    }

    impl DirectoryDiff<'_> {
        pub(crate) fn output(&self) -> io::Result<(String, bool)> {
            let list = |dir: &Path| -> io::Result<HashSet<String>> {
                Ok(fs::read_dir(dir)?
                    .flatten()
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect())
            };
            let (prev_names, next_names) = (list(self.prev_dir)?, list(self.next_dir)?);

            let mut out = String::new();
            let mut changed = false;
            let mut only_prev: Vec<_> = prev_names.difference(&next_names).cloned().collect();
            only_prev.sort();
            for name in only_prev {
                changed = true;
                let line = format!("Only in {}: {name}\n", self.prev_dir.display());
                out.push_str(&line);
                let path = self.prev_dir.join(&name);
                if is_regular_file(&path) {
                    let data = fs::read(&path)?;
                    let (rendered, _) =
                        file_output(&path, Path::new("/dev/null"), &data, &[], self.options);
                    out.push_str(&rendered);
                }
            }
            let mut only_next: Vec<_> = next_names.difference(&prev_names).cloned().collect();
            only_next.sort();
            for name in only_next {
                changed = true;
                let line = format!("Only in {}: {name}\n", self.next_dir.display());
                out.push_str(&line);
                let path = self.next_dir.join(&name);
                if is_regular_file(&path) {
                    let data = fs::read(&path)?;
                    let (rendered, _) =
                        file_output(Path::new("/dev/null"), &path, &[], &data, self.options);
                    out.push_str(&rendered);
                }
            }
            let mut common: Vec<_> = prev_names.intersection(&next_names).cloned().collect();
            common.sort();
            for name in common {
                let (rendered, diff_changed) = PathDiff {
                    prev_path: &self.prev_dir.join(&name),
                    next_path: &self.next_dir.join(&name),
                    options: self.options,
                }
                .output()?;
                if diff_changed {
                    changed = true;
                    out.push_str(&rendered);
                }
            }
            Ok((out, changed))
        }
    }

    fn is_regular_file(path: &Path) -> bool {
        fs::metadata(path).is_ok_and(|metadata| metadata.file_type().is_file())
    }

    pub(crate) struct GitExternalDiff<'a> {
        pub(crate) path: &'a str,
        pub(crate) old_file: &'a Path,
        pub(crate) old_hex: &'a str,
        pub(crate) old_mode: &'a str,
        pub(crate) new_file: &'a Path,
        pub(crate) new_hex: &'a str,
        pub(crate) new_mode: &'a str,
        pub(crate) new_path: Option<&'a str>,
        pub(crate) info: Option<&'a str>,
        pub(crate) options: DiffOptions,
    }

    impl GitExternalDiff<'_> {
        pub(crate) fn output(&self) -> io::Result<String> {
            let new_path = self.new_path.unwrap_or(self.path);
            let prev_name = format!("a/{}", self.path);
            let next_name = format!("b/{new_path}");
            let is_new_file = self.old_hex == NULL_SHA;
            let is_deleted_file = self.new_hex == NULL_SHA;
            let prev_data = read_side(is_new_file, self.old_file)?;
            let next_data = read_side(is_deleted_file, self.new_file)?;
            let (diff_out, diff_changed) = file_output(
                Path::new(&prev_name),
                Path::new(&next_name),
                &prev_data,
                &next_data,
                self.options,
            );

            let mut meta = Vec::new();
            if is_new_file {
                meta.push(format!("new file mode {}", self.new_mode));
            } else if is_deleted_file {
                meta.push(format!("deleted file mode {}", self.old_mode));
            } else if self.old_mode != self.new_mode {
                meta.push(format!("old mode {}", self.old_mode));
                meta.push(format!("new mode {}", self.new_mode));
            }
            if !diff_changed && meta.is_empty() && self.info.is_none() {
                return Ok(String::new());
            }

            let title = format!("pdiff.rs git {prev_name} {next_name}");
            let mut out = String::new();
            if self.options.color {
                out.push_str(&colored_line(&ansi("1", &title)));
            } else {
                push_line(&mut out, &title);
            }
            for line in meta {
                push_line(&mut out, &line);
            }
            if let Some(info) = self.info {
                push_line(&mut out, info);
            }
            if diff_changed && !is_new_file && !is_deleted_file {
                push_line(
                    &mut out,
                    &format!("index {}..{}", self.old_hex, self.new_hex),
                );
            }
            out.push_str(&diff_out);
            Ok(out)
        }
    }

    fn read_side(null_sha: bool, path: &Path) -> io::Result<Vec<u8>> {
        if null_sha {
            Ok(Vec::new())
        } else {
            fs::read(path)
        }
    }

    pub(crate) fn default_git_context() -> usize {
        if invoked_by_git() {
            git_config_context().unwrap_or(GIT_DEFAULT_CONTEXT)
        } else {
            GIT_DEFAULT_CONTEXT
        }
    }

    pub(crate) fn invoked_by_git() -> bool {
        ["GIT_DIFF_PATH_COUNTER", "GIT_EXTERNAL_DIFF"]
            .iter()
            .any(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()))
    }

    fn git_config_context() -> Option<usize> {
        let output = std::process::Command::new("git")
            .args(["config", "--get", "diff.context"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let value = String::from_utf8_lossy(&output.stdout);
        value
            .trim()
            .parse::<i64>()
            .ok()
            .and_then(|context| usize::try_from(context).ok())
    }
}

pub(crate) mod cli {

    const DEFAULT_CONTEXT: usize = 16;

    use std::io::{self, BufRead, Write};
    use std::path::PathBuf;

    use crate::render::StdinRefiner;
    use crate::sources::{
        default_git_context, invoked_by_git, DiffOptions, GitExternalDiff, PathDiff,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum ColorMode {
        Auto,
        Always,
        Never,
    }

    pub(crate) enum CliError {
        Help(String),
        Message(String),
    }

    pub(crate) enum Command {
        Diff(DiffArgs),
        Stdin(StdinArgs),
        Git(GitArgs),
    }

    pub(crate) struct DiffArgs {
        pub(crate) old_path: PathBuf,
        pub(crate) new_path: PathBuf,
        pub(crate) context: usize,
        pub(crate) find_moves: bool,
        pub(crate) color: ColorMode,
        pub(crate) whitespace: bool,
    }

    pub(crate) struct StdinArgs {
        pub(crate) color: ColorMode,
    }

    pub(crate) struct GitArgs {
        pub(crate) path: String,
        pub(crate) old_file: PathBuf,
        pub(crate) old_hex: String,
        pub(crate) old_mode: String,
        pub(crate) new_file: PathBuf,
        pub(crate) new_hex: String,
        pub(crate) new_mode: String,
        pub(crate) new_path: Option<String>,
        pub(crate) info: Option<String>,
        pub(crate) context: Option<usize>,
        pub(crate) find_moves: bool,
        pub(crate) color: ColorMode,
        pub(crate) whitespace: bool,
    }

    pub(crate) fn usage() -> &'static str {
        "usage: ./pdiff.rs <command> [options] <args...>

Pretty diff tool with patience diff, word-level refinement, ANSI color,
stdin unified-diff refinement, git external-diff mode, and simple move detection.

commands:
  diff OLD NEW
        Pretty-diff OLD and NEW; both may be files or directories.
  stdin
        Refine a unified diff read from stdin, e.g.: git diff | ./pdiff.rs stdin
  git PATH OLD_FILE OLD_HEX OLD_MODE NEW_FILE NEW_HEX NEW_MODE [NEW_PATH [INFO]]
        External diff driver: git config diff.external '<path>/pdiff.rs git'

options:
  -U, --unified, --context N
        Number of context lines (diff default: 16; git default: git's
        diff.context setting, else 3).
      --find-moves, --no-find-moves
        Detect moved blocks (default: find).
      --color always|auto|never
        When to color output (default: auto).
      --whitespace, --no-whitespace
        Show whitespace-only changes (default: hidden).
  -h, --help
        Show this help.

Exit status: `diff` reports changes with 1; other subcommands exit 0.
"
    }

    struct RawOptions {
        context: Option<usize>,
        find_moves: Option<bool>,
        color: Option<ColorMode>,
        whitespace: Option<bool>,
        positional: Vec<String>,
    }

    pub(crate) fn parse_args(argv: &[String]) -> Result<Command, CliError> {
        let Some((name, rest)) = argv.split_first() else {
            return Err(CliError::Message(format!("missing command\n\n{}", usage())));
        };
        if name == "-h" || name == "--help" {
            return Err(CliError::Help(usage().to_string()));
        }
        let options = parse_options(rest)?;
        match name.as_str() {
            "diff" => build_diff(&options),
            "stdin" => build_stdin(&options),
            "git" => build_git(&options),
            other => Err(CliError::Message(format!("unknown command: {other}"))),
        }
    }

    fn parse_options(args: &[String]) -> Result<RawOptions, CliError> {
        let mut options = RawOptions {
            context: None,
            find_moves: None,
            color: None,
            whitespace: None,
            positional: Vec::new(),
        };
        let mut flags_done = false;
        let mut i = 0;
        while i < args.len() {
            let arg = args[i].as_str();
            if !flags_done && arg == "--" {
                flags_done = true;
            } else if !flags_done && arg.len() > 1 && arg.starts_with('-') {
                i += flag_extra_tokens(args, i, &mut options)?;
            } else {
                options.positional.push(arg.to_string());
            }
            i += 1;
        }
        Ok(options)
    }

    fn flag_extra_tokens(
        args: &[String],
        i: usize,
        options: &mut RawOptions,
    ) -> Result<usize, CliError> {
        let arg = args[i].as_str();
        let value_arg = || -> Result<String, CliError> {
            args.get(i + 1)
                .cloned()
                .ok_or_else(|| CliError::Message(format!("argument {arg}: expected one argument")))
        };

        if let Some(long) = arg.strip_prefix("--") {
            let name = long.split_once('=').map_or(long, |(name, _)| name);
            match name {
                "help" => Err(CliError::Help(usage().to_string())),
                "color" => {
                    let value = inline_value(long)
                        .map(str::to_string)
                        .map_or_else(value_arg, Ok)?;
                    options.color = Some(parse_color(arg, &value)?);
                    Ok(usize::from(inline_value(long).is_none()))
                }
                "unified" | "context" => {
                    let value = inline_value(long)
                        .map(str::to_string)
                        .map_or_else(value_arg, Ok)?;
                    options.context = Some(parse_context(arg, &value)?);
                    Ok(usize::from(inline_value(long).is_none()))
                }
                "find-moves" => {
                    reject_inline(arg, inline_value(long))?;
                    options.find_moves = Some(true);
                    Ok(0)
                }
                "no-find-moves" => {
                    reject_inline(arg, inline_value(long))?;
                    options.find_moves = Some(false);
                    Ok(0)
                }
                "whitespace" => {
                    reject_inline(arg, inline_value(long))?;
                    options.whitespace = Some(true);
                    Ok(0)
                }
                "no-whitespace" => {
                    reject_inline(arg, inline_value(long))?;
                    options.whitespace = Some(false);
                    Ok(0)
                }
                _ => Err(CliError::Message(format!("unknown option: {arg}"))),
            }
        } else if let Some(value) = arg.strip_prefix("-U") {
            let (value, consumed) = if value.is_empty() {
                (value_arg()?, 1_usize)
            } else {
                (value.to_string(), 0)
            };
            options.context = Some(parse_context(arg, &value)?);
            Ok(consumed)
        } else if arg == "-h" {
            Err(CliError::Help(usage().to_string()))
        } else {
            Err(CliError::Message(format!("unknown option: {arg}")))
        }
    }

    fn reject_inline(arg: &str, inline: Option<&str>) -> Result<(), CliError> {
        if inline.is_some() {
            return Err(CliError::Message(format!("argument {arg}: takes no value")));
        }
        Ok(())
    }

    fn inline_value(long: &str) -> Option<&str> {
        long.split_once('=').map(|(_, value)| value)
    }

    fn parse_color(flag: &str, value: &str) -> Result<ColorMode, CliError> {
        match value {
            "auto" => Ok(ColorMode::Auto),
            "always" => Ok(ColorMode::Always),
            "never" => Ok(ColorMode::Never),
            _ => Err(CliError::Message(format!(
                "argument {flag}: invalid choice: '{value}' (choose from 'always', 'auto', 'never')"
            ))),
        }
    }

    fn parse_context(flag: &str, value: &str) -> Result<usize, CliError> {
        match value.parse::<i64>() {
            Ok(context) => usize::try_from(context).map_err(|_| {
                CliError::Message(format!("argument {flag}: value out of range: '{value}'"))
            }),
            _ => Err(CliError::Message(format!(
                "argument {flag}: must be a non-negative integer, got '{value}'"
            ))),
        }
    }

    fn build_diff(options: &RawOptions) -> Result<Command, CliError> {
        let [old_path, new_path] = options.positional.as_slice() else {
            return Err(CliError::Message(
                "diff requires exactly two paths: OLD NEW".to_string(),
            ));
        };
        Ok(Command::Diff(DiffArgs {
            old_path: PathBuf::from(old_path),
            new_path: PathBuf::from(new_path),
            context: options.context.unwrap_or(DEFAULT_CONTEXT),
            find_moves: options.find_moves.unwrap_or(true),
            color: options.color.unwrap_or(ColorMode::Auto),
            whitespace: options.whitespace.unwrap_or(false),
        }))
    }

    fn build_stdin(options: &RawOptions) -> Result<Command, CliError> {
        if !options.positional.is_empty() {
            return Err(CliError::Message(
                "stdin takes no positional arguments".to_string(),
            ));
        }
        Ok(Command::Stdin(StdinArgs {
            color: options.color.unwrap_or(ColorMode::Auto),
        }))
    }

    fn build_git(options: &RawOptions) -> Result<Command, CliError> {
        let positional = &options.positional;
        if !(7..=9).contains(&positional.len()) {
            return Err(CliError::Message(
                "git requires PATH OLD_FILE OLD_HEX OLD_MODE NEW_FILE NEW_HEX NEW_MODE [NEW_PATH [INFO]]"
                    .to_string(),
            ));
        }
        Ok(Command::Git(GitArgs {
            path: positional[0].clone(),
            old_file: PathBuf::from(&positional[1]),
            old_hex: positional[2].clone(),
            old_mode: positional[3].clone(),
            new_file: PathBuf::from(&positional[4]),
            new_hex: positional[5].clone(),
            new_mode: positional[6].clone(),
            new_path: positional.get(7).cloned(),
            info: positional.get(8).cloned(),
            context: options.context,
            find_moves: options.find_moves.unwrap_or(true),
            color: options.color.unwrap_or(ColorMode::Auto),
            whitespace: options.whitespace.unwrap_or(false),
        }))
    }

    pub(crate) fn resolve_color(mode: ColorMode, stdout_is_tty: bool, git_mode: bool) -> bool {
        match mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => stdout_is_tty || (git_mode && invoked_by_git()),
        }
    }

    pub(crate) fn execute(
        command: &Command,
        input: &mut dyn BufRead,
        output: &mut dyn Write,
        stdout_is_tty: bool,
    ) -> io::Result<i32> {
        match command {
            Command::Diff(args) => {
                let color = resolve_color(args.color, stdout_is_tty, false);
                let (out, changed) = PathDiff {
                    prev_path: &args.old_path,
                    next_path: &args.new_path,
                    options: DiffOptions {
                        context: args.context,
                        color,
                        ignore_whitespace: !args.whitespace,
                        find_moves: args.find_moves,
                    },
                }
                .output()?;
                output.write_all(out.as_bytes())?;
                Ok(i32::from(changed))
            }
            Command::Stdin(args) => {
                let color = resolve_color(args.color, stdout_is_tty, false);
                let mut data = Vec::new();
                input.read_to_end(&mut data)?;
                let rendered = StdinRefiner { data: &data, color }.render();
                output.write_all(rendered.as_bytes())?;
                Ok(0)
            }
            Command::Git(args) => {
                let color = resolve_color(args.color, stdout_is_tty, true);
                let out = GitExternalDiff {
                    path: &args.path,
                    old_file: &args.old_file,
                    old_hex: &args.old_hex,
                    old_mode: &args.old_mode,
                    new_file: &args.new_file,
                    new_hex: &args.new_hex,
                    new_mode: &args.new_mode,
                    new_path: args.new_path.as_deref(),
                    info: args.info.as_deref(),
                    options: DiffOptions {
                        context: args.context.unwrap_or_else(default_git_context),
                        color,
                        ignore_whitespace: !args.whitespace,
                        find_moves: args.find_moves,
                    },
                }
                .output()?;
                output.write_all(out.as_bytes())?;
                Ok(0)
            }
        }
    }
}

use std::io::IsTerminal;
use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let command = match cli::parse_args(&argv) {
        Ok(command) => command,
        Err(cli::CliError::Help(text)) => {
            println!("{text}");
            return ExitCode::SUCCESS;
        }
        Err(cli::CliError::Message(message)) => {
            eprintln!("pdiff.rs: {message}");
            return ExitCode::from(2);
        }
    };
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let tty = stdout.is_terminal();
    match cli::execute(&command, &mut stdin.lock(), &mut stdout.lock(), tty) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            eprintln!("pdiff.rs: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {

    use std::fs;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::cli::{self, CliError, ColorMode, Command};
    use super::diff::{detect_moves, Hunk, Kind, LineDiff, Range};
    use super::render::{ansi, StdinRefiner};
    use super::sources::{path_kind, FileDiff, PathKind};
    use super::text::{is_binary, move_key, normalize_lines, split_lines};
    const CLEAR_EOL: &str = "\x1b[0m \x1b[0m\x1b[K";

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("pdiff-test-{}-{name}-{id}", std::process::id()));
            fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }

        fn write(&self, relative: &str, contents: &str) -> String {
            let path = self.0.join(relative);
            fs::create_dir_all(path.parent().expect("relative path has parent"))
                .expect("create parents");
            fs::write(&path, contents).expect("write file");
            path.display().to_string()
        }

        fn fifo(&self, name: &str) -> String {
            let path = self.0.join(name);
            let status = std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .expect("run mkfifo");
            assert!(status.success());
            path.display().to_string()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn run(dir: &TempDir, arguments: &[&str], input: &[u8]) -> (i32, String) {
        let argv: Vec<String> = arguments.iter().map(|arg| (*arg).to_string()).collect();
        let command = match cli::parse_args(&argv) {
            Ok(command) => command,
            Err(CliError::Message(message)) => panic!("parse failed: {message}"),
            Err(CliError::Help(_)) => panic!("unexpected help"),
        };
        let mut buffer = input.to_vec();
        let mut reader = Cursor::new(buffer.as_mut_slice());
        let mut output = Vec::new();
        let code = cli::execute(&command, &mut reader, &mut output, false).expect("execute");
        let text = String::from_utf8(output)
            .expect("utf8 output")
            .replace(&format!("{}/", dir.0.display()), "")
            .replace(CLEAR_EOL, "");
        (code, text)
    }

    fn expect_lines(actual: &str, expected: &[&str]) {
        let mut wanted = expected.join("\n");
        if !expected.is_empty() {
            wanted.push('\n');
        }
        assert_eq!(actual, wanted);
    }

    #[test]
    fn simple_change_colored() {
        let dir = TempDir::new("simple");
        let old = dir.write("simple/old.txt", "\napple\nbanana\ncherry\n");
        let new = dir.write("simple/new.txt", "\napple\nBANANA\ncherry\n");
        let (code, out) = run(&dir, &["diff", "--color", "always", &old, &new], b"");
        assert_eq!(code, 1);
        expect_lines(
            &out,
            &[
                "------ simple/old.txt",
                "++++++ simple/new.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,4 +1,4 @@ ============================================================\x1b[0m",
                "\x1b[100m |\x1b[0m",
                "\x1b[100m |\x1b[0m apple",
                "\x1b[41m-|\x1b[0m \x1b[31mbanana\x1b[0m",
                "\x1b[42m+|\x1b[0m \x1b[32mBANANA\x1b[0m",
                "\x1b[100m |\x1b[0m cherry",
            ],
        );
    }

    #[test]
    fn whitespace_changes_are_hidden_by_default() {
        let dir = TempDir::new("ws-hidden");
        let old = dir.write("whitespace/old.txt", "\nx = 1\n");
        let new = dir.write("whitespace/new.txt", "\nx  = 1\n");
        let (code, out) = run(&dir, &["diff", "--color", "always", &old, &new], b"");
        assert_eq!((code, out.as_str()), (0, ""));
    }

    #[test]
    fn whitespace_changes_render_unified_when_shown() {
        let dir = TempDir::new("ws-shown");
        let old = dir.write("whitespace/old.txt", "\nx = 1\n");
        let new = dir.write("whitespace/new.txt", "\nx  = 1\n");
        let (code, out) = run(
            &dir,
            &["diff", "--color", "always", "--whitespace", &old, &new],
            b"",
        );
        assert_eq!(code, 1);
        expect_lines(
            &out,
            &[
                "------ whitespace/old.txt",
                "++++++ whitespace/new.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,2 +1,2 @@ ============================================================\x1b[0m",
                "\x1b[100m |\x1b[0m",
                "\x1b[43m!|\x1b[0m x  = 1",
            ],
        );
    }

    #[test]
    fn moved_block_is_detected_and_refined() {
        let dir = TempDir::new("move");
        let old = dir.write(
            "move/old.txt",
            "\nalpha\nmoved-one\nmoved-two-old\nmoved-three\nbeta-1\nbeta-2\nbeta-3\nbeta-4\nbeta-5\ngamma\n",
        );
        let new = dir.write(
            "move/new.txt",
            "\nalpha\nbeta-1\nbeta-2\nbeta-3\nbeta-4\nbeta-5\nmoved-one\nmoved-two-new\nmoved-three\ngamma\n",
        );
        let (code, out) = run(&dir, &["diff", "--color", "always", &old, &new], b"");
        assert_eq!(code, 1);
        expect_lines(
            &out,
            &[
                "------ move/old.txt",
                "++++++ move/new.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,11 +1,11 @@ ============================================================\x1b[0m",
                "\x1b[100m |\x1b[0m",
                "\x1b[100m |\x1b[0m alpha",
                "\x1b[45m<|\x1b[0m \x1b[90mmoved-one\x1b[0m",
                "\x1b[45m<|\x1b[0m \x1b[90mmoved-two-\x1b[0m\x1b[31;1mold\x1b[0m",
                "\x1b[45m<|\x1b[0m \x1b[90mmoved-three\x1b[0m",
                "\x1b[100m |\x1b[0m beta-1",
                "\x1b[100m |\x1b[0m beta-2",
                "\x1b[100m |\x1b[0m beta-3",
                "\x1b[100m |\x1b[0m beta-4",
                "\x1b[100m |\x1b[0m beta-5",
                "\x1b[46m>|\x1b[0m \x1b[33mmoved-one\x1b[0m",
                "\x1b[46m>|\x1b[0m \x1b[33mmoved-two-\x1b[0m\x1b[32;1mnew\x1b[0m",
                "\x1b[46m>|\x1b[0m \x1b[33mmoved-three\x1b[0m",
                "\x1b[100m |\x1b[0m gamma",
            ],
        );
    }

    #[test]
    fn word_level_refinement_highlights_changes() {
        let dir = TempDir::new("indent");
        let old = dir.write(
            "indent/old.txt",
            "\nclass X:\n    x: int\n\n\ndef a(x: int):\n    y = x + 7\n    print(y)\n",
        );
        let new = dir.write(
            "indent/new.txt",
            "\nclass X:\n    x: int\n\n    def a(self):\n        y = self.x + 7\n        print(y)\n",
        );
        let (code, out) = run(&dir, &["diff", "--color", "always", &old, &new], b"");
        assert_eq!(code, 1);
        expect_lines(
            &out,
            &[
                "------ indent/old.txt",
                "++++++ indent/new.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,8 +1,7 @@ ============================================================\x1b[0m",
                "\x1b[100m |\x1b[0m",
                "\x1b[100m |\x1b[0m class X:",
                "\x1b[100m |\x1b[0m     x: int",
                "\x1b[100m |\x1b[0m",
                "\x1b[41m-|\x1b[0m",
                "\x1b[41m-|\x1b[0m \x1b[90mdef a(\x1b[0m\x1b[31mx: int\x1b[0m\x1b[90m):\x1b[0m",
                "\x1b[41m-|\x1b[0m \x1b[31m    \x1b[0m\x1b[90my = x + 7\x1b[0m",
                "\x1b[42m+|\x1b[0m \x1b[32m    \x1b[0mdef a(\x1b[32mself\x1b[0m):",
                "\x1b[42m+|\x1b[0m \x1b[32m        \x1b[0my = \x1b[32mself.\x1b[0mx + 7",
                "\x1b[100m |\x1b[0m         print(y)",
            ],
        );
    }

    #[test]
    fn indented_moved_block_is_detected() {
        let dir = TempDir::new("indent-move");
        let old = dir.write(
            "indent_move/old.txt",
            "\nclass X:\n    x: int\n\ndef b(asdf):\n    print(asdf)\n\ndef c():\n    b(7)\n    b(\"asdf\")\n    b(\"zxcv\")\n\ndef a(self: X) -> None:\n    y = self.x + 7\n    z = self.x + 5 * y\n    print(y, z)\n",
        );
        let new = dir.write(
            "indent_move/new.txt",
            "\nclass X:\n    x: int\n\n    def a(self) -> None:\n        y = self.x + 7\n        z = self.x + 5 * y\n        print(y, z)\n\ndef b(asdf):\n    print(asdf)\n\ndef c():\n    b(7)\n    b(\"asdf\")\n    b(\"zxcv\")\n",
        );
        let (code, out) = run(&dir, &["diff", "--color", "always", &old, &new], b"");
        assert_eq!(code, 1);
        expect_lines(
            &out,
            &[
                "------ indent_move/old.txt",
                "++++++ indent_move/new.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,16 +1,16 @@ ============================================================\x1b[0m",
                "\x1b[100m |\x1b[0m",
                "\x1b[100m |\x1b[0m class X:",
                "\x1b[100m |\x1b[0m     x: int",
                "\x1b[100m |\x1b[0m",
                "\x1b[46m>|\x1b[0m \x1b[32;1m    \x1b[0m\x1b[33mdef a(self) -> None:\x1b[0m",
                "\x1b[46m>|\x1b[0m \x1b[32;1m        \x1b[0m\x1b[33my = self.x + 7\x1b[0m",
                "\x1b[46m>|\x1b[0m \x1b[32;1m        \x1b[0m\x1b[33mz = self.x + 5 * y\x1b[0m",
                "\x1b[46m>|\x1b[0m \x1b[32;1m        \x1b[0m\x1b[33mprint(y, z)\x1b[0m",
                "\x1b[46m>|\x1b[0m",
                "\x1b[100m |\x1b[0m def b(asdf):",
                "\x1b[100m |\x1b[0m     print(asdf)",
                "\x1b[100m |\x1b[0m",
                "\x1b[100m |\x1b[0m def c():",
                "\x1b[100m |\x1b[0m     b(7)",
                "\x1b[100m |\x1b[0m     b(\"asdf\")",
                "\x1b[100m |\x1b[0m     b(\"zxcv\")",
                "\x1b[45m<|\x1b[0m",
                "\x1b[45m<|\x1b[0m \x1b[90mdef a(self\x1b[0m\x1b[31;1m: X\x1b[0m\x1b[90m) -> None:\x1b[0m",
                "\x1b[45m<|\x1b[0m \x1b[31;1m    \x1b[0m\x1b[90my = self.x + 7\x1b[0m",
                "\x1b[45m<|\x1b[0m \x1b[31;1m    \x1b[0m\x1b[90mz = self.x + 5 * y\x1b[0m",
                "\x1b[45m<|\x1b[0m \x1b[31;1m    \x1b[0m\x1b[90mprint(y, z)\x1b[0m",
            ],
        );
    }

    #[test]
    fn directory_tree_is_diffed_recursively() {
        let dir = TempDir::new("directory");
        dir.write("directory/old/removed.txt", "\nremoved\n");
        dir.write("directory/old/changed.txt", "\nold value\n");
        dir.write("directory/old/same.txt", "\nsame\n");
        dir.write("directory/old/subdir/nested.txt", "\nold nested\n");
        dir.write("directory/old/typeflip/file.txt", "\ninside old dir\n");
        dir.write("directory/new/added.txt", "\nadded\n");
        dir.write("directory/new/changed.txt", "\nnew value\n");
        dir.write("directory/new/same.txt", "\nsame\n");
        dir.write("directory/new/subdir/nested.txt", "\nnew nested\n");
        dir.write("directory/new/typeflip", "\nnew plain file\n");
        let (code, out) = run(
            &dir,
            &[
                "diff",
                "--color",
                "always",
                &format!("{}/directory/old", dir.0.display()),
                &format!("{}/directory/new", dir.0.display()),
            ],
            b"",
        );
        assert_eq!(code, 1);
        expect_lines(
            &out,
            &[
                "Only in directory/old: removed.txt",
                "------ directory/old/removed.txt",
                "++++++ /dev/null",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,2 +1,0 @@ ============================================================\x1b[0m",
                "\x1b[41m-|\x1b[0m",
                "\x1b[41m-|\x1b[0m \x1b[31mremoved\x1b[0m",
                "Only in directory/new: added.txt",
                "------ /dev/null",
                "++++++ directory/new/added.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,0 +1,2 @@ ============================================================\x1b[0m",
                "\x1b[42m+|\x1b[0m",
                "\x1b[42m+|\x1b[0m \x1b[32madded\x1b[0m",
                "------ directory/old/changed.txt",
                "++++++ directory/new/changed.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,2 +1,2 @@ ============================================================\x1b[0m",
                "\x1b[100m |\x1b[0m",
                "\x1b[41m-|\x1b[0m \x1b[31mold\x1b[0m\x1b[90m value\x1b[0m",
                "\x1b[42m+|\x1b[0m \x1b[32mnew\x1b[0m value",
                "------ directory/old/subdir/nested.txt",
                "++++++ directory/new/subdir/nested.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,2 +1,2 @@ ============================================================\x1b[0m",
                "\x1b[100m |\x1b[0m",
                "\x1b[41m-|\x1b[0m \x1b[31mold\x1b[0m\x1b[90m nested\x1b[0m",
                "\x1b[42m+|\x1b[0m \x1b[32mnew\x1b[0m nested",
                "Files directory/old/typeflip and directory/new/typeflip are not the same type",
            ],
        );
    }

    #[test]
    fn stdin_diff_is_refined_colored() {
        let dir = TempDir::new("stdin-color");
        let input = b"--- a/sample.txt\n+++ b/sample.txt\n@@ -1,1 +1,1 @@\n-banana split\n+banana split now\n";
        let (code, out) = run(&dir, &["stdin", "--color", "always"], input);
        assert_eq!(code, 0);
        expect_lines(
            &out,
            &[
                "--- a/sample.txt",
                "+++ b/sample.txt",
                "\x1b[1m@@ -1,1 +1,1 @@\x1b[0m",
                "\x1b[31m-\x1b[0m\x1b[90mbanana split\x1b[0m",
                "\x1b[32m+\x1b[0mbanana split\x1b[32m now\x1b[0m",
            ],
        );
    }

    #[test]
    fn stdin_diff_passes_through_plain() {
        let dir = TempDir::new("stdin-plain");
        let input = b"--- a/sample.txt\n+++ b/sample.txt\n@@ -1,1 +1,1 @@\n-banana split\n+banana split now\n";
        let (code, out) = run(&dir, &["stdin"], input);
        assert_eq!(code, 0);
        assert_eq!(out, String::from_utf8_lossy(input).as_ref());
    }

    #[test]
    fn stdin_without_trailing_newline_stays_open() {
        let dir = TempDir::new("stdin-nonl");
        let input = b"--- a/sample.txt\n+++ b/sample.txt\n@@ -1,1 +1,1 @@\n-banana split\n+banana split now";
        let (code, out) = run(&dir, &["stdin", "--color", "always"], input);
        assert_eq!(code, 0);
        assert!(out.ends_with("\x1b[32m now\x1b[0m"));
        assert!(!out.ends_with('\n'));
    }

    #[test]
    fn git_subcommand_renders_metadata() {
        let dir = TempDir::new("git-basic");
        let old = dir.write("git/old.txt", "\napple\nbanana\ncherry\n");
        let new = dir.write("git/new.txt", "\napple\nBANANA\ncherry\n");
        let (code, out) = run(
            &dir,
            &[
                "git", "--color", "always", "file.txt", &old, "aaa111", "100644", &new, "bbb222",
                "100644",
            ],
            b"",
        );
        assert_eq!(code, 0);
        expect_lines(
            &out,
            &[
                "\x1b[1mpdiff.rs git a/file.txt b/file.txt\x1b[0m",
                "index aaa111..bbb222",
                "------ a/file.txt",
                "++++++ b/file.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -1,4 +1,4 @@ ============================================================\x1b[0m",
                "\x1b[100m |\x1b[0m",
                "\x1b[100m |\x1b[0m apple",
                "\x1b[41m-|\x1b[0m \x1b[31mbanana\x1b[0m",
                "\x1b[42m+|\x1b[0m \x1b[32mBANANA\x1b[0m",
                "\x1b[100m |\x1b[0m cherry",
            ],
        );
    }

    #[test]
    fn git_subcommand_honors_explicit_context() {
        let dir = TempDir::new("git-u0");
        let old = dir.write("git/old.txt", "\napple\nbanana\ncherry\n");
        let new = dir.write("git/new.txt", "\napple\nBANANA\ncherry\n");
        let (code, out) = run(
            &dir,
            &[
                "git", "--color", "always", "-U0", "file.txt", &old, "aaa111", "100644", &new,
                "bbb222", "100644",
            ],
            b"",
        );
        assert_eq!(code, 0);
        expect_lines(
            &out,
            &[
                "\x1b[1mpdiff.rs git a/file.txt b/file.txt\x1b[0m",
                "index aaa111..bbb222",
                "------ a/file.txt",
                "++++++ b/file.txt",
                "\x1b[100m@|\x1b[0m \x1b[1m@@ -3,1 +3,1 @@ ============================================================\x1b[0m",
                "\x1b[41m-|\x1b[0m \x1b[31mbanana\x1b[0m",
                "\x1b[42m+|\x1b[0m \x1b[32mBANANA\x1b[0m",
            ],
        );
    }

    #[test]
    fn git_rename_with_info_prints_no_index_line() {
        let dir = TempDir::new("git-rename");
        let old = dir.write("git/rename-old.txt", "\nunchanged\n");
        let new = dir.write("git/rename-new.txt", "\nunchanged\n");
        let info = "similarity index 100%\nrename from .local/opt/findfile.nvim\nrename to .local/opt/nvim_plugins/findfile.nvim";
        let (code, out) = run(
            &dir,
            &[
                "git",
                "--color",
                "always",
                ".local/opt/findfile.nvim",
                &old,
                "aaa111",
                "100644",
                &new,
                "bbb222",
                "100644",
                ".local/opt/nvim_plugins/findfile.nvim",
                info,
            ],
            b"",
        );
        assert_eq!(code, 0);
        expect_lines(
            &out,
            &[
                "\x1b[1mpdiff.rs git a/.local/opt/findfile.nvim b/.local/opt/nvim_plugins/findfile.nvim\x1b[0m",
                info,
            ],
        );
    }

    #[test]
    fn empty_file_against_content_shows_addition_hunk() {
        let dir = TempDir::new("empty");
        let old = dir.write("empty/old.txt", "");
        let new = dir.write("empty/new.txt", "a\n");
        let (code, out) = run(&dir, &["diff", "--color", "never", &old, &new], b"");
        assert_eq!(code, 1);
        expect_lines(
            &out,
            &[
                "------ empty/old.txt",
                "++++++ empty/new.txt",
                "@| @@ -1,0 +1,1 @@ ============================================================",
                "+| a",
            ],
        );
    }

    #[test]
    fn trailing_newline_only_difference_changes_nothing() {
        let dir = TempDir::new("trailnl");
        let old = dir.write("trailnl/old.txt", "x\ny");
        let new = dir.write("trailnl/new.txt", "x\ny\n");
        let (code, out) = run(&dir, &["diff", "--color", "never", &old, &new], b"");
        assert_eq!((code, out.as_str()), (0, ""));
    }

    #[test]
    fn binary_files_are_reported_not_diffed() {
        let dir = TempDir::new("binary");
        let old = dir.write("bin/old.bin", "\0abc\n");
        let new = dir.write("bin/new.bin", "\0xyz\n");
        let (code, out) = run(&dir, &["diff", "--color", "never", &old, &new], b"");
        assert_eq!(
            (code, out.as_str()),
            (1, "Binary files bin/old.bin and bin/new.bin differ\n")
        );
    }

    #[test]
    fn fifo_counts_as_a_file() {
        let dir = TempDir::new("fifo");
        let path = dir.fifo("pipe");
        assert_eq!(path_kind(Path::new(&path)), PathKind::File);
    }

    #[test]
    fn hunks_split_when_context_runs_out() {
        let flat = vec![
            Range::same(vec!["a".into()]),
            Range::removed(vec!["x".into()]),
            Range::same(vec![
                "b".into(),
                "c".into(),
                "d".into(),
                "e".into(),
                "f".into(),
                "g".into(),
                "h".into(),
            ]),
            Range::added(vec!["y".into()]),
            Range::same(vec!["z".into()]),
        ];
        let hunks = Hunk::from_flat_ranges(&flat, 0);
        assert_eq!(hunks.len(), 2);
        assert_eq!(
            hunks[0].header(),
            "@@ -2,1 +2,0 @@ ============================================================"
        );
        assert_eq!(
            hunks[1].header(),
            "@@ -10,0 +9,1 @@ ============================================================"
        );
    }

    #[test]
    fn similar_blocks_pair_as_fuzzy_moves() {
        let mut ranges = vec![
            Range::same(vec!["h".into()]),
            Range::removed(vec!["a".into(), "b".into(), "c".into()]),
            Range::same(vec!["m".into()]),
            Range::added(vec!["a".into(), "b".into(), "X".into()]),
        ];
        detect_moves(&mut ranges, false);
        assert_eq!(ranges[1].kind, Kind::MoveFrom);
        assert_eq!(ranges[3].kind, Kind::MoveTo);
    }

    #[test]
    fn dissimilar_blocks_stay_removed_and_added() {
        let mut ranges = vec![
            Range::same(vec!["h".into()]),
            Range::removed(vec!["a".into(), "b".into(), "c".into()]),
            Range::same(vec!["m".into()]),
            Range::added(vec!["p".into(), "q".into(), "r".into()]),
        ];
        detect_moves(&mut ranges, false);
        assert_eq!(ranges[1].kind, Kind::Prev);
        assert_eq!(ranges[3].kind, Kind::Next);
    }

    #[test]
    fn patience_finds_anchors_between_changes() {
        let prev: Vec<_> = ["a", "unique-1", "b", "junk", "c"]
            .map(str::to_string)
            .to_vec();
        let next: Vec<_> = ["a", "unique-2", "b", "free", "c"]
            .map(str::to_string)
            .to_vec();
        let kinds: Vec<_> = LineDiff {
            prev: &prev,
            next: &next,
        }
        .ranges()
        .iter()
        .map(|range| range.kind)
        .collect();
        assert_eq!(
            kinds,
            [
                Kind::Same,
                Kind::Replace,
                Kind::Same,
                Kind::Replace,
                Kind::Same
            ]
        );
    }

    #[test]
    fn identical_inputs_produce_no_output() {
        let data = b"same\ntext\n";
        let diff = FileDiff {
            prev_data: data,
            next_data: data,
            prev_name: "a",
            next_name: "b",
            context: 16,
            color: false,
            ignore_whitespace: false,
            find_moves: false,
        };
        assert_eq!(diff.output(), (String::new(), false));
    }

    #[test]
    fn empty_stdin_renders_nothing() {
        let refiner = StdinRefiner {
            data: b"",
            color: true,
        };
        assert!(refiner.render().is_empty());
        let newlines = StdinRefiner {
            data: b"\n\n",
            color: false,
        };
        assert!(newlines.render().is_empty());
    }

    #[test]
    fn ansi_restarts_styles_at_newlines_only() {
        assert_eq!(
            ansi("31", "red\nline"),
            "\x1b[31mred\x1b[0m\n\x1b[31mline\x1b[0m"
        );
        assert_eq!(ansi("90", "gray\r\n"), "\x1b[90mgray\x1b[0m\r\n");
        assert!(!ansi("31", "red\n").contains("\x1b[31m\n"));
        assert_eq!(ansi("1", ""), "\x1b[1m\x1b[0m");
    }

    #[test]
    fn text_helpers_behave() {
        assert_eq!(split_lines(b"apple\nbanana\n"), ["apple", "banana"]);
        assert!(split_lines(b"").is_empty());
        assert_eq!(split_lines(b"a\n\n"), ["a"]);
        let lines = vec!["x  = 1".to_string()];
        assert_eq!(normalize_lines(&lines), ["x=1"]);
        assert_eq!(move_key(&lines, true), "x=1");
        assert_eq!(move_key(&lines, false), "x  = 1");
        assert!(is_binary(b"a\0b"));
        assert!(!is_binary(b"abc"));
    }

    #[test]
    fn flag_forms_are_accepted() {
        let argv: Vec<String> = [
            "diff",
            "--color=never",
            "-U5",
            "--no-find-moves",
            "--whitespace",
            "--",
            "-odd",
            "new",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
        let Command::Diff(parsed) = cli::parse_args(&argv).ok().expect("parses") else {
            panic!("expected diff command");
        };
        assert_eq!(parsed.context, 5);
        assert!(!parsed.find_moves);
        assert!(parsed.whitespace);
        assert_eq!(parsed.color, ColorMode::Never);
        assert_eq!(parsed.old_path, PathBuf::from("-odd"));
    }

    #[test]
    fn invalid_invocations_are_rejected() {
        let missing: Vec<String> = vec![];
        assert!(matches!(
            cli::parse_args(&missing),
            Err(CliError::Message(_))
        ));
        let unknown: Vec<String> = ["diff".to_string(), "--bogus".to_string()].to_vec();
        assert!(matches!(
            cli::parse_args(&unknown),
            Err(CliError::Message(_))
        ));
        let negative_context: Vec<String> = ["diff".to_string(), "-U-1".to_string()].to_vec();
        assert!(matches!(
            cli::parse_args(&negative_context),
            Err(CliError::Message(_))
        ));
        let bad_arity: Vec<String> = ["git".to_string(), "onlyone".to_string()].to_vec();
        assert!(matches!(
            cli::parse_args(&bad_arity),
            Err(CliError::Message(_))
        ));
    }

    #[test]
    fn help_works_with_and_without_a_subcommand() {
        for flag in ["-h", "--help"] {
            let top_level = [flag.to_string()];
            assert!(matches!(
                cli::parse_args(&top_level),
                Err(CliError::Help(_))
            ));
            let in_subcommand = ["diff".to_string(), flag.to_string()];
            assert!(matches!(
                cli::parse_args(&in_subcommand),
                Err(CliError::Help(_))
            ));
        }
    }

    #[test]
    fn color_resolution_follows_mode_and_environment() {
        assert!(cli::resolve_color(ColorMode::Always, false, false));
        assert!(!cli::resolve_color(ColorMode::Never, true, true));
        assert!(cli::resolve_color(ColorMode::Auto, true, false));
        assert!(!cli::resolve_color(ColorMode::Auto, false, false));
    }

    #[test]
    fn default_git_context_outside_git_is_three() {
        assert_eq!(super::sources::default_git_context(), 3);
    }
}
