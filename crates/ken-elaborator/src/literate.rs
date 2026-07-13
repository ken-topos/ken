//! Literate `.ken.md` extraction.
//!
//! V1 support is intentionally byte-simple: exact column-0 fence openers are
//! classified against a fixed four-entry table (`` ```ken ``, `` ```ken
//! ignore ``, `` ```ken reject ``, `` ```ken example ``); everything else is
//! prose. The compiler input preserves byte length and newlines by blanking
//! prose (and checked-but-not-tangled bodies) with ASCII spaces.

use std::ops::Range;

use crate::error::{ElabError, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KenMdExtraction {
    pub source: String,
    pub compiled_ranges: Vec<Range<usize>>,
    /// `` ```ken reject `` block bodies (original-`src` byte offsets). Not
    /// present in `source` — must fail to elaborate against the module.
    pub reject_ranges: Vec<Range<usize>>,
    /// `` ```ken example `` block bodies (original-`src` byte offsets). Not
    /// present in `source` — must elaborate, but does not tangle into the
    /// module.
    pub example_ranges: Vec<Range<usize>>,
    /// Every complete recognized Ken fence, including `` ```ken ignore ``.
    /// These ranges are offsets into the original Markdown source and are
    /// additive to the compiler-facing extraction fields above.
    pub fences: Vec<KenMdFence>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KenMdFenceRole {
    Source,
    Ignore,
    Reject,
    Example,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KenMdFence {
    pub role: KenMdFenceRole,
    pub body_range: Range<usize>,
    pub opener_span: Span,
    pub closer_span: Span,
}

pub fn extract_ken_md(src: &str) -> Result<KenMdExtraction, ElabError> {
    let mut out: Vec<u8> = src
        .as_bytes()
        .iter()
        .map(|&b| if b == b'\n' { b'\n' } else { b' ' })
        .collect();
    let mut compiled_ranges = Vec::new();
    let mut reject_ranges = Vec::new();
    let mut example_ranges = Vec::new();
    let mut fences = Vec::new();
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
                if is_backtick_fence_line(line) {
                    match classify_fence_opener(line) {
                        FenceOpener::Source => {
                            state = FenceState::Compiled {
                                opener_start: line_start,
                                opener_end: line_end,
                                body_start: next_line,
                            };
                        }
                        FenceOpener::Ignore => {
                            state = FenceState::Ignored {
                                opener_start: line_start,
                                opener_end: line_end,
                                body_start: next_line,
                            };
                        }
                        FenceOpener::OtherLanguage => {
                            state = FenceState::ProseFence;
                        }
                        FenceOpener::Reject => {
                            state = FenceState::Checked {
                                role: CheckedRole::Reject,
                                opener_start: line_start,
                                opener_end: line_end,
                                body_start: next_line,
                            };
                        }
                        FenceOpener::Example => {
                            state = FenceState::Checked {
                                role: CheckedRole::Example,
                                opener_start: line_start,
                                opener_end: line_end,
                                body_start: next_line,
                            };
                        }
                        FenceOpener::UnrecognizedRole => {
                            return Err(ElabError::ParseError {
                                msg: format!(
                                    "unrecognized 'ken' fence role: '{}' (expected exactly \
                                     '```ken', '```ken ignore', '```ken reject', or \
                                     '```ken example')",
                                    String::from_utf8_lossy(&line[3..])
                                ),
                                span: Span::new(line_start, line_end),
                            });
                        }
                    }
                }
            }
            FenceState::ProseFence => {
                if line == b"```" {
                    state = FenceState::Prose;
                }
            }
            FenceState::Ignored {
                opener_start,
                opener_end,
                body_start,
            } => {
                if line == b"```" {
                    fences.push(KenMdFence {
                        role: KenMdFenceRole::Ignore,
                        body_range: body_start..line_start,
                        opener_span: Span::new(opener_start, opener_end),
                        closer_span: Span::new(line_start, line_end),
                    });
                    state = FenceState::Prose;
                }
            }
            FenceState::Compiled {
                opener_start,
                opener_end,
                body_start,
            } => {
                if line == b"```" {
                    out[body_start..line_start].copy_from_slice(&bytes[body_start..line_start]);
                    compiled_ranges.push(body_start..line_start);
                    fences.push(KenMdFence {
                        role: KenMdFenceRole::Source,
                        body_range: body_start..line_start,
                        opener_span: Span::new(opener_start, opener_end),
                        closer_span: Span::new(line_start, line_end),
                    });
                    state = FenceState::Prose;
                }
            }
            FenceState::Checked {
                role,
                opener_start,
                opener_end,
                body_start,
            } => {
                if line == b"```" {
                    match role {
                        CheckedRole::Reject => reject_ranges.push(body_start..line_start),
                        CheckedRole::Example => example_ranges.push(body_start..line_start),
                    }
                    fences.push(KenMdFence {
                        role: match role {
                            CheckedRole::Reject => KenMdFenceRole::Reject,
                            CheckedRole::Example => KenMdFenceRole::Example,
                        },
                        body_range: body_start..line_start,
                        opener_span: Span::new(opener_start, opener_end),
                        closer_span: Span::new(line_start, line_end),
                    });
                    state = FenceState::Prose;
                }
            }
        }

        line_start = next_line;
    }

    match state {
        FenceState::Compiled { opener_start, .. } => {
            return Err(ElabError::ParseError {
                msg: "unterminated ken fence".to_string(),
                span: Span::new(opener_start, opener_start + "```ken".len()),
            });
        }
        FenceState::Checked {
            role, opener_start, ..
        } => {
            let opener = match role {
                CheckedRole::Reject => "```ken reject",
                CheckedRole::Example => "```ken example",
            };
            return Err(ElabError::ParseError {
                msg: format!("unterminated {} fence", opener),
                span: Span::new(opener_start, opener_start + opener.len()),
            });
        }
        _ => {}
    }

    let source = String::from_utf8(out).map_err(|e| {
        ElabError::Internal(format!("ken-md extraction produced invalid UTF-8: {}", e))
    })?;
    Ok(KenMdExtraction {
        source,
        compiled_ranges,
        reject_ranges,
        example_ranges,
        fences,
    })
}

