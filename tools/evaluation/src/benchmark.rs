//! Structural and label validation for versioned synthetic benchmark snapshots.

use crate::campaign::digest;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

fn property<'a>(value: &'a Value, name: &str) -> Result<&'a Value, String> {
    value.get(name).ok_or_else(|| format!("falta {name}"))
}

fn number(value: &Value, name: &str) -> Result<u64, String> {
    property(value, name)?
        .as_u64()
        .ok_or_else(|| format!("{name} no es entero"))
}

fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    property(value, name)?
        .as_str()
        .ok_or_else(|| format!("{name} no es texto"))
}

fn array<'a>(value: &'a Value, name: &str) -> Result<&'a Vec<Value>, String> {
    property(value, name)?
        .as_array()
        .ok_or_else(|| format!("{name} no es lista"))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

fn read_jsonl(path: &Path) -> Result<Vec<Value>, String> {
    let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    BufReader::new(file)
        .lines()
        .enumerate()
        .filter_map(|(index, line)| match line {
            Ok(line) if line.trim().is_empty() => None,
            other => Some((index, other)),
        })
        .map(|(index, line)| {
            let line = line.map_err(|error| error.to_string())?;
            serde_json::from_str(&line)
                .map_err(|error| format!("{} línea {}: {error}", path.display(), index + 1))
        })
        .collect()
}

pub fn verify_hashes(root: &Path) -> Result<(), String> {
    let manifest = read_json(&root.join("manifest.json"))?;
    let hashes = property(&manifest, "files_sha256")?
        .as_object()
        .ok_or("files_sha256 inválido")?;
    if hashes.is_empty() {
        return Err("no hay archivos para verificar".into());
    }
    for (name, expected) in hashes {
        if name.contains('/') || name.contains('\\') || name.starts_with('.') {
            return Err(format!("nombre de archivo inválido: {name}"));
        }
        let actual = digest(&root.join(name))?;
        if expected.as_str() != Some(actual.as_str()) {
            return Err(format!("hash distinto: {name}"));
        }
    }
    println!("{}", json!({"valid":true,"files":hashes.len()}));
    Ok(())
}

