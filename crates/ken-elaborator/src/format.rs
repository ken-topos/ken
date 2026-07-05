//! Canonical surface formatter helpers (`31 §1c`).
//!
//! This is intentionally a lexical normalizer for the D3 slice: it preserves
//! layout and comments while canonicalizing accepted ASCII operator spellings
//! to Unicode outside strings/comments.

fn ident_tail(src: &str, start: usize) -> Option<(String, usize)> {
    let mut end = start;
    let mut out = String::new();
    for (offset, c) in src[start..].char_indices() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '\'' {
            out.push(c);
            end = start + offset + c.len_utf8();
        } else {
            break;
        }
    }
    if out.is_empty() { None } else { Some((out, end)) }
}

fn canonical_ident(ident: &str) -> Option<&'static str> {
    match ident {
        "Omega" => Some("Ω"),
        "Sigma" => Some("Σ"),
        "Pi" => Some("Π"),
        "forall" => Some("∀"),
        "exists" => Some("∃"),
        "not" => Some("¬"),
        "level" | "l" => Some("ℓ"),
        _ => None,
    }
}

/// Normalize accepted ASCII notation to canonical Unicode outside strings and
/// `--` line comments. Keywords stay ASCII.
pub fn canonical_unicode(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut pos = 0;

    while pos < src.len() {
        let rest = &src[pos..];

        if rest.starts_with("--") {
            if let Some(nl) = rest.find('\n') {
                out.push_str(&rest[..=nl]);
                pos += nl + 1;
            } else {
                out.push_str(rest);
                break;
            }
            continue;
        }

        let c = rest.chars().next().unwrap();
        if c == '"' {
            out.push(c);
            pos += c.len_utf8();
            while pos < src.len() {
                let c = src[pos..].chars().next().unwrap();
                out.push(c);
                pos += c.len_utf8();
                if c == '\\' {
                    if pos < src.len() {
                        let escaped = src[pos..].chars().next().unwrap();
                        out.push(escaped);
                        pos += escaped.len_utf8();
                    }
                    continue;
                }
                if c == '"' || c == '\n' {
                    break;
                }
            }
            continue;
        }

        let replacement = if rest.starts_with("->") {
            Some(("→", 2))
        } else if rest.starts_with("=>") {
            Some(("⇒", 2))
        } else if rest.starts_with("===") {
            Some(("≡", 3))
        } else if rest.starts_with("<=") {
            Some(("≤", 2))
        } else if rest.starts_with(">=") {
            Some(("≥", 2))
        } else if rest.starts_with("/=") {
            Some(("≠", 2))
        } else if rest.starts_with("<:") {
            Some(("⊑", 2))
        } else if rest.starts_with("><") {
            Some(("×", 2))
        } else if rest.starts_with("\\/") {
            Some(("∨", 2))
        } else if rest.starts_with("/\\") {
            Some(("∧", 2))
        } else {
            None
        };
        if let Some((glyph, width)) = replacement {
            out.push_str(glyph);
            pos += width;
            continue;
        }

        if c == '\\' {
            out.push('λ');
            pos += c.len_utf8();
            continue;
        }

        if c.is_ascii_alphabetic() || c == '_' {
            if let Some((ident, end)) = ident_tail(src, pos) {
                if let Some(glyph) = canonical_ident(&ident) {
                    out.push_str(glyph);
                } else {
                    out.push_str(&ident);
                }
                pos = end;
                continue;
            }
        }

        out.push(c);
        pos += c.len_utf8();
    }

    out
}
