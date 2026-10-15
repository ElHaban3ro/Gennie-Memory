use crate::model::{SearchRequest, SearchResponse, SeedMemory, SeedRequest, SeedResponse};
use rusqlite::Connection;
use std::collections::HashSet;

pub fn search(conn: &Connection, request: &SearchRequest) -> Result<SearchResponse, String> {
    if request.query.trim().is_empty() || request.query.chars().count() > 8000 {
        return Err("query debe tener entre 1 y 8000 caracteres".into());
    }
    if request.scope_kind.trim().is_empty() || request.scope_id.trim().is_empty() {
        return Err("scope_kind y scope_id son obligatorios".into());
    }
    if request
        .entity_id
        .as_ref()
        .is_some_and(|id| id.trim().is_empty() || id.len() > 128)
    {
        return Err("entity_id debe tener entre 1 y 128 bytes".into());
    }
    if !valid_utc(&request.as_of) {
        return Err("as_of debe usar UTC RFC 3339 con milisegundos".into());
    }
    if !(1..=20).contains(&request.limit.unwrap_or(8)) {
        return Err("limit debe estar entre 1 y 20".into());
    }
    let expression = fts_expression(&request.query);
    if expression.is_empty() {
        return Ok(SearchResponse {
            status: "complete",
            method: "fts5",
            candidates: vec![],
            semantic_coverage: "disabled",
        });
    }
    let candidates = crate::repo::search(conn, request, &expression)?;
    Ok(SearchResponse {
        status: "complete",
        method: "fts5",
        candidates,
        semantic_coverage: "disabled",
    })
}

pub fn compose_seed(conn: &Connection, request: &SeedRequest) -> Result<SeedResponse, String> {
    if request.classifier.trim().is_empty() || request.classifier.chars().count() > 128 {
        return Err("classifier debe tener entre 1 y 128 caracteres".into());
    }
    if request.classification_status != "complete" && request.classification_status != "failed" {
        return Err("classification_status debe ser complete o failed".into());
    }
    let (method, semantic_coverage, retrieval_status, candidates): (
        &'static str,
        &'static str,
        &'static str,
        Vec<SeedMemory>,
    ) = match (&request.search, &request.hybrid) {
        (Some(text_request), None) => {
            let result = search(conn, text_request)?;
            let candidates = result
                .candidates
                .into_iter()
                .map(|candidate| SeedMemory {
                    memory_id: candidate.memory_id,
                    revision: candidate.revision,
                    scope_kind: text_request.scope_kind.clone(),
                    scope_id: text_request.scope_id.clone(),
                    summary: candidate.summary,
                    conditions: candidate.conditions,
                    source: candidate.source,
                    state: candidate.state,
                })
                .collect();
            (
                "fts5_external_classifier",
                result.semantic_coverage,
                result.status,
                candidates,
            )
        }
        (None, Some(hybrid_request)) => {
            let result = crate::embedding::search_hybrid(conn, hybrid_request)?;
            let candidates = result
                .candidates
                .into_iter()
                .map(|candidate| SeedMemory {
                    memory_id: candidate.memory_id,
                    revision: candidate.revision,
                    scope_kind: hybrid_request.search.scope_kind.clone(),
                    scope_id: hybrid_request.search.scope_id.clone(),
                    summary: candidate.summary,
                    conditions: candidate.conditions,
                    source: candidate.source,
                    state: candidate.state,
                })
                .collect();
            (
                "hybrid_external_classifier",
                result.semantic_coverage,
                result.status,
                candidates,
            )
        }
        _ => return Err("indica exactamente una búsqueda: search o hybrid".into()),
    };
    if request.presented.len() > 20 {
        return Err("presented no puede contener más de 20 candidatos".into());
    }
    let mut presented_ids = HashSet::new();
    for presented in &request.presented {
        if !presented_ids.insert(&presented.memory_id) {
            return Err("presented contiene duplicados".into());
        }
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.memory_id == presented.memory_id)
            .ok_or_else(|| {
                format!(
                    "presented_id {} no pertenece a los candidatos autorizados",
                    presented.memory_id
                )
            })?;
        if candidate.revision != presented.revision {
            return Err(format!(
                "presented_id {} tiene una revisión obsoleta",
                presented.memory_id
            ));
        }
    }
    if request.classification_status == "failed" {
        if !request.selected.is_empty() {
            return Err("un clasificador fallido no puede seleccionar memorias".into());
        }
        return Ok(SeedResponse {
            status: "degraded",
            method,
            classifier: request.classifier.clone(),
            semantic_coverage,
            memories: vec![],
            seed_text: None,
        });
    }
    if request.selected.len() > 8 {
        return Err("la semilla no puede contener más de 8 memorias".into());
    }
    let mut seen = HashSet::new();
    let mut memories = Vec::with_capacity(request.selected.len());
    for selected in &request.selected {
        if !seen.insert(&selected.memory_id) {
            return Err("selected contiene duplicados".into());
        }
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.memory_id == selected.memory_id)
            .ok_or_else(|| {
                format!(
                    "selected_id {} no pertenece a los candidatos autorizados",
                    selected.memory_id
                )
            })?;
        if candidate.revision != selected.revision {
            return Err(format!(
                "selected_id {} tiene una revisión obsoleta",
                selected.memory_id
            ));
        }
        if !presented_ids.contains(&selected.memory_id) {
            return Err(format!(
                "selected_id {} no se presentó al clasificador",
                selected.memory_id
            ));
        }
        memories.push(candidate.clone());
    }
    let seed_text = if memories.is_empty() {
        None
    } else {
        Some(
            memories
                .iter()
                .map(|item| {
                    let mut text =
                        format!("[{}@{}] {}", item.memory_id, item.revision, item.summary);
                    if let Some(conditions) = &item.conditions {
                        text.push_str(&format!("\n  Condiciones: {conditions}"));
                    }
                    if let Some(source) = &item.source {
                        text.push_str(&format!(
                            "\n  Fuente declarada: {} / {}",
                            source.kind, source.reference
                        ));
                        if let Some(locator) = &source.locator {
                            text.push_str(&format!(" / {locator}"));
                        }
                    }
                    text
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
    };
    Ok(SeedResponse {
        status: retrieval_status,
        method,
        classifier: request.classifier.clone(),
        semantic_coverage,
        memories,
        seed_text,
    })
}

pub(crate) fn valid_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(bytes.len() == 24
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'.'
        && bytes[23] == b'Z'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, byte)| matches!(i, 4 | 7 | 10 | 13 | 16 | 19 | 23) || byte.is_ascii_digit()))
    {
        return false;
    }
    let number = |start: usize, end: usize| value[start..end].parse::<u32>().unwrap_or(0);
    let year = number(0, 4);
    let month = number(5, 7);
    let day = number(8, 10);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    year > 0
        && day > 0
        && day <= days
        && number(11, 13) < 24
        && number(14, 16) < 60
        && number(17, 19) < 60
}