pub fn validate_general_memory(root: &Path) -> Result<(), String> {
    let manifest = read_json(&root.join("manifest.json"))?;
    if string(&manifest, "version")? != "general-memory-v1"
        || property(&manifest, "synthetic")?.as_bool() != Some(true)
    {
        return Err("manifiesto general-memory-v1 inválido".into());
    }
    let memories = read_json(&root.join("memories.json"))?;
    let memories = memories.as_array().ok_or("memories no es lista")?;
    if memories.len() as u64 != number(&manifest, "memories")? {
        return Err("conteo de recuerdos distinto".into());
    }
    let mut by_id = HashMap::new();
    let mut by_scope: HashMap<String, u64> = HashMap::new();
    let mut by_type: HashMap<String, u64> = HashMap::new();
    for memory in memories {
        let id = string(memory, "id")?;
        if by_id.insert(id.to_owned(), memory).is_some() {
            return Err(format!("recuerdo duplicado: {id}"));
        }
        if string(memory, "state")? != "active" {
            return Err(format!("recuerdo no activo: {id}"));
        }
        *by_scope
            .entry(string(memory, "scope_kind")?.to_owned())
            .or_default() += 1;
        *by_type
            .entry(string(memory, "memory_type")?.to_owned())
            .or_default() += 1;
    }
    for (field, actual) in [
        ("memories_by_scope", by_scope),
        ("memories_by_type", by_type),
    ] {
        let declared = property(&manifest, field)?
            .as_object()
            .ok_or(format!("{field} inválido"))?;
        if actual.len() != declared.len() {
            return Err(format!("{field} tiene categorías diferentes"));
        }
        for (key, value) in declared {
            if actual.get(key).copied() != value.as_u64() {
                return Err(format!("{field}.{key} distinto"));
            }
        }
    }
    let queries = read_jsonl(&root.join("queries.jsonl"))?;
    let unscoped = read_jsonl(&root.join("unscoped-queries.jsonl"))?;
    if queries.len() as u64 != number(&manifest, "queries")?
        || unscoped.len() as u64 != number(&manifest, "unscoped_queries")?
    {
        return Err("conteo de consultas distinto".into());
    }
    let mut seen = HashSet::new();
    let mut positive = 0u64;
    for query in &queries {
        let id = string(query, "case_id")?;
        if !seen.insert(id.to_owned()) {
            return Err(format!("consulta duplicada: {id}"));
        }
        let expected = array(query, "expected_memory_ids")?;
        let label = string(query, "label")?;
        if expected.is_empty() {
            if label != "negative" {
                return Err(format!("etiqueta incorrecta: {id}"));
            }
        } else {
            positive += 1;
            if label != "positive" {
                return Err(format!("etiqueta incorrecta: {id}"));
            }
        }
        for item in expected {
            let memory_id = item.as_str().ok_or("ID esperado inválido")?;
            let memory = by_id
                .get(memory_id)
                .ok_or(format!("recuerdo esperado ausente: {memory_id}"))?;
            if memory.get("scope_kind") != query.get("scope_kind")
                || memory.get("scope_id") != query.get("scope_id")
            {
                return Err(format!("ámbito incorrecto: {id}"));
            }
        }
    }
    if positive != number(&manifest, "positive_queries")?
        || (queries.len() as u64 - positive) != number(&manifest, "negative_queries")?
    {
        return Err("conteo positivo/negativo distinto".into());
    }
    for query in &unscoped {
        let id = string(query, "case_id")?;
        if !seen.insert(id.to_owned()) {
            return Err(format!("consulta duplicada: {id}"));
        }
        for item in array(query, "expected_memory_ids")? {
            let memory_id = item.as_str().ok_or("ID esperado inválido")?;
            let memory = by_id
                .get(memory_id)
                .ok_or(format!("recuerdo esperado ausente: {memory_id}"))?;
            if memory.get("scope_kind") != query.get("expected_scope_kind")
                || memory.get("scope_id") != query.get("expected_scope_id")
            {
                return Err(format!("ámbito unscoped incorrecto: {id}"));
            }
        }
    }
    println!(
        "{}",
        json!({"valid":true,"kind":"general-memory-v1","memories":memories.len(),
        "queries":queries.len(),"unscoped":unscoped.len(),"memories_sha256":digest(&root.join("memories.json"))?,
        "queries_sha256":digest(&root.join("queries.jsonl"))?})
    );
    Ok(())
}

pub fn validate_candidate_matrix(root: &Path) -> Result<(), String> {
    let manifest = read_json(&root.join("manifest.json"))?;
    if string(&manifest, "kind")? != "candidate_matrix_v1" || number(&manifest, "version")? != 1 {
        return Err("manifiesto candidate-matrix-v1 inválido".into());
    }
    let corpus_path = root.join("corpus.json");
    if string(&manifest, "corpus_sha256")? != digest(&corpus_path)? {
        return Err("hash de corpus distinto".into());
    }
    let corpus = read_json(&corpus_path)?;
    let corpus = corpus.as_array().ok_or("corpus no es lista")?;
    if corpus.len() as u64 != number(&manifest, "active_memories")? {
        return Err("conteo de recuerdos distinto".into());
    }
    let mut by_id = HashMap::new();
    for memory in corpus {
        let id = string(memory, "id")?;
        if by_id.insert(id.to_owned(), memory).is_some() {
            return Err(format!("recuerdo duplicado: {id}"));
        }
        if string(memory, "state")? != "active" {
            return Err(format!("recuerdo no activo: {id}"));
        }
    }
    for variant in ["scoped", "unscoped"] {
        let path = root.join(format!("{variant}.jsonl"));
        let expected_hash = property(&manifest, "case_sha256")?
            .get(variant)
            .and_then(Value::as_str)
            .ok_or("falta hash de casos")?;
        if expected_hash != digest(&path)? {
            return Err(format!("hash distinto: {variant}"));
        }
        let rows = read_jsonl(&path)?;
        if rows.len() as u64 != number(&manifest, "cases_per_variant")? {
            return Err(format!("conteo distinto: {variant}"));
        }
        let mut ids = HashSet::new();
        let mut positive = 0u64;
        for case in &rows {
            let id = string(case, "id")?;
            if !ids.insert(id.to_owned()) {
                return Err(format!("caso duplicado: {id}"));
            }
            let expected = array(case, "expected_ids")?;
            if !expected.is_empty() {
                positive += 1;
            }
            for value in expected {
                let memory_id = value.as_str().ok_or("ID esperado inválido")?;
                let memory = by_id
                    .get(memory_id)
                    .ok_or(format!("recuerdo esperado ausente: {memory_id}"))?;
                let request = property(case, "request")?;
                if memory.get("scope_kind") != request.get("scope_kind")
                    || memory.get("scope_id") != request.get("scope_id")
                {
                    return Err(format!("ámbito incorrecto: {id}"));
                }
                if variant == "scoped" && memory.get("entity_id") != request.get("entity_id") {
                    return Err(format!("entidad incorrecta: {id}"));
                }
            }
        }
        if positive != number(&manifest, "positive_per_variant")? {
            return Err(format!("positivos distintos: {variant}"));
        }
    }
    println!(
        "{}",
        json!({"valid":true,"kind":"candidate-matrix-v1","memories":corpus.len(),
        "cases_per_variant":number(&manifest,"cases_per_variant")?})
    );
    Ok(())
}

