//! The live agent-drive suite (`xtask agent-drive`): the REAL `ownpurse-tui` in a pseudo-terminal on a temp
//! copy of the demo fixture, driven only through the real `ownpurse ui …` CLI, as an agent drives it. After
//! every step the state the bridge reports must be what the terminal actually shows.

pub mod pty;
pub mod schema;
pub mod suite;
pub mod term;
pub mod world;

pub use world::World;

/// `xtask agent-drive [explore|run] [--bin-dir DIR]`.
pub fn run(args: &[String]) -> Result<(), String> {
    let dir = args
        .iter()
        .position(|a| a == "--bin-dir")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| crate::demo::fixtures_dir().join("../../target/release"));
    // absolute: the TUI is spawned with the world's folder as its working directory
    let abs = |p: std::path::PathBuf| std::fs::canonicalize(&p).unwrap_or(p);
    let dir = abs(dir);
    let mut bins = world::Bins::in_dir(&dir)?;
    bins.bumped = args
        .iter()
        .position(|a| a == "--bumped")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from)
        .map(abs);
    bins.same_version = args
        .iter()
        .position(|a| a == "--same-version")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from)
        .map(abs);
    match args.first().map(String::as_str) {
        Some("explore") => suite::explore(bins),
        Some("dump-states") => {
            suite::dump_states(bins, args.get(1).map(String::as_str).unwrap_or("states"))
        }
        Some("probe") => {
            let script: Vec<&str> = args[1..]
                .iter()
                .take_while(|a| *a != "--bin-dir")
                .map(String::as_str)
                .collect();
            suite::probe(bins, &script)
        }
        _ => suite::all(bins, args),
    }
}
