//! CAT-5 D1/D2 acceptance for the parsing package source/span and parser core.
//!
//! This loads the real package file and checks the bounded D1/D2 surface:
//! byte-artifact `Source`, half-open byte `Span`, explicit validity proofs,
//! located values, total parse results, and zero trusted-base delta.

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::Decl;
use ken_kernel::GlobalId;
use std::collections::HashSet;

const PARSING_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/Parsing/Parsing.ken.md");
const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const DIAGNOSTIC_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Diagnostics/Core.ken.md");
const CURSOR_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Parsing/Cursor.ken.md");
const DECODER_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Parsing/Decoder.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Transport must elaborate first");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Collections must elaborate second");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("LawfulClasses must elaborate third");
    env.elaborate_ken_md_file(DIAGNOSTIC_KEN_MD)
        .expect("Capability.Diagnostics.Core must elaborate fourth");
    env.elaborate_ken_md_file(CURSOR_KEN_MD)
        .expect("Capability.Parsing.Cursor must elaborate fifth");
    env.elaborate_ken_md_file(DECODER_KEN_MD)
        .expect("Capability.Parsing.Decoder must elaborate sixth");
    env
}

fn mk_env() -> ElabEnv {
    let mut env = dependency_env();
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
    store.list_char_ids = Some(ListCharIds {
        nil_id: env.prelude_env.nil_id,
        cons_id: env.prelude_env.cons_id,
    });
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

/// The Pi-telescope depth of a constructor type (its declared field arity).
/// Constructor types are literal Pi chains, so a plain recursive count is
/// safe -- no whnf/substitution is needed.
fn pi_arity(term: &ken_kernel::Term) -> usize {
    match term {
        ken_kernel::Term::Pi(_, codomain) => 1 + pi_arity(codomain),
        _ => 0,
    }
}

/// The head `GlobalId` a type term applies -- the inductive former or constant
/// at the spine of `Nat`, `List Nat`, etc. Used to check a constructor field's
/// declared TYPE (not just its arity), so a claim like "wraps exactly one Nat"
/// is carried by the term rather than by an assertion message.
fn head_global(term: &ken_kernel::Term) -> Option<ken_kernel::GlobalId> {
    match term {
        ken_kernel::Term::IndFormer { id, .. } | ken_kernel::Term::Const { id, .. } => Some(*id),
        ken_kernel::Term::App(function, _) => head_global(function),
        _ => None,
    }
}

#[test]
fn cat5_d1_source_span_package_elaborates_zero_delta() {
    let mut env = dependency_env();
    let base_trusted: HashSet<GlobalId> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(PARSING_KEN_MD)
        .expect("catalog/packages/Capability/Parsing/Parsing.ken.md must elaborate");
    let after_trusted: HashSet<GlobalId> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        base_trusted, after_trusted,
        "parsing.ken must preserve the trusted-base set exactly"
    );

    for name in [
        "source_id",
        "source_bytes",
        "source_bytes::utf8",
        "source_length",
        "span_start",
        "span_end",
        "byte_cursor_source",
        "byte_cursor_position",
        "byte_cursor_remaining",
        "byte_cursor_peek",
        "byte_cursor_advance",
        "byte_cursor_locate",
        "byte_cursor_ops",
        "nat_leq_bool",
        "LessEqNat",
        "LessEqNat::refl",
        "LessEqNat::zero_left",
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
        "decoder_parse_error",
        "parser_from_decoder",
        "parser_pure",
        "parser_fail",
        "syntax_root",
        "syntax_children",
        "erase_spans",
        "list_append",
        "ValidLocatedList",
        "ValidSyntax",
        "bool_expr_eq",
        "syntax_leaf",
        "syntax_node_unary",
        "syntax_node_binary",
        "byte_code_decoder",
        "true_token_decoder",
        "false_token_decoder",
        "not_open_token_decoder",
        "and_open_token_decoder",
        "spaces_decoder",
        "bool_true_decoder",
        "bool_false_decoder",
        "bool_not_decoder",
        "bool_and_decoder",
        "bool_decoder_layer",
        "bool_expression_decoder",
        "complete_bool_decoder",
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
    // CLAIM LEDGER (Q-CLAIM-CLOSURE AC-3): ParseResult exposes Parsed/Failed
    // with the pinned arg counts (evaluated); Parser is total over a well-formed
    // (Source, in-bounds start) pair; ParsedValid/FailedValid/ParserLaws are
    // checked (Transparent) declarations; the bespoke fuel recursion
    // (parse_bool_expr_at_fuel/skip_spaces_fuel) is retired; parser_from_decoder
    // specializes the shared Decoder; decoder_recursive/decoder_many type-arg
    // pins consciously dropped in favor of the D3 roundtrip's behavior, not
    // subsumed by it [R4 -- see foot of test].
    // Add an assert -> add its claim here.
    //
    // Rework (Q-RESIDUE, 2026-07-21): the surface contract is "the declared
    // shape typechecks and evaluates as claimed", not "the source text is
    // spelled this way" -- checked against the elaborated env, mirroring
    // `constrained_instance_elaboration.rs`'s structural style. The
    // `!contains("= Axiom")` check is dropped: `cat5_d1_source_span_package_
    // elaborates_zero_delta` above already proves zero new trusted-base delta
    // across the whole package, a strictly stronger semantic proof that no
    // Axiom was introduced.
    let mut env = mk_env();
    let mut store = make_store(&env);

    // ParseError carries a SourceId and Span, both recoverable by accessor.
    env.elaborate_file(
        r#"
        const parse_error_probe : ParseError = MkParseError (MkSourceId (Suc Zero)) (MkSpan Zero (Suc Zero))
        const parse_error_probe_source : SourceId = error_source parse_error_probe
        const parse_error_probe_span : Span = error_span parse_error_probe
        "#,
    )
    .expect("ParseError must carry source identity and a span with accessors");
    let probe_source = eval_def(&env, &mut store, "parse_error_probe_source");
    assert_eq!(
        ctor_args(&env, &probe_source, "MkSourceId").len(),
        1,
        "SourceId must be recoverable from a ParseError by error_source"
    );
    let probe_span = eval_def(&env, &mut store, "parse_error_probe_span");
    assert_eq!(
        span_bounds(&env, &probe_span),
        (0, 1),
        "Span must be recoverable from a ParseError by error_span"
    );

    // ParseResult is the total Parsed/Failed surface: constructed and
    // evaluated, not read off the data declaration's spelling.
    env.elaborate_file(
        r#"
        const parsed_probe : ParseResult Bool = Parsed Bool True (MkSpan Zero (Suc Zero)) (Suc Zero)
        const failed_probe : ParseResult Bool = Failed Bool parse_error_probe
        "#,
    )
    .expect("ParseResult must expose both the Parsed and Failed outcomes");
    let parsed_probe = eval_def(&env, &mut store, "parsed_probe");
    assert_eq!(
        ctor_args(&env, &parsed_probe, "Parsed").len(),
        4,
        "Parsed must carry type/value/span/next args"
    );
    let failed_probe = eval_def(&env, &mut store, "failed_probe");
    assert_eq!(
        ctor_args(&env, &failed_probe, "Failed").len(),
        2,
        "Failed must carry type/ParseError args"
    );

    // Parser is total over a well-formed (source, start) pair: this only
    // typechecks if Parser really unfolds to the pinned three-argument Pi
    // chain (Source, Nat, LessEqNat-proof) -> ParseResult a. ParsedValid/
    // FailedValid/ParserLaws content is exercised as real proof obligations
    // by the sibling law tests below (`cat5_d2_success_parser_carries_valid_
    // consumed_span_from_start` et al.), not duplicated here.
    env.elaborate_file(
        r#"
        fn total_parser_shape_probe (s : Source) (start : Nat) (h : LessEqNat start (source_length s)) : ParseResult Bool =
          (parser_pure Bool True) s start h
        "#,
    )
    .expect("Parser must be total over (Source, in-bounds start)");
    for law_prop in ["ParsedValid", "FailedValid", "ParserLaws"] {
        let id = env.globals[law_prop];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{law_prop} must be a checked declaration"
        );
    }

    // D2 must specialize the shared Decoder and retire CAT-5's bespoke fuel
    // recursion -- real name-resolution facts against the elaborated env.
    for retired in ["parse_bool_expr_at_fuel", "skip_spaces_fuel"] {
        assert!(
            !env.globals.contains_key(retired),
            "{retired} must not survive as a package export"
        );
    }
    assert!(
        env.globals.contains_key("parser_from_decoder"),
        "D2 must specialize the shared Decoder via parser_from_decoder"
    );

    // R4 (Q-CLAIM-CLOSURE): Q-RESIDUE dropped D2's type-argument pins on
    // `decoder_recursive` and `decoder_many` (the shared Decoder's recursion
    // and repetition combinators, defined in `Decoder.ken.md`). Their behavior
    // is exercised by `cat5_d3_bool_parser_printer_formatter_roundtrip_on_
    // source_bytes`: the roundtrip parses nested `BAnd`/`BNot` expressions
    // (recursive decoding) and multi-token sequences (repetition/`many`), and
    // asserts exact output bytes.
    //
    // These pins are recorded as CONSCIOUSLY DROPPED in favor of that
    // behavioral contract -- NOT subsumed by it. The roundtrip is the better
    // contract for what the combinators DO, but it is not a strict superset of
    // the dropped pins: the exact call/signature spelling of `decoder_recursive`
    // /`decoder_many` can drift while the roundtrip stays green (a refactor that
    // reroutes through a differently-typed combinator, say). The trade is
    // deliberate -- behavior over spelling -- and if that roundtrip test is
    // removed, this cross-reference is the signal to restore the pins.
}

