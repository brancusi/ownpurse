//! `ownpurse help-markdown` (hidden): the CLI reference as Markdown, generated from the clap definitions so it
//! cannot drift. Every command gets an `<!-- example:NAME -->` marker that `rust/scripts/docs-inputs.sh` fills with
//! real output from the demo fixture. Commands that call Xero are marked `⇅`; unported ones are listed last.

use clap::{Arg, ArgAction, Command};

/// Commands that need the network or the token, answered with `not_available` (see `NOT_AVAILABLE`).
pub fn markdown(cmd: &Command, network: &[&str]) -> String {
    let mut s = String::new();
    s.push_str("# ownpurse CLI reference\n\n");
    s.push_str(&format!(
        "Generated from `ownpurse {}` (`ownpurse help-markdown`). Every read command works offline over the local \
         record. Commands marked ⇅ call Xero (`sync`, `login`, `orgs`, `report fetch`, `history`, `attachments`). The last table lists the commands that are \
         not yet available (coming in v0.9.x).\n\n",
        env!("CARGO_PKG_VERSION")
    ));

    s.push_str("## Global options\n\nValid before or after the command.\n\n");
    s.push_str("| Option | Meaning |\n|---|---|\n");
    for a in cmd.get_arguments().filter(|a| a.is_global_set()) {
        s.push_str(&format!("| {} | {} |\n", flag(a), help(a)));
    }
    s.push_str(
        "| `-h`, `--help` | help for any command |\n| `-V`, `--version` | print the version |\n\n",
    );

    s.push_str("## Exit codes\n\n| Code | Meaning |\n|---|---|\n");
    s.push_str("| 0 | success |\n");
    s.push_str(
        "| 1 | a runtime error (structured: `error: <code>`, `message:`, `help[]`), e.g. `config`, `parity_mismatch` \
         (`parity --strict`), `doctor` (a failed check) |\n",
    );
    s.push_str(
        "| 2 | a usage error (`error: usage`) or a command that is not yet available \
         (`error: not_available`, coming in v0.9.x), `jev_not_enabled` (`classify suggest --engine jev`), `filter` (a bad \
         classify filter), `too_many` (a classify write over 50 lines without `--allow-bulk`), `bad_caption` |\n",
    );
    s.push_str(
        "| 4 | refused by the TUI (`ui send`): `view_changed` (the view moved since `--expect-rev`) or `user_busy` \
         (the user is typing or in an overlay); or a classify write refused with `view_changed` (the store moved \
         since `--expect-store-rev`) or `user_busy` (the line is open in a picker or editor); nothing was changed |\n\n",
    );
    s.push_str(
        "Errors print to stdout in the same format as results (`--json` for JSON), so agents can parse both.\n\n",
    );

    s.push_str("## Commands\n\n");
    s.push_str("### `ownpurse tui`\n\nThe terminal UI (read-only; zero network calls while navigating). Same as `ownpurse-tui`.\n\n");
    s.push_str("```text\nownpurse tui [--config FILE] [--profile NAME] [--offline] [--theme ledger-dark|ledger-light|ledger-dark-256|ledger-light-256|ansi|no-color] [--glyphs unicode|ascii] [--no-mouse] [--no-agent]\nownpurse tui --keys [markdown|json|conflicts]\n```\n\n");
    s.push_str("<!-- example:tui -->\n\n");
    for sub in cmd.get_subcommands().filter(|c| !c.is_hide_set()) {
        section(&mut s, sub, sub.get_name());
        // a command group (`ui`): each of its commands gets its own section
        for inner in sub
            .get_subcommands()
            .filter(|c| !c.is_hide_set() && c.get_name() != "help")
        {
            let name = format!("{} {}", sub.get_name(), inner.get_name());
            section(&mut s, inner, &name);
            // one more level (`classify rules check`)
            for leaf in inner
                .get_subcommands()
                .filter(|c| !c.is_hide_set() && c.get_name() != "help")
            {
                section(&mut s, leaf, &format!("{name} {}", leaf.get_name()));
            }
        }
    }

    s.push_str("## Not yet available (coming in v0.9.x)\n\n");
    s.push_str(
        "Running one of these answers `not_available` (exit 2).\n\n\
         | Command | What it will do |\n|---|---|\n",
    );
    for n in network {
        s.push_str(&format!("| `ownpurse {n}` | {} |\n", what_it_does(n)));
    }
    s.push('\n');
    s
}

