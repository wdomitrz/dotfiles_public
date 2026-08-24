#!/usr/bin/env rscript.sh
// Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
// AGPL License

//# dependencies:
//# i3ipc = "0.10"

//! Keeps an "up to N columns" layout on the focused workspace: newly focused
//! windows are split so that no more than `n` columns exist, and after
//! windows appear, disappear or move, the focused window is snapped to the
//! nearest multiple of `workspace_width / n`.
//!
//! Unlike the reference Python implementation, fullscreen windows are not
//! special-cased: the `i3ipc` tree type does not expose `fullscreen_mode`.

use std::process::ExitCode;

use crate::args::Args;

/// Command-line interface: flag parsing and the resulting configuration.
mod args {
    const USAGE: &str = "usage: ./i3_n_column_layout.rs [--number-of-columns N]";
    const DEFAULT_NUMBER_OF_COLUMNS: f64 = 2.0;

    pub(crate) struct Args {
        number_of_columns: f64,
    }

    impl Args {
        // Defaults live here, next to the flag handling.
        pub(crate) fn parse(argv: &[String]) -> Result<Self, String> {
            let mut number_of_columns = DEFAULT_NUMBER_OF_COLUMNS;
            let mut i = 0;
            while i < argv.len() {
                match argv[i].as_str() {
                    "--number-of-columns" => {
                        i += 1;
                        let raw = argv
                            .get(i)
                            .ok_or("argument --number-of-columns: expected one argument")?;
                        number_of_columns = raw.parse::<f64>().map_err(|_| {
                            format!("argument --number-of-columns: invalid number: {raw}")
                        })?;
                    }
                    "-h" | "--help" => return Err(USAGE.to_string()),
                    other => return Err(format!("unknown argument: {other}")),
                }
                i += 1;
            }
            if !number_of_columns.is_finite() || number_of_columns <= 0.0 {
                return Err(format!(
                    "argument --number-of-columns: expected a positive number, got {number_of_columns}"
                ));
            }
            Ok(Self { number_of_columns })
        }

        pub(crate) fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
            super::layout::listen(self.number_of_columns)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{Args, DEFAULT_NUMBER_OF_COLUMNS};

        #[test]
        fn parses_defaults() {
            let parsed = Args::parse(&[]).expect("parses");
            assert!((parsed.number_of_columns - DEFAULT_NUMBER_OF_COLUMNS).abs() < f64::EPSILON);
        }

        #[test]
        fn parses_number_of_columns() {
            let argv = ["--number-of-columns".to_string(), "3.5".to_string()];
            let parsed = Args::parse(&argv).expect("parses");
            assert!((parsed.number_of_columns - 3.5).abs() < f64::EPSILON);
        }

        #[test]
        fn rejects_bad_arguments() {
            assert!(Args::parse(&["--number-of-columns".to_string()]).is_err());
            assert!(Args::parse(&["--number-of-columns".to_string(), "x".to_string()]).is_err());
            assert!(Args::parse(&["--number-of-columns".to_string(), "-1".to_string()]).is_err());
            assert!(Args::parse(&["--bogus".to_string()]).is_err());
        }
    }
}

/// Pure geometry of the n-column layout: how wide a container should be and
/// where to place a new split, given the workspace width.
mod geometry {
    /// Width the container should have so that it is a multiple of the column
    /// unit (`workspace_width / n`), rounding down to the nearest multiple and
    /// snapping up only when within one column-unit of the next one.
    pub(crate) fn snapped_width(width: f64, workspace_width: f64, n: f64) -> f64 {
        let unit = (workspace_width / n).floor();
        if unit <= 0.0 {
            return width;
        }
        let mut size_delta = width % unit;
        if -n <= size_delta - unit {
            size_delta -= unit;
        }
        width - size_delta
    }

    /// Splitting direction for the focused container: horizontal while there
    /// is still room for another full column, vertical afterwards.
    pub(crate) fn split_direction(width: f64, workspace_width: f64, n: f64) -> &'static str {
        if width > (2.0 * workspace_width / n).floor() - n {
            "horizontal"
        } else {
            "vertical"
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{snapped_width, split_direction};

        fn assert_snapped(width: f64, workspace_width: f64, n: f64, expected: f64) {
            assert!((snapped_width(width, workspace_width, n) - expected).abs() < 1e-9);
        }

        #[test]
        fn snaps_down_to_nearest_lower_multiple() {
            assert_snapped(1930.0, 1920.0, 2.0, 1920.0);
            assert_snapped(1890.0, 1920.0, 2.0, 960.0);
        }

        #[test]
        fn snaps_up_when_close_enough_to_next_multiple() {
            assert_snapped(2878.0, 1920.0, 2.0, 2880.0);
        }

        #[test]
        fn keeps_aligned_widths() {
            assert_snapped(960.0, 1920.0, 2.0, 960.0);
        }

        #[test]
        fn splits_horizontally_while_there_is_room() {
            // threshold: 2 * 1920 / 3 - 3 = 1277
            assert_eq!(split_direction(1300.0, 1920.0, 3.0), "horizontal");
        }

        #[test]
        fn splits_vertically_once_the_room_is_used_up() {
            assert_eq!(split_direction(900.0, 1920.0, 3.0), "vertical");
        }

        #[test]
        fn thresholds_use_floor_division_like_the_reference() {
            // threshold: floor(2 * 1000 / 3) - 3 = 663, not 663.666...
            assert_eq!(split_direction(663.3, 1000.0, 3.0), "horizontal");
        }
    }
}

