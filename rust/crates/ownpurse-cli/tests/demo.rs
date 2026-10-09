//! Demo is discoverable and never falls back to a live project when there is no terminal.
use std::process::Command;

fn cli() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ownpurse"));
    c.env(
        "OWNPURSE_CONFIG",
        "/missing/live-config-must-not-be-read.json",
    );
    c
}

#[test]
fn help_explains_the_scripted_mock_data_demo() {
    let output = cli().args(["demo", "--help"]).output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("isolated mock database"), "{text}");
    assert!(text.contains("--step-seconds"), "{text}");
}

#[test]
fn nonterminal_refusal_does_not_load_the_live_config() {
    let output = cli().args(["--json", "demo"]).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["error"], "demo_terminal_required");
}

#[test]
fn invalid_playback_pace_is_rejected_before_starting() {
    for seconds in ["0", "61"] {
        let output = cli()
            .args(["demo", "--step-seconds", seconds])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
    }
}
