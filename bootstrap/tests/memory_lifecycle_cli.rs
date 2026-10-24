use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
struct Sandbox(PathBuf);
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Sandbox {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "gennie-lifecycle-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_gennie-memory"));
        c.env("GENNIE_MEMORY_DATABASE_PATH", self.0.join("memory.db"))
            .env("GENNIE_MEMORY_OPENROUTER_API_KEY", "");
        c
    }
    fn run(&self, name: &str, request: &Value) -> Value {
        let path = self.0.join("request.json");
        fs::write(&path, request.to_string()).unwrap();
        let output = self.command().arg(name).arg(path).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}
#[test]
fn lifecycle_and_queue_persist_across_cli_and_jsonl_sessions() {
    let s = Sandbox::new();
    let scope = json!({"kind":"project","id":"demo"});
    let queued = json!({"action":"enqueue","scope":scope,"space_id":"test-space"});
    let job = s.run("embedding-job", &queued);
    assert_eq!(job["status"], "pending");
    let write = json!({"schema_version":1,"operation_id":"123e4567-e89b-42d3-a456-426614174000","action":"create","scope":scope,"memory":{"type":"fact","original_text":"cobalt","summary":"cobalt","source":{"kind":"document","ref":"synthetic"}}});
    let saved = s.run("mem-write", &write);
    let mut get = queued.clone();
    get["action"] = json!("get");
    assert_eq!(s.run("embedding-job", &get)["generation"], 2);
    let mut cancel = queued.clone();
    cancel["action"] = json!("cancel");
    s.run("embedding-job", &cancel);
    assert_eq!(s.run("embedding-job", &get)["status"], "cancelled");
    let list = json!({"scope":scope,"as_of":"2026-10-24T12:00:00.000Z","limit":20});
    assert_eq!(
        s.run("mem-review-list", &list)["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let mut control = json!({"schema_version":1,"operation_id":"123e4567-e89b-42d3-a456-426614174001","action":"withdraw","memory_id":saved["memory_id"],"expected_revision":1,"scope":scope,"as_of":"2026-10-24T12:00:00.000Z","next_review_at":null,"evidence":{"original_text":"The operator requested this action.","source":{"kind":"user","ref":"synthetic-request"}}});
    s.run("mem-lifecycle", &control);
    assert_eq!(s.run("mem-lifecycle", &control)["replayed"], true);
    assert!(
        s.run("mem-review-list", &list)["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    control["action"] = json!("restore");
    control["expected_revision"] = json!(2);
    control["operation_id"] = json!("123e4567-e89b-42d3-a456-426614174002");
    let mut child = s
        .command()
        .arg("session")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let input = format!(
        "{{invalid}}\n{}\n{}\n",
        json!({"op":"mem_lifecycle","request":control}),
        json!({"op":"mem_review_list","request":list})
    );
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let rows: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|v| serde_json::from_str(v).unwrap())
        .collect();
    assert_eq!(rows[0]["status"], "failed");
    assert_eq!(rows[1]["status"], "complete");
    assert_eq!(rows[2]["response"]["items"].as_array().unwrap().len(), 1);
    control["action"] = json!("delete");
    control["expected_revision"] = json!(3);
    control["operation_id"] = json!("123e4567-e89b-42d3-a456-426614174003");
    s.run("mem-lifecycle", &control);
    assert!(
        s.run("mem-review-list", &list)["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