#[test]
fn cat5_d3_bool_expression_surface_is_package_owned() {
    // CLAIM LEDGER (Q-CLAIM-CLOSURE AC-3): BoolExpr's four constructors compose
    // (BAnd/BTrue/BNot/BFalse, evaluated); Syntax is constructible from a
    // Located root + a List of Located children; erase_spans and ValidSyntax
    // have their pinned signatures [R4 -- type-pinned, was bare contains_key];
    // parser/print/format exist with exactly the pinned types (roundtrip
    // BEHAVIOR is the sibling roundtrip test). Add an assert -> add its claim.
    //
    // Rework (Q-RESIDUE, 2026-07-21): elaborate-then-assert-structurally, per
    // language-leader's guidance. The parser/printer/formatter roundtrip
    // BEHAVIOR (including the canonical ASCII token bytes) is proven by the
    // sibling test `cat5_d3_bool_parser_printer_formatter_roundtrip_on_
    // source_bytes` below -- exact-byte assertions there would fail if a
    // non-canonical token encoding were used, which is a strictly stronger
    // net than grepping the decoder's Rust-side token literals. This test
    // pins only the package-owned data/type shape. The `!contains("= Axiom")`
    // check is dropped for the same reason as in D2.
    let mut env = mk_env();
    let mut store = make_store(&env);

    // BoolExpr is the package-owned four-constructor surface.
    env.elaborate_file("const bool_expr_probe : BoolExpr = BAnd BTrue (BNot BFalse)")
        .expect("BoolExpr's four constructors must compose as declared");
    let probe = eval_def(&env, &mut store, "bool_expr_probe");
    let and_args = ctor_args(&env, &probe, "BAnd");
    assert_eq!(and_args.len(), 2, "BAnd must carry two BoolExpr args");
    assert!(
        matches!(&and_args[0], EvalVal::Ctor { id, args, .. } if *id == env.globals["BTrue"] && args.is_empty())
    );
    let not_args = ctor_args(&env, &and_args[1], "BNot");
    assert_eq!(not_args.len(), 1, "BNot must carry one BoolExpr arg");
    assert!(
        matches!(&not_args[0], EvalVal::Ctor { id, args, .. } if *id == env.globals["BFalse"] && args.is_empty())
    );

    // Syntax a is package-owned located syntax (a Located root + a List of
    // Located children), not a compiler AST.
    env.elaborate_file(
        r#"
        const syntax_probe : Syntax BoolExpr =
          MkSyntax
            BoolExpr
            (MkLocated BoolExpr (MkSourceId Zero) (MkSpan Zero (Suc Zero)) BTrue)
            (Nil (Located BoolExpr))
        "#,
    )
    .expect("Syntax must be constructible from a Located root and a List of Located children");
    let syntax_probe = eval_def(&env, &mut store, "syntax_probe");
    let (root, children) = syntax_root_and_children(&env, &syntax_probe);
    let root_args = ctor_args(&env, root, "MkLocated");
    assert_eq!(
        root_args.len(),
        4,
        "MkLocated must carry type/source/span/value args"
    );
    assert!(
        matches!(&root_args[3], EvalVal::Ctor { id, args, .. } if *id == env.globals["BTrue"] && args.is_empty())
    );
    assert!(
        matches!(children, EvalVal::Ctor { id, args, .. } if *id == env.globals["Nil"] && args.len() == 1)
    );
    // R4 (Q-CLAIM-CLOSURE): these two were narrowed from signature-pinning to a
    // bare `contains_key` presence check -- strictly weaker than the parser/
    // printer/formatter probes a few lines down, which pin full types. A
    // presence check passes even if the signature drifts. Restore type-pinning
    // so the narrowing is undone rather than merely acknowledged, consistent
    // with the neighbours.
    env.elaborate_file(
        r#"
        fn erase_spans_shape_probe (x : Syntax BoolExpr) : BoolExpr = erase_spans x
        fn valid_syntax_shape_probe (s : Source) (x : Syntax BoolExpr) : Prop =
          ValidSyntax BoolExpr s x
        "#,
    )
    .expect(
        "erase_spans must be `Syntax BoolExpr -> BoolExpr` and ValidSyntax must be \
         `(a : Type) (s : Source) (x : Syntax a) -> Prop`",
    );

    // parser/printer/formatter exist with exactly the pinned types; the
    // roundtrip behavior is proven by the sibling test below, not restated
    // here.
    env.elaborate_file(
        r#"
        const parse_bool_expr_shape_probe : Parser (Syntax BoolExpr) = parse_bool_expr
        fn print_bool_expr_shape_probe (e : BoolExpr) : Bytes = print_bool_expr e
        fn format_bool_expr_shape_probe (s : Source) : Result ParseError Bytes = format_bool_expr s
        "#,
    )
    .expect("parser/printer/formatter must carry exactly the pinned D3 types");
}

