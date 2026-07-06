//! CAT-5 D1/D2 acceptance for the parsing package source/span and parser core.
//!
//! This loads the real package file and checks the bounded D1/D2 surface:
//! byte-artifact `Source`, half-open byte `Span`, explicit validity proofs,
//! located values, total parse results, and zero trusted-base delta.

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal};
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

fn lit_to_eval(v: &NumericLitVal, mkdecimalpair_id: GlobalId) -> EvalVal {
    match v {
        NumericLitVal::Int(n) => EvalVal::from(*n),
        NumericLitVal::Float(f) => EvalVal::Float(*f),
        NumericLitVal::Float32(f) => EvalVal::Float32(*f),
        NumericLitVal::Decimal { coeff, exp } => {
            ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
        }
        NumericLitVal::Str(s) => EvalVal::Str(s.clone()),
    }
}

fn make_store(env: &ElabEnv) -> EvalStore {
    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, v) in &env.num_values {
        store.num_values.insert(*id, lit_to_eval(v, mkdecimalpair_id));
    }
    store
}

fn eval_def(env: &ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    let id = env
        .globals
        .get(name)
        .copied()
        .unwrap_or_else(|| panic!("{name} should be in scope"));
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, store),
        _ => EvalVal::Unknown,
    }
}

fn nat_count(env: &ElabEnv, v: &EvalVal) -> u64 {
    match v {
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.zero_id && args.is_empty() => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.suc_id && args.len() == 1 => {
            1 + nat_count(env, &args[0])
        }
        other => panic!("expected a Nat Ctor chain, got {other:?}"),
    }
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
        "EmptyBytes",
        "NonEmptyBytes",
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
        "errorSource",
        "errorSpan",
        "Parser",
        "ParsedValid",
        "FailedValid",
        "ParseResultValid",
        "ParserValid",
        "ParseResultTotal",
        "ParserTotal",
        "ParseResultSourceLocal",
        "ParserSourceLocal",
        "ParserLaws",
        "parserPure",
        "parserFail",
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

    for name in ["SourceId", "Source", "Span", "Located", "ParseError", "ParseResult"] {
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
fn cat5_d2_parser_result_surface_is_total_and_located() {
    assert!(
        PARSING_KEN.contains("data ParseError = MkParseError SourceId Span")
            && PARSING_KEN.contains("fn errorSource (err : ParseError) : SourceId =")
            && PARSING_KEN.contains("fn errorSpan (err : ParseError) : Span ="),
        "ParseError must carry source identity and a span with accessors"
    );
    assert!(
        PARSING_KEN.contains("data ParseResult a =")
            && PARSING_KEN.contains("Parsed a Span Nat")
            && PARSING_KEN.contains("Failed ParseError"),
        "ParseResult must be the total Parsed/Failed result surface"
    );
    assert!(
        PARSING_KEN.contains("const Parser (a : Type) : Type =")
            && PARSING_KEN
                .contains("(s : Source) -> (start : Nat) -> LessEqNat start (sourceLength s)")
            && PARSING_KEN.contains("-> ParseResult a"),
        "Parser must be total over well-formed source/start inputs"
    );
    assert!(
        PARSING_KEN.contains("fn ParsedValid")
            && PARSING_KEN.contains("Equal Nat (spanStart consumed) start")
            && PARSING_KEN.contains("Equal Nat (spanEnd consumed) next")
            && PARSING_KEN.contains("fn FailedValid")
            && PARSING_KEN.contains("Equal SourceId (errorSource err) (sourceId s)")
            && PARSING_KEN.contains("ValidSpan s (errorSpan err)")
            && PARSING_KEN.contains("fn ParserLaws"),
        "D2 laws must state success validity, failure validity, totality, and source locality"
    );
    assert!(
        !PARSING_KEN.contains("fn repeatWithFuel")
            && !PARSING_KEN.contains("fn parseResultNext")
            && !PARSING_KEN.contains("fn repeat (")
            && !PARSING_KEN.contains("fn many ("),
        "D2 repetition is deferred; the package must not export broken repeat/many helpers"
    );
    assert!(
        !PARSING_KEN.contains("= Axiom"),
        "CAT-5 D2 package must not use Axiom"
    );
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
            && PARSING_KEN.contains("fn EmptyBytes (bs : Bytes) : Prop =")
            && PARSING_KEN.contains("fn NonEmptyBytes (bs : Bytes) : Prop =")
            && PARSING_KEN.contains("fn UnitByteLength (unit : Bytes) : Prop =")
            && PARSING_KEN.contains("fn SourceLength (unit : Bytes) (bs : Bytes) (n : Nat) : Prop =")
            && PARSING_KEN.contains("bytes_concat left right")
            && PARSING_KEN.contains("NonEmptyBytes unit")
            && PARSING_KEN.contains("bytes_length bs")
            && PARSING_KEN.contains("cat5NatToInt unit n"),
        "SourceLength must tie sourceLength to sourceBytes through a non-empty byte-atomic unit witness"
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

    let mut store = make_store(&env);
    let projected_bytes = eval_def(&env, &mut store, "cat5_projected_bytes");
    assert_eq!(
        projected_bytes,
        EvalVal::Bytes(b"abc".to_vec()),
        "sourceBytes must execute through a concrete class-backed Source instance"
    );

    let projected_length = eval_def(&env, &mut store, "cat5_projected_length");
    assert_eq!(
        nat_count(&env, &projected_length),
        3,
        "sourceLength must execute through the same class-backed Source instance"
    );

    let projected_utf8 = eval_def(&env, &mut store, "cat5_projected_utf8");
    assert_eq!(
        projected_utf8,
        EvalVal::Unknown,
        "projecting a noncomputational proof field must not manufacture evidence"
    );
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
fn cat5_d1_multibyte_unit_cannot_make_three_bytes_report_one() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            data Cat5BadUnitSource = MkCat5BadUnitSource

            const cat5_abc_bytes : Bytes = bytes_encode "abc"
            const cat5_bad_unit_valid : UnitByteLength cat5_abc_bytes = Refl
            const cat5_utf8_valid : IsUtf8 cat5_abc_bytes = Axiom
            const cat5_bad_length_valid
              : SourceLength cat5_abc_bytes cat5_abc_bytes (Suc Zero) =
              Refl

            instance Source Cat5BadUnitSource {
              sourceIdField = MkSourceId Zero ;
              sourceBytesField = cat5_abc_bytes ;
              sourceLengthField = Suc Zero ;
              sourceLengthUnitField = cat5_abc_bytes ;
              sourceLengthUnitValidField = cat5_bad_unit_valid ;
              sourceUtf8Field = cat5_utf8_valid ;
              sourceLengthValidField = cat5_bad_length_valid
            }
            "#,
        )
        .expect_err("a multi-byte unit must not let three source bytes report length one");
    let msg = format!("{err}");
    assert!(
        msg.contains("Refl")
            || msg.contains("not convertible")
            || msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "multi-byte unit counterexample should reject during proof checking, got {msg}"
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

#[test]
fn cat5_d2_success_parser_carries_valid_consumed_span_from_start() {
    let mut env = mk_env();
    env.elaborate_file(
        r#"
        const cat5_success_parser : Parser Bool =
          parserPure Bool True

        const cat5_success_parser_valid : ParserValid Bool cat5_success_parser =
          \s. \start. \h.
            andIntro
              (ValidSpan s (MkSpan start start))
              (And
                (Equal Nat (spanStart (MkSpan start start)) start)
                (Equal Nat (spanEnd (MkSpan start start)) start))
              (validZeroWidthSpan s start h)
              (andIntro
                (Equal Nat (spanStart (MkSpan start start)) start)
                (Equal Nat (spanEnd (MkSpan start start)) start)
                Refl
                Refl)

        const cat5_success_parser_laws : ParserLaws Bool cat5_success_parser =
          andIntro
            (ParserValid Bool cat5_success_parser)
            (And (ParserTotal Bool cat5_success_parser) (ParserSourceLocal Bool cat5_success_parser))
            cat5_success_parser_valid
            (andIntro
              (ParserTotal Bool cat5_success_parser)
              (ParserSourceLocal Bool cat5_success_parser)
              (\s. \start. \h. tt)
              (\s. \start. \h. validZeroWidthSpan s start h))
        "#,
    )
    .expect("successful parser must carry a valid consumed span with spanStart = start");
}

#[test]
fn cat5_d2_failed_parser_carries_same_source_valid_span() {
    let mut env = mk_env();
    env.elaborate_file(
        r#"
        const cat5_failed_parser : Parser Bool =
          parserFail Bool

        const cat5_failed_parser_valid : ParserValid Bool cat5_failed_parser =
          \s. \start. \h.
            andIntro
              (Equal SourceId (errorSource (MkParseError (sourceId s) (MkSpan start start))) (sourceId s))
              (ValidSpan s (errorSpan (MkParseError (sourceId s) (MkSpan start start))))
              Refl
              (validZeroWidthSpan s start h)

        const cat5_failed_parser_source_local : ParserSourceLocal Bool cat5_failed_parser =
          \s. \start. \h. Refl
        "#,
    )
    .expect("failed parser must return a located same-source error span");
}

#[test]
fn cat5_d2_failure_with_wrong_source_rejected_by_law() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const cat5_bad_failed_parser : Parser Bool =
              \s. \start. \h.
                Failed Bool (MkParseError (MkSourceId (Suc Zero)) (MkSpan start start))

            const cat5_bad_failed_parser_valid : ParserValid Bool cat5_bad_failed_parser =
              \s. \start. \h.
                andIntro
                  (Equal SourceId (errorSource (MkParseError (MkSourceId (Suc Zero)) (MkSpan start start))) (sourceId s))
                  (ValidSpan s (errorSpan (MkParseError (MkSourceId (Suc Zero)) (MkSpan start start))))
                  Refl
                  (validZeroWidthSpan s start h)
            "#,
        )
        .expect_err("failure validity must reject an error source different from the input Source");
    let msg = format!("{err}");
    assert!(
        msg.contains("Refl")
            || msg.contains("not convertible")
            || msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "wrong-source parse failure should reject during proof checking, got {msg}"
    );
}

#[test]
fn cat5_d2_failure_with_invalid_span_rejected_by_law() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const cat5_bad_span_failed_parser : Parser Bool =
              \s. \start. \h.
                Failed Bool (MkParseError (sourceId s) (MkSpan (Suc (Suc Zero)) (Suc Zero)))

            const cat5_bad_span_failed_parser_valid : ParserValid Bool cat5_bad_span_failed_parser =
              \s. \start. \h.
                andIntro
                  (Equal SourceId (errorSource (MkParseError (sourceId s) (MkSpan (Suc (Suc Zero)) (Suc Zero)))) (sourceId s))
                  (ValidSpan s (errorSpan (MkParseError (sourceId s) (MkSpan (Suc (Suc Zero)) (Suc Zero)))))
                  Refl
                  (andIntro
                    (LessEqNat (spanStart (MkSpan (Suc (Suc Zero)) (Suc Zero))) (spanEnd (MkSpan (Suc (Suc Zero)) (Suc Zero))))
                    (LessEqNat (spanEnd (MkSpan (Suc (Suc Zero)) (Suc Zero))) (sourceLength s))
                    tt
                    (lessEqNatZeroLeft (sourceLength s)))
            "#,
        )
        .expect_err("failure validity must reject invalid error spans");
    let msg = format!("{err}");
    assert!(
        msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "invalid failure span should reject during proof checking, got {msg}"
    );
}