/// Reading the i3 layout tree and reacting to window events with commands.
mod layout {
    use i3ipc::{
        event::{inner::WindowChange, Event, WindowEventInfo},
        reply::{Node, NodeLayout, NodeType},
        I3Connection, I3EventListener, Subscription,
    };

    use super::geometry::{snapped_width, split_direction};

    /// Listens to window events forever, applying the n-column layout.
    pub(crate) fn listen(n: f64) -> Result<(), Box<dyn std::error::Error>> {
        let mut listener = I3EventListener::connect()?;
        let mut commands = I3Connection::connect()?;
        listener.subscribe(&[Subscription::Window])?;
        for event in listener.listen() {
            match event? {
                Event::WindowEvent(event) => handle_window(&mut commands, &event, n)?,
                Event::WorkspaceEvent(_)
                | Event::OutputEvent(_)
                | Event::ModeEvent(_)
                | Event::BarConfigEvent(_)
                | Event::BindingEvent(_) => {}
            }
        }
        Ok(())
    }

    /// The focused container together with the tree context needed to decide
    /// how (and whether) to react.
    struct FocusContext<'a> {
        con: &'a Node,
        parent: Option<&'a Node>,
        workspace_width: Option<i32>,
        /// Reached through a `floating_nodes` list somewhere down the tree.
        floating: bool,
    }

    impl FocusContext<'_> {
        /// Floating windows manage their own geometry and should be left alone.
        fn ignorable(&self) -> bool {
            self.floating
        }

        fn parent_layout_is_tabbed_or_stacked(&self) -> bool {
            self.parent.is_some_and(|parent| {
                matches!(parent.layout, NodeLayout::Stacked | NodeLayout::Tabbed)
            })
        }
    }

    fn locate_focused(root: &Node) -> Option<FocusContext<'_>> {
        fn walk<'a>(
            con: &'a Node,
            ancestors: &mut Vec<&'a Node>,
            floating: bool,
            found: &mut Option<FocusContext<'a>>,
        ) {
            if found.is_some() {
                return;
            }
            if con.focused {
                let workspace_width = ancestors
                    .iter()
                    .find(|ancestor| ancestor.nodetype == NodeType::Workspace)
                    .map(|workspace| workspace.rect.2);
                *found = Some(FocusContext {
                    con,
                    parent: ancestors.last().copied(),
                    workspace_width,
                    floating,
                });
                return;
            }
            ancestors.push(con);
            for (child, via_floating) in con
                .nodes
                .iter()
                .zip(std::iter::repeat(false))
                .chain(con.floating_nodes.iter().zip(std::iter::repeat(true)))
            {
                walk(child, ancestors, floating || via_floating, found);
            }
            ancestors.pop();
        }

        let mut ancestors = Vec::new();
        let mut found = None;
        walk(root, &mut ancestors, false, &mut found);
        found
    }

    fn handle_window(
        commands: &mut I3Connection,
        event: &WindowEventInfo,
        n: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tree = commands.get_tree()?;
        let Some(context) = locate_focused(&tree) else {
            return Ok(());
        };
        match event.change {
            WindowChange::New
            | WindowChange::Close
            | WindowChange::Move
            | WindowChange::FullscreenMode => resize_to_nth(commands, &context, n),
            WindowChange::Focus => up_to_n_columns(commands, &context, n),
            WindowChange::Title
            | WindowChange::Floating
            | WindowChange::Urgent
            | WindowChange::Unknown => Ok(()),
        }
    }

    fn resize_to_nth(
        commands: &mut I3Connection,
        context: &FocusContext<'_>,
        n: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if context.ignorable() || context.parent_layout_is_tabbed_or_stacked() {
            return Ok(());
        }
        // In tabbed/stacked parents the child rect is offset by decorations;
        // a mismatched y against the parent means it is not directly sized.
        if context
            .parent
            .is_some_and(|parent| parent.rect.1 != context.con.rect.1)
        {
            return Ok(());
        }
        let Some(workspace_width) = context.workspace_width else {
            return Ok(());
        };
        let target = snapped_width(f64::from(context.con.rect.2), f64::from(workspace_width), n);
        commands.run_command(&format!("resize set width {target:.0}"))?;
        Ok(())
    }

    fn up_to_n_columns(
        commands: &mut I3Connection,
        context: &FocusContext<'_>,
        n: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if context.ignorable() {
            return Ok(());
        }
        let Some(workspace_width) = context.workspace_width else {
            return Ok(());
        };
        let direction =
            split_direction(f64::from(context.con.rect.2), f64::from(workspace_width), n);
        commands.run_command(&format!("split {direction}"))?;
        Ok(())
    }
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let parsed = match Args::parse(&argv) {
        Ok(parsed) => parsed,
        Err(message) if message.starts_with("usage:") => {
            println!("{message}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("i3_n_column_layout: {message}");
            return ExitCode::from(2);
        }
    };
    match parsed.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("i3_n_column_layout: {error}");
            ExitCode::FAILURE
        }
    }
}
