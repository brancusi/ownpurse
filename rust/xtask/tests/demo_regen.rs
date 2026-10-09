//! The committed fixtures can never drift from their specs: regenerate everything into a temp folder and
//! compare byte for byte (twice over, so the generator itself is deterministic). And the specs keep the
//! collision rule: every invented amount has irregular cents.

use xtask::demo::{
    check, demo_amounts, engine_amounts, fixtures_dir, generate, load_spec, round_amounts,
};

#[test]
fn committed_fixtures_equal_their_specs() {
    let n = check(&fixtures_dir()).unwrap_or_else(|e| panic!("{e}"));
    assert!(n >= 30, "{n} files");
}

#[test]
fn two_runs_are_byte_identical() {
    let base = std::env::temp_dir().join(format!("xtask-twice-{}", std::process::id()));
    let (a, b) = (base.join("a"), base.join("b"));
    let files = generate(&fixtures_dir(), &a).unwrap();
    assert_eq!(generate(&fixtures_dir(), &b).unwrap(), files);
    for f in &files {
        assert!(
            std::fs::read(a.join(f)).unwrap() == std::fs::read(b.join(f)).unwrap(),
            "{f} differs between two runs"
        );
    }
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn every_spec_amount_has_irregular_cents() {
    let engine = load_spec(&fixtures_dir().join("engine.json")).unwrap();
    let world = load_spec(&fixtures_dir().join("demo/world.json")).unwrap();
    for (name, amounts) in [
        ("engine.json", engine_amounts(&engine).unwrap()),
        ("demo/world.json", demo_amounts(&world).unwrap()),
    ] {
        assert!(amounts.len() > 20, "{name}: only {} amounts", amounts.len());
        let bad = round_amounts(&amounts);
        assert!(bad.is_empty(), "{name}: amounts ending .00 or .50: {bad:?}");
    }
    assert_eq!(
        round_amounts(&[
            "12.00".into(),
            "3.50".into(),
            "-7.5".into(),
            "6083.29".into(),
            "0.01".into()
        ]),
        ["12.00", "3.50", "-7.5"]
    );
}