#[test]
fn cat5_d2_repetition_is_deferred_no_unguarded_many_or_repeat() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const cat5_zero_width_parser : Parser Bool =
              parserPure Bool True

            const cat5_unguarded_repeat : Parser (List Bool) =
              repeat Bool cat5_zero_width_parser
            "#,
        )
        .expect_err("unguarded repetition must not be exported by the D2 package");
    let msg = format!("{err}");
    assert!(
        msg.contains("unresolved type 'repeat'")
            || msg.contains("unresolved")
            || msg.contains("Unresolved"),
        "unguarded repeat should reject at producer/name resolution, got {msg}"
    );
}

#[test]
fn cat5_d2_broken_fuel_repetition_producer_path_is_not_exported() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const cat5_one_byte_parser : Parser Bool =
              \s. \start. \h.
                Parsed Bool True (MkSpan start (Suc start)) (Suc start)

            const cat5_repeat_two : Parser (List Bool) =
              repeatWithFuel Bool (Suc (Suc Zero)) cat5_one_byte_parser
            "#,
        )
        .expect_err("D2 must not export the broken fuel repetition helper");
    let msg = format!("{err}");
    assert!(
        msg.contains("unresolved type 'repeatWithFuel'")
            || msg.contains("unresolved")
            || msg.contains("Unresolved"),
        "old consuming repeatWithFuel producer path should reject at name resolution, got {msg}"
    );
}
