//! B2 acceptance tests — `Temporal Σ` datatype + export-flow conformance cases
//! from `conformance/behavioral/temporal/seed-temporal.md` (TE-A … TE-F).
//!
//! Each test maps to a named conformance case. Every test routes **real
//! machinery** and asserts a **structural property** — never a synthetic
//! literal where a real elaboration/admission/export is asserted (`72 §9`):
//!
//! - **TE-A** — the real `declare_inductive` + `check_positivity` (the kernel
//!   that exists now) admit the first-order `Temporal` and reject the HOAS
//!   variant (the non-degenerate positivity pair).
//! - **TE-B** — the no-kernel-modality absence net, pinned on the **structural
//!   signature** (the `Term` enum variant / `▷`), not the lexeme — `later` is
//!   plain English in kernel prose (`obs.rs` "reduces lazily by later whnf
//!   calls", `subst.rs` "by a later iteration"); a lexeme grep is bistable.
//! - **TE-C** — the derived operators elaborate to the `until`/`not` core (a
//!   structural assertion on the elaborated `Temporal` term head).
//! - **TE-D** — a surface `temporal{}` block elaborates to the §3 constructors,
//!   tagged `delegated`, human-visible (verbatim source, not erased).
//! - **TE-E** — a real `temporal{}` value routed through the real B1 emitter
//!   (`emit_checked_target_export`) lands in `T`/`delegated`, never `Q`/`P`; the one-way gate
//!   holds even beside a real `Verdict::Proved` (no promotion edge).
//! - **TE-F** — `closed` (the `elim_Temporal` analog) computes the bound/free
//!   verdict flip (reason-*about*); and the obligation is **not dischargeable
//!   in Ken** (no modality + delegated-only export — reason-*with* is
//!   impossible).
//!
//! Built from `/spec` (`72`) + `/conformance` (`seed-temporal.md`) only.

use std::collections::BTreeSet;

use ken_elaborator::{
    closed,
    compiler_driver::{compile_checked_target_denotation, CompilerSource},
    effects::row::EffectRow,
    elaborate_temporal_expr, emit_checked_target_export,
    error::Span,
    extract::{ObligationId, ObligationTriple, ProvKind, Provenance},
    prover::Verdict,
    serialize_export, temporal_hoas_inductive_spec, temporal_inductive_spec, ElabEnv, ExportError,
    GEntry, Pred, TEntry, Temporal, TemporalExpr, Var,
};

fn emit_export(
    target_name: &str,
    results: &[(ObligationTriple, Verdict)],
    trusted_base: &BTreeSet<GlobalId>,
    legacy_alphabet: EffectRow,
    generators: Vec<GEntry>,
    temporal: Vec<TEntry>,
) -> Result<ken_elaborator::BehavioralExport, ExportError> {
    assert!(legacy_alphabet.effects().next().is_none());
    let source = format!("fn {target_name} (value : Unit) : Unit = value");
    let denotation = compile_checked_target_denotation(
        &format!("b2_acceptance_{target_name}"),
        CompilerSource::new("fixture.ken", source),
        target_name,
    )
    .expect("pure checked target denotation");
    emit_checked_target_export(&denotation, results, trusted_base, generators, temporal)
}
use ken_kernel::{
    declare_inductive,
    inductive::{check_positivity, method_type, peel_app, peel_pi, recursive_args},
    CtorSpec, GlobalEnv, GlobalId, InductiveSpec, KernelError, Level, Term,
};

// ─── TE-A. `Temporal` is ordinary inert data — admitted by K1 (AC1) ──────────

