//! Mechanical UTF8 evidence coordinates; the author owns semantic attribution.
use super::packet::{LIMIT, read_limited};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    text: String,
    quote: String,
    // Value keeps the field mandatory while allowing explicit null.
    context: Value,
}

fn unique_offset(text: &str, needle: &str, label: &str) -> Result<usize, String> {
    if needle.trim().is_empty() {
        return Err(format!("{label} vacío"));
    }
    let first = text
        .find(needle)
        .ok_or_else(|| format!("{label} ausente en el texto elegido"))?;
    // Restart after one codepoint, not the whole match: overlaps remain ambiguous.
    let next = first + text[first..].chars().next().unwrap().len_utf8();
    if text[next..].contains(needle) {
        return Err(format!("{label} ambiguo: se requiere una ocurrencia única"));
    }
    Ok(first)
}

/// Exact byte offsets only. A unique context is not proof of correct attribution.
pub fn locate(text: &str, quote: &str, context: Option<&str>) -> Result<Value, String> {
    if text.len() > LIMIT || quote.len() > LIMIT || context.is_some_and(|c| c.len() > LIMIT) {
        return Err("texto de evidencia supera el límite local de 16 MiB".into());
    }
    if text.trim().is_empty() {
        return Err("texto de evidencia vacío".into());
    }
    let start = if let Some(context) = context {
        let context_start = unique_offset(text, context, "contexto")?;
        context_start + unique_offset(context, quote, "cita")?
    } else {
        unique_offset(text, quote, "cita")?
    };
    Ok(json!({"start_byte":start,"end_byte":start + quote.len(),"quote":quote}))
}

pub fn run(input: &Path, output: &Path) -> Result<(), String> {
    let bytes = read_limited(input)?;
    let input: Input = serde_json::from_slice(&bytes)
        .map_err(|_| "JSON de evidencia inválido: exige text, quote y context (string o null)")?;
    let context = match &input.context {
        Value::Null => None,
        Value::String(s) => Some(s.as_str()),
        _ => return Err("contexto de evidencia debe ser string o null".into()),
    };
    let evidence = locate(&input.text, &input.quote, context)?;
    super::write_report(output, &evidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_quote_requires_author_context() {
        let text = "Duna es impresora; Loma es impresora.";
        assert!(
            locate(text, "impresora", None)
                .unwrap_err()
                .contains("ambiguo")
        );
        let result = locate(text, "impresora", Some("Loma es impresora.")).unwrap();
        assert_eq!(
            result,
            json!({"start_byte":27,"end_byte":36,"quote":"impresora"})
        );
        assert!(locate(text, "Duna", Some("Loma es impresora.")).is_err());
    }

    #[test]
    fn utf8_coordinates_preserve_exact_original_bytes() {
        let text = "ñ 🙂: café; otra café";
        let result = locate(text, "café", Some("otra café")).unwrap();
        let start = result["start_byte"].as_u64().unwrap() as usize;
        let end = result["end_byte"].as_u64().unwrap() as usize;
        assert_eq!((start, end), (21, 26));
        assert_eq!(&text.as_bytes()[start..end], "café".as_bytes());
        assert!(locate(text, "cafe\u{301}", Some("otra café")).is_err());
    }

    #[test]
    fn repeated_context_and_overlapping_occurrences_are_ambiguous() {
        assert!(locate("x café; x café", "café", Some("x café")).is_err());
        assert!(locate("aaa", "aa", None).is_err());
        assert!(locate("aaa", "a", Some("aa")).is_err());
        assert!(locate("aaaa!", "aa", Some("aaaa!")).is_err());
    }

    #[test]
    fn absent_empty_and_oversized_inputs_are_rejected() {
        for (text, quote, context) in [
            ("", "x", None),
            ("x", "", None),
            ("x", " ", None),
            ("x", "y", None),
            ("x", "x", Some("")),
            ("x", "x", Some("y")),
        ] {
            assert!(locate(text, quote, context).is_err());
        }
        assert!(locate(&"x".repeat(LIMIT + 1), "x", None).is_err());
    }

    #[test]
    fn file_adapter_enforces_closed_input_and_never_overwrites() {
        let dir = std::env::temp_dir().join(format!(
            "gennie-memory-evidence-locator-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let input = dir.join("input.json");
        let output = dir.join("output.json");
        for invalid in [
            r#"{"text":"x","quote":"x"}"#,
            r#"{"text":"x","quote":"x","context":null,"gold":true}"#,
            r#"{"text":"x","quote":"x","context":null,"text":"y"}"#,
            r#"{"text":"x","quote":"x","context":42}"#,
        ] {
            std::fs::write(&input, invalid).unwrap();
            assert!(run(&input, &output).is_err());
            assert!(!output.exists());
        }
        std::fs::write(&input, r#"{"text":"x","quote":"x","context":null}"#).unwrap();
        run(&input, &output).unwrap();
        let original = std::fs::read(&output).unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&original).unwrap(),
            json!({"start_byte":0,"end_byte":1,"quote":"x"})
        );
        assert!(run(&input, &output).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), original);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
