//! Closed, opt-in configuration-request vocabulary contract.
use serde_json::Value;
use std::collections::BTreeSet;
pub(super) const VERSION: &str = "documentary_configuration_slots_v1";
pub(super) fn valid(vocabulary: &Value) -> bool {
    let Some(profile) = vocabulary.get("configuration_request_profile") else {
        return true;
    };
    let Some(object) = profile.as_object() else {
        return false;
    };
    if object.len() != 2 || profile["version"] != VERSION {
        return false;
    }
    let Some(keys) = profile["keys"].as_array() else {
        return false;
    };
    if keys.is_empty() || keys.len() > 128 {
        return false;
    }
    let mut seen = BTreeSet::new();
    keys.iter().all(|key| {
        key.as_str().is_some_and(|key| {
            !key.is_empty()
                && key.len() <= 128
                && key
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                && seen.insert(key)
        })
    })
}
pub(super) fn permits(vocabulary: &Value, key: &str) -> bool {
    valid(vocabulary)
        && vocabulary.get("configuration_request_profile").is_some()
        && vocabulary["configuration_request_profile"]["keys"]
            .as_array()
            .is_some_and(|keys| {
                keys.iter()
                    .any(|registered| registered.as_str() == Some(key))
            })
}