/// One command's section: about, usage, arguments, the example marker.
fn section(s: &mut String, sub: &Command, name: &str) {
    s.push_str(&format!("### `ownpurse {name}`\n\n"));
    if let Some(about) = sub.get_about() {
        s.push_str(&format!("{about}\n\n"));
    }
    let mut sub = sub.clone().bin_name(format!("ownpurse {name}"));
    let usage = sub.render_usage().to_string();
    s.push_str(&format!(
        "```text\n{}\n```\n\n",
        usage.trim_start_matches("Usage: ")
    ));
    let pos: Vec<&Arg> = sub.get_positionals().collect();
    let opts: Vec<&Arg> = sub
        .get_arguments()
        .filter(|a| !a.is_positional() && !a.is_global_set() && a.get_id() != "help")
        .collect();
    if !pos.is_empty() || !opts.is_empty() {
        s.push_str("| Argument | Meaning |\n|---|---|\n");
        for a in pos.iter().chain(opts.iter()) {
            s.push_str(&format!("| {} | {} |\n", flag(a), help(a)));
        }
        s.push('\n');
    }
    s.push_str(&format!("<!-- example:{} -->\n\n", name.replace(' ', "-")));
}

/// One line per command that is not ported yet (the reference's last table).
fn what_it_does(cmd: &str) -> &'static str {
    match cmd {
        "report" => {
            "Fetch a Xero report (trial balance, balance sheet, P&L, bank summary) into the record"
        }
        "history" => "Show Xero's history and notes for a document, cached in the record",
        "attachments" => "List a document's attachments and save the files",
        "export" => "Write JSON snapshots of the record for the ledger loader",
        "init" => "Create a config file with one profile",
        "q" => "Run read-only SQL against the local record",
        "verify" => "Verify the record's append-only integrity chain",
        "skills" => "Install the agent skill that describes this tool",
        "status" => "Show login state, connected orgs and calls left",
        _ => "",
    }
}

fn flag(a: &Arg) -> String {
    if a.is_positional() {
        let v = a
            .get_value_names()
            .and_then(|v| v.first())
            .map(|v| v.to_string())
            .unwrap_or_else(|| a.get_id().to_string().to_uppercase());
        return if a.is_required_set() {
            format!("`<{v}>`")
        } else {
            format!("`[{v}]`")
        };
    }
    let mut f = String::new();
    if let Some(sh) = a.get_short() {
        f.push_str(&format!("`-{sh}`, "));
    }
    let long = a.get_long().unwrap_or_default();
    let takes = matches!(a.get_action(), ArgAction::Set | ArgAction::Append);
    if takes {
        let v = a
            .get_value_names()
            .and_then(|v| v.first())
            .map(|v| v.to_string())
            .unwrap_or_else(|| a.get_id().to_string().to_uppercase());
        f.push_str(&format!("`--{long} <{v}>`"));
    } else {
        f.push_str(&format!("`--{long}`"));
    }
    f
}

fn help(a: &Arg) -> String {
    let mut h = a
        .get_help()
        .map(|h| h.to_string())
        .unwrap_or_default()
        .replace('|', "\\|");
    let vals: Vec<String> = a
        .get_possible_values()
        .iter()
        .map(|v| format!("`{}`", v.get_name()))
        .collect();
    if !vals.is_empty() && a.get_action().takes_values() {
        h.push_str(&format!(" (one of {})", vals.join(", ")));
    }
    let defaults: Vec<String> = a
        .get_default_values()
        .iter()
        .map(|v| v.to_string_lossy().to_string())
        .collect();
    if !defaults.is_empty() && a.get_action().takes_values() {
        h.push_str(&format!(" (default `{}`)", defaults.join(",")));
    }
    if a.is_required_set() && !a.is_positional() {
        h.push_str(" (required)");
    }
    h
}
