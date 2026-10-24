//! Explicit HTTPS acquisition; no credentials, redirects or automatic recall I/O.
use gennie_memory_kernel::digest::sha256_hex;
use gennie_memory_retrieval::source_verification::{MAX_SOURCE_BYTES, RegisterRequest};
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImportRequest {
    pub registration: RegisterRequest,
    pub url: String,
    pub expected_sha256: String,
}

fn validate(request: &ImportRequest) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(&request.url).map_err(|_| "source_import_url_invalid")?;
    let id = |value: &str| {
        !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.:".contains(&b))
    };
    if request.registration.schema_version != 1
        || !id(request.registration.actor_id.as_deref().unwrap_or("local"))
        || !id(&request.registration.scope.id)
        || !id(&request.registration.version)
        || !matches!(
            request.registration.scope.kind.as_str(),
            "user" | "agent" | "project" | "conversation"
        )
        || !matches!(
            request.registration.source.kind.as_str(),
            "user" | "document" | "tool" | "agent_discovery"
        )
        || request.url.chars().count() > 512
        || request.url.chars().any(char::is_control)
        || url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.port_or_known_default() != Some(443)
        || request.registration.source.reference != request.url
        || request.registration.source.locator.is_some()
        || !request.registration.text.is_empty()
        || request.expected_sha256.len() != 64
        || !request
            .expected_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("source_import_request_invalid".into());
    }
    Ok(url)
}

pub(crate) fn import(
    conn: &rusqlite::Connection,
    request: ImportRequest,
) -> Result<serde_json::Value, String> {
    let url = validate(&request)?;
    // The personal operator explicitly names the source. Never use the OpenRouter client,
    // its authorization, environment proxies or redirects for source acquisition.
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_| "source_import_transport_unavailable")?;
    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "text/plain, text/markdown")
        .send()
        .map_err(|_| "source_import_transport_failed")?;
    if !response.status().is_success() {
        return Err("source_import_http_failed".into());
    }
    if response
        .content_length()
        .is_some_and(|n| n > MAX_SOURCE_BYTES as u64)
    {
        return Err("source_import_size_limit".into());
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim();
    if !matches!(content_type, "text/plain" | "text/markdown") {
        return Err("source_import_content_type_unsupported".into());
    }
    let mut bytes = Vec::new();
    response
        .take(MAX_SOURCE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "source_import_read_failed")?;
    let text = checked_text(bytes, &request.expected_sha256)?;
    let mut registration = request.registration;
    registration.text = text;
    let receipt = gennie_memory_retrieval::source_verification::register(conn, &registration)?;
    Ok(
        serde_json::json!({"snapshot":receipt,"acquisition":{"profile":"explicit_https_import_sha256_v1","url":request.url,"sha256":request.expected_sha256,"transport":"https_certificate_validated","redirects":false,"author_identity":"not_assessed","factual_truth":"not_assessed"}}),
    )
}
fn checked_text(bytes: Vec<u8>, expected: &str) -> Result<String, String> {
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err("source_import_size_limit".into());
    }
    if sha256_hex(&bytes) != expected {
        return Err("source_import_hash_mismatch".into());
    }
    String::from_utf8(bytes).map_err(|_| "source_import_utf8_invalid".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn request(url: &str) -> ImportRequest {
        serde_json::from_value(serde_json::json!({"registration":{"schema_version":1,"scope":{"kind":"project","id":"synthetic"},"source":{"kind":"document","ref":url},"version":"v1","text":""},"url":url,"expected_sha256":sha256_hex(b"test")})).unwrap()
    }
    #[test]
    fn explicit_origin_rejects_credentials_redirect_surface_and_other_protocols() {
        assert!(validate(&request("https://example.com/doc")).is_ok());
        for url in [
            "http://example.com/doc",
            "file:///tmp/doc",
            "https://user:secret@example.com/doc",
            "https://example.com/doc?token=x",
            "https://example.com/doc#fragment",
            "https://example.com:444/doc",
        ] {
            assert!(validate(&request(url)).is_err(), "{url}");
        }
        let mut r = request("https://example.com/doc");
        r.registration.source.reference = "other".into();
        assert!(validate(&r).is_err());
    }
    #[test]
    fn imported_bytes_must_match_hash_utf8_and_limit() {
        assert_eq!(
            checked_text(b"test".to_vec(), &sha256_hex(b"test")).unwrap(),
            "test"
        );
        assert!(checked_text(b"other".to_vec(), &sha256_hex(b"test")).is_err());
        assert!(checked_text(vec![255], &sha256_hex(&[255])).is_err());
        let bytes = vec![b'x'; MAX_SOURCE_BYTES + 1];
        let hash = sha256_hex(&bytes);
        assert!(checked_text(bytes, &hash).is_err());
    }
}
