mod assertion_census;
mod assertion_ids_probe;
mod assertion_probe;
mod assertion_sets_probe;
mod assertion_tokens_probe;
mod benchmark;
mod campaign;
mod charts;
mod closeout;
mod codex_benchmark;
mod cohort;
mod complete_provider_probe;
mod config_selection;
mod config_sweep;
mod contribution_known_probe;
mod contribution_role_contract;
mod contribution_role_probe;
mod distributed_reference;
mod documentary_extraction_contract;
mod documentary_v3;
mod documentary_v3_io;
mod engram_benchmark;
mod evidence;
mod factor_audit;
mod family_calibration;
mod flow_compatibility;
mod full_prefix_probe;
mod independent_projection;
mod independent_projection_audit;
mod independent_projection_roles;
mod independent_role_audit;
mod isolated_provider_probe;
mod literal_identity_probe;
mod local_isolation_probe;
mod opposite_task_probe;
mod per_memory_matrix;
mod policy_probe;
mod predicate_probe;
mod projection_contract;
mod projection_ids;
mod projection_sets;
mod projection_tokens;
mod proposition_bank;
mod proposition_contract;
mod proposition_examples_contract;
mod proposition_examples_probe;
mod proposition_isolation_contract;
mod proposition_isolation_probe;
mod proposition_probe;
mod quote_role_contract;
mod quoted_role_probe;
mod qwen_reasoning_probe;
mod reference_catalog;
mod reference_coverage;
mod retrieval;
mod revision_matrix;
mod role_batch_probe;
mod roles;
mod scoped_definition_probe;
mod scoped_family_probe;
mod score_diagnostic;
mod search_benchmark;
mod semantic_check_contract;
mod semantic_link_contract;
mod semantic_link_direct_contract;
mod semantic_link_local_contract;
mod semantic_link_span_contract;
mod semantic_presentation;
mod semantic_representation;
mod slot_contract;
mod source_frame_audit;
mod source_review;
mod sparse_critic_probe;
mod stability;
mod typed_facts;
mod verified;

