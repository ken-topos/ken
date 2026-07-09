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

const PARSING_KEN: &str = include_str!("../../../catalog/packages/parsing/parsing.ken");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(PARSING_KEN)
        .expect("catalog/packages/parsing/parsing.ken must elaborate");
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
        store
            .num_values
            .insert(*id, lit_to_eval(v, mkdecimalpair_id));
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

fn ctor_args<'a>(env: &ElabEnv, v: &'a EvalVal, ctor: &str) -> &'a [EvalVal] {
    let expected = env
        .globals
        .get(ctor)
        .copied()
        .unwrap_or_else(|| panic!("{ctor} should be in scope"));
    match v {
        EvalVal::Ctor { id, args, .. } if *id == expected => args.as_ref().as_slice(),
        other => panic!("expected {ctor}, got {other:?}"),
    }
}

fn span_bounds(env: &ElabEnv, v: &EvalVal) -> (u64, u64) {
    let args = ctor_args(env, v, "MkSpan");
    assert_eq!(
        args.len(),
        2,
        "MkSpan must have start/end args, got {args:?}"
    );
    (nat_count(env, &args[0]), nat_count(env, &args[1]))
}

fn located_span<'a>(env: &ElabEnv, v: &'a EvalVal) -> &'a EvalVal {
    let args = ctor_args(env, v, "MkLocated");
    assert!(
        args.len() >= 3,
        "MkLocated must carry type/source/span/value args, got {args:?}"
    );
    &args[2]
}

fn syntax_root_and_children<'a>(env: &ElabEnv, v: &'a EvalVal) -> (&'a EvalVal, &'a EvalVal) {
    let args = ctor_args(env, v, "MkSyntax");
    assert!(
        args.len() >= 3,
        "MkSyntax must carry type/root/children args, got {args:?}"
    );
    (&args[1], &args[2])
}

fn collect_located_list_spans(env: &ElabEnv, v: &EvalVal, out: &mut Vec<(u64, u64)>) {
    let nil_id = env.globals["Nil"];
    let cons_id = env.globals["Cons"];
    match v {
        EvalVal::Ctor { id, .. } if *id == nil_id => {}
        EvalVal::Ctor { id, args, .. } if *id == cons_id => {
            assert!(
                args.len() >= 3,
                "Cons must carry type/head/tail args, got {args:?}"
            );
            out.push(span_bounds(env, located_span(env, &args[1])));
            collect_located_list_spans(env, &args[2], out);
        }
        other => panic!("expected List (Located _), got {other:?}"),
    }
}

fn syntax_spans(env: &ElabEnv, v: &EvalVal) -> Vec<(u64, u64)> {
    let (root, children) = syntax_root_and_children(env, v);
    let mut out = vec![span_bounds(env, located_span(env, root))];
    collect_located_list_spans(env, children, &mut out);
    out
}

fn nat_expr(n: usize) -> String {
    let mut s = "Zero".to_string();
    for _ in 0..n {
        s = format!("Suc ({s})");
    }
    s
}

fn neutralize_fixture_proofs(env: &ElabEnv, store: &mut EvalStore, names: &[&str]) {
    for name in names {
        let id = env
            .globals
            .get(*name)
            .copied()
            .unwrap_or_else(|| panic!("{name} should be in scope"));
        store.num_values.insert(id, EvalVal::Neutral);
    }
}

