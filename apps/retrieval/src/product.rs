//! Versioned product boundary. Legacy experimental contracts remain unchanged.
mod diagnostics;
#[cfg(test)]
mod tests;

use crate::classifier::{Classifier, PartialDecisionUsage};
use crate::model::*;
use crate::segmentation::{Segmentation, segment};
pub use diagnostics::Issue;
use gennie_memory_kernel::digest::sha256_hex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn capabilities() -> serde_json::Value {
    serde_json::json!({
        "contract_version":1, "transport":"openrouter", "evidence":"implemented_adapter_contract",
        "live_availability":"unverified", "prices":"unverified", "http_timeout_ms":20000,
        "max_operation_ms":25000, "automatic_retries":false, "automatic_fallback":false, "routing_scope":"gennie_memory_adapter_only",
        "token_budget":{"profile":crate::token_budget::PROFILE,"input_basis":"serialized_request_json","provider_tokenizer":"unverified","input_default":crate::token_budget::DEFAULT_INPUT,"seed_default":crate::token_budget::DEFAULT_SEED,"maximum":65536,"input_scope":"classification_plus_equivalence_review"},
        "source_verification":{"profile":crate::source_verification::PROFILE,"basis":"operator_registered_snapshot","policies":["report","require_verified"],"default":"report","locator":"utf8:start:end","semantic_support":"not_assessed","external_connectors":false},
        "text_segmentation":"local_no_model", "context_resolution":"caller_supplied",
        "providers":([ClassifierProvider::Decisions, ClassifierProvider::Jev, ClassifierProvider::Jev114].iter().map(|p|
            serde_json::json!({"provider":p, "requested_model":p.model_id(), "wire":"typed_decisions",
                "policies":[{"id":"baseline","candidate_limit":20,"relations":"unassessed","max_selected":1},
                    {"id":"per_memory_v1","candidate_limit":16,"relations":"provider_judgments","max_selected":16,"experimental":true},
                    {"id":"per_memory_v2","candidate_limit":8,"relations":"provider_judgments","max_selected":8,"experimental":true}],
                "equivalence_review":{"baseline":"second_request_when_selected_and_multiple_candidates","max_suggestions":1,"automatic_merge":false},
                "reason":"not_provided", "scores":"not_exposed_by_product_v1", "receipt":"optional_reported",
                "provider_context_limit":"unverified", "refusal":"reported_as_failure_if_not_valid_choice"})).collect::<Vec<_>>()),
        "legacy_only":[{"provider":"gemma3_4b","wire":"structured_chat_completions","product_v1":"unsupported"}]
    })
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProductRequest {
    pub contract_version: u32,
    pub search: SearchRequest,
    #[serde(default)]
    pub provider: ClassifierProvider,
    #[serde(default)]
    pub selection_policy: SelectionPolicy,
    pub request_id: Option<String>,
    #[serde(default)]
    pub context: Vec<ContextItem>,
    #[serde(default)]
    pub unresolved_references: Vec<UnresolvedReference>,
    #[serde(default)]
    pub budgets: Budgets,
    #[serde(default)]
    pub source_policy: crate::source_verification::SourcePolicy,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextItem {
    pub id: String,
    pub source: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedReference {
    pub start_byte: usize,
    pub end_byte: usize,
    pub required_field: ClarificationField,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClarificationField {
    Entity,
    Time,
    Condition,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Budgets {
    pub input_bytes: usize,
    pub seed_bytes: usize,
    pub input_tokens: usize,
    pub seed_tokens: usize,
    pub deadline_ms: u64,
}
impl Default for Budgets {
    fn default() -> Self {
        Self {
            input_bytes: 32_768,
            seed_bytes: 16_384,
            input_tokens: crate::token_budget::DEFAULT_INPUT,
            seed_tokens: crate::token_budget::DEFAULT_SEED,
            deadline_ms: 25_000,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Completed,
    Degraded,
    Failed,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    EvidenceSelected,
    NoCandidates,
    Abstained,
    ClarificationRequired,
    NotEvaluated,
}

#[derive(Debug, Serialize)]
pub struct Stage {
    pub name: &'static str,
    pub status: &'static str,
    pub elapsed_ms: f64,
    pub reason_code: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct Clarification {
    pub start_byte: usize,
    pub end_byte: usize,
    pub required_field: ClarificationField,
    pub origin: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Coverage {
    pub input: &'static str,
    pub semantic_index: &'static str,
    pub retrieval: &'static str,
    pub candidate_limit: u32,
    pub returned_candidates: usize,
    pub candidates_presented: usize,
    pub classification: &'static str,
    pub equivalence_review: &'static str,
    pub question: &'static str,
    pub seed: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Assessment {
    pub memory_id: String,
    pub revision: i64,
    pub selected: bool,
    pub included_in_seed: bool,
    pub relation: &'static str,
    pub origin: &'static str,
    pub validation: &'static str,
    pub provenance_status: &'static str,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PotentialEquivalent {
    pub anchor: SelectedCandidate,
    pub candidate: SelectedCandidate,
    pub relation: &'static str,
    pub decision: &'static str,
}

#[derive(Debug, Default, Serialize)]
pub struct Usage {
    pub classifier_invocations: usize,
    pub receipt_status: &'static str,
    pub served_model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reported_cost_usd: Option<f64>,
    pub cost_status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub contract_version: u32,
    pub implementation_sha256: String,
    pub request_id: String,
    pub execution_status: ExecutionStatus,
    pub outcome: Outcome,
    pub effective_request: Option<ProductRequest>,
    pub input_sha256: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub total_ms: f64,
    pub segmentation: Option<Segmentation>,
    pub context_segmentations: Vec<Segmentation>,
    pub ambiguity: &'static str,
    pub clarifications: Vec<Clarification>,
    pub retrieval: Option<SearchResponse>,
    pub assessments: Vec<Assessment>,
    pub potential_equivalents: Vec<PotentialEquivalent>,
    pub seed: Option<SeedResponse>,
    pub coverage: Coverage,
    pub stages: Vec<Stage>,
    pub issues: Vec<Issue>,
    pub usage: Usage,
    pub token_accounting: crate::token_budget::Accounting,
    pub source_verification: crate::source_verification::SourceReport,
    pub requested_model: Option<&'static str>,
    pub local_review: &'static str,
}

impl ProductResponse {
    fn stage(
        &mut self,
        name: &'static str,
        status: &'static str,
        started: Instant,
        reason: Option<&'static str>,
    ) {
        self.stages.push(Stage {
            name,
            status,
            elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
            reason_code: reason,
        });
    }
    fn fail(&mut self, issue: Issue, degraded: bool) {
        self.execution_status = if degraded {
            ExecutionStatus::Degraded
        } else {
            ExecutionStatus::Failed
        };
        self.outcome = Outcome::NotEvaluated;
        self.issues.push(issue);
    }
}

fn request_id() -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

/// Also wraps malformed input in the product error contract, without echoing it.
pub fn execute_json(
    conn: &Connection,
    input: &str,
    classifier: &impl Classifier,
) -> ProductResponse {
    let started = Instant::now();
    static IMPLEMENTATION_HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let mut out = ProductResponse {
        contract_version: 1,
        implementation_sha256: IMPLEMENTATION_HASH
            .get_or_init(|| {
                sha256_hex(
                    concat!(
                        include_str!("product.rs"),
                        include_str!("product/diagnostics.rs"),
                        include_str!("segmentation.rs"),
                        include_str!("service.rs"),
                        include_str!("token_budget.rs"),
                        include_str!("source_verification.rs")
                    )
                    .as_bytes(),
                )
            })
            .clone(),
        request_id: request_id(),
        execution_status: ExecutionStatus::Completed,
        outcome: Outcome::NotEvaluated,
        effective_request: None,
        input_sha256: (input.len() <= 131_072).then(|| sha256_hex(input.as_bytes())),
        started_at: crate::repo::utc_now(conn).ok(),
        finished_at: None,
        total_ms: 0.0,
        segmentation: None,
        context_segmentations: vec![],
        ambiguity: "not_assessed",
        clarifications: vec![],
        retrieval: None,
        assessments: vec![],
        potential_equivalents: vec![],
        seed: None,
        coverage: Coverage {
            input: "unknown",
            semantic_index: "not_applicable",
            retrieval: "not_evaluated",
            candidate_limit: 20,
            returned_candidates: 0,
            candidates_presented: 0,
            classification: "not_evaluated",
            equivalence_review: "not_evaluated",
            question: "unknown",
            seed: "not_evaluated",
        },
        stages: vec![],
        issues: vec![],
        usage: Usage {
            receipt_status: "not_called",
            cost_status: "not_called",
            ..Usage::default()
        },
        token_accounting: crate::token_budget::Accounting::default(),
        source_verification: crate::source_verification::SourceReport::default(),
        requested_model: None,
        local_review: "not_available_dashboard_removed",
    };
    let parsed = (input.len() <= 131_072)
        .then(|| serde_json::from_str::<ProductRequest>(input).ok())
        .flatten();
    if let Some(mut request) = parsed {
        request.search.limit = Some(request.search.limit.unwrap_or(20));
        request.search.query_mode = Some(request.search.query_mode.unwrap_or(QueryMode::Keywords));
        if let Some(id) = request.request_id.as_ref().filter(|id| valid_id(id)) {
            out.request_id = id.clone();
        }
        out.coverage.candidate_limit = request.search.limit.unwrap();
        out.requested_model = Some(request.provider.model_id());
        execute(conn, &request, classifier, &mut out, started);
        // Only valid input is reflected. Failed input may include arbitrary secrets.
        if out
            .stages
            .iter()
            .any(|s| s.name == "validation" && s.status == "completed")
        {
            out.effective_request = Some(request);
        }
    } else {
        out.stage("validation", "failed", started, Some("invalid_request"));
        out.fail(
            Issue::local(
                "invalid_request",
                "validation",
                "La solicitud no cumple el JSON del contrato v1 o supera 128 KiB.",
                "correct_request",
            ),
            false,
        );
    }
    for name in [
        "validation",
        "segmentation",
        "search",
        "classification",
        "composition",
    ] {
        if !out.stages.iter().any(|s| s.name == name) {
            out.stages.push(Stage {
                name,
                status: "skipped",
                elapsed_ms: 0.0,
                reason_code: Some("prior_stage_stopped"),
            });
        }
    }
    out.finished_at = crate::repo::utc_now(conn).ok();
    out.total_ms = started.elapsed().as_secs_f64() * 1000.0;
    out
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.:".contains(&b))
}

fn validate(r: &ProductRequest) -> Result<(), Issue> {
    let invalid =
        |message| Issue::local("invalid_request", "validation", message, "correct_request");
    if r.contract_version != 1 {
        return Err(invalid("contract_version debe ser 1."));
    }
    if r.request_id.as_ref().is_some_and(|id| !valid_id(id)) {
        return Err(invalid(
            "request_id debe ser un identificador ASCII de hasta 128 bytes.",
        ));
    }
    crate::service::validate_search_request(&r.search).map_err(|_| invalid("Consulta, ámbito, entidad, fecha UTC con milisegundos o límite de candidatos inválidos."))?;
    if !matches!(
        r.search.scope_kind.as_str(),
        "user" | "agent" | "project" | "conversation"
    ) || !valid_id(&r.search.scope_id)
    {
        return Err(invalid(
            "El ámbito debe tener un tipo admitido y un identificador válido.",
        ));
    }
    if !(1..=32_768).contains(&r.budgets.input_bytes)
        || !(1..=65_536).contains(&r.budgets.seed_bytes)
        || !(1..=65_536).contains(&r.budgets.input_tokens)
        || !(1..=65_536).contains(&r.budgets.seed_tokens)
        || !(1..=25_000).contains(&r.budgets.deadline_ms)
    {
        return Err(invalid(
            "Presupuestos fuera de rango: entrada 1–32768 bytes, semilla 1–65536 bytes, tokens de entrada y semilla 1–65536, plazo 1–25000 ms.",
        ));
    }
    let total = r.search.query.len()
        + r.context
            .iter()
            .map(|c| c.text.len() + c.id.len() + c.source.len())
            .sum::<usize>();
    let mut ids = std::collections::HashSet::new();
    if total > r.budgets.input_bytes
        || r.context.len() > 16
        || r.context.iter().any(|c| {
            !valid_id(&c.id)
                || !ids.insert(&c.id)
                || c.source.trim().is_empty()
                || c.source.len() > 256
                || c.text.trim().is_empty()
        })
    {
        return Err(invalid(
            "La entrada o el contexto exceden el presupuesto, o contienen referencias vacías/duplicadas.",
        ));
    }
    if r.unresolved_references.len() > 16
        || r.unresolved_references.iter().any(|u| {
            u.start_byte >= u.end_byte || r.search.query.get(u.start_byte..u.end_byte).is_none()
        })
    {
        return Err(invalid(
            "Las referencias sin resolver deben apuntar a rangos UTF-8 de la consulta; máximo 16.",
        ));
    }
    if r.provider == ClassifierProvider::Gemma3_4B
        || !matches!(
            r.selection_policy,
            SelectionPolicy::Baseline | SelectionPolicy::PerMemoryV1 | SelectionPolicy::PerMemoryV2
        )
    {
        return Err(Issue::local(
            "unsupported_capability",
            "validation",
            "El contrato v1 admite Decisions/Jev con baseline, per_memory_v1 o per_memory_v2; esta combinación no está habilitada.",
            "choose_supported_policy",
        ));
    }
    Ok(())
}

fn execute(
    conn: &Connection,
    r: &ProductRequest,
    classifier: &impl Classifier,
    out: &mut ProductResponse,
    started: Instant,
) {
    if let Err(issue) = validate(r) {
        out.stage("validation", "failed", started, Some(issue.code));
        out.fail(issue, false);
        return;
    }
    out.source_verification.policy = r.source_policy;
    out.token_accounting.input_limit = r.budgets.input_tokens;
    out.token_accounting.seed_limit = r.budgets.seed_tokens;
    out.stage("validation", "completed", started, None);
    let deadline = started + Duration::from_millis(r.budgets.deadline_ms);
    let phase = Instant::now();
    let segments = segment(&r.search.query).and_then(|query| {
        r.context
            .iter()
            .map(|c| segment(&c.text))
            .collect::<Result<Vec<_>, _>>()
            .map(|context| (query, context))
    });
    match segments {
        Ok((query, context)) => {
            out.segmentation = Some(query);
            out.context_segmentations = context;
            out.coverage.input = "complete";
        }
        Err(code) => {
            out.stage("segmentation", "failed", phase, Some(code));
            out.fail(Issue::local(code, "segmentation", "No se pudo representar el texto íntegro dentro de los límites de segmentación.", "reduce_input"), false);
            return;
        }
    }
    out.stage("segmentation", "completed", phase, None);
    if !r.unresolved_references.is_empty() {
        out.ambiguity = "caller_reported";
        out.outcome = Outcome::ClarificationRequired;
        out.clarifications = r
            .unresolved_references
            .iter()
            .map(|u| Clarification {
                start_byte: u.start_byte,
                end_byte: u.end_byte,
                required_field: u.required_field,
                origin: "caller",
                message: match u.required_field {
                    ClarificationField::Entity => {
                        "¿A qué entidad se refiere el fragmento señalado?"
                    }
                    ClarificationField::Time => {
                        "¿A qué fecha o periodo se refiere el fragmento señalado?"
                    }
                    ClarificationField::Condition => {
                        "¿Qué condición se aplica al fragmento señalado?"
                    }
                },
            })
            .collect();
        return;
    }
    if expired(out, deadline, "search") {
        return;
    }
    let phase = Instant::now();
    match crate::service::search(conn, &r.search) {
        Ok(found) => {
            out.coverage.returned_candidates = found.candidates.len();
            out.retrieval = Some(found);
        }
        Err(_) => {
            out.stage("search", "failed", phase, Some("storage_error"));
            out.fail(
                Issue::local(
                    "storage_error",
                    "search",
                    "No se pudo completar la búsqueda en el almacén local.",
                    "inspect_local_storage",
                ),
                false,
            );
            return;
        }
    }
    out.coverage.retrieval = out.retrieval.as_ref().unwrap().coverage.retrieval;
    if out.retrieval.as_ref().unwrap().status == "degraded" {
        out.stage("search", "degraded", phase, Some("fts_index_incomplete"));
        out.fail(
            Issue::local(
                "fts_index_incomplete",
                "search",
                "Faltan documentos de revisiones elegibles en la proyección textual.",
                "rebuild_text_projection",
            ),
            true,
        );
        return;
    }
    if out.retrieval.as_ref().unwrap().coverage.query == "empty_after_normalization" {
        out.stage("search", "completed", phase, None);
        out.outcome = Outcome::NotEvaluated;
        out.issues.push(Issue::local(
            "query_empty_after_normalization",
            "search",
            "La consulta no conserva términos buscables; reformula o usa query_mode natural.",
            "rephrase_query",
        ));
        return;
    }
    out.stage("search", "completed", phase, None);
    if expired(out, deadline, "classification") {
        return;
    }
    let candidates = out.retrieval.as_ref().unwrap().candidates.clone();
    if candidates.is_empty() {
        out.outcome = Outcome::NoCandidates;
        out.coverage.seed = "not_applicable";
        out.token_accounting.seed_status = "not_applicable";
        return;
    }
    if expired(out, deadline, "classification") {
        return;
    }
    let source_phase = Instant::now();
    let candidates = out.retrieval.as_ref().unwrap().candidates.clone();
    if !refresh_sources(conn, r, &candidates, out, source_phase) {
        return;
    }
    if expired(out, deadline, "source_verification") {
        return;
    }
    out.stage("source_verification", "completed", source_phase, None);
    let limit = r
        .selection_policy
        .per_memory_candidate_limit()
        .unwrap_or(candidates.len());
    out.coverage.candidates_presented = candidates.len().min(limit);
    // Keep query and context in separate JSON fields; never alter the FTS query.
    let classifier_query = if r.context.is_empty() {
        r.search.query.clone()
    } else {
        serde_json::json!({"query": r.search.query, "context": r.context, "context_origin": "caller_supplied_data"}).to_string()
    };
    let phase = Instant::now();
    out.usage.classifier_invocations = 1;
    let bounded = classifier.classify_budgeted_until(
        &classifier_query,
        &r.search.scope_id,
        &candidates,
        r.provider,
        r.selection_policy,
        deadline,
        out.token_accounting.remaining(),
    );
    out.token_accounting.extend(bounded.requests);
    let decision = match bounded.result {
        Ok(decision) => decision,
        Err(error) => {
            out.usage = usage(
                error.partial_usage.as_ref(),
                true,
                !matches!(
                    error.code,
                    "missing_api_key"
                        | "provider_input_limit"
                        | "deadline_not_supported"
                        | "operation_deadline"
                        | "input_token_limit"
                        | "token_budget_not_supported"
                        | "token_counter_unavailable"
                        | "token_counter_invalid_utf8"
                ),
            );
            let issue = Issue::provider(&error);
            out.stage("classification", "failed", phase, Some(issue.code));
            out.coverage.classification = "failed";
            out.fail(issue, true);
            return;
        }
    };
    out.usage = usage(
        Some(&PartialDecisionUsage {
            served_model: decision.served_model.clone(),
            input_tokens: decision.input_tokens,
            output_tokens: decision.output_tokens,
            reported_cost_usd: decision.reported_cost_usd,
        }),
        false,
        true,
    );
    let candidates = out.retrieval.as_ref().unwrap().candidates.clone();
    let valid = decision.consensus.is_none()
        && decision.applicability.is_none()
        && decision.slot_evidence.is_none()
        && if r.selection_policy == SelectionPolicy::Baseline {
            decision.per_memory.is_none()
                && decision
                    .selected_id
                    .as_ref()
                    .is_none_or(|id| candidates.iter().any(|c| c.memory_id == *id))
        } else {
            decision.per_memory.as_ref().is_some_and(|trace| {
                trace.valid(
                    &candidates[..candidates.len().min(limit)],
                    decision.selected_id.as_deref(),
                ) && trace.relations_match_task(false)
                    && trace.factor_checks.is_none()
                    && trace.scoped_checks.is_none()
                    && trace.verification.is_none()
                    && trace.choice_scores.is_none()
                    && trace.request_sha256.is_none()
            })
        };
    if !valid {
        out.stage("classification", "failed", phase, Some("invalid_selection"));
        out.coverage.classification = "failed";
        out.fail(Issue::local("invalid_selection", "classification", "La respuesta no identifica candidatos y revisiones válidos con el esquema solicitado.", "inspect_provider_response"), true);
        return;
    }
    if r.selection_policy == SelectionPolicy::Baseline {
        if let Some(anchor_id) = decision.selected_id.as_deref() {
            let anchor = candidates
                .iter()
                .find(|candidate| candidate.memory_id == anchor_id)
                .unwrap();
            if candidates.len() > 1 {
                let phase = Instant::now();
                out.usage.classifier_invocations = 2;
                let bounded = classifier.suggest_equivalent_budgeted_until(
                    &r.search.scope_id,
                    anchor,
                    &candidates,
                    r.provider,
                    deadline,
                    out.token_accounting.remaining(),
                );
                out.token_accounting.extend(bounded.requests);
                match bounded.result {
                    Ok(suggestion)
                        if suggestion.per_memory.is_none()
                            && suggestion.consensus.is_none()
                            && suggestion.applicability.is_none()
                            && suggestion.slot_evidence.is_none()
                            && suggestion.selected_id.as_deref().is_none_or(|id| {
                                id != anchor_id && candidates.iter().any(|c| c.memory_id == id)
                            }) =>
                    {
                        let primary = PartialDecisionUsage {
                            served_model: decision.served_model.clone(),
                            input_tokens: decision.input_tokens,
                            output_tokens: decision.output_tokens,
                            reported_cost_usd: decision.reported_cost_usd,
                        };
                        let secondary = PartialDecisionUsage {
                            served_model: suggestion.served_model,
                            input_tokens: suggestion.input_tokens,
                            output_tokens: suggestion.output_tokens,
                            reported_cost_usd: suggestion.reported_cost_usd,
                        };
                        out.usage = combined_usage(&primary, Some(&secondary), false);
                        out.coverage.equivalence_review = "complete";
                        if let Some(candidate) = suggestion
                            .selected_id
                            .and_then(|id| candidates.iter().find(|c| c.memory_id == id))
                        {
                            out.potential_equivalents.push(PotentialEquivalent {
                                anchor: SelectedCandidate {
                                    memory_id: anchor.memory_id.clone(),
                                    revision: anchor.revision,
                                },
                                candidate: SelectedCandidate {
                                    memory_id: candidate.memory_id.clone(),
                                    revision: candidate.revision,
                                },
                                relation: "possible_duplicate",
                                decision: "agent_review_required",
                            });
                        }
                        out.stage("equivalence_review", "completed", phase, None);
                    }
                    Ok(suggestion) => {
                        let primary = PartialDecisionUsage {
                            served_model: decision.served_model.clone(),
                            input_tokens: decision.input_tokens,
                            output_tokens: decision.output_tokens,
                            reported_cost_usd: decision.reported_cost_usd,
                        };
                        let secondary = PartialDecisionUsage {
                            served_model: suggestion.served_model,
                            input_tokens: suggestion.input_tokens,
                            output_tokens: suggestion.output_tokens,
                            reported_cost_usd: suggestion.reported_cost_usd,
                        };
                        out.usage = combined_usage(&primary, Some(&secondary), true);
                        out.coverage.equivalence_review = "invalid_response";
                        out.execution_status = ExecutionStatus::Degraded;
                        out.issues.push(Issue::local("invalid_equivalence_suggestion", "equivalence_review", "El proveedor devolvió un candidato fuera del conjunto presentado; no se aplica.", "inspect_provider_response"));
                        out.stage(
                            "equivalence_review",
                            "failed",
                            phase,
                            Some("invalid_equivalence_suggestion"),
                        );
                    }
                    Err(error) if crate::token_budget::is_local_error(error.code) => {
                        out.usage.classifier_invocations = 1;
                        out.coverage.equivalence_review = "omitted_budget";
                        out.execution_status = ExecutionStatus::Degraded;
                        let mut issue = Issue::provider(&error);
                        issue.stage = "equivalence_review";
                        out.issues.push(issue);
                        out.stage("equivalence_review", "skipped", phase, Some(error.code));
                    }
                    Err(error) => {
                        let primary = PartialDecisionUsage {
                            served_model: decision.served_model.clone(),
                            input_tokens: decision.input_tokens,
                            output_tokens: decision.output_tokens,
                            reported_cost_usd: decision.reported_cost_usd,
                        };
                        out.usage = combined_usage(&primary, error.partial_usage.as_ref(), true);
                        out.coverage.equivalence_review = "unavailable";
                        out.execution_status = ExecutionStatus::Degraded;
                        out.issues.push(Issue::provider(&error));
                        out.stage("equivalence_review", "failed", phase, Some(error.code));
                    }
                }
            } else {
                out.coverage.equivalence_review = "not_needed";
            }
        } else {
            out.coverage.equivalence_review = "not_needed";
        }
    } else {
        out.coverage.equivalence_review = "not_applicable";
    }
    out.coverage.classification = if out.coverage.candidates_presented < candidates.len() {
        "partial"
    } else {
        "complete"
    };
    let selected_ids: Vec<&str> = if let Some(trace) = &decision.per_memory {
        trace.selected_ids()
    } else {
        decision.selected_id.as_deref().into_iter().collect()
    };
    out.assessments = candidates
        .iter()
        .map(|c| {
            let assessed = decision.per_memory.as_ref().and_then(|t| {
                t.items
                    .iter()
                    .find(|a| a.memory_id == c.memory_id && a.revision == c.revision)
            });
            Assessment {
                memory_id: c.memory_id.clone(),
                revision: c.revision,
                selected: selected_ids.contains(&c.memory_id.as_str()),
                included_in_seed: false,
                relation: if out
                    .potential_equivalents
                    .iter()
                    .any(|pair| pair.candidate.memory_id == c.memory_id)
                {
                    "possible_duplicate"
                } else {
                    assessed.map_or("unassessed", |a| match a.relation {
                        crate::per_memory::EvidenceRelation::SupportsAnswer => "supports_answer",
                        crate::per_memory::EvidenceRelation::RefutesPremise => "refutes_premise",
                        crate::per_memory::EvidenceRelation::PendingOrPartial => "partial_context",
                        crate::per_memory::EvidenceRelation::Unrelated => "unrelated",
                        crate::per_memory::EvidenceRelation::UsefulEvidence => "unassessed",
                    })
                },
                origin: if out
                    .potential_equivalents
                    .iter()
                    .any(|pair| pair.candidate.memory_id == c.memory_id)
                {
                    "second_pass_suggestion"
                } else if assessed.is_some() {
                    "provider"
                } else {
                    "not_assessed"
                },
                validation: "structural_only",
                provenance_status: source_status(out, &c.memory_id, c.revision),
                reason: out
                    .potential_equivalents
                    .iter()
                    .any(|pair| pair.candidate.memory_id == c.memory_id)
                    .then(|| {
                        "Possible equivalent; presented separately for agent review.".to_owned()
                    }),
            }
        })
        .collect();
    out.stage("classification", "completed", phase, None);
    if expired(out, deadline, "composition") {
        return;
    }
    let phase = Instant::now();
    let candidates = out.retrieval.as_ref().unwrap().candidates.clone();
    let presented = candidates
        .iter()
        .map(|c| SelectedCandidate {
            memory_id: c.memory_id.clone(),
            revision: c.revision,
        })
        .collect();
    let selected = out
        .assessments
        .iter()
        .filter(|a| {
            a.selected
                || out
                    .potential_equivalents
                    .iter()
                    .any(|pair| pair.candidate.memory_id == a.memory_id)
        })
        .map(|a| SelectedCandidate {
            memory_id: a.memory_id.clone(),
            revision: a.revision,
        })
        .collect();
    let seed = crate::service::compose_seed_with_limit(
        conn,
        &SeedRequest {
            search: Some(r.search.clone()),
            hybrid: None,
            presented,
            selected,
            classifier: r.provider.model_id().into(),
            classification_status: "complete".into(),
        },
        limit.max(8),
    );
    match seed {
        Ok(mut seed) => {
            seed.potential_equivalents = out
                .potential_equivalents
                .iter()
                .map(|pair| SeedEquivalenceSuggestion {
                    anchor_memory_id: pair.anchor.memory_id.clone(),
                    anchor_revision: pair.anchor.revision,
                    candidate_memory_id: pair.candidate.memory_id.clone(),
                    candidate_revision: pair.candidate.revision,
                    relation: pair.relation,
                    decision: pair.decision,
                })
                .collect();
            if !seed.potential_equivalents.is_empty() {
                if let Some(text) = seed.seed_text.as_mut() {
                    text.push_str("\n[Revisión requerida: los recuerdos anteriores pueden ser equivalentes; se mantienen separados. El agente debe decidir si los combina o si son distintos.]");
                }
            }
            // Re-read the current source version after the classifier and composition.
            let included: Vec<_> = candidates
                .iter()
                .filter(|c| {
                    seed.memories
                        .iter()
                        .any(|m| m.memory_id == c.memory_id && m.revision == c.revision)
                })
                .cloned()
                .collect();
            if !refresh_sources(conn, r, &included, out, phase) {
                return;
            }
            for assessment in &mut out.assessments {
                if let Some(check) = out.source_verification.checks.iter().find(|c| {
                    c.memory_id == assessment.memory_id && c.revision == assessment.revision
                }) {
                    assessment.provenance_status = provenance(&check.status);
                }
            }
            let blocked = included.iter().any(|c| {
                let status = source_status(out, &c.memory_id, c.revision);
                status == "rejected"
                    || (r.source_policy
                        == crate::source_verification::SourcePolicy::RequireVerified
                        && status != "verified_literal")
            });
            if blocked {
                out.coverage.seed = "omitted_source_verification";
                out.token_accounting.seed_status = "omitted_source_verification";
                out.stage(
                    "source_verification",
                    "failed",
                    phase,
                    Some("source_verification_blocked"),
                );
                out.fail(Issue::local("source_verification_blocked", "source_verification", "La procedencia de un recuerdo incluido está rechazada o no cumple la verificación exigida; se conserva la selección sin publicar semilla.", "register_or_correct_source_and_verify"), true);
                return;
            }
            for memory in &mut seed.memories {
                memory.provenance_status = source_status(out, &memory.memory_id, memory.revision);
            }
            if let Some(text) = seed.seed_text.as_mut() {
                text.push_str("\n[Procedencia local: ");
                text.push_str(
                    &seed
                        .memories
                        .iter()
                        .map(|m| format!("{}@{}={}", m.memory_id, m.revision, m.provenance_status))
                        .collect::<Vec<_>>()
                        .join("; "),
                );
                text.push_str(". Solo se comprueba el texto original contra snapshots aportados por el operador; veracidad, resumen y condiciones añadidas sin evaluar.]");
            }
            let seed_text = seed.seed_text.as_deref().unwrap_or("");
            let tokens = match crate::token_budget::count(seed_text) {
                Ok(tokens) => tokens,
                Err(_) => {
                    out.token_accounting.seed_status = "counter_unavailable";
                    out.stage(
                        "composition",
                        "failed",
                        phase,
                        Some("token_counter_unavailable"),
                    );
                    out.fail(Issue::local("token_counter_unavailable", "composition", "No se pudo medir la semilla; se conserva la selección sin publicar texto.", "inspect_token_counter"), true);
                    return;
                }
            };
            out.token_accounting.seed_tokens = Some(tokens);
            if seed_text.len() > r.budgets.seed_bytes {
                out.token_accounting.seed_status = "omitted_byte_budget";
                out.stage("composition", "failed", phase, Some("seed_byte_limit"));
                out.coverage.seed = "omitted_budget";
                out.fail(Issue::local("seed_byte_limit", "composition", "La semilla completa supera el presupuesto; se conserva la selección sin publicar texto truncado.", "increase_seed_budget"), true);
                return;
            }
            if tokens > r.budgets.seed_tokens {
                out.token_accounting.seed_status = "omitted_token_budget";
                out.stage("composition", "failed", phase, Some("seed_token_limit"));
                out.coverage.seed = "omitted_budget";
                out.fail(Issue::local("seed_token_limit", "composition", "La semilla completa supera el presupuesto de tokens; se conserva la selección sin publicar texto truncado.", "increase_seed_token_budget"), true);
                return;
            }
            out.token_accounting.seed_status = "complete";
            out.seed = Some(seed);
        }
        Err(_) => {
            out.stage("composition", "failed", phase, Some("composition_failed"));
            out.fail(
                Issue::local(
                    "composition_failed",
                    "composition",
                    "No se pudo revalidar y componer la semilla desde los recuerdos vigentes.",
                    "repeat_after_review",
                ),
                true,
            );
            return;
        }
    }
    if expired(out, deadline, "composition") {
        out.seed = None;
        out.token_accounting.seed_status = "omitted_deadline";
        return;
    }
    for a in &mut out.assessments {
        a.included_in_seed = a.selected
            || out
                .potential_equivalents
                .iter()
                .any(|pair| pair.candidate.memory_id == a.memory_id);
    }
    out.coverage.seed = "complete";
    out.stage("composition", "completed", phase, None);
    out.outcome = if out.assessments.iter().any(|a| a.selected) {
        Outcome::EvidenceSelected
    } else {
        Outcome::Abstained
    };
}

fn expired(out: &mut ProductResponse, deadline: Instant, stage: &'static str) -> bool {
    if Instant::now() < deadline {
        return false;
    }
    out.fail(
        Issue::local(
            "operation_deadline",
            stage,
            "Se agotó el plazo total de recuperación; no se publica una semilla tardía.",
            "review_deadline",
        ),
        out.retrieval.is_some(),
    );
    out.stages.push(Stage {
        name: stage,
        status: "failed",
        elapsed_ms: 0.0,
        reason_code: Some("operation_deadline"),
    });
    true
}

fn usage(receipt: Option<&PartialDecisionUsage>, failed: bool, attempted: bool) -> Usage {
    let any = receipt.is_some_and(|u| {
        u.input_tokens.is_some() || u.output_tokens.is_some() || u.reported_cost_usd.is_some()
    });
    let complete = receipt.is_some_and(|u| {
        u.input_tokens.is_some() && u.output_tokens.is_some() && u.reported_cost_usd.is_some()
    });
    Usage {
        classifier_invocations: usize::from(attempted),
        receipt_status: if !attempted {
            "not_called"
        } else if complete {
            "complete"
        } else if any {
            "partial"
        } else {
            "unknown"
        },
        served_model: receipt.and_then(|u| u.served_model.clone()),
        input_tokens: receipt.and_then(|u| u.input_tokens),
        output_tokens: receipt.and_then(|u| u.output_tokens),
        reported_cost_usd: receipt.and_then(|u| u.reported_cost_usd),
        cost_status: if !attempted {
            "not_called"
        } else if receipt.is_some_and(|u| u.reported_cost_usd.is_some()) {
            if failed {
                "reported_on_failure"
            } else {
                "reported"
            }
        } else {
            "unknown"
        },
    }
}

fn combined_usage(
    primary: &PartialDecisionUsage,
    second: Option<&PartialDecisionUsage>,
    second_failed: bool,
) -> Usage {
    let sum = |left: Option<u64>, right: Option<u64>| match (left, right) {
        (Some(left), Some(right)) => left.checked_add(right),
        (None, right) => right,
        (left, None) => left,
    };
    let second_cost = second.and_then(|receipt| receipt.reported_cost_usd);
    let cost = match (primary.reported_cost_usd, second_cost) {
        (Some(left), Some(right)) => Some(left + right).filter(|value| value.is_finite()),
        (None, right) => right,
        (left, None) => left,
    };
    let complete = primary.input_tokens.is_some()
        && primary.output_tokens.is_some()
        && primary.reported_cost_usd.is_some()
        && second.is_some_and(|receipt| {
            receipt.input_tokens.is_some()
                && receipt.output_tokens.is_some()
                && receipt.reported_cost_usd.is_some()
        })
        && !second_failed;
    let any = primary.input_tokens.is_some()
        || primary.output_tokens.is_some()
        || primary.reported_cost_usd.is_some()
        || second.is_some_and(|receipt| {
            receipt.input_tokens.is_some()
                || receipt.output_tokens.is_some()
                || receipt.reported_cost_usd.is_some()
        });
    Usage {
        classifier_invocations: 2,
        receipt_status: if complete {
            "complete"
        } else if any {
            "partial"
        } else {
            "unknown"
        },
        served_model: second
            .and_then(|receipt| receipt.served_model.clone())
            .or_else(|| primary.served_model.clone()),
        input_tokens: sum(
            primary.input_tokens,
            second.and_then(|receipt| receipt.input_tokens),
        ),
        output_tokens: sum(
            primary.output_tokens,
            second.and_then(|receipt| receipt.output_tokens),
        ),
        reported_cost_usd: cost,
        cost_status: if second_failed || !complete && cost.is_some() {
            "partial_known"
        } else if complete {
            "reported"
        } else {
            "unknown"
        },
    }
}

fn provenance(status: &str) -> &'static str {
    match status {
        "verified_literal" => "verified_literal",
        "rejected" => "rejected",
        "not_provided" => "not_provided",
        _ => "declared_unverified",
    }
}
fn source_status(out: &ProductResponse, id: &str, revision: i64) -> &'static str {
    out.source_verification
        .checks
        .iter()
        .find(|c| c.memory_id == id && c.revision == revision)
        .map_or("declared_unverified", |c| provenance(&c.status))
}
fn refresh_sources(
    conn: &Connection,
    request: &ProductRequest,
    candidates: &[Candidate],
    out: &mut ProductResponse,
    started: Instant,
) -> bool {
    let scope = crate::write::Scope {
        kind: request.search.scope_kind.clone(),
        id: request.search.scope_id.clone(),
    };
    match crate::source_verification::check_candidates(
        conn,
        request.search.actor_id.as_deref().unwrap_or("local"),
        &scope,
        candidates,
    ) {
        Ok(checks) => {
            for check in checks {
                out.source_verification.checks.retain(|old| {
                    old.memory_id != check.memory_id || old.revision != check.revision
                });
                out.source_verification.checks.push(check);
            }
            true
        }
        Err(_) => {
            out.coverage.seed = "omitted_source_verification";
            out.token_accounting.seed_status = "omitted_source_verification";
            out.stage(
                "source_verification",
                "failed",
                started,
                Some("source_check_failed"),
            );
            out.fail(Issue::local("source_check_failed", "source_verification", "No se pudo revalidar la memoria y su procedencia local; no se publica una semilla incierta.", "inspect_source_registry_and_revision"), true);
            false
        }
    }
}
