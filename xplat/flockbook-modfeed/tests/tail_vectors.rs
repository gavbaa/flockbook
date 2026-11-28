//! The first view of a log when its end holds no chat: the cases every side shares
//! (Tests/HerdrKitTests/Fixtures/tail-vectors.json). The Mac app (Sources/HerdrKitChecks/ChecksTailVectors.swift)
//! reads the same file: a rule changed on one side only fails here or there.

use herdr_core::ItemKind;
use modfeedkit::tail;
use serde_json::Value;
use std::path::PathBuf;

fn vectors() -> Value {
    serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Tests/HerdrKitTests/Fixtures/tail-vectors.json"))).unwrap()
}

/// The log a case describes: each entry's lines, `repeat` times, `{i}` the repeat's number, `{pad}` that many q's.
fn log(case: &Value) -> String {
    let mut out = String::new();
    for e in case["entries"].as_array().unwrap() {
        let lines: Vec<&str> = match e.get("lines") {
            Some(ls) => ls.as_array().unwrap().iter().map(|l| l.as_str().unwrap()).collect(),
            None => vec![e["line"].as_str().unwrap()],
        };
        let pad = "q".repeat(e.get("pad").and_then(Value::as_u64).unwrap_or(0) as usize);
        for i in 0..e.get("repeat").and_then(Value::as_u64).unwrap_or(1) {
            for l in &lines {
                out += &l.replace("{i}", &i.to_string()).replace("{pad}", &pad);
                out.push('\n');
            }
        }
    }
    out
}

#[test]
fn sizes() {
    let v = vectors();
    assert_eq!(v["tail_bytes"].as_u64(), Some(tail::BYTES));
    assert_eq!(v["search_max"].as_u64(), Some(tail::SEARCH_MAX));
}

#[test]
fn first_view() {
    let dir = std::env::temp_dir().join(format!("tail-vectors-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut wrong = vec![];
    for (n, c) in vectors()["cases"].as_array().unwrap().iter().enumerate() {
        let path: PathBuf = dir.join(format!("{n}.jsonl"));
        std::fs::write(&path, log(c)).unwrap();
        let (_, items) = tail::read_last(&path, tail::items_of).unwrap();
        let got: Vec<String> = items
            .iter()
            .filter_map(|it| match &it.kind {
                ItemKind::UserText(t) | ItemKind::AssistantText(t) => Some(t.clone()),
                _ => None,
            })
            .collect();
        let want: Vec<String> = c["want"].as_array().unwrap().iter().map(|w| w.as_str().unwrap().to_string()).collect();
        if got != want {
            wrong.push(format!("{}: {got:?}", c["name"].as_str().unwrap()));
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(wrong.is_empty(), "tail vectors: {wrong:#?}");
}
