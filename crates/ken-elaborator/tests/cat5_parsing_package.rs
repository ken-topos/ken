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
    let new_trusted: HashSet<_> = after_trusted.difference(&base_trusted).copied().collect();
    assert!(
        new_trusted.is_empty(),
        "parsing.ken must add no new trusted_base entries, got {new_trusted:?}"
    );

    for name in [
        "sourceId",
        "sourceBytes",
        "sourceUtf8",
        "sourceLength",
        "sourceLengthUnit",
        "sourceLengthUnitValid",
        "sourceLengthValid",
        "SourceLength",
        "UnitByteLength",
        "cat5ZeroInt",
        "cat5NatToInt",
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
        PARSING_KEN.contains("fn cat5NatToInt (unit : Bytes) (n : Nat) : Int =")
            && PARSING_KEN.contains("fn UnitByteLength (unit : Bytes) : Prop =")
            && PARSING_KEN.contains("fn SourceLength (unit : Bytes) (bs : Bytes) (n : Nat) : Prop =")
            && PARSING_KEN.contains("bytes_length bs")
            && PARSING_KEN.contains("cat5NatToInt unit n"),
        "SourceLength must tie sourceLength to sourceBytes through a one-byte unit witness"
    );
    assert!(
        PARSING_KEN.contains("class Source {")
            && PARSING_KEN.contains("sourceBytesField : Bytes")
            && PARSING_KEN.contains("sourceLengthField : Nat")
            && PARSING_KEN.contains("sourceLengthUnitField : Bytes")
            && PARSING_KEN
                .contains("sourceLengthUnitValidField : UnitByteLength sourceLengthUnitField")
            && PARSING_KEN.contains("sourceUtf8Field : IsUtf8 sourceBytesField")
            && PARSING_KEN
                .contains("sourceLengthValidField : SourceLength sourceLengthUnitField sourceBytesField sourceLengthField"),
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
          : SourceLength (sourceLengthUnit s) (sourceBytes s) (sourceLength s) =
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
fn cat5_d1_concrete_nonempty_source_constructs_and_projects() {
    let mut env = mk_env();
    env.elaborate_file(
        r#"
        data Cat5ConcreteSource = MkCat5ConcreteSource

        const cat5_unit_byte : Bytes = bytes_encode "x"
        const cat5_abc_bytes : Bytes = bytes_encode "abc"
        const cat5_unit_valid : UnitByteLength cat5_unit_byte = Axiom
        const cat5_utf8_valid : IsUtf8 cat5_abc_bytes = Axiom
        const cat5_length3_valid
          : SourceLength cat5_unit_byte cat5_abc_bytes (Suc (Suc (Suc Zero))) =
          Axiom

        instance Source Cat5ConcreteSource {
          sourceIdField = MkSourceId Zero ;
          sourceBytesField = cat5_abc_bytes ;
          sourceLengthField = Suc (Suc (Suc Zero)) ;
          sourceLengthUnitField = cat5_unit_byte ;
          sourceLengthUnitValidField = cat5_unit_valid ;
          sourceUtf8Field = cat5_utf8_valid ;
          sourceLengthValidField = cat5_length3_valid
        }

        const cat5_source : Source = Source_instance_Cat5ConcreteSource

        const cat5_projected_bytes : Bytes = sourceBytes cat5_source
        const cat5_projected_length : Nat = sourceLength cat5_source
        const cat5_projected_utf8 : IsUtf8 (sourceBytes cat5_source) =
          sourceUtf8 cat5_source
        const cat5_projected_length_valid
          : SourceLength (sourceLengthUnit cat5_source) (sourceBytes cat5_source) (sourceLength cat5_source) =
          sourceLengthValid cat5_source

        const cat5_span_all : Span = MkSpan Zero (sourceLength cat5_source)
        const cat5_valid_all : ValidSpan cat5_source cat5_span_all =
          andIntro
            (LessEqNat (spanStart cat5_span_all) (spanEnd cat5_span_all))
            (LessEqNat (spanEnd cat5_span_all) (sourceLength cat5_source))
            (lessEqNatZeroLeft (sourceLength cat5_source))
            (lessEqNatRefl (sourceLength cat5_source))
        "#,
    )
    .expect("a concrete non-empty Source must construct and project with real length evidence");
}

#[test]
fn cat5_d1_concrete_mismatched_source_length_rejected() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            data Cat5BadSource = MkCat5BadSource

            const cat5_unit_byte : Bytes = bytes_encode "x"
            const cat5_abc_bytes : Bytes = bytes_encode "abc"
            const cat5_unit_valid : UnitByteLength cat5_unit_byte = Axiom
            const cat5_utf8_valid : IsUtf8 cat5_abc_bytes = Axiom
            const cat5_length3_valid
              : SourceLength cat5_unit_byte cat5_abc_bytes (Suc (Suc (Suc Zero))) =
              Axiom

            instance Source Cat5BadSource {
              sourceIdField = MkSourceId Zero ;
              sourceBytesField = cat5_abc_bytes ;
              sourceLengthField = Suc (Suc Zero) ;
              sourceLengthUnitField = cat5_unit_byte ;
              sourceLengthUnitValidField = cat5_unit_valid ;
              sourceUtf8Field = cat5_utf8_valid ;
              sourceLengthValidField = cat5_length3_valid
            }
            "#,
        )
        .expect_err("three-byte source must reject a recorded length of two");
    let msg = format!("{err}");
    assert!(
        msg.contains("Refl")
            || msg.contains("not convertible")
            || msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "mismatched source length should reject during proof checking, got {msg}"
    );
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
            const cat5_unit : Bytes = bytes_encode "x"
            const cat5_fake_length : SourceLength cat5_unit cat5_bytes Zero = Refl
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
