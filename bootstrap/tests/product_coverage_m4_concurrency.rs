//! Explicit bounded local observation, never a default stress/CI campaign.
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Barrier, mpsc},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
struct Sandbox(PathBuf);
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
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
        }
        let _ = self.child.wait();
    }
}
impl Session {
    fn open(s: &Sandbox) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_gennie-memory"))
            .arg("session")
            .current_dir(&s.0)
            .env("GENNIE_MEMORY_DATABASE_PATH", s.0.join("memory.db"))
            .env("GENNIE_MEMORY_OPENROUTER_API_KEY", "")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        Self {
            input: child.stdin.take(),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
        }
    }
    fn call(&mut self, op: &str, request: Value) -> Value {
        writeln!(
            self.input.as_mut().unwrap(),
            "{}",
            json!({"op":op,"request":request})
        )
        .unwrap();
        self.input.as_mut().unwrap().flush().unwrap();
        let mut line = String::new();
        assert!(
            self.output.read_line(&mut line).unwrap() > 0,
            "session terminated"
        );
        serde_json::from_str(&line).unwrap()
    }
}
fn request(scope: &str, query: &str) -> Value {
    json!({"query":query,"scope_kind":"project","scope_id":scope,"actor_id":"local","as_of":"2026-10-24T12:00:00.000Z","limit":20})
}
fn write(scope: &str, worker: usize, step: usize, id: Option<&str>) -> Value {
    let mut v = json!({"schema_version":1,"operation_id":format!("cccccccc-0000-4000-8000-{worker:04}{step:08}"),"action":if id.is_some(){"revise"}else{"create"},"scope":{"kind":"project","id":scope},"memory":{"type":"fact","original_text":format!("m4marker owner{worker} step{step}"),"summary":format!("m4marker owner{worker} step{step}"),"source":{"kind":"document","ref":"synthetic:m4-concurrency"}}});
    if let Some(id) = id {
        v["memory_id"] = json!(id);
        v["expected_revision"] = json!(step);
    }
    v
}
fn observe(
    s: &mut Session,
    op: &str,
    r: Value,
    worker: &str,
    start: Instant,
    tx: &mpsc::Sender<Value>,
) -> Value {
    let begin = start.elapsed().as_secs_f64() * 1000.;
    tx.send(json!({"event":"started","worker":worker,"op":op,"start_ms":begin}))
        .unwrap();
    let v = s.call(op, r);
    let end = start.elapsed().as_secs_f64() * 1000.;
    tx.send(json!({"event":"finished","worker":worker,"op":op,"start_ms":begin,"end_ms":end,"latency_ms":end-begin,"status":v["status"]})).unwrap();
    assert_eq!(v["status"], "complete", "{v}");
    v["response"].clone()
}
fn pace(start: Instant, index: usize, period_ms: u64) {
    let deadline = start + Duration::from_millis(index as u64 * period_ms);
    if let Some(left) = deadline.checked_duration_since(Instant::now()) {
        thread::sleep(left);
    }
}
fn percentile(values: &mut [f64], q: f64) -> f64 {
    values.sort_by(f64::total_cmp);
    values[((values.len() as f64 * q).ceil() as usize)
        .saturating_sub(1)
        .min(values.len() - 1)]
}
#[test]
#[ignore = "explicit 30-second bounded writer/reader/maintenance campaign; see protocol"]
fn writers_readers_and_lifecycle_overlap_with_measured_contention_for_thirty_seconds() {
    let root = std::env::temp_dir().join(format!(
        "gennie-m4-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let sandbox = Arc::new(Sandbox(root));
    let mut seed = Session::open(&sandbox);
    let seeded = seed.call("mem_write", write("maintenance", 9, 0, None));
    assert_eq!(seeded["status"], "complete", "{seeded}");
    let control_id = seeded["response"]["memory_id"].as_str().unwrap().to_owned();
    drop(seed);
    let database_bytes_before = fs::metadata(sandbox.0.join("memory.db")).unwrap().len();
    let barrier = Arc::new(Barrier::new(6));
    let start = Instant::now();
    let (tx, rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();
    let mut workers = vec![];
    for worker in 0..5 {
        let sandbox = Arc::clone(&sandbox);
        let barrier = Arc::clone(&barrier);
        let tx = tx.clone();
        let ready = ready_tx.clone();
        let control_id = control_id.clone();
        workers.push(thread::spawn(move||{
        let name=if worker<2{format!("writer-{worker}")}else if worker<4{format!("reader-{worker}")}else{"maintenance".into()};
        let mut session=Session::open(&sandbox);assert_eq!(session.call("search",request("empty","m4marker"))["status"],"complete");ready.send((name.clone(),session.child.id())).unwrap();barrier.wait();let workload_start=Instant::now();tx.send(json!({"event":"workload_started","worker":name,"at_ms":start.elapsed().as_secs_f64()*1000.})).unwrap();
        if worker<2 {
            let scope=format!("writer-{worker}");let mut id=None;
            for step in 0..120 {pace(workload_start,step,250);let r=observe(&mut session,"mem_write",write(&scope,worker,step,id.as_deref()),&name,start,&tx);if id.is_none(){id=Some(r["memory_id"].as_str().unwrap().to_owned());}assert_eq!(r["revision"],step+1);}
        }else if worker<4 {
            for step in 0..240 {pace(workload_start,step,125);let out=observe(&mut session,"search",request(&format!("writer-{}",worker-2),"m4marker"),&name,start,&tx);let cs=out["candidates"].as_array().unwrap();assert!(cs.len()<=1);for c in cs {assert!(c["summary"].as_str().unwrap().contains(&format!("owner{}",worker-2)));}assert_eq!(out["coverage"]["retrieval"],"complete");}
        }else {
            let mut revision=1;
            for step in 0..60 {pace(workload_start,step,500);
                for action in ["withdraw","restore"] {let r=json!({"schema_version":1,"operation_id":format!("dddddddd-0000-4000-8000-{revision:012}"),"action":action,"memory_id":control_id,"expected_revision":revision,"scope":{"kind":"project","id":"maintenance"},"as_of":"2026-10-24T12:00:00.000Z","next_review_at":null,"evidence":{"original_text":"Synthetic bounded maintenance requested by operator.","source":{"kind":"user","ref":"synthetic:m4-maintenance"}}});let out=observe(&mut session,"mem_lifecycle",r,&name,start,&tx);revision+=1;assert_eq!(out["revision"],revision);}
                for action in ["enqueue","cancel"] {observe(&mut session,"embedding_job",json!({"action":action,"scope":{"kind":"project","id":"maintenance"},"space_id":"m4-no-encoder"}),&name,start,&tx);}
            }
        }
        pace(workload_start,1,30000);
        tx.send(json!({"event":"workload_finished","worker":name,"at_ms":start.elapsed().as_secs_f64()*1000.,"workload_ms":workload_start.elapsed().as_secs_f64()*1000.})).unwrap();
    }));
    }
    drop(tx);
    drop(ready_tx);
    let mut own_processes = vec![];
    for _ in 0..5 {
        own_processes.push(ready_rx.recv_timeout(Duration::from_secs(10)).unwrap());
    }
    let sample_running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let sample_flag = Arc::clone(&sample_running);
    let process_ids = own_processes
        .iter()
        .map(|(_, pid)| pid.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let sampler = thread::spawn(move || {
        let mut samples = vec![];
        while sample_flag.load(std::sync::atomic::Ordering::Relaxed) {
            let out = Command::new("ps")
                .args(["-p", &process_ids, "-o", "pid=,rss=,%cpu="])
                .output()
                .unwrap();
            let mut processes = vec![];
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let fields: Vec<_> = line.split_whitespace().collect();
                if fields.len() == 3 {
                    processes.push(json!({"pid":fields[0].parse::<u32>().unwrap(),"rss_kib":fields[1].parse::<u64>().unwrap(),"cpu_lifetime_percent":fields[2].parse::<f64>().unwrap()}));
                }
            }
            samples
                .push(json!({"at_ms":start.elapsed().as_secs_f64()*1000.,"processes":processes}));
            for _ in 0..10 {
                if !sample_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
        samples
    });
    let lock = Connection::open(sandbox.0.join("memory.db")).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let lock_begin = start.elapsed().as_secs_f64() * 1000.;
    barrier.wait();
    // Observe submitted writer operations while another process owns the writer
    // lock. A successful reader under the same lock demonstrates real overlap.
    let probe_deadline = Instant::now() + Duration::from_millis(300);
    let mut events = vec![];
    while Instant::now() < probe_deadline {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(10)) {
            events.push(event);
        }
    }
    let before_release = start.elapsed().as_secs_f64() * 1000.;
    for writer in ["writer-0", "writer-1", "maintenance"] {
        assert!(
            events
                .iter()
                .any(|e| e["worker"] == writer && e["event"] == "started")
        );
        assert!(
            !events
                .iter()
                .any(|e| e["worker"] == writer && e["event"] == "finished")
        );
    }
    assert!(
        events
            .iter()
            .any(|e| e["worker"].as_str().unwrap().starts_with("reader")
                && e["event"] == "finished"
                && e["status"] == "complete")
    );
    lock.execute_batch("ROLLBACK").unwrap();
    for worker in workers {
        worker.join().unwrap();
    }
    sample_running.store(false, std::sync::atomic::Ordering::Relaxed);
    let resource_samples = sampler.join().unwrap();
    assert!(resource_samples.len() >= 20);
    events.extend(rx.try_iter());
    let finished: Vec<_> = events.iter().filter(|e| e["event"] == "finished").collect();
    assert_eq!(finished.len(), 960);
    let mut by_worker = serde_json::Map::new();
    for name in [
        "writer-0",
        "writer-1",
        "reader-2",
        "reader-3",
        "maintenance",
    ] {
        let own: Vec<_> = finished.iter().filter(|e| e["worker"] == name).collect();
        let mut times: Vec<_> = own
            .iter()
            .map(|e| e["latency_ms"].as_f64().unwrap())
            .collect();
        by_worker.insert(name.into(),json!({"operations":own.len(),"p50_ms":percentile(&mut times,0.5),"p95_ms":percentile(&mut times,0.95),"max_ms":percentile(&mut times,1.0)}));
    }
    let mut final_session = Session::open(&sandbox);
    for writer in 0..2 {
        let out = final_session.call("search", request(&format!("writer-{writer}"), "m4marker"));
        assert_eq!(out["response"]["candidates"][0]["revision"], 120);
    }
    let integrity: String = lock
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    let foreign_keys: i64 = lock
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(foreign_keys, 0);
    let report = json!({"schema_version":1,"database_bytes_before":database_bytes_before,"database_bytes_after":fs::metadata(sandbox.0.join("memory.db")).unwrap().len(),"resource_samples":resource_samples,"processes":own_processes,"duration_ms":start.elapsed().as_secs_f64()*1000.,"workers":by_worker,"completed_operations":finished.len(),"writer_lock":{"begin_ms":lock_begin,"release_ms":before_release,"both_writers_submitted_and_pending":true,"read_completed_during_lock":true},"final_writer_revision":120,"integrity_check":integrity,"foreign_key_violations":foreign_keys,"events":events,"limits":"30-second bounded synthetic local observation; paced workloads, no encoder/provider, RSS samples in KiB and ps lifetime CPU percentage at ~1Hz for owned child PIDs; CPU is not per-operation accounting or months-long stability proof."});
    if let Ok(path) = std::env::var("GENNIE_M4_OBSERVATION_REPORT") {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .unwrap();
        file.write_all(serde_json::to_string_pretty(&report).unwrap().as_bytes())
            .unwrap();
        file.sync_all().unwrap();
    } else {
        eprintln!(
            "{}",
            json!({"completed_operations":finished.len(),"workers":report["workers"],"duration_ms":report["duration_ms"]})
        );
    }
}

#[test]
fn a_cli_writer_waits_for_an_existing_reserved_lock_then_commits_once() {
    let path = std::env::temp_dir().join(format!(
        "gennie-m4-lock-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    let sandbox = Arc::new(Sandbox(path));
    let mut session = Session::open(&sandbox);
    assert_eq!(
        session.call("search", request("lock", "m4marker"))["status"],
        "complete"
    );
    let lock = Connection::open(sandbox.0.join("memory.db")).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let (start_tx, start_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        start_tx.send(()).unwrap();
        let result = session.call("mem_write", write("lock", 8, 0, None));
        done_tx.send(result).unwrap();
    });
    start_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let early = done_rx.recv_timeout(Duration::from_millis(150));
    lock.execute_batch("ROLLBACK").unwrap();
    let completed = match &early {
        Ok(v) => v.clone(),
        Err(_) => done_rx.recv_timeout(Duration::from_secs(6)).unwrap(),
    };
    worker.join().unwrap();
    assert!(
        early.is_err(),
        "writer completed before reserved lock released: {completed}"
    );
    assert_eq!(completed["status"], "complete", "{completed}");
    let n: i64 = lock
        .query_row(
            "SELECT count(*) FROM memories WHERE scope_id='lock'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 1);
}

fn assert_maintenance_waits(op: &str, job: bool) {
    let path = std::env::temp_dir().join(format!(
        "gennie-m4-maintenance-lock-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    let sandbox = Arc::new(Sandbox(path));
    let mut session = Session::open(&sandbox);
    let saved = session.call("mem_write", write("lock", 8, 0, None));
    assert_eq!(saved["status"], "complete");
    let request = if job {
        json!({"action":"enqueue","scope":{"kind":"project","id":"lock"},"space_id":"m4-no-encoder"})
    } else {
        json!({"schema_version":1,"operation_id":"bbbbbbbb-0000-4000-8000-000000000001","action":"withdraw","memory_id":saved["response"]["memory_id"],"expected_revision":1,"scope":{"kind":"project","id":"lock"},"as_of":"2026-10-24T12:00:00.000Z","evidence":{"original_text":"Synthetic withdrawal requested by the operator.","source":{"kind":"user","ref":"synthetic:m4-lock"}}})
    };
    let lock = Connection::open(sandbox.0.join("memory.db")).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let (start_tx, start_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let op = op.to_owned();
    let worker = thread::spawn(move || {
        start_tx.send(()).unwrap();
        let result = session.call(&op, request);
        done_tx.send(result).unwrap();
    });
    start_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let early = done_rx.recv_timeout(Duration::from_millis(150));
    lock.execute_batch("ROLLBACK").unwrap();
    let completed = match &early {
        Ok(v) => v.clone(),
        Err(_) => done_rx.recv_timeout(Duration::from_secs(6)).unwrap(),
    };
    worker.join().unwrap();
    assert!(
        early.is_err(),
        "maintenance completed before reserved lock released: {completed}"
    );
    assert_eq!(completed["status"], "complete", "{completed}");
}
#[test]
fn lifecycle_transition_waits_for_reserved_lock() {
    assert_maintenance_waits("mem_lifecycle", false);
}
#[test]
fn embedding_job_waits_for_reserved_lock() {
    assert_maintenance_waits("embedding_job", true);
}
