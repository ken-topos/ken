//! Literate `.ken.md` extraction.
//!
//! V1 support is intentionally byte-simple: exact column-0 ```ken fences are
//! compiled, everything else is prose. The compiler input preserves byte
//! length and newlines by blanking prose with ASCII spaces.

use std::ops::Range;

use crate::error::{ElabError, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KenMdExtraction {
    pub source: String,
    pub compiled_ranges: Vec<Range<usize>>,
}

pub fn extract_ken_md(src: &str) -> Result<KenMdExtraction, ElabError> {
    let mut out: Vec<u8> = src
        .as_bytes()
        .iter()
        .map(|&b| if b == b'\n' { b'\n' } else { b' ' })
        .collect();
    let mut compiled_ranges = Vec::new();
    let bytes = src.as_bytes();
    let mut line_start = 0usize;
    let mut state = FenceState::Prose;

    while line_start < bytes.len() {
        let line_end = match bytes[line_start..].iter().position(|&b| b == b'\n') {
            Some(offset) => line_start + offset,
            None => bytes.len(),
        };
        let next_line = if line_end < bytes.len() {
            line_end + 1
        } else {
            line_end
        };
        let line = &bytes[line_start..line_end];

        match state {
            FenceState::Prose => {
                if line == b"```ken" {
                    state = FenceState::Compiled {
                        opener_start: line_start,
                        body_start: next_line,
                    };
                } else if is_backtick_fence_line(line) {
                    state = FenceState::ProseFence;
                }
            }
            FenceState::ProseFence => {
                if line == b"```" {
                    state = FenceState::Prose;
                }
            }
            FenceState::Compiled {
                opener_start: _,
                body_start,
            } => {
                if line == b"```" {
                    out[body_start..line_start].copy_from_slice(&bytes[body_start..line_start]);
                    compiled_ranges.push(body_start..line_start);
                    state = FenceState::Prose;
                }
            }
        }

        line_start = next_line;
    }

    if let FenceState::Compiled { opener_start, .. } = state {
        return Err(ElabError::ParseError {
            msg: "unterminated ken fence".to_string(),
            span: Span::new(opener_start, opener_start + "```ken".len()),
        });
    }

    let source = String::from_utf8(out).map_err(|e| {
        ElabError::Internal(format!("ken-md extraction produced invalid UTF-8: {}", e))
    })?;
    Ok(KenMdExtraction {
        source,
        compiled_ranges,
    })
}

pub fn validate_ken_md_fences(extraction: &KenMdExtraction) -> Result<(), ElabError> {
    for range in &extraction.compiled_ranges {
        let source = extraction.source_for_range(range);
        crate::parser::parse_decls(&source)?;
    }
    Ok(())
}

impl KenMdExtraction {
    fn source_for_range(&self, keep: &Range<usize>) -> String {
        let mut bytes = self.source.as_bytes().to_vec();
        for range in &self.compiled_ranges {
            if range == keep {
                continue;
            }
            for byte in &mut bytes[range.clone()] {
                if *byte != b'\n' {
                    *byte = b' ';
                }
            }
        }
        String::from_utf8(bytes).expect("blanking compiled ranges preserves UTF-8")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FenceState {
    Prose,
    ProseFence,
    Compiled {
        opener_start: usize,
        body_start: usize,
    },
}

fn is_backtick_fence_line(line: &[u8]) -> bool {
    line.starts_with(b"```")
}