use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("iromem-eval: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let command = args
        .next()
        .ok_or("uso: iromem-eval <comando> ...; consulte docs/evaluation/contract-v1.md")?;
    match command.as_str() {
        "source-context-reference-v1" | "source-context-reference-v2" => {
            let mode = args.next().ok_or("falta operación bind o consume")?;
            let input = args.next().ok_or("falta entrada de referencia")?;
            let manifest = args.next().ok_or("falta manifiesto de referencia")?;
            let output = args.next().ok_or("falta salida nueva de referencia")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            let transport = if command == "source-context-reference-v2" {
                documentary_v3_io::source_context_reference::run_v2
            } else {
                documentary_v3_io::source_context_reference::run
            };
            transport(
                &mode,
                Path::new(&input),
                Path::new(&manifest),
                Path::new(&output),
            )
        }
        "assemble-documentary-v3-delivery" => {
            let packet = args.next().ok_or("falta paquete V3")?;
            let hash = args.next().ok_or("falta hash congelado V3")?;
            let items = args.next().ok_or("faltan items semánticos V3")?;
            let output = args.next().ok_or("falta salida nueva V3")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            documentary_v3_io::assembly::run(
                Path::new(&packet),
                &hash,
                Path::new(&items),
                Path::new(&output),
            )
        }
        "classify-documentary-context-v2" => {
            let input = args.next().ok_or("falta entrada de contexto documental")?;
            let output = args
                .next()
                .ok_or("falta salida nueva de contexto documental")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            documentary_v3_io::readings::run_context(Path::new(&input), Path::new(&output))
        }
        "validate-documentary-readings-v1" | "classify-documentary-readings-v1" => {
            let input = args
                .next()
                .ok_or("falta entrada de lecturas documentales")?;
            let output = args
                .next()
                .ok_or("falta salida nueva de lecturas documentales")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            documentary_v3_io::readings::run(
                Path::new(&input),
                Path::new(&output),
                command == "classify-documentary-readings-v1",
            )
        }
        "locate-documentary-v3-evidence" => {
            let input = args.next().ok_or("falta entrada de evidencia V3")?;
            let output = args.next().ok_or("falta salida nueva de evidencia V3")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            documentary_v3_io::evidence_locator::run(Path::new(&input), Path::new(&output))
        }
        "audit-documentary-v3-delivery" => {
            let packet = args.next().ok_or("falta paquete V3")?;
            let hash = args.next().ok_or("falta hash congelado del paquete V3")?;
            let delivery = args.next().ok_or("falta ruta de entrega V3")?;
            let output = args.next().ok_or("falta salida nueva V3")?;
            let assembled_only = match args.next().as_deref() {
                None => false,
                Some("--assembled-only") => true,
                Some(_) => return Err("opción de entrega V3 desconocida".into()),
            };
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            documentary_v3_io::delivery::run(
                Path::new(&packet),
                &hash,
                Path::new(&delivery),
                Path::new(&output),
                assembled_only,
            )
        }
        "select-documentary-lexical-context-v1" => {
            let input = args.next().ok_or("falta entrada de selección léxica")?;
            let trust = args.next().ok_or("falta manifiesto externo de revisión")?;
            let output = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            documentary_v3_io::lexical_clause_context::run(
                Path::new(&input),
                Path::new(&trust),
                Path::new(&output),
            )
        }
        "validate-documentary-v3" | "classify-documentary-v3" => {
            let input = args.next().ok_or("falta entrada V3")?;
            let output = args.next().ok_or("falta salida nueva V3")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            documentary_v3_io::run(
                Path::new(&input),
                Path::new(&output),
                command == "classify-documentary-v3",
            )
        }
        "prepare-typed-clause-extraction" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare_typed_clauses(
                Path::new(&d),
            )
        }
        "prepare-typed-clause-capacity" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare_typed_clause_full(
                Path::new(&d),
            )
        }
        "prepare-grounded-references" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare_grounded_references(
                Path::new(&d),
            )
        }
        "prepare-grounded-revision" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare_grounded_revision(
                Path::new(&d),
            )
        }
        "prepare-documentary-extraction-openai-guided-json" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare_openai_guided_json(
                Path::new(&d),
            )
        }
        "prepare-documentary-extraction-openai-guided-v5" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare_openai_guided_v5(
                Path::new(&d),
            )
        }
        "prepare-documentary-extraction-openai-v4" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare_openai_pilot_v4(
                Path::new(&d),
            )
        }
        "prepare-documentary-extraction-openai-v3" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare_openai_pilot_v3(
                Path::new(&d),
            )
        }
        "prepare-documentary-extraction-json-v2" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare_json_v2(Path::new(&d))
        }
        "prepare-documentary-extraction" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::prepare(Path::new(&d))
        }
        "validate-semantic-check-labels" => {
            let d = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::validate_annotations(Path::new(&d))
        }
        "report-semantic-check" => {
            let d = args.next().ok_or("falta directorio")?;
            let out = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::report(Path::new(&d), Path::new(&out))
        }
        "report-semantic-agents" => {
            let d = args.next().ok_or("falta directorio")?;
            let out = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::report_agents(Path::new(&d), Path::new(&out))
        }
        "prepare-semantic-basis-scope" => {
            let d = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::prepare_basis_scope(Path::new(&d))
        }
        "report-semantic-basis-scope" => {
            let d = args.next().ok_or("falta directorio")?;
            let out = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::report_basis_scope(Path::new(&d), Path::new(&out))
        }
        "prepare-semantic-capacity" => {
            let d = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::prepare_capacity(Path::new(&d))
        }
        "report-semantic-capacity" => {
            let d = args.next().ok_or("falta directorio")?;
            let out = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::report_capacity(Path::new(&d), Path::new(&out))
        }
        "prepare-semantic-schema" => {
            let d = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::prepare_constrained(Path::new(&d))
        }
        "report-semantic-schema" => {
            let d = args.next().ok_or("falta directorio")?;
            let out = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::report_constrained(Path::new(&d), Path::new(&out))
        }
        "prepare-semantic-coordinates" => {
            let d = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::prepare_coordinates(Path::new(&d))
        }
        "report-semantic-coordinates" => {
            let d = args.next().ok_or("falta directorio")?;
            let out = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::report_coordinates(Path::new(&d), Path::new(&out))
        }
        "prepare-semantic-presentation" => {
            let d = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::prepare_presentation(Path::new(&d))
        }
        "report-semantic-presentation" => {
            let d = args.next().ok_or("falta directorio")?;
            let out = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::report_presentation(Path::new(&d), Path::new(&out))
        }
        "prepare-semantic-check" => {
            let output = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_representation::prepare(Path::new(&output))
        }
        "audit-distributed-reference" => {
            let input = args.next().ok_or("falta entrada")?;
            let registry = args.next().ok_or("falta registro")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::reference_matrix::audit_distributed(Path::new(&input),Path::new(&registry),Path::new(&output))
        }
        "audit-reference-catalog" => {
            let input = args.next().ok_or("falta entrada")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::reference_matrix::audit(
                Path::new(&input),
                Path::new(&output),
            )
        }
        "audit-reference-coverage" => {
            let attempt = args.next().ok_or("falta ejecución")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            reference_coverage::audit(Path::new(&attempt), Path::new(&output))
        }
        "prepare-codex-entry-bank" => {
            let d = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::codex::bank::prepare(
                Path::new(&d),
            )
        }
        "normalize-codex-entry-bank" => {
            let d = args.next().ok_or("falta directorio")?;
            let out = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::codex::bank::report(
                Path::new(&d),
                Path::new(&out),
            )
        }
        "prepare-codex-entry-extraction" => {
            let d = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::codex::prepare_entry(
                Path::new(&d),
            )
        }
        "normalize-codex-extraction" => {
            let d = args.next().ok_or("falta directorio")?;
            let out = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::codex::report(
                Path::new(&d),
                Path::new(&out),
            )
        }
        "report-documentary-extraction" => {
            let d = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::automatic_extraction::report(
                Path::new(&d),
                Path::new(&output),
            )
        }
        "score-documentary-known100" => {
            let bank = args.next().ok_or("falta banco")?;
            let raw = args.next().ok_or("falta resultado")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::known_sufficiency::score(
                Path::new(&bank),
                Path::new(&raw),
                Path::new(&output),
            )
        }
        "audit-epistemic-binding-v5" => {
            let input = args.next().ok_or("falta entrada")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::audit(Path::new(&input), Path::new(&output))
        }
        "score-epistemic-binding-v5" => {
            let bank = args.next().ok_or("falta banco")?;
            let raw = args.next().ok_or("falta resultado")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::record_projection::score(
                Path::new(&bank),
                Path::new(&raw),
                Path::new(&output),
            )
        }
        "audit-evidence-state-v4" => {
            let input = args.next().ok_or("falta entrada")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::audit(Path::new(&input), Path::new(&output))
        }
        "score-evidence-state-v4" => {
            let bank = args.next().ok_or("falta banco")?;
            let raw = args.next().ok_or("falta resultado")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::v2::score(Path::new(&bank), Path::new(&raw), Path::new(&output))
        }
        "score-typed-facts" => {
            let input = args.next().ok_or("falta entrada")?;
            let raw = args.next().ok_or("falta resultado")?;
            let labels = args.next().ok_or("faltan etiquetas")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::score(
                Path::new(&input),
                Path::new(&raw),
                Path::new(&labels),
                Path::new(&output),
            )
        }
        "audit-typed-facts" => {
            let input = args.next().ok_or("falta entrada")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            typed_facts::audit(Path::new(&input), Path::new(&output))
        }
        "audit-flow-compatibility" => {
            let input = args.next().ok_or("falta entrada")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            flow_compatibility::audit(Path::new(&input), Path::new(&output))
        }
        "prepare-proposition-examples" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            proposition_examples_probe::prepare(Path::new(&directory))
        }
        "report-proposition-examples" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            proposition_examples_probe::report(Path::new(&directory), Path::new(&output))
        }
        "prepare-proposition-decisions" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            proposition_probe::prepare_decisions(Path::new(&directory))
        }
        "prepare-proposition-isolation" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            proposition_isolation_probe::prepare(Path::new(&directory))
        }
        "report-proposition-isolation" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            proposition_isolation_probe::report(Path::new(&directory), Path::new(&output))
        }
        "prepare-proposition-contribution" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            proposition_probe::prepare(Path::new(&directory))
        }
        "report-proposition-contribution" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            proposition_probe::report(Path::new(&directory), Path::new(&output))
        }
        "validate-proposition-contribution" => {
            let directory = args.next().ok_or("falta banco")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&proposition_bank::validate(Path::new(&directory))?)
                    .map_err(|e| e.to_string())?
            );
            Ok(())
        }
        "prepare-qwen-reasoning-total-deadline" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            qwen_reasoning_probe::prepare_total_deadline(Path::new(&directory))
        }
        "prepare-qwen-reasoning-pair" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            qwen_reasoning_probe::prepare(Path::new(&directory))
        }
        "report-qwen-reasoning-pair" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            qwen_reasoning_probe::report(Path::new(&directory), Path::new(&output))
        }
        "prepare-contribution-family-known" => {
            let family =
                contribution_role_contract::Family::parse(&args.next().ok_or("falta familia")?)?;
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_known_probe::prepare_family(Path::new(&directory), family)
        }
        "prepare-contribution-family-minimal" => {
            let family =
                contribution_role_contract::Family::parse(&args.next().ok_or("falta familia")?)?;
            let bank = args.next().ok_or("falta banco")?;
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_role_probe::prepare_family(Path::new(&bank), Path::new(&directory), family)
        }
        "prepare-local-isolation-diagnostic" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            local_isolation_probe::prepare(Path::new(&directory))
        }
        "report-local-isolation-diagnostic" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            local_isolation_probe::report(Path::new(&directory), Path::new(&output))
        }
        "prepare-contribution-local-known-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_known_probe::prepare_local(Path::new(&directory))
        }
        "prepare-contribution-local-probe" => {
            let bank = args.next().ok_or("falta banco")?;
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_role_probe::prepare_local(Path::new(&bank), Path::new(&directory))
        }
        "prepare-spanish-choice-known" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_known_probe::prepare_spanish_contrast(Path::new(&directory))
        }
        "prepare-negative-choice-known" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_known_probe::prepare_negative_contrast(Path::new(&directory))
        }
        "prepare-contribution-known-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_known_probe::prepare(Path::new(&directory))
        }
        "report-contribution-known-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_known_probe::report(Path::new(&directory), Path::new(&output))
        }
        "prepare-contribution-shared-probe" => {
            let bank = args.next().ok_or("falta banco")?;
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_role_probe::prepare_shared(Path::new(&bank), Path::new(&directory))
        }
        "prepare-contribution-strict-probe" => {
            let bank = args.next().ok_or("falta banco")?;
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_role_probe::prepare_strict(Path::new(&bank), Path::new(&directory))
        }
        "prepare-contribution-complete-probe" => {
            let bank = args.next().ok_or("falta banco")?;
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_role_probe::prepare_complete(Path::new(&bank), Path::new(&directory))
        }
        "prepare-contribution-role-probe" => {
            let bank = args.next().ok_or("falta banco")?;
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_role_probe::prepare(Path::new(&bank), Path::new(&directory))
        }
        "report-contribution-role-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            contribution_role_probe::report(Path::new(&directory), Path::new(&output))
        }
        "prepare-semantic-link-payloads" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_link_contract::prepare(Path::new(&directory))
        }
        "prepare-semantic-link-span-payloads" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_link_span_contract::prepare(Path::new(&directory))
        }
        "prepare-semantic-link-local-payloads" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_link_local_contract::prepare(Path::new(&directory))
        }
        "prepare-semantic-link-direct-payloads" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            semantic_link_direct_contract::prepare(Path::new(&directory))
        }
        "prepare-quoted-role-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            quoted_role_probe::prepare(Path::new(&directory))
        }
        "report-quoted-role-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            quoted_role_probe::report(Path::new(&directory), Path::new(&output))
        }
        "prepare-direct-perplexity-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            sparse_critic_probe::prepare_direct_perplexity(Path::new(&directory))
        }
        "prepare-replacement-critic-screen" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            sparse_critic_probe::replacement::prepare(Path::new(&directory))
        }
        "report-replacement-critic-screen" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            sparse_critic_probe::replacement::report(Path::new(&directory), Path::new(&output))
        }
        "prepare-http2-critic-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            sparse_critic_probe::prepare_http2(Path::new(&directory))
        }
        "prepare-targeted-critic-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            sparse_critic_probe::prepare_targeted(Path::new(&directory))
        }
        "prepare-context-critic-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            sparse_critic_probe::prepare_context(Path::new(&directory))
        }
        "prepare-sparse-critic-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            sparse_critic_probe::prepare(Path::new(&directory))
        }
        "report-sparse-critic-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            sparse_critic_probe::report(Path::new(&directory), Path::new(&output))
        }

        "report-literal-identity-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            literal_identity_probe::report(Path::new(&directory), Path::new(&output))
        }
        "prepare-independent-role-matcher" => {
            let parent = args.next().ok_or("falta padre")?;
            let directory = args.next().ok_or("falta directorio")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            independent_role_audit::prepare(Path::new(&parent), Path::new(&directory))
        }
        "audit-source-frame-review" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            source_frame_audit::report(Path::new(&directory), Path::new(&output))
        }
        "report-independent-role-matcher" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            independent_role_audit::report(Path::new(&directory), Path::new(&output))
        }
        "report-independent-projection" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            independent_projection_audit::report(Path::new(&directory), Path::new(&output))
        }
        "report-assertion-projection-census" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            assertion_census::report(Path::new(&directory), Path::new(&output))
        }
        "report-assertion-projection-sets-v4-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            assertion_sets_probe::report(Path::new(&directory), Path::new(&output))
        }
        "report-assertion-projection-ids-v3-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            assertion_ids_probe::report(Path::new(&directory), Path::new(&output))
        }
        "report-assertion-projection-tokens-v2-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            assertion_tokens_probe::report(Path::new(&directory), Path::new(&output))
        }
        "report-assertion-projection-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            assertion_probe::report(Path::new(&directory), Path::new(&output))
        }
        "report-predicate-relation-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta salida")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            predicate_probe::report(Path::new(&directory), Path::new(&output))
        }
        "report-full-prefix-dimension-probe" => {
            let directory = args.next().ok_or("falta directorio del diagnóstico")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            full_prefix_probe::report(Path::new(&directory), Path::new(&output))
        }
        "report-complete-premise-probe" => {
            let directory = args.next().ok_or("falta directorio del diagnóstico")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            complete_provider_probe::report_premise(Path::new(&directory), Path::new(&output))
        }
        "report-complete-provider-probe" => {
            let directory = args.next().ok_or("falta directorio del diagnóstico")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            complete_provider_probe::report(Path::new(&directory), Path::new(&output))
        }
        "report-opposite-task-probe" => {
            let base = args.next().ok_or("falta diagnóstico inicial")?;
            let directory = args.next().ok_or("falta diagnóstico de vía alternativa")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            opposite_task_probe::report(Path::new(&base), Path::new(&directory), Path::new(&output))
        }
        "report-isolated-role-provider-probe" => {
            let directory = args.next().ok_or("falta directorio del diagnóstico")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            isolated_provider_probe::report(Path::new(&directory), Path::new(&output))
        }
        "report-role-batch-isolation" => {
            let directory = args.next().ok_or("falta directorio del diagnóstico")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            role_batch_probe::report(Path::new(&directory), Path::new(&output))
        }
        "report-scoped-family-calibration" => {
            let d = args.next().ok_or("falta directorio")?;
            let output = args.next().ok_or("falta informe")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            family_calibration::report(Path::new(&d), Path::new(&output))
        }
        "prepare-scoped-family-probe" | "report-scoped-family-probe" => {
            let d = args.next().ok_or("falta directorio")?;
            if command.starts_with("prepare-") {
                if args.next().is_some() {
                    return Err("demasiados argumentos".into());
                }
                scoped_family_probe::prepare(Path::new(&d))
            } else {
                let output = args.next().ok_or("falta informe")?;
                if args.next().is_some() {
                    return Err("demasiados argumentos".into());
                }
                scoped_family_probe::report(Path::new(&d), Path::new(&output))
            }
        }
        "prepare-scoped-definition-probe"
        | "report-scoped-definition-probe"
        | "prepare-scoped-inline-probe"
        | "report-scoped-inline-probe"
        | "prepare-scoped-presupposition-probe"
        | "report-scoped-presupposition-probe"
        | "prepare-scoped-role-probe"
        | "report-scoped-role-probe"
        | "prepare-scoped-role-criteria-probe"
        | "report-scoped-role-criteria-probe"
        | "prepare-scoped-path-probe"
        | "report-scoped-path-probe" => {
            let directory = args.next().ok_or("falta directorio")?;
            if command.starts_with("prepare-scoped-") {
                if args.next().is_some() {
                    return Err("demasiados argumentos".into());
                }
                if command == "prepare-scoped-path-probe" {
                    scoped_definition_probe::prepare_path(Path::new(&directory))
                } else if command == "prepare-scoped-role-criteria-probe" {
                    scoped_definition_probe::prepare_role_criteria(Path::new(&directory))
                } else if command == "prepare-scoped-role-probe" {
                    scoped_definition_probe::prepare_role(Path::new(&directory))
                } else if command == "prepare-scoped-presupposition-probe" {
                    scoped_definition_probe::prepare_presupposition(Path::new(&directory))
                } else if command == "prepare-scoped-inline-probe" {
                    scoped_definition_probe::prepare_inline(Path::new(&directory))
                } else {
                    scoped_definition_probe::prepare(Path::new(&directory))
                }
            } else {
                let output = args.next().ok_or("falta informe nuevo")?;
                if args.next().is_some() {
                    return Err("demasiados argumentos".into());
                }
                if command == "report-scoped-path-probe" {
                    scoped_definition_probe::report_path(Path::new(&directory), Path::new(&output))
                } else if command == "report-scoped-role-criteria-probe" {
                    scoped_definition_probe::report_role_criteria(
                        Path::new(&directory),
                        Path::new(&output),
                    )
                } else if command == "report-scoped-role-probe" {
                    scoped_definition_probe::report_role(Path::new(&directory), Path::new(&output))
                } else if command == "report-scoped-presupposition-probe" {
                    scoped_definition_probe::report_presupposition(
                        Path::new(&directory),
                        Path::new(&output),
                    )
                } else if command == "report-scoped-inline-probe" {
                    scoped_definition_probe::report_inline(
                        Path::new(&directory),
                        Path::new(&output),
                    )
                } else {
                    scoped_definition_probe::report(Path::new(&directory), Path::new(&output))
                }
            }
        }
        "report-scoped-stability" => {
            let directory = args.next().ok_or("falta directorio del diagnóstico")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            stability::report(Path::new(&directory), Path::new(&output))
        }
        "report-scoped-preservation" => {
            let root = args.next().ok_or("falta banco")?;
            let earlier = args.next().ok_or("falta campaña anterior")?;
            let current = args.next().ok_or("falta campaña actual")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            policy_probe::report_preservation(
                Path::new(&root),
                Path::new(&earlier),
                Path::new(&current),
                Path::new(&output),
            )
        }
        "report-policy-probe"
        | "report-factor-probe"
        | "report-scoped-probe"
        | "report-scoped-v2-probe"
        | "report-scoped-v3-probe"
        | "report-scoped-anchor-probe" => {
            let directory = args.next().ok_or("falta directorio del piloto")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            if command == "report-scoped-anchor-probe" {
                policy_probe::report_scoped_anchor(Path::new(&directory), Path::new(&output))
            } else if command == "report-scoped-v3-probe" {
                policy_probe::report_scoped_flat(Path::new(&directory), Path::new(&output))
            } else if command == "report-scoped-v2-probe" {
                policy_probe::report_scoped_compact(Path::new(&directory), Path::new(&output))
            } else if command == "report-scoped-probe" {
                policy_probe::report_scoped(Path::new(&directory), Path::new(&output))
            } else if command == "report-factor-probe" {
                policy_probe::report_factors(Path::new(&directory), Path::new(&output))
            } else {
                policy_probe::report(Path::new(&directory), Path::new(&output))
            }
        }
        "evaluate-role-contract" => {
            let input = args.next().ok_or("falta banco de roles anotados")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            roles::run(Path::new(&input), Path::new(&output))
        }
        "validate-slot-contract" => {
            let input = args.next().ok_or("falta matriz de slots")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            slot_contract::run(Path::new(&input), Path::new(&output))
        }
        "prepare-revision-matrix-blind-review" => {
            let preflight = args.next().ok_or("falta directorio preflight")?;
            let arm_results = args.next().ok_or("falta arm-results.jsonl")?;
            let output = args.next().ok_or("falta directorio de salida nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            revision_matrix::prepare_blind_review(
                Path::new(&preflight),
                Path::new(&arm_results),
                Path::new(&output),
            )
        }
        "score-revision-matrix-blind-review" => {
            let preflight = args.next().ok_or("falta directorio preflight")?;
            let arm_results = args.next().ok_or("falta arm-results.jsonl")?;
            let private_mapping = args.next().ok_or("falta private/mapping.json")?;
            let reviewer_a = args.next().ok_or("falta adjudicaciones del revisor A")?;
            let reviewer_b = args.next().ok_or("falta adjudicaciones del revisor B")?;
            let adjudication = args.next().ok_or("falta desempate adjudicado")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            revision_matrix::score_blind_review(
                Path::new(&preflight),
                Path::new(&arm_results),
                Path::new(&private_mapping),
                Path::new(&reviewer_a),
                Path::new(&reviewer_b),
                Path::new(&adjudication),
                Path::new(&output),
            )
        }
        "render-utility-comparison" => {
            let report = args.next().ok_or("falta informe de comparación")?;
            let output = args.next().ok_or("falta directorio nuevo de figuras")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            charts::render_utility(Path::new(&report), Path::new(&output))
        }
        "prepare-utility-comparison" => {
            let root = args.next().ok_or("falta banco")?;
            let output = args.next().ok_or("falta directorio nuevo")?;
            let candidate = args.next().unwrap_or_else(|| "utility_v1".into());
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            evidence::prepare_utility(Path::new(&root), Path::new(&output), &candidate)
        }
        "report-utility-comparison" => {
            let root = args.next().ok_or("falta banco")?;
            let directory = args.next().ok_or("falta directorio de campaña auditada")?;
            let output = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            evidence::report_utility(Path::new(&root), Path::new(&directory), Path::new(&output))
        }
        "validate-verified-pilot" | "report-verified-pilot" => {
            let directory = args.next().ok_or("falta directorio del piloto")?;
            let output = if command == "report-verified-pilot" {
                Some(args.next().ok_or("falta informe nuevo")?)
            } else {
                None
            };
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            match output {
                Some(output) => verified::report_pilot(Path::new(&directory), Path::new(&output)),
                None => verified::validate_pilot(Path::new(&directory)),
            }
        }
        "audit-source-review" => {
            let directory = args.next().ok_or("falta directorio de revisión")?;
            let output = args.next().ok_or("falta salida nueva")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            source_review::run(Path::new(&directory), Path::new(&output))
        }
        "prepare-score-diagnostic" => {
            let root = args.next().ok_or("falta banco")?;
            let directory = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            score_diagnostic::prepare(Path::new(&root), Path::new(&directory))
        }
        "validate-score-diagnostic" | "report-score-diagnostic" => {
            let directory = args.next().ok_or("falta directorio")?;
            let output = if command == "report-score-diagnostic" {
                Some(args.next().ok_or("falta salida nueva")?)
            } else {
                None
            };
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            match output {
                Some(output) => score_diagnostic::report(Path::new(&directory), Path::new(&output)),
                None => score_diagnostic::validate(Path::new(&directory)),
            }
        }
        "report-per-memory-matrix" => {
            let directory = args.next().ok_or("falta directorio de diagnóstico")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            per_memory_matrix::run(Path::new(&directory), Path::new(&output))
        }
        "validate-per-memory-matrix" => {
            let directory = args.next().ok_or("falta directorio de diagnóstico")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            per_memory_matrix::validate(Path::new(&directory))
        }
        "report-per-memory-projection" => {
            let directory = args.next().ok_or("falta directorio de diagnóstico")?;
            let output = args.next().ok_or("falta informe nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            per_memory_matrix::run_projected(Path::new(&directory), Path::new(&output))
        }
        "validate-evidence-benchmark" => {
            let directory = args.next().ok_or("falta directorio del banco")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            evidence::validate(Path::new(&directory))
        }
        "report-evidence-retrieval" => {
            let directory = args.next().ok_or("falta directorio del banco")?;
            let results = args.next().ok_or("falta búsqueda JSONL")?;
            let output = args.next().ok_or("falta salida nueva JSON")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            evidence::report_retrieval(
                Path::new(&directory),
                Path::new(&results),
                Path::new(&output),
            )
        }
        "prepare-config-selection" => {
            let suite = args.next().ok_or("falta suite JSON")?;
            let output = args.next().ok_or("falta directorio nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            config_selection::prepare(Path::new(&suite), Path::new(&output))
        }
        "report-config-selection" => {
            let suite = args.next().ok_or("falta suite JSON")?;
            let directory = args.next().ok_or("falta directorio de campañas")?;
            let output = args.next().ok_or("falta informe JSON")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            config_selection::report(Path::new(&suite), Path::new(&directory), Path::new(&output))
        }
        "screen-search-configs" => {
            let suite = args.next().ok_or("falta suite JSON de configuraciones")?;
            let binary = args.next().ok_or("falta binario iromem")?;
            let output = args.next().ok_or("falta directorio nuevo de resultados")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para screen-search-configs".into());
            }
            config_sweep::run(Path::new(&suite), Path::new(&binary), Path::new(&output))
        }
        "audit-m5" => {
            let suite = args.next().ok_or("falta suite JSON de cierre")?;
            let output = args.next().ok_or("falta informe JSON de auditoría")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para audit-m5".into());
            }
            closeout::run(Path::new(&suite), Path::new(&output))
        }
        "retrieval" => {
            let results = args.next().ok_or("falta archivo JSONL de resultados")?;
            let queries = args.next().ok_or("falta archivo JSONL de consultas")?;
            let output = args.next();
            if args.next().is_some() {
                return Err("demasiados argumentos para retrieval".into());
            }
            retrieval::run(
                Path::new(&results),
                Path::new(&queries),
                output.as_deref().map(Path::new),
            )
        }
        "validate-benchmark" => {
            let directory = args.next().ok_or("falta directorio del banco")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para validate-benchmark".into());
            }
            benchmark::validate(Path::new(&directory))
        }
        "validate-general-memory" => {
            let directory = args.next().ok_or("falta directorio general-memory-v1")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            benchmark::validate_general_memory(Path::new(&directory))
        }
        "validate-conversation-m5-development" => {
            let directory = args
                .next()
                .ok_or("falta directorio conversation-m5-v2-development")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            benchmark::validate_conversation_m5_development(Path::new(&directory))
        }
        "validate-candidate-matrix" => {
            let directory = args.next().ok_or("falta directorio candidate-matrix-v1")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            benchmark::validate_candidate_matrix(Path::new(&directory))
        }
        "verify-hashes" => {
            let directory = args.next().ok_or("falta directorio de paquete")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            benchmark::verify_hashes(Path::new(&directory))
        }
        "select-cohort" => {
            let queries = args.next().ok_or("faltan consultas del fixture")?;
            let memories = args.next().ok_or("faltan recuerdos del fixture")?;
            let first = args
                .next()
                .ok_or("falta primer sujeto")?
                .parse::<u32>()
                .map_err(|_| "primer sujeto inválido")?;
            let last = args
                .next()
                .ok_or("falta último sujeto")?
                .parse::<u32>()
                .map_err(|_| "último sujeto inválido")?;
            let output = args.next().ok_or("falta archivo de salida nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            cohort::run(
                Path::new(&queries),
                Path::new(&memories),
                first,
                last,
                Path::new(&output),
            )
        }
        "search-benchmark" => {
            let cases = args.next().ok_or("falta archivo JSONL de casos")?;
            let binary = args.next().ok_or("falta binario iromem")?;
            let database = args.next().ok_or("falta base SQLite")?;
            let output = args
                .next()
                .ok_or("falta archivo JSONL nuevo de resultados")?;
            let split = args.next().ok_or("falta partición")?;
            let mode = args.next().ok_or("falta modo keywords o natural")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para search-benchmark".into());
            }
            search_benchmark::run(
                Path::new(&cases),
                Path::new(&binary),
                Path::new(&database),
                Path::new(&output),
                &split,
                &mode,
            )
        }
        "campaign" => {
            let cases = args.next().ok_or("falta archivo JSONL de casos")?;
            let binary = args.next().ok_or("falta binario iromem")?;
            let database = args.next().ok_or("falta base SQLite")?;
            let journal = args.next().ok_or("falta diario JSONL")?;
            let provider = args.next().ok_or("falta proveedor decisions o jev")?;
            let split = args.next().ok_or("falta partición")?;
            let max_calls = args
                .next()
                .ok_or("falta máximo técnico de llamadas")?
                .parse::<usize>()
                .map_err(|_| "máximo de llamadas inválido")?;
            let selection_policy = args.next().unwrap_or_else(|| "baseline".into());
            if args.next().is_some() {
                return Err("demasiados argumentos para campaign".into());
            }
            campaign::run(
                Path::new(&cases),
                Path::new(&binary),
                Path::new(&database),
                Path::new(&journal),
                &provider,
                &split,
                max_calls,
                &selection_policy,
            )
        }
        "campaign-report" => {
            let journal = args.next().ok_or("falta diario JSONL")?;
            let output = args.next();
            if args.next().is_some() {
                return Err("demasiados argumentos para campaign-report".into());
            }
            campaign::report(Path::new(&journal), output.as_deref().map(Path::new))
        }
        "codex-prompt" => {
            let journal = args.next().ok_or("falta diario de recall")?;
            let cases = args.next().ok_or("falta archivo JSONL de casos")?;
            let selected = args.next().ok_or("falta JSON de IDs seleccionados")?;
            let codex = args.next().ok_or("falta binario Codex")?;
            let output = args.next().ok_or("falta directorio de salida nuevo")?;
            let mode = args.next().ok_or("falta modo seed o control")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para codex-prompt".into());
            }
            codex_benchmark::run(
                Path::new(&journal),
                Path::new(&cases),
                Path::new(&selected),
                Path::new(&codex),
                Path::new(&output),
                &mode,
            )
        }
        "engram-benchmark" => {
            let cases = args.next().ok_or("falta archivo JSONL de casos")?;
            let memories = args.next().ok_or("falta fixture de recuerdos")?;
            let binary = args.next().ok_or("falta binario Engram")?;
            let data_dir = args.next().ok_or("falta directorio de datos nuevo")?;
            let output = args.next().ok_or("falta JSONL de salida nuevo")?;
            let split = args.next().ok_or("falta partición")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para engram-benchmark".into());
            }
            engram_benchmark::run(
                Path::new(&cases),
                Path::new(&memories),
                Path::new(&binary),
                Path::new(&data_dir),
                Path::new(&output),
                &split,
            )
        }
        "codex-engram" => {
            let cases = args.next().ok_or("falta JSONL de casos")?;
            let selected = args.next().ok_or("falta JSON de IDs seleccionados")?;
            let codex = args.next().ok_or("falta binario Codex")?;
            let engram = args.next().ok_or("falta binario Engram")?;
            let data_dir = args.next().ok_or("falta base Engram aislada")?;
            let output = args.next().ok_or("falta directorio de salida nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para codex-engram".into());
            }
            codex_benchmark::run_engram(
                Path::new(&cases),
                Path::new(&selected),
                Path::new(&codex),
                Path::new(&engram),
                Path::new(&data_dir),
                Path::new(&output),
            )
        }
        "render-charts" => {
            let mut inputs = Vec::new();
            for _ in 0..6 {
                inputs.push(args.next().ok_or("faltan informes para las gráficas")?);
            }
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para render-charts".into());
            }
            charts::run(
                inputs.iter().map(Path::new).collect::<Vec<_>>().as_slice(),
                Path::new(&output),
            )
        }
        "render-keyword-charts" => {
            let mut inputs = Vec::new();
            for _ in 0..6 {
                inputs.push(args.next().ok_or("faltan informes")?);
            }
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            charts::render_keyword_improvement(
                inputs.iter().map(Path::new).collect::<Vec<_>>().as_slice(),
                Path::new(&output),
            )
        }
        "render-current-charts" => {
            let search = args.next().ok_or("falta informe de búsqueda")?;
            let campaign = args.next().ok_or("falta informe de campaña")?;
            let engram = args.next().ok_or("falta informe Engram")?;
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            charts::render_current_performance(
                Path::new(&search),
                Path::new(&campaign),
                Path::new(&engram),
                Path::new(&output),
            )
        }
        "render-codex-current" => {
            let seed = args.next().ok_or("falta resumen Codex con iROMEM")?;
            let engram = args.next().ok_or("falta resumen Codex con Engram")?;
            let paired = args.next().ok_or("falta informe emparejado")?;
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            charts::render_current_codex(
                Path::new(&seed),
                Path::new(&engram),
                Path::new(&paired),
                Path::new(&output),
            )
        }
        "render-full-comparison" => {
            let aggregate = args.next().ok_or("falta agregado de la campaña")?;
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            charts::render_full_comparison(Path::new(&aggregate), Path::new(&output))
        }
        "paired-flow-report" => {
            let journal = args.next().ok_or("falta diario de recall")?;
            let seed = args.next().ok_or("falta resumen Codex con semilla")?;
            let engram = args.next().ok_or("falta resumen Codex con Engram")?;
            let control = args.next().ok_or("falta resumen Codex de control")?;
            let output = args.next().ok_or("falta JSON de salida nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            codex_benchmark::paired_report(
                Path::new(&journal),
                Path::new(&seed),
                Path::new(&engram),
                Path::new(&control),
                Path::new(&output),
            )
        }
        _ => Err("comando desconocido; consulte docs/evaluation/contract-v1.md".into()),
    }
}
