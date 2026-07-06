//! CAT-5 D1 acceptance for the parsing package source/span core.
//!
//! This loads the real package file and checks the bounded D1 surface only:
//! byte-artifact `Source`, half-open byte `Span`, explicit validity proofs,
//! located values, and zero trusted-base delta.

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv};
use ken_kernel::Decl;
use ken_kernel::GlobalId;
use std::collections::HashSet;

const PARSING_KEN: &str = include_str!("../../../packages/parsing/parsing.ken");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(PARSING_KEN)
        .expect("packages/parsing/parsing.ken must elaborate");
    env
}

#[test]
fn cat5_d1_source_span_package_elaborates_zero_delta() {
    let base_env = ElabEnv::new().expect("base env");
    let base_trusted: HashSet<GlobalId> = base_env.env.trusted_base().into_iter().collect();
    let env = mk_env();
    let after_trusted: HashSet<GlobalId> = env.env.trusted_base().into_iter().collect();
    let new_trusted: HashSet<_> = after_trusted.difference(&base_trusted).collect();
    assert!(
        new_trusted.is_empty(),
        "parsing.ken must add no new trusted_base entries, got {new_trusted:?}"
    );

    for name in [
        "sourceId",
        "sourceBytes",
        "sourceUtf8",
        "sourceLength",
        "spanStart",
        "spanEnd",
        "cat5LeqNat",
        "LessEqNat",
        "lessEqNatRefl",
        "lessEqNatZeroLeft",
        "ValidSpan",
        "validZeroWidthSpan",
        "locatedSource",
        "locatedSpan",
        "locatedValue",
        "ValidLocated",
    ] {
        let id = env
            .globals
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("{name} should be exported by parsing.ken"));
        match env.env.lookup(id) {
            Some(Decl::Transparent { .. }) => {}
            other => panic!("{name} must be a transparent checked definition, got {other:?}"),
        }
        let delta = trusted_base_delta(&env.env, id);
        let new_delta: HashSet<_> = delta.difference(&base_trusted).collect();
        assert!(
            new_delta.is_empty(),
            "{name} must add zero new trusted_base delta, got {new_delta:?}"
        );
    }

    for name in ["SourceId", "Source", "Span", "Located"] {
        let id = env
            .globals
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("{name} should be exported by parsing.ken"));
        assert!(
            !env.env.trusted_base().contains(&id),
            "{name}'s type id must never enter trusted_base()"
        );
    }
}

#[test]
fn cat5_d1_source_span_surface_is_byte_artifact_and_source_explicit() {
    assert!(
        PARSING_KEN.contains("data Source = MkSource SourceId Bytes Nat"),
        "Source must be byte-artifact based with explicit byte length"
    );
    assert!(
        !PARSING_KEN.contains("data Source = MkSource SourceId String")
            && !PARSING_KEN.contains("String Nat"),
        "Source must not use normalized String as the offset basis"
    );
    assert!(
        PARSING_KEN.contains("data Span = MkSpan Nat Nat"),
        "Span must carry only byte endpoints"
    );
    assert!(
        PARSING_KEN.contains("data Located a = MkLocated SourceId Span a")
            && PARSING_KEN.contains("fn ValidLocated"),
        "source identity must be supplied by Located/validity, not by bare Span"
    );
    assert!(
        !PARSING_KEN.contains("= Axiom"),
        "CAT-5 D1 package must not use Axiom"
    );
}

#[test]
fn cat5_d1_valid_half_open_bounds_and_zero_width_offsets_check() {
    let mut env = mk_env();
    env.elaborate_file(
        r#"
        const cat5_sid : SourceId = MkSourceId Zero
        const cat5_source : Source =
          MkSource cat5_sid (bytes_encode "abc") (Suc (Suc (Suc Zero)))

        const cat5_span_zero_at_one : Span = MkSpan (Suc Zero) (Suc Zero)
        const cat5_valid_zero_at_one : ValidSpan cat5_source cat5_span_zero_at_one =
          andIntro
            (LessEqNat (spanStart cat5_span_zero_at_one) (spanEnd cat5_span_zero_at_one))
            (LessEqNat (spanEnd cat5_span_zero_at_one) (sourceLength cat5_source))
            tt
            tt

        const cat5_span_one_two : Span = MkSpan (Suc Zero) (Suc (Suc Zero))
        const cat5_valid_one_two : ValidSpan cat5_source cat5_span_one_two =
          andIntro
            (LessEqNat (spanStart cat5_span_one_two) (spanEnd cat5_span_one_two))
            (LessEqNat (spanEnd cat5_span_one_two) (sourceLength cat5_source))
            tt
            tt

        const cat5_located_true : Located Bool =
          MkLocated Bool (sourceId cat5_source) cat5_span_one_two True

        const cat5_valid_located_true : ValidLocated Bool cat5_source cat5_located_true =
          andIntro
            (Equal SourceId (locatedSource Bool cat5_located_true) (sourceId cat5_source))
            (ValidSpan cat5_source (locatedSpan Bool cat5_located_true))
            tt
            cat5_valid_one_two
        "#,
    )
    .expect("valid half-open and zero-width spans should check");
}

#[test]
fn cat5_d1_end_past_source_length_rejected() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const cat5_sid : SourceId = MkSourceId Zero
            const cat5_source : Source =
              MkSource cat5_sid (bytes_encode "ab") (Suc (Suc Zero))
            const cat5_bad_span : Span = MkSpan Zero (Suc (Suc (Suc Zero)))
            const cat5_bad_valid : ValidSpan cat5_source cat5_bad_span =
              andIntro
                (LessEqNat (spanStart cat5_bad_span) (spanEnd cat5_bad_span))
                (LessEqNat (spanEnd cat5_bad_span) (sourceLength cat5_source))
                tt
                tt
            "#,
        )
        .expect_err("end > sourceLength must not typecheck as a ValidSpan");
    let msg = format!("{err}");
    assert!(
        msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "bad end bound should reject during proof checking, got {msg}"
    );
}

#[test]
fn cat5_d1_start_after_end_rejected() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const cat5_sid : SourceId = MkSourceId Zero
            const cat5_source : Source =
              MkSource cat5_sid (bytes_encode "ab") (Suc (Suc Zero))
            const cat5_bad_span : Span = MkSpan (Suc (Suc Zero)) (Suc Zero)
            const cat5_bad_valid : ValidSpan cat5_source cat5_bad_span =
              andIntro
                (LessEqNat (spanStart cat5_bad_span) (spanEnd cat5_bad_span))
                (LessEqNat (spanEnd cat5_bad_span) (sourceLength cat5_source))
                tt
                tt
            "#,
        )
        .expect_err("start > end must not typecheck as a ValidSpan");
    let msg = format!("{err}");
    assert!(
        msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "bad start/end ordering should reject during proof checking, got {msg}"
    );
}
