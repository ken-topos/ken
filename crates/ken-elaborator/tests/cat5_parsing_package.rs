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

const PARSING_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Parsing/Parsing.ken.md");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_ken_md_file(PARSING_KEN_MD)
        .expect("catalog/packages/Capability/Parsing/Parsing.ken.md must elaborate");
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
    env.elaborate_ken_md_file(PARSING_KEN_MD)
        .expect("catalog/packages/Capability/Parsing/Parsing.ken.md must elaborate");
    let after_trusted: HashSet<GlobalId> = env.env.trusted_base().into_iter().collect();
    let new_trusted: HashSet<_> = after_trusted.difference(&base_trusted).copied().collect();
    assert!(
        new_trusted.is_empty(),
        "parsing.ken must add no new trusted_base entries, got {new_trusted:?}"
    );

    for name in [
        "source_id",
        "source_bytes",
        "source_utf8",
        "source_length",
        "source_length_unit",
        "source_length_unit_valid",
        "source_length_valid",
        "SourceLength",
        "UnitByteLength",
        "EmptyBytes",
        "NonEmptyBytes",
        "byte_unit_zero_int",
        "byte_unit_nat_to_int",
        "span_start",
        "span_end",
        "nat_leq_bool",
        "LessEqNat",
        "less_eq_nat_refl",
        "less_eq_nat_zero_left",
        "ValidSpan",
        "valid_zero_width_span",
        "located_source",
        "located_span",
        "located_value",
        "ValidLocated",
        "error_source",
        "error_span",
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
        "parser_pure",
        "parser_fail",
        "syntax_root",
        "syntax_children",
        "erase_spans",
        "list_append",
        "ValidLocatedList",
        "ValidSyntax",
        "bool_expr_eq",
        "nat_eq_bool",
        "nat_add",
        "nat_lt_bool",
        "bool_and",
        "source_byte_eq",
        "source_byte_eq_at",
        "starts_true_token",
        "starts_false_token",
        "starts_not_open_token",
        "starts_and_open_token",
        "skip_spaces_fuel",
        "skip_spaces",
        "syntax_leaf",
        "syntax_node_unary",
        "syntax_node_binary",
        "parse_bool_expr_at_fuel",
        "parse_bool_expr",
        "print_bool_expr",
        "format_bool_expr",
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
        PARSING_KEN_MD.contains("data ParseError = MkParseError SourceId Span")
            && PARSING_KEN_MD.contains("fn error_source (err : ParseError) : SourceId =")
            && PARSING_KEN_MD.contains("fn error_span (err : ParseError) : Span ="),
        "ParseError must carry source identity and a span with accessors"
    );
    assert!(
        PARSING_KEN_MD.contains("data ParseResult a =")
            && PARSING_KEN_MD.contains("Parsed a Span Nat")
            && PARSING_KEN_MD.contains("Failed ParseError"),
        "ParseResult must be the total Parsed/Failed result surface"
    );
    assert!(
        PARSING_KEN_MD.contains("const Parser (a : Type) : Type =")
            && PARSING_KEN_MD
                .contains("(s : Source) -> (start : Nat) -> LessEqNat start (source_length s)")
            && PARSING_KEN_MD.contains("-> ParseResult a"),
        "Parser must be total over well-formed source/start inputs"
    );
    assert!(
        PARSING_KEN_MD.contains("fn ParsedValid")
            && PARSING_KEN_MD.contains("Equal Nat (span_start consumed) start")
            && PARSING_KEN_MD.contains("Equal Nat (span_end consumed) next")
            && PARSING_KEN_MD.contains("fn FailedValid")
            && PARSING_KEN_MD.contains("Equal SourceId (error_source err) (source_id s)")
            && PARSING_KEN_MD.contains("ValidSpan s (error_span err)")
            && PARSING_KEN_MD.contains("fn ParserLaws"),
        "D2 laws must state success validity, failure validity, totality, and source locality"
    );
    assert!(
        !PARSING_KEN_MD.contains("fn repeatWithFuel")
            && !PARSING_KEN_MD.contains("fn parseResultNext")
            && !PARSING_KEN_MD.contains("fn repeat (")
            && !PARSING_KEN_MD.contains("fn many ("),
        "D2 repetition is deferred; the package must not export broken repeat/many helpers"
    );
    assert!(
        !PARSING_KEN_MD.contains("= Axiom"),
        "CAT-5 D2 package must not use Axiom"
    );
}

