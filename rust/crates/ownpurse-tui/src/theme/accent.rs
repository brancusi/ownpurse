//! Scope accents (theme spec v2 §2): the families, their assignment to orgs (`tui.accents`), and the one
//! derivation from the scope. Only the scope decides the accent, never the screen.

use std::collections::BTreeMap;

/// The accent families every theme defines, in assignment order after `harbor` (cross-sectional).
pub const FAMILIES: [&str; 6] = ["harbor", "ember", "iris", "rose", "sea", "graphite"];
/// The order unlisted orgs take accents in (so no two orgs match unless chosen).
pub const ORG_ORDER: [&str; 4] = ["ember", "iris", "rose", "sea"];
/// The cross-sectional default (all, groups, pairs, Compare).
pub const CROSS: &str = "harbor";

/// `tui.accents`: org alias (or `cross`) → family. Invalid names are refused when the config loads.
pub type Assignment = BTreeMap<String, String>;

pub fn is_family(name: &str) -> bool {
    FAMILIES.contains(&name)
}

/// Each org's accent: configured ones first, then the unlisted orgs alphabetically, each taking the next
/// family in `ORG_ORDER` that no org uses yet (cycling when all four are taken). Deterministic: the same
/// config gives the same assignment, so nothing has to be stored.
pub fn assign(aliases: &[String], configured: &Assignment) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for a in aliases {
        if let Some(f) = configured.get(a).filter(|f| is_family(f)) {
            out.insert(a.clone(), f.clone());
        }
    }
    let mut rest: Vec<&String> = aliases.iter().filter(|a| !out.contains_key(*a)).collect();
    rest.sort();
    let mut n = 0;
    for a in rest {
        let used: Vec<&String> = out.values().collect();
        let f = ORG_ORDER
            .iter()
            .find(|f| !used.iter().any(|u| u == f))
            .copied()
            .unwrap_or_else(|| {
                let f = ORG_ORDER[n % ORG_ORDER.len()];
                n += 1;
                f
            });
        out.insert(a.clone(), f.to_string());
    }
    out
}

/// The cross-sectional family (`tui.accents.cross`, default harbor).
pub fn cross(configured: &Assignment) -> String {
    configured
        .get("cross")
        .filter(|f| is_family(f))
        .cloned()
        .unwrap_or_else(|| CROSS.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn configured_first_then_alphabetical_unused() {
        let mut c = Assignment::new();
        c.insert("roast".into(), "iris".into());
        let got = assign(&a(&["roast", "hold", "design", "lab"]), &c);
        assert_eq!(got["roast"], "iris");
        // design, hold, lab alphabetically take the unused ember, rose, sea
        assert_eq!(got["design"], "ember");
        assert_eq!(got["hold"], "rose");
        assert_eq!(got["lab"], "sea");
        assert_eq!(cross(&c), "harbor");
        c.insert("cross".into(), "graphite".into());
        assert_eq!(cross(&c), "graphite");
    }

    #[test]
    fn more_orgs_than_families_cycle() {
        let got = assign(&a(&["a", "b", "c", "d", "e", "f"]), &Assignment::new());
        assert_eq!(got["a"], "ember");
        assert_eq!(got["d"], "sea");
        assert_eq!(got["e"], "ember");
        assert_eq!(got["f"], "iris");
    }
}
