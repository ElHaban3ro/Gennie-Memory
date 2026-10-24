use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "gennie-reconciliation-{}-{}-{}",
            SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn cmd(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_gennie-memory"));
        c.env("GENNIE_MEMORY_DATABASE_PATH", self.0.join("db.sqlite"))
            .env("GENNIE_MEMORY_OPENROUTER_API_KEY", "");
        c
    }
    fn run(&self, op: &str, v: Value) -> Value {
        let p = self.0.join(format!("{op}.json"));
        fs::write(&p, v.to_string()).unwrap();
        let out = self.cmd().arg(op).arg(p).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn create(i: u32, text: &str) -> Value {
    json!({"schema_version":1,"operation_id":format!("123e4567-e89b-42d3-a456-{i:012}"),"action":"create","scope":{"kind":"project","id":"synthetic"},"topic_key":"paper/color","memory":{"original_text":text,"summary":text,"type":"preference","source":{"kind":"user","ref":format!("msg:{i}")}}})
}
#[test]
fn cli_proposals_resolve_and_jsonl_confirmation_audit_are_durable() {
    let s = Sandbox::new();
    let first = s.run("mem-write", create(1, "Use blue paper."));
    let id = first["memory_id"].as_str().unwrap();
    let p = s.run("mem-write", create(2, "Use red paper."));
    assert_eq!(p["status"], "needs_resolution");
    let prop = p["proposal_id"].as_str().unwrap();
    let inspected = s.run(
        "mem-proposal-get",
        json!({"schema_version":1,"scope":{"kind":"project","id":"synthetic"},"proposal_id":prop}),
    );
    assert_eq!(inspected["targets"][0]["revision"], 1);
    s.run("source-register",json!({"schema_version":1,"scope":{"kind":"project","id":"synthetic"},"source":{"kind":"user","ref":"resolution:1"},"version":"v1","text":"Keep blue paper."}));
    let discarded=s.run("mem-resolve",json!({"schema_version":1,"operation_id":"123e4567-e89b-42d3-a456-000000000003","proposal_id":prop,"expected_targets":[{"memory_id":id,"revision":1}],"decision":"keep_existing","evidence":{"original_text":"Keep blue paper.","source":{"kind":"user","ref":"resolution:1"}}}));
    assert_eq!(discarded["outcome"], "discarded");
    let lines = vec![
        json!({"op":"mem_write","request":{"schema_version":1,"operation_id":"123e4567-e89b-42d3-a456-000000000004","action":"confirm","memory_id":id,"expected_revision":1,"evidence":{"original_text":"Keep blue paper.","source":{"kind":"user","ref":"resolution:1"}}}}),
        json!({"op":"mem_audit","request":{"schema_version":1,"scope":{"kind":"project","id":"synthetic"},"memory_id":id}}),
        json!({"op":"mem_operation_get","request":{"schema_version":1,"scope":{"kind":"project","id":"synthetic"},"operation_id":"123e4567-e89b-42d3-a456-000000000004"}}),
    ];
    let mut child = s
        .cmd()
        .arg("session")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    writeln!(input, "{{invalid").unwrap();
    for line in lines {
        writeln!(input, "{line}").unwrap()
    }
    drop(input);
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rows: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0]["status"], "failed");
    assert_eq!(rows[1]["response"]["outcome"], "confirmed");
    assert_eq!(rows[1]["response"]["revision"], 2);
    assert_eq!(
        rows[2]["response"]["entries"][1]["provenance"]["status"],
        "verified_literal"
    );
    assert_eq!(rows[3]["response"]["outcome"], "confirmed");
    let replay = s.run("mem-write", create(1, "Use blue paper."));
    assert_eq!(replay["replayed"], true);
    assert_eq!(replay["revision"], 1);
}
#[test]
fn strict_cli_proposal_is_not_published_as_a_memory() {
    let s = Sandbox::new();
    let mut request = create(1, "Use blue paper.");
    request["source_policy"] = json!("require_verified");
    let proposal = s.run("mem-write", request);
    assert_eq!(proposal["status"], "needs_resolution");
    assert!(proposal["memory_id"].is_null());
    let receipt=s.run("mem-operation-get",json!({"schema_version":1,"scope":{"kind":"project","id":"synthetic"},"operation_id":"123e4567-e89b-42d3-a456-000000000001"}));
    assert_eq!(receipt, proposal);
}
