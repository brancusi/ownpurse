//! The chat-approval import (design §2 "Decision 3"): find, in the user's OWN Claude Code session transcript
//! (JSONL, one entry per line), a message the user TYPED that quotes the exact package hash.
//!
//! Accepted only when every one of these holds for the entry:
//! - `type == "user"`, `message.role == "user"`, `userType == "external"`;
//! - `origin.kind == "human"` (typed at the prompt; task notifications, tool results, cross-session or relayed
//!   agent messages carry another origin, or none, and are refused);
//! - not `isMeta`, not `isCompactSummary`, not `isSidechain` (a subagent transcript's "user" is its parent agent);
//! - `sessionId` equals the given session id, and the file's stem is that id;
//! - the text is a plain string (or only text blocks), and contains the full 64-hex hash.
//!
//! Everything else in the file is ignored. A file under a `subagents/` folder is refused outright.

use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct ChatApproval {
    pub session: String,
    pub uuid: String,
    pub at: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportError {
    /// The file can't be read, or isn't this session's own transcript.
    Source(String),
    /// No typed user message quotes the hash (with the near misses, for the message).
    NotFound { refused: Vec<(String, String)> },
}

fn text_of(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => {
            let mut out = vec![];
            for b in blocks {
                if b.get("type").and_then(Value::as_str) != Some("text") {
                    return None;
                }
                out.push(b.get("text").and_then(Value::as_str)?.to_string());
            }
            Some(out.join("\n"))
        }
        _ => None,
    }
}

/// Why an entry that quotes the hash is NOT a typed user message (None when it is one).
fn refusal(e: &Value, session: &str) -> Option<String> {
    let s = |k: &str| e.get(k).and_then(Value::as_str);
    let t = |k: &str| e.get(k).and_then(Value::as_bool).unwrap_or(false);
    if s("type") != Some("user") {
        return Some(format!("type {}", s("type").unwrap_or("?")));
    }
    if e.pointer("/message/role").and_then(Value::as_str) != Some("user") {
        return Some("not a user-role message".into());
    }
    if s("userType") != Some("external") {
        return Some("not an external user".into());
    }
    if t("isSidechain") {
        return Some("a subagent (sidechain) message".into());
    }
    if t("isMeta") || t("isCompactSummary") {
        return Some("a meta or summary entry".into());
    }
    match e.pointer("/origin/kind").and_then(Value::as_str) {
        Some("human") => {}
        Some(k) => return Some(format!("origin {k}, not typed by the user")),
        None => return Some("no origin: not provably typed by the user".into()),
    }
    if s("sessionId") != Some(session) {
        return Some("another session".into());
    }
    None
}

/// Find the approval for `hash` in `path` (the session `session`'s transcript).
pub fn find(path: &Path, session: &str, hash: &str) -> Result<ChatApproval, ImportError> {
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ImportError::Source(
            "the hash must be the full 64-hex package hash".into(),
        ));
    }
    if path.components().any(|c| c.as_os_str() == "subagents") {
        return Err(ImportError::Source(
            "a subagent transcript is never the user's own session".into(),
        ));
    }
    if path.file_stem().and_then(|s| s.to_str()) != Some(session) {
        return Err(ImportError::Source(format!(
            "the transcript file is not session {session}'s ({})",
            path.display()
        )));
    }
    let raw = std::fs::read_to_string(path)
        .map_err(|e| ImportError::Source(format!("{}: {e}", path.display())))?;
    let mut refused = vec![];
    let mut found = None;
    for line in raw.lines() {
        if !line.contains(hash) {
            continue;
        }
        let Ok(e) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(text) = e.pointer("/message/content").and_then(text_of) else {
            refused.push((
                e.get("uuid").and_then(Value::as_str).unwrap_or("?").into(),
                "not plain text (a tool result or attachment)".into(),
            ));
            continue;
        };
        if !text.contains(hash) {
            continue;
        }
        let uuid = e
            .get("uuid")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        match refusal(&e, session) {
            Some(why) => refused.push((uuid, why)),
            None => {
                found = Some(ChatApproval {
                    session: session.into(),
                    uuid,
                    at: e
                        .get("timestamp")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .into(),
                    text,
                });
            }
        }
    }
    found.ok_or(ImportError::NotFound { refused })
}

// ---------------------------------------------------------------- recording and checking approvals

use super::{
    AllOps, ApprovalRecord, ApprovalScope, ApprovalStore, Package, Via, WriteError, exit, seal,
};

/// Environment variables that show an agent session (names only; values are never read or printed).
pub fn agent_markers<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut out: Vec<String> = names
        .into_iter()
        .filter(|k| {
            k.starts_with("CLAUDE")
                || k.starts_with("CODEX")
                || k.starts_with("GEMINI_CLI")
                || k.starts_with("CURSOR_AGENT")
                || *k == "OWNPURSE_AGENT"
                || *k == "OWNPURSE_BRIDGE"
                || *k == "AI_AGENT"
        })
        .map(str::to_string)
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The response approves: an approval word, and no negation.
pub fn affirmative(text: &str) -> Result<(), String> {
    let t = text.to_lowercase().replace('’', "'");
    for neg in [
        "not approve",
        "don't approve",
        "dont approve",
        "do not approve",
        "never approve",
        "unapprove",
        "disapprove",
        "not approved",
        "reject",
        "revoke",
    ] {
        if t.contains(neg) {
            return Err(format!(
                "the response contains {neg:?}: it is not an approval"
            ));
        }
    }
    if ["approve", "approval", "go ahead", "authorize", "authorise"]
        .iter()
        .any(|w| t.contains(w))
    {
        Ok(())
    } else {
        Err("the response doesn't say it approves (e.g. \"approved\")".into())
    }
}

