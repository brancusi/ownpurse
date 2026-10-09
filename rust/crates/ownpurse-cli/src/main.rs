//! `ownpurse`: the record-only Rust port of the `ownpurse` read commands.
//!
//! Reads come from the local record only (as `ownpurse --offline`); nothing here calls Xero, reads the
//! OAuth token file or writes to the record. Commands that need the network, the token or write files
//! answer with a structured `not_available` error (exit 2) saying it is coming.
//! Exit codes: 0 ok, 1 runtime error, 2 usage error. Errors print on stdout in the same format.

mod app;
mod classify;
mod cmds;
mod config_cmd;
mod demo;
mod docs_md;
mod doctor;
mod ic;
mod ledger;
mod manifest;
mod net;
mod out;
mod statements;
mod ui;
mod usage;
mod viz;
mod walkthrough;
mod write;
mod write_run;

use app::{App, CliError, Global, Res};
use clap::{CommandFactory, Parser, Subcommand};

/// Commands this binary does not implement yet (network, token, file writes, interactive).
const NOT_AVAILABLE: [&str; 5] = ["init", "q", "verify", "skills", "status"];

#[derive(Parser, Debug)]
#[command(
    name = "ownpurse",
    version = env!("OWNPURSE_FULL_VERSION"),
    about = "ownpurse read commands over the local record (no network)",
    infer_long_args = true
)]
struct Cli {
    /// profile from the config (default: default_profile)
    #[arg(long, global = true)]
    profile: Option<String>,
    /// machine-readable output
    #[arg(long, global = true)]
    json: bool,
    /// never call Xero (network commands refuse; read commands never call Xero anyway)
    #[arg(long, global = true)]
    offline: bool,
    /// show all rows and extra columns
    #[arg(long, global = true)]
    full: bool,
    /// rows to show (default 50)
    #[arg(long, global = true)]
    limit: Option<usize>,
    /// read main + this manifest (tb, gl, parity, ic, compare, classify queue; default: the current manifest)
    #[arg(long, global = true, value_name = "NAME")]
    manifest: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum UiCmd {
    /// List the running TUIs an agent can talk to (stale entries are pruned)
    Ls,
    /// The view state of a running TUI as data: tab, scope, as-of, basis, filter, the selected and visible rows (anchors), strip facts, size, `rev`
    State {
        /// which TUI: an id, pid, label or `scope:<alias>` (required when several run; `ownpurse ui ls`)
        #[arg(long)]
        session: Option<String>,
        /// every row of the screen's list, not just those on screen (read-only: the view doesn't move)
        #[arg(long = "all-rows", conflicts_with = "rows")]
        all_rows: bool,
        /// rows A..B of the list (1-based, the `#n` of `select`; read-only)
        #[arg(long, value_name = "A..B")]
        rows: Option<String>,
        /// only these fields, comma-separated dotted paths (e.g. `tab,breadcrumb,view,selected.account_code`)
        #[arg(long, value_name = "PATHS")]
        fields: Option<String>,
    },
    /// The frame the TUI shows now (text, ansi, html, svg or png), with the `rev` it belongs to
    Screen {
        /// output format
        #[arg(long, default_value = "text", value_parser = ui::FORMATS)]
        format: String,
        /// write to this file (required for png)
        #[arg(long)]
        out: Option<std::path::PathBuf>,
        /// which TUI: an id, pid, label or `scope:<alias>` (required when several run)
        #[arg(long)]
        session: Option<String>,
        /// render a saved `ui state` instead of a live TUI (no session needed; runs ownpurse-tui --state)
        #[arg(long = "from-state", value_name = "FILE", conflicts_with = "session")]
        from_state: Option<std::path::PathBuf>,
        /// with --from-state: render even if the record changed since the state was saved (marked drifted)
        #[arg(long = "allow-record-drift", requires = "from_state")]
        allow_record_drift: bool,
        /// with --from-state: render even if the config changed beyond a reorder (marked drifted)
        #[arg(long = "allow-config-drift", requires = "from_state")]
        allow_config_drift: bool,
        /// with --from-state: render even if labels changed since the state was saved (marked drifted)
        #[arg(long = "allow-classify-drift", requires = "from_state")]
        allow_classify_drift: bool,
    },
    /// Change the view (view verbs only): tab <n|name>, scope <alias|group:x|all>, asof <date>, filter <text>, clear-filter, select <#n|anchor|text>, drill, up (one level, as Esc), basis year|history, differs on|off, compare <org,org…>, reload, open (the document on screen in Xero; a link, no call); cosmetic (never refused while the user types): theme <name>, accent <family|#hex>, overlay set <token=value…> | overlay clear [token…] (last)
    Send {
        /// verbs and their arguments, e.g. `tab 2 scope design`; or with --json one message: '{"verb": "tab", "args": ["2"]}'
        #[arg(required = true)]
        verbs: Vec<String>,
        /// refuse (exit 4, view_changed) unless the view is still at this rev (from `ui state`)
        #[arg(long = "expect-rev", visible_alias = "expect-seq")]
        expect_rev: Option<u64>,
        /// refuse (exit 4, view_mismatch) unless the view matches: PATH=VALUE, PATH!=VALUE or PATH~TEXT over `ui state` fields (repeatable, AND-ed), e.g. `--expect tab=ledger --expect breadcrumb~"Trading Clearing"`
        #[arg(long = "expect", value_name = "PRED")]
        expect: Vec<String>,
        /// also return the new frame in this format
        #[arg(long, value_parser = ui::FORMATS)]
        screen: Option<String>,
        /// write the frame (--screen) to this file (required for png)
        #[arg(long)]
        out: Option<std::path::PathBuf>,
        /// machine-readable output (as --json; use it when --json carries a message)
        #[arg(long = "json-out")]
        json_out: bool,
        /// also set the footer caption, in the same update (atomic: a refused verb applies nothing)
        #[arg(long)]
        caption: Option<String>,
        /// the client name captions show as their author (default `agent`)
        #[arg(long)]
        client: Option<String>,
        /// print the whole view state (text) instead of the compact summary
        #[arg(long = "state")]
        full_state: bool,
        /// which TUI: an id, pid, label or `scope:<alias>` (required when several run)
        #[arg(long)]
        session: Option<String>,
    },
    /// Wait until the TUI's state meets a condition (answered by the TUI when it holds; no polling): idle, running=none, rev>N, store_rev>N, record>N (each also >=N); `timeout` (exit 4) with the current values otherwise
    Wait {
        /// the condition, e.g. `idle` or `rev>12`
        #[arg(long)]
        until: String,
        /// seconds to wait at most (default 30, max 600)
        #[arg(long, default_value_t = 30)]
        timeout: u64,
        /// which TUI: an id, pid, label or `scope:<alias>` (required when several run)
        #[arg(long)]
        session: Option<String>,
    },
    /// Upgrade a running TUI in place to the newer installed ownpurse-tui, view restored exactly (the one non-view action an agent may send; no prompt): upgraded, deferred (a sync runs), up_to_date, preflight_failed or exec_failed
    Upgrade {
        /// which TUI: an id, pid, label or `scope:<alias>` (required when several run)
        #[arg(long)]
        session: Option<String>,
    },
    /// Pulse a running TUI's header to get the user's attention: the CLI pushes overlay states and restores the prior overlay after (always, Ctrl-C included); at most three flashes a second (too_fast)
    Flash {
        /// which TUI: an id, pid, label or `scope:<alias>` (required when several run)
        #[arg(long)]
        session: Option<String>,
        /// how many flashes (1–5)
        #[arg(long, default_value_t = 3)]
        times: u32,
        /// how long each flash shows (200ms, 0.3s); --on + --off must be at least 334 ms
        #[arg(long, default_value = "200ms")]
        on: String,
        /// the pause after each flash
        #[arg(long, default_value = "200ms")]
        off: String,
        /// the overlay to flash, token=value (repeatable; default: the header band at 3× its tint)
        #[arg(long = "token", value_name = "TOKEN=VALUE")]
        tokens: Vec<String>,
        /// flash this accent family instead
        #[arg(long, conflicts_with = "tokens")]
        accent: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum RulesCmd {
    /// Validate classify.toml: exit 0 when valid, 1 with every problem as `file:line: path: reason`
    Check,
}

/// The flags of every classify write an agent sends.
#[derive(clap::Args, Debug, Clone)]
struct AgentArgs {
    /// the store_rev you last read (`classify queue --json`); a write is refused with view_changed (exit 4) if the store moved
    #[arg(long = "expect-store-rev", value_name = "N")]
    expect_store_rev: Option<i64>,
    /// your name: the write is recorded as agent:<NAME>
    #[arg(long, value_name = "NAME")]
    client: Option<String>,
    /// one line (at most 120 characters) shown with the write and in the detail pane
    #[arg(long)]
    caption: Option<String>,
    /// allow more than 50 lines in one write
    #[arg(long = "allow-bulk")]
    allow_bulk: bool,
    /// also replace labels a human confirmed (they are skipped otherwise)
    #[arg(long)]
    overwrite: bool,
    /// send through this running TUI (default: the TUI running this profile, if any)
    #[arg(long, conflicts_with = "direct")]
    session: Option<String>,
    /// write the store directly even when a TUI is running
    #[arg(long)]
    direct: bool,
}

impl AgentArgs {
    fn flags(&self) -> classify::AgentFlags {
        classify::AgentFlags {
            expect_store_rev: self.expect_store_rev,
            client: self.client.clone(),
            caption: self.caption.clone(),
            allow_bulk: self.allow_bulk,
            overwrite: self.overwrite,
            session: self.session.clone(),
            direct: self.direct,
        }
    }
}

#[derive(Subcommand, Debug)]
enum GroupCmd {
    /// Create a group (agent write): its kind (from [groups].kinds) and a title
    Create {
        /// the group kind, e.g. split, intercompany_chain
        group_kind: String,
        /// the title
        title: String,
        #[command(flatten)]
        agent: AgentArgs,
    },
    /// Remove lines from a group (agent write)
    Remove {
        /// the group id, e.g. g12
        group: String,
        /// the lines (anchors)
        #[arg(required = true)]
        anchors: Vec<String>,
        #[command(flatten)]
        agent: AgentArgs,
    },
    /// Set a group's status (agent write): open, complete, gap or accountant
    Status {
        /// the group id, e.g. g12
        group: String,
        /// open, complete, gap or accountant
        status: String,
        #[command(flatten)]
        agent: AgentArgs,
    },
    /// Add lines to a group (agent write)
    Add {
        /// the group id, e.g. g12
        group: String,
        /// the lines (anchors)
        #[arg(required = true)]
        anchors: Vec<String>,
        /// origin, mirror or settlement
        #[arg(long)]
        role: Option<String>,
        #[command(flatten)]
        agent: AgentArgs,
    },
}

#[derive(Subcommand, Debug)]
enum ClassifyConfigCmd {
    /// Add a project to classify.toml (agent write; --dry-run shows the diff, --confirm writes it)
    AddProject {
        /// the new project id (letters, digits, - _ .)
        id: String,
        /// its display name
        name: String,
        /// an alias rules match in descriptions (repeatable)
        #[arg(long = "alias")]
        aliases: Vec<String>,
        /// show the diff only
        #[arg(long = "dry-run")]
        dry_run: bool,
        /// write it (required for a write)
        #[arg(long)]
        confirm: bool,
        #[command(flatten)]
        agent: AgentArgs,
    },
}

/// The writer's flags for manifest changes.
#[derive(clap::Args, Debug, Clone, Default)]
struct ManifestWriteArgs {
    /// your name: the change is recorded as agent:<NAME> (default: human)
    #[arg(long, value_name = "NAME")]
    client: Option<String>,
    /// the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
    #[arg(long = "expect-rev", value_name = "N")]
    expect_rev: Option<i64>,
    /// undo or discard in a manifest another actor created (agents only need it)
    #[arg(long)]
    overwrite: bool,
}

impl ManifestWriteArgs {
    fn flags(&self) -> manifest::WriteFlags {
        manifest::WriteFlags {
            client: self.client.clone(),
            expect_rev: self.expect_rev,
            overwrite: self.overwrite,
        }
    }
}

/// A new journal's flags.
#[derive(clap::Args, Debug, Clone, Default)]
struct JournalArgs {
    /// the org alias
    #[arg(long)]
    org: Option<String>,
    /// the journal date (YYYY-MM-DD)
    #[arg(long)]
    date: Option<String>,
    #[arg(long)]
    narration: Option<String>,
    /// yes|no: show on cash-basis reports (required; Xero's ShowOnCashBasisReports)
    #[arg(long = "cash-basis", value_name = "yes|no")]
    cash_basis: Option<String>,
    /// CODE=AMOUNT[:description], repeated; debit positive, credit negative (or `Dr`/`Cr` before the amount)
    #[arg(long = "line", value_name = "CODE=AMOUNT[:DESC]")]
    lines: Vec<String>,
    /// the journal as JSON ({org, date, narration, cash_basis, lines}) or CSV (account,description,amount)
    #[arg(long, conflicts_with = "lines")]
    file: Option<String>,
    /// why (shown in reviews)
    #[arg(long)]
    why: Option<String>,
    /// the accountant's decision reference (an adjusting-entry id, an email)
    #[arg(long)]
    accountant: Option<String>,
}

impl JournalArgs {
    fn flags(&self) -> manifest::JournalFlags {
        manifest::JournalFlags {
            org: self.org.clone(),
            date: self.date.clone(),
            narration: self.narration.clone(),
            cash_basis: self.cash_basis.clone(),
            lines: self.lines.clone(),
            file: self.file.clone(),
            why: self.why.clone(),
            accountant: self.accountant.clone(),
        }
    }
}

#[derive(Subcommand, Debug)]
enum ManifestCmd {
    /// Create a manifest (changes collected locally, reviewed as a diff) and switch to it
    New {
        name: String,
        /// why: one line, shown in the list, the review and the package title
        why: Option<String>,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// List manifests (main first) with their stage and issues
    List,
    /// Make a manifest (or main) the default for the read commands
    Switch { name: String },
    /// The entries, the stage and the commands; or one entry's field diff (`show <seq>`, `--patch` for the raw ops)
    Show {
        seq: Option<i64>,
        /// the raw RFC 6902 patch
        #[arg(long)]
        patch: bool,
    },
    /// The manifest's events: entries, undos, notes, acknowledgements, plans
    Log,
    /// The effect against main: TB, statements, intercompany bridges, must-be-zero accounts
    Diff {
        /// as-of date (default: the latest cached Xero TB date)
        #[arg(long)]
        date: Option<String>,
        /// CSV instead of text
        #[arg(long)]
        csv: bool,
    },
    /// Every issue, locally (no Xero call): errors block plan, warnings need acknowledging
    Check {
        #[arg(long)]
        date: Option<String>,
    },
    /// Acknowledge a warning (an undoable event): `ack <entry> <code> [--path P]`
    Ack {
        entry: i64,
        code: String,
        #[arg(long)]
        path: Option<String>,
        /// withdraw the acknowledgement
        #[arg(long)]
        withdraw: bool,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// Compile the checked manifest into the sealed write package (local; prints the approve and apply commands)
    Plan {
        #[arg(long)]
        date: Option<String>,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// The owner approves the planned package at his own terminal (= write approve-record; refused in agent sessions)
    Approve {
        name: String,
        /// the planned package's full hash
        #[arg(long)]
        package: String,
        #[arg(long)]
        response_file: std::path::PathBuf,
        #[arg(long)]
        i_am_the_owner: bool,
    },
    /// ⇅ Apply the approved package (= write run; needs OWNPURSE_WRITE_RUN=<hash> for this one run)
    Apply {
        name: String,
        /// the planned package's full hash
        #[arg(long)]
        package: String,
        #[arg(long)]
        batch: Option<usize>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Add any document change: its "after" (`--after file.json`) or an RFC 6902 patch (`--patch file.json`)
    Add {
        /// the entity (ManualJournals, BankTransactions, …)
        #[arg(long)]
        entity: String,
        #[arg(long)]
        org: String,
        /// the document id (omit for a new document)
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        after: Option<String>,
        #[arg(long)]
        patch: Option<String>,
        #[arg(long, default_value = "")]
        why: String,
        #[arg(long)]
        accountant: Option<String>,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// A new manual journal (NoTax; check reports an unbalanced one)
    Journal {
        #[command(flatten)]
        journal: JournalArgs,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// Replace a new-journal entry with an edited one (shown as "edited → n")
    Edit {
        seq: i64,
        #[command(flatten)]
        journal: JournalArgs,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// Recode a bank line's account and/or description (org:bt:id#LINE; LINE a line id, n<k> or L<k>)
    Recode {
        line: String,
        #[arg(long)]
        account: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, default_value = "")]
        why: String,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// Void a document (org:mj:id)
    Void {
        doc: String,
        #[arg(long, default_value = "")]
        why: String,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// Attach a PDF, CSV or TXT to a journal (org:mj:id <file>; permanent once applied)
    Attach {
        doc: String,
        file: std::path::PathBuf,
        #[arg(long, default_value = "")]
        why: String,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// A note in a journal's or bank transaction's history (org:mj:id or org:bt:id "<text>"; permanent once
    /// applied; at most 250 characters)
    Note {
        doc: String,
        text: String,
        #[arg(long, default_value = "")]
        why: String,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// How a reconciled bank line's recode is carried out: `reclass` (a reclass journal, the default) or `browser`
    /// (unreconcile, edit, re-reconcile: a checklist step); `resolve default <how>` sets the manifest's default
    Resolve {
        /// the entry number, or `default`
        entry: String,
        /// reclass | browser
        how: String,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// Undo an entry (default: the newest), or `--restore` it
    Undo {
        seq: Option<i64>,
        #[arg(long)]
        restore: bool,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// Attach an accountant decision reference and/or a note to an entry
    Annotate {
        seq: i64,
        #[arg(long)]
        accountant: Option<String>,
        #[arg(long)]
        note: Option<String>,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
    /// Discard a manifest (a marker; its history stays)
    Discard {
        name: Option<String>,
        #[command(flatten)]
        w: ManifestWriteArgs,
    },
}

#[derive(Subcommand, Debug)]
enum WriteCmd {
    /// Compile changes (ops.json) against the local record into an unsealed package (no Xero call)
    Plan {
        /// the requested operations (JSON)
        input: std::path::PathBuf,
        /// where to write the package (default: `<input>.package.json` beside it)
        #[arg(long)]
        out: Option<std::path::PathBuf>,
    },
    /// Seal a package (its SHA-256), write the review PDF and CSV for the accountant, print the review
    Seal {
        package: std::path::PathBuf,
        /// folder for the PDF and CSV (default: beside the package)
        #[arg(long)]
        dir: Option<std::path::PathBuf>,
    },
    /// Re-write the review PDF and CSV of a sealed package
    Export {
        package: std::path::PathBuf,
        #[arg(long)]
        dir: Option<std::path::PathBuf>,
    },
    /// The owner records his approval at his own terminal (refused in agent sessions)
    ApproveRecord {
        package: std::path::PathBuf,
        /// a file with the owner's response, verbatim (it must quote the hash)
        #[arg(long)]
        response_file: std::path::PathBuf,
        /// required: you are the owner
        #[arg(long)]
        i_am_the_owner: bool,
        /// op ids it covers, comma-separated (default: all)
        #[arg(long)]
        scope: Option<String>,
    },
    /// Import the owner's typed chat approval (quoting the full hash) from his own session transcript
    ApproveImport {
        package: std::path::PathBuf,
        /// the session's transcript (`<session id>.jsonl`)
        #[arg(long)]
        transcript: std::path::PathBuf,
        /// the session id
        #[arg(long)]
        session: String,
        #[arg(long)]
        scope: Option<String>,
    },
    /// Whether an approval is recorded for the package's hash (exit 0, or 3)
    ApproveCheck { package: std::path::PathBuf },
    /// Record the owner's standing Demo Company approval for this package (demo profiles only; agents may run it)
    ApproveDemo { package: std::path::PathBuf },
    /// ⇅ Execute the next batch (at most 10) of an approved package: needs allow_writes, OWNPURSE_WRITE_RUN=<hash> and a recorded approval
    Run {
        package: std::path::PathBuf,
        /// operations in this batch (1–10, default 10)
        #[arg(long)]
        batch: Option<usize>,
        /// check preconditions live and print each request; send nothing
        #[arg(long)]
        dry_run: bool,
    },
    /// ⇅ Re-read each landed target and the trial balance movement against the expected deltas (exit 5 on a discrepancy)
    Verify { package: std::path::PathBuf },
    /// ⇅ Settle an uncertain attempt by read-back (never by re-sending)
    Reconcile {
        package: std::path::PathBuf,
        op: String,
    },
    /// ⇅ Build the recovery package for what landed (unsealed; it needs its own approval)
    Recover {
        package: std::path::PathBuf,
        /// one operation (default: every landed one)
        op: Option<String>,
        #[arg(long)]
        out: Option<std::path::PathBuf>,
    },
    /// The attempt log of a package and the record's chain (no Xero call)
    Status { package: std::path::PathBuf },
    /// Milestone 1: write a probe's plan input from the synced Demo Company record (Demo Company profiles only)
    Probe {
        /// recode-reconciled | void-after-attachment | idempotency | line-limit
        name: String,
        #[arg(long)]
        org: Option<String>,
        /// line-limit: the journal's line count (default 100)
        #[arg(long)]
        lines: Option<usize>,
        /// folder for the inputs (default: probes)
        #[arg(long)]
        out: Option<std::path::PathBuf>,
    },
    /// ⇅ Milestone 1, Demo Company only: re-send a landed journal under its own Idempotency-Key (the op must be flagged probe_replay)
    ProbeReplay {
        package: std::path::PathBuf,
        op: String,
        /// change the narration (the marker kept) under the same key
        #[arg(long)]
        changed: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ClassifyCmd {
    /// Set labels on lines (agent write): anchors or a filter (status:open unless it says), then <type>=<id>…
    Set {
        /// anchors or filter words, and labels as <type>=<id>, e.g. `org:roast month:2025-09 project=harvest`
        #[arg(required = true)]
        words: Vec<String>,
        #[command(flatten)]
        agent: AgentArgs,
    },
    /// Add a note to a line or a group (agent write)
    Note {
        /// an anchor or a group id (g12)
        target: String,
        /// the note
        text: String,
        #[command(flatten)]
        agent: AgentArgs,
    },
    /// Groups (agent writes)
    Group {
        #[command(subcommand)]
        action: GroupCmd,
    },
    /// Undo an agent write: --batch (the latest agent batch), an anchor (the last agent write on it) or --events
    Undo {
        /// the line whose last agent write to undo
        anchor: Option<String>,
        /// the latest agent batch
        #[arg(long)]
        batch: bool,
        /// these event ids (agent events only), comma-separated
        #[arg(long, value_delimiter = ',')]
        events: Vec<i64>,
        #[command(flatten)]
        agent: AgentArgs,
    },
    /// Edit classify.toml (agent writes)
    Config {
        #[command(subcommand)]
        action: ClassifyConfigCmd,
    },
    /// Run the rules over the lines in the filter and store their suggestions (local store only; nothing is confirmed, nothing goes to Xero)
    Suggest {
        /// queue filter, e.g. `org:roast month:2025-09 status:all` (default: open lines)
        filter: Vec<String>,
        /// report what would be written (per rule: lines matched, suggestions per label type); append nothing
        #[arg(long)]
        dry_run: bool,
        /// `rules` (Jev is not enabled in this release: `jev` refuses with jev_not_enabled, nothing is sent)
        #[arg(long, default_value = "rules", value_parser = ["rules", "jev"])]
        engine: String,
    },
    /// Check the classification config
    Rules {
        #[command(subcommand)]
        action: RulesCmd,
    },
    /// One CSV row per line: Xero facts, each label and its source, stale, notes (default: every line)
    Export {
        /// CSV on stdout (the only format)
        #[arg(long)]
        csv: bool,
        /// queue filter, e.g. `org:design project:refit`
        filter: Vec<String>,
    },
    /// The classification queue as data: lines without confirmed required labels, or with stale ones, largest first
    Queue {
        /// queue filter: `org:<alias> acct:<prefix> month:YYYY-MM year:YYYY project:<id> conf<0.7 by:agent since:today status:open|all|stale` and words
        filter: Vec<String>,
        /// `stale` (stale lines first, then largest; the default), `amount` (largest first) or `date`
        #[arg(long, value_parser = ["stale", "amount", "date"])]
        sort: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum WalkCmd {
    /// Create an empty walkthrough file (walkthroughs.dir, default walkthroughs/ beside ownpurse.json)
    New {
        id: String,
        #[arg(long)]
        title: String,
        /// lay | informed
        #[arg(long)]
        audience: Option<String>,
    },
    /// Append a step: narration, optional anchored hint/rings, view verbs, checks, a visualisation spec
    AddStep {
        id: String,
        #[arg(long)]
        title: String,
        /// the body: plain English, **bold**, [term] glossary links
        #[arg(long)]
        text: String,
        /// a one-line "why this matters" (lay mode)
        #[arg(long)]
        why: Option<String>,
        /// an anchor (row:code=6140, chip:stage, diff:/Date, …); repeat for rings on more
        #[arg(long = "anchor")]
        anchors: Vec<String>,
        /// the hint text on the first anchor
        #[arg(long)]
        hint: Option<String>,
        #[arg(long = "hint-title")]
        hint_title: Option<String>,
        /// an arrow from the hint to its anchor
        #[arg(long)]
        arrow: bool,
        #[arg(long)]
        ring: bool,
        #[arg(long)]
        spotlight: bool,
        /// a view verb the step applies (`tab 2`, `scope roast`, `select 6140`, `drill`); repeat, in order
        #[arg(long = "view")]
        verbs: Vec<String>,
        /// an assertion: `amount(org, account, YYYY-MM-DD) = value` or `count(org, account[, from, to]) = n`
        #[arg(long = "check")]
        checks: Vec<String>,
        /// a visualisation spec (JSON `{kind, …}`)
        #[arg(long)]
        viz: Option<String>,
    },
    /// A walkthrough's steps
    Show { id: String },
    /// The walkthroughs in the folder
    List,
    /// Lay every step out at 140x40, 100x30 and 80x24: anchors resolve, checks hold, glossary links exist, no box over a protected region
    Validate { id: String },
    /// Write every step's frame (text, svg, png) for review or as a slideshow
    Render {
        id: String,
        #[arg(long)]
        out: std::path::PathBuf,
        /// txt, svg, png, html (comma-separated)
        #[arg(long, default_value = "txt")]
        format: String,
        #[arg(long, default_value = "100x30")]
        sizes: String,
    },
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Launch a scripted walkthrough in the real TUI over an isolated mock database (offline; no setup)
    Demo {
        /// seconds each step stays on screen; moving the view yourself stops autoplay
        #[arg(long, default_value_t = 8, value_parser = clap::value_parser!(u64).range(1..=60))]
        step_seconds: u64,
    },
    /// Manifests: collect changes as document diffs, check them locally, plan them into a sealed write package (nothing reaches Xero before approve + apply)
    Manifest {
        #[command(subcommand)]
        action: ManifestCmd,
    },
    /// Walkthroughs: guided, replayable explanations played in the TUI (`W`); files beside ownpurse.json
    Walkthrough {
        #[command(subcommand)]
        action: WalkCmd,
    },
    /// Classify lines locally: rules suggest project, bearer and treatment; labels live in db/classify.sqlite, never in Xero
    Classify {
        #[command(subcommand)]
        action: ClassifyCmd,
    },
    /// The agent bridge: read and change a running TUI's view (view only; local socket)
    Ui {
        #[command(subcommand)]
        action: UiCmd,
    },
    /// Xero changes as a sealed, approved package (plan, seal, approve; the run side is separate)
    Write {
        #[command(subcommand)]
        action: WriteCmd,
    },
    /// Check the setup (binaries on PATH, config, record, token file permissions); read-only, prints fixes
    Doctor,
    /// ⇅ Mirror entities into the local record (calls Xero)
    Sync {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// entities to sync (default: all), e.g. Accounts BankTransactions
        entities: Vec<String>,
        /// settings tables only
        #[arg(long, conflicts_with = "transactions")]
        reference: bool,
        /// transaction tables only
        #[arg(long)]
        transactions: bool,
        /// `jsonl`: one JSON progress event per line on stderr (for the TUI)
        #[arg(long, hide = true, value_parser = ["jsonl"])]
        progress: Option<String>,
    },
    /// ⇅ Browser consent (PKCE): read access plus anything already granted; one consent covers every org
    Login {
        /// seconds to wait for consent (default: config)
        #[arg(long)]
        timeout: Option<u64>,
        /// print the URL only
        #[arg(long)]
        no_browser: bool,
        /// also request the write scopes (Phase 8; only for a profile with allow_writes)
        #[arg(long)]
        write: bool,
        /// `jsonl`: one JSON progress event per line on stderr (for the TUI)
        #[arg(long, hide = true, value_parser = ["jsonl"])]
        progress: Option<String>,
    },
    /// ⇅ List connected organisations (1 call; `--offline` reads the token file's last list)
    Orgs,
    /// The CLI reference as Markdown (for the docs site)
    #[command(hide = true)]
    HelpMarkdown,
    /// Chart of accounts (one org or all)
    Accounts {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// bank accounts only
        #[arg(long)]
        bank: bool,
        /// account type, e.g. EXPENSE, BANK, CURRLIAB
        #[arg(long = "type", value_name = "TYPE")]
        kind: Option<String>,
        /// substring of name, contact, reference or description
        #[arg(long)]
        search: Option<String>,
        /// include archived
        #[arg(long)]
        archived: bool,
    },
    /// Organisation details: plan class, lock dates
    Org {
        /// org alias or tenant id
        org: String,
    },
    /// Contacts
    Contacts {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// name contains
        search: Option<String>,
        /// include archived
        #[arg(long)]
        archived: bool,
    },
    /// Bank transactions (spend/receive money)
    Txns {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// from date (YYYY-MM-DD, inclusive)
        #[arg(long = "from", value_name = "DATE")]
        since: Option<String>,
        /// to date (YYYY-MM-DD, inclusive)
        #[arg(long = "to", value_name = "DATE")]
        until: Option<String>,
        /// bank account code or name substring
        #[arg(long)]
        account: Option<String>,
        /// document status, e.g. AUTHORISED, POSTED, DRAFT
        #[arg(long)]
        status: Option<String>,
        /// substring anywhere in the record
        #[arg(long)]
        search: Option<String>,
        /// include DELETED
        #[arg(long)]
        deleted: bool,
    },
    /// Manual journals
    Journals {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// from date (YYYY-MM-DD, inclusive)
        #[arg(long = "from", value_name = "DATE")]
        since: Option<String>,
        /// to date (YYYY-MM-DD, inclusive)
        #[arg(long = "to", value_name = "DATE")]
        until: Option<String>,
        /// document status, e.g. AUTHORISED, POSTED, DRAFT
        #[arg(long)]
        status: Option<String>,
        /// substring of name, contact, reference or description
        #[arg(long)]
        search: Option<String>,
    },
    /// Bank transfers
    Transfers {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// from date (YYYY-MM-DD, inclusive)
        #[arg(long = "from", value_name = "DATE")]
        since: Option<String>,
        /// to date (YYYY-MM-DD, inclusive)
        #[arg(long = "to", value_name = "DATE")]
        until: Option<String>,
    },
    /// One record: current payload and version history
    Show {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// entity name, e.g. BankTransactions, ManualJournals, Invoices
        entity: String,
        /// id or unique prefix
        id: String,
    },
    /// The document's page in the Xero web app, through the org switch (local; no Xero call)
    Open {
        /// org alias or tenant id
        org: String,
        /// BankTransactions, ManualJournals, Invoices, or BankRec (a bank account's reconcile queue)
        entity: String,
        /// id or unique prefix (BankRec: the bank account's AccountID)
        id: String,
        /// also open it in the browser (macOS `open`)
        #[arg(long)]
        launch: bool,
    },
    /// List any mirrored entity (Invoices, Payments, CreditNotes, TaxRates, …)
    List {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// entity name, e.g. BankTransactions, ManualJournals, Invoices
        entity: String,
        /// from date (YYYY-MM-DD, inclusive)
        #[arg(long = "from", value_name = "DATE")]
        since: Option<String>,
        /// to date (YYYY-MM-DD, inclusive)
        #[arg(long = "to", value_name = "DATE")]
        until: Option<String>,
        /// document status, e.g. AUTHORISED, POSTED, DRAFT
        #[arg(long)]
        status: Option<String>,
        /// substring anywhere in the record
        #[arg(long)]
        search: Option<String>,
        /// comma list of payload paths, e.g. Contact.Name,Total
        #[arg(long)]
        fields: Option<String>,
    },
    /// Local general ledger: postings derived from mirrored documents
    Gl {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// from date (YYYY-MM-DD, inclusive)
        #[arg(long = "from", value_name = "DATE")]
        since: Option<String>,
        /// to date (YYYY-MM-DD, inclusive)
        #[arg(long = "to", value_name = "DATE")]
        until: Option<String>,
        /// code, name, or id (substring match if no exact match)
        #[arg(long)]
        account: Option<String>,
        /// running balance from local documents only (default opens from the cached Xero TB)
        #[arg(long)]
        full_history: bool,
        /// BankTransactions | BankTransfers | ManualJournals | Invoices | ...
        #[arg(long)]
        source: Option<String>,
        /// substring of name, contact, reference or description
        #[arg(long)]
        search: Option<String>,
    },
    /// Local trial balance computed from the ledger (exact decimals)
    Tb {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// as-of date (YYYY-MM-DD)
        #[arg(long)]
        date: String,
        /// open from Xero's cached TB at this date, add later activity
        #[arg(long)]
        since: Option<String>,
        /// balances from local documents only (default: cached Xero TB at prior year-end + activity)
        #[arg(long)]
        full_history: bool,
        /// 2–4 orgs side by side (rows merge only by code / map)
        #[arg(long)]
        compare: bool,
        /// with --compare: code (default) or account_map category
        #[arg(long, value_parser = ["code", "map"])]
        align: Option<String>,
    },
    /// Profit and loss from the local ledger (accrual or --cash), checked against Xero's cached report
    Pl {
        /// org alias or tenant id
        org: String,
        /// period start (default: the fiscal year's start)
        #[arg(long)]
        from: Option<String>,
        /// period end (YYYY-MM-DD)
        #[arg(long)]
        to: String,
        /// cash basis (Xero's paymentsOnly)
        #[arg(long)]
        cash: bool,
        /// exit 1 when anything differs from Xero's cached report
        #[arg(long)]
        strict: bool,
    },
    /// Balance sheet from the local ledger (accrual or --cash), checked against Xero's cached report
    Bs {
        /// org alias or tenant id
        org: String,
        /// as-of date (YYYY-MM-DD)
        #[arg(long)]
        date: String,
        /// cash basis (Xero's paymentsOnly)
        #[arg(long)]
        cash: bool,
        /// exit 1 when anything differs from Xero's cached report
        #[arg(long)]
        strict: bool,
        /// open from Xero's cached balance sheet at this date (default: the prior fiscal year-end, when cached)
        #[arg(long)]
        since: Option<String>,
        /// balances from local documents only
        #[arg(long)]
        full_history: bool,
    },
    /// Compare the local trial balance with Xero's cached one, account by account
    Parity {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// as-of date (YYYY-MM-DD)
        #[arg(long)]
        date: String,
        /// open from Xero's cached TB at this date
        #[arg(long)]
        since: Option<String>,
        /// reuse a cached Xero TB snapshot (always the case here: nothing is fetched)
        #[arg(long)]
        cached: bool,
        /// exit 1 when anything differs
        #[arg(long)]
        strict: bool,
    },
    /// Xero's reports: fetch one into the record (⇅ one call per org), or list what the record holds
    Report {
        #[command(subcommand)]
        action: ReportCmd,
    },
    /// A file for the accountant from the record (zero calls): tb / gl from the local ledger, bs / pl from
    /// Xero's cached reports
    Export {
        /// org alias or tenant id
        org: String,
        /// tb | gl | bs | pl
        kind: String,
        /// as-of date for tb / bs (default: the last fiscal year-end)
        #[arg(long)]
        date: Option<String>,
        /// period start for gl / pl (default: that fiscal year's start)
        #[arg(long)]
        from: Option<String>,
        /// period end for gl / pl
        #[arg(long)]
        to: Option<String>,
        /// CSV (the default)
        #[arg(long)]
        csv: bool,
        /// a spreadsheet (not yet: use --csv)
        #[arg(long)]
        xlsx: bool,
        /// the file to write (default `<org>-<kind>-<date>.csv` here; never over an existing file)
        #[arg(long)]
        out: Option<std::path::PathBuf>,
    },
    /// ⇅ A document's attachments from Xero, into the record; `--get` saves one file (`--cached`: the record)
    Attachments {
        /// org alias or tenant id
        org: String,
        /// BankTransactions, ManualJournals, Invoices, Contacts, …
        entity: String,
        /// the document's id
        id: String,
        /// what the record last saw (zero calls)
        #[arg(long)]
        cached: bool,
        /// save this attachment's bytes (by file name)
        #[arg(long)]
        get: Option<String>,
        /// where to save it (default: the file name here; never over an existing file)
        #[arg(long)]
        out: Option<std::path::PathBuf>,
    },
    /// ⇅ A document's history and notes from Xero, into the record (`--cached`: what the record holds)
    History {
        /// org alias or tenant id
        org: String,
        /// BankTransactions, ManualJournals, Invoices, Contacts, BankTransfers, Payments, …
        entity: String,
        /// the document's id
        id: String,
        /// what the record last saw (zero calls)
        #[arg(long)]
        cached: bool,
    },
    /// API usage: calls, quotas, endpoints, records fetched
    Usage {
        /// window start (default: today, UTC)
        #[arg(long)]
        since: Option<String>,
        /// window of the last N days
        #[arg(long)]
        days: Option<i64>,
        /// recent calls to list
        #[arg(long, default_value_t = 20)]
        calls: i64,
    },
    /// Effective settings with their source; links; edit
    Config {
        /// show (effective settings + origin), path, links, edit
        #[arg(default_value = "show", value_parser = ["show", "path", "links", "edit"])]
        action: String,
    },
    /// Intercompany pairs from config: both sides, difference, bridge, residual (zero calls)
    Ic {
        /// pair name from `intercompany.pairs` (default: every pair)
        pair: Option<String>,
        /// as-of date (default: the scoped org's last fiscal year-end)
        #[arg(long = "as-of", value_name = "DATE")]
        as_of: Option<String>,
    },
    /// Explainer models (bridge, flow, t_account, timeline, before_after, checklist): the numbers the TUI's panels show (zero calls)
    Viz {
        /// bridge | flow | t_account | timeline | before_after | checklist
        #[arg(value_parser = ["bridge", "flow", "t_account", "timeline", "before_after", "checklist"])]
        kind: String,
        /// bridge: the pair id; t_account, timeline, before_after: the org; checklist: the file (relative to the config's folder)
        target: Option<String>,
        /// t_account, timeline: the account code
        account: Option<String>,
        /// the date (bridge, flow owed; the period's end otherwise). Default: the first org's last fiscal year-end
        #[arg(long = "as-of", value_name = "DATE")]
        as_of: Option<String>,
        /// the period's first day (default: the fiscal-year start)
        #[arg(long, value_name = "DATE")]
        from: Option<String>,
        /// the period's last day (default: --as-of)
        #[arg(long, value_name = "DATE")]
        to: Option<String>,
        /// flow: the orgs to show, comma-separated (default: every org)
        #[arg(long)]
        orgs: Option<String>,
        /// flow: owed (balances at the date) or paid (movements over the period)
        #[arg(long, value_parser = ["owed", "paid"])]
        basis: Option<String>,
        /// t_account: the largest postings listed per side (default 5)
        #[arg(long)]
        top: Option<usize>,
        /// the whole spec as JSON (the TUI's `viz` verb takes the same), overriding the arguments
        #[arg(long)]
        spec: Option<String>,
    },
}

/// The first positional word (the subcommand), skipping global flags and their values.
fn command_word(args: &[String]) -> Option<&str> {
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--profile" || a == "--limit" {
            i += 2;
            continue;
        }
        if a.starts_with('-') {
            i += 1;
            continue;
        }
        return Some(a);
    }
    None
}

fn run(cli: Cli) -> Res<()> {
    let g = Global {
        profile: cli.profile.clone(),
        json: cli.json,
        full: cli.full,
        limit: cli.limit,
        offline: cli.offline,
        manifest: cli.manifest.clone(),
    };
    if let Cmd::Demo { step_seconds } = &cli.cmd {
        return demo::run(*step_seconds);
    }
    if matches!(cli.cmd, Cmd::Doctor) {
        return doctor::doctor(&g);
    }
    if let Cmd::Ui { action } = &cli.cmd {
        return match action {
            UiCmd::Ls => ui::ls(&g),
            UiCmd::Wait {
                until,
                timeout,
                session,
            } => ui::wait_until(&g, session.as_deref(), until, *timeout),
            UiCmd::Upgrade { session } => ui::upgrade(&g, session.as_deref()),
            UiCmd::Flash {
                session,
                times,
                on,
                off,
                tokens,
                accent,
            } => ui::flash(
                &g,
                &ui::FlashArgs {
                    session: session.clone(),
                    times: *times,
                    on: on.clone(),
                    off: off.clone(),
                    tokens: tokens.clone(),
                    accent: accent.clone(),
                },
            ),
            UiCmd::State {
                session,
                all_rows,
                rows,
                fields,
            } => ui::state(
                &g,
                session.as_deref(),
                if *all_rows {
                    Some("all".to_string())
                } else {
                    rows.clone()
                },
                fields.as_deref(),
            ),
            UiCmd::Screen {
                format,
                out,
                session,
                from_state: Some(f),
                allow_record_drift,
                allow_config_drift,
                allow_classify_drift,
            } => {
                let _ = session;
                ui::screen_from_state(
                    &g,
                    format,
                    out.clone(),
                    f,
                    *allow_record_drift,
                    *allow_config_drift,
                    *allow_classify_drift,
                )
            }
            UiCmd::Screen {
                format,
                out,
                session,
                ..
            } => ui::screen(&g, format, out.clone(), session.as_deref()),
            UiCmd::Send {
                verbs,
                expect_rev,
                expect,
                screen,
                out,
                json_out,
                caption,
                client,
                session,
                full_state,
            } => ui::send(
                &g,
                &ui::SendArgs {
                    verbs: verbs.clone(),
                    expect_rev: *expect_rev,
                    expect: expect.clone(),
                    screen: screen.clone(),
                    json_out: *json_out,
                    out: out.clone(),
                    caption: caption.clone(),
                    client: client.clone(),
                    session: session.clone(),
                    full_state: *full_state,
                },
            ),
        };
    }
    if let Cmd::Classify {
        action: ClassifyCmd::Suggest { engine, .. },
    } = &cli.cmd
        && engine == "jev"
    {
        // before the config, record or store open: Jev is refused with nothing read and nothing sent
        return Err(classify::jev_not_enabled());
    }
    if matches!(cli.cmd, Cmd::HelpMarkdown) {
        print!("{}", docs_md::markdown(&Cli::command(), &NOT_AVAILABLE));
        return Ok(());
    }
    let mut app = App::new(g)?;
    match cli.cmd {
        Cmd::Demo { .. } | Cmd::Doctor | Cmd::HelpMarkdown | Cmd::Ui { .. } => {
            unreachable!("handled before the config loads")
        }
        Cmd::Sync {
            org,
            entities,
            reference,
            transactions,
            progress,
        } => net::sync(
            &mut app,
            &org,
            &entities,
            reference,
            transactions,
            progress.is_some(),
        ),
        Cmd::Login {
            timeout,
            no_browser,
            progress,
            write,
        } => net::login(&mut app, timeout, no_browser, progress.is_some(), write),
        Cmd::Orgs => net::orgs(&mut app),
        Cmd::Accounts {
            org,
            bank,
            kind,
            search,
            archived,
        } => cmds::accounts(
            &mut app,
            &cmds::AccountsArgs {
                org,
                bank,
                kind,
                search,
                archived,
            },
        ),
        Cmd::Org { org } => cmds::org(&mut app, &org),
        Cmd::Contacts {
            org,
            search,
            archived,
        } => cmds::contacts(&mut app, &org, search.as_deref(), archived),
        Cmd::Txns {
            org,
            since,
            until,
            account,
            status,
            search,
            deleted,
        } => cmds::txns(
            &mut app,
            &cmds::TxnArgs {
                org,
                since,
                until,
                account,
                status,
                search,
                deleted,
            },
        ),
        Cmd::Journals {
            org,
            since,
            until,
            status,
            search,
        } => cmds::journals(
            &mut app,
            &cmds::JournalArgs {
                org,
                since,
                until,
                status,
                search,
            },
        ),
        Cmd::Transfers { org, since, until } => {
            cmds::transfers(&mut app, &org, since.as_deref(), until.as_deref())
        }
        Cmd::Show { org, entity, id } => cmds::show(&mut app, &org, &entity, &id),
        Cmd::Open {
            org,
            entity,
            id,
            launch,
        } => cmds::open(&mut app, &org, &entity, &id, launch),
        Cmd::List {
            org,
            entity,
            since,
            until,
            status,
            search,
            fields,
        } => cmds::list(
            &mut app,
            &cmds::ListArgs {
                org,
                entity,
                since,
                until,
                status,
                search,
                fields,
            },
        ),
        Cmd::Gl {
            org,
            since,
            until,
            account,
            full_history,
            source,
            search,
        } => ledger::gl(
            &mut app,
            &ledger::GlArgs {
                org,
                since,
                until,
                account,
                full_history,
                source,
                search,
            },
        ),
        Cmd::Tb {
            org,
            date,
            full_history,
            compare: true,
            align,
            ..
        } => ledger::tb_compare(
            &mut app,
            &ledger::CompareArgs {
                org,
                date,
                full_history,
                align,
            },
        ),
        Cmd::Tb {
            org,
            date,
            since,
            full_history,
            ..
        } => ledger::tb(
            &mut app,
            &ledger::TbArgs {
                org,
                date,
                since,
                full_history,
            },
        ),
        Cmd::Export {
            org,
            kind,
            date,
            from,
            to,
            xlsx,
            out,
            ..
        } => ledger::export(&mut app, &org, &kind, date, from, to, xlsx, out),
        Cmd::Attachments {
            org,
            entity,
            id,
            cached,
            get,
            out,
        } => net::attachments(
            &mut app,
            &org,
            &entity,
            &id,
            cached,
            get.as_deref(),
            out.as_deref(),
        ),
        Cmd::History {
            org,
            entity,
            id,
            cached,
        } => net::history(&mut app, &org, &entity, &id, cached),
        Cmd::Report { action } => match action {
            ReportCmd::Fetch {
                org,
                kind,
                date,
                from,
                to,
                cash,
            } => net::report_fetch(&mut app, &org, &kind, date, from, to, cash),
            ReportCmd::List { org, kind } => cmds::report_list(&mut app, &org, kind.as_deref()),
        },
        Cmd::Pl {
            org,
            from,
            to,
            cash,
            strict,
        } => statements::pl(
            &mut app,
            &statements::StatementArgs {
                org,
                from,
                to,
                cash,
                strict,
                since: None,
                full_history: false,
            },
        ),
        Cmd::Bs {
            org,
            date,
            cash,
            strict,
            since,
            full_history,
        } => statements::bs(
            &mut app,
            &statements::StatementArgs {
                org,
                from: None,
                to: date,
                cash,
                strict,
                since,
                full_history,
            },
        ),
        Cmd::Parity {
            org,
            date,
            since,
            strict,
            ..
        } => ledger::parity(
            &mut app,
            &ledger::ParityArgs {
                org,
                date,
                since,
                strict,
            },
        ),
        Cmd::Usage { since, days, calls } => usage::usage(&mut app, since.as_deref(), days, calls),
        Cmd::Config { action } => config_cmd::config(&mut app, &action),
        Cmd::Ic { pair, as_of } => ic::ic(&mut app, pair.as_deref(), as_of.as_deref()),
        Cmd::Viz {
            kind,
            target,
            account,
            as_of,
            from,
            to,
            orgs,
            basis,
            top,
            spec,
        } => viz::viz(
            &mut app,
            &viz::Args {
                kind,
                target,
                account,
                as_of,
                from,
                to,
                orgs,
                basis,
                top,
                spec,
            },
        ),
        Cmd::Write { action } => match action {
            WriteCmd::Plan { input, out } => write::plan_cmd(&mut app, &input, out.as_deref()),
            WriteCmd::Seal { package, dir } => write::seal_cmd(&mut app, &package, dir.as_deref()),
            WriteCmd::Export { package, dir } => {
                write::export_cmd(&mut app, &package, dir.as_deref())
            }
            WriteCmd::ApproveRecord {
                package,
                response_file,
                i_am_the_owner,
                scope,
            } => write::approve_record(
                &mut app,
                &package,
                &response_file,
                i_am_the_owner,
                scope.as_deref(),
            ),
            WriteCmd::ApproveImport {
                package,
                transcript,
                session,
                scope,
            } => write::approve_import(&mut app, &package, &transcript, &session, scope.as_deref()),
            WriteCmd::ApproveCheck { package } => write::approve_check(&mut app, &package),
            WriteCmd::ApproveDemo { package } => write_run::approve_demo_cmd(&app, &package),
            WriteCmd::Run {
                package,
                batch,
                dry_run,
            } => write_run::run_cmd(&app, &package, batch, dry_run),
            WriteCmd::Verify { package } => write_run::verify_cmd(&app, &package),
            WriteCmd::Reconcile { package, op } => write_run::reconcile_cmd(&app, &package, &op),
            WriteCmd::Recover { package, op, out } => {
                write_run::recover_cmd(&app, &package, op.as_deref(), out.as_deref())
            }
            WriteCmd::Status { package } => write_run::status_cmd(&mut app, &package),
            WriteCmd::Probe {
                name,
                org,
                lines,
                out,
            } => write_run::probe_cmd(&mut app, &name, org.as_deref(), lines, out.as_deref()),
            WriteCmd::ProbeReplay {
                package,
                op,
                changed,
            } => write_run::replay_cmd(&app, &package, &op, changed),
        },
        Cmd::Manifest { action } => match action {
            ManifestCmd::New { name, why, w } => {
                manifest::new(&mut app, &name, why.as_deref().unwrap_or(""), &w.flags())
            }
            ManifestCmd::List => manifest::list(&mut app),
            ManifestCmd::Switch { name } => manifest::switch(&mut app, &name),
            ManifestCmd::Show { seq, patch } => manifest::show(&mut app, seq, patch),
            ManifestCmd::Log => manifest::log(&mut app),
            ManifestCmd::Diff { date, csv } => manifest::diff(&mut app, date, csv),
            ManifestCmd::Check { date } => manifest::check_cmd(&mut app, date),
            ManifestCmd::Ack {
                entry,
                code,
                path,
                withdraw,
                w,
            } => manifest::ack(&mut app, entry, &code, path, withdraw, &w.flags()),
            ManifestCmd::Plan { date, w } => manifest::plan(&mut app, date, &w.flags()),
            ManifestCmd::Approve {
                name,
                package,
                response_file,
                i_am_the_owner,
            } => manifest::approve(&mut app, &name, &package, &response_file, i_am_the_owner),
            ManifestCmd::Apply {
                name,
                package,
                batch,
                dry_run,
            } => manifest::apply(&mut app, &name, &package, batch, dry_run),
            ManifestCmd::Add {
                entity,
                org,
                id,
                after,
                patch,
                why,
                accountant,
                w,
            } => manifest::add(
                &mut app,
                &entity,
                &org,
                id,
                after,
                patch,
                &why,
                accountant,
                &w.flags(),
            ),
            ManifestCmd::Journal { journal, w } => {
                manifest::journal(&mut app, &journal.flags(), &w.flags())
            }
            ManifestCmd::Edit { seq, journal, w } => {
                manifest::edit(&mut app, seq, &journal.flags(), &w.flags())
            }
            ManifestCmd::Recode {
                line,
                account,
                description,
                why,
                w,
            } => manifest::recode(&mut app, &line, account, description, &why, &w.flags()),
            ManifestCmd::Void { doc, why, w } => manifest::void(&mut app, &doc, &why, &w.flags()),
            ManifestCmd::Attach { doc, file, why, w } => {
                manifest::attach(&mut app, &doc, &file, &why, &w.flags())
            }
            ManifestCmd::Note { doc, text, why, w } => {
                manifest::note(&mut app, &doc, &text, &why, &w.flags())
            }
            ManifestCmd::Undo { seq, restore, w } => {
                manifest::undo(&mut app, seq, restore, &w.flags())
            }
            ManifestCmd::Annotate {
                seq,
                accountant,
                note,
                w,
            } => manifest::annotate(&mut app, seq, accountant, note, &w.flags()),
            ManifestCmd::Discard { name, w } => manifest::discard(&mut app, name, &w.flags()),
            ManifestCmd::Resolve { entry, how, w } => {
                manifest::resolve(&mut app, &entry, &how, &w.flags())
            }
        },
        Cmd::Walkthrough { action } => match action {
            WalkCmd::New {
                id,
                title,
                audience,
            } => walkthrough::new(&mut app, &id, &title, audience.as_deref()),
            WalkCmd::AddStep {
                id,
                title,
                text,
                why,
                anchors,
                hint,
                hint_title,
                arrow,
                ring,
                spotlight,
                verbs,
                checks,
                viz,
            } => walkthrough::add_step(
                &mut app,
                &id,
                walkthrough::StepArgs {
                    title,
                    text,
                    why,
                    anchors,
                    hint,
                    hint_title,
                    arrow,
                    ring,
                    spotlight,
                    verbs,
                    checks,
                    viz,
                },
            ),
            WalkCmd::Show { id } => walkthrough::show(&mut app, &id),
            WalkCmd::List => walkthrough::list(&mut app),
            WalkCmd::Validate { id } => walkthrough::validate(&mut app, &id),
            WalkCmd::Render {
                id,
                out,
                format,
                sizes,
            } => walkthrough::render(&mut app, &id, &out, &format, &sizes),
        },
        Cmd::Classify { action } => match action {
            ClassifyCmd::Suggest {
                filter,
                engine,
                dry_run,
            } => classify::suggest(&mut app, &filter, &engine, dry_run),
            ClassifyCmd::Rules {
                action: RulesCmd::Check,
            } => classify::rules_check(&mut app),
            ClassifyCmd::Export { csv, filter } => classify::export(&mut app, &filter, csv),
            ClassifyCmd::Set { words, agent } => {
                classify::agent_write(&mut app, classify::AgentOp::Set { words }, &agent.flags())
            }
            ClassifyCmd::Note {
                target,
                text,
                agent,
            } => classify::agent_write(
                &mut app,
                classify::AgentOp::Note { target, text },
                &agent.flags(),
            ),
            ClassifyCmd::Group {
                action:
                    GroupCmd::Add {
                        group,
                        anchors,
                        role,
                        agent,
                    },
            } => classify::agent_write(
                &mut app,
                classify::AgentOp::GroupAdd {
                    group,
                    anchors,
                    role,
                },
                &agent.flags(),
            ),
            ClassifyCmd::Group {
                action:
                    GroupCmd::Create {
                        group_kind,
                        title,
                        agent,
                    },
            } => classify::agent_write(
                &mut app,
                classify::AgentOp::GroupCreate { group_kind, title },
                &agent.flags(),
            ),
            ClassifyCmd::Group {
                action:
                    GroupCmd::Remove {
                        group,
                        anchors,
                        agent,
                    },
            } => classify::agent_write(
                &mut app,
                classify::AgentOp::GroupRemove { group, anchors },
                &agent.flags(),
            ),
            ClassifyCmd::Group {
                action:
                    GroupCmd::Status {
                        group,
                        status,
                        agent,
                    },
            } => classify::agent_write(
                &mut app,
                classify::AgentOp::GroupStatus { group, status },
                &agent.flags(),
            ),
            ClassifyCmd::Undo {
                anchor,
                batch,
                events,
                agent,
            } => classify::agent_write(
                &mut app,
                classify::AgentOp::Undo {
                    batch,
                    anchor,
                    events,
                },
                &agent.flags(),
            ),
            ClassifyCmd::Config {
                action:
                    ClassifyConfigCmd::AddProject {
                        id,
                        name,
                        aliases,
                        dry_run,
                        confirm,
                        agent,
                    },
            } => classify::agent_write(
                &mut app,
                classify::AgentOp::AddProject {
                    id,
                    name,
                    aliases,
                    dry_run,
                    confirm,
                },
                &agent.flags(),
            ),
            ClassifyCmd::Queue { filter, sort } => {
                classify::queue(&mut app, &filter, sort.as_deref())
            }
        },
    }
}

/// `ownpurse tui [args…]` runs the `ownpurse-tui` binary installed next to this one (else on PATH), passing every
/// argument after `tui` through. The TUI is a separate binary so it can never link a write path (invariant V1).
fn launch_tui(args: &[String]) -> Res<()> {
    let rest: Vec<&String> = args
        .iter()
        .skip_while(|a| a.as_str() != "tui")
        .skip(1)
        .collect();
    let sibling = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("ownpurse-tui")))
        .filter(|p| p.exists());
    let program = sibling.unwrap_or_else(|| std::path::PathBuf::from("ownpurse-tui"));
    match std::process::Command::new(&program).args(&rest).status() {
        Ok(st) if st.success() => Ok(()),
        Ok(st) => std::process::exit(st.code().unwrap_or(1)),
        Err(_) => Err(
            CliError::usage("The TUI binary `ownpurse-tui` was not found.")
                .hint("Install both with rust/scripts/install.sh."),
        ),
    }
}

fn dispatch(args: &[String]) -> Res<()> {
    let asks_help = args
        .iter()
        .any(|a| matches!(a.as_str(), "--help" | "-h" | "--version" | "-V"));
    if command_word(args) == Some("tui") {
        return launch_tui(args);
    }
    match command_word(args) {
        None if !asks_help => return Err(CliError::not_available("status")),
        Some(w) if NOT_AVAILABLE.contains(&w) => return Err(CliError::not_available(w)),
        _ => {}
    }
    if args.iter().any(|a| a == "--fresh") {
        let w = command_word(args).unwrap_or("");
        return Err(CliError::not_available(&format!("{w} --fresh")));
    }
    let argv = std::iter::once("ownpurse".to_string()).chain(args.iter().cloned());
    let cli = match Cli::try_parse_from(argv) {
        Ok(c) => c,
        Err(e) => {
            use clap::error::ErrorKind;
            if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                let _ = e.print();
                return Ok(());
            }
            let text = e.to_string();
            let first = text
                .lines()
                .next()
                .unwrap_or("")
                .trim_start_matches("error: ")
                .to_string();
            return Err(CliError::usage(first).hint("Run `ownpurse --help`."));
        }
    };
    run(cli)
}

#[derive(Subcommand, Debug)]
enum ReportCmd {
    /// ⇅ Fetch a report into the record (TrialBalance, BalanceSheet, ProfitAndLoss, BankSummary)
    Fetch {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// trialbalance | balancesheet | pl | banksummary
        kind: String,
        /// as-of date for trialbalance / balancesheet (default: the last fiscal year-end)
        #[arg(long)]
        date: Option<String>,
        /// period start for pl / banksummary (default: that fiscal year's start)
        #[arg(long)]
        from: Option<String>,
        /// period end for pl / banksummary (default: the last fiscal year-end)
        #[arg(long)]
        to: Option<String>,
        /// cash basis (paymentsOnly)
        #[arg(long)]
        cash: bool,
    },
    /// The reports the record holds (zero calls)
    List {
        /// org alias, tenant id, `group:<name>`, comma list, or `all`
        org: String,
        /// only this report (trialbalance, balancesheet, pl, banksummary)
        kind: Option<String>,
    },
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let as_json = args.iter().any(|a| a == "--json");
    let code = match dispatch(&args) {
        Ok(()) => 0,
        Err(e) => {
            e.emit(as_json);
            e.exit
        }
    };
    std::process::exit(code);
}
