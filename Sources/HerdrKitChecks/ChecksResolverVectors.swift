import Foundation
@testable import HerdrKit

// The process tree and Claude Code pid files: the cases every side shares, from
// Tests/HerdrKitTests/Fixtures/resolver-vectors.json, which the Rust resolver (xplat/herdr-core/tests/resolver_vectors.rs)
// reads too. A rule changed on one side only fails here or there.

/// A `procStart` that is a number: Linux clock ticks or a Windows FILETIME. The Swift resolver reads only `ps lstart`
/// text so far (issue #32), so these cases wait for it.
private func numeric(_ c: [String: Any]) -> Bool {
    (c["files"] as! [String: String]).values.contains { f in
        guard let o = try? JSONSerialization.jsonObject(with: Data(f.utf8)) as? [String: Any], let ps = o["procStart"] as? String else { return false }
        return !ps.isEmpty && ps.utf8.allSatisfy { (48...57).contains($0) }
    }
}

func checksResolverVectors() throws {
    let v = try JSONSerialization.jsonObject(with: fixture("resolver-vectors.json")) as! [String: Any]
    let utc = TimeZone(identifier: "UTC")!, iso = ISO8601DateFormatter()
    for c in v["ps"] as! [[String: Any]] {
        let name = c["name"] as! String
        let got = SessionResolver.parsePS(c["text"] as! String, timeZone: utc)
        let want = (c["want"] as! [[Any]]).map { w in
            SessionResolver.Proc(pid: Int32(w[0] as! Int), ppid: Int32(w[1] as! Int), start: iso.date(from: w[2] as! String)!, line: w[3] as! String)
        }
        check(got == want, "resolver vectors: ps, \(name): \(got.map { "\($0.pid) \($0.ppid) \(iso.string(from: $0.start)) [\($0.line)]" })")
    }
    for c in v["claude_session"] as! [[String: Any]] where !numeric(c) {
        let name = c["name"] as! String
        let procs = SessionResolver.parsePS(c["ps"] as! String, timeZone: utc)
        let files = c["files"] as! [String: String]
        let got = SessionResolver.claudeSessionSource(pids: (c["pids"] as! [Int]).map { Int32($0) }, procs: procs) { pid in
            files[String(pid)].map { Data($0.utf8) }
        }
        let want = (c["want"] as? [Any]).map { ($0[0] as! String, Int32($0[1] as! Int)) }
        check(got?.sid == want?.0 && got?.pid == want?.1, "resolver vectors: claude session, \(name): \(got.map { "\($0.sid) \($0.pid)" } ?? "none")")
    }
}