/// TE-A1: temporal/ordinary-inductive-admitted-by-k1 (AC1)
///
/// The first-order `Temporal Σ` datatype, declared via the real L2 `data`
/// machinery (`declare_inductive` → `check_positivity` + `build_types`), is
/// admitted by K1's strict-positivity check **without** the K1.5 W-style path:
/// every recursive occurrence is direct (empty branching telescope). The
/// eliminator is the ordinary generated `elim_Temporal`. Half of the
/// positivity pair — alone it is green-vs-green under an over-permissive
/// check; the net is the pair with TE-A2.
#[test]
fn ordinary_inductive_admitted_by_k1() {
    let mut env = GlobalEnv::new();

    // Two distinct concrete small types instantiate the deferred `Pred Σ`
    // and `Var` carriers at the ordinary eliminator use site.
    let pred_id = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec {
            args: vec![],
            target_indices: vec![],
        }],
    })
    .expect("concrete small predicate carrier");
    let var_id = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![],
    })
    .expect("concrete small variable carrier");

    let d_id = declare_inductive(&mut env, temporal_inductive_spec)
        .expect("first-order Temporal is admitted by K1");

    let ind = env
        .inductive(d_id)
        .expect("Temporal family is in the env after admission");
    // The LTL/μ core: atom/not/and/or/next/until/mu/nu/var.
    assert_eq!(ind.constructors.len(), 9, "the §3 constructor set");
    assert_eq!(
        ind.params,
        vec![Term::ty(Level::zero()), Term::ty(Level::zero())],
        "the witness former has exactly the two small type parameters P and V"
    );
    let (former_params, former_result) = peel_pi(&ind.former_type);
    assert_eq!(
        former_params.len(),
        2,
        "Temporal's former has two parameters"
    );
    assert_eq!(
        former_result,
        Term::ty(Level::zero()),
        "Temporal P V remains in Type 0"
    );
    assert_eq!(
        ind.constructors[0].args,
        vec![Term::var(1)],
        "atom takes P from the two-parameter telescope"
    );
    for k in [6, 7] {
        assert_eq!(
            ind.constructors[k].args[0],
            Term::var(0),
            "mu/nu take V before their recursive body"
        );
    }
    assert_eq!(
        ind.constructors[8].args,
        vec![Term::var(0)],
        "var takes V from the two-parameter telescope"
    );

    // Re-run the real strict-positivity check on the admitted family — the
    // spec's "admitted by K1" grounded against the kernel that exists now.
    assert!(
        check_positivity(&ind).is_ok(),
        "real check_positivity admits the first-order Temporal"
    );

    // No K1.5 W-style admission path: every recursive arg is **direct** (an
    // empty branching telescope). `recursive_args` returns the Π-bound
    // (branching) telescope per recursive arg; empty ⇒ plain K1, not K1.5.
    let expected_recursive_positions: &[&[usize]] =
        &[&[], &[0], &[0, 1], &[0, 1], &[0], &[0, 1], &[1], &[1], &[]];
    for (c, expected_positions) in ind.constructors.iter().zip(expected_recursive_positions) {
        let recursive = recursive_args(c, d_id, 2);
        assert_eq!(
            recursive.iter().map(|(pos, _, _)| *pos).collect::<Vec<_>>(),
            *expected_positions,
            "recursive positions for constructor {:?}",
            c.id
        );
        for (pos, branching_tel, idxs) in recursive {
            assert!(
                branching_tel.is_empty(),
                "no K1.5 W-style path: {:?}'s recursive arg is direct (empty branching telescope)",
                c.id
            );
            assert!(idxs.is_empty(), "Temporal remains non-indexed");

            let (head, args) = peel_app(&c.args[pos]);
            assert_eq!(head, Term::indformer(d_id, vec![]));
            assert_eq!(
                args,
                vec![Term::var(pos + 1), Term::var(pos)],
                "recursive D P V is fully applied at constructor depth {pos}"
            );
        }
    }

    // The eliminator is the **ordinary generated** `elim_Temporal` — the real
    // `method_type` machinery produces a method type for `Temporal`'s
    // constructors (no special form, no kernel extension). Motive `λφ. Type 0`.
    let pred_ty = Term::indformer(pred_id, vec![]);
    let var_ty = Term::indformer(var_id, vec![]);
    let temporal_ty = Term::app(
        Term::app(Term::indformer(d_id, vec![]), pred_ty.clone()),
        var_ty.clone(),
    );
    let motive = Term::lam(temporal_ty, Term::ty(Level::zero()));
    let m0 = method_type(&ind, 0, &motive, &[pred_ty, var_ty], &[]);
    assert!(
        matches!(m0, Term::Pi(..)),
        "the ordinary eliminator generates a method type for `atom` (got {:?})",
        m0
    );
}

