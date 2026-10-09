//! Hit targets (spec §7): registered per draw as `(Rect, Target)`; a click dispatches through the same
//! actions as keys (a footer key click runs the key's action). The last registered target on top wins.

use crate::app::Tab;
use ratatui::layout::{Position, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Run a keymap action (footer keys, header scope, breadcrumb).
    Action(&'static str),
    Tab(Tab),
    /// A row of the current view's table (index into its visible rows).
    Row(usize),
    /// A cell of the current view's table: (visible row, column) — Overview parity cells (M1).
    Cell(usize, usize),
    PaletteRow(usize),
    PickerRow(usize),
    /// A help row: close help, then run the binding's action.
    HelpRow(&'static str),
    /// A confirm-band button (0 = Cancel / Close).
    Button(usize),
    /// A glossary term in the narration panel (index into the shown body's terms).
    Term(usize),
    /// A step on the walkthrough progress strip.
    WalkStep(usize),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hits {
    pub targets: Vec<(Rect, Target)>,
    /// The open overlay's box: a click outside it closes the overlay and does nothing underneath.
    pub overlay: Option<Rect>,
    /// The scrollable content area (wheel events).
    pub content: Option<Rect>,
    /// Where this frame drew each thing an overlay can point at (`row:…`, `cell:…`, `diff:…`, `chip:…`,
    /// `panel:…`, `viz:…`).
    pub anchors: crate::anchors::Anchors,
    /// The overlay layers as placed over this frame (none drawn: None).
    pub plan: Option<caretline_layers::Plan>,
    /// Where overlay boxes may go on this frame (below the tabs, above the narration panel and bands).
    pub layer_area: Option<Rect>,
}

impl Hits {
    pub fn add(&mut self, rect: Rect, target: Target) {
        if rect.width > 0 && rect.height > 0 {
            self.targets.push((rect, target));
        }
    }

    pub fn at(&self, x: u16, y: u16) -> Option<Target> {
        let p = Position { x, y };
        self.targets
            .iter()
            .rev()
            .find(|(r, _)| r.contains(p))
            .map(|(_, t)| *t)
    }

    /// Where a target is (tests and key scripts click "the footer's g").
    pub fn find(&self, target: Target) -> Option<Rect> {
        self.targets
            .iter()
            .find(|(_, t)| *t == target)
            .map(|(r, _)| *r)
    }
}
