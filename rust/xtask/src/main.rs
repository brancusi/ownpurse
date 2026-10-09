//! Repository checks and generators for the release gate (`rust/scripts/check.sh` runs them):
//!
//!     cargo run -q -p xtask -- hygiene [--self-test]           no retired toolchain files or calls, no real paths,
//!                                                              UUIDs only from uuid-allow.txt, synthetic configs
//!     cargo run -q -p xtask -- docs-grep [--self-test]         no retired-tool references in any doc or the CLI help
//!     cargo run -q -p xtask -- denylist [--self-test] [DIR …] [--no-tracked]
//!                                                              the PRIVATE denylist (never in this repo; see denylist.rs)
//!     cargo run -q -p xtask -- docs-inputs <out-dir>           the docs-site inputs, from the demo fixture only
//!     cargo run -q -p xtask -- demo fixtures|states|check     the demo and engine fixtures from their specs (demo/)
//!
//! Exit codes: 0 clean, 1 findings (or a missing private list where one is required), 2 usage.

mod denylist;
mod docs;
mod docs_inputs;
mod files;
mod hygiene;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest: Vec<String> = args.iter().skip(1).cloned().collect();
    let self_test = rest.iter().any(|a| a == "--self-test");
    let code = match args.first().map(String::as_str) {
        Some("hygiene") if self_test => hygiene::self_test(),
        Some("hygiene") => hygiene::run(),
        Some("docs-grep") if self_test => docs::self_test(),
        Some("docs-grep") => docs::run(),
        Some("denylist") if self_test => denylist::self_test(),
        Some("denylist") => denylist::run(&rest),
        Some("docs-inputs") => match rest.first() {
            Some(out) => docs_inputs::run(std::path::Path::new(out)),
            None => usage(),
        },
        Some("demo") => match xtask::demo::run(&rest) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("xtask: {e}");
                1
            }
        },
        Some("agent-drive") => match xtask::live::run(&rest) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("xtask agent-drive: {e}");
                1
            }
        },
        _ => usage(),
    };
    std::process::exit(code);
}

fn usage() -> i32 {
    eprintln!(
        "usage: xtask hygiene [--self-test] | docs-grep [--self-test] | denylist [--self-test] [DIR …] [--no-tracked] | docs-inputs <out-dir> | demo fixtures|states|check"
    );
    2
}