/// TE-A2: temporal/hoas-foil-reaches-positivity-not-universe-gate (AC1,
/// soundness)
///
/// The HOAS variant of the fixpoint binder — `mu : (Temporal → Temporal) →
/// Temporal` — places `Temporal` in a **negative** position (the domain of the
/// inner arrow). The **same** `check_positivity` that admits the first-order
/// datatype (TE-A1) **rejects** this. The non-degenerate pair (one datatype,
/// two binder encodings, opposite verdicts) is the sole net for "first-order
/// binding is load-bearing"; the verdict flips on the structural discriminator.
#[test]
fn hoas_foil_reaches_positivity_violation_not_universe_gate() {
    let mut env = GlobalEnv::new();
    let err = declare_inductive(&mut env, temporal_hoas_inductive_spec)
        .expect_err("HOAS mu must be rejected by strict positivity");
    assert!(
        matches!(err, KernelError::PositivityViolation(_)),
        "HOAS mu breaks strict positivity (negative occurrence), got {:?}",
        err
    );
}

// ─── TE-B. No kernel modality — the structural absence (AC1, soundness) ──────

/// TE-B1: temporal/no-modal-construct-in-kernel (AC1, soundness)
///
/// The data-only decision (`OQ-temporal`) is realized as an **absence in the
/// kernel**: no `▷`/later/tick/Löb/clock construct exists in the term language.
/// The net is pinned on the **structural signature** — the `Term` enum variant
/// — not the lexeme. **Collision named verbatim:** the English word "later"
/// appears in kernel prose (`obs.rs` "reduces lazily by later `whnf` calls",
/// `subst.rs` "by a later iteration") and is NOT a modal construct; a lexeme
/// grep for "later" would false-alarm on that prose (or, tuned to pass, be
/// permissive enough to miss a real `Term::Later`). The net targets the
/// construct signature, so the English "later" in `obs.rs` does not trip it.
///
/// **Disconfirming check:** a kernel that grew a `Term::Later` variant would
/// make `term_src` contain `"Later"` → red. It does not → the absence is
/// guard-gated, not coincidental.
#[test]
fn no_modal_construct_in_kernel() {
    let term_src = include_str!("../../ken-kernel/src/term.rs");

    // The construct signature is absent from the term-language definition.
    // (`Term::Later`/`Tick`/`Clock`/`Lob`/`Modality` would be variants here.)
    assert!(!term_src.contains("Later"), "no `Term::Later` modality");
    assert!(!term_src.contains("Tick"), "no tick-variable construct");
    assert!(!term_src.contains("Clock"), "no clock-structure construct");
    assert!(!term_src.contains("Modality"), "no modality construct");
    assert!(!term_src.contains("▷"), "no `▷`/later modality symbol");

    // Collision named verbatim: the English word "later" IS in kernel prose
    // (incidental, not a construct) — the net is robust to this lexeme.
    let obs_src = include_str!("../../ken-kernel/src/obs.rs");
    assert!(
        obs_src.contains("later"),
        "the lexeme collision is real: `later` is plain English in `obs.rs` prose"
    );
    // And that prose does not make `term.rs` contain a construct — the net
    // targets the construct signature, not the lexeme.
    assert!(
        !term_src.contains("later"),
        "the term language has no `later` construct (the prose collision is in `obs.rs`, not `term.rs`)"
    );
}

/// TE-B2: temporal/inert-to-conversion (AC1, soundness)
///
/// `Temporal` is inert to conversion: a program with a `Temporal` value has
/// the **same** conversion algorithm and typing judgments as one without. The
/// kernel's conversion (`conv.rs`) has no `Temporal`-specific branch — adding
/// `Temporal` adds an inductive type and its ordinary ι-rule, nothing more.
///
/// **Disconfirming check:** a kernel that added a `Temporal`-specific conv/η
/// rule would make `conv_src` contain `"Temporal"` → red. It does not →
/// conversion is byte-for-byte unchanged.
#[test]
fn inert_to_conversion() {
    let conv_src = include_str!("../../ken-kernel/src/conv.rs");
    assert!(
        !conv_src.contains("Temporal") && !conv_src.contains("temporal"),
        "conversion has no `Temporal`-specific branch — adding `Temporal` leaves conv unchanged"
    );
}