/// Format every recognized Ken fence body and splice the replacements back
/// without touching Markdown prose or fence markers.
pub fn format_ken_md(src: &str) -> Result<String, ElabError> {
    let extraction = extract_ken_md(src)?;
    let mut replacements = Vec::with_capacity(extraction.fences.len());

    for fence in &extraction.fences {
        let body = &src[fence.body_range.clone()];
        let replacement = match crate::layout::format_ken(body) {
            Ok(formatted) => formatted,
            Err(_)
                if matches!(fence.role, KenMdFenceRole::Ignore | KenMdFenceRole::Reject) =>
            {
                crate::format::canonicalize_lexed_tokens(body)?
            }
            Err(ElabError::ParseError { msg, span }) => {
                let role = match fence.role {
                    KenMdFenceRole::Source => "ken",
                    KenMdFenceRole::Example => "ken example",
                    KenMdFenceRole::Ignore | KenMdFenceRole::Reject => unreachable!(),
                };
                return Err(ElabError::ParseError {
                    msg: format!("non-parseable `{role}` fence body: {msg}"),
                    span: Span::new(
                        fence.body_range.start + span.start,
                        fence.body_range.start + span.end,
                    ),
                });
            }
            Err(error) => return Err(error),
        };
        replacements.push((fence.body_range.clone(), replacement));
    }

    let mut formatted = src.to_owned();
    replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    for (range, replacement) in replacements {
        formatted.replace_range(range, &replacement);
    }
    Ok(formatted)
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
    Ignored {
        opener_start: usize,
        opener_end: usize,
        body_start: usize,
    },
    Compiled {
        opener_start: usize,
        opener_end: usize,
        body_start: usize,
    },
    Checked {
        role: CheckedRole,
        opener_start: usize,
        opener_end: usize,
        body_start: usize,
    },
}

/// The role of a `` ```ken reject `` / `` ```ken example `` checked-but-not-
/// tangled fence (`catalog-literate-fence-roles` §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckedRole {
    Reject,
    Example,
}

fn is_backtick_fence_line(line: &[u8]) -> bool {
    line.starts_with(b"```")
}

/// The four-entry V1 fence-role table (`catalog-literate-fence-roles` §2/§3).
///
/// Byte-simple by design: the info string (everything after `` ``` ``) must
/// be *exactly* one of `ken`, `ken ignore`, `ken reject`, or `ken example` —
/// no extra tokens, no CommonMark attributes. A `ken`-prefixed opener that
/// isn't one of these four is a hard extraction-time error rather than a
/// silent fall-through to prose, so a typo'd role can never silently
/// downgrade a checked block into unchecked prose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FenceOpener {
    /// Not a `ken`-tagged fence at all (a different language, or bare
    /// ` ``` `) — unaffected, stays `ProseFence`.
    OtherLanguage,
    Source,
    Ignore,
    Reject,
    Example,
    UnrecognizedRole,
}

fn classify_fence_opener(line: &[u8]) -> FenceOpener {
    debug_assert!(is_backtick_fence_line(line));
    let info = &line[3..];
    if info == b"ken" {
        return FenceOpener::Source;
    }
    if info == b"ken ignore" {
        return FenceOpener::Ignore;
    }
    if info == b"ken reject" {
        return FenceOpener::Reject;
    }
    if info == b"ken example" {
        return FenceOpener::Example;
    }
    if info.starts_with(b"ken ") {
        return FenceOpener::UnrecognizedRole;
    }
    FenceOpener::OtherLanguage
}
