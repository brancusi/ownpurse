//! `help-markdown` covers every command (from clap), marks network commands, and stays hidden from `--help`.

use std::process::Command;

fn run(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(args)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn reference_covers_every_command_and_marks_network_ones() {
    let md = run(&["help-markdown"]);
    for c in [
        "tui",
        "doctor",
        "accounts",
        "org",
        "contacts",
        "txns",
        "journals",
        "transfers",
        "show",
        "list",
        "gl",
        "tb",
        "pl",
        "bs",
        "parity",
        "usage",
        "config",
        "ic",
        "ui ls",
        "ui state",
        "ui screen",
        "ui send",
        "classify suggest",
        "classify rules check",
        "classify export",
        "classify queue",
        "classify set",
        "classify note",
        "classify group add",
        "classify group create",
        "classify group remove",
        "classify group status",
        "classify undo",
        "classify config add-project",
    ] {
        assert!(md.contains(&format!("### `ownpurse {c}`")), "missing {c}");
        assert!(
            md.contains(&format!("<!-- example:{} -->", c.replace(' ', "-"))),
            "no example marker for {c}"
        );
    }
    for n in ["init", "q"] {
        assert!(
            md.contains(&format!("| `ownpurse {n}` |")),
            "{n} is listed unmarked"
        );
    }
    assert!(md.contains("## Exit codes"));
    assert!(md.contains("| 4 | refused by the TUI"));
    assert!(md.contains("`--expect-rev <EXPECT_REV>`"));
    assert!(
        !md.contains("(default `false`)"),
        "flags never show a false default"
    );
    assert!(md.contains("`--from <DATE>`"));
}

#[test]
fn hidden_from_help() {
    assert!(!run(&["--help"]).contains("help-markdown"));
}