// ─── TE-C. Derived operators elaborate to the core (AC2) ────────────────────

/// TE-Across the `◇`/`□`/`leadsto` dimensions, each derived operator is its
/// own structural case (the multi-dimensional-guard rule).

/// TE-C1: temporal/eventually-elaborates-to-until-true (AC2)
///
/// `◇φ` elaborates to `until (atom ⊤) φ` — the head is `until`, not a
/// dedicated `eventually`/`diamond` constructor (no such constructor exists).
#[test]
fn eventually_elaborates_to_until_true() {
    let t = elaborate_temporal_expr(&TemporalExpr::Eventually(Box::new(TemporalExpr::Atom(
        "p".into(),
    ))));
    let expected = Temporal::Until(
        Box::new(Temporal::Atom(Pred::Top)),
        Box::new(Temporal::Atom(Pred::Event("p".into()))),
    );
    assert_eq!(t, expected, "◇φ := until (atom ⊤) φ — head `until`");
}

/// TE-C2: temporal/always-elaborates-to-not-until-not (AC2)
///
/// `□φ` elaborates to `not (until (atom ⊤) (not φ))` — head `not`, inner
/// `until`, innermost `not φ`. The `□` dimension is distinct from `◇`: a
/// primitive-`□` bug (head `box`) is undetected by TE-C1 alone.
#[test]
fn always_elaborates_to_not_until_not() {
    let t = elaborate_temporal_expr(&TemporalExpr::Always(Box::new(TemporalExpr::Atom(
        "p".into(),
    ))));
    let expected = Temporal::Not(Box::new(Temporal::Until(
        Box::new(Temporal::Atom(Pred::Top)),
        Box::new(Temporal::Not(Box::new(Temporal::Atom(Pred::Event(
            "p".into(),
        ))))),
    )));
    assert_eq!(
        t, expected,
        "□φ := not (until (atom ⊤) (not φ)) — head `not`"
    );
}

/// TE-C3: temporal/leadsto-elaborates-to-box-of-or-diamond (AC2)
///
/// `p ~> q` elaborates to `□ (not p or ◇ q)` ⇒ fully
/// `not (until (atom ⊤) (not (or (not p) (until (atom ⊤) q))))` — built
/// entirely from the `until`/`not`/`or`/`atom` core; two layers of derivation,
/// so a primitive at any layer surfaces in the elaborated tree.
#[test]
fn leadsto_elaborates_to_box_of_or_diamond() {
    let t = elaborate_temporal_expr(&TemporalExpr::Leadsto(
        Box::new(TemporalExpr::Atom("p".into())),
        Box::new(TemporalExpr::Atom("q".into())),
    ));
    let expected = Temporal::Not(Box::new(Temporal::Until(
        Box::new(Temporal::Atom(Pred::Top)),
        Box::new(Temporal::Not(Box::new(Temporal::Or(
            Box::new(Temporal::Not(Box::new(Temporal::Atom(Pred::Event(
                "p".into(),
            ))))),
            Box::new(Temporal::Until(
                Box::new(Temporal::Atom(Pred::Top)),
                Box::new(Temporal::Atom(Pred::Event("q".into()))),
            )),
        )))),
    )));
    assert_eq!(
        t, expected,
        "p ~> q := □(¬p ∨ ◇q) = not(until(atom⊤, not(or(not p, until(atom⊤, q)))))"
    );
}

// ─── TE-D. Surface `temporal{}` → `delegated`, human-visible (AC3) ───────────

