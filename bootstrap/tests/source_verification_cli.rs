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
            "gennie-sources-{}-{}-{}",
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
fn source() -> Value {
    json!({"schema_version":1,"scope":{"kind":"project","id":"source-demo"},"source":{"kind":"document","ref":"synthetic-note"},"version":"v1","text":"Encabezado\nNo enviar si llueve en Bogotá 😀.\nFin."})
}
fn write() -> Value {
    json!({"schema_version":1,"operation_id":"223e4567-e89b-42d3-a456-000000001100","action":"create","scope":{"kind":"project","id":"source-demo"},"memory":{"type":"fact","original_text":"No enviar si llueve en Bogotá 😀.","summary":"No enviar si llueve en Bogotá 😀.","source":{"kind":"document","ref":"synthetic-note"}}})
}
#[test]
fn register_write_verify_and_reopen_keep_the_same_local_receipt() {
    let sandbox = Sandbox::new();
    let registered = sandbox.run("source-register", &source());
    assert_eq!(registered["trust_basis"], "operator_registered_snapshot");
    let saved = sandbox.run("mem-write", &write());
    let verify = json!({"schema_version":1,"scope":{"kind":"project","id":"source-demo"},"memory_id":saved["memory_id"],"revision":saved["revision"]});
    let first = sandbox.run("source-verify", &verify);
    assert_eq!(first["status"], "verified_literal");
    assert_eq!(first["semantic_support"], "not_assessed");
    assert_eq!(sandbox.run("source-verify", &verify), first);
    assert_eq!(first["snapshot_id"], registered["snapshot_id"]);
    let request = json!({"contract_version":1,"source_policy":"require_verified","budgets":{"input_tokens":1},"search":{"query":"Bogotá llueve","scope_kind":"project","scope_id":"source-demo","as_of":"2099-01-01T00:00:00.000Z"}});
    let path = sandbox.0.join("recall.json");
    fs::write(&path, request.to_string()).unwrap();
    let result = sandbox
        .command()
        .env("GENNIE_MEMORY_OPENROUTER_API_KEY", "synthetic-not-a-secret")
        .arg("recall-v1")
        .arg(path)
        .output()
        .unwrap();
    assert!(result.status.success());
    let recall: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(recall["issues"][0]["code"], "input_token_limit");
    assert_eq!(recall["source_verification"]["checks"][0], first);
    assert_eq!(recall["usage"]["classifier_invocations"], 0);
}
#[test]
fn jsonl_new_versions_failures_and_old_version_replays_preserve_current_snapshot() {
    let sandbox = Sandbox::new();
    let saved = sandbox.run("mem-write", &write());
    let verify = json!({"schema_version":1,"scope":{"kind":"project","id":"source-demo"},"memory_id":saved["memory_id"],"revision":1});
    let mut v2 = source();
    v2["version"] = "v2".into();
    v2["text"] = "La cita ya no aparece aquí.".into();
    let mut foreign = verify.clone();
    foreign["actor_id"] = "other".into();
    let mut conflict = v2.clone();
    conflict["text"] = "Contenido distinto bajo versión ocupada.".into();
    let lines = [
        json!({"op":"source_register","request":source()}),
        json!({"op":"source_verify","request":verify}),
        json!({"op":"source_register","request":v2}),
        json!({"op":"source_verify","request":verify}),
        json!({"op":"source_verify","request":foreign}),
        json!({"op":"source_register","request":conflict}),
        json!({"op":"source_register","request":source()}),
        json!({"op":"source_verify","request":verify}),
    ];
    let mut child = sandbox
        .command()
        .arg("session")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut input = child.stdin.take().unwrap();
        for line in lines {
            writeln!(input, "{line}").unwrap();
        }
    }
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let rows: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(rows.len(), 8);
    assert_eq!(rows[1]["response"]["status"], "verified_literal");
    assert_eq!(rows[3]["response"]["status"], "rejected");
    assert_eq!(rows[4]["status"], "failed");
    assert!(rows[4]["response"].is_null());
    assert_eq!(rows[5]["status"], "failed");
    assert_eq!(rows[6]["response"]["replayed"], true);
    assert_eq!(rows[7]["response"], rows[3]["response"]);
}