fn is_opaque(id: &str) -> bool {
    id.len() == 22
        && id.starts_with("m_")
        && id[2..]
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn label_entity(id: &str) -> Result<&str, String> {
    let pieces: Vec<&str> = id.split('_').collect();
    let index = if ["q", "q2", "q5"].contains(&pieces[0]) {
        pieces.len().checked_sub(2)
    } else {
        pieces.len().checked_sub(1)
    }
    .ok_or_else(|| format!("ID inválido: {id}"))?;
    Ok(pieces[index])
}

pub fn validate(root: &Path) -> Result<(), String> {
    let manifest = read_json(&root.join("manifest.json"))?;
    if property(&manifest, "synthetic")?.as_bool() != Some(true) {
        return Err("el banco no está marcado como sintético".into());
    }
    let version = number(&manifest, "version")?;
    if !(1..=5).contains(&version) {
        return Err("versión de banco desconocida".into());
    }
    let files = property(&manifest, "files")?
        .as_object()
        .ok_or("files inválido")?;
    for (name, expected) in files {
        if name.contains('/') || name.contains('\\') || name.starts_with('.') {
            return Err(format!("nombre de archivo inválido: {name}"));
        }
        let actual = digest(&root.join(name))?;
        if expected.as_str() != Some(&actual) {
            return Err(format!("hash distinto: {name}"));
        }
    }
    let corpus = read_json(&root.join("corpus.json"))?;
    let revisions = corpus.as_array().ok_or("corpus inválido")?;
    if revisions.len() as u64 != number(&manifest, "fixture_revision_count")? {
        return Err("número de revisiones distinto".into());
    }
    let mut revision_keys = HashSet::new();
    let mut current: HashMap<String, &Value> = HashMap::new();
    for row in revisions {
        let id = string(row, "id")?;
        let revision = number(row, "revision")?;
        if !revision_keys.insert((id.to_owned(), revision)) {
            return Err(format!("revisión duplicada: {id}"));
        }
        if version >= 4 && !is_opaque(id) {
            return Err(format!("ID opaco inválido: {id}"));
        }
        if version <= 3
            && !["synthetic_", "shadow_", "expired_"]
                .iter()
                .any(|prefix| id.starts_with(prefix))
        {
            return Err(format!("ID de fixture inválido: {id}"));
        }
        if current
            .get(id)
            .is_none_or(|prior| number(prior, "revision").unwrap_or(0) < revision)
        {
            current.insert(id.to_owned(), row);
        }
    }
    if current.len() as u64 != number(&manifest, "unique_memories")? {
        return Err("número de recuerdos únicos distinto".into());
    }
    if files.contains_key("forms.json") {
        let forms = read_json(&root.join("forms.json"))?;
        let forms = forms.as_array().ok_or("forms inválido")?;
        if forms.len() as u64 != number(&manifest, "prompt_form_count")? {
            return Err("cantidad de formas distinta".into());
        }
        let mut templates = HashSet::new();
        for (index, form) in forms.iter().enumerate() {
            if number(form, "id")? != index as u64 || !templates.insert(string(form, "template")?) {
                return Err("formas duplicadas o fuera de orden".into());
            }
        }
    }
    let file = File::open(root.join("queries.jsonl")).map_err(|error| error.to_string())?;
    let mut count = 0u64;
    let mut positive = 0u64;
    let mut negative = 0u64;
    let mut seen = HashSet::new();
    let mut partitions: HashMap<String, String> = HashMap::new();
    let mut partition_counts: HashMap<String, u64> = HashMap::new();
    let mut segment_counts: HashMap<String, u64> = HashMap::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let case: Value =
            serde_json::from_str(&line).map_err(|error| format!("línea {}: {error}", index + 1))?;
        let id = string(&case, "id")?;
        if !seen.insert(id.to_owned()) {
            return Err(format!("consulta duplicada: {id}"));
        }
        let split = string(&case, "split")?;
        let segment = string(&case, "segment")?;
        let entity = label_entity(id)?;
        if partitions
            .insert(entity.to_owned(), split.to_owned())
            .is_some_and(|prior| prior != split)
        {
            return Err(format!("entidad cruzada entre particiones: {entity}"));
        }
        *partition_counts.entry(split.to_owned()).or_default() += 1;
        *segment_counts.entry(segment.to_owned()).or_default() += 1;
        if string(&case, "label_source")? != "deterministic_fixture_rule" {
            return Err(format!("etiqueta no determinista: {id}"));
        }
        let expected = array(&case, "expected_ids")?;
        let forbidden = array(&case, "forbidden_ids")?;
        let forbidden_ids: HashSet<&str> = forbidden
            .iter()
            .map(|item| item.as_str().ok_or("ID prohibido inválido"))
            .collect::<Result<_, _>>()?;
        for item in expected {
            let identity = item.as_str().ok_or("ID esperado inválido")?;
            if forbidden_ids.contains(identity) {
                return Err(format!("ID esperado y prohibido: {identity}"));
            }
            let row = current
                .get(identity)
                .ok_or_else(|| format!("recuerdo esperado ausente: {identity}"))?;
            let request = property(&case, "request")?;
            if string(row, "state")? != "active"
                || string(row, "scope_kind")? != string(request, "scope_kind")?
                || string(row, "scope_id")? != string(request, "scope_id")?
            {
                return Err(format!("ámbito o estado incorrecto: {id}"));
            }
            if [3, 5].contains(&version) && row.get("entity_id") != request.get("entity_id") {
                return Err(format!("entidad incorrecta: {id}"));
            }
            let as_of = string(request, "as_of")?;
            if row
                .get("valid_from")
                .and_then(Value::as_str)
                .is_some_and(|date| date > as_of)
                || row
                    .get("valid_until")
                    .and_then(Value::as_str)
                    .is_some_and(|date| date <= as_of)
            {
                return Err(format!("vigencia incorrecta: {id}"));
            }
            let fact = string(&case, "expected_fact")?;
            if !string(row, "summary")?.contains(fact) {
                return Err(format!("hecho esperado ausente: {id}"));
            }
            if version == 5
                && (row.get("conditions") != case.get("expected_condition")
                    || row.get("source") != case.get("expected_source"))
            {
                return Err(format!("condición o fuente distinta: {id}"));
            }
        }
        if expected.is_empty() {
            negative += 1;
        } else {
            positive += 1;
        }
        count += 1;
    }
    if count != number(&manifest, "query_count")?
        || positive != number(&manifest, "positive_queries")?
        || negative != number(&manifest, "negative_queries")?
    {
        return Err("conteos de consultas distintos".into());
    }
    for (key, value) in property(&manifest, "partitions")?
        .as_object()
        .ok_or("partitions inválido")?
    {
        if partition_counts.get(key).copied().unwrap_or(0)
            != value.as_u64().ok_or("partición inválida")?
        {
            return Err(format!("conteo de partición distinto: {key}"));
        }
    }
    if let Some(segments) = manifest.get("segments").and_then(Value::as_object) {
        for (key, value) in segments {
            if segment_counts.get(key).copied().unwrap_or(0)
                != value.as_u64().ok_or("segmento inválido")?
            {
                return Err(format!("conteo de segmento distinto: {key}"));
            }
        }
    }
    println!(
        "{}",
        json!({"valid":true,"fixture_revisions":revisions.len(),"queries":count,
                           "entity_partitions":partitions.len()})
    );
    Ok(())
}