/// TE-D1: temporal/block-elaborates-delegated-and-visible (AC3)
///
/// A surface `temporal { eventually settled }` claim run through the real
/// elaborator (`parse → resolve → elaborate`) elaborates to the §3
/// constructors (`until (atom ⊤) settled`), is tagged **`delegated`** (pinned
/// via the real export flow), and is **human-visible** (the verbatim formula
/// text is carried, not erased).
#[test]
fn block_elaborates_delegated_and_visible() {
    let mut elab = ElabEnv::new().expect("ElabEnv");
    let result = elab
        .elaborate_decl_v1("temporal safety { eventually settled }")
        .expect("elaborate temporal block");
    assert_eq!(
        result.temporal_obligations.len(),
        1,
        "one delegated temporal obligation"
    );
    let obl = &result.temporal_obligations[0];

    // Elaborates to the §3 constructors: `eventually settled` = `until (atom ⊤) settled`.
    assert_eq!(
        obl.formula,
        Temporal::Until(
            Box::new(Temporal::Atom(Pred::Top)),
            Box::new(Temporal::Atom(Pred::Event("settled".into()))),
        ),
        "the derived `◇` elaborates to the `until` core"
    );

    // Human-visible: the verbatim formula text is carried (not erased, `72 §4`).
    assert!(
        obl.source.contains("eventually") && obl.source.contains("settled"),
        "human-visible source carried verbatim: {:?}",
        obl.source
    );

    // Tagged `delegated` — pinned via the REAL export flow (not a status string
    // on the obligation; the status is the constant `delegated` in `serialize_export`).
    let tentry = TEntry {
        obligation_id: obl.id.clone(),
        formula: obl.formula.clone(),
    };
    let export = emit_export(
        "safety",
        &[],
        &BTreeSet::new(),
        EffectRow::empty(),
        vec![],
        vec![tentry],
    )
    .expect("export");
    let ser = serialize_export(&export);
    assert_eq!(
        ser["obligations"][0]["status"].as_str().unwrap(),
        "delegated",
        "the temporal claim is tagged delegated (never proved/tested/unknown)"
    );
    assert!(
        ser["guarantees"].as_array().unwrap().is_empty(),
        "a delegated claim is not a guarantee (Q)"
    );
}

// ─── TE-E. Export flow — `T`/`delegated`, never `Q`/`P`, one-way (AC4) ────────

/// TE-E1: temporal/value-projects-to-T-delegated-never-q (AC4, soundness)
///
/// A real elaborated `temporal{}` value routed through the **real B1 emitter**
/// (`emit_export`) lands in `T` (`obligations`) tagged `delegated`, and is
/// **absent** from `Q` (`guarantees`) and `P` (`assumptions`) — the total,
/// constant mapping `Temporal`-in-source ↦ `delegated` ↦ `T` (`72 §5`).
#[test]
fn value_projects_to_t_delegated_never_q() {
    let mut elab = ElabEnv::new().expect("ElabEnv");
    let result = elab
        .elaborate_decl_v1("temporal ltl_safety { always not crashed }")
        .expect("elaborate");
    let obl = &result.temporal_obligations[0];

    // A real elaborated `temporal{}` value as the `TEntry` body.
    let tentry = TEntry {
        obligation_id: obl.id.clone(),
        formula: obl.formula.clone(),
    };
    let export = emit_export(
        "f",
        &[],
        &BTreeSet::new(),
        EffectRow::empty(),
        vec![],
        vec![tentry],
    )
    .expect("export");

    // Lands in T (obligations).
    assert_eq!(export.obligations.len(), 1, "the temporal value lands in T");
    assert_eq!(export.obligations[0].obligation_id, obl.id);
    // Absent from Q (guarantees) and P (assumptions).
    assert!(
        export.guarantees.is_empty(),
        "never Q — a delegated property is not a guarantee"
    );
    assert!(
        export.assumptions.is_empty(),
        "never P — a delegated property is not an assumption"
    );

    let ser = serialize_export(&export);
    assert_eq!(
        ser["obligations"][0]["status"].as_str().unwrap(),
        "delegated",
        "status is the constant `delegated`"
    );
    assert!(ser["guarantees"].as_array().unwrap().is_empty(), "Q empty");
    assert!(ser["assumptions"].as_array().unwrap().is_empty(), "P empty");
    // The formula body is present in the wire form (B2 fills the TEntry body).
    assert!(
        ser["obligations"][0]["formula"].is_string(),
        "the elaborated Temporal value is the T-entry body"
    );
}

