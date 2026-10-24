//! Bounded, append-only product campaign. A new directory is required; no retries.
use super::*;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub(super) struct Session {
    child: Child,
    input: Option<ChildStdin>,
    output: Receiver<Result<Value, String>>,
    reader: Option<JoinHandle<()>>,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
impl Session {
    pub(super) fn start(binary: &Path, database: &Path, dir: &Path) -> Result<Self, String> {
        let mut child = command(binary, database, dir)
            .arg("session")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| "no se pudo iniciar session")?;
        let input = child.stdin.take();
        let stdout = child.stdout.take().ok_or("sin stdout")?;
        let (sender, output) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                let mut bytes = vec![];
                let result = stdout
                    .by_ref()
                    .take(8_388_609)
                    .read_until(b'\n', &mut bytes);
                if bytes.is_empty() {
                    break;
                }
                let value = if result.is_err() || bytes.len() > 8_388_608 {
                    Err("respuesta de sesión inválida/excedida".into())
                } else {
                    serde_json::from_slice(&bytes).map_err(|_| "JSON de sesión inválido".into())
                };
                if sender.send(value).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            output,
            reader: Some(reader),
        })
    }
    pub(super) fn request(
        &mut self,
        operation: &str,
        request: &Value,
        timeout: Duration,
    ) -> Result<Value, String> {
        let input = self.input.as_mut().ok_or("sesión cerrada")?;
        serde_json::to_writer(&mut *input, &json!({"op":operation,"request":request}))
            .map_err(|_| "no se pudo escribir solicitud")?;
        input
            .write_all(b"\n")
            .and_then(|_| input.flush())
            .map_err(|_| "no se pudo enviar solicitud")?;
        let envelope = self
            .output
            .recv_timeout(timeout)
            .map_err(|_| "sesión interrumpida o sin respuesta; operación incierta")??;
        if envelope["status"] != "complete" {
            return Err("sesión fallida; operación incierta".into());
        }
        envelope
            .get("response")
            .cloned()
            .ok_or("sin respuesta; operación incierta".into())
    }
}
pub(super) fn command(binary: &Path, database: &Path, dir: &Path) -> Command {
    let mut command = Command::new(binary);
    command
        .env("GENNIE_MEMORY_DATABASE_PATH", database)
        .env(
            "GENNIE_MEMORY_EVALUATION_CAPTURE_PATH",
            dir.join("http.jsonl"),
        )
        .env(
            "GENNIE_MEMORY_EVALUATION_RESPONSE_CONTENT_PATH",
            dir.join("response-content.jsonl"),
        );
    command
}
pub(super) fn fresh(path: &Path) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(|e| e.to_string())
}
pub(super) fn append(file: &mut File, value: &Value) -> Result<(), String> {
    serde_json::to_writer(&mut *file, value).map_err(|e| e.to_string())?;
    file.write_all(b"\n")
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
pub(super) fn save(path: &Path, value: &Value) -> Result<(), String> {
    append(&mut fresh(path)?, value)
}

pub(super) fn run(suite_path: &Path, binary: &Path, output: &Path) -> Result<Value, String> {
    let structural = validate(suite_path)?;
    let suite: Suite = serde_json::from_slice(&read(suite_path)?).map_err(|e| e.to_string())?;
    let root = suite_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let binary = binary.canonicalize().map_err(|e| e.to_string())?;
    let policy: Value = serde_json::from_slice(&read(&root.join(&suite.policy_path))?)
        .map_err(|e| e.to_string())?;
    let binding = binding::Binding::load(
        &root,
        suite
            .corpus
            .as_ref()
            .ok_or("campaña requiere corpus ligado")?,
        policy.clone(),
    )?;
    let evidence_path = root.join("provider-verification.json");
    let evidence: Value =
        serde_json::from_slice(&read(&evidence_path)?).map_err(|e| e.to_string())?;
    if evidence["campaign_model"] != policy["provider"]
        || evidence["input_usd_per_million"]
            .as_f64()
            .is_none_or(|c| !c.is_finite() || c <= 0.0)
        || evidence["output_usd_per_million"] != 0
        || evidence["key_check"] != "accepted"
        || evidence["budget_at_least_campaign_limit"] != true
    {
        return Err("falta verificación de precio/modelo/acceso/presupuesto".into());
    }
    let a = root.join("review-a-v2.json");
    let b = root.join("review-b-v2.json");
    let packet = root.join("review-packet.jsonl");
    let rubric = root.join("review-rubric.md");
    let review = audit_reviews(&[suite_path, &packet, &rubric, &a, &b])?;
    if review["label_agreement"] != true {
        return Err("etiquetas con desacuerdo".into());
    }
    // Reserve is an accounting guard, not an upstream billing guarantee.
    let reserve = evidence["per_request_reserve_usd"]
        .as_f64()
        .filter(|c| c.is_finite() && *c > 0.0)
        .ok_or("reserva inválida")?;
    if suite.protocol.max_cost_usd.is_none_or(|c| reserve > c) {
        return Err("campaña requiere tope monetario finito con reserva".into());
    }
    std::fs::create_dir(output).map_err(|e| format!("se requiere directorio nuevo: {e}"))?;
    let output = output.canonicalize().map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    let mut journal = fresh(&output.join("journal.jsonl"))?;
    append(
        &mut journal,
        &json!({"kind":"manifest","suite_sha256":structural["suite_sha256"],"policy_sha256":suite.policy_sha256,
        "binary_sha256":crate::campaign::digest(&binary)?,"provider_evidence_sha256":digest(&read(&evidence_path)?),"review":review,
        "protocol":{"max_calls":suite.protocol.max_calls,"max_seconds":suite.protocol.max_seconds,"max_cost_usd":suite.protocol.max_cost_usd},"per_request_reserve_usd":reserve,"automatic_retries":false}),
    )?;
    let start = Instant::now();
    let deadline = start + Duration::from_secs(suite.protocol.max_seconds);
    let mut reports = vec![];
    let mut calls = 0_u64;
    let mut known_cost = 0.0;
    let mut final_opened = false;
    let mut pending_remote = false;
    let result = (|| {
        let database = output.join("corpus.db");
        // Loading fixtures does not instantiate the HTTP classifier/capture files.
        let corpus = root.join(&suite.corpus.as_ref().unwrap().path);
        let status = command(&binary, &database, &output)
            .arg("load-fixtures")
            .arg(corpus)
            .stdout(Stdio::null())
            .status()
            .map_err(|_| "no se pudo cargar corpus")?;
        if !status.success() {
            return Err("el producto rechazó el corpus".into());
        }
        let mut session = Session::start(&binary, &database, &output)?;
        for split in ["development", "calibration", "final"] {
            let part = suite
                .partitions
                .iter()
                .find(|p| p.split == split)
                .ok_or("partición ausente")?;
            let raw = read(&root.join(&part.path))?;
            let mut observations = vec![];
            for line in std::str::from_utf8(&raw)
                .map_err(|_| "casos no UTF-8")?
                .lines()
                .filter(|l| !l.trim().is_empty())
            {
                let case: Case = serde_json::from_str(line).map_err(|e| e.to_string())?;
                if Instant::now() >= deadline {
                    return Err("presupuesto de tiempo agotado".into());
                }
                if split == "final" {
                    final_opened = true;
                }
                // Search is local and label-free; only reserve when runtime can call.
                let needs_clarification = case.request["unresolved_references"]
                    .as_array()
                    .is_some_and(|a| !a.is_empty());
                let may_call = if needs_clarification {
                    false
                } else {
                    let search = session.request(
                        "search",
                        &case.request["search"],
                        deadline
                            .saturating_duration_since(Instant::now())
                            .min(Duration::from_secs(30)),
                    )?;
                    let candidates = search["candidates"]
                        .as_array()
                        .ok_or("preflight local sin candidatos")?;
                    append(
                        &mut journal,
                        &json!({"kind":"local_search","split":split,"id":case.id,"response":search}),
                    )?;
                    !candidates.is_empty()
                };
                if may_call
                    && (calls >= suite.protocol.max_calls
                        || known_cost + reserve > suite.protocol.max_cost_usd.unwrap())
                {
                    return Err("presupuesto de llamadas/coste insuficiente".into());
                }
                append(
                    &mut journal,
                    &json!({"kind":"started","split":split,"id":case.id,"sequence":observations.len(),"request_sha256":digest(&serde_json::to_vec(&case.request).map_err(|e|e.to_string())?),"elapsed_ms":start.elapsed().as_secs_f64()*1000.0}),
                )?;
                pending_remote = may_call;
                if may_call {
                    calls += 1;
                }
                let response = session.request(
                    "recall_v1",
                    &case.request,
                    deadline
                        .saturating_duration_since(Instant::now())
                        .min(Duration::from_secs(30)),
                )?;
                append(
                    &mut journal,
                    &json!({"kind":"finished","split":split,"id":case.id,"response":response,"elapsed_ms":start.elapsed().as_secs_f64()*1000.0}),
                )?;
                let usage = &response["usage"];
                if usage["receipt_status"] != "not_called" {
                    if !may_call {
                        return Err("clasificación inesperada tras preflight local".into());
                    }
                    let cost = usage["reported_cost_usd"]
                        .as_f64()
                        .filter(|c| c.is_finite() && *c >= 0.0)
                        .ok_or("consumo desconocido: se detiene sin reintento")?;
                    known_cost += cost;
                    if usage["receipt_status"] != "complete"
                        || cost > reserve
                        || known_cost > suite.protocol.max_cost_usd.unwrap()
                    {
                        return Err("recibo incompleto o presupuesto excedido".into());
                    }
                }
                if usage["receipt_status"] == "not_called" && may_call {
                    calls -= 1;
                }
                pending_remote = false;
                observations.push(grade(&case, &response, &binding.memories)?);
            }
            let report = summarize(split, &observations, &suite.protocol);
            save(&output.join(format!("{split}-report.json")), &report)?;
            let passed = report["gate_passed"] == true;
            reports.push(report);
            if !passed {
                return Err(format!(
                    "puerta de {split} incumplida; se conservan resultados y se detiene avance"
                ));
            }
        }
        Ok::<(), String>(())
    })();
    let summary = json!({"status":if result.is_ok(){"completed"}else{"stopped"},"stop_reason":result.err(),"reports":reports,
        "remote_calls_with_receipt_or_attempt":calls,"known_cost_usd":known_cost,"total_cost_usd":if pending_remote{None}else{Some(known_cost)},"cost_complete":!pending_remote,"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,
        "final_opened":final_opened,"final_completed":reports.iter().any(|r|r["split"]=="final"),"human_validation":false,"product_adoption":"not_implied"});
    append(
        &mut journal,
        &json!({"kind":"campaign_terminal","summary":summary}),
    )?;
    save(&output.join("summary.json"), &summary)?;
    Ok(summary)
}

pub(super) fn grade(case: &Case, response: &Value, corpus: &[Value]) -> Result<Value, String> {
    let assessments = response["assessments"]
        .as_array()
        .ok_or("respuesta sin assessments")?;
    let actual: HashSet<_> = assessments
        .iter()
        .filter(|a| a["included_in_seed"] == true)
        .filter_map(|a| a["memory_id"].as_str())
        .collect();
    let expected: HashSet<_> = case
        .expected_evidence
        .iter()
        .map(|e| e.memory_id.as_str())
        .collect();
    let found = expected.intersection(&actual).count();
    let role_matches = case.expected_evidence.iter().all(|e| {
        assessments.iter().any(|a| {
            a["memory_id"] == e.memory_id
                && a["revision"].as_u64() == Some(e.revision)
                && a["relation"] == e.relation
                && a["included_in_seed"] == true
        })
    });
    let mut seed_valid = true;
    if let Some(memories) = response.pointer("/seed/memories").and_then(Value::as_array) {
        let mut seed_ids = HashSet::new();
        let mut rendered = Vec::new();
        for m in memories {
            let id = m["memory_id"].as_str().ok_or("semilla sin id")?;
            let mut text = format!(
                "[{}@{}] {}",
                id,
                m["revision"].as_u64().ok_or("revisión inválida")?,
                m["summary"].as_str().ok_or("resumen ausente")?
            );
            if let Some(conditions) = m["conditions"].as_str() {
                text.push_str(&format!("\n  Condiciones: {conditions}"));
            }
            if let Some(source) = m["source"].as_object() {
                text.push_str(&format!(
                    "\n  Fuente declarada: {} / {}",
                    source["kind"].as_str().ok_or("fuente inválida")?,
                    source["ref"].as_str().ok_or("referencia inválida")?
                ));
                if let Some(locator) = source.get("locator").and_then(Value::as_str) {
                    text.push_str(&format!(" / {locator}"));
                }
            }
            rendered.push(text);
            seed_valid &= seed_ids.insert(id)
                && actual.contains(id)
                && corpus.iter().any(|source| {
                    source["id"] == id
                        && [
                            "revision",
                            "scope_kind",
                            "scope_id",
                            "summary",
                            "conditions",
                            "source",
                            "state",
                        ]
                        .iter()
                        .all(|k| source[*k] == m[*k])
                });
        }
        seed_valid &= seed_ids == actual;
        let expected_text = if rendered.is_empty() {
            Value::Null
        } else {
            json!(rendered.join("\n"))
        };
        seed_valid &= response["seed"]["seed_text"] == expected_text;
    } else {
        seed_valid &= actual.is_empty();
    }
    let total = response["total_ms"]
        .as_f64()
        .filter(|v| v.is_finite() && *v >= 0.0)
        .ok_or("duración inválida")?;
    Ok(
        json!({"id":case.id,"expected_outcome":case.expected_outcome,"actual_outcome":response["outcome"],
        "execution_status":response["execution_status"],"expected_evidence":expected.len(),"useful_selected":found,
        "unrelated_selections":actual.difference(&expected).count(),"outcome_matches":response["outcome"]==case.expected_outcome,
        "relations_match":role_matches,"seed_faithful":seed_valid,"seed_bytes":response["seed"]["seed_text"].as_str().map_or(0,str::len),"total_ms":total,"usage":response["usage"],"issues":response["issues"]}),
    )
}
pub(super) fn summarize(split: &str, rows: &[Value], p: &Protocol) -> Value {
    let sum = |key: &str| {
        rows.iter()
            .map(|r| r[key].as_u64().unwrap_or(0))
            .sum::<u64>()
    };
    let expected = sum("expected_evidence");
    let found = sum("useful_selected");
    let recall = if expected == 0 {
        1.0
    } else {
        found as f64 / expected as f64
    };
    let degraded = rows
        .iter()
        .filter(|r| r["execution_status"] != "completed")
        .count() as u64;
    let mut times: Vec<_> = rows.iter().filter_map(|r| r["total_ms"].as_f64()).collect();
    times.sort_by(f64::total_cmp);
    let percentile = |q: f64| {
        if times.is_empty() {
            0.0
        } else {
            times[((times.len() as f64 * q).ceil() as usize).saturating_sub(1)]
        }
    };
    let gate = !rows.is_empty()
        && recall >= p.min_useful_recall
        && sum("unrelated_selections") <= p.max_unrelated_selections
        && degraded <= p.max_degradations
        && percentile(0.95) <= p.max_p95_ms
        && rows.iter().all(|r| {
            r["outcome_matches"] == true
                && r["relations_match"] == true
                && r["seed_faithful"] == true
        });
    json!({"split":split,"cases":rows.len(),"useful_recall":recall,"unrelated_selections":sum("unrelated_selections"),"degradations":degraded,"p50_ms":percentile(0.5),"p95_ms":percentile(0.95),"percentile_method":"nearest_rank","gate_passed":gate,"observations":rows})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn altered_seed_text_fails_even_when_memory_metadata_matches() {
        let case: Case = serde_json::from_value(json!({"id":"c","source_groups":["s"],"entity_groups":["e"],"family_groups":["f"],"request":{},"expected_outcome":"evidence_selected","expected_evidence":[{"memory_id":"m","revision":1,"relation":"supports_answer"}]})).unwrap();
        let source = json!({"id":"m","revision":1,"scope_kind":"project","scope_id":"p","summary":"Texto fiel.","conditions":null,"source":null,"state":"active"});
        let mut memory = source.clone();
        memory["memory_id"] = json!("m");
        let mut response = json!({"assessments":[{"memory_id":"m","revision":1,"relation":"supports_answer","included_in_seed":true}],"seed":{"memories":[memory],"seed_text":"[m@1] Texto fiel."},"total_ms":1.0,"execution_status":"completed","outcome":"evidence_selected"});
        assert_eq!(
            grade(&case, &response, &[source.clone()]).unwrap()["seed_faithful"],
            true
        );
        response["seed"]["seed_text"] = json!("[m@1] Texto alterado.");
        assert_eq!(
            grade(&case, &response, &[source]).unwrap()["seed_faithful"],
            false
        );
    }
    #[test]
    fn gates_reject_wrong_roles_foreign_evidence_and_technical_abstentions() {
        let p = Protocol {
            max_calls: 12,
            max_seconds: 300,
            max_cost_usd: Some(0.1),
            unlimited_cost_authorized: false,
            min_useful_recall: 1.0,
            max_unrelated_selections: 0,
            max_degradations: 0,
            max_p95_ms: 1500.0,
            label_source: "blind_synthetic_adjudication".into(),
        };
        let good = json!({"expected_evidence":1,"useful_selected":1,"unrelated_selections":0,"execution_status":"completed","outcome_matches":true,"relations_match":true,"seed_faithful":true,"total_ms":10.0});
        assert_eq!(
            summarize("development", &[good.clone()], &p)["gate_passed"],
            true
        );
        for (key, value) in [
            ("relations_match", json!(false)),
            ("unrelated_selections", json!(1)),
            ("execution_status", json!("degraded")),
            ("seed_faithful", json!(false)),
            ("total_ms", json!(1501)),
        ] {
            let mut row = good.clone();
            row[key] = value;
            assert_eq!(
                summarize("development", &[row], &p)["gate_passed"],
                false,
                "{key}"
            );
        }
    }
}

/// Recompute gates from the immutable journal, including the exact emitted seed.
pub(super) fn audit(suite_path: &Path, binary: &Path, dir: &Path) -> Result<Value, String> {
    let validation = validate(suite_path)?;
    let suite: Suite = serde_json::from_slice(&read(suite_path)?).map_err(|e| e.to_string())?;
    let root = suite_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let binding = binding::Binding::load(
        root,
        suite.corpus.as_ref().ok_or("falta corpus")?,
        serde_json::from_slice(&read(&root.join(&suite.policy_path))?)
            .map_err(|e| e.to_string())?,
    )?;
    let journal_bytes = read(&dir.join("journal.jsonl"))?;
    let rows: Vec<Value> = std::str::from_utf8(&journal_bytes)
        .map_err(|_| "diario no UTF-8")?
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    let manifest = rows.first().ok_or("diario vacío")?;
    if manifest["kind"] != "manifest"
        || manifest["suite_sha256"] != validation["suite_sha256"]
        || manifest["binary_sha256"] != crate::campaign::digest(binary)?
    {
        return Err("diario ligado a otra suite o binario".into());
    }
    let mut pending = None;
    let mut finished = HashMap::new();
    let mut seen = HashSet::new();
    let mut terminal = false;
    for row in rows.iter().skip(1) {
        if terminal {
            return Err("eventos después del terminal".into());
        }
        match row["kind"].as_str() {
            Some("local_search") if pending.is_none() => {}
            Some("started") if pending.is_none() => {
                let id = row["id"].as_str().ok_or("inicio sin ID")?.to_string();
                if !seen.insert(id.clone()) {
                    return Err("inicio duplicado".into());
                }
                pending = Some((id, row["split"].clone(), row["request_sha256"].clone()));
            }
            Some("finished") => {
                let (id, split, hash) = pending.take().ok_or("terminal sin inicio")?;
                if row["id"] != id || row["split"] != split {
                    return Err("terminal no corresponde a inicio".into());
                }
                finished.insert(id, (split, hash, row["response"].clone()));
            }
            Some("campaign_terminal") if pending.is_none() => {
                terminal = true;
            }
            _ => return Err("secuencia inválida o llamada incierta".into()),
        }
    }
    if !terminal || pending.is_some() {
        return Err("diario interrumpido".into());
    }
    let mut reports = vec![];
    let mut cost = 0.0;
    let mut calls = 0;
    for split in ["development", "calibration", "final"] {
        if reports.iter().any(|r: &Value| r["gate_passed"] != true)
            && finished.values().any(|(s, _, _)| s == split)
        {
            return Err("se abrió una reserva tras fallar puerta".into());
        }
        let partition = suite
            .partitions
            .iter()
            .find(|p| p.split == split)
            .ok_or("sin partición")?;
        let bytes = read(&root.join(&partition.path))?;
        let mut observations = vec![];
        for line in std::str::from_utf8(&bytes)
            .map_err(|_| "partición no UTF-8")?
            .lines()
            .filter(|l| !l.trim().is_empty())
        {
            let case: Case = serde_json::from_str(line).map_err(|e| e.to_string())?;
            if let Some((recorded_split, hash, response)) = finished.remove(&case.id) {
                if recorded_split != split
                    || hash
                        != digest(&serde_json::to_vec(&case.request).map_err(|e| e.to_string())?)
                {
                    return Err("solicitud o partición alterada".into());
                }
                if response["usage"]["receipt_status"] != "not_called" {
                    calls += 1;
                    if response["usage"]["receipt_status"] != "complete" {
                        return Err("recibo incompleto".into());
                    }
                    cost += response["usage"]["reported_cost_usd"]
                        .as_f64()
                        .filter(|v| v.is_finite() && *v >= 0.0)
                        .ok_or("coste inválido")?;
                }
                observations.push(grade(&case, &response, &binding.memories)?);
            }
        }
        if !observations.is_empty() {
            if observations.len() != partition.count {
                return Err("partición incompleta".into());
            }
            reports.push(summarize(split, &observations, &suite.protocol));
        }
    }
    if !finished.is_empty() {
        return Err("casos ajenos al banco".into());
    }
    Ok(
        json!({"journal_sha256":digest(&journal_bytes),"suite_sha256":validation["suite_sha256"],"reports":reports,
        "all_three_gates_passed":reports.len()==3&&reports.iter().all(|r|r["gate_passed"]==true),
        "calls":calls,"reported_cost_usd":cost,"exact_seed_text_checked":true,"network_calls_for_audit":0}),
    )
}
