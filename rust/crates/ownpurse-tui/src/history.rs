//! Navigation history (v0.10.2): browser-style back/forward over view identities, and the agent's
//! caption, which lives exactly as long as one view identity.
//!
//! - A navigation is any change of tab, scope, as-of, basis, drill path or filter (cursor moves within a
//!   list are not; the selection is captured when you leave). The view you left is pushed on `back` and
//!   `forward` is cleared, as in a browser. Bounded to `HISTORY_MAX` entries (oldest dropped).
//! - Alt+← / Alt+→ (and the agent's `back` / `forward`) move through it; Esc stays "up one level".

use crate::agent::ViewSnap;
use crate::app::{App, Scope, Tab};

/// Entries kept on each side.
pub const HISTORY_MAX: usize = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct History {
    /// Older views, the most recent last.
    pub back: Vec<ViewSnap>,
    /// Views gone back from, the nearest last.
    pub forward: Vec<ViewSnap>,
}

/// The agent's caption: a one-line description of the current view.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Caption {
    pub text: String,
    /// `agent`, or the client name the request carried.
    pub by: String,
    /// The view revision the caption was set at.
    pub rev: u64,
    /// Set in the update that sets it (a send that changes the view and captions it in one step).
    #[serde(skip)]
    pub fresh: bool,
}

/// What makes two views "the same place": the history and the caption's lifetime key on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    tab: Tab,
    scope: Scope,
    as_of: String,
    full_history: bool,
    filter: String,
    drill: Vec<String>,
}

// The caption rules are shared with the classification writes (core::bridge), so both refuse the same text.
pub use ownpurse_core::bridge::{CAPTION_MAX, caption_error};

impl App {
    /// The current view identity.
    pub fn identity(&self) -> Identity {
        let mut drill = Vec::new();
        match self.ui.tab {
            Tab::Ledger if self.cmp_active() => {
                if let Some(c) = &self.ui.cmp {
                    drill.push(format!("compare:{:?}:{}", c.orgs, c.level));
                    if c.level == 2 {
                        drill.push(c.l2_row.clone().unwrap_or_default());
                    }
                }
            }
            Tab::Ledger => {
                let n = self.nav();
                drill.push(format!("level:{}", self.ledger_level()));
                if self.ledger_level() >= 2 {
                    drill.push(n.account.clone().unwrap_or_default());
                }
                if self.ledger_level() >= 3 {
                    drill.push(format!("{:?}", n.doc));
                }
                if let Some(v) = &self.ui.via {
                    drill.push(format!("via:{}", v.pair));
                }
            }
            Tab::Intercompany => {
                let n = self.ic_nav();
                drill.push(format!("level:{}", self.ic_level()));
                if self.ic_level() >= 2 {
                    drill.push(n.pair.clone().unwrap_or_default());
                }
                if self.ic_level() >= 3 {
                    drill.push(format!("{:?}", n.doc));
                }
            }
            Tab::Activity if self.ui.activity.grouped => drill.push("grouped".into()),
            Tab::Classify => {
                drill.push(format!("mode:{}", self.ui.classify.mode.word()));
                drill.push(format!("filter:{}", self.ui.classify.filter));
            }
            Tab::Manifest => {
                drill.push(format!(
                    "manifest:{}",
                    self.ui.manifest.clone().unwrap_or_default()
                ));
                drill.push(format!("mode:{}", self.mf_mode().word()));
                if let Some(d) = &self.ui.manifest_tab.diff {
                    drill.push(format!("entry:{}", d.seq));
                }
            }
            _ => {}
        }
        Identity {
            tab: self.ui.tab,
            scope: self.ui.scope.clone(),
            as_of: self.ui.as_of.clone(),
            full_history: self.ui.full_history,
            filter: self.ui.filter.clone(),
            drill,
        }
    }

    /// After a navigation (identity changed, not by a history move): remember where we were.
    pub(crate) fn history_push(&mut self, left: ViewSnap) {
        let h = &mut self.ui.history;
        h.back.push(left);
        if h.back.len() > HISTORY_MAX {
            h.back.remove(0);
        }
        h.forward.clear();
    }

    /// Alt+← / agent `back`: the previous view (None at the start of the history).
    pub(crate) fn history_back(&mut self) -> bool {
        let Some(prev) = self.ui.history.back.pop() else {
            return false;
        };
        let here = ViewSnap::of(&self.ui);
        self.ui.history.forward.push(here);
        if self.ui.history.forward.len() > HISTORY_MAX {
            self.ui.history.forward.remove(0);
        }
        prev.restore_view(&mut self.ui);
        true
    }

    /// Alt+→ / agent `forward`.
    pub(crate) fn history_forward(&mut self) -> bool {
        let Some(next) = self.ui.history.forward.pop() else {
            return false;
        };
        let here = ViewSnap::of(&self.ui);
        self.ui.history.back.push(here);
        if self.ui.history.back.len() > HISTORY_MAX {
            self.ui.history.back.remove(0);
        }
        next.restore_view(&mut self.ui);
        true
    }

    /// A one-line summary of the current view (history notices: "agent: back → design › Trial balance").
    pub fn view_summary(&self) -> String {
        let mut c = crate::views::crumbs(self);
        if !self.ui.filter.is_empty() {
            c.push(format!("filter “{}”", self.ui.filter));
        }
        c.join(" › ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captions_reject_controls_escapes_and_invisibles() {
        assert!(caption_error("Green-bean card spend, 2025 · 3 lines · 1,990.90").is_none());
        assert!(caption_error("東京 · ünï · 🙂").is_none());
        for bad in [
            "a\nb",
            "a\rb",
            "tab\there",
            "\x1b[31mred",
            "\x1b]0;title\x07",
            "\x1bP1$r\x1b\\",
            "\u{9b}31m",
            "del\x7f",
            "zero\u{200b}width",
            "bidi\u{202e}evil",
            "\u{2066}iso\u{2069}",
            "bom\u{feff}",
        ] {
            assert!(caption_error(bad).is_some(), "{bad:?}");
        }
        assert!(caption_error(&"x".repeat(120)).is_none());
        assert!(caption_error(&"x".repeat(121)).is_some());
        assert!(caption_error("  ").is_some());
    }
}
