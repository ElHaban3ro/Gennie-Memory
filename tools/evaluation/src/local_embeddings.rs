//! Offline local-model comparison through the real product JSONL path.
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

struct Session {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.input.take();
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
impl Session {
    fn read(&mut self) -> Result<Value, String> {
        let mut line = String::new();
        self.output
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&line).map_err(|e| format!("embedding_session_response: {e}"))
    }
    fn call(&mut self, op: &str, request: Value) -> Result<Value, String> {
        writeln!(
            self.input.as_mut().ok_or("session closed")?,
            "{}",
            json!({"op":op,"request":request})
        )
        .map_err(|e| e.to_string())?;
        self.input
            .as_mut()
            .unwrap()
            .flush()
            .map_err(|e| e.to_string())?;
        let result = self.read()?;
        if result["status"] != "complete" {
            return Err(format!("{result}"));
        }
        Ok(result)
    }
}
fn quantile(values: &[f64], q: f64) -> f64 {
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    if v.is_empty() {
        return 0.0;
    }
    v[((v.len() as f64 * q).ceil() as usize)
        .saturating_sub(1)
        .min(v.len() - 1)]
}

pub fn run(binary: &Path, settings: &Path, suite: &Path, out: &Path) -> Result<(), String> {
    let suite_bytes = fs::read(suite).map_err(|e| e.to_string())?;
    let suite: Value = serde_json::from_slice(&suite_bytes).map_err(|e| e.to_string())?;
    let corpus = suite["corpus"].as_array().ok_or("corpus missing")?;
    let cases = suite["cases"].as_array().ok_or("cases missing")?;
    if corpus.is_empty() || cases.is_empty() || cases.len() > 100 || suite["rounds"] != 3 {
        return Err("embedding_suite_invalid".into());
    }
    fs::create_dir(out).map_err(|e| format!("se requiere directorio nuevo: {e}"))?;
    let db = out.join("memory.db");
    let corpus_path = out.join("corpus.json");
    fs::write(
        &corpus_path,
        serde_json::to_vec_pretty(corpus).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let loaded = Command::new(binary)
        .args(["load-fixtures"])
        .arg(&corpus_path)
        .env("GENNIE_MEMORY_DATABASE_PATH", &db)
        .output()
        .map_err(|e| e.to_string())?;
    if !loaded.status.success() {
        return Err(String::from_utf8_lossy(&loaded.stderr).into());
    }
    let mut child = Command::new(binary)
        .arg("embedding-session")
        .arg(settings)
        .env("GENNIE_MEMORY_DATABASE_PATH", &db)
        .env("GENNIE_MEMORY_OPENROUTER_API_KEY", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut session = Session {
        input: child.stdin.take(),
        output: BufReader::new(child.stdout.take().unwrap()),
        child,
    };
    let ready = session.read()?;
    if ready["status"] != "ready" {
        return Err(format!("{ready}"));
    }
    fs::write(
        out.join("runtime.json"),
        serde_json::to_vec_pretty(&ready).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    let spec = json!({"scope":{"kind":"project","id":"embedding-study"},"deadline_ms":300000});
    let prepared = session.call("prepare", spec)?;
    fs::write(
        out.join("prepared-index.json"),
        serde_json::to_vec(&prepared["response"]).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    let published = session.call("publish", prepared["response"].clone())?;
    let index = published["response"]["index_id"].clone();
    let mut records = fs::File::create(out.join("searches.jsonl")).map_err(|e| e.to_string())?;
    let mut by_mode = serde_json::Map::new();
    for mode in ["fts5", "semantic", "hybrid"] {
        let mut top1 = 0;
        let mut top5 = 0;
        let mut top20 = 0;
        let mut positive = 0;
        let mut empty = 0;
        let mut empty_positive = 0;
        let mut latencies = Vec::new();
        let mut groups = serde_json::Map::new();
        let mut wrong = Vec::new();
        for round in 0..3 {
            for case in cases {
                let request = json!({"query":case["query"],"scope_kind":"project","scope_id":"embedding-study","as_of":"2026-10-24T12:00:00.000Z","limit":20});
                let response = if mode == "fts5" {
                    session.call("text_search", request)?
                } else {
                    session.call("search",json!({"index_id":index,"search":request,"deadline_ms":30000,"semantic_only":mode=="semantic"}))?
                };
                let body = &response["response"];
                latencies.push(if mode == "fts5" {
                    response["elapsed_ms"].as_f64().unwrap()
                } else {
                    body["total_ms"].as_f64().unwrap()
                });
                let returned = body["candidates"].as_array().ok_or("candidates missing")?;
                if round == 0 {
                    let expected = case["expected_ids"]
                        .as_array()
                        .ok_or("expected_ids missing")?;
                    if returned.is_empty() {
                        empty += 1;
                    }
                    if !expected.is_empty() {
                        positive += 1;
                        if returned.is_empty() {
                            empty_positive += 1;
                        }
                        let rank = returned
                            .iter()
                            .position(|v| expected.contains(&v["memory_id"]))
                            .map(|p| p + 1);
                        top1 += usize::from(rank.is_some_and(|r| r <= 1));
                        top5 += usize::from(rank.is_some_and(|r| r <= 5));
                        top20 += usize::from(rank.is_some_and(|r| r <= 20));
                        let group = case["group"].as_str().unwrap();
                        let g = groups
                            .entry(group)
                            .or_insert_with(|| json!({"cases":0,"top1":0,"top5":0}));
                        g["cases"] = json!(g["cases"].as_u64().unwrap() + 1);
                        g["top1"] = json!(g["top1"].as_u64().unwrap() + u64::from(rank == Some(1)));
                        g["top5"] = json!(
                            g["top5"].as_u64().unwrap() + u64::from(rank.is_some_and(|r| r <= 5))
                        );
                        if rank.is_none_or(|r| r > 5) {
                            wrong.push(case["id"].clone());
                        }
                    }
                }
                writeln!(
                    records,
                    "{}",
                    json!({"mode":mode,"round":round,"case":case,"result":response})
                )
                .map_err(|e| e.to_string())?;
            }
        }
        by_mode.insert(mode.into(),json!({"positives":positive,"top1":top1,"top5":top5,"top20":top20,"empty_queries":empty,"empty_positive_queries":empty_positive,"p50_ms":quantile(&latencies,0.5),"p95_ms":quantile(&latencies,0.95),"samples":latencies.len(),"groups":groups,"misses_top5":wrong}));
    }
    session.input.take();
    let status = session.child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("embedding_session_failed".into());
    }
    use sha2::{Digest, Sha256};
    let report = json!({"schema_version":1,"suite_sha256":Sha256::digest(&suite_bytes).iter().map(|b|format!("{b:02x}")).collect::<String>(),"profile":ready["profile"],"model_load_ms":ready["model_load_ms"],"preparation_ms":prepared["elapsed_ms"],"publication_ms":published["elapsed_ms"],"index":published["response"],"metrics":by_mode,"limits":"Synthetic same-author controls; candidate retrieval only, no semantic classification or calibrated abstention. Load excluded from warm query latency. FTS default unchanged."});
    fs::write(
        out.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    Ok(())
}

pub fn scale(binary: &Path, settings: &Path, out: &Path) -> Result<(), String> {
    fs::create_dir(out).map_err(|e| format!("se requiere directorio nuevo: {e}"))?;
    let db = out.join("scale.db");
    let mut child = Command::new(binary)
        .arg("embedding-session")
        .arg(settings)
        .env("GENNIE_MEMORY_DATABASE_PATH", &db)
        .env("GENNIE_MEMORY_OPENROUTER_API_KEY", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut session = Session {
        input: child.stdin.take(),
        output: BufReader::new(child.stdout.take().unwrap()),
        child,
    };
    let ready = session.read()?;
    if ready["status"] != "ready" {
        return Err(format!("{ready}"));
    }
    let fixture = |n: usize| json!({"id":format!("scale-{n:06}"),"scope_kind":"project","scope_id":"embedding-scale","revision":1,"memory_type":"fact","original_text":"The cobalt sample belongs to the laboratory.","summary":"The cobalt sample belongs to the laboratory.","state":"active","valid_from":null,"valid_until":null});
    session.call("load_fixtures", json!([fixture(0)]))?;
    let template = session.call(
        "prepare",
        json!({"scope":{"kind":"project","id":"embedding-scale"},"deadline_ms":30000}),
    )?["response"]
        .clone();
    let mut results = Vec::new();
    let mut previous = 1;
    for size in [100, 1000, 10000] {
        let extra: Vec<_> = (previous..size).map(&fixture).collect();
        session.call("load_fixtures", json!(extra))?;
        previous = size;
        let mut batch = template.clone();
        let source = &template["sources"][0];
        let chunks = template["chunks"]
            .as_array()
            .ok_or("template chunks missing")?;
        let mut sources = Vec::with_capacity(size);
        let mut expanded = Vec::with_capacity(size * chunks.len());
        for n in 0..size {
            let id = json!(format!("scale-{n:06}"));
            let mut source = source.clone();
            source["memory_id"] = id.clone();
            sources.push(source);
            for c in chunks {
                let mut c = c.clone();
                c["memory_id"] = id.clone();
                expanded.push(c);
            }
        }
        batch["sources"] = json!(sources);
        batch["chunks"] = json!(expanded);
        let path = out.join(format!("prepared-{size}.json"));
        fs::write(
            &path,
            serde_json::to_vec(&batch).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        drop(batch);
        let receipt = session.call("publish_file", json!({"path":path}))?;
        let mut exact = Vec::new();
        let mut total = Vec::new();
        for _ in 0..3 {
            let found=session.call("search",json!({"index_id":receipt["response"]["index_id"],"search":{"query":"cobalt sample","scope_kind":"project","scope_id":"embedding-scale","as_of":"2026-10-24T12:00:00.000Z","limit":20},"deadline_ms":30000,"max_scored_chunks":size*2,"semantic_only":true}))?;
            if found["response"]["scored_chunks"] != size * 2 {
                return Err("scale scored denominator mismatch".into());
            }
            exact.push(found["response"]["exact_search_ms"].as_f64().unwrap());
            total.push(found["response"]["total_ms"].as_f64().unwrap());
        }
        results.push(json!({"memories":size,"chunks":size*2,"dimensions":384,"rounds":3,"exact_ms":exact,"exact_p95_ms":quantile(&exact,0.95),"total_ms":total,"total_p95_ms":quantile(&total,0.95),"publication_ms":receipt["elapsed_ms"]}));
    }
    session.input.take();
    let status = session.child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("embedding_scale_session_failed".into());
    }
    let report = json!({"profile":ready["profile"],"results":results,"limits":"Controlled repeated texts and real exported vectors; measures SQL/filter/normalization/cosine and snapshot verification, not semantic diversity or concurrent load. Total includes query encoding, excludes model load."});
    fs::write(
        out.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    Ok(())
}
