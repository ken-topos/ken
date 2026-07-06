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
    let mut env = ElabEnv::new().expect("base env");
    let base_trusted: HashSet<GlobalId> = env.env.trusted_base().into_iter().collect();
    env.elaborate_file(PARSING_KEN)
        .expect("packages/parsing/parsing.ken must elaborate");
    let after_trusted: HashSet<GlobalId> = env.env.trusted_base().into_iter().collect();
    let mut new_trusted: HashSet<_> = after_trusted.difference(&base_trusted).copied().collect();
    new_trusted.remove(&env.class_env.record_nil_id);
    new_trusted.remove(&env.class_env.record_nil_val_id);
    assert!(
        new_trusted.is_empty(),
        "parsing.ken must add no new trusted_base entries, got {new_trusted:?}"
    );

    for name in [
        "sourceId",
        "sourceBytes",
        "sourceUtf8",
        "sourceLength",
        "sourceLengthValid",
        "SourceLength",
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
        PARSING_KEN.contains("fn IsUtf8 (bs : Bytes) : Prop =")
            && PARSING_KEN.contains("bytes_encode (bytes_decode bs)")
            && !PARSING_KEN.contains("Equal Bytes bs bs"),
        "IsUtf8 must be round-trip evidence over the source bytes, not reflexive equality"
    );
    assert!(
        PARSING_KEN.contains("fn SourceLength (bs : Bytes) (n : Nat) : Prop =")
            && PARSING_KEN.contains("bytes_length bs")
            && PARSING_KEN.contains("match n"),
        "SourceLength must tie sourceLength to sourceBytes"
    );
    assert!(
        PARSING_KEN.contains("class Source {")
            && PARSING_KEN.contains("sourceBytesField : Bytes")
            && PARSING_KEN.contains("sourceLengthField : Nat")
            && PARSING_KEN.contains("sourceUtf8Field : IsUtf8 sourceBytesField")
            && PARSING_KEN
                .contains("sourceLengthValidField : SourceLength sourceBytesField sourceLengthField"),
        "Source must be a dependent record carrying bytes, length, and both proof fields"
    );
    assert!(
        !PARSING_KEN.contains("data Source = MkSource"),
        "Source must not expose the old unconstrained MkSource constructor"
    );
    assert!(
        !PARSING_KEN.contains("sourceBytesField : String")
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
        fn cat5_span_zero_zero (s : Source) : Span = MkSpan Zero Zero

        fn cat5_valid_zero_zero (s : Source) : ValidSpan s (cat5_span_zero_zero s) =
          andIntro
            (LessEqNat (spanStart (cat5_span_zero_zero s)) (spanEnd (cat5_span_zero_zero s)))
            (LessEqNat (spanEnd (cat5_span_zero_zero s)) (sourceLength s))
            (lessEqNatRefl Zero)
            (lessEqNatZeroLeft (sourceLength s))

        fn cat5_span_zero_at_offset (offset : Nat) : Span = MkSpan offset offset

        fn cat5_valid_zero_at_offset (s : Source) (offset : Nat)
          : LessEqNat offset (sourceLength s) -> ValidSpan s (cat5_span_zero_at_offset offset) =
          \h. validZeroWidthSpan s offset h

        fn cat5_source_utf8_projects (s : Source) : IsUtf8 (sourceBytes s) =
          sourceUtf8 s

        fn cat5_source_length_valid_projects (s : Source)
          : SourceLength (sourceBytes s) (sourceLength s) =
          sourceLengthValid s

        fn cat5_located_true (s : Source) : Located Bool =
          MkLocated Bool (sourceId s) (cat5_span_zero_zero s) True

        fn cat5_valid_located_true (s : Source) : ValidLocated Bool s (cat5_located_true s) =
          andIntro
            (Equal SourceId (locatedSource Bool (cat5_located_true s)) (sourceId s))
            (ValidSpan s (locatedSpan Bool (cat5_located_true s)))
            Refl
            (cat5_valid_zero_zero s)
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
            const cat5_bad_span : Span = MkSpan Zero (Suc (Suc (Suc Zero)))
            fn cat5_bad_valid (s : Source) : ValidSpan s cat5_bad_span =
              andIntro
                (LessEqNat (spanStart cat5_bad_span) (spanEnd cat5_bad_span))
                (LessEqNat (spanEnd cat5_bad_span) (sourceLength s))
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
            const cat5_bad_span : Span = MkSpan (Suc (Suc Zero)) (Suc Zero)
            fn cat5_bad_valid (s : Source) : ValidSpan s cat5_bad_span =
              andIntro
                (LessEqNat (spanStart cat5_bad_span) (spanEnd cat5_bad_span))
                (LessEqNat (spanEnd cat5_bad_span) (sourceLength s))
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

#[test]
fn cat5_d1_old_unconstrained_source_constructor_rejected() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const cat5_sid : SourceId = MkSourceId Zero
            const cat5_bad_source : Source =
              MkSource cat5_sid (bytes_encode "abc") (Suc (Suc (Suc Zero)))
            "#,
        )
        .expect_err("old three-argument MkSource constructor must not be available");
    let msg = format!("{err}");
    assert!(
        msg.contains("unresolved type 'MkSource'")
            || msg.contains("unresolved")
            || msg.contains("UnresolvedCon"),
        "old MkSource path should reject at producer/name resolution, got {msg}"
    );
}

#[test]
fn cat5_d1_reflexive_utf8_and_length_proofs_rejected() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const cat5_bytes : Bytes = bytes_encode "abc"
            const cat5_fake_utf8 : IsUtf8 cat5_bytes = Refl
            "#,
        )
        .expect_err("IsUtf8 must not be provable by reflexive equality over arbitrary bytes");
    let msg = format!("{err}");
    assert!(
        msg.contains("Refl")
            || msg.contains("not convertible")
            || msg.contains("Type mismatch")
            || msg.contains("Kernel rejected"),
        "fake UTF-8 Refl proof should reject by proof checking, got {msg}"
    );

    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const cat5_bytes : Bytes = bytes_encode "abc"
            const cat5_fake_length : SourceLength cat5_bytes Zero = Refl
            "#,
        )
        .expect_err("SourceLength must not be provable by reflexive equality for arbitrary bytes/length");
    let msg = format!("{err}");
    assert!(
        msg.contains("Refl")
            || msg.contains("not convertible")
            || msg.contains("Type mismatch")
            || msg.contains("Kernel rejected"),
        "fake SourceLength Refl proof should reject by proof checking, got {msg}"
    );
}