fn fts_expression(query: &str) -> String {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
        .collect();
    terms.join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FixtureMemory;

    #[test]
    fn search_filters_scope_and_old_revision() {
        let mut conn = Connection::open_in_memory().unwrap();
        iromem_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        let fixtures = vec![
            FixtureMemory {
                id: "m1".into(),
                scope_kind: "user".into(),
                scope_id: "u1".into(),
                entity_id: None,
                revision: 1,
                memory_type: "preference".into(),
                original_text: "Prefiero Fer".into(),
                summary: "Llamar Fer".into(),
                conditions: None,
                source: None,
                state: "corrected".into(),
                valid_from: None,
                valid_until: None,
            },
            FixtureMemory {
                id: "m1".into(),
                scope_kind: "user".into(),
                scope_id: "u1".into(),
                entity_id: None,
                revision: 2,
                memory_type: "preference".into(),
                original_text: "Prefiero Fernando".into(),
                summary: "Llamar Fernando".into(),
                conditions: None,
                source: None,
                state: "active".into(),
                valid_from: None,
                valid_until: None,
            },
            FixtureMemory {
                id: "m2".into(),
                scope_kind: "user".into(),
                scope_id: "u2".into(),
                entity_id: None,
                revision: 1,
                memory_type: "preference".into(),
                original_text: "Prefiero Fernando".into(),
                summary: "Llamar Fernando".into(),
                conditions: None,
                source: None,
                state: "active".into(),
                valid_from: None,
                valid_until: None,
            },
        ];
        crate::repo::load_fixtures(&mut conn, &fixtures).unwrap();
        let response = search(
            &conn,
            &SearchRequest {
                query: "Fernando".into(),
                scope_kind: "user".into(),
                scope_id: "u1".into(),
                entity_id: None,
                as_of: "2026-10-15T12:00:00.000Z".into(),
                limit: None,
            },
        )
        .unwrap();
        assert_eq!(response.candidates.len(), 1);
        assert_eq!(response.candidates[0].memory_id, "m1");
        assert_eq!(response.candidates[0].revision, 2);
    }

    #[test]
    fn fts_input_is_not_an_operator() {
        assert_eq!(fts_expression("Fer OR x*"), "\"Fer\" OR \"OR\" OR \"x\"");
    }

    #[test]
    fn invalid_calendar_day_is_rejected() {
        assert!(!valid_utc("2026-02-30T12:00:00.000Z"));
        assert!(valid_utc("2024-02-29T12:00:00.000Z"));
    }

    #[test]
    fn seed_rechecks_selection_against_authorized_candidates() {
        let mut conn = Connection::open_in_memory().unwrap();
        iromem_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        crate::repo::load_fixtures(
            &mut conn,
            &[
                FixtureMemory {
                    id: "allowed".into(),
                    scope_kind: "project".into(),
                    scope_id: "a".into(),
                    entity_id: None,
                    revision: 1,
                    memory_type: "fact".into(),
                    original_text: "Formato breve".into(),
                    summary: "Usar formato breve".into(),
                    conditions: None,
                    source: None,
                    state: "active".into(),
                    valid_from: None,
                    valid_until: None,
                },
                FixtureMemory {
                    id: "foreign".into(),
                    scope_kind: "project".into(),
                    scope_id: "b".into(),
                    entity_id: None,
                    revision: 1,
                    memory_type: "fact".into(),
                    original_text: "Formato extenso".into(),
                    summary: "Usar formato extenso".into(),
                    conditions: None,
                    source: None,
                    state: "active".into(),
                    valid_from: None,
                    valid_until: None,
                },
            ],
        )
        .unwrap();
        let mut request = SeedRequest {
            search: Some(SearchRequest {
                query: "formato".into(),
                scope_kind: "project".into(),
                scope_id: "a".into(),
                entity_id: None,
                as_of: "2026-10-15T12:00:00.000Z".into(),
                limit: Some(8),
            }),
            hybrid: None,
            presented: vec![crate::model::SelectedCandidate {
                memory_id: "allowed".into(),
                revision: 1,
            }],
            selected: vec![crate::model::SelectedCandidate {
                memory_id: "foreign".into(),
                revision: 1,
            }],
            classifier: "synthetic-test".into(),
            classification_status: "complete".into(),
        };
        assert!(
            compose_seed(&conn, &request)
                .unwrap_err()
                .contains("autorizados")
        );
        request.selected = vec![crate::model::SelectedCandidate {
            memory_id: "allowed".into(),
            revision: 0,
        }];
        assert!(
            compose_seed(&conn, &request)
                .unwrap_err()
                .contains("obsoleta")
        );
        request.selected[0].revision = 1;
        let response = compose_seed(&conn, &request).unwrap();
        assert_eq!(response.memories.len(), 1);
        assert_eq!(
            response.seed_text.as_deref(),
            Some("[allowed@1] Usar formato breve")
        );
        request.presented.clear();
        assert!(
            compose_seed(&conn, &request)
                .unwrap_err()
                .contains("no se presentó")
        );
    }

    #[test]
    fn seed_preserves_conditions_and_declared_source() {
        use crate::model::{FixtureMemory, SelectedCandidate, SourceRef};
        let mut conn = Connection::open_in_memory().unwrap();
        iromem_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        let source = SourceRef {
            kind: "document".into(),
            reference: "synthetic:manual@v2".into(),
            locator: Some("section-4".into()),
        };
        let fixture = FixtureMemory {
            id: "conditional".into(),
            scope_kind: "project".into(),
            scope_id: "synthetic".into(),
            entity_id: None,
            revision: 1,
            memory_type: "procedure".into(),
            original_text: "Si el cliente pide urgencia, usar canal rojo.".into(),
            summary: "Si el cliente pide urgencia, usar canal rojo.".into(),
            conditions: Some("Solo si el cliente pide urgencia".into()),
            source: Some(source.clone()),
            state: "active".into(),
            valid_from: None,
            valid_until: None,
        };
        crate::repo::load_fixtures(&mut conn, &[fixture.clone()]).unwrap();
        let search_request = SearchRequest {
            query: "urgencia".into(),
            scope_kind: "project".into(),
            scope_id: "synthetic".into(),
            entity_id: None,
            as_of: "2026-10-15T12:00:00.000Z".into(),
            limit: Some(8),
        };
        let candidates = search(&conn, &search_request).unwrap().candidates;
        assert_eq!(candidates[0].conditions, fixture.conditions);
        assert_eq!(candidates[0].source, Some(source.clone()));
        let selected = SelectedCandidate {
            memory_id: "conditional".into(),
            revision: 1,
        };
        let response = compose_seed(
            &conn,
            &SeedRequest {
                search: Some(search_request),
                hybrid: None,
                presented: vec![SelectedCandidate {
                    memory_id: selected.memory_id.clone(),
                    revision: selected.revision,
                }],
                selected: vec![selected],
                classifier: "synthetic-test".into(),
                classification_status: "complete".into(),
            },
        )
        .unwrap();
        assert_eq!(response.memories[0].conditions, fixture.conditions);
        assert_eq!(response.memories[0].source, Some(source));
        let text = response.seed_text.unwrap();
        assert!(text.contains("Condiciones: Solo si el cliente pide urgencia"));
        assert!(text.contains("Fuente declarada: document / synthetic:manual@v2 / section-4"));

        let mut invalid = fixture;
        invalid.id = "invalid".into();
        invalid.source.as_mut().unwrap().kind = "unknown".into();
        assert!(crate::repo::load_fixtures(&mut conn, &[invalid]).is_err());
    }
}
