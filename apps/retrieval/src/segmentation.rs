//! Lossless views into source text. A boundary is not semantic independence.
use gennie_memory_kernel::digest::sha256_hex;
use serde::{Deserialize, Serialize};

pub const MAX_TEXT_BYTES: usize = 262_144;
pub const MAX_SEGMENTS: usize = 512;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SegmentRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct Segment {
    pub segment_id: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub text: String,
    pub boundary: &'static str,
    pub exceeds_soft_limit: bool,
}

#[derive(Debug, Serialize)]
pub struct Segmentation {
    pub version: &'static str,
    pub original_sha256: String,
    pub original_bytes: usize,
    pub coverage: &'static str,
    pub semantic_independence: &'static str,
    pub segments: Vec<Segment>,
    pub warnings: Vec<&'static str>,
}

pub fn segment(text: &str) -> Result<Segmentation, &'static str> {
    if text.trim().is_empty() {
        return Err("empty_text");
    }
    if text.len() > MAX_TEXT_BYTES {
        return Err("text_byte_limit");
    }
    let mut segments = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut code_ticks = 0;
    let mut brackets = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '`' {
            let mut count = 1;
            while chars.peek().is_some_and(|(_, next)| *next == '`') {
                chars.next();
                count += 1;
            }
            if code_ticks == count {
                code_ticks = 0;
            } else if code_ticks == 0 {
                code_ticks = count;
            }
            continue;
        }
        if code_ticks != 0 {
            continue;
        }
        if let Some(close) = quote {
            let slashes = text[..i].bytes().rev().take_while(|b| *b == b'\\').count();
            if c == close && slashes % 2 == 0 {
                quote = None;
            }
            continue;
        }
        quote = match c {
            '"' => Some('"'),
            '“' => Some('”'),
            '«' => Some('»'),
            '\'' if i == 0
                || text[..i]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace) =>
            {
                Some('\'')
            }
            _ => None,
        };
        if quote.is_some() {
            continue;
        }
        match c {
            '(' => brackets.push(')'),
            '[' => brackets.push(']'),
            '{' => brackets.push('}'),
            ')' | ']' | '}' if brackets.last() == Some(&c) => {
                brackets.pop();
            }
            _ => {}
        }
        if !brackets.is_empty() {
            continue;
        }
        let end = i + c.len_utf8();
        let rest = &text[end..];
        let paragraph = c == '\n' && rest.starts_with(['\n', '\r']);
        let sentence = matches!(c, '.' | '?' | '!')
            && rest.chars().next().is_some_and(char::is_whitespace)
            && (c != '.' || safe_period(&text[start..i]));
        if paragraph || sentence {
            let whitespace_end = rest
                .char_indices()
                .find(|(_, ch)| !ch.is_whitespace())
                .map_or(text.len(), |(offset, _)| end + offset);
            let boundary = if paragraph {
                "paragraph"
            } else {
                "sentence_candidate"
            };
            push(&mut segments, text, start, whitespace_end, boundary)?;
            start = whitespace_end;
            while chars.peek().is_some_and(|(pos, _)| *pos < start) {
                chars.next();
            }
        }
    }
    if start < text.len() {
        push(&mut segments, text, start, text.len(), "end_of_input")?;
    }
    let mut warnings = Vec::new();
    if quote.is_some() || code_ticks != 0 || !brackets.is_empty() {
        warnings.push("unclosed_delimiter_preserved");
    }
    if segments.iter().any(|s| s.exceeds_soft_limit) {
        warnings.push("long_segment_preserved");
    }
    Ok(Segmentation {
        version: "literal_segments_v1",
        original_sha256: sha256_hex(text.as_bytes()),
        original_bytes: text.len(),
        coverage: "complete",
        semantic_independence: "not_assessed",
        segments,
        warnings,
    })
}

fn push(
    out: &mut Vec<Segment>,
    text: &str,
    start: usize,
    end: usize,
    boundary: &'static str,
) -> Result<(), &'static str> {
    if out.len() == MAX_SEGMENTS {
        return Err("segment_count_limit");
    }
    out.push(Segment {
        segment_id: out.len(),
        start_byte: start,
        end_byte: end,
        text: text[start..end].into(),
        boundary,
        exceeds_soft_limit: end - start > 512,
    });
    Ok(())
}

fn safe_period(prefix: &str) -> bool {
    let word = prefix.split_whitespace().next_back().unwrap_or("");
    if word.contains(['.', '/', '@', ':', '\\']) || word.chars().any(char::is_numeric) {
        return false;
    }
    let word = word.to_lowercase();
    !matches!(
        word.as_str(),
        "etc" | "aprox" | "pág" | "págs" | "núm" | "ud" | "uds" | "dra" | "prof" | "ing" | "lic"
    ) && word.chars().count() > 3
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lossless_unicode_and_conditions() {
        let text =
            "  Si llueve, no enviar.\r\n¿Y mañana?  Sí, únicamente con permiso.\n\nFin 🦀.  ";
        let result = segment(text).unwrap();
        assert!(result.segments.len() >= 3);
        assert_eq!(
            result
                .segments
                .iter()
                .map(|s| s.text.as_str())
                .collect::<String>(),
            text
        );
        let mut end = 0;
        for s in result.segments {
            assert_eq!(s.start_byte, end);
            assert_eq!(&text[s.start_byte..s.end_byte], s.text);
            end = s.end_byte;
        }
        assert_eq!(end, text.len());
    }
    #[test]
    fn protects_abbreviations_numbers_urls_quotes_and_code() {
        for text in [
            "El Dr. Pérez pagó 3.14 euros.",
            "Ver https://a.example/r. Después consultar.",
            "Dijo: \"No enviar. Si llueve.\"",
            "Usar `x. y` (sin salir. Todavía).",
            "```rust\nlet a = 2.0;\n\n// sigue. Igual\n```",
            "Correo a@b.org. Luego esperar.",
        ] {
            let r = segment(text).unwrap();
            assert_eq!(r.segments.len(), 1, "{text}");
            assert_eq!(r.segments[0].text, text);
        }
    }
    #[test]
    fn rejects_limits_instead_of_returning_partial_success() {
        assert_eq!(segment("  ").unwrap_err(), "empty_text");
        assert_eq!(
            segment(&"x".repeat(MAX_TEXT_BYTES + 1)).unwrap_err(),
            "text_byte_limit"
        );
        assert_eq!(
            segment(&"Palabra. ".repeat(MAX_SEGMENTS + 1)).unwrap_err(),
            "segment_count_limit"
        );
        // Long whitespace runs must be scanned only at actual boundaries.
        let spaced = format!("x{}y", " ".repeat(MAX_TEXT_BYTES - 2));
        let spaced_result = segment(&spaced).unwrap();
        assert_eq!(spaced_result.segments.len(), 1);
        assert_eq!(spaced_result.segments[0].text, spaced);
        let long = segment(&"a".repeat(1024)).unwrap();
        assert_eq!(long.segments.len(), 1);
        assert!(long.warnings.contains(&"long_segment_preserved"));
    }
}