#[test]
fn cat5_d1_source_span_surface_is_byte_artifact_and_source_explicit() {
    // CLAIM LEDGER (Q-CLAIM-CLOSURE AC-3): IsUtf8 is Decl::Transparent;
    // source_bytes : Source -> Bytes (probe); Source.field_names ==
    // [id,bytes,utf8] (no cached-length field); MkSource is not a global; Span
    // constructible from two Nat; SourceId is a one-constructor inductive whose
    // single field is Nat [R3 -- field TYPE now checked, not just arity];
    // span_to_byte_range/span_origin/*_faithful/ValidLocated exported; Located
    // constructible; extraction names none of the forbidden tokens [R4 --
    // compiler/AST/String restored]; extraction never says `data SourceId =`
    // [R4 -- provenance guard restored]. Add an assert -> add its claim here.
    //
    // Rework (Q-RESIDUE, 2026-07-21): elaborate-then-assert-structurally. This
    // test pins the declared SHAPE; behavior is proven by siblings below. The
    // `!contains("= Axiom")` check is dropped for the same reason as in D2/D3.
    //
    // R2 (Q-CLAIM-CLOSURE): the earlier draft of this comment claimed the
    // sibling test proves `source_length`'s byte-view computation
    // "behaviorally". It does not, and cannot: `source_length`-by-name is
    // neutral in both the evaluator and the kernel (see that test's RESIDUAL
    // note), so a hostile redefinition of it leaves both tests green. What is
    // actually pinned, split across the two:
    //   - IsUtf8's round-trip-not-reflexive claim -> `cat5_d1_reflexive_utf8_proof_rejected`;
    //   - the instance-projected `source_bytes` byte count (== 3) ->
    //     `cat5_d1_concrete_nonempty_source_constructs_and_projects`;
    //   - `source_length`'s `Source -> Nat` SIGNATURE -> `total_parser_shape_probe`'s
    //     `LessEqNat start (source_length s)` bound in the D2 surface test.
    // `source_length` is shape-pinned only. Neither test pins its dynamic value,
    // and the concrete test documents why.
    let mut env = mk_env();

    assert!(matches!(
        env.env.lookup(env.globals["IsUtf8"]),
        Some(Decl::Transparent { .. })
    ));
    env.elaborate_file("fn source_bytes_type_probe (s : Source) : Bytes = source_bytes s")
        .expect("source_bytes must return Bytes, not a String-based view");

    // Source carries exactly id/bytes/UTF-8-evidence: its class field list is
    // read straight from the class registry, not grepped from a field-name
    // substring. A 4th field (a cached length carrier) would show up here.
    assert_eq!(
        env.class_env.classes["Source"].field_names,
        vec!["source_id_field", "source_bytes_field", "source_utf8_field"],
        "Source must carry exactly id/bytes/utf8-proof, no cached length field"
    );
    assert!(
        !env.globals.contains_key("MkSource"),
        "the old unconstrained MkSource constructor must not be an exported global"
    );

    env.elaborate_file("const span_probe : Span = MkSpan Zero Zero")
        .expect("Span must be constructible from two Nat endpoints");

    // SourceId lives in Capability.Diagnostics.Core (already elaborated by
    // `dependency_env()` before Parsing.ken.md), not redefined here: it
    // wraps exactly one Nat.
    //
    // R4 (Q-CLAIM-CLOSURE): Q-RESIDUE dropped a `!contains("data SourceId =")`
    // source-text guard against the package locally re-declaring SourceId
    // instead of importing it from Diagnostics.Core. This registry lookup
    // checks SourceId's SHAPE (one Nat field, below) but not its provenance:
    // a same-shaped local type would pass this lookup and the Nat field check.
    // Elaboration is NOT blind to it, however -- measured: inserting
    // `data SourceId = MkSourceIdLocal Nat` into Parsing.ken.md is REJECTED at
    // package elaboration (`KernelRejected(TypeMismatch)`), because
    // `span_origin`'s `SourceOrigin source ...` cross-reference binds `source`
    // to the shadowing local type, which no longer unifies with `Origin`'s
    // `SourceOrigin : SourceId -> ByteRange -> Origin` elaborated earlier
    // against the genuine import. So the in-file threat is already caught by
    // nominal typing. The substring guard RESTORED below is kept as cheap
    // defense-in-depth: it still catches a redeclaration positioned to dodge
    // that cross-reference, and states the intent where the shape check cannot.
    // (An earlier draft of this note claimed elaboration *accepts* a local
    // redeclaration; that measurement was an isolated post-hoc `elaborate_file`
    // on the built env, not the package source in situ -- QA caught the
    // mismatch, and it is corrected here.)
    let source_id_inductive = env
        .env
        .inductive(env.globals["SourceId"])
        .expect("SourceId inductive");
    assert_eq!(source_id_inductive.constructors.len(), 1);
    let source_id_ctor_type = &source_id_inductive.constructors[0].type_;
    // R3 (Q-CLAIM-CLOSURE): `pi_arity` counts the Pi-telescope depth and
    // discards each domain, so on its own it proves only "exactly one field" --
    // the "one Nat" half of the message was carried by the message alone, and
    // changing `MkSourceId`'s field to any other single-argument type left this
    // green. Check the field COUNT and the field TYPE separately so both halves
    // of the claim are evidence.
    assert_eq!(
        pi_arity(source_id_ctor_type),
        1,
        "SourceId must wrap exactly one field"
    );
    let ken_kernel::Term::Pi(field_type, _) = source_id_ctor_type else {
        panic!("SourceId's constructor must be a one-field Pi, got {source_id_ctor_type:?}");
    };
    assert_eq!(
        head_global(field_type),
        Some(env.globals["Nat"]),
        "SourceId's single field must be Nat, not {field_type:?}"
    );

    for name in [
        "span_to_byte_range",
        "span_origin",
        "span_to_byte_range_faithful",
        "span_origin_source_faithful",
        "ValidLocated",
    ] {
        assert!(
            env.globals.contains_key(name),
            "{name} must be exported by the Parsing package"
        );
    }
    env.elaborate_file(
        "const located_probe : Located BoolExpr = MkLocated BoolExpr (MkSourceId Zero) (MkSpan Zero Zero) BTrue",
    )
    .expect("Located must carry SourceId, Span, and a value");

    // The trailing extraction-level forbidden-token scan is genuinely
    // extraction-level (guarding the literate emission, not a typed
    // declaration) -- left as-is per language-leader's review; there is no
    // elaborated-term equivalent to check it against.
    let extracted = ken_elaborator::literate::extract_ken_md(PARSING_KEN_MD)
        .expect("Capability.Parsing must extract");
    // R4 (Q-CLAIM-CLOSURE): restore the dropped `!contains("data SourceId =")`
    // provenance guard as a real substring check -- defense-in-depth, NOT the
    // sole catcher. As measured above, an in-file redeclaration is already
    // rejected at package elaboration via `span_origin`'s cross-reference; this
    // guard adds a cheap, explicit second line that also catches a
    // redeclaration reordered to dodge that cross-reference, and it makes the
    // "import, never re-declare" intent legible where the shape check cannot.
    // Substring (not whitespace-token) because the tell is the multi-token
    // phrase `data SourceId =`.
    assert!(
        !extracted.source.contains("data SourceId ="),
        "Capability.Parsing must import SourceId from Diagnostics.Core, not \
         re-declare it locally with `data SourceId =`"
    );
    // R4 (Q-CLAIM-CLOSURE): `compiler`, `AST`, and `String` restore three
    // source-text guards dropped in Q-RESIDUE with no replacement. The first
    // two guard that the package's own emission never describes itself as a
    // compiler/AST (it is a package-owned located-syntax surface, not compiler
    // internals -- cf. the D3 comment); `String` restores D1's
    // `!contains("String Nat")` guard that source lengths stay byte/Nat-based
    // and never route through a String view. All three are currently absent as
    // whitespace tokens, so this is a restored guard, not a new constraint.
    for forbidden in [
        "byte_unit_zero_int",
        "SourceLength",
        "UnitByteLength",
        "bytes_length",
        "bytes_slice",
        "bytes_at",
        "compiler",
        "AST",
        "String",
    ] {
        assert!(
            !extracted
                .source
                .split_whitespace()
                .any(|token| token == forbidden),
            "structural parsing emission must not name `{forbidden}`"
        );
    }
}

