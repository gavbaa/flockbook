import Foundation
@testable import HerdrKit

// The first view of a log when its end holds no chat: the cases every side shares, from
// Tests/HerdrKitTests/Fixtures/tail-vectors.json, which the mod's helper (xplat/flockbook-modfeed/tests/tail_vectors.rs)
// reads too. A rule changed on one side only fails here or there.

/// The log a case describes: each entry's lines, `repeat` times, `{i}` the repeat's number, `{pad}` that many q's.
private func tailLog(_ c: [String: Any]) -> String {
    var out = ""
    for e in c["entries"] as! [[String: Any]] {
        let lines = e["lines"] as? [String] ?? [e["line"] as! String]
        let pad = String(repeating: "q", count: e["pad"] as? Int ?? 0)
        for i in 0..<(e["repeat"] as? Int ?? 1) {
            for l in lines { out += l.replacingOccurrences(of: "{i}", with: "\(i)").replacingOccurrences(of: "{pad}", with: pad) + "\n" }
        }
    }
    return out
}

@MainActor func checksTailVectors() async throws {
    let v = try JSONSerialization.jsonObject(with: fixture("tail-vectors.json")) as! [String: Any]
    check(v["tail_bytes"] as? Int == Int(SessionLog.fullTail), "tail vectors: the full tail is 2 MB")
    check(v["search_max"] as? Int == Int(SessionLog.searchMax), "tail vectors: the search goes 8 MB back")
    let root = NSTemporaryDirectory() + "hktv-\(UUID().uuidString)"
    try FileManager.default.createDirectory(atPath: root + "/p", withIntermediateDirectories: true)
    defer { try? FileManager.default.removeItem(atPath: root) }
    final class Box: @unchecked Sendable { let lock = NSLock(); var items: [ConversationItem] = []; var at = Date() }
    for (n, c) in (v["cases"] as! [[String: Any]]).enumerated() {
        let name = c["name"] as! String, sid = "tv\(n)"
        try tailLog(c).write(toFile: root + "/p/\(sid).jsonl", atomically: true, encoding: .utf8)
        let box = Box(), log = SessionLog(sessionId: sid)
        let feed = Task.detached {
            for await s in log.snapshots(projectsDir: root, pollInterval: .milliseconds(20)) {
                box.lock.withLock { if s.items != box.items { box.items = s.items; box.at = Date() } }
            }
        }
        // No snapshot comes when nothing changes: the view is settled when it has not changed for 1 s (at most 10 s).
        let began = Date()
        try? await Task.sleep(for: .milliseconds(200))
        while Date().timeIntervalSince(began) < 10, box.lock.withLock({ Date().timeIntervalSince(box.at) }) < 1 {
            try? await Task.sleep(for: .milliseconds(50))
        }
        feed.cancel()
        let got = box.lock.withLock { box.items }.compactMap { i -> String? in
            switch i.kind { case .userText(let t), .assistantText(let t): return t; default: return nil }
        }
        check(got == c["want"] as! [String], "tail vectors: \(name): \(got)")
    }
}
