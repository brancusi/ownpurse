//! Anchors: where this frame drew each thing an overlay can point at (walkthroughs design §1).
//!
//! Every draw fills one [`Anchors`] (it lives in the frame's [`Hits`](crate::hit::Hits)), keyed by a
//! stable name `kind:key`, never a screen position:
//!
//! | name | what |
//! |---|---|
//! | `row:<anchor>` | a table row by its stable anchor (`code=6140`, `log_id=12`, `doc=mj-303`, `system_item=…`) |
//! | `cell:<row anchor>/<column key>` | one cell of such a row |
//! | `diff:<json path>` | a field or line of the generic diff view (`diff:/JournalLines/n1/AccountCode`) |
//! | `chip:<name>` | a header chip, a tab, the manifest stage strip |
//! | `panel:<name>` | a panel (the narration panel, the apply panel, …) |
//! | `viz:<element id>` | a part of a visualisation panel ([`VizPanel`], or [`Anchors::viz`] while drawing) |
//!
//! caretline-layers resolves `Anchor::Host { kind, key }` through the map ([`Anchors::map`]); a name
//! `kind:key` is that anchor's text form ([`host_anchor`]). Rebuilt every frame; nothing here is stored.

use caretline_layers::{Anchor, AnchorKey, AnchorMap, Off, Resolved};
use ratatui::layout::Rect;
use serde_json::Value;

/// A visualisation panel's anchors: each element's id (without the `viz:` prefix) and where it was drawn.
/// A panel that keeps its element rects while drawing hands them over with [`Anchors::register`].
pub trait VizPanel {
    fn anchors(&self) -> Vec<(String, Rect)>;
}

/// The anchors of one frame. Equal frames record equal anchors (the order is the draw order).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Anchors {
    map: AnchorMap,
    names: Vec<String>,
}

fn cl_rect(r: Rect) -> caretline_layers::Rect {
    caretline_layers::Rect::new(r.x, r.y, r.width, r.height)
}

/// `kind:key` → (kind, key). The key may itself hold `:` (`cell:code=1:2/amount`); the kind never does.
pub fn split(name: &str) -> Option<(&str, &str)> {
    let (k, rest) = name.split_once(':')?;
    (!k.is_empty() && !rest.is_empty()).then_some((k, rest))
}

/// The caretline anchor for a name (`row:code=6140` → `Host { kind: "row", key: "code=6140" }`).
pub fn host_anchor(name: &str) -> Option<Anchor> {
    let (kind, key) = split(name)?;
    Some(Anchor::Host {
        kind: kind.into(),
        key: key.into(),
    })
}