#[test]
fn cat5_d3_bool_expression_surface_is_package_owned() {
    assert!(
        PARSING_KEN_MD.contains("data BoolExpr =")
            && PARSING_KEN_MD.contains("BTrue")
            && PARSING_KEN_MD.contains("BFalse")
            && PARSING_KEN_MD.contains("BNot BoolExpr")
            && PARSING_KEN_MD.contains("BAnd BoolExpr BoolExpr"),
        "D3 must expose the package-owned BoolExpr data surface"
    );
    assert!(
        PARSING_KEN_MD.contains("data Syntax a = MkSyntax (Located a) (List (Located a))")
            && PARSING_KEN_MD.contains("fn erase_spans (x : Syntax BoolExpr) : BoolExpr =")
            && PARSING_KEN_MD.contains("fn ValidSyntax"),
        "D3 Syntax must be package-owned located syntax, not compiler AST"
    );
    assert!(
        PARSING_KEN_MD.contains("const parse_bool_expr : Parser (Syntax BoolExpr) =")
            && PARSING_KEN_MD.contains("fn print_bool_expr (e : BoolExpr) : Bytes =")
            && PARSING_KEN_MD.contains("fn format_bool_expr (s : Source) : Result ParseError Bytes ="),
        "D3 must export parser, printer, and formatter with the pinned types"
    );
    assert!(
        PARSING_KEN_MD.contains("bytes_at (source_bytes s)")
            && PARSING_KEN_MD.contains("bytes_encode \"true\"")
            && PARSING_KEN_MD.contains("bytes_encode \"false\"")
            && PARSING_KEN_MD.contains("bytes_encode \"(not \"")
            && PARSING_KEN_MD.contains("bytes_encode \"(and \""),
        "D3 must operate over Source bytes and canonical ASCII token bytes"
    );
    assert!(
        !PARSING_KEN_MD.contains("compiler")
            && !PARSING_KEN_MD.contains("AST")
            && !PARSING_KEN_MD.contains("= Axiom"),
        "D3 package surface must not route through compiler ASTs or package axioms"
    );
}

