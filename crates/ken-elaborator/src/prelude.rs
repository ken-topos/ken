//! L3 prelude: collection inductives + Ω connectives/predicates (`37`).
//!
//! Registered at `ElabEnv::empty()` so the L3 combinator / `unfoldUpTo` / `sort`
//! views (declared in conformance tests, driving the recursive-view-through-SCT
//! wiring in `elab.rs`) can reference them globally. All are ordinary kernel
//! declarations — **no new kernel rule** (`14 §5` / `34 §1`): the inductives
//! ride the landed `data` machinery; the Ω constants are postulates with Pi
//! types, used in **applied form** (the surface parser has no `≡` / `∧` tokens,
//! so propositions name these constants by application).
//!
//! `Nat` is the Peano inductive `data Nat = Zero | Suc Nat` (replacing the
//! earlier placeholder postulate) so the fuel-bounded `unfoldUpTo` can
//! pattern-match on its fuel (`37 §5`). This is the L1-numerics precedent
//! applied to Nat: the inductive former is `Term::IndFormer`, so call sites
//! that built `Term::Const(nat_id)` for the placeholder postulate use
//! `Term::indformer(nat_id)` for the inductive.
//!
//! **L-classes staging boundary (`37 §6`, crossed by L3b).** `DecEq` / `Ord`
//! are *named* here (postulated predicates) so the refinement shapes elaborate.
//! L3b wires **user-type instancing + constraint resolution** (`where Ord a`,
//! `where DecEq K`) via the Lc-landed `instance_search` (`classes.rs:91`):
//! the `where` clause on a `view` declaration is checked against
//! `instance_search` before the body elaborates, emitting `NoInstance` on
//! failure. `Map`/`Set` are abstract postulates declared here; the `DecEq`
//! gate is enforced at elaboration time (`37 §6`).

use ken_kernel::{
    declare_def, declare_primitive, env::PrimReduction, subst::weaken, GlobalId, Level, Term,
};

use crate::error::ElabError;
use crate::ElabEnv;

/// A zeroed placeholder `PreludeEnv` for `ElabEnv` construction; overwritten by
/// `register_prelude` before the env is returned. The `GlobalId(0)` values are
/// never observed (no real declaration has id 0).
pub fn empty_prelude_env() -> PreludeEnv {
    let z = GlobalId(0);
    PreludeEnv {
        nat_id: z,
        zero_id: z,
        suc_id: z,
        list_id: z,
        nil_id: z,
        cons_id: z,
        option_id: z,
        none_id: z,
        some_id: z,
        result_id: z,
        err_id: z,
        ok_id: z,
        prod_id: z,
        mkprod_id: z,
        equal_id: z,
        and_id: z,
        issorted_id: z,
        perm_id: z,
        byte_length_id: z,
        char_length_id: z,
        string_to_list_char_id: z,
        list_char_to_string_id: z,
        // VAL1-surface Console/IO declarations (`36 §2.1`, VAL1-surface).
        unit_id: z,
        mkunit_id: z,
        console_op_id: z,
        write_id: z,
        itree_id: z,
        ret_id: z,
        vis_id: z,
        io_id: z,
        print_line_id: z,
        mkdecimalpair_id: z,
        state_op_id: z,
        get_id: z,
        put_id: z,
        sum_id: z,
        inl_id: z,
        inr_id: z,
        resp_state_id: z,
        resp_sum_id: z,
        bind_id: z,
        run_state_id: z,
        get_fn_id: z,
        put_fn_id: z,
    }
}