/// The ops an approval covers: `all` (also when none are listed), or listed ids of the package.
pub fn scope_of(p: &Package, scope: &[String]) -> Result<ApprovalScope, WriteError> {
    if scope.is_empty() || scope.iter().any(|s| s == "all") {
        return Ok(ApprovalScope::All(AllOps::All));
    }
    for s in scope {
        if !p.ops.iter().any(|o| &o.id == s) {
            return Err(WriteError::new(
                "usage",
                exit::USAGE,
                format!("scope: {s} is not an op of this package"),
            ));
        }
    }
    Ok(ApprovalScope::Ops(scope.to_vec()))
}

/// What a TTY approval needs (the CLI gathers these; this decides).
pub struct TtyApproval<'a> {
    pub package: &'a Package,
    /// The owner's response, verbatim.
    pub verbatim: &'a str,
    /// The hash prefix the owner typed at the confirmation prompt.
    pub typed: &'a str,
    pub i_am_owner: bool,
    /// stdin AND stdout are terminals.
    pub tty: bool,
    /// The environment's variable NAMES.
    pub env_names: Vec<String>,
    pub scope: Vec<String>,
    pub now: String,
}

/// Refuse an agent session (an agent marker in the environment, or no TTY).
pub fn refuse_agent(tty: bool, env_names: &[String]) -> Result<(), WriteError> {
    let markers = agent_markers(env_names.iter().map(String::as_str));
    if !markers.is_empty() {
        return Err(WriteError::new(
            "agent_session",
            exit::NOT_APPROVED,
            format!(
                "refused: this is an agent session ({}); only the owner records an approval, at his own terminal",
                markers.join(", ")
            ),
        ));
    }
    if !tty {
        return Err(WriteError::new(
            "agent_session",
            exit::NOT_APPROVED,
            "refused: approve-record runs only interactively at a terminal (stdin and stdout a TTY)",
        ));
    }
    Ok(())
}

/// `write approve-record`: refuses agent sessions, a non-TTY, a response that doesn't quote the hash or
/// doesn't approve, and a mistyped confirmation; else the record to append.
pub fn record_tty(a: &TtyApproval<'_>) -> Result<ApprovalRecord, WriteError> {
    refuse_agent(a.tty, &a.env_names)?;
    if !a.i_am_owner {
        return Err(WriteError::new(
            "usage",
            exit::USAGE,
            "approve-record needs --i-am-the-owner",
        ));
    }
    let hash = seal::verify(a.package)?;
    if !a.verbatim.contains(&hash[..12]) {
        return Err(WriteError::new(
            "not_approved",
            exit::NOT_APPROVED,
            format!(
                "the response must quote the package hash (at least its first 12 characters, {})",
                &hash[..12]
            ),
        ));
    }
    affirmative(a.verbatim).map_err(|e| WriteError::new("not_approved", exit::NOT_APPROVED, e))?;
    let typed = a.typed.trim();
    if typed.len() < 12 || !hash.starts_with(typed) {
        return Err(WriteError::new(
            "hash_mismatch",
            exit::NOT_APPROVED,
            "the typed confirmation is not the package hash (type at least its first 12 characters)",
        ));
    }
    Ok(ApprovalRecord {
        hash,
        verbatim: a.verbatim.into(),
        via: Via::Tty,
        where_: "tty".into(),
        at: a.now.clone(),
        scope: scope_of(a.package, &a.scope)?,
        recorded_at: a.now.clone(),
    })
}

/// `write approve-import`: the owner's typed chat message quoting the FULL hash, from his own session's
/// transcript (see [`find`]).
pub fn record_chat(
    package: &Package,
    transcript: &Path,
    session: &str,
    scope: &[String],
    now: &str,
) -> Result<ApprovalRecord, WriteError> {
    let hash = seal::verify(package)?;
    let found = find(transcript, session, &hash).map_err(|e| match e {
        ImportError::Source(m) => WriteError::new("usage", exit::USAGE, m),
        ImportError::NotFound { refused } => {
            let mut e = WriteError::new(
                "not_approved",
                exit::NOT_APPROVED,
                format!(
                    "no message the user typed in session {session} quotes the full hash {hash}"
                ),
            );
            for (u, w) in refused {
                e = e.help(format!("refused {u}: {w}"));
            }
            e
        }
    })?;
    affirmative(&found.text).map_err(|e| WriteError::new("not_approved", exit::NOT_APPROVED, e))?;
    Ok(ApprovalRecord {
        hash,
        verbatim: found.text,
        via: Via::Chat,
        where_: format!(
            "transcript:{}#session={}&message={}",
            transcript.display(),
            found.session,
            found.uuid
        ),
        at: found.at,
        scope: scope_of(package, scope)?,
        recorded_at: now.into(),
    })
}

/// `write approve-check`: the approvals for this package's recomputed hash (refuses when none).
pub fn check(
    store: &dyn ApprovalStore,
    package: &Package,
) -> Result<Vec<ApprovalRecord>, WriteError> {
    let hash = seal::verify(package)?;
    let found = store.approvals(&hash)?;
    if found.is_empty() {
        return Err(WriteError::new(
            "not_approved",
            exit::NOT_APPROVED,
            format!("no approval is recorded for {}", &hash[..12]),
        )
        .help("the owner records one: `ownpurse write approve-record` at his terminal, or a chat approval imported with `write approve-import`"));
    }
    Ok(found)
}

/// Whether the approvals cover an op.
pub fn covers(approvals: &[ApprovalRecord], op: &str) -> bool {
    approvals.iter().any(|a| a.scope.covers(op))
}
