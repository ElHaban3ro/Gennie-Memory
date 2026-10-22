//! Shared opt-in operation budget. Nested adapters never reset the active counter.
use serde_json::Value;
use std::cell::RefCell;
use std::io::Write;

const MAX_STEPS: usize = 1_000_000;
const MAX_BYTES: usize = 16 * 1024 * 1024;
#[derive(Default)]
struct Budget {
    steps: usize,
    bytes: usize,
    exhausted: bool,
    nominal_enabled: bool,
    possessive_enabled: bool,
    lexical_enabled: bool,
    composition_enabled: bool,
    preflight_nodes: usize,
    preflight_evidence: usize,
    preflight_owners: usize,
}
thread_local! { static ACTIVE: RefCell<Option<Budget>> = const { RefCell::new(None) }; }
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        ACTIVE.with(|b| *b.borrow_mut() = None);
    }
}

pub(crate) fn charge(steps: usize, bytes: usize) -> bool {
    ACTIVE.with(|b| {
        let mut b = b.borrow_mut();
        let Some(b) = b.as_mut() else {
            return true;
        };
        match (b.steps.checked_add(steps), b.bytes.checked_add(bytes)) {
            (Some(s), Some(n)) if !b.exhausted && s <= MAX_STEPS && n <= MAX_BYTES => {
                b.steps = s;
                b.bytes = n;
                true
            }
            _ => {
                b.exhausted = true;
                false
            }
        }
    })
}
/// Test assertions consume actual remaining capacity after preflight accounting.
#[cfg(test)]
pub(crate) fn remaining_for_test() -> Option<(usize, usize)> {
    ACTIVE.with(|b| {
        b.borrow().as_ref().filter(|b| !b.exhausted).map(|b| {
            (
                MAX_STEPS.saturating_sub(b.steps),
                MAX_BYTES.saturating_sub(b.bytes),
            )
        })
    })
}
pub(crate) fn nominal_enabled() -> bool {
    ACTIVE.with(|b| b.borrow().as_ref().is_some_and(|b| b.nominal_enabled))
}
pub(crate) fn possessive_enabled() -> bool {
    ACTIVE.with(|b| b.borrow().as_ref().is_some_and(|b| b.possessive_enabled))
}
pub(crate) fn lexical_enabled() -> bool {
    ACTIVE.with(|b| b.borrow().as_ref().is_some_and(|b| b.lexical_enabled))
}
pub(crate) fn composition_enabled() -> bool {
    ACTIVE.with(|b| b.borrow().as_ref().is_some_and(|b| b.composition_enabled))
}
fn scalar_evidence_enabled() -> bool {
    lexical_enabled() || composition_enabled()
}
pub(crate) fn active() -> bool {
    ACTIVE.with(|b| b.borrow().is_some())
}
pub(crate) fn exhausted() -> bool {
    ACTIVE.with(|b| b.borrow().as_ref().is_some_and(|b| b.exhausted))
}
struct Count(usize);
impl Write for Count {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("size overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(crate) fn serialized_bytes(v: &Value) -> Option<usize> {
    let mut count = Count(0);
    serde_json::to_writer(&mut count, v).ok()?;
    Some(count.0)
}
fn cumulative_limits(nodes: usize, evidence: usize, owners: usize) -> bool {
    ACTIVE.with(|b| {
        let mut b = b.borrow_mut();
        let Some(b) = b.as_mut() else {
            return true;
        };
        if !b.lexical_enabled && !b.composition_enabled {
            return true;
        }
        b.preflight_nodes = b.preflight_nodes.saturating_add(nodes);
        b.preflight_evidence = b.preflight_evidence.saturating_add(evidence);
        b.preflight_owners = b.preflight_owners.saturating_add(owners);
        b.preflight_nodes <= 100_000 && b.preflight_evidence <= 4096 && b.preflight_owners <= 512
    })
}
fn preflight_nodes(inputs: &[(&Value, usize)], wrapper: bool) -> bool {
    // Only a bounded borrowed pointer stack is allocated; no input Value is cloned.
    if inputs.len() > 2 || !charge(0, inputs.len().saturating_mul(64)) {
        return false;
    }
    let mut stack = inputs.to_vec();
    let (mut nodes, mut evidence, mut owners) = (usize::from(wrapper), 0usize, 0usize);
    if wrapper && (!charge(1, 0) || !cumulative_limits(1, 0, 0)) {
        return false;
    }
    while let Some((v, depth)) = stack.pop() {
        nodes += 1;
        if nodes > 100_000 || depth > 64 || !charge(1, 0) || !cumulative_limits(1, 0, 0) {
            return false;
        }
        match v {
            Value::Array(xs) => {
                if stack
                    .len()
                    .checked_add(xs.len())
                    .is_none_or(|n| n > 100_000usize.saturating_sub(nodes))
                    || !charge(0, xs.len().saturating_mul(64))
                {
                    return false;
                }
                stack.extend(xs.iter().map(|x| (x, depth + 1)));
            }
            Value::Object(m) => {
                if stack
                    .len()
                    .checked_add(m.len())
                    .is_none_or(|n| n > 100_000usize.saturating_sub(nodes))
                    || !charge(0, m.len().saturating_mul(64))
                {
                    return false;
                }
                if scalar_evidence_enabled()
                    && m.contains_key("start_byte")
                    && m.contains_key("end_byte")
                    && m.contains_key("quote")
                {
                    evidence += 1;
                    if evidence > 4096 || !cumulative_limits(0, 1, 0) {
                        return false;
                    }
                }
                for (key, x) in m {
                    if key == "evidence"
                        || (scalar_evidence_enabled() && key == "boundary_evidence")
                    {
                        if let Some(xs) = x.as_array() {
                            if !scalar_evidence_enabled() {
                                evidence += xs.len();
                            }
                            if xs.len() > 32 || evidence > 4096 {
                                return false;
                            }
                        }
                    }
                    if matches!(key.as_str(), "claims" | "requests") {
                        let additional_owners = x.as_array().map_or(0, Vec::len);
                        owners += additional_owners;
                        if !cumulative_limits(0, 0, additional_owners) || owners > 512 {
                            return false;
                        }
                    }
                    stack.push((x, depth + 1));
                }
            }
            _ => {}
        }
    }
    true
}
fn pair_dimensions(source: &Value, question: &Value) -> bool {
    let leaves = |raw: &Value, table: &str| -> Option<(usize, usize)> {
        if raw["version"] == "documentary_reading_set_v1" {
            let xs = raw["readings"].as_array()?;
            if xs.is_empty() || xs.len() > 8 {
                return None;
            }
            Some((
                xs.len(),
                xs.iter()
                    .map(|x| x["document"][table].as_array().map_or(0, Vec::len))
                    .sum(),
            ))
        } else {
            Some((1, raw[table].as_array().map_or(0, Vec::len)))
        }
    };
    let source = source.get("raw").unwrap_or(source);
    let question = question.get("raw").unwrap_or(question);
    let (Some((ns, cs)), Some((nq, rq))) = (leaves(source, "claims"), leaves(question, "requests"))
    else {
        return false;
    };
    ns.checked_mul(nq).is_some_and(|n| n <= 64) && cs.checked_mul(rq).is_some_and(|n| n <= 65536)
}
pub(crate) fn preflight(input: &Value) -> bool {
    preflight_nodes(&[(input, 0)], false)
        && (input.get("source").is_none()
            || input.get("question").is_none()
            || pair_dimensions(&input["source"], &input["question"]))
        && serialized_bytes(input).is_some_and(|n| n <= MAX_BYTES)
}
/// Same combined bounds as a source/question wrapper, without materializing that wrapper.
pub(crate) fn with_pair_operation<T>(
    vocabulary: &Value,
    source: &Value,
    question: &Value,
    f: impl FnOnce() -> T,
) -> Result<T, String> {
    with_preflight(
        vocabulary,
        || {
            preflight_nodes(&[(source, 1), (question, 1)], true)
                && pair_dimensions(source, question)
                && serialized_bytes(source)
                    .zip(serialized_bytes(question))
                    .and_then(|(a, b)| a.checked_add(b))
                    .and_then(|n| n.checked_add("{\"question\":,\"source\":}".len()))
                    .is_some_and(|n| n <= MAX_BYTES)
        },
        f,
    )
}
/// A nested call borrows the outer operation; exhaustion invalidates its entire result.
pub(crate) fn with_operation<T>(
    vocabulary: &Value,
    input: &Value,
    f: impl FnOnce() -> T,
) -> Result<T, String> {
    with_preflight(vocabulary, || preflight(input), f)
}
fn with_preflight<T>(
    vocabulary: &Value,
    preflight: impl FnOnce() -> bool,
    f: impl FnOnce() -> T,
) -> Result<T, String> {
    if vocabulary.get("request_context_profile").is_none() {
        return Ok(f());
    }
    let outer = ACTIVE.with(|b| b.borrow().is_some());
    let _reset = if outer {
        None
    } else {
        ACTIVE.with(|b| {
            *b.borrow_mut() = Some(Budget {
                composition_enabled: super::request_context_profile::permits(
                    vocabulary,
                    "linguistic_composition_until_measurement_v1",
                ) || super::request_context_profile::permits(
                    vocabulary,
                    super::nominal_event_context::FEATURE,
                ),
                lexical_enabled: super::request_context_profile::permits(
                    vocabulary,
                    "lexical_clause_context",
                ),
                possessive_enabled: super::request_context_profile::permits(
                    vocabulary,
                    "nominal_possessive",
                ),
                nominal_enabled: super::request_context_profile::permits(
                    vocabulary,
                    "nominal_complement_de",
                ),
                ..Budget::default()
            })
        });
        Some(Reset)
    };
    if (!outer && !preflight()) || exhausted() {
        charge(MAX_STEPS + 1, 0);
        return Err("request_context_limit".into());
    }
    let result = f();
    if exhausted() {
        Err("request_context_limit".into())
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn nested_calls_share_counter_and_exhaustion_is_atomic() {
        let v = json!({"request_context_profile":{}});
        let result = with_operation(&v, &json!({}), || {
            assert!(charge(600_000, 0));
            assert!(with_operation(&v, &json!({}), || charge(500_000, 0)).is_err());
        });
        assert!(result.is_err());
        assert!(!exhausted());
        assert!(with_operation(&v, &json!({}), || charge(1, 0)).unwrap());
    }
    #[test]
    fn global_owners_and_evidence_cannot_hide_in_separate_leaves() {
        let v = json!({"request_context_profile":{}});
        let input =
            json!({"a":{"claims":vec![json!({});257]},"b":{"requests":vec![json!({});256]}});
        assert!(with_operation(&v, &input, || true).is_err());
        let input = json!({"evidence":vec![json!({});33]});
        assert!(with_operation(&v, &input, || true).is_err());
        assert!(with_operation(&json!({}), &input, || true).unwrap());
    }
    #[test]
    fn borrowed_pair_preflight_combines_limits_before_running_the_consumer() {
        let v = json!({"request_context_profile":{"version":"documentary_request_context_v1","features":["nominal_complement_de","nonagentive_nominal_event_with_contexts_v1"]},"fixed_action_rows":["remove"]});
        let source = json!({"claims":vec![json!({});257]});
        let question = json!({"requests":vec![json!({});256]});
        let mut called = false;
        assert!(
            with_pair_operation(&v, &source, &question, || {
                called = true;
            })
            .is_err()
        );
        assert!(!called);
        let source = json!({"padding":"s".repeat(MAX_BYTES/2)});
        let question = json!({"padding":"q".repeat(MAX_BYTES/2)});
        assert!(
            with_pair_operation(&v, &source, &question, || {
                called = true;
            })
            .is_err()
        );
        assert!(!called);
        let wide = json!(vec![Value::Null; 100_001]);
        assert!(
            with_pair_operation(&v, &wide, &json!({}), || {
                called = true;
            })
            .is_err()
        );
        assert!(!called);
        // Wrapper depth is part of the same bound as the previous combined input.
        let mut deep = json!(0);
        for _ in 0..64 {
            deep = json!([deep]);
        }
        assert!(
            with_pair_operation(&v, &deep, &json!({}), || {
                called = true;
            })
            .is_err()
        );
        assert!(!called);
        let source = json!({"claims":[{}]});
        let question = json!({"requests":[{}]});
        assert_eq!(
            with_pair_operation(&v, &source, &question, || 42).unwrap(),
            42
        );
        // The legacy policy still does not activate a contextual budget.
        assert!(with_pair_operation(&json!({}), &deep, &question, || true).unwrap());
    }
    #[test]
    fn materialization_bytes_are_accumulated_and_no_legacy_budget_is_active() {
        let v = json!({"request_context_profile":{}});
        assert!(
            with_operation(&v, &json!({}), || {
                let (_, remaining_bytes) = remaining_for_test().unwrap();
                assert!(charge(1, remaining_bytes));
                charge(1, 1)
            })
            .is_err()
        );
        assert!(with_operation(&json!({}), &json!({}), || charge(usize::MAX, usize::MAX)).unwrap());
    }
}