/// GlobalIds for the L3 prelude types + Ω constants.
#[derive(Debug, Clone)]
pub struct PreludeEnv {
    // `Nat` (Peano) — the `unfoldUpTo` fuel.
    pub nat_id: GlobalId,
    pub zero_id: GlobalId,
    pub suc_id: GlobalId,
    // `List a` — the transparent inductive (`34 §1`).
    pub list_id: GlobalId,
    pub nil_id: GlobalId,
    pub cons_id: GlobalId,
    // `Option a`, `Result e a` — L2 sums, pre-registered so views reference them.
    pub option_id: GlobalId,
    pub none_id: GlobalId,
    pub some_id: GlobalId,
    pub result_id: GlobalId,
    pub err_id: GlobalId,
    pub ok_id: GlobalId,
    // `Prod a b` — the `a × s` product (the `unfoldUpTo` step payload).
    pub prod_id: GlobalId,
    pub mkprod_id: GlobalId,
    // Ω connectives / predicates (postulates, applied form).
    /// `Equal : Π(A:Type). A → A → Ω` — propositional equality (the `≡`).
    pub equal_id: GlobalId,
    /// `And : Ω → Ω → Ω` — conjunction (the `∧`).
    pub and_id: GlobalId,
    /// `isSorted : Π(A:Type). List A → Ω`.
    pub issorted_id: GlobalId,
    /// `Perm : Π(A:Type). List A → List A → Ω`.
    pub perm_id: GlobalId,
    // L3a String surface ops (`37 §2`). `byte_length` / `char_length` return
    // `Int` (the `bytes_length` L6 precedent + numeric-literal default; the
    // spec's `Nat` is the concept, `Int` the buildable-now spelling).
    pub byte_length_id: GlobalId,
    pub char_length_id: GlobalId,
    /// `string_to_list_char : String → List Char` (total, `37 §2.3`).
    pub string_to_list_char_id: GlobalId,
    /// `list_char_to_string : List Char → String` (total, `37 §2.3`).
    pub list_char_to_string_id: GlobalId,
    // VAL1-surface: Console/IO declarations (`36 §2.1`, VAL1-surface).
    /// `Unit` — the one-element type (`data Unit = MkUnit`).
    pub unit_id: GlobalId,
    pub mkunit_id: GlobalId,
    /// `ConsoleOp` — Console effect operations (`Write : String → ConsoleOp`).
    pub console_op_id: GlobalId,
    pub write_id: GlobalId,
    /// `ITree R` — the simplified W-style interaction tree (`36 §2.1`, K1.5).
    /// `Ret : R → ITree R ; Vis : (Nat → ITree R) → ITree R`.
    pub itree_id: GlobalId,
    pub ret_id: GlobalId,
    pub vis_id: GlobalId,
    /// `IO : Type → Type` — Console-effect IO type (postulate; prim reduction
    /// held until `wp/VAL1-console-exec` lands).
    pub io_id: GlobalId,
    /// `print_line : String → IO Unit` — the surface Console print postulate.
    pub print_line_id: GlobalId,
    /// `MkDecimalPair` — the derived `Decimal`'s constructor (`18a §5.6.1`),
    /// surfaced so literal-conversion call sites outside this crate can
    /// build a `Decimal` `EvalVal` from a `(coeff, exp)` pair.
    pub mkdecimalpair_id: GlobalId,
    // `[State s]` direct effect surface (VAL2 #10 / OQ-C·C2, `36 §4.5`) —
    // built directly via `effects::state` (see that module's doc comment).
    /// `StateOp s = Get | Put s` (`36 §2.1`).
    pub state_op_id: GlobalId,
    pub get_id: GlobalId,
    pub put_id: GlobalId,
    /// `Sum a b = InL a | InR b` — the effect-op container coproduct (`⊕`).
    pub sum_id: GlobalId,
    pub inl_id: GlobalId,
    pub inr_id: GlobalId,
    /// `resp_state : (s:Type) -> StateOp s -> Type`.
    pub resp_state_id: GlobalId,
    /// `resp_sum : (s f:Type) -> (RespF:f->Type) -> Sum (StateOp s) f -> Type`.
    pub resp_sum_id: GlobalId,
    /// `bind` over the lifted `ITree`.
    pub bind_id: GlobalId,
    /// `runState` — the `§4.2` `elim_ITree` fold at `F` (`36 §4.5.3`).
    pub run_state_id: GlobalId,
    /// `get : Unit -> ITree (Sum (StateOp s) f) (resp_sum s f RespF) s`.
    pub get_fn_id: GlobalId,
    /// `put : s -> ITree (Sum (StateOp s) f) (resp_sum s f RespF) Unit`.
    pub put_fn_id: GlobalId,
}