#[test]
fn cat5_d1_source_span_surface_is_byte_artifact_and_source_explicit() {
    assert!(
        PARSING_KEN_MD.contains("fn IsUtf8 (bs : Bytes) : Prop =")
            && PARSING_KEN_MD.contains("bytes_encode (bytes_decode bs)")
            && !PARSING_KEN_MD.contains("Equal Bytes bs bs"),
        "IsUtf8 must be round-trip evidence over the source bytes, not reflexive equality"
    );
    assert!(
        PARSING_KEN_MD.contains("fn byte_unit_nat_to_int (unit : Bytes) (n : Nat) : Int =")
            && PARSING_KEN_MD.contains("fn EmptyBytes (bs : Bytes) : Prop =")
            && PARSING_KEN_MD.contains("fn NonEmptyBytes (bs : Bytes) : Prop =")
            && PARSING_KEN_MD.contains("fn UnitByteLength (unit : Bytes) : Prop =")
            && PARSING_KEN_MD.contains("fn SourceLength (unit : Bytes) (bs : Bytes) (n : Nat) : Prop =")
            && PARSING_KEN_MD.contains("bytes_concat left right")
            && PARSING_KEN_MD.contains("NonEmptyBytes unit")
            && PARSING_KEN_MD.contains("bytes_length bs")
            && PARSING_KEN_MD.contains("byte_unit_nat_to_int unit n"),
        "SourceLength must tie source_length to source_bytes through a non-empty byte-atomic unit witness"
    );
    assert!(
        PARSING_KEN_MD.contains("class Source {")
            && PARSING_KEN_MD.contains("source_bytes_field : Bytes")
            && PARSING_KEN_MD.contains("source_length_field : Nat")
            && PARSING_KEN_MD.contains("source_length_unit_field : Bytes")
            && PARSING_KEN_MD
                .contains("source_length_unit_valid_field : UnitByteLength source_length_unit_field")
            && PARSING_KEN_MD.contains("source_utf8_field : IsUtf8 source_bytes_field")
            && PARSING_KEN_MD
                .contains("source_length_valid_field : SourceLength source_length_unit_field source_bytes_field source_length_field"),
        "Source must be a dependent record carrying bytes, length, and both proof fields"
    );
    assert!(
        !PARSING_KEN_MD.contains("data Source = MkSource"),
        "Source must not expose the old unconstrained MkSource constructor"
    );
    assert!(
        !PARSING_KEN_MD.contains("source_bytes_field : String") && !PARSING_KEN_MD.contains("String Nat"),
        "Source must not use normalized String as the offset basis"
    );
    assert!(
        PARSING_KEN_MD.contains("data Span = MkSpan Nat Nat"),
        "Span must carry only byte endpoints"
    );
    assert!(
        PARSING_KEN_MD.contains("data Located a = MkLocated SourceId Span a")
            && PARSING_KEN_MD.contains("fn ValidLocated"),
        "source identity must be supplied by Located/validity, not by bare Span"
    );
    assert!(
        !PARSING_KEN_MD.contains("= Axiom"),
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
          and_intro
            (LessEqNat (span_start (zero_width_span_at_start s)) (span_end (zero_width_span_at_start s)))
            (LessEqNat (span_end (zero_width_span_at_start s)) (source_length s))
            (less_eq_nat_refl Zero)
            (less_eq_nat_zero_left (source_length s))

        fn zero_width_span_at_offset (offset : Nat) : Span = MkSpan offset offset

        fn valid_zero_width_span_at_offset (s : Source) (offset : Nat)
          : LessEqNat offset (source_length s) -> ValidSpan s (zero_width_span_at_offset offset) =
          \h. valid_zero_width_span s offset h

        fn source_utf8_projects (s : Source) : IsUtf8 (source_bytes s) =
          source_utf8 s

        fn source_length_valid_projects (s : Source)
          : SourceLength (source_length_unit s) (source_bytes s) (source_length s) =
          source_length_valid s

        fn located_true_value (s : Source) : Located Bool =
          MkLocated Bool (source_id s) (zero_width_span_at_start s) True

        fn valid_located_true_value (s : Source) : ValidLocated Bool s (located_true_value s) =
          and_intro
            (Equal SourceId (located_source Bool (located_true_value s)) (source_id s))
            (ValidSpan s (located_span Bool (located_true_value s)))
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
          source_id_field = MkSourceId Zero ;
          source_bytes_field = sample_abc_bytes ;
          source_length_field = Suc (Suc (Suc Zero)) ;
          source_length_unit_field = source_unit_byte ;
          source_length_unit_valid_field = source_unit_valid ;
          source_utf8_field = sample_utf8_valid ;
          source_length_valid_field = sample_length3_valid
        }

        const sample_source : Source = Source_instance_ConcreteByteSource

        const projected_bytes : Bytes = source_bytes sample_source
        const projected_length : Nat = source_length sample_source
        const projected_utf8 : IsUtf8 (source_bytes sample_source) =
          source_utf8 sample_source
        const projected_length_valid
          : SourceLength (source_length_unit sample_source) (source_bytes sample_source) (source_length sample_source) =
          source_length_valid sample_source

        const full_source_span : Span = MkSpan Zero (source_length sample_source)
        const full_source_span_valid : ValidSpan sample_source full_source_span =
          and_intro
            (LessEqNat (span_start full_source_span) (span_end full_source_span))
            (LessEqNat (span_end full_source_span) (source_length sample_source))
            (less_eq_nat_zero_left (source_length sample_source))
            (less_eq_nat_refl (source_length sample_source))
        "#,
    )
    .expect("a concrete non-empty Source must construct and project with real length evidence");

    let mut store = make_store(&env);
    let projected_bytes = eval_def(&env, &mut store, "projected_bytes");
    assert_eq!(
        projected_bytes,
        EvalVal::Bytes(b"abc".to_vec()),
        "source_bytes must execute through a concrete class-backed Source instance"
    );

    let projected_length = eval_def(&env, &mut store, "projected_length");
    assert_eq!(
        nat_count(&env, &projected_length),
        3,
        "source_length must execute through the same class-backed Source instance"
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
              source_id_field = MkSourceId Zero ;
              source_bytes_field = sample_abc_bytes ;
              source_length_field = Suc (Suc Zero) ;
              source_length_unit_field = source_unit_byte ;
              source_length_unit_valid_field = source_unit_valid ;
              source_utf8_field = sample_utf8_valid ;
              source_length_valid_field = sample_length3_valid
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
              source_id_field = MkSourceId Zero ;
              source_bytes_field = sample_abc_bytes ;
              source_length_field = Suc Zero ;
              source_length_unit_field = sample_abc_bytes ;
              source_length_unit_valid_field = invalid_unit_valid ;
              source_utf8_field = sample_utf8_valid ;
              source_length_valid_field = invalid_length_valid
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
              and_intro
                (LessEqNat (span_start invalid_span) (span_end invalid_span))
                (LessEqNat (span_end invalid_span) (source_length s))
                tt
                tt
            "#,
        )
        .expect_err("end > source_length must not typecheck as a ValidSpan");
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
              and_intro
                (LessEqNat (span_start invalid_span) (span_end invalid_span))
                (LessEqNat (span_end invalid_span) (source_length s))
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
          parser_pure Bool True

        const success_parser_valid : ParserValid Bool success_parser =
          \s. \start. \h.
            and_intro
              (ValidSpan s (MkSpan start start))
              (And
                (Equal Nat (span_start (MkSpan start start)) start)
                (Equal Nat (span_end (MkSpan start start)) start))
              (valid_zero_width_span s start h)
              (and_intro
                (Equal Nat (span_start (MkSpan start start)) start)
                (Equal Nat (span_end (MkSpan start start)) start)
                Refl
                Refl)

        const success_parser_laws : ParserLaws Bool success_parser =
          and_intro
            (ParserValid Bool success_parser)
            (And (ParserTotal Bool success_parser) (ParserSourceLocal Bool success_parser))
            success_parser_valid
            (and_intro
              (ParserTotal Bool success_parser)
              (ParserSourceLocal Bool success_parser)
              (\s. \start. \h. tt)
              (\s. \start. \h. valid_zero_width_span s start h))
        "#,
    )
    .expect("successful parser must carry a valid consumed span with span_start = start");
}