/// TE-E2: temporal/ward-green-keeps-delegated-never-promoted (AC4, soundness)
///
/// The one-way gate (`71 §5.1`/I4): a `delegated` `T` obligation **stays
/// `delegated`** even after a `Ward` green discharge, and is **never** promoted
/// to `Q`. The gate is a **structural absence of a promotion edge**, not a
/// runtime check: `QEntry` is built only in the `Verdict::Proved` arm of
/// `emit_export`; the `temporal` parameter flows straight into `T`.
///
/// Discriminating: even beside a real kernel `Verdict::Proved` (a different
/// obligation → `Q`), the temporal `T` is NOT promoted. An emitter with a
/// promotion edge (dump everything into `Q`, or accept a Ward green into `Q`)
/// would land the temporal id in `Q` → red. It does not → the case flips.
#[test]
fn ward_green_keeps_delegated_never_promoted() {
    let mut elab = ElabEnv::new().expect("ElabEnv");
    let result = elab
        .elaborate_decl_v1("temporal liveness { eventually done }")
        .expect("elaborate");
    let obl = &result.temporal_obligations[0];
    let tentry = TEntry {
        obligation_id: obl.id.clone(),
        formula: obl.formula.clone(),
    };

    // A real kernel-proved obligation (a different id) — its hole is absent
    // from `trusted_base`, so it projects to `Q`. This is the honest path; it
    // must NOT drag the temporal `T` into `Q`.
    let proved = ObligationTriple {
        id: ObligationId("g1.ensures.0".into()),
        hole_id: GlobalId(999),
        context: vec![],
        phi: Term::omega(Level::zero()),
        goal_closed: Term::omega(Level::zero()),
        provenance: Provenance {
            kind: ProvKind::Ensures { index: 0 },
            span: Span::zero(),
        },
    };
    let results = vec![(
        proved,
        Verdict::Proved {
            cert: Term::omega(Level::zero()),
        },
    )];
    let export = emit_export(
        "f",
        &results,
        &BTreeSet::new(), // g1's hole is absent → Q; no trusted_base membership for the temporal id
        EffectRow::empty(),
        vec![],
        vec![tentry],
    )
    .expect("export");

    // The proved obligation IS in Q (the honest path works).
    assert!(
        export
            .guarantees
            .iter()
            .any(|q| q.obligation_id == "g1.ensures.0"),
        "the real proved obligation projects to Q"
    );
    // The temporal obligation stays in T — never promoted to Q.
    assert!(
        export.obligations.iter().any(|t| t.obligation_id == obl.id),
        "the temporal obligation stays in T"
    );
    assert!(
        !export.guarantees.iter().any(|q| q.obligation_id == obl.id),
        "the temporal obligation is never promoted to Q (one-way gate, I4)"
    );
    // Its status stays `delegated` (a Ward green re-enters only as a TEntry,
    // never re-stamping the obligation `proved`).
    let ser = serialize_export(&export);
    let t_json = ser["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["obligation_id"].as_str() == Some(obl.id.as_str()))
        .expect("temporal entry in serialized T");
    assert_eq!(
        t_json["status"].as_str().unwrap(),
        "delegated",
        "a Ward green never re-stamps the obligation `proved`"
    );
}

// ─── TE-F. Reason-*about*, not -*with* — both faces (AC5) ────────────────────

/// TE-F1: temporal/closedness-metatheorem-typechecks-via-elim (AC5)
///
/// `closed : Temporal Σ → Bool` is ordinary structural recursion over the
/// datatype (the `elim_Temporal` analog) with a binder environment — no
/// trace/satisfaction model, no new kernel power. It **computes
/// discriminatingly**: `var X` bound by an enclosing `mu X`/`nu X` → `true`;
/// the same `var X` free → `false` (the bound/free verdict flip). `closed` is
/// preserved by the structural/derived operations. (The definition type-checks
/// as ordinary static proof — it is a well-typed Rust structural recursion,
/// compiled here.)
#[test]
fn closedness_metatheorem_typechecks_via_elim() {
    // Bound: `var X` under an enclosing `mu X` → closed.
    let bound = Temporal::Mu {
        var: Var("X".into()),
        body: Box::new(Temporal::Var(Var("X".into()))),
    };
    assert!(closed(&bound), "a var bound by an enclosing mu is closed");

    // Free: `var X` with no enclosing binder → not closed. The verdict FLIPS.
    let free = Temporal::Var(Var("X".into()));
    assert!(
        !closed(&free),
        "a free var is not closed (the flip pins that `closed` inspects structure)"
    );

    // `nu` binds too (the greatest-fixpoint binder).
    let bound_nu = Temporal::Nu {
        var: Var("Z".into()),
        body: Box::new(Temporal::Var(Var("Z".into()))),
    };
    assert!(closed(&bound_nu), "nu binds vars too");

    // A var bound by an OUTER mu, referenced inside a derived op under it, is closed.
    let nested = Temporal::Mu {
        var: Var("W".into()),
        body: Box::new(Temporal::eventually(&Temporal::Var(Var("W".into())))),
    };
    assert!(
        closed(&nested),
        "a var bound by an outer mu, referenced inside `eventually`, is closed"
    );

    // A surface formula with no fixpoint vars is closed.
    let closed_formula = elaborate_temporal_expr(&TemporalExpr::Eventually(Box::new(
        TemporalExpr::Atom("p".into()),
    )));
    assert!(
        closed(&closed_formula),
        "a formula with no fixpoint vars is closed"
    );

    // Preservation: `closed(eventually(φ))` == `closed(φ)` — derived/structural
    // ops build `closed` from `closed`.
    assert!(
        closed(&Temporal::eventually(&bound)),
        "eventually of a closed formula is closed (preservation)"
    );
    assert!(
        !closed(&Temporal::eventually(&free)),
        "eventually of a free-var formula is not closed (preservation)"
    );
}

/// TE-F2: temporal/obligation-not-dischargeable-in-ken (AC5, soundness)
///
/// There is **no way** to discharge a temporal obligation `□(req → ◇resp)`
/// inside Ken — no `▷`/modality, no internal model-check, no kernel decision
/// procedure over a system's infinite traces. The only outcomes are (a)
/// reason **about** the formula as data (TE-F1) or (b) **export + delegate** to
/// `Ward` (TE-E). The impossibility is the composition of TE-B1 (no modality)
/// + the delegated-only export path + the absence of any internal discharge
/// function (`sat`/`compile` are deferred to the Ward encoding pass).
#[test]
fn obligation_not_dischargeable_in_ken() {
    // `□(req → ◇resp)` = `always (not req or eventually resp)`.
    let req = Temporal::Atom(Pred::Event("req".into()));
    let resp = Temporal::Atom(Pred::Event("resp".into()));
    let liveness = Temporal::always(&Temporal::Or(
        Box::new(Temporal::Not(Box::new(req))),
        Box::new(Temporal::eventually(&resp)),
    ));

    // (a) No kernel modality to discharge it with (TE-B1): the term language
    // has no `▷`/later/tick/Modality construct.
    let term_src = include_str!("../../ken-kernel/src/term.rs");
    assert!(
        !term_src.contains("Later")
            && !term_src.contains("Tick")
            && !term_src.contains("Modality")
            && !term_src.contains("▷"),
        "no modal construct exists to discharge a temporal obligation in Ken"
    );

    // (b) Its only projection is `delegated`/`T` (TE-E): route through the real
    // emitter — it lands in T, never Q (Ken did not discharge it).
    let tentry = TEntry {
        obligation_id: "sys.liveness.0".into(),
        formula: liveness.clone(),
    };
    let export = emit_export(
        "sys",
        &[],
        &BTreeSet::new(),
        EffectRow::empty(),
        vec![],
        vec![tentry],
    )
    .expect("export");
    assert_eq!(
        export.obligations.len(),
        1,
        "the obligation is delegated (T)"
    );
    assert!(
        export.guarantees.is_empty(),
        "never Q — the obligation is not dischargeable in Ken"
    );
    assert_eq!(
        serialize_export(&export)["obligations"][0]["status"]
            .as_str()
            .unwrap(),
        "delegated",
        "the discharge arrives only out-of-band (delegated)"
    );

    // (c) No internal model-check / discharge function exists — `sat`/`compile`
    // are deferred to the joint Ward encoding pass (`72 §6.2`/§6.3). The
    // about-operation `closed` IS present (reason-about works, TE-F1).
    let temporal_src = include_str!("../src/temporal.rs");
    assert!(
        !temporal_src.contains("fn sat")
            && !temporal_src.contains("fn compile")
            && !temporal_src.contains("fn model_check")
            && !temporal_src.contains("fn discharge"),
        "no internal discharge path — `sat`/`compile`/`model_check` are deferred to Ward"
    );
    assert!(
        temporal_src.contains("fn closed"),
        "the about-operation `closed` is present (reason-about works); reason-with does not"
    );
}

// ─── Cross-case sweep — the constant verdict mapping (`72 §5`) ───────────────

/// Every `Temporal`-in-source obligation maps to **exactly** `delegated`/`T`,
/// never `Q`/`P`/`unknown`. The mapping is total and constant (the verdict-
/// mapping silence is foreclosed at source). Swept across the `◇`/`□`/`~>`/
/// `next` operators.
#[test]
fn cross_case_verdict_mapping_is_constant() {
    let mut elab = ElabEnv::new().expect("ElabEnv");
    let formulas = [
        "temporal a { eventually settled }",
        "temporal b { always not crashed }",
        "temporal c { req leadsto resp }",
        "temporal d { next req }",
    ];
    for src in formulas {
        let result = elab.elaborate_decl_v1(src).expect("elaborate");
        assert_eq!(
            result.temporal_obligations.len(),
            1,
            "{}: one temporal obligation",
            src
        );
        let obl = &result.temporal_obligations[0];
        let tentry = TEntry {
            obligation_id: obl.id.clone(),
            formula: obl.formula.clone(),
        };
        let export = emit_export(
            "f",
            &[],
            &BTreeSet::new(),
            EffectRow::empty(),
            vec![],
            vec![tentry],
        )
        .expect("export");
        assert_eq!(export.obligations.len(), 1, "{}: lands in T", src);
        assert!(export.guarantees.is_empty(), "{}: never Q", src);
        assert!(export.assumptions.is_empty(), "{}: never P", src);
        assert_eq!(
            serialize_export(&export)["obligations"][0]["status"]
                .as_str()
                .unwrap(),
            "delegated",
            "{}: status is the constant `delegated`",
            src
        );
    }
}

#[test]
fn temporal_symbols_consume_the_one_checked_b1_alphabet() {
    let denotation = compile_checked_target_denotation(
        "b2_consumer_closure",
        CompilerSource::new(
            "consumer.ken",
            r#"
proc target (_value : Unit)
  : HostIO AFull (Result IOError Unit) visits [Console] =
  host_console AFull (Result IOError Unit) (flush Stdout)
"#,
        ),
        "target",
    )
    .expect("checked B1 producer");
    let export = emit_checked_target_export(
        &denotation,
        &[],
        &BTreeSet::new(),
        vec![],
        vec![TEntry {
            obligation_id: "console-eventual".to_string(),
            formula: Temporal::eventually(&Temporal::Atom(Pred::Event("Console".to_string()))),
        }],
    )
    .expect("B2 consumes checked B1 export");

    let Temporal::Until(_, eventual) = &export.obligations[0].formula else {
        panic!("eventually encoding");
    };
    let Temporal::Atom(Pred::Event(symbol)) = eventual.as_ref() else {
        panic!("event symbol");
    };
    assert!(
        export.alphabet.contains(symbol),
        "B2's T symbol is a member of the B1-derived Σ; B2 derives no second set"
    );
}

// ─── Sanity: the kernel spec helpers build well-formed InductiveSpecs ────────

/// The `temporal_inductive_spec` / `temporal_hoas_inductive_spec` helpers
/// build the constructor set the §3 core pins (`72 §3`): 9 constructors for
/// the first-order family; the HOAS variant swaps `mu`/`nu` for the negative
/// `(Temporal → Temporal)` arg. Guards against an accidental constructor-count
/// drift.
#[test]
fn temporal_spec_constructor_shape() {
    let d = GlobalId(0);
    let spec = temporal_inductive_spec(d);
    assert_eq!(
        spec.constructors.len(),
        9,
        "the §3 core: atom/not/and/or/next/until/mu/nu/var"
    );
    // Non-indexed family: every constructor targets the empty index.
    for c in &spec.constructors {
        assert!(c.target_indices.is_empty(), "Temporal is non-indexed");
    }
    let hoas = temporal_hoas_inductive_spec(d);
    assert_eq!(
        hoas.constructors.len(),
        9,
        "HOAS variant keeps the 9-ctor count"
    );
}
