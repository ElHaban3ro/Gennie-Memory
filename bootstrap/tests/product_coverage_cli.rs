//! Short local CLI controls: persistence, JSONL recovery, concurrent readers.
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc, Barrier,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "gennie-product-coverage-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_gennie-memory"));
        command
            .current_dir(&self.0)
            .env("GENNIE_MEMORY_DATABASE_PATH", self.0.join("memory.db"))
            .env("GENNIE_MEMORY_OPENROUTER_API_KEY", "");
        command
    }
    fn load(&self) {
        let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures/product-coverage-v1/corpus.json");
        let out = self
            .command()
            .arg("load-fixtures")
            .arg(corpus)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    fn run(&self, op: &str, request: &Value) -> Value {
        let file = self.0.join("request.json");
        fs::write(&file, request.to_string()).unwrap();
        let out = self.command().arg(op).arg(file).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    }
    fn session(&self, lines: &[String]) -> Vec<Value> {
        let mut child = self
            .command()
            .arg("session")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        for line in lines {
            writeln!(stdin, "{line}").unwrap();
        }
        drop(stdin);
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn query() -> Value {
    json!({"query":"ambar","scope_kind":"project","scope_id":"coverage","actor_id":"local","entity_id":"person-a","memory_types":["fact"],"as_of":"2026-10-24T12:00:00.000Z","limit":20})
}

fn assert_private_result(out: &Value) {
    let candidates = out["candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), 1, "{out}");
    assert_eq!(candidates[0]["memory_id"], "private-allowed");
    assert_eq!(out["coverage"]["retrieval"], "complete");
    assert!(!out.to_string().contains("SYNTHETIC_PRIVATE_"));
}

#[test]
fn failed_reimport_and_process_reopen_preserve_revisions_identity_and_isolation() {
    let sandbox = Sandbox::new();
    sandbox.load();
    for _ in 0..3 {
        // Import is deliberately not idempotent: duplicate revisions reject the
        // transaction. Reopening must still expose the original complete state.
        let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures/product-coverage-v1/corpus.json");
        let rejected = sandbox
            .command()
            .arg("load-fixtures")
            .arg(corpus)
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        assert_private_result(&sandbox.run("search", &query()));
        let mut revised = query();
        revised["query"] = json!("cobrenuevo");
        let out = sandbox.run("search", &revised);
        assert_eq!(out["candidates"].as_array().unwrap().len(), 1);
        assert_eq!(out["candidates"][0]["revision"], 2);
        revised["query"] = json!("cobreviejo");
        assert!(
            sandbox.run("search", &revised)["candidates"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        revised["query"] = json!("obsidiana");
        assert_eq!(
            sandbox.run("search", &revised)["candidates"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }
}

#[test]
fn jsonl_recovers_after_multiple_invalid_operations_and_keeps_authorization_per_request() {
    let sandbox = Sandbox::new();
    sandbox.load();
    let mut invalid = query();
    invalid["limit"] = json!(0);
    let mut other = query();
    other["actor_id"] = json!("unknown");
    let mut wrong_scope = query();
    wrong_scope["scope_id"] = json!("empty");
    let lines = vec![
        "{malformed".into(),
        json!({"op":"search","request":query()}).to_string(),
        json!({"op":"search","request":invalid}).to_string(),
        json!({"op":"search","request":other}).to_string(),
        json!({"op":"unsupported_operation","request":{}}).to_string(),
        json!({"op":"search","request":wrong_scope}).to_string(),
        json!({"op":"search","request":query()}).to_string(),
    ];
    let rows = sandbox.session(&lines);
    assert_eq!(rows.len(), lines.len());
    for index in [0, 2, 4] {
        assert_eq!(rows[index]["status"], "failed");
    }
    for index in [1, 6] {
        assert_private_result(&rows[index]["response"]);
    }
    for index in [3, 5] {
        assert_eq!(rows[index]["status"], "complete");
        assert!(
            rows[index]["response"]["candidates"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn four_concurrent_process_sessions_keep_filters_for_ninety_six_reads() {
    let sandbox = Arc::new(Sandbox::new());
    sandbox.load();
    let barrier = Arc::new(Barrier::new(4));
    let workers: Vec<_> = (0..4)
        .map(|worker| {
            let sandbox = Arc::clone(&sandbox);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let mut own = query();
                if worker % 2 == 1 {
                    own["entity_id"] = json!("person-b");
                }
                let lines = vec![json!({"op":"search","request":own}).to_string(); 24];
                barrier.wait();
                let rows = sandbox.session(&lines);
                assert_eq!(rows.len(), 24);
                for row in rows {
                    assert_eq!(row["status"], "complete");
                    if worker % 2 == 0 {
                        assert_private_result(&row["response"]);
                    } else {
                        let candidates = row["response"]["candidates"].as_array().unwrap();
                        assert_eq!(candidates.len(), 1);
                        assert_eq!(candidates[0]["memory_id"], "private-entity");
                        let text = row.to_string();
                        for forbidden in [
                            "SYNTHETIC_PRIVATE_ACTOR_782",
                            "SYNTHETIC_PRIVATE_SCOPE_349",
                            "SYNTHETIC_PRIVATE_TYPE_625",
                        ] {
                            assert!(!text.contains(forbidden));
                        }
                    }
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_private_result(&sandbox.run("search", &query()));
}
