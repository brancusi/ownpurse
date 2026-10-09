//! Link templates: `links` (+ `extra_links`, already appended by
//! `Settings::build`) with `per: app | org | bank_account` expanded over the orgs and their active bank
//! accounts. Placeholders are `{name}` (lowercase and `_`); a template whose name or url has an unknown or
//! empty placeholder is skipped (an org not yet synced has no short code, so its links don't show).
//! `per: document` templates are resolved per document elsewhere and skipped here.

use serde_json::Value;
use std::collections::HashMap;

pub type Values = HashMap<String, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub name: String,
    pub url: String,
    pub description: String,
    /// `app`, `org` or `bank_account`.
    pub per: String,
    /// The org alias for per-org and per-bank-account links.
    pub org: Option<String>,
}

/// Fill `{placeholders}`. Strict: an unknown or empty value is `None`; lenient leaves it in place.
pub fn fill(template: &str, values: &Values, strict: bool) -> Option<String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let name_len = after
            .find(|c: char| !(c.is_ascii_lowercase() || c == '_'))
            .unwrap_or(after.len());
        if name_len > 0 && after[name_len..].starts_with('}') {
            let key = &after[..name_len];
            match values.get(key).filter(|v| !v.is_empty()) {
                Some(v) => out.push_str(v),
                None if strict => return None,
                None => out.push_str(&rest[i..i + name_len + 2]),
            }
            rest = &after[name_len + 1..];
        } else {
            out.push('{');
            rest = after;
        }
    }
    out.push_str(rest);
    Some(out)
}

fn text(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or("").to_string()
}

/// Expand every non-document template. `context` holds `client_id`, `redirect_uri`, `profile`; each org
/// map holds `org`, `org_name`, `short_code`, `tenant_id`; each bank map adds `account_id`, `account_name`.
pub fn expand(links: &[Value], context: &Values, orgs: &[Values], banks: &[Values]) -> Vec<Link> {
    let app = [Values::new()];
    let mut out = Vec::new();
    for link in links {
        let per = link.get("per").and_then(Value::as_str).unwrap_or("app");
        let items: &[Values] = match per {
            "document" => continue,
            "org" => orgs,
            "bank_account" => banks,
            _ => &app,
        };
        for item in items {
            let mut values = context.clone();
            values.extend(item.iter().map(|(k, v)| (k.clone(), v.clone())));
            let (Some(name), Some(url)) = (
                fill(&text(link, "name"), &values, true),
                fill(&text(link, "url"), &values, true),
            ) else {
                continue;
            };
            out.push(Link {
                name,
                url,
                description: fill(&text(link, "description"), &values, false).unwrap_or_default(),
                per: per.to_string(),
                org: item.get("org").cloned(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn vals(kv: &[(&str, &str)]) -> Values {
        kv.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn fill_strict_and_lenient() {
        let v = vals(&[("org_name", "Tallowmere"), ("empty", "")]);
        assert_eq!(
            fill("Open {org_name}", &v, true).as_deref(),
            Some("Open Tallowmere")
        );
        assert_eq!(fill("{missing}", &v, true), None);
        assert_eq!(fill("{empty}", &v, true), None);
        assert_eq!(
            fill("a {missing} b", &v, false).as_deref(),
            Some("a {missing} b")
        );
        assert_eq!(
            fill("{Not} {x-y} {", &v, true).as_deref(),
            Some("{Not} {x-y} {")
        );
    }

    #[test]
    fn expands_per_app_org_and_bank_account() {
        let links = vec![
            json!({"name": "Status", "url": "https://status"}),
            json!({"name": "Open {org_name}", "url": "https://x/?s={short_code}", "per": "org",
                   "description": "for {org} {unknown}"}),
            json!({"name": "{org_name} · {account_name}: reconcile", "url": "https://r?{account_id}",
                   "per": "bank_account"}),
            json!({"name": "Doc", "url": "https://d/{document_id}", "per": "document"}),
            json!({"name": "Callback", "url": "{redirect_uri}"}),
        ];
        let ctx = vals(&[("redirect_uri", "http://localhost:8765/callback")]);
        let orgs = vec![
            vals(&[
                ("org", "roast"),
                ("org_name", "Tallowmere"),
                ("short_code", "!A"),
            ]),
            vals(&[("org", "never"), ("org_name", "never"), ("short_code", "")]),
        ];
        let banks = vec![vals(&[
            ("org", "roast"),
            ("org_name", "Tallowmere"),
            ("account_id", "a-1"),
            ("account_name", "Operating Account"),
        ])];
        let got = expand(&links, &ctx, &orgs, &banks);
        let names: Vec<&str> = got.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Status",
                "Open Tallowmere",
                "Tallowmere · Operating Account: reconcile",
                "Callback"
            ]
        );
        assert_eq!(got[1].description, "for roast {unknown}");
        assert_eq!(got[1].org.as_deref(), Some("roast"));
        assert_eq!(got[2].url, "https://r?a-1");
        assert_eq!(got[3].url, "http://localhost:8765/callback");
        assert_eq!(got[3].per, "app");
    }
}