#[test]
fn cat5_d1_source_span_package_elaborates_zero_delta() {
    let mut env = ElabEnv::new().expect("base env");
    let base_trusted: HashSet<GlobalId> = env.env.trusted_base().into_iter().collect();
    env.elaborate_file(PARSING_KEN)
        .expect("catalog/packages/parsing/parsing.ken must elaborate");
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
        "byteUnitZeroInt",
        "byteUnitNatToInt",
        "spanStart",
        "spanEnd",
        "natLeqBool",
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
        "syntaxRoot",
        "syntaxChildren",
        "eraseSpans",
        "listAppend",
        "ValidLocatedList",
        "ValidSyntax",
        "boolExprEq",
        "natEqBool",
        "natAdd",
        "natLtBool",
        "boolAnd",
        "sourceByteEq",
        "sourceByteEqAt",
        "startsTrueToken",
        "startsFalseToken",
        "startsNotOpenToken",
        "startsAndOpenToken",
        "skipSpacesFuel",
        "skipSpaces",
        "syntaxLeaf",
        "syntaxNodeUnary",
        "syntaxNodeBinary",
        "parseBoolExprAtFuel",
        "parseBoolExpr",
        "printBoolExpr",
        "formatBoolExpr",
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

    for name in [
        "SourceId",
        "Source",
        "Span",
        "Located",
        "ParseError",
        "ParseResult",
        "BoolExpr",
        "Syntax",
    ] {
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
fn cat5_d3_bool_expression_surface_is_package_owned() {
    assert!(
        PARSING_KEN.contains("data BoolExpr =")
            && PARSING_KEN.contains("BTrue")
            && PARSING_KEN.contains("BFalse")
            && PARSING_KEN.contains("BNot BoolExpr")
            && PARSING_KEN.contains("BAnd BoolExpr BoolExpr"),
        "D3 must expose the package-owned BoolExpr data surface"
    );
    assert!(
        PARSING_KEN.contains("data Syntax a = MkSyntax (Located a) (List (Located a))")
            && PARSING_KEN.contains("fn eraseSpans (x : Syntax BoolExpr) : BoolExpr =")
            && PARSING_KEN.contains("fn ValidSyntax"),
        "D3 Syntax must be package-owned located syntax, not compiler AST"
    );
    assert!(
        PARSING_KEN.contains("const parseBoolExpr : Parser (Syntax BoolExpr) =")
            && PARSING_KEN.contains("fn printBoolExpr (e : BoolExpr) : Bytes =")
            && PARSING_KEN.contains("fn formatBoolExpr (s : Source) : Result ParseError Bytes ="),
        "D3 must export parser, printer, and formatter with the pinned types"
    );
    assert!(
        PARSING_KEN.contains("bytes_at (sourceBytes s)")
            && PARSING_KEN.contains("bytes_encode \"true\"")
            && PARSING_KEN.contains("bytes_encode \"false\"")
            && PARSING_KEN.contains("bytes_encode \"(not \"")
            && PARSING_KEN.contains("bytes_encode \"(and \""),
        "D3 must operate over Source bytes and canonical ASCII token bytes"
    );
    assert!(
        !PARSING_KEN.contains("compiler")
            && !PARSING_KEN.contains("AST")
            && !PARSING_KEN.contains("= Axiom"),
        "D3 package surface must not route through compiler ASTs or package axioms"
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
        PARSING_KEN.contains("fn byteUnitNatToInt (unit : Bytes) (n : Nat) : Int =")
            && PARSING_KEN.contains("fn EmptyBytes (bs : Bytes) : Prop =")
            && PARSING_KEN.contains("fn NonEmptyBytes (bs : Bytes) : Prop =")
            && PARSING_KEN.contains("fn UnitByteLength (unit : Bytes) : Prop =")
            && PARSING_KEN.contains("fn SourceLength (unit : Bytes) (bs : Bytes) (n : Nat) : Prop =")
            && PARSING_KEN.contains("bytes_concat left right")
            && PARSING_KEN.contains("NonEmptyBytes unit")
            && PARSING_KEN.contains("bytes_length bs")
            && PARSING_KEN.contains("byteUnitNatToInt unit n"),
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
        !PARSING_KEN.contains("sourceBytesField : String") && !PARSING_KEN.contains("String Nat"),
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
        fn zero_width_span_at_start (s : Source) : Span = MkSpan Zero Zero

        fn valid_zero_width_span_at_start (s : Source) : ValidSpan s (zero_width_span_at_start s) =
          andIntro
            (LessEqNat (spanStart (zero_width_span_at_start s)) (spanEnd (zero_width_span_at_start s)))
            (LessEqNat (spanEnd (zero_width_span_at_start s)) (sourceLength s))
            (lessEqNatRefl Zero)
            (lessEqNatZeroLeft (sourceLength s))

        fn zero_width_span_at_offset (offset : Nat) : Span = MkSpan offset offset

        fn valid_zero_width_span_at_offset (s : Source) (offset : Nat)
          : LessEqNat offset (sourceLength s) -> ValidSpan s (zero_width_span_at_offset offset) =
          \h. validZeroWidthSpan s offset h

        fn source_utf8_projects (s : Source) : IsUtf8 (sourceBytes s) =
          sourceUtf8 s

        fn source_length_valid_projects (s : Source)
          : SourceLength (sourceLengthUnit s) (sourceBytes s) (sourceLength s) =
          sourceLengthValid s

        fn located_true_value (s : Source) : Located Bool =
          MkLocated Bool (sourceId s) (zero_width_span_at_start s) True

        fn valid_located_true_value (s : Source) : ValidLocated Bool s (located_true_value s) =
          andIntro
            (Equal SourceId (locatedSource Bool (located_true_value s)) (sourceId s))
            (ValidSpan s (locatedSpan Bool (located_true_value s)))
            Refl
            (valid_zero_width_span_at_start s)
        "#,
    )
    .expect("valid half-open and zero-width spans should check");
}

#[test]
fn cat5_d1_concrete_nonempty_source_constructs_and_projects() {
    let mut env = mk_env();
    env.elaborate_file(
        r#"
        data ConcreteByteSource = MkConcreteByteSource

        const source_unit_byte : Bytes = bytes_encode "x"
        const sample_abc_bytes : Bytes = bytes_encode "abc"
        const source_unit_valid : UnitByteLength source_unit_byte = Axiom
        const sample_utf8_valid : IsUtf8 sample_abc_bytes = Axiom
        const sample_length3_valid
          : SourceLength source_unit_byte sample_abc_bytes (Suc (Suc (Suc Zero))) =
          Axiom

        instance Source ConcreteByteSource {
          sourceIdField = MkSourceId Zero ;
          sourceBytesField = sample_abc_bytes ;
          sourceLengthField = Suc (Suc (Suc Zero)) ;
          sourceLengthUnitField = source_unit_byte ;
          sourceLengthUnitValidField = source_unit_valid ;
          sourceUtf8Field = sample_utf8_valid ;
          sourceLengthValidField = sample_length3_valid
        }

        const sample_source : Source = Source_instance_ConcreteByteSource

        const projected_bytes : Bytes = sourceBytes sample_source
        const projected_length : Nat = sourceLength sample_source
        const projected_utf8 : IsUtf8 (sourceBytes sample_source) =
          sourceUtf8 sample_source
        const projected_length_valid
          : SourceLength (sourceLengthUnit sample_source) (sourceBytes sample_source) (sourceLength sample_source) =
          sourceLengthValid sample_source

        const full_source_span : Span = MkSpan Zero (sourceLength sample_source)
        const full_source_span_valid : ValidSpan sample_source full_source_span =
          andIntro
            (LessEqNat (spanStart full_source_span) (spanEnd full_source_span))
            (LessEqNat (spanEnd full_source_span) (sourceLength sample_source))
            (lessEqNatZeroLeft (sourceLength sample_source))
            (lessEqNatRefl (sourceLength sample_source))
        "#,
    )
    .expect("a concrete non-empty Source must construct and project with real length evidence");

    let mut store = make_store(&env);
    let projected_bytes = eval_def(&env, &mut store, "projected_bytes");
    assert_eq!(
        projected_bytes,
        EvalVal::Bytes(b"abc".to_vec()),
        "sourceBytes must execute through a concrete class-backed Source instance"
    );

    let projected_length = eval_def(&env, &mut store, "projected_length");
    assert_eq!(
        nat_count(&env, &projected_length),
        3,
        "sourceLength must execute through the same class-backed Source instance"
    );

    let projected_utf8 = eval_def(&env, &mut store, "projected_utf8");
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
            data MismatchedLengthSource = MkMismatchedLengthSource

            const source_unit_byte : Bytes = bytes_encode "x"
            const sample_abc_bytes : Bytes = bytes_encode "abc"
            const source_unit_valid : UnitByteLength source_unit_byte = Axiom
            const sample_utf8_valid : IsUtf8 sample_abc_bytes = Axiom
            const sample_length3_valid
              : SourceLength source_unit_byte sample_abc_bytes (Suc (Suc (Suc Zero))) =
              Axiom

            instance Source MismatchedLengthSource {
              sourceIdField = MkSourceId Zero ;
              sourceBytesField = sample_abc_bytes ;
              sourceLengthField = Suc (Suc Zero) ;
              sourceLengthUnitField = source_unit_byte ;
              sourceLengthUnitValidField = source_unit_valid ;
              sourceUtf8Field = sample_utf8_valid ;
              sourceLengthValidField = sample_length3_valid
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
            data MultiByteUnitSource = MkMultiByteUnitSource

            const sample_abc_bytes : Bytes = bytes_encode "abc"
            const invalid_unit_valid : UnitByteLength sample_abc_bytes = Refl
            const sample_utf8_valid : IsUtf8 sample_abc_bytes = Axiom
            const invalid_length_valid
              : SourceLength sample_abc_bytes sample_abc_bytes (Suc Zero) =
              Refl

            instance Source MultiByteUnitSource {
              sourceIdField = MkSourceId Zero ;
              sourceBytesField = sample_abc_bytes ;
              sourceLengthField = Suc Zero ;
              sourceLengthUnitField = sample_abc_bytes ;
              sourceLengthUnitValidField = invalid_unit_valid ;
              sourceUtf8Field = sample_utf8_valid ;
              sourceLengthValidField = invalid_length_valid
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
            const invalid_span : Span = MkSpan Zero (Suc (Suc (Suc Zero)))
            fn invalid_span_valid (s : Source) : ValidSpan s invalid_span =
              andIntro
                (LessEqNat (spanStart invalid_span) (spanEnd invalid_span))
                (LessEqNat (spanEnd invalid_span) (sourceLength s))
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
            const invalid_span : Span = MkSpan (Suc (Suc Zero)) (Suc Zero)
            fn invalid_span_valid (s : Source) : ValidSpan s invalid_span =
              andIntro
                (LessEqNat (spanStart invalid_span) (spanEnd invalid_span))
                (LessEqNat (spanEnd invalid_span) (sourceLength s))
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
            const sample_source_id : SourceId = MkSourceId Zero
            const invalid_source : Source =
              MkSource sample_source_id (bytes_encode "abc") (Suc (Suc (Suc Zero)))
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
            const sample_bytes : Bytes = bytes_encode "abc"
            const fake_utf8 : IsUtf8 sample_bytes = Refl
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
            const sample_bytes : Bytes = bytes_encode "abc"
            const sample_unit : Bytes = bytes_encode "x"
            const fake_length : SourceLength sample_unit sample_bytes Zero = Refl
            "#,
        )
        .expect_err(
            "SourceLength must not be provable by reflexive equality for arbitrary bytes/length",
        );
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
        const success_parser : Parser Bool =
          parserPure Bool True

        const success_parser_valid : ParserValid Bool success_parser =
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

        const success_parser_laws : ParserLaws Bool success_parser =
          andIntro
            (ParserValid Bool success_parser)
            (And (ParserTotal Bool success_parser) (ParserSourceLocal Bool success_parser))
            success_parser_valid
            (andIntro
              (ParserTotal Bool success_parser)
              (ParserSourceLocal Bool success_parser)
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
        const failed_parser : Parser Bool =
          parserFail Bool

        const failed_parser_valid : ParserValid Bool failed_parser =
          \s. \start. \h.
            andIntro
              (Equal SourceId (errorSource (MkParseError (sourceId s) (MkSpan start start))) (sourceId s))
              (ValidSpan s (errorSpan (MkParseError (sourceId s) (MkSpan start start))))
              Refl
              (validZeroWidthSpan s start h)

        const failed_parser_source_local : ParserSourceLocal Bool failed_parser =
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
            const wrong_source_failed_parser : Parser Bool =
              \s. \start. \h.
                Failed Bool (MkParseError (MkSourceId (Suc Zero)) (MkSpan start start))

            const wrong_source_failed_parser_valid : ParserValid Bool wrong_source_failed_parser =
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
            const invalid_span_failed_parser : Parser Bool =
              \s. \start. \h.
                Failed Bool (MkParseError (sourceId s) (MkSpan (Suc (Suc Zero)) (Suc Zero)))

            const invalid_span_failed_parser_valid : ParserValid Bool invalid_span_failed_parser =
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
            const zero_width_parser : Parser Bool =
              parserPure Bool True

            const unguarded_repeat : Parser (List Bool) =
              repeat Bool zero_width_parser
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
            const one_byte_parser : Parser Bool =
              \s. \start. \h.
                Parsed Bool True (MkSpan start (Suc start)) (Suc start)

            const repeat_two : Parser (List Bool) =
              repeatWithFuel Bool (Suc (Suc Zero)) one_byte_parser
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

#[test]
fn cat5_d3_bool_parser_printer_formatter_roundtrip_on_source_bytes() {
    let mut env = mk_env();
    let printed_len = nat_expr(22);
    let bad_len = nat_expr(14);
    env.elaborate_file(&format!(
        r#"
        data PrintedBoolExprSource = MkPrintedBoolExprSource
        data FormattedBoolExprSource = MkFormattedBoolExprSource
        data InfixBoolExprSource = MkInfixBoolExprSource

        const bool_expr_source_unit : Bytes = bytes_encode "x"
        const bool_expr_unit_valid : UnitByteLength bool_expr_source_unit = Axiom

        const representative_bool_expr : BoolExpr =
          BAnd BTrue (BNot BFalse)

        const printed_bool_expr_bytes : Bytes =
          printBoolExpr representative_bool_expr

        const printed_bool_expr_utf8 : IsUtf8 printed_bool_expr_bytes = Axiom
        const printed_bool_expr_length_valid
          : SourceLength bool_expr_source_unit printed_bool_expr_bytes ({printed_len}) =
          Axiom

        instance Source PrintedBoolExprSource {{
          sourceIdField = MkSourceId (Suc (Suc Zero)) ;
          sourceBytesField = printed_bool_expr_bytes ;
          sourceLengthField = ({printed_len}) ;
          sourceLengthUnitField = bool_expr_source_unit ;
          sourceLengthUnitValidField = bool_expr_unit_valid ;
          sourceUtf8Field = printed_bool_expr_utf8 ;
          sourceLengthValidField = printed_bool_expr_length_valid
        }}

        const printed_bool_expr_source : Source = Source_instance_PrintedBoolExprSource

        const parse_printed_bool_expr : ParseResult (Syntax BoolExpr) =
          parseBoolExpr printed_bool_expr_source Zero (lessEqNatZeroLeft (sourceLength printed_bool_expr_source))

        const parse_printed_bool_expr_erases : Bool =
          match parse_printed_bool_expr {{
            Parsed syntax consumed next => boolExprEq (eraseSpans syntax) representative_bool_expr ;
            Failed err => False
          }}

        const parsed_bool_expr_syntax : Syntax BoolExpr =
          match parse_printed_bool_expr {{
            Parsed syntax consumed next => syntax ;
            Failed err => syntaxLeaf printed_bool_expr_source Zero Zero BFalse
          }}

        const format_printed_bool_expr : Result ParseError Bytes =
          formatBoolExpr printed_bool_expr_source

        const formatted_bool_expr_bytes : Bytes =
          match format_printed_bool_expr {{
            Ok bs => bs ;
            Err err => bytes_encode "ERR"
          }}

        const formatted_bool_expr_utf8 : IsUtf8 formatted_bool_expr_bytes = Axiom
        const formatted_bool_expr_length_valid
          : SourceLength bool_expr_source_unit formatted_bool_expr_bytes ({printed_len}) =
          Axiom

        instance Source FormattedBoolExprSource {{
          sourceIdField = MkSourceId (Suc (Suc (Suc Zero))) ;
          sourceBytesField = formatted_bool_expr_bytes ;
          sourceLengthField = ({printed_len}) ;
          sourceLengthUnitField = bool_expr_source_unit ;
          sourceLengthUnitValidField = bool_expr_unit_valid ;
          sourceUtf8Field = formatted_bool_expr_utf8 ;
          sourceLengthValidField = formatted_bool_expr_length_valid
        }}

        const formatted_bool_expr_source : Source = Source_instance_FormattedBoolExprSource

        const reformat_bool_expr : Result ParseError Bytes =
          formatBoolExpr formatted_bool_expr_source

        const reformatted_bool_expr_bytes : Bytes =
          match reformat_bool_expr {{
            Ok bs => bs ;
            Err err => bytes_encode "ERR"
          }}

        const infix_bool_expr_bytes : Bytes = bytes_encode "true and false"
        const infix_bool_expr_utf8 : IsUtf8 infix_bool_expr_bytes = Axiom
        const infix_bool_expr_length_valid
          : SourceLength bool_expr_source_unit infix_bool_expr_bytes ({bad_len}) =
          Axiom

        instance Source InfixBoolExprSource {{
          sourceIdField = MkSourceId (Suc (Suc (Suc (Suc Zero)))) ;
          sourceBytesField = infix_bool_expr_bytes ;
          sourceLengthField = ({bad_len}) ;
          sourceLengthUnitField = bool_expr_source_unit ;
          sourceLengthUnitValidField = bool_expr_unit_valid ;
          sourceUtf8Field = infix_bool_expr_utf8 ;
          sourceLengthValidField = infix_bool_expr_length_valid
        }}

        const infix_bool_expr_source : Source = Source_instance_InfixBoolExprSource

        const parse_infix_bool_expr : ParseResult (Syntax BoolExpr) =
          parseBoolExpr infix_bool_expr_source Zero (lessEqNatZeroLeft (sourceLength infix_bool_expr_source))
        "#,
    ))
    .expect("D3 Boolean parser/printer/formatter producer path must elaborate");

    let mut store = make_store(&env);
    neutralize_fixture_proofs(
        &env,
        &mut store,
        &[
            "record_nil_val",
            "bool_expr_unit_valid",
            "printed_bool_expr_utf8",
            "printed_bool_expr_length_valid",
            "formatted_bool_expr_utf8",
            "formatted_bool_expr_length_valid",
            "infix_bool_expr_utf8",
            "infix_bool_expr_length_valid",
        ],
    );
    let printed = eval_def(&env, &mut store, "printed_bool_expr_bytes");
    assert_eq!(
        printed,
        EvalVal::Bytes(b"(and true (not false))".to_vec()),
        "printBoolExpr must emit canonical ASCII bytes"
    );

    let parsed = eval_def(&env, &mut store, "parse_printed_bool_expr");
    let parsed_args = ctor_args(&env, &parsed, "Parsed");
    assert!(
        parsed_args.len() >= 4,
        "Parsed must carry type/value/span/next args, got {parsed_args:?}"
    );
    let syntax = parsed_args[1].clone();
    let expected_expr = eval_def(&env, &mut store, "representative_bool_expr");
    let (root, _) = syntax_root_and_children(&env, &syntax);
    let root_args = ctor_args(&env, root, "MkLocated");
    assert_eq!(
        root_args[3], expected_expr,
        "parseBoolExpr (printBoolExpr e) must erase back to e"
    );
    assert!(
        matches!(
            eval_def(&env, &mut store, "parse_printed_bool_expr_erases"),
            EvalVal::Ctor { id, .. } if id == env.globals["True"]
        ),
        "the checked surface erasure witness must evaluate to True"
    );

    let formatted = eval_def(&env, &mut store, "format_printed_bool_expr");
    let formatted_args = ctor_args(&env, &formatted, "Ok");
    assert!(
        formatted_args.len() >= 3,
        "Ok must carry error type/value type/payload args, got {formatted_args:?}"
    );
    assert_eq!(
        formatted_args[2],
        EvalVal::Bytes(b"(and true (not false))".to_vec()),
        "formatBoolExpr must preserve the erased tree by printing canonical bytes"
    );

    let idempotent = eval_def(&env, &mut store, "reformat_bool_expr");
    let idempotent_args = ctor_args(&env, &idempotent, "Ok");
    assert_eq!(
        idempotent_args[2],
        EvalVal::Bytes(b"(and true (not false))".to_vec()),
        "formatBoolExpr must be idempotent on generated bytes"
    );

    let spans = syntax_spans(&env, &syntax);
    assert_eq!(
        spans,
        vec![(0, 22), (5, 9), (10, 21), (15, 20)],
        "parsed syntax must expose valid spans for every package-owned syntax node"
    );
    assert!(
        spans.iter().all(|(start, end)| start <= end && *end <= 22),
        "all parsed syntax spans must be within the concrete Source length: {spans:?}"
    );

    let bad = eval_def(&env, &mut store, "parse_infix_bool_expr");
    assert!(
        matches!(bad, EvalVal::Ctor { id, .. } if id == env.globals["Failed"]),
        "`true and false` must reject; D3 has no implicit precedence table"
    );
}