#[test]
fn cat5_d2_failed_parser_carries_same_source_valid_span() {
    let mut env = mk_env();
    env.elaborate_file(
        r#"
        const failed_parser : Parser Bool =
          parser_fail Bool

        const failed_parser_valid : ParserValid Bool failed_parser =
          \s. \start. \h.
            and_intro
              (Equal SourceId (error_source (MkParseError (source_id s) (MkSpan start start))) (source_id s))
              (ValidSpan s (error_span (MkParseError (source_id s) (MkSpan start start))))
              Refl
              (valid_zero_width_span s start h)

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
                and_intro
                  (Equal SourceId (error_source (MkParseError (MkSourceId (Suc Zero)) (MkSpan start start))) (source_id s))
                  (ValidSpan s (error_span (MkParseError (MkSourceId (Suc Zero)) (MkSpan start start))))
                  Refl
                  (valid_zero_width_span s start h)
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
                Failed Bool (MkParseError (source_id s) (MkSpan (Suc (Suc Zero)) (Suc Zero)))

            const invalid_span_failed_parser_valid : ParserValid Bool invalid_span_failed_parser =
              \s. \start. \h.
                and_intro
                  (Equal SourceId (error_source (MkParseError (source_id s) (MkSpan (Suc (Suc Zero)) (Suc Zero)))) (source_id s))
                  (ValidSpan s (error_span (MkParseError (source_id s) (MkSpan (Suc (Suc Zero)) (Suc Zero)))))
                  Refl
                  (and_intro
                    (LessEqNat (span_start (MkSpan (Suc (Suc Zero)) (Suc Zero))) (span_end (MkSpan (Suc (Suc Zero)) (Suc Zero))))
                    (LessEqNat (span_end (MkSpan (Suc (Suc Zero)) (Suc Zero))) (source_length s))
                    tt
                    (less_eq_nat_zero_left (source_length s)))
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
              parser_pure Bool True

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
          print_bool_expr representative_bool_expr

        const printed_bool_expr_utf8 : IsUtf8 printed_bool_expr_bytes = Axiom
        const printed_bool_expr_length_valid
          : SourceLength bool_expr_source_unit printed_bool_expr_bytes ({printed_len}) =
          Axiom

        instance Source PrintedBoolExprSource {{
          source_id_field = MkSourceId (Suc (Suc Zero)) ;
          source_bytes_field = printed_bool_expr_bytes ;
          source_length_field = ({printed_len}) ;
          source_length_unit_field = bool_expr_source_unit ;
          source_length_unit_valid_field = bool_expr_unit_valid ;
          source_utf8_field = printed_bool_expr_utf8 ;
          source_length_valid_field = printed_bool_expr_length_valid
        }}

        const printed_bool_expr_source : Source = Source_instance_PrintedBoolExprSource

        const parse_printed_bool_expr : ParseResult (Syntax BoolExpr) =
          parse_bool_expr printed_bool_expr_source Zero (less_eq_nat_zero_left (source_length printed_bool_expr_source))

        const parse_printed_bool_expr_erases : Bool =
          match parse_printed_bool_expr {{
            Parsed syntax consumed next => bool_expr_eq (erase_spans syntax) representative_bool_expr ;
            Failed err => False
          }}

        const parsed_bool_expr_syntax : Syntax BoolExpr =
          match parse_printed_bool_expr {{
            Parsed syntax consumed next => syntax ;
            Failed err => syntax_leaf printed_bool_expr_source Zero Zero BFalse
          }}

        const format_printed_bool_expr : Result ParseError Bytes =
          format_bool_expr printed_bool_expr_source

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
          source_id_field = MkSourceId (Suc (Suc (Suc Zero))) ;
          source_bytes_field = formatted_bool_expr_bytes ;
          source_length_field = ({printed_len}) ;
          source_length_unit_field = bool_expr_source_unit ;
          source_length_unit_valid_field = bool_expr_unit_valid ;
          source_utf8_field = formatted_bool_expr_utf8 ;
          source_length_valid_field = formatted_bool_expr_length_valid
        }}

        const formatted_bool_expr_source : Source = Source_instance_FormattedBoolExprSource

        const reformat_bool_expr : Result ParseError Bytes =
          format_bool_expr formatted_bool_expr_source

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
          source_id_field = MkSourceId (Suc (Suc (Suc (Suc Zero)))) ;
          source_bytes_field = infix_bool_expr_bytes ;
          source_length_field = ({bad_len}) ;
          source_length_unit_field = bool_expr_source_unit ;
          source_length_unit_valid_field = bool_expr_unit_valid ;
          source_utf8_field = infix_bool_expr_utf8 ;
          source_length_valid_field = infix_bool_expr_length_valid
        }}

        const infix_bool_expr_source : Source = Source_instance_InfixBoolExprSource

        const parse_infix_bool_expr : ParseResult (Syntax BoolExpr) =
          parse_bool_expr infix_bool_expr_source Zero (less_eq_nat_zero_left (source_length infix_bool_expr_source))
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
        "print_bool_expr must emit canonical ASCII bytes"
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
        "parse_bool_expr (print_bool_expr e) must erase back to e"
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
        "format_bool_expr must preserve the erased tree by printing canonical bytes"
    );

    let idempotent = eval_def(&env, &mut store, "reformat_bool_expr");
    let idempotent_args = ctor_args(&env, &idempotent, "Ok");
    assert_eq!(
        idempotent_args[2],
        EvalVal::Bytes(b"(and true (not false))".to_vec()),
        "format_bool_expr must be idempotent on generated bytes"
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
