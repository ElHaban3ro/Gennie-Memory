use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
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
        static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "gennie-isolation-{}-{}-{}",
            SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_gennie-memory"));
        command.env("GENNIE_MEMORY_DATABASE_PATH", self.0.join("memory.db"));
        command.env("GENNIE_MEMORY_OPENROUTER_API_KEY", "");
        command
    }
    fn run(&self, command: &str, input: &Value) -> Value {
        let file = self.0.join("request.json");
        fs::write(&file, serde_json::to_vec(input).unwrap()).unwrap();
        let output = self.command().arg(command).arg(file).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}

#[test]
fn entity_write_search_and_jsonl_use_the_same_persisted_contract() {
    let sandbox = Sandbox::new();
    let entity = json!({"schema_version":1,"operation_id":"123e4567-e89b-42d3-a456-000000000001","kind":"person","label":"Ana","scope":{"kind":"project","id":"demo"},"source":{"kind":"document","ref":"synthetic"}});
    let created = sandbox.run("entity-resolve", &entity);
    assert_eq!(created["status"], "created");
    assert_eq!(sandbox.run("entity-resolve", &entity)["replayed"], true);
    let write = json!({"schema_version":1,"operation_id":"223e4567-e89b-42d3-a456-000000000001","action":"create","entity_id":created["entity_id"],"scope":{"kind":"project","id":"demo"},"memory":{"type":"fact","original_text":"Ana usa azul cobalto.","summary":"Ana usa azul cobalto.","source":{"kind":"document","ref":"synthetic"}}});
    let saved = sandbox.run("mem-write", &write);
    assert_eq!(saved["outcome"], "created");
    let query = json!({"query":"\"azul cobalto\"","scope_kind":"project","scope_id":"demo","entity_id":created["entity_id"],"memory_types":["fact"],"bm25_weights":[1.0,1.5],"as_of":"2026-10-24T12:00:00.000Z","limit":1});
    let found = sandbox.run("search", &query);
    assert_eq!(found["candidates"][0]["memory_id"], saved["memory_id"]);
    assert_eq!(found["coverage"]["retrieval"], "complete");
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
        writeln!(input, "{}", json!({"op":"entity_resolve","request":entity})).unwrap();
        writeln!(input, "{}", json!({"op":"search","request":query})).unwrap();
        let mut other = query.clone();
        other["actor_id"] = json!("other");
        writeln!(input, "{}", json!({"op":"search","request":other})).unwrap();
        // A malformed line must not poison the reused connection.
        writeln!(input, "not json").unwrap();
        writeln!(input, "{}", json!({"op":"search","request":query})).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lines: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 5);
    assert_eq!(lines[0]["response"]["replayed"], true);
    assert_eq!(
        lines[1]["response"]["candidates"][0]["memory_id"],
        saved["memory_id"]
    );
    assert!(
        lines[2]["response"]["candidates"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(lines[3]["status"], "failed");
    assert_eq!(lines[4]["response"]["coverage"]["matched_memories"], 1);
}

#[test]
fn recall_token_budget_is_enforced_by_cli_and_each_jsonl_operation() {
    let sandbox = Sandbox::new();
    let fixtures =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/conditional-corpus.json");
    let loaded = sandbox
        .command()
        .arg("load-fixtures")
        .arg(fixtures)
        .output()
        .unwrap();
    assert!(
        loaded.status.success(),
        "{}",
        String::from_utf8_lossy(&loaded.stderr)
    );
    let request = json!({"contract_version":1,"search":{"query":"agenda reuniones remotas Proyecto Sintético","scope_kind":"project","scope_id":"synthetic_meetings","as_of":"2026-10-15T12:00:00.000Z"},"budgets":{"input_tokens":1}});
    let file = sandbox.0.join("recall-request.json");
    fs::write(&file, request.to_string()).unwrap();
    // Nonempty synthetic key builds the native payload. The one-token guard must
    // reject locally before reqwest can contact the provider.
    let output = sandbox
        .command()
        .env("GENNIE_MEMORY_OPENROUTER_API_KEY", "synthetic-not-a-secret")
        .arg("recall-v1")
        .arg(file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let direct: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(direct["issues"][0]["code"], "input_token_limit");
    assert_eq!(direct["token_accounting"]["requests"][0]["accepted"], false);
    assert_eq!(direct["usage"]["classifier_invocations"], 0);
    let mut child = sandbox
        .command()
        .env("GENNIE_MEMORY_OPENROUTER_API_KEY", "synthetic-not-a-secret")
        .arg("session")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut input = child.stdin.take().unwrap();
        for _ in 0..2 {
            writeln!(input, "{}", json!({"op":"recall_v1","request":request})).unwrap();
        }
    }
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let rows: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(rows.len(), 2);
    for row in rows {
        assert_eq!(row["response"]["issues"][0]["code"], "input_token_limit");
        assert_eq!(
            row["response"]["token_accounting"]["requests"],
            direct["token_accounting"]["requests"]
        );
        assert_eq!(
            row["response"]["token_accounting"]["accepted_input_tokens"],
            0
        );
        assert_eq!(row["response"]["usage"]["cost_status"], "not_called");
    }
}