/// Register the L3 prelude in `elab` (called from `ElabEnv::empty`).
pub fn register_prelude(elab: &mut ElabEnv) -> Result<PreludeEnv, ElabError> {
    let omega0 = Term::omega(Level::Zero);
    let type0 = Term::ty(Level::Zero);

    // ── Inductives (landed `data` machinery, `34 §1`) ──────────────────────
    // `Nat` is the Peano inductive (replaces the placeholder postulate Nat).
    elab.elaborate_decl("data Nat = Zero | Suc Nat")
        .map_err(|e| ElabError::Internal(format!("prelude Nat failed: {}", e)))?;
    elab.elaborate_decl("data List a = Nil | Cons a (List a)")
        .map_err(|e| ElabError::Internal(format!("prelude List failed: {}", e)))?;
    elab.elaborate_decl("data Option a = None | Some a")
        .map_err(|e| ElabError::Internal(format!("prelude Option failed: {}", e)))?;
    elab.elaborate_decl("data Result e a = Err e | Ok a")
        .map_err(|e| ElabError::Internal(format!("prelude Result failed: {}", e)))?;
    elab.elaborate_decl("data Prod a b = MkProd a b")
        .map_err(|e| ElabError::Internal(format!("prelude Prod failed: {}", e)))?;

    // VAL1-surface inductives — declared before `lookup` closure to avoid
    // conflicting borrows (elaborate_decl needs &mut elab).
    elab.elaborate_decl("data Unit = MkUnit")
        .map_err(|e| ElabError::Internal(format!("prelude Unit failed: {}", e)))?;
    elab.elaborate_decl("data ConsoleOp = Write String")
        .map_err(|e| ElabError::Internal(format!("prelude ConsoleOp failed: {}", e)))?;

    // `ITree (E:Type) (Resp:E->Type) (R:Type)` — the LIFTED, effect-generic
    // interaction tree (State-effect-build, VAL2 #10 / OQ-C·C2, `36 §4.5.6`).
    // Dependent-response `Vis` (`Resp op`, non-`Unit` for `State.Get`) can't be
    // expressed by the surface `data` parser (no ctor arg may depend on an
    // earlier arg's VALUE) — built directly via `declare_inductive`
    // (`effects::state::declare_itree`), a real kernel inductive re-checked
    // exactly like any surface `data` (AC1: `ken-kernel` untouched, no new
    // `Term`/`Decl` variant — `crate::effects::state`'s module doc explains
    // why hand-building here is SAFER than reopening `compile_match_matrix`).
    // Replaces the earlier Console-hardwired 1-param `ITree r = Ret r | Vis
    // ConsoleOp (Unit -> ITree r)` — Console's own `print_line` prim
    // reduction is untyped/erased at eval time (its STATIC type is the
    // separate `IO Unit` postulate below, unaffected by this arity change)
    // and only needs its `ConsoleIds.params_len` bumped at call sites.
    let (itree_id, ret_id, vis_id) =
        crate::effects::state::declare_itree(&mut elab.env).map_err(ElabError::Internal)?;
    elab.globals.insert("ITree".to_string(), itree_id);
    elab.globals.insert("Ret".to_string(), ret_id);
    elab.globals.insert("Vis".to_string(), vis_id);

    let lookup = |name: &str| -> Result<GlobalId, ElabError> {
        elab.globals
            .get(name)
            .copied()
            .ok_or_else(|| ElabError::Internal(format!("prelude: '{}' not registered", name)))
    };

    let nat_id = lookup("Nat")?;
    let zero_id = lookup("Zero")?;
    let suc_id = lookup("Suc")?;
    let list_id = lookup("List")?;
    let nil_id = lookup("Nil")?;
    let cons_id = lookup("Cons")?;
    let option_id = lookup("Option")?;
    let none_id = lookup("None")?;
    let some_id = lookup("Some")?;
    let result_id = lookup("Result")?;
    let err_id = lookup("Err")?;
    let ok_id = lookup("Ok")?;
    let prod_id = lookup("Prod")?;
    let mkprod_id = lookup("MkProd")?;
    // VAL1-surface inductives (declared before lookup closure above).
    let unit_id = lookup("Unit")?;
    let mkunit_id = lookup("MkUnit")?;
    let console_op_id = lookup("ConsoleOp")?;
    let write_id = lookup("Write")?;
    let itree_id = lookup("ITree")?;
    let ret_id = lookup("Ret")?;
    let vis_id = lookup("Vis")?;
    // `lookup` is last used above; NLL ends its borrow here.

    // ── `[State s]` direct effect surface (VAL2 #10 / OQ-C·C2, `36 §4.5`) ──
    // Every definition below is a real kernel `Decl::Inductive`/`Decl::Def`
    // (`declare_inductive`/`declare_def`, kernel-rechecked) — never a
    // `declare_primitive`/`declare_postulate` — hand-built in
    // `effects::state` for the same reason `ITree` itself is (dependent
    // ctor-arg/motive shapes the surface `data`/`match` machinery can't
    // express yet). See that module's doc comment for the full rationale.
    use crate::effects::state as state_eff;
    let (state_op_id, get_id, put_id) =
        state_eff::declare_state_op(&mut elab.env).map_err(ElabError::Internal)?;
    elab.globals.insert("StateOp".to_string(), state_op_id);
    elab.globals.insert("Get".to_string(), get_id);
    elab.globals.insert("Put".to_string(), put_id);

    let (sum_id, inl_id, inr_id) = state_eff::declare_sum(&mut elab.env).map_err(ElabError::Internal)?;
    elab.globals.insert("Sum".to_string(), sum_id);
    elab.globals.insert("InL".to_string(), inl_id);
    elab.globals.insert("InR".to_string(), inr_id);

    let resp_state_id = state_eff::declare_resp_state(&mut elab.env, state_op_id, unit_id)
        .map_err(ElabError::Internal)?;
    elab.globals.insert("resp_state".to_string(), resp_state_id);

    let resp_sum_id = state_eff::declare_resp_sum(&mut elab.env, state_op_id, sum_id, resp_state_id)
        .map_err(ElabError::Internal)?;
    elab.globals.insert("resp_sum".to_string(), resp_sum_id);

    let bind_id = state_eff::declare_bind(&mut elab.env, itree_id, vis_id).map_err(ElabError::Internal)?;
    elab.globals.insert("bind".to_string(), bind_id);

    let run_state_id = state_eff::declare_run_state(
        &mut elab.env,
        itree_id,
        ret_id,
        vis_id,
        state_op_id,
        get_id,
        put_id,
        sum_id,
        inl_id,
        inr_id,
        resp_state_id,
        resp_sum_id,
        unit_id,
        mkunit_id,
    )
    .map_err(ElabError::Internal)?;
    elab.globals.insert("runState".to_string(), run_state_id);

    let get_fn_id = state_eff::declare_get(
        &mut elab.env, itree_id, ret_id, vis_id, state_op_id, get_id, sum_id, inl_id, resp_sum_id, unit_id,
    )
    .map_err(ElabError::Internal)?;
    elab.globals.insert("get".to_string(), get_fn_id);

    let put_fn_id = state_eff::declare_put(
        &mut elab.env, itree_id, ret_id, vis_id, state_op_id, put_id, sum_id, inl_id, resp_sum_id, unit_id, mkunit_id,
    )
    .map_err(ElabError::Internal)?;
    elab.globals.insert("put".to_string(), put_fn_id);

    // ── Ω constants (ES2: real definitions, demoted out of `trusted_base()`) ─
    // `Equal : Π(A:Type). Π(x:A). Π(y:A). Ω`  (the `≡`).
    // de Bruijn: Pi(Type, Pi(Var 0, Pi(Var 1, Ω₀)))  — A=Var0, x=Var0, y=Var1
    // under their binders.
    //
    // ES2: DELETE the postulate — reference the kernel's native computing
    // `Eq A t u : Ω` (`16 §2`, `term.rs::Term::Eq`) instead of assuming an
    // opaque axiom. `Equal` becomes a transparent alias
    // `λA.λx.λy. Eq A x y`, re-checked and out of `trusted_base()`, so it
    // keeps `Eq`'s computation (`refl`/`J`) rather than forfeiting it.
    let equal_ty = Term::pi(
        type0.clone(),
        Term::pi(Term::var(0), Term::pi(Term::var(1), omega0.clone())),
    );
    let equal_body = Term::lam(
        type0.clone(),
        Term::lam(
            Term::var(0),
            Term::lam(
                Term::var(1),
                Term::Eq(
                    Box::new(Term::var(2)),
                    Box::new(Term::var(1)),
                    Box::new(Term::var(0)),
                ),
            ),
        ),
    );
    let equal_id = declare_def(&mut elab.env, vec![], equal_ty, equal_body)
        .map_err(|e| ElabError::Internal(format!("prelude Equal failed: {}", e)))?;
    elab.globals.insert("Equal".to_string(), equal_id);

    // `And : Ω → Ω → Ω`  (the `∧`).
    //
    // ES2: derived Ω-connective (`16 §1.3`) — `And A B := Σ(_:A).B`, which
    // `sort_sigma` (`check.rs`) classifies at `Ω` precisely because BOTH
    // components are `Ω` (the both-Ω-keyed conjunction case; a relevant
    // first component would stay in `Type`, per the same rule).
    let and_ty = Term::pi(omega0.clone(), Term::pi(omega0.clone(), omega0.clone()));
    let and_body = Term::lam(
        omega0.clone(),
        Term::lam(omega0.clone(), Term::sigma(Term::var(1), weaken(&Term::var(0), 1))),
    );
    let and_id = declare_def(&mut elab.env, vec![], and_ty, and_body)
        .map_err(|e| ElabError::Internal(format!("prelude And failed: {}", e)))?;
    elab.globals.insert("And".to_string(), and_id);

    // `Prop` — a surface-nameable alias for `Ω₀`, so a recursive `view` can
    // carry an explicit return-type ANNOTATION landing on `Ω` (required: a
    // self-recursive declaration needs a type annotation, `declare_def`
    // can't infer one — but the surface parser has no `Ω` TOKEN at all).
    // `Prop`'s own kernel type is `Type (suc 0)` (the `Ω-Form` rule,
    // `Omega(l) : Type (suc l)`); its BODY is `Term::Omega(Zero)` literally,
    // so kernel `check`/`convert` sees `Prop ≡ Ω₀` by ordinary δ-unfolding —
    // used only as a type-position spelling, never as a value.
    let prop_id = declare_def(&mut elab.env, vec![], Term::ty(Level::Zero.suc()), omega0.clone())
        .map_err(|e| ElabError::Internal(format!("prelude Prop failed: {}", e)))?;
    elab.globals.insert("Prop".to_string(), prop_id);

    // `tt : Top` — the kernel's prelude `Top`-introduction constant (K5,
    // `16 §1.3`), already unconditionally declared by `GlobalEnv::new()`.
    // Surfaced here so a structure-class law proof can close a goal that
    // whnf's to `Top` (K7: an operation-wrapped `IsTrue`/`Equal` goal, the
    // same slot `Refl` occupied before the operand-whnf completeness fix,
    // `51 §6`). Ordinary global reference — `tt` infers to `Top`, so it
    // needs no dedicated check-mode arm; the `check()` fallback (infer,
    // then unify against `expected`) handles it like any other constant.
    elab.globals.insert("tt".to_string(), elab.env.tt_id());

    // `Pair : Type -> Type -> Type` — the non-dependent Sigma pair,
    // `k × v` (`52-map.md §4`, `13 §3`): "the Σ-pair... distinct from the
    // inductive `Prod`". Built exactly like `And`'s `Σ(_:A).B` above — the
    // kernel Sigma/Pair/Proj1/Proj2 formers are already landed
    // (`ken-kernel/src/term.rs`); only the concrete `×` INFIX surface
    // spelling is missing, which `52-map.md §4`'s own hedge tags `(oracle)`
    // ("any still-open surface-syntax token is tagged `(oracle)`") — the
    // mechanism (Sigma) is landed, so this is a named-application spelling
    // of it, not a new kernel feature or a workaround. `mkPair`/`pairFst`/
    // `pairSnd` are the corresponding intro/elim helpers (no surface `.1`/
    // `.2` projection exists for a bare Sigma — that syntax is reserved for
    // class-dictionary records only, `elab.rs::infer_proj`).
    let pair_ty_ty = Term::pi(type0.clone(), Term::pi(type0.clone(), type0.clone()));
    let pair_ty_body = Term::lam(
        type0.clone(),
        Term::lam(type0.clone(), Term::sigma(Term::var(1), weaken(&Term::var(0), 1))),
    );
    let pair_ty_id = declare_def(&mut elab.env, vec![], pair_ty_ty, pair_ty_body)
        .map_err(|e| ElabError::Internal(format!("prelude Pair failed: {}", e)))?;
    elab.globals.insert("Pair".to_string(), pair_ty_id);

    // `mkPair : (a:Type) -> (b:Type) -> a -> b -> Pair a b`.
    let pair_app_at_len2 =
        Term::app(Term::app(Term::const_(pair_ty_id, vec![]), Term::var(1)), Term::var(0));
    let pair_app_at_len4 =
        Term::app(Term::app(Term::const_(pair_ty_id, vec![]), Term::var(3)), Term::var(2));
    let mkpair_ty = Term::pi(
        type0.clone(),
        Term::pi(
            type0.clone(),
            Term::pi(Term::var(1), Term::pi(Term::var(1), pair_app_at_len4)),
        ),
    );
    let mkpair_body = Term::lam(
        type0.clone(),
        Term::lam(
            type0.clone(),
            Term::lam(Term::var(1), Term::lam(Term::var(1), Term::pair(Term::var(1), Term::var(0)))),
        ),
    );
    let mkpair_id = declare_def(&mut elab.env, vec![], mkpair_ty, mkpair_body)
        .map_err(|e| ElabError::Internal(format!("prelude mkPair failed: {}", e)))?;
    elab.globals.insert("mkPair".to_string(), mkpair_id);

    // `pairFst : (a:Type) -> (b:Type) -> Pair a b -> a`.
    let fst_ty = Term::pi(
        type0.clone(),
        Term::pi(type0.clone(), Term::pi(pair_app_at_len2.clone(), Term::var(2))),
    );
    let fst_body = Term::lam(
        type0.clone(),
        Term::lam(type0.clone(), Term::lam(pair_app_at_len2.clone(), Term::proj1(Term::var(0)))),
    );
    let fst_id = declare_def(&mut elab.env, vec![], fst_ty, fst_body)
        .map_err(|e| ElabError::Internal(format!("prelude pairFst failed: {}", e)))?;
    elab.globals.insert("pairFst".to_string(), fst_id);

    // `pairSnd : (a:Type) -> (b:Type) -> Pair a b -> b`.
    let snd_ty = Term::pi(
        type0.clone(),
        Term::pi(type0.clone(), Term::pi(pair_app_at_len2.clone(), Term::var(1))),
    );
    let snd_body = Term::lam(
        type0.clone(),
        Term::lam(type0.clone(), Term::lam(pair_app_at_len2, Term::proj2(Term::var(0)))),
    );
    let snd_id = declare_def(&mut elab.env, vec![], snd_ty, snd_body)
        .map_err(|e| ElabError::Internal(format!("prelude pairSnd failed: {}", e)))?;
    elab.globals.insert("pairSnd".to_string(), snd_id);

    // `Decimal`/`Char` DEMOTE→derived (`18a §5.6`/`§5.9`, Phase-2 tranche #2).
    // Must run here: after `Equal`/`And`/`Prop`/`tt` (needed by `IsTrue`),
    // before the L3a String-ops registration below (needs `Char` in
    // `elab.globals`).
    let decimal_char_env = crate::decimal_char::register_decimal_char(elab)
        .map_err(|e| ElabError::Internal(format!("Decimal/Char demote failed: {}", e)))?;

    // `IntN<->Int` conversion floor + `checked_*`/`saturating_*` DEMOTE
    // (`18a §5.7`, Phase-2 tranche #4). Needs the 8 `IntN`/`UIntN` type ids
    // (from `register_numeric_env`, already run) and `Option`/`Some`/`None`
    // (declared above, before this function's `lookup` closure).
    crate::conversions::register_conversions(elab)
        .map_err(|e| ElabError::Internal(format!("IntN<->Int conversions failed: {}", e)))?;

    // `isSorted : Π(a:Type). (a → a → Bool) → List a → Ω` (ES2-remainder,
    // `37 §6`: the explicit-comparator form, `Ord`-class deferred).
    //
    // `isSorted leq Nil = ⊤`, `isSorted leq (x::Nil) = ⊤`,
    // `isSorted leq (x::y::r) = IsTrue (leq x y) ∧ isSorted leq (y::r)`, with
    // `IsTrue (leq x y) := Eq Bool (leq x y) True` (the `Equal`/kernel-`Eq`
    // alias already landed by ES2) and `∧` the already-demoted `And`. `⊤` is
    // spelled `Equal Bool True True` (reflexively true, no dedicated Ω-truth
    // constant exists in this prelude).
    let list_a = |a: Term| Term::app(Term::indformer(list_id, vec![]), a);
    let _ = &list_a; // still used below for Perm's raw-term construction
    elab.elaborate_decl(
        "view isSorted (a : Type) (leq : a -> a -> Bool) (xs : List a) : Prop = \
         match xs { \
           Nil => Equal Bool True True ; \
           Cons x xs2 => match xs2 { \
             Nil => Equal Bool True True ; \
             Cons y r => And (Equal Bool (leq x y) True) (isSorted a leq xs2) \
           } \
         }",
    )
    .map_err(|e| ElabError::Internal(format!("prelude isSorted failed: {}", e)))?;
    let issorted_id = elab
        .globals
        .get("isSorted")
        .copied()
        .ok_or_else(|| ElabError::Internal("prelude: 'isSorted' not registered".into()))?;

    // `Perm : Π(A:Type). List A → List A → Ω`  (comparator-free, `37 §6`
    // ES2-remainder ruling `evt_3cn9v6em54yej`, closing ES1's "spec picks
    // one" fork in favor of truncation over count-equality — count-equality
    // needs `DecEq a` to `count`, which this ruling explicitly defers).
    //
    // `Perm xs ys := ‖Perm_rel xs ys‖` — `Perm_rel` (`refl`/`swap`/`trans`/
    // `cons`) is proof-RELEVANT (a proof records *which* permutation), so it
    // lands in `Type`, not `Ω` directly (`16 §1.3` forbids a proof-relevant
    // `Type → Ω`, the relevance leak that would admit `Bool` and collapse
    // `true ≡ false`); propositional truncation (`Term::Trunc`) is the
    // `Ω`-safe bridge. `Perm_rel` is a genuinely INDEXED family — the
    // surface `data` declaration machinery (`data.rs::elab_data_decl`)
    // always builds NON-indexed families (indices/target_indices hardcoded
    // to `vec![]`, the AC5-deferred limitation noted in `l2_acceptance.rs`)
    // — so `Perm_rel` is built directly against the kernel's own
    // `declare_inductive`/`InductiveSpec` (which DOES support indices; only
    // the elaborator's surface convenience wrapper doesn't expose them),
    // exactly the technique `data.rs` itself uses internally, one level
    // lower. `Perm_rel`'s constructors are pure internal plumbing (never
    // pattern-matched by user code in this WP's scope) so they are not
    // registered in `elab.globals`.
    let perm_ty = Term::pi(
        type0.clone(),
        Term::pi(
            list_a(Term::var(0)),
            Term::pi(list_a(Term::var(1)), omega0.clone()),
        ),
    );
    let cons_app = |a: Term, h: Term, t: Term| -> Term {
        Term::app(
            Term::app(Term::app(Term::Constructor { id: cons_id, level_args: vec![] }, a), h),
            t,
        )
    };
    let perm_rel_id = ken_kernel::declare_inductive(&mut elab.env, |perm_rel_id| {
        let perm_rel_at = |a: Term, xs: Term, ys: Term| -> Term {
            Term::app(
                Term::app(Term::app(Term::indformer(perm_rel_id, vec![]), a), xs),
                ys,
            )
        };
        ken_kernel::InductiveSpec {
            level_params: vec![],
            // `Δ_p = [a : Type 0]`.
            params: vec![type0.clone()],
            // `Δ_i = [List a, List a]` — indices telescope like args: the
            // first is in context `[a]` (`a` = Var(0)); the second is in
            // context `[a, idx0]`, so `a` has shifted to Var(1).
            indices: vec![list_a(Term::var(0)), list_a(Term::var(1))],
            level: Level::Zero,
            constructors: vec![
                // `perm_refl : (xs:List a) -> Perm_rel a xs xs`.
                ken_kernel::CtorSpec {
                    args: vec![list_a(Term::var(0))],
                    // ctx [a, xs]: xs = Var(0).
                    target_indices: vec![Term::var(0), Term::var(0)],
                },
                // `perm_swap : (x y:a)(r:List a) ->
                //    Perm_rel a (x::y::r) (y::x::r)`.
                ken_kernel::CtorSpec {
                    args: vec![
                        Term::var(0),           // x:a,        ctx [a]
                        Term::var(1),           // y:a,        ctx [a,x]      (a=Var1)
                        list_a(Term::var(2)),   // r:List a,   ctx [a,x,y]    (a=Var2)
                    ],
                    // ctx [a,x,y,r]: a=Var3, x=Var2, y=Var1, r=Var0.
                    target_indices: vec![
                        cons_app(Term::var(3), Term::var(2), cons_app(Term::var(3), Term::var(1), Term::var(0))),
                        cons_app(Term::var(3), Term::var(1), cons_app(Term::var(3), Term::var(2), Term::var(0))),
                    ],
                },
                // `perm_trans : (xs ys zs:List a) ->
                //    Perm_rel a xs ys -> Perm_rel a ys zs -> Perm_rel a xs zs`.
                ken_kernel::CtorSpec {
                    args: vec![
                        list_a(Term::var(0)), // xs, ctx [a]
                        list_a(Term::var(1)), // ys, ctx [a,xs]        (a=Var1)
                        list_a(Term::var(2)), // zs, ctx [a,xs,ys]     (a=Var2)
                        // p1 : Perm_rel a xs ys, ctx [a,xs,ys,zs]: a=Var3,xs=Var2,ys=Var1.
                        perm_rel_at(Term::var(3), Term::var(2), Term::var(1)),
                        // p2 : Perm_rel a ys zs, ctx [a,xs,ys,zs,p1]: a=Var4,ys=Var2,zs=Var1.
                        perm_rel_at(Term::var(4), Term::var(2), Term::var(1)),
                    ],
                    // ctx [a,xs,ys,zs,p1,p2]: a=Var5,xs=Var4,ys=Var3,zs=Var2.
                    target_indices: vec![Term::var(4), Term::var(2)],
                },
                // `perm_cons : (x:a)(xs ys:List a) ->
                //    Perm_rel a xs ys -> Perm_rel a (x::xs) (x::ys)`.
                ken_kernel::CtorSpec {
                    args: vec![
                        Term::var(0),         // x:a, ctx [a]
                        list_a(Term::var(1)), // xs, ctx [a,x]        (a=Var1)
                        list_a(Term::var(2)), // ys, ctx [a,x,xs]     (a=Var2)
                        // p : Perm_rel a xs ys, ctx [a,x,xs,ys]: a=Var3,xs=Var1,ys=Var0.
                        perm_rel_at(Term::var(3), Term::var(1), Term::var(0)),
                    ],
                    // ctx [a,x,xs,ys,p]: a=Var4,x=Var3,xs=Var2,ys=Var1.
                    target_indices: vec![
                        cons_app(Term::var(4), Term::var(3), Term::var(2)),
                        cons_app(Term::var(4), Term::var(3), Term::var(1)),
                    ],
                },
            ],
        }
    })
    .map_err(|e| ElabError::Internal(format!("prelude Perm_rel failed: {}", e)))?;

    let perm_body = Term::lam(
        type0.clone(),
        Term::lam(
            list_a(Term::var(0)),
            Term::lam(
                list_a(Term::var(1)),
                Term::Trunc(Box::new(Term::app(
                    Term::app(
                        Term::app(Term::indformer(perm_rel_id, vec![]), Term::var(2)),
                        Term::var(1),
                    ),
                    Term::var(0),
                ))),
            ),
        ),
    );
    let perm_id = declare_def(&mut elab.env, vec![], perm_ty, perm_body)
        .map_err(|e| ElabError::Internal(format!("prelude Perm failed: {}", e)))?;
    elab.globals.insert("Perm".to_string(), perm_id);

    // `Map`/`Set` (`37 §6`) were RETIRED here (ES2's audited
    // `declare_primitive` OpaqueType re-class) — Map-build (`52-map.md`,
    // VAL2 #8 / OQ-A) supersedes that placeholder with a **proved, pure**
    // ordered BST (`packages/collections/map.ken`'s `Tree k v` + `insert`/
    // `lookup`/`member`/`toList`/`fromList`/`fold`/`Set*`), a transparent
    // inductive admitted via `declare_inductive`/`declare_def` — kernel-
    // rechecked, not audited-opaque. Net-negative `trusted_base()` delta
    // (`52 §1.1`/`§9` AC1): the two entries below are GONE, nothing new is
    // added here (the map's carrier/ops/laws are package Ken, not prelude
    // primitives). `Map`/`Set` are no longer prelude-global names; a
    // program spells the carrier `Tree k v` (`52 §3`), matching the spec's
    // own naming — there are not two `Map`s (`52 §1.1`).

    // ── L3a String surface ops (`37 §2`) ───────────────────────────────────
    // `String` (bytes layer) + `Int` (numeric tower) + `Char` (numeric) +
    // `List` (prelude) are all in globals now.
    let string_id = elab
        .globals
        .get("String")
        .copied()
        .ok_or_else(|| ElabError::Internal("prelude: String not registered".into()))?;
    let int_id = elab
        .globals
        .get("Int")
        .copied()
        .ok_or_else(|| ElabError::Internal("prelude: Int not registered".into()))?;
    let char_id = elab
        .globals
        .get("Char")
        .copied()
        .ok_or_else(|| ElabError::Internal("prelude: Char not registered".into()))?;

    let string_t = Term::const_(string_id, vec![]);
    let int_t = Term::const_(int_id, vec![]);
    let char_t = Term::const_(char_id, vec![]);
    let list_char_t = Term::app(Term::indformer(list_id, vec![]), char_t);

    // `declare_primitive` helper: register a prim op + bind it in globals.
    let mut reg_prim =
        |name: &'static str, ty: Term, symbol: &'static str| -> Result<GlobalId, ElabError> {
            let id = declare_primitive(&mut elab.env, vec![], ty, PrimReduction::Op { symbol })
                .map_err(|e| ElabError::Internal(format!("prim {} failed: {}", name, e)))?;
            elab.globals.insert(name.to_string(), id);
            Ok(id)
        };

    // `byte_length : String → Int` — the stored NFC UTF-8 byte count (`37 §2.2`).
    let byte_length_id = reg_prim(
        "byte_length",
        Term::pi(string_t.clone(), int_t.clone()),
        "byte_length",
    )?;
    // `char_length : String → Int` — the Unicode scalar-value count (`37 §2.2`).
    let char_length_id = reg_prim(
        "char_length",
        Term::pi(string_t.clone(), int_t.clone()),
        "char_length",
    )?;
    // `string_to_list_char : String → List Char` (total, `37 §2.3`).
    let string_to_list_char_id = reg_prim(
        "string_to_list_char",
        Term::pi(string_t.clone(), list_char_t.clone()),
        "string_to_list_char",
    )?;
    // `list_char_to_string : List Char → String` (total, `37 §2.3`).
    let list_char_to_string_id = reg_prim(
        "list_char_to_string",
        Term::pi(list_char_t.clone(), string_t.clone()),
        "list_char_to_string",
    )?;

    // Drop reg_prim before borrowing elab for IO/print_line declarations.
    // NLL cannot end the reg_prim borrow while print_line = reg_prim(...) appears
    // after the IO declaration; an explicit drop lets IO borrow elab cleanly.
    drop(reg_prim);

    // `IO : Type → Type` — the Console-effect IO type.
    //
    // ES2: DERIVABLE — `IO` is a definition, not an assumed axiom: it is
    // `ITree` (`36`) specialized to the Console effect. `IO A := ITree
    // ConsoleOp (\_:ConsoleOp. Unit) A` — Console's every op has a `Unit`
    // response (matches the pre-lift hardwired `Unit -> ITree r` shape),
    // now expressed as a CONSTANT `Resp` function over the lifted, 3-param
    // `ITree` (State-effect-build) rather than baked into the family itself.
    let io_ty = Term::pi(type0.clone(), type0.clone());
    let const_unit_resp = Term::lam(Term::indformer(console_op_id, vec![]), Term::indformer(unit_id, vec![]));
    let io_body = Term::lam(
        type0.clone(),
        crate::effects::state::itree3_standalone(
            itree_id,
            Term::indformer(console_op_id, vec![]),
            weaken(&const_unit_resp, 1),
            Term::var(0),
        ),
    );
    let io_id = declare_def(&mut elab.env, vec![], io_ty, io_body)
        .map_err(|e| ElabError::Internal(format!("prelude IO failed: {}", e)))?;
    elab.globals.insert("IO".to_string(), io_id);

    // `print_line : String → IO Unit` — Console print, per the VAL1-step2
    // wiring (`6789e42`).
    //
    // `console_resp : ConsoleOp -> Type = \_. Unit` — every Console op has a
    // `Unit` response; NAMED (not an inline argument-position lambda) so
    // `Vis`/`Ret`'s `Resp` param is always a plain `Const` reference,
    // matching the shape the ORIGINAL 1-param `print_line` body already used
    // successfully (a single continuation lambda, nothing else).
    elab.elaborate_decl("view console_resp (op : ConsoleOp) : Type = Unit")
        .map_err(|e| ElabError::Internal(format!("prelude console_resp failed: {}", e)))?;

    // ES2: DERIVABLE — a definition in terms of the `ITree` constructors
    // (`Vis`/`Ret`) + the `Console.Op` payload (`Write`), not an irreducible
    // primitive: `print_line s := Vis (Write s) (\_. Ret MkUnit)`. Declared
    // via surface syntax (not a hand-built raw `Term`) so ordinary δ/ι
    // reduction produces the `Vis` node directly — no bespoke `apply`
    // interception needed. `ITree`'s 3 explicit params (State-effect-build
    // lift) are supplied explicitly: `E=ConsoleOp`, `Resp=console_resp`, `R=Unit`.
    elab.elaborate_decl(
        "view print_line (s : String) : IO Unit = \
         Vis ConsoleOp console_resp Unit (Write s) (\\_. Ret ConsoleOp console_resp Unit MkUnit)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude print_line failed: {}", e)))?;
    let print_line_id = elab
        .globals
        .get("print_line")
        .copied()
        .ok_or_else(|| ElabError::Internal("prelude: 'print_line' not registered".into()))?;

    Ok(PreludeEnv {
        nat_id,
        zero_id,
        suc_id,
        list_id,
        nil_id,
        cons_id,
        option_id,
        none_id,
        some_id,
        result_id,
        err_id,
        ok_id,
        prod_id,
        mkprod_id,
        equal_id,
        and_id,
        issorted_id,
        perm_id,
        byte_length_id,
        char_length_id,
        string_to_list_char_id,
        list_char_to_string_id,
        unit_id,
        mkunit_id,
        console_op_id,
        write_id,
        itree_id,
        ret_id,
        vis_id,
        io_id,
        print_line_id,
        mkdecimalpair_id: decimal_char_env.mkdecimalpair_id,
        state_op_id,
        get_id,
        put_id,
        sum_id,
        inl_id,
        inr_id,
        resp_state_id,
        resp_sum_id,
        bind_id,
        run_state_id,
        get_fn_id,
        put_fn_id,
    })
}
