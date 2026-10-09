//! Docs grep: no tracked doc (README*, CONTRIBUTING*, SECURITY*, CHANGELOG*, docs/**, any *.md) and not the
//! CLI's own help reference (`ownpurse help-markdown`) may mention the retired toolchain (its name, its project
//! file, its runner, a `.py` file), its CLI (`ownpurse` + `-py`), or the old placeholder wording ("for now",
//! "Not in the Rust CLI"). Case-insensitive.

use crate::files::{self, Item};
use regex::Regex;

fn rule() -> Regex {
    let py = ["py", "thon"].concat();
    let project = ["py", "project"].concat();
    let cli = ["ownpurse", "-py"].concat();
    let not_in = ["not in the ", "rust cli"].concat();
    Regex::new(&format!(
        r"(?i){py}|{project}|\bu{v}\s+(?:run|tool|sync|pip)\b|\bpip\s+install\b|\.py\b|{cli}|\bfor now\b|{not_in}",
        v = "v"
    ))
    .expect("docs regex")
}

/// A tracked file the docs rule applies to.
pub fn is_doc(name: &str) -> bool {
    let base = name.rsplit('/').next().unwrap_or(name);
    let upper = base.to_uppercase();
    name.starts_with("docs/")
        || name.ends_with(".md")
        || ["README", "CONTRIBUTING", "SECURITY", "CHANGELOG"]
            .iter()
            .any(|p| upper.starts_with(p))
}

pub fn check(items: &[Item]) -> Vec<String> {
    let re = rule();
    let mut hits = Vec::new();
    for it in items {
        let Some(text) = files::text(&it.bytes) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            if let Some(m) = re.find(line) {
                hits.push(format!(
                    "{}:{}: {:?}",
                    it.name,
                    n + 1,
                    m.as_str().to_lowercase()
                ));
            }
        }
    }
    hits
}

/// The CLI's help reference, as the docs site gets it.
fn help_markdown() -> Option<Item> {
    let out = std::process::Command::new(env!("CARGO"))
        .args(["run", "-q", "-p", "ownpurse-cli", "--", "help-markdown"])
        .current_dir(files::root().join("rust"))
        .output()
        .ok()?;
    out.status.success().then(|| Item {
        name: "ownpurse help-markdown".into(),
        bytes: out.stdout,
    })
}

pub fn run() -> i32 {
    let root = files::root();
    let mut items: Vec<Item> = files::tracked(&root)
        .into_iter()
        .filter(|i| is_doc(&i.name))
        .collect();
    let n = items.len();
    match help_markdown() {
        Some(h) => items.push(h),
        None => {
            println!("docs-grep: could not run `ownpurse help-markdown`");
            return 1;
        }
    }
    let hits = check(&items);
    for h in &hits {
        println!("docs-grep: {h}");
    }
    if hits.is_empty() {
        println!("docs-grep: OK ({n} docs + the CLI help)");
        0
    } else {
        println!("docs-grep: {} finding(s)", hits.len());
        1
    }
}

pub fn self_test() -> i32 {
    let item = |name: &str, body: String| Item {
        name: name.into(),
        bytes: body.into_bytes(),
    };
    let planted = [
        ["Built with Py", "thon 3."].concat(),
        ["see py", "project.toml"].concat(),
        ["u", "v run xero"].concat(),
        ["u", "v tool install x"].concat(),
        ["pip", " install x"].concat(),
        ["run gen", ".py first"].concat(),
        ["ownpurse", "-py q"].concat(),
        "this works for now".into(),
        ["Not in the ", "Rust CLI"].concat(),
    ];
    let mut ok = true;
    for p in &planted {
        if check(&[item("docs/x.md", p.clone())]).is_empty() {
            println!("docs-grep self-test: NOT caught: {p:?}");
            ok = false;
        }
    }
    let clean = "Install with ./install.sh; `ownpurse tui`; copy.pyramid; uv-index; for nowhere"
        .to_string();
    if !check(&[item("docs/ok.md", clean)]).is_empty() {
        println!("docs-grep self-test: clean text flagged");
        ok = false;
    }
    for (name, doc) in [
        ("README.md", true),
        ("docs/TUI.md", true),
        ("rust/tests/fixtures/demo/README.md", true),
        ("CHANGELOG.md", true),
        ("rust/crates/x/src/lib.rs", false),
    ] {
        if is_doc(name) != doc {
            println!("docs-grep self-test: is_doc({name}) != {doc}");
            ok = false;
        }
    }
    println!("docs-grep self-test: {}", if ok { "OK" } else { "FAILED" });
    if ok { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    #[test]
    fn self_test_passes() {
        assert_eq!(super::self_test(), 0);
    }
}
