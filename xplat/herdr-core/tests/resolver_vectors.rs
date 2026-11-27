//! The process tree and Claude Code pid files: the cases every side shares (Tests/HerdrKitTests/Fixtures/resolver-vectors.json).
//! The Mac app (Sources/HerdrKitChecks/ChecksResolverVectors.swift) reads the same file: a rule changed on one side only
//! fails here or there.

use chrono::{DateTime, Utc};
use herdr_core::resolver::{claude_session_source_with_ticks, parse_ps_in};
use serde_json::Value;

fn vectors() -> Value {
    serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Tests/HerdrKitTests/Fixtures/resolver-vectors.json"))).unwrap()
}

#[test]
fn ps_lines() {
    for c in vectors()["ps"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let got: Vec<(i32, i32, String, String)> = parse_ps_in(c["text"].as_str().unwrap(), &Utc)
            .into_iter()
            .map(|p| (p.pid, p.ppid, p.start.format("%Y-%m-%dT%H:%M:%SZ").to_string(), p.line))
            .collect();
        let want: Vec<(i32, i32, String, String)> = c["want"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| (w[0].as_i64().unwrap() as i32, w[1].as_i64().unwrap() as i32, w[2].as_str().unwrap().to_string(), w[3].as_str().unwrap().to_string()))
            .collect();
        assert_eq!(got, want, "ps, {name}");
    }
}

#[test]
fn claude_sessions() {
    for c in vectors()["claude_session"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let procs = parse_ps_in(c["ps"].as_str().unwrap(), &Utc);
        let pids: Vec<i32> = c["pids"].as_array().unwrap().iter().map(|p| p.as_i64().unwrap() as i32).collect();
        // The machine's clock: its boot time and clock ticks per second. None: a machine without one (a Mac).
        let clock = c.get("clock").map(|k| (k["btime"].as_i64().unwrap(), k["hz"].as_u64().unwrap()));
        let ticks = |n: u64| clock.and_then(|(btime, hz)| DateTime::<Utc>::from_timestamp(btime + (n / hz) as i64, 0));
        let files = &c["files"];
        let got = claude_session_source_with_ticks(&pids, &procs, ticks, |pid| {
            files.get(pid.to_string()).and_then(|f| f.as_str()).map(|f| f.as_bytes().to_vec())
        });
        let want = c["want"].as_array().map(|w| (w[0].as_str().unwrap().to_string(), w[1].as_i64().unwrap() as i32));
        assert_eq!(got, want, "claude session, {name}");
    }
}