/// The text form of a row's stable anchor (the JSON object `ui state` shows): `k=v` pairs joined by
/// `,`, strings unquoted (`{"code": "6140"}` → `code=6140`). A non-object anchor is its JSON.
pub fn row_key(anchor: &Value) -> String {
    match anchor {
        Value::Object(m) => m
            .iter()
            .map(|(k, v)| match v {
                Value::String(s) => format!("{k}={s}"),
                o => format!("{k}={o}"),
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::String(s) => s.clone(),
        o => o.to_string(),
    }
}

/// The short names a row is also known by: each `k=v` pair of its anchor but the org (`account_code` also
/// as `code`), so `row:code=6140` finds the account's row. The first is the row's short name.
pub fn row_aliases(anchor: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Value::Object(m) = anchor {
        if let Some(c) = m.get("account_code").and_then(Value::as_str) {
            out.push(format!("code={c}"));
        }
        for (k, v) in m {
            if k == "org" && m.len() > 1 {
                continue;
            }
            let v = match v {
                Value::String(s) => s.clone(),
                o => o.to_string(),
            };
            let pair = format!("{k}={v}");
            if !out.contains(&pair) {
                out.push(pair);
            }
        }
    }
    out
}

/// A row's short name (`code=6140`, `log_id=12`, `org=roast`; the full anchor when it has no pairs).
pub fn row_short(anchor: &Value) -> String {
    row_aliases(anchor)
        .into_iter()
        .next()
        .unwrap_or_else(|| row_key(anchor))
}

impl Anchors {
    /// Record `name` (`kind:key`) at `rect`; call again to add cells (a wrapped row). Empty rects and
    /// names without a kind are ignored.
    pub fn put(&mut self, name: &str, rect: Rect) {
        let Some((kind, key)) = split(name) else {
            return;
        };
        if rect.width == 0 || rect.height == 0 {
            return;
        }
        if !self.names.iter().any(|n| n == name) {
            self.names.push(name.to_string());
        }
        self.map.put(AnchorKey::host(kind, key), cl_rect(rect));
    }

    /// Record `name` at `rect` without listing it (a short name for something already listed): it
    /// resolves, but `names()` shows the main name only. A name another thing already took stays its.
    pub fn alias(&mut self, name: &str, rect: Rect) {
        let Some((kind, key)) = split(name) else {
            return;
        };
        if rect.width == 0 || rect.height == 0 || self.names.iter().any(|n| n == name) {
            return;
        }
        let k = AnchorKey::host(kind, key);
        if self
            .map
            .get(&k)
            .is_some_and(|r| !r.rects.iter().any(|x| x.y == rect.y))
        {
            return;
        }
        self.map.put(k, cl_rect(rect));
    }

    /// Record that `name`'s target exists but is scrolled out of sight, that way.
    pub fn put_off(&mut self, name: &str, off: Off) {
        let Some((kind, key)) = split(name) else {
            return;
        };
        if !self.names.iter().any(|n| n == name) {
            self.names.push(name.to_string());
        }
        self.map.put_off(AnchorKey::host(kind, key), off);
    }

    pub fn row(&mut self, key: &str, rect: Rect) {
        self.put(&format!("row:{key}"), rect);
    }

    pub fn cell(&mut self, row: &str, col: &str, rect: Rect) {
        self.put(&format!("cell:{row}/{col}"), rect);
    }

    /// A diff field by its JSON Pointer (`/JournalLines/n1/AccountCode`).
    pub fn diff(&mut self, path: &str, rect: Rect) {
        self.put(&format!("diff:{path}"), rect);
    }

    pub fn chip(&mut self, name: &str, rect: Rect) {
        self.put(&format!("chip:{name}"), rect);
    }

    pub fn panel(&mut self, name: &str, rect: Rect) {
        self.put(&format!("panel:{name}"), rect);
    }

    /// One element of a visualisation (`id` without the `viz:` prefix).
    pub fn viz(&mut self, id: &str, rect: Rect) {
        self.put(&format!("viz:{id}"), rect);
    }

    /// A visualisation panel's element anchors, as `viz:<id>`.
    pub fn register(&mut self, panel: &dyn VizPanel) {
        for (id, rect) in panel.anchors() {
            self.viz(&id, rect);
        }
    }

    /// Where `name` is this frame (`None`: not drawn).
    pub fn get(&self, name: &str) -> Option<&Resolved> {
        let (kind, key) = split(name)?;
        self.map.get(&AnchorKey::host(kind, key))
    }

    /// The bounding rect of `name`'s cells (`None`: not drawn, or off screen).
    pub fn rect(&self, name: &str) -> Option<Rect> {
        let r = self.get(name)?;
        if r.rects.is_empty() {
            return None;
        }
        let b = r.bounds();
        Some(Rect::new(b.x, b.y, b.w, b.h))
    }

    /// Every recorded name, in draw order.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// The resolver caretline-layers' `plan` takes.
    pub fn map(&self) -> &AnchorMap {
        &self.map
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caretline_layers::Resolve;
    use serde_json::json;

    struct Bars;
    impl VizPanel for Bars {
        fn anchors(&self) -> Vec<(String, Rect)> {
            vec![("bridge/start".into(), Rect::new(2, 3, 10, 1))]
        }
    }

    #[test]
    fn names_resolve_as_host_anchors() {
        let mut a = Anchors::default();
        a.row(&row_key(&json!({"code": "6140"})), Rect::new(0, 5, 40, 1));
        a.cell("code=6140", "amount", Rect::new(30, 5, 10, 1));
        a.chip("stage", Rect::new(0, 0, 0, 1)); // empty: ignored
        a.register(&Bars);
        assert_eq!(
            a.names(),
            ["row:code=6140", "cell:code=6140/amount", "viz:bridge/start"]
        );
        let r = a
            .map()
            .resolve(&host_anchor("viz:bridge/start").unwrap())
            .unwrap();
        assert_eq!(r.rects, vec![caretline_layers::Rect::new(2, 3, 10, 1)]);
        assert_eq!(a.rect("row:code=6140"), Some(Rect::new(0, 5, 40, 1)));
        assert!(a.get("chip:stage").is_none());
        assert_eq!(split("cell:a:b/c"), Some(("cell", "a:b/c")));
        assert_eq!(row_key(&json!({"org": "roast", "n": 3})), "org=roast,n=3");
    }
}