#[test]
fn cat5_d1_valid_half_open_bounds_and_zero_width_offsets_check() {
    let mut env = mk_env();
    env.elaborate_file(
        r#"
        fn zero_width_span_at_start (s : Source) : Span = MkSpan Zero Zero

        lemma valid_zero_width_span_at_start (s : Source) : ValidSpan s (zero_width_span_at_start s) =
          and_intro
            (LessEqNat (span_start (zero_width_span_at_start s)) (span_end (zero_width_span_at_start s)))
            (LessEqNat (span_end (zero_width_span_at_start s)) (source_length s))
            (LessEqNat::refl Zero)
            (LessEqNat::zero_left (source_length s))

        fn zero_width_span_at_offset (offset : Nat) : Span = MkSpan offset offset

        lemma valid_zero_width_span_at_offset (s : Source) (offset : Nat)
          : LessEqNat offset (source_length s) -> ValidSpan s (zero_width_span_at_offset offset) =
          \h. valid_zero_width_span s offset h

        lemma source_utf8_projects (s : Source) : IsUtf8 (source_bytes s) =
          source_bytes::utf8 s

        fn located_true_value (s : Source) : Located Bool =
          MkLocated Bool (source_id s) (zero_width_span_at_start s) True

        lemma valid_located_true_value (s : Source) : ValidLocated Bool s (located_true_value s) =
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
    // CLAIM LEDGER (Q-CLAIM-CLOSURE AC-3): projected_bytes == b"abc" (source_bytes
    // through the instance); projected_length == 3 (bytes_nat_length on the raw
    // const, byte arithmetic only); projected_byte_view_length == 3 [NEW, R2 --
    // the byte-view length through the instance]; projected_utf8 == Neutral (a
    // noncomputational proof field manufactures no evidence). Add an eval+assert
    // -> add its claim here.
    let mut env = mk_env();
    env.elaborate_file(
        r#"
        data ConcreteByteSource = MkConcreteByteSource

        const sample_abc_bytes : Bytes = bytes_encode "abc"
        lemma sample_utf8_valid : IsUtf8 sample_abc_bytes = Axiom

        instance Source ConcreteByteSource {
          source_id_field = MkSourceId Zero ;
          source_bytes_field = sample_abc_bytes ;
          source_utf8_field = sample_utf8_valid
        }

        const sample_source : Source = Source_instance_ConcreteByteSource

        const projected_bytes : Bytes = source_bytes sample_source
        const projected_length : Nat = bytes_nat_length sample_abc_bytes
        const projected_byte_view_length : Nat = bytes_nat_length (source_bytes sample_source)
        lemma projected_utf8 : IsUtf8 (source_bytes sample_source) =
          source_bytes::utf8 sample_source
        const full_source_span : Span = MkSpan Zero (source_length sample_source)
        lemma full_source_span_valid : ValidSpan sample_source full_source_span =
          and_intro
            (LessEqNat (span_start full_source_span) (span_end full_source_span))
            (LessEqNat (span_end full_source_span) (source_length sample_source))
            (LessEqNat::zero_left (source_length sample_source))
            (LessEqNat::refl (source_length sample_source))
        "#,
    )
    .expect("a concrete non-empty Source must compute its structural length");

    let mut store = make_store(&env);
    neutralize_fixture_proofs(&env, &mut store, &["sample_utf8_valid"]);
    let projected_bytes = eval_def(&env, &mut store, "projected_bytes");
    assert_eq!(
        projected_bytes,
        EvalVal::Bytes(b"abc".to_vec()),
        "source_bytes must execute through a concrete class-backed Source instance"
    );

    let mut length_store = make_store(&env);
    neutralize_fixture_proofs(&env, &mut length_store, &["sample_utf8_valid"]);
    // `projected_length` computes the byte count on the raw `sample_abc_bytes`
    // constant via `bytes_nat_length` -- it corroborates the arithmetic but
    // does NOT route through `source_length` or the Source instance, so its
    // message says exactly that.
    let projected_length = eval_def(&env, &mut length_store, "projected_length");
    assert_eq!(
        nat_count(&env, &projected_length),
        3,
        "bytes_nat_length must count the raw source bytes as 3"
    );
    // R2 (Q-CLAIM-CLOSURE): the "source_length executes through the
    // class-backed Source instance" claim was previously asserted with this
    // message on `projected_length` above, which never calls `source_length`
    // and never touches the Source instance -- redefining `source_length` to a
    // constant left the test green.
    //
    // Now the byte-view computation is evaluated for real over the instance,
    // and the residual is stated rather than papered over.
    //
    // What this pins is the INSTANCE-PROJECTED source_bytes byte count, and only
    // that. `source_length` is *definitionally* `bytes_nat_length
    // s.source_bytes_field` and `source_bytes` is `s.source_bytes_field`
    // (Parsing.ken.md:62,64) -- the SAME field -- so `bytes_nat_length
    // (source_bytes sample_source)` runs the same arithmetic `source_length`
    // would, over the `source_bytes` projection that reduces to the instance's
    // concrete bytes. That is a strict improvement over the old `bytes_nat_length
    // sample_abc_bytes` check, which was disconnected from the Source instance
    // entirely -- but it does NOT bind `source_length`'s body (see the residual),
    // so the claim is scoped to the source_bytes count, not to source_length.
    //
    // ⚠ RESIDUAL, measured both ways: `source_length`-BY-NAME cannot be pinned
    // behaviorally in this system, because its inlined raw `.source_bytes_field`
    // access stays NEUTRAL in both engines -- `source_length sample_source`
    // evaluates to `Unknown` in the interpreter, and `Eq Nat (source_length
    // sample_source) 3` is rejected by the kernel as "not convertible". So a
    // hostile redefinition of `source_length` to a constant would NOT be caught
    // here (only the definitionally-equal `source_bytes` form reduces). What
    // this test carries is the byte-view arithmetic over the instance plus the
    // definitional identity; `source_length`'s `Source -> Nat` signature is
    // pinned separately by `total_parser_shape_probe`'s `LessEqNat start
    // (source_length s)` bound in the D2 surface test. Pinning its dynamic value
    // would require the evaluator/conversion to reduce raw instance-field access
    // -- outside this WP's scope.
    let mut byte_view_store = make_store(&env);
    neutralize_fixture_proofs(&env, &mut byte_view_store, &["sample_utf8_valid"]);
    let projected_byte_view_length =
        eval_def(&env, &mut byte_view_store, "projected_byte_view_length");
    assert_eq!(
        nat_count(&env, &projected_byte_view_length),
        3,
        "the instance-projected source_bytes byte count must be 3 -- this pins \
         `bytes_nat_length (source_bytes sample_source)`, NOT `source_length` \
         itself, whose body is not bindable in this evaluator (see the residual \
         note above); `source_length` is shape-pinned only"
    );

    let projected_utf8 = eval_def(&env, &mut store, "projected_utf8");
    assert_eq!(
        projected_utf8,
        EvalVal::Neutral,
        "projecting a noncomputational proof field must not manufacture evidence"
    );
}

#[test]
fn cat5_d1_end_past_source_length_rejected() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const invalid_span : Span = MkSpan Zero (Suc (Suc (Suc Zero)))
            lemma invalid_span_valid (s : Source) : ValidSpan s invalid_span =
              and_intro
                (LessEqNat (span_start invalid_span) (span_end invalid_span))
                (LessEqNat (span_end invalid_span) (source_length s))
                Proved
                Proved
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
            lemma invalid_span_valid (s : Source) : ValidSpan s invalid_span =
              and_intro
                (LessEqNat (span_start invalid_span) (span_end invalid_span))
                (LessEqNat (span_end invalid_span) (source_length s))
                Proved
                Proved
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
fn cat5_d1_reflexive_utf8_proof_rejected() {
    let mut env = mk_env();
    let err = env
        .elaborate_file(
            r#"
            const sample_bytes : Bytes = bytes_encode "abc"
            lemma fake_utf8 : IsUtf8 sample_bytes = Refl
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
}

#[test]
fn cat5_d2_success_parser_carries_valid_consumed_span_from_start() {
    let mut env = mk_env();
    env.elaborate_file(
        r#"
        const success_parser : Parser Bool =
          parser_pure Bool True

        lemma success_parser_valid : ParserValid Bool success_parser =
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

        lemma success_parser_laws : ParserLaws Bool success_parser =
          and_intro
            (ParserValid Bool success_parser)
            (And (ParserTotal Bool success_parser) (ParserSourceLocal Bool success_parser))
            success_parser_valid
            (and_intro
              (ParserTotal Bool success_parser)
              (ParserSourceLocal Bool success_parser)
              (\s. \start. \h. Proved)
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

        lemma failed_parser_valid : ParserValid Bool failed_parser =
          \s. \start. \h.
            and_intro
              (Equal SourceId (error_source (MkParseError (source_id s) (MkSpan start start))) (source_id s))
              (ValidSpan s (error_span (MkParseError (source_id s) (MkSpan start start))))
              Refl
              (valid_zero_width_span s start h)

        lemma failed_parser_source_local : ParserSourceLocal Bool failed_parser =
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

            lemma wrong_source_failed_parser_valid : ParserValid Bool wrong_source_failed_parser =
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

            lemma invalid_span_failed_parser_valid : ParserValid Bool invalid_span_failed_parser =
              \s. \start. \h.
                and_intro
                  (Equal SourceId (error_source (MkParseError (source_id s) (MkSpan (Suc (Suc Zero)) (Suc Zero)))) (source_id s))
                  (ValidSpan s (error_span (MkParseError (source_id s) (MkSpan (Suc (Suc Zero)) (Suc Zero)))))
                  Refl
                  (and_intro
                    (LessEqNat (span_start (MkSpan (Suc (Suc Zero)) (Suc Zero))) (span_end (MkSpan (Suc (Suc Zero)) (Suc Zero))))
                    (LessEqNat (span_end (MkSpan (Suc (Suc Zero)) (Suc Zero))) (source_length s))
                    Proved
                    (LessEqNat::zero_left (source_length s)))
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
fn cat5_d2_legacy_unguarded_repeat_is_not_exported() {
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
        .expect_err("the legacy unguarded repetition entry point must stay absent");
    let msg = format!("{err}");
    assert!(
        msg.contains("unresolved type 'repeat'")
            || msg.contains("unresolved")
            || msg.contains("Unresolved"),
        "unguarded repeat should reject at producer/name resolution, got {msg}"
    );
}

#[test]
fn cat5_d2_legacy_caller_budget_repetition_is_not_exported() {
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
        .expect_err("CAT-5 must not retain the old caller-budget repetition helper");
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
    env.elaborate_file(
        r#"
        data PrintedBoolExprSource = MkPrintedBoolExprSource
        data FormattedBoolExprSource = MkFormattedBoolExprSource
        data InfixBoolExprSource = MkInfixBoolExprSource

        const representative_bool_expr : BoolExpr =
          BAnd BTrue (BNot BFalse)

        const printed_bool_expr_bytes : Bytes =
          print_bool_expr representative_bool_expr

        lemma printed_bool_expr_utf8 : IsUtf8 printed_bool_expr_bytes = Axiom
        instance Source PrintedBoolExprSource {
          source_id_field = MkSourceId (Suc (Suc Zero)) ;
          source_bytes_field = printed_bool_expr_bytes ;
          source_utf8_field = printed_bool_expr_utf8
        }

        const printed_bool_expr_source : Source = Source_instance_PrintedBoolExprSource

        const parse_printed_bool_expr : ParseResult (Syntax BoolExpr) =
          parse_bool_expr printed_bool_expr_source Zero (LessEqNat::zero_left (source_length printed_bool_expr_source))

        const parse_printed_bool_expr_erases : Bool =
          match parse_printed_bool_expr {
            Parsed syntax consumed next |-> bool_expr_eq (erase_spans syntax) representative_bool_expr ;
            Failed err |-> False
          }

        const parsed_bool_expr_syntax : Syntax BoolExpr =
          match parse_printed_bool_expr {
            Parsed syntax consumed next |-> syntax ;
            Failed err |-> syntax_leaf printed_bool_expr_source Zero Zero BFalse
          }

        const format_printed_bool_expr : Result ParseError Bytes =
          format_bool_expr printed_bool_expr_source

        const formatted_bool_expr_bytes : Bytes =
          match format_printed_bool_expr {
            Ok bs |-> bs ;
            Err err |-> bytes_encode "ERR"
          }

        lemma formatted_bool_expr_utf8 : IsUtf8 formatted_bool_expr_bytes = Axiom
        instance Source FormattedBoolExprSource {
          source_id_field = MkSourceId (Suc (Suc (Suc Zero))) ;
          source_bytes_field = formatted_bool_expr_bytes ;
          source_utf8_field = formatted_bool_expr_utf8
        }

        const formatted_bool_expr_source : Source = Source_instance_FormattedBoolExprSource

        const reformat_bool_expr : Result ParseError Bytes =
          format_bool_expr formatted_bool_expr_source

        const reformatted_bool_expr_bytes : Bytes =
          match reformat_bool_expr {
            Ok bs |-> bs ;
            Err err |-> bytes_encode "ERR"
          }

        const infix_bool_expr_bytes : Bytes = bytes_encode "true and false"
        lemma infix_bool_expr_utf8 : IsUtf8 infix_bool_expr_bytes = Axiom
        instance Source InfixBoolExprSource {
          source_id_field = MkSourceId (Suc (Suc (Suc (Suc Zero)))) ;
          source_bytes_field = infix_bool_expr_bytes ;
          source_utf8_field = infix_bool_expr_utf8
        }

        const infix_bool_expr_source : Source = Source_instance_InfixBoolExprSource

        const parse_infix_bool_expr : ParseResult (Syntax BoolExpr) =
          parse_bool_expr infix_bool_expr_source Zero (LessEqNat::zero_left (source_length infix_bool_expr_source))
        "#,
    )
    .expect("D3 Boolean parser/printer/formatter producer path must elaborate");

    let mut store = make_store(&env);
    neutralize_fixture_proofs(
        &env,
        &mut store,
        &[
            "record_nil_val",
            "printed_bool_expr_utf8",
            "formatted_bool_expr_utf8",
            "infix_bool_expr_utf8",
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
