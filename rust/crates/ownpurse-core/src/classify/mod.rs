//! Classification: local labels on Xero lines (project, bearer, treatment, should-be account), rules that
//! suggest them, notes and groups. Nothing here touches Xero or the record: lines are READ from the record,
//! and the only things written are the classification store (`db/classify.sqlite`, append-only, hash-chained)
//! and the classification config (`classify.toml`, atomic replace with numbered backups). Invariants V14–V20.
//!
//! - [`config`]: `classify.toml` (catalog, rules, queue, groups, tree) parsed with spans, validated, edited.
//! - [`store`]: the event log (the only write path) and the materialised reads rebuilt from it.
//! - [`anchor`]: classifiable lines from the record, their anchors and fingerprints (V19).
//! - [`rules`]: the rules engine (suggestions, would-match, evidence).
//! - [`view`]: the queue model the CLI and the TUI share ([`view::Workspace`]).
//!
//! Confidence is an exact decimal in `0..=1` (no `f64`, as everywhere in core).

pub mod anchor;
pub mod config;
pub mod groups;
pub mod request;
pub mod rules;
pub mod store;
pub mod synth;
pub mod view;

pub use anchor::{Anchor, DocType, Fingerprint, Line};
pub use rust_decimal::Decimal as Confidence;
pub use view::{
    BulkPreview, Cell, ClassifyPaths, Filter, GroupMembership, LabelState, LabelView, LineDetail,
    NoteView, QueueCounts, QueueRow, RuleStat, Since, Sort, Status, SuggestSummary,
    SuggestionState, Triple, Workspace, WriteCtx, WriteOutcome,
};

/// A structured classification error. `code` is stable (CLI `error:` field and exit code):
/// `config` (1), `conflict` (1, classify.toml changed on disk), `store` (1), `integrity` (1), `record` (1),
/// `not_found` (2), `filter` (2), `jev_not_enabled` (2), `too_many` (2), `user_busy` (4), `view_changed` (4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifyError {
    pub code: &'static str,
    pub message: String,
    pub hints: Vec<String>,
}

impl ClassifyError {
    pub fn new(code: &'static str, message: impl Into<String>) -> ClassifyError {
        ClassifyError {
            code,
            message: message.into(),
            hints: vec![],
        }
    }
    pub fn hint(mut self, h: impl Into<String>) -> ClassifyError {
        self.hints.push(h.into());
        self
    }
    /// Exit code for the CLI.
    pub fn exit_code(&self) -> i32 {
        match self.code {
            "not_found" | "filter" | "jev_not_enabled" | "too_many" | "usage" | "bad_caption" => 2,
            "user_busy" | "view_changed" => 4,
            _ => 1,
        }
    }
}

impl std::fmt::Display for ClassifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ClassifyError {}

impl From<rusqlite::Error> for ClassifyError {
    fn from(e: rusqlite::Error) -> Self {
        ClassifyError::new("store", format!("classify store: {e}"))
    }
}

impl From<crate::record::RecordError> for ClassifyError {
    fn from(e: crate::record::RecordError) -> Self {
        ClassifyError::new("record", e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ClassifyError>;

/// Who wrote an event: `human`, `agent:<name>`, `rule:<id>` (suggest runs) or `jev:<tree-hash>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Actor(String);

impl Actor {
    pub fn human() -> Actor {
        Actor("human".into())
    }
    pub fn agent(name: &str) -> Actor {
        Actor(format!("agent:{name}"))
    }
    pub fn rule(id: &str) -> Actor {
        Actor(format!("rule:{id}"))
    }
    /// Parse `human`, `agent:<name>`, `rule:<id>`, `jev:<x>`; anything else is refused.
    pub fn parse(s: &str) -> Result<Actor> {
        let ok = s == "human"
            || ["agent:", "rule:", "jev:"]
                .iter()
                .any(|p| s.strip_prefix(p).is_some_and(|r| !r.is_empty()));
        if ok {
            Ok(Actor(s.to_string()))
        } else {
            Err(ClassifyError::new(
                "usage",
                format!("actor '{s}': expected human, agent:<name>, rule:<id> or jev:<x>"),
            ))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn is_agent(&self) -> bool {
        self.0.starts_with("agent:")
    }
    pub fn is_human(&self) -> bool {
        self.0 == "human"
    }
}

impl std::fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One send (a key press, a CLI call, an agent send): every event it writes carries the same batch id, so
/// "undo the latest agent batch" undoes exactly one send.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BatchId(pub String);

impl BatchId {
    pub fn new(s: impl Into<String>) -> BatchId {
        BatchId(s.into())
    }
}

/// Where a label's or suggestion's value came from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Source {
    Human,
    Agent(String),
    Rule(String),
    Jev(String),
}

impl Source {
    pub fn parse(s: &str) -> Option<Source> {
        if s == "human" {
            return Some(Source::Human);
        }
        let (k, v) = s.split_once(':')?;
        let v = v.to_string();
        match k {
            "agent" => Some(Source::Agent(v)),
            "rule" => Some(Source::Rule(v)),
            "jev" => Some(Source::Jev(v)),
            _ => None,
        }
    }
    pub fn glyph(&self) -> Glyph {
        match self {
            Source::Human => Glyph::Human,
            Source::Agent(_) => Glyph::Agent,
            Source::Rule(_) => Glyph::Rule,
            Source::Jev(_) => Glyph::Jev,
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Human => f.write_str("human"),
            Source::Agent(n) => write!(f, "agent:{n}"),
            Source::Rule(n) => write!(f, "rule:{n}"),
            Source::Jev(n) => write!(f, "jev:{n}"),
        }
    }
}

/// The queue's source glyph: `R` rule, `J` Jev, `H` human, `A` agent, `·` none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Glyph {
    Rule,
    Jev,
    Human,
    Agent,
    None,
}

impl Glyph {
    pub fn char(self) -> char {
        match self {
            Glyph::Rule => 'R',
            Glyph::Jev => 'J',
            Glyph::Human => 'H',
            Glyph::Agent => 'A',
            Glyph::None => '·',
        }
    }
}

/// Why a rule fired: which condition matched which text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    /// The condition key, e.g. `payee`, `description_has_alias`.
    pub condition: String,
    /// The text (or value) that matched, e.g. `Harvest` or `-412.37`.
    pub matched: String,
}
