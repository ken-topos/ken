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
    declare_def, declare_inductive, declare_primitive, env::PrimReduction, subst::weaken, CtorSpec,
    GlobalId, InductiveSpec, Level, Term,
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
        stream_id: z,
        stdin_id: z,
        stdout_id: z,
        stderr_id: z,
        console_op_id: z,
        read_id: z,
        write_id: z,
        flush_id: z,
        is_terminal_id: z,
        read_result_id: z,
        chunk_id: z,
        eof_id: z,
        itree_id: z,
        ret_id: z,
        vis_id: z,
        io_id: z,
        print_line_id: z,
        mkdecimalpair_id: z,
        state_op_id: z,
        get_id: z,
        put_id: z,
        coproduct_id: z,
        inl_id: z,
        inr_id: z,
        resp_state_id: z,
        resp_coproduct_id: z,
        bind_id: z,
        run_state_id: z,
        get_fn_id: z,
        put_fn_id: z,
        inject_l_id: z,
        inject_r_id: z,
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
    /// `is_sorted : Π(A:Type). List A → Ω`.
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
    /// `Stream` — the process's three ambient console streams.
    pub stream_id: GlobalId,
    pub stdin_id: GlobalId,
    pub stdout_id: GlobalId,
    pub stderr_id: GlobalId,
    /// `ConsoleOp` — the stream-indexed Console effect algebra.
    pub console_op_id: GlobalId,
    pub read_id: GlobalId,
    pub write_id: GlobalId,
    pub flush_id: GlobalId,
    pub is_terminal_id: GlobalId,
    /// Total response carrier for Console reads.
    pub read_result_id: GlobalId,
    pub chunk_id: GlobalId,
    pub eof_id: GlobalId,
    /// `ITree R` — the simplified W-style interaction tree (`36 §2.1`, K1.5).
    /// `Ret : R → ITree R ; Vis : (Nat → ITree R) → ITree R`.
    pub itree_id: GlobalId,
    pub ret_id: GlobalId,
    pub vis_id: GlobalId,
    /// `IO : Type → Type` — ordinary Console-effect ITree specialization.
    pub io_id: GlobalId,
    /// `print_line : String → IO Unit` — ordinary surface compatibility helper.
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
    /// `Coproduct a b = InL a | InR b` — the effect-op container coproduct (`⊕`).
    pub coproduct_id: GlobalId,
    pub inl_id: GlobalId,
    pub inr_id: GlobalId,
    /// `resp_state : (s:Type) -> StateOp s -> Type`.
    pub resp_state_id: GlobalId,
    /// `resp_coproduct : (s f:Type) -> (RespF:f->Type) -> Coproduct (StateOp s) f -> Type`.
    pub resp_coproduct_id: GlobalId,
    /// `bind` over the lifted `ITree`.
    pub bind_id: GlobalId,
    /// `run_state` — the `§4.2` `elim_ITree` fold at `F` (`36 §4.5.3`).
    pub run_state_id: GlobalId,
    /// `get : Unit -> ITree (Coproduct (StateOp s) f) (resp_coproduct s f RespF) s`.
    pub get_fn_id: GlobalId,
    /// `put : s -> ITree (Coproduct (StateOp s) f) (resp_coproduct s f RespF) Unit`.
    pub put_fn_id: GlobalId,
    /// `inject_l : (g h:Type)(rg:g->Type)(rh:h->Type)(a:Type) -> ITree g rg a
    ///   -> ITree (Coproduct g h) (resp_coproduct g h rg rh) a` (`effect-composition` D2).
    pub inject_l_id: GlobalId,
    /// `inject_r` — the mirror inclusion, `h ↪ Coproduct g h`.
    pub inject_r_id: GlobalId,
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
    elab.elaborate_decl("data Utf8Error = InvalidUtf8")
        .map_err(|e| ElabError::Internal(format!("prelude Utf8Error failed: {}", e)))?;
    elab.elaborate_decl("data Prod a b = MkProd a b")
        .map_err(|e| ElabError::Internal(format!("prelude Prod failed: {}", e)))?;

    // VAL1-surface inductives — declared before `lookup` closure to avoid
    // conflicting borrows (elaborate_decl needs &mut elab).
    elab.elaborate_decl("data Unit = MkUnit")
        .map_err(|e| ElabError::Internal(format!("prelude Unit failed: {}", e)))?;
    elab.elaborate_decl("data Stream = Stdin | Stdout | Stderr")
        .map_err(|e| ElabError::Internal(format!("prelude Stream failed: {}", e)))?;
    elab.elaborate_decl(
        "data ConsoleOp = Read Stream Int | Write Stream Bytes | Flush Stream | IsTerminal Stream",
    )
    .map_err(|e| ElabError::Internal(format!("prelude ConsoleOp failed: {}", e)))?;
    elab.elaborate_decl("data ReadResult = Chunk Bytes | Eof")
        .map_err(|e| ElabError::Internal(format!("prelude ReadResult failed: {}", e)))?;
    elab.elaborate_decl(
        "data IOError = NotFound | PermissionDenied | CapabilityDenied | BrokenPipe | Interrupted | AlreadyExists | InvalidInput | IsDirectory | NotDirectory | NotEmpty | Unsupported | Other Int",
    )
    .map_err(|e| ElabError::Internal(format!("prelude IOError failed: {}", e)))?;
    elab.elaborate_decl(
        "data FileOperation = OpReadFile | OpWriteFile | OpAppendFile | OpMetadata | OpReadDirectory | OpCreateDirectory | OpRemoveFile | OpRemoveDirectory | OpRename",
    )
    .map_err(|e| ElabError::Internal(format!("prelude FileOperation failed: {}", e)))?;
    elab.elaborate_decl("data FileError = MkFileError FileOperation (Option Bytes) IOError")
        .map_err(|e| ElabError::Internal(format!("prelude FileError failed: {}", e)))?;
    elab.elaborate_decl("data FileKind = KFile | KDirectory | KSymlink | KOther")
        .map_err(|e| ElabError::Internal(format!("prelude FileKind failed: {}", e)))?;
    elab.elaborate_decl("data FileMetadata = MkFileMetadata Int FileKind")
        .map_err(|e| ElabError::Internal(format!("prelude FileMetadata failed: {}", e)))?;
    elab.elaborate_decl("data DirEntry = MkDirEntry Bytes FileKind")
        .map_err(|e| ElabError::Internal(format!("prelude DirEntry failed: {}", e)))?;
    elab.elaborate_decl("data CreatePolicy = CreateNew | CreateOrTruncate | CreateOrKeep")
        .map_err(|e| ElabError::Internal(format!("prelude CreatePolicy failed: {}", e)))?;

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
    // ConsoleOp (Unit -> ITree r)` — Console now uses the general encoding
    // below, and the host driver handles the resulting algebra directly.
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
    let stream_id = lookup("Stream")?;
    let stdin_id = lookup("Stdin")?;
    let stdout_id = lookup("Stdout")?;
    let stderr_id = lookup("Stderr")?;
    let console_op_id = lookup("ConsoleOp")?;
    let read_id = lookup("Read")?;
    let write_id = lookup("Write")?;
    let flush_id = lookup("Flush")?;
    let is_terminal_id = lookup("IsTerminal")?;
    let read_result_id = lookup("ReadResult")?;
    let chunk_id = lookup("Chunk")?;
    let eof_id = lookup("Eof")?;
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

    let (coproduct_id, inl_id, inr_id) =
        state_eff::declare_coproduct(&mut elab.env).map_err(ElabError::Internal)?;
    elab.globals.insert("Coproduct".to_string(), coproduct_id);
    elab.globals.insert("InL".to_string(), inl_id);
    elab.globals.insert("InR".to_string(), inr_id);

    let resp_state_id = state_eff::declare_resp_state(&mut elab.env, state_op_id, unit_id)
        .map_err(ElabError::Internal)?;
    elab.globals.insert("resp_state".to_string(), resp_state_id);

    let resp_coproduct_id = state_eff::declare_resp_coproduct(&mut elab.env, coproduct_id)
        .map_err(ElabError::Internal)?;
    elab.globals
        .insert("resp_coproduct".to_string(), resp_coproduct_id);

    let bind_id =
        state_eff::declare_bind(&mut elab.env, itree_id, vis_id).map_err(ElabError::Internal)?;
    elab.globals.insert("bind".to_string(), bind_id);

    let run_state_id = state_eff::declare_run_state(
        &mut elab.env,
        itree_id,
        ret_id,
        vis_id,
        state_op_id,
        get_id,
        put_id,
        coproduct_id,
        inl_id,
        inr_id,
        resp_state_id,
        resp_coproduct_id,
        unit_id,
        mkunit_id,
    )
    .map_err(ElabError::Internal)?;
    elab.globals.insert("run_state".to_string(), run_state_id);

    let get_fn_id = state_eff::declare_get(
        &mut elab.env,
        itree_id,
        ret_id,
        vis_id,
        state_op_id,
        get_id,
        coproduct_id,
        inl_id,
        resp_coproduct_id,
        resp_state_id,
        unit_id,
    )
    .map_err(ElabError::Internal)?;
    elab.globals.insert("get".to_string(), get_fn_id);

    let put_fn_id = state_eff::declare_put(
        &mut elab.env,
        itree_id,
        ret_id,
        vis_id,
        state_op_id,
        put_id,
        coproduct_id,
        inl_id,
        resp_coproduct_id,
        resp_state_id,
        unit_id,
        mkunit_id,
    )
    .map_err(ElabError::Internal)?;
    elab.globals.insert("put".to_string(), put_fn_id);

    // `inject_l`/`inject_r` — the general coproduct injection morphism
    // (`effect-composition` D2, doc §D2.1): the un-specialized form of
    // `get`/`put`'s hand-baked `InL` (`state.rs::declare_get`/`declare_put`
    // stay unchanged — State's tagging is *subsumed*, not forked, §D2.5).
    let inject_l_id = state_eff::declare_inject_l(
        &mut elab.env,
        itree_id,
        ret_id,
        vis_id,
        coproduct_id,
        resp_coproduct_id,
        inl_id,
    )
    .map_err(ElabError::Internal)?;
    elab.globals.insert("inject_l".to_string(), inject_l_id);

    let inject_r_id = state_eff::declare_inject_r(
        &mut elab.env,
        itree_id,
        ret_id,
        vis_id,
        coproduct_id,
        resp_coproduct_id,
        inr_id,
    )
    .map_err(ElabError::Internal)?;
    elab.globals.insert("inject_r".to_string(), inject_r_id);

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
        Term::lam(
            omega0.clone(),
            Term::sigma(Term::var(1), weaken(&Term::var(0), 1)),
        ),
    );
    let and_id = declare_def(&mut elab.env, vec![], and_ty, and_body)
        .map_err(|e| ElabError::Internal(format!("prelude And failed: {}", e)))?;
    elab.globals.insert("And".to_string(), and_id);

    // `and_intro`/`and_fst`/`and_snd` -- the Sigma intro/elim helpers for `And`,
    // built EXACTLY like `Pair`/`mk_pair`/`pair_fst`/`pair_snd` below (Map-
    // build), but Omega-sorted (`omega0`, not `type0`) since `And`'s two
    // arguments are themselves PROPOSITIONS, not types. `And A B := Sigma
    // (_:A).B` is a Sigma at Omega (`sort_sigma` classifies it there because
    // both components are Omega, `16 SS1.3`) -- the surface has no pair-
    // literal/`.1`/`.2` syntax for a bare Sigma (that's reserved for class-
    // dictionary records, `elab.rs::infer_proj`), so a multi-conjunct `And`
    // proof (e.g. `map-verified-laws`' `Ordered`-preservation goal, a nested
    // `And` of `allKeys` bounds) needs a named intro/elim, not a re-
    // application of `And` itself as if it were its own constructor. Zero
    // trusted_base delta: both reduce through the already-trusted
    // `Term::Pair`/`Term::Proj1`/`Term::Proj2` (Sigma is already in
    // `trusted_base()` via `And`/`Pair` themselves).
    let and_app_at_len2 = Term::app(
        Term::app(Term::const_(and_id, vec![]), Term::var(1)),
        Term::var(0),
    );
    let and_app_at_len4 = Term::app(
        Term::app(Term::const_(and_id, vec![]), Term::var(3)),
        Term::var(2),
    );

    // `and_intro : (a:Prop) -> (b:Prop) -> a -> b -> And a b`.
    let and_intro_ty = Term::pi(
        omega0.clone(),
        Term::pi(
            omega0.clone(),
            Term::pi(Term::var(1), Term::pi(Term::var(1), and_app_at_len4)),
        ),
    );
    let and_intro_body = Term::lam(
        omega0.clone(),
        Term::lam(
            omega0.clone(),
            Term::lam(
                Term::var(1),
                Term::lam(Term::var(1), Term::pair(Term::var(1), Term::var(0))),
            ),
        ),
    );
    let and_intro_id = declare_def(&mut elab.env, vec![], and_intro_ty, and_intro_body)
        .map_err(|e| ElabError::Internal(format!("prelude and_intro failed: {}", e)))?;
    elab.globals.insert("and_intro".to_string(), and_intro_id);

    // `and_fst : (a:Prop) -> (b:Prop) -> And a b -> a`.
    let and_fst_ty = Term::pi(
        omega0.clone(),
        Term::pi(
            omega0.clone(),
            Term::pi(and_app_at_len2.clone(), Term::var(2)),
        ),
    );
    let and_fst_body = Term::lam(
        omega0.clone(),
        Term::lam(
            omega0.clone(),
            Term::lam(and_app_at_len2.clone(), Term::proj1(Term::var(0))),
        ),
    );
    let and_fst_id = declare_def(&mut elab.env, vec![], and_fst_ty, and_fst_body)
        .map_err(|e| ElabError::Internal(format!("prelude and_fst failed: {}", e)))?;
    elab.globals.insert("and_fst".to_string(), and_fst_id);

    // `and_snd : (a:Prop) -> (b:Prop) -> And a b -> b`.
    let and_snd_ty = Term::pi(
        omega0.clone(),
        Term::pi(
            omega0.clone(),
            Term::pi(and_app_at_len2.clone(), Term::var(1)),
        ),
    );
    let and_snd_body = Term::lam(
        omega0.clone(),
        Term::lam(
            omega0.clone(),
            Term::lam(and_app_at_len2, Term::proj2(Term::var(0))),
        ),
    );
    let and_snd_id = declare_def(&mut elab.env, vec![], and_snd_ty, and_snd_body)
        .map_err(|e| ElabError::Internal(format!("prelude and_snd failed: {}", e)))?;
    elab.globals.insert("and_snd".to_string(), and_snd_id);

    // `Prop` — a surface-nameable alias for `Ω₀`, so a recursive `view` can
    // carry an explicit return-type ANNOTATION landing on `Ω` (required: a
    // self-recursive declaration needs a type annotation, `declare_def`
    // can't infer one — but the surface parser has no `Ω` TOKEN at all).
    // `Prop`'s own kernel type is `Type (suc 0)` (the `Ω-Form` rule,
    // `Omega(l) : Type (suc l)`); its BODY is `Term::Omega(Zero)` literally,
    // so kernel `check`/`convert` sees `Prop ≡ Ω₀` by ordinary δ-unfolding —
    // used only as a type-position spelling, never as a value.
    let prop_id = declare_def(
        &mut elab.env,
        vec![],
        Term::ty(Level::Zero.suc()),
        omega0.clone(),
    )
    .map_err(|e| ElabError::Internal(format!("prelude Prop failed: {}", e)))?;
    elab.globals.insert("Prop".to_string(), prop_id);

    // `Proved : Top` — the kernel's prelude `Top`-introduction constant (K5,
    // `16 §1.3`), already unconditionally declared by `GlobalEnv::new()`.
    // Surfaced here so a structure-class law proof can close a goal that
    // whnf's to `Top` (K7: an operation-wrapped `IsTrue`/`Equal` goal, the
    // same slot `Refl` occupied before the operand-whnf completeness fix,
    // `51 §6`). Ordinary global reference — `Proved` infers to `Top`, so it
    // needs no dedicated check-mode arm; the `check()` fallback (infer,
    // then unify against `expected`) handles it like any other constant.
    elab.globals.insert("Proved".to_string(), elab.env.tt_id());

    // `Bottom : Ω₀` — the kernel's prelude falsity proposition (K5, `16
    // §1.3`), already unconditionally declared by `GlobalEnv::new()`, same
    // status as `Proved`/`Top` above. Surfaced here (not previously nameable —
    // only the checked-mode `absurd h` sugar could reach it) so a law's TYPE
    // ANNOTATION can spell an order-distinctness negation `P -> Bottom`
    // directly (`map-verified-laws`' `distinct key key' := And (...) (...)
    // -> Bottom`, `52-map.md §5.2` locality law) — `absurd` alone lets you
    // *eliminate* a `Bottom`-typed hypothesis but never lets a `view`
    // signature *name* the type. Zero trusted_base delta: same existing
    // prelude constant, no new declaration.
    elab.globals
        .insert("Bottom".to_string(), elab.env.bottom_id());

    // `Top : Ω₀` — surface the kernel's existing truth proposition alongside
    // `Bottom`. Its sole inhabitant is the surface spelling `Proved` above.
    elab.globals.insert("Top".to_string(), elab.env.top_id());

    // `Not : Ω → Ω` (`¬A := A → Bottom`) — the surface has no EXPRESSION-
    // position `->` (only a `view`'s TYPE-annotation position parses the
    // Pi-sugar; `parse_expr` lacks it entirely, confirmed empirically), so
    // a Prop-returning `view` BODY cannot spell `A -> Bottom` directly as a
    // VALUE (e.g. `NoDup`'s per-entry negation predicate passed to
    // `allInList`, `map-verified-laws` law 5, `54 §4`). Built exactly like
    // `And`'s `Σ(_:A).B` above, using the already-trusted `Term::Pi` and
    // the existing `Bottom` constant. Zero `trusted_base` delta: ordinary
    // Pi-formation + the pre-existing `Bottom`, no new kernel primitive.
    let bottom_const = Term::const_(elab.env.bottom_id(), vec![]);
    let not_ty = Term::pi(omega0.clone(), omega0.clone());
    let not_body = Term::lam(
        omega0.clone(),
        Term::pi(Term::var(0), weaken(&bottom_const, 1)),
    );
    let not_id = declare_def(&mut elab.env, vec![], not_ty, not_body)
        .map_err(|e| ElabError::Internal(format!("prelude Not failed: {}", e)))?;
    elab.globals.insert("Not".to_string(), not_id);

    // `Pair : Type -> Type -> Type` — the non-dependent Sigma pair,
    // `k × v` (`52-map.md §4`, `13 §3`): "the Σ-pair... distinct from the
    // inductive `Prod`". Built exactly like `And`'s `Σ(_:A).B` above — the
    // kernel Sigma/Pair/Proj1/Proj2 formers are already landed
    // (`ken-kernel/src/term.rs`); only the concrete `×` INFIX surface
    // spelling is missing, which `52-map.md §4`'s own hedge tags `(oracle)`
    // ("any still-open surface-syntax token is tagged `(oracle)`") — the
    // mechanism (Sigma) is landed, so this is a named-application spelling
    // of it, not a new kernel feature or a workaround. `mk_pair`/`pair_fst`/
    // `pair_snd` are the corresponding intro/elim helpers (no surface `.1`/
    // `.2` projection exists for a bare Sigma — that syntax is reserved for
    // class-dictionary records only, `elab.rs::infer_proj`).
    let pair_ty_ty = Term::pi(type0.clone(), Term::pi(type0.clone(), type0.clone()));
    let pair_ty_body = Term::lam(
        type0.clone(),
        Term::lam(
            type0.clone(),
            Term::sigma(Term::var(1), weaken(&Term::var(0), 1)),
        ),
    );
    let pair_ty_id = declare_def(&mut elab.env, vec![], pair_ty_ty, pair_ty_body)
        .map_err(|e| ElabError::Internal(format!("prelude Pair failed: {}", e)))?;
    elab.globals.insert("Pair".to_string(), pair_ty_id);

    // `mk_pair : (a:Type) -> (b:Type) -> a -> b -> Pair a b`.
    let pair_app_at_len2 = Term::app(
        Term::app(Term::const_(pair_ty_id, vec![]), Term::var(1)),
        Term::var(0),
    );
    let pair_app_at_len4 = Term::app(
        Term::app(Term::const_(pair_ty_id, vec![]), Term::var(3)),
        Term::var(2),
    );
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
            Term::lam(
                Term::var(1),
                Term::lam(Term::var(1), Term::pair(Term::var(1), Term::var(0))),
            ),
        ),
    );
    let mkpair_id = declare_def(&mut elab.env, vec![], mkpair_ty, mkpair_body)
        .map_err(|e| ElabError::Internal(format!("prelude mk_pair failed: {}", e)))?;
    elab.globals.insert("mk_pair".to_string(), mkpair_id);

    // `pair_fst : (a:Type) -> (b:Type) -> Pair a b -> a`.
    let fst_ty = Term::pi(
        type0.clone(),
        Term::pi(
            type0.clone(),
            Term::pi(pair_app_at_len2.clone(), Term::var(2)),
        ),
    );
    let fst_body = Term::lam(
        type0.clone(),
        Term::lam(
            type0.clone(),
            Term::lam(pair_app_at_len2.clone(), Term::proj1(Term::var(0))),
        ),
    );
    let fst_id = declare_def(&mut elab.env, vec![], fst_ty, fst_body)
        .map_err(|e| ElabError::Internal(format!("prelude pair_fst failed: {}", e)))?;
    elab.globals.insert("pair_fst".to_string(), fst_id);

    // `pair_snd : (a:Type) -> (b:Type) -> Pair a b -> b`.
    let snd_ty = Term::pi(
        type0.clone(),
        Term::pi(
            type0.clone(),
            Term::pi(pair_app_at_len2.clone(), Term::var(1)),
        ),
    );
    let snd_body = Term::lam(
        type0.clone(),
        Term::lam(
            type0.clone(),
            Term::lam(pair_app_at_len2, Term::proj2(Term::var(0))),
        ),
    );
    let snd_id = declare_def(&mut elab.env, vec![], snd_ty, snd_body)
        .map_err(|e| ElabError::Internal(format!("prelude pair_snd failed: {}", e)))?;
    elab.globals.insert("pair_snd".to_string(), snd_id);

    // `Decimal`/`Char` DEMOTE→derived (`18a §5.6`/`§5.9`, Phase-2 tranche #2).
    // Must run here: after `Equal`/`And`/`Prop`/`Proved` (needed by `IsTrue`),
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

    // `is_sorted : Π(a:Type). (a → a → Bool) → List a → Ω` (ES2-remainder,
    // `37 §6`: the explicit-comparator form, `Ord`-class deferred).
    //
    // `is_sorted leq Nil = ⊤`, `is_sorted leq (x::Nil) = ⊤`,
    // `is_sorted leq (x::y::r) = IsTrue (leq x y) ∧ is_sorted leq (y::r)`, with
    // `IsTrue (leq x y) := Eq Bool (leq x y) True` (the `Equal`/kernel-`Eq`
    // alias already landed by ES2) and `∧` the already-demoted `And`. `⊤` is
    // spelled `Top`, surfaced above from the kernel's existing truth constant.
    let list_a = |a: Term| Term::app(Term::indformer(list_id, vec![]), a);
    let _ = &list_a; // still used below for Perm's raw-term construction
    elab.elaborate_decl(
        "fn is_sorted (a : Type) (leq : a -> a -> Bool) (xs : List a) : Prop = \
         match xs { \
           Nil |-> Top ; \
           Cons x xs2 |-> match xs2 { \
             Nil |-> Top ; \
             Cons y r |-> And (Equal Bool (leq x y) True) (is_sorted a leq xs2) \
           } \
         }",
    )
    .map_err(|e| ElabError::Internal(format!("prelude is_sorted failed: {}", e)))?;
    let issorted_id = elab
        .globals
        .get("is_sorted")
        .copied()
        .ok_or_else(|| ElabError::Internal("prelude: 'is_sorted' not registered".into()))?;

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
            Term::app(
                Term::app(
                    Term::Constructor {
                        id: cons_id,
                        level_args: vec![],
                    },
                    a,
                ),
                h,
            ),
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
                        Term::var(0),         // x:a,        ctx [a]
                        Term::var(1),         // y:a,        ctx [a,x]      (a=Var1)
                        list_a(Term::var(2)), // r:List a,   ctx [a,x,y]    (a=Var2)
                    ],
                    // ctx [a,x,y,r]: a=Var3, x=Var2, y=Var1, r=Var0.
                    target_indices: vec![
                        cons_app(
                            Term::var(3),
                            Term::var(2),
                            cons_app(Term::var(3), Term::var(1), Term::var(0)),
                        ),
                        cons_app(
                            Term::var(3),
                            Term::var(1),
                            cons_app(Term::var(3), Term::var(2), Term::var(0)),
                        ),
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

    // `Or : Ω → Ω → Type` — the two-constructor sum over PROPOSITIONS
    // (`map-verified-laws`' `boolDichotomy` reflect-combinator envelope,
    // `54-map-verified-laws.md §3`). Must be `Type`-valued (proof-relevant:
    // case-splitting on WHICH disjunct holds must be informative, unlike an
    // `Ω`-valued disjunction which would make `Inl`/`Inr` proof-irrelevantly
    // equal — `[[proof-relevant-inductive-cannot-be-declared-at-omega]]`),
    // but its two PAYLOADS are themselves `Ω`-classified propositions (e.g.
    // `Equal Bool (leq k k') True`) — a strictly different shape from every
    // other sum in this catalog (`Option`/`Result`, whose payloads are
    // `Type`-classified). The surface `data` sugar (`data.rs::elab_data_decl`)
    // hardcodes every parameter to `Type 0` (no way to spell an `Ω`-sorted
    // parameter there), so — mirroring `Perm_rel` immediately above, the
    // SAME "kernel `declare_inductive` directly, one level below the
    // elaborator's surface convenience wrapper" technique — `Or` is built
    // against the kernel API, which DOES support arbitrary parameter sorts.
    // Zero trusted_base delta: an ordinary `declare_inductive` admission,
    // kernel-rechecked, identical trust category to every other `data`.
    // Unlike `Perm_rel`'s ctors (internal plumbing, never surface-matched),
    // `Or`/`Inl`/`Inr` ARE surface-referenced (pattern-matched by
    // `boolDichotomy`'s callers), so all three are registered in
    // `elab.globals` below.
    let or_id = ken_kernel::declare_inductive(&mut elab.env, |_or_id| {
        ken_kernel::InductiveSpec {
            level_params: vec![],
            // `Δ_p = [a : Ω₀, b : Ω₀]` — params innermost-first: ctor-arg
            // context has `b` at `Var(0)` (last param), `a` at `Var(1)`
            // (first param), per `data.rs`'s own documented convention.
            params: vec![omega0.clone(), omega0.clone()],
            indices: vec![],
            level: Level::Zero,
            constructors: vec![
                // `Inl : a -> Or a b`, ctx `[a,b]`: `a` = Var(1).
                ken_kernel::CtorSpec {
                    args: vec![Term::var(1)],
                    target_indices: vec![],
                },
                // `Inr : b -> Or a b`, ctx `[a,b]`: `b` = Var(0).
                ken_kernel::CtorSpec {
                    args: vec![Term::var(0)],
                    target_indices: vec![],
                },
            ],
        }
    })
    .map_err(|e| ElabError::Internal(format!("prelude Or failed: {}", e)))?;
    elab.globals.insert("Or".to_string(), or_id);
    let or_ind = elab
        .env
        .inductive(or_id)
        .ok_or_else(|| {
            ElabError::Internal("prelude: 'Or' inductive not found after declare".into())
        })?
        .clone();
    elab.globals
        .insert("Inl".to_string(), or_ind.constructors[0].id);
    elab.globals
        .insert("Inr".to_string(), or_ind.constructors[1].id);

    // `Empty : Type0` — the computational false (DS-1, `docs/program/wp/
    // catalog-ds-1-empty-dec.md` Fork 1), zero params/zero constructors.
    // Bootstrapped here — same as every other prelude `data` above
    // (`Nat`/`List`/`Option`/…) — because `Dec`'s `No` constructor (next)
    // needs `Empty`'s `GlobalId` at kernel-declare time, before any catalog
    // `.ken.md` file is elaborated.
    //
    // **Build-time finding (empirically confirmed, not a design fork):**
    // Fork 1's `data Empty : Type0 =` does NOT parse as *written* — the
    // legacy `data D = …` arm doesn't take a `:`-ascribed family type
    // (the explicit-family `data Empty : Type0 where { }` spelling now
    // does, `docs/program/wp/ds-1-findings-remediation.md` FR-1, landed:
    // `parse_data_decl`/`parse_explicit_data_decl` both admit a zero-
    // constructor case now). This bootstrap still bypasses the parser
    // regardless of that fix — `Dec`'s `No` constructor needs `Empty`'s
    // `GlobalId` before any catalog `.ken.md` file (hence any surface
    // `data`) can elaborate at all, an ordering constraint FR-1 doesn't
    // touch — via the SAME technique `ElabEnv::empty()` already uses to
    // bootstrap `Bool` before the full `ElabEnv` exists: call
    // `data::elab_data_decl` (the real surface-data ELABORATION function,
    // `data.rs`) directly with an empty ctor list — zero new trust
    // category, same mechanism as every other prelude `data`.
    let empty_id = crate::data::elab_data_decl(
        &mut elab.env,
        &mut elab.globals,
        "Empty",
        &[],
        &[],
        &crate::error::Span::zero(),
    )
    .map_err(|e| ElabError::Internal(format!("prelude Empty failed: {}", e)))?;

    // `Dec (P : Omega) : Type0 = Yes P | No (P -> Empty)` (DS-1 Fork 2) —
    // Lean's `Decidable` shape, kernel-direct (`declare_inductive`) because
    // surface `data` hardcodes every param to `Type0`
    // (`crates/ken-elaborator/src/data.rs:45`) and has no way to spell the
    // Ω-sorted `P`. Zero new trusted-base category: an ordinary
    // `declare_inductive` admission, kernel-rechecked, identical trust
    // category to `Or`/`Perm_rel` immediately above. Confirmed to admit and
    // to large-eliminate into a `Type0` motive by the DS-1 build-step-1
    // smoke test (`crates/ken-elaborator/tests/ds1_smoke_test.rs`).
    let dec_id = ken_kernel::declare_inductive(&mut elab.env, |_dec_id| {
        ken_kernel::InductiveSpec {
            level_params: vec![],
            // `Δ_p = [P : Ω₀]` — ctor-arg context has `P` at `Var(0)`.
            params: vec![omega0.clone()],
            indices: vec![],
            level: Level::Zero,
            constructors: vec![
                // `Yes : P -> Dec P`.
                ken_kernel::CtorSpec {
                    args: vec![Term::var(0)],
                    target_indices: vec![],
                },
                // `No : (P -> Empty) -> Dec P`.
                ken_kernel::CtorSpec {
                    args: vec![Term::pi(Term::var(0), Term::indformer(empty_id, vec![]))],
                    target_indices: vec![],
                },
            ],
        }
    })
    .map_err(|e| ElabError::Internal(format!("prelude Dec failed: {}", e)))?;
    elab.globals.insert("Dec".to_string(), dec_id);
    let dec_ind = elab
        .env
        .inductive(dec_id)
        .ok_or_else(|| {
            ElabError::Internal("prelude: 'Dec' inductive not found after declare".into())
        })?
        .clone();
    elab.globals
        .insert("Yes".to_string(), dec_ind.constructors[0].id);
    elab.globals
        .insert("No".to_string(), dec_ind.constructors[1].id);

    // `decide : (P:Omega) -> Dec P -> Bool` — the kernel-direct accessor
    // (DS-1 deliverable 1). `Dec`/`Yes`/`No` are already real `globals`
    // entries, so this is ordinary surface `match` dispatch (the same
    // constructor-name-in-globals resolution `boolDichotomy` uses to match
    // on the kernel-direct `Or`/`Inl`/`Inr` immediately above) — no raw
    // `Term::Elim` construction needed.
    elab.elaborate_decl(
        "fn decide (P : Omega) (d : Dec P) : Bool = match d { Yes p ↦ True ; No f ↦ False }",
    )
    .map_err(|e| ElabError::Internal(format!("prelude decide failed: {}", e)))?;

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
    // ordered BST (`catalog/packages/Data/Collections/Map.ken`'s `Tree k v` + `insert`/
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

    // Console's response family is a genuine non-constant large elimination,
    // kernel-checked as ordinary Ken. Host failures remain total values.
    elab.elaborate_decl(
        "fn console_resp (op : ConsoleOp) : Type = match op { \
           Read stream limit |-> Result IOError ReadResult; \
           Write stream bytes |-> Result IOError Unit; \
           Flush stream |-> Result IOError Unit; \
           IsTerminal stream |-> Bool \
         }",
    )
    .map_err(|e| ElabError::Internal(format!("prelude console_resp failed: {}", e)))?;

    // `IO` is the existing `ITree` specialized to the one Console algebra.
    elab.elaborate_decl("const IO (a : Type) : Type = ITree ConsoleOp console_resp a")
        .map_err(|e| ElabError::Internal(format!("prelude IO failed: {}", e)))?;
    let io_id = elab
        .globals
        .get("IO")
        .copied()
        .ok_or_else(|| ElabError::Internal("prelude: 'IO' not registered".into()))?;

    elab.elaborate_decl(
        "proc read (stream : Stream) (limit : Int) : IO (Result IOError ReadResult) visits [Console] = \
         Vis ConsoleOp console_resp (Result IOError ReadResult) (Read stream limit) \
           (\\r. Ret ConsoleOp console_resp (Result IOError ReadResult) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude read failed: {}", e)))?;
    elab.elaborate_decl(
        "proc write (stream : Stream) (bytes : Bytes) : IO (Result IOError Unit) visits [Console] = \
         Vis ConsoleOp console_resp (Result IOError Unit) (Write stream bytes) \
           (\\r. Ret ConsoleOp console_resp (Result IOError Unit) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude write failed: {}", e)))?;
    elab.elaborate_decl(
        "proc flush (stream : Stream) : IO (Result IOError Unit) visits [Console] = \
         Vis ConsoleOp console_resp (Result IOError Unit) (Flush stream) \
           (\\r. Ret ConsoleOp console_resp (Result IOError Unit) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude flush failed: {}", e)))?;
    elab.elaborate_decl(
        "proc is_terminal (stream : Stream) : IO Bool visits [Console] = \
         Vis ConsoleOp console_resp Bool (IsTerminal stream) \
           (\\r. Ret ConsoleOp console_resp Bool r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude is_terminal failed: {}", e)))?;

    // Compatibility helper retained for existing examples. It is still an
    // ordinary definition and now emits exact UTF-8 bytes plus one newline.
    elab.elaborate_decl(
        "proc print_line (s : String) : IO Unit visits [Console] = \
         Vis ConsoleOp console_resp Unit \
           (Write Stdout (bytes_concat (bytes_encode s) \
             (bytes_encode (list_char_to_string (Cons Char (10 : Int) (Nil Char)))))) \
           (\\_. Ret ConsoleOp console_resp Unit MkUnit)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude print_line failed: {}", e)))?;
    let print_line_id = elab
        .globals
        .get("print_line")
        .copied()
        .ok_or_else(|| ElabError::Internal("prelude: 'print_line' not registered".into()))?;

    // ── `[FS]` real file I/O (VAL2 #9 / OQ-B, FS-driver-build D1) ──────────
    //
    // Mirrors `print_line`'s own real-`view` pattern above (`ITree`/`Vis`/
    // `Ret` applied at their 3 explicit type params) — a genuine kernel-
    // rechecked reduction with no `apply()` interception: `print_line`
    // reduces by ordinary delta/iota through its real body, and `read_bytes`
    // does too.
    //
    // `Auth = ANone | APartial | AFull` — the authority-level lattice
    // (fs-read-file-lines-flip D2, operator-locked "type IS the manifest"
    // ruling). An ordinary checked inductive — data, not a proposition; the
    // ordering `ANone ⊑ APartial ⊑ AFull` lives in Rust at the CLI + driver
    // (`capabilities.rs`), never as a type-level order proof.
    elab.elaborate_decl("data Auth = ANone | APartial | AFull")
        .map_err(|e| ElabError::Internal(format!("prelude Auth failed: {}", e)))?;
    let auth_id = elab
        .globals
        .get("Auth")
        .copied()
        .ok_or_else(|| ElabError::Internal("prelude: 'Auth' not registered".into()))?;
    // `Auth` is an INDUCTIVE family (declared via the `data` machinery), so
    // referencing it as a type is `Term::IndFormer`, never `Term::Const`
    // (the kernel treats the two as non-convertible, `elab.rs:351-352`).
    let auth_t = Term::indformer(auth_id, vec![]);

    // `Cap : Auth -> Type0` — authority-indexed opaque former (D2, operator +
    // Architect ruling `evt_fgkd29xbf35q`). Registers via the SAME
    // `declare_primitive(…, OpaqueType)` path the former bare `Cap : Type0`
    // placeholder used — `classify` accepts a Π former type and `OpaqueType`
    // imposes no kind restriction (Architect's AC1 closure: zero kernel
    // delta, no new `Term`/`Decl`). `Cap` stays one opaque postulate; opaque
    // formers never δ-unfold (`conv.rs` never references `OpaqueType`), so
    // `Cap APartial`/`Cap ANone` are genuinely distinct stuck-neutral types.
    let cap_ty = Term::pi(auth_t.clone(), type0.clone());
    let cap_id = declare_primitive(&mut elab.env, vec![], cap_ty, PrimReduction::OpaqueType)
        .map_err(|e| ElabError::Internal(format!("prelude Cap failed: {}", e)))?;
    elab.globals.insert("Cap".to_string(), cap_id);

    let bytes_t = elab
        .globals
        .get("Bytes")
        .copied()
        .map(|id| Term::const_(id, vec![]))
        .ok_or_else(|| ElabError::Internal("prelude: 'Bytes' not registered".into()))?;

    // `FSOp : Auth -> Type0`; every operation carries its authority token and
    // raw-byte path in the node. The surface `data` parser cannot express the
    // value-kind `Auth` parameter, so this remains an ordinary hand-built
    // kernel inductive.
    // — a genuinely Auth-PARAMETERIZED family (a uniform parameter, not a
    // varying GADT index: the sole constructor always targets `FSOp` at its
    // OWN parameter, exactly like `List a`). Hand-built via
    // `declare_inductive` (not surface `data`) because the surface `data`
    // parser hard-codes every parameter's kind to `Type 0`
    // (`data.rs::elab_data_decl`'s `params: (0..m).map(|_| Term::ty(...))`)
    // — it cannot express a parameter of kind `Auth`. Same technique
    // `effects::state` already uses for `ITree` (dependent shapes the
    // surface parser can't express), applied to the smallest case needing
    // it here. The capability is carried IN the op node (`ReadFile cap
    // path`) per D3's capability-carrying (not ambient authority) design —
    // the driver's FS arm reads both fields off the `Vis` node before any
    // syscall.
    let create_policy_t = elab
        .globals
        .get("CreatePolicy")
        .copied()
        .map(|id| Term::indformer(id, vec![]))
        .ok_or_else(|| ElabError::Internal("prelude: CreatePolicy missing".into()))?;
    let bool_t = elab
        .globals
        .get("Bool")
        .copied()
        .map(|id| Term::indformer(id, vec![]))
        .ok_or_else(|| ElabError::Internal("prelude: Bool missing".into()))?;
    let fsop_id = declare_inductive(&mut elab.env, |_fsop_id| InductiveSpec {
        level_params: vec![],
        params: vec![auth_t.clone()],
        indices: vec![],
        level: Level::Zero,
        constructors: {
            let cap_a = || Term::app(Term::const_(cap_id, vec![]), Term::var(0));
            let named = |args| CtorSpec {
                args,
                target_indices: vec![],
            };
            vec![
                named(vec![cap_a(), bytes_t.clone()]),
                named(vec![
                    cap_a(),
                    bytes_t.clone(),
                    create_policy_t,
                    bytes_t.clone(),
                ]),
                named(vec![cap_a(), bytes_t.clone(), bytes_t.clone()]),
                named(vec![cap_a(), bytes_t.clone()]),
                named(vec![cap_a(), bytes_t.clone()]),
                named(vec![cap_a(), bool_t.clone(), bytes_t.clone()]),
                named(vec![cap_a(), bytes_t.clone()]),
                named(vec![cap_a(), bool_t, bytes_t.clone()]),
                named(vec![cap_a(), bytes_t.clone(), bytes_t.clone()]),
            ]
        },
    })
    .map_err(|e| ElabError::Internal(format!("prelude FSOp failed: {}", e)))?;
    elab.globals.insert("FSOp".to_string(), fsop_id);
    let fs_ctor_ids: Vec<_> = elab
        .env
        .inductive(fsop_id)
        .expect("FSOp present after declaration")
        .constructors
        .iter()
        .map(|ctor| ctor.id)
        .collect();
    for (name, id) in [
        "ReadFile",
        "WriteFile",
        "AppendFile",
        "Metadata",
        "ReadDirectory",
        "CreateDirectory",
        "RemoveFile",
        "RemoveDirectory",
        "Rename",
    ]
    .into_iter()
    .zip(fs_ctor_ids)
    {
        elab.globals.insert(name.to_string(), id);
    }

    // `fs_resp : (a : Auth) -> FSOp a -> Type` is a genuine non-constant
    // large elimination. Every arm returns a total `Result FileError _`, with
    // the success payload selected by the operation constructor.
    elab.elaborate_decl(
        "fn fs_resp (a : Auth) (op : FSOp a) : Type = match op { \
           ReadFile cap path |-> Result FileError Bytes; \
           WriteFile cap path policy contents |-> Result FileError Unit; \
           AppendFile cap path contents |-> Result FileError Unit; \
           Metadata cap path |-> Result FileError FileMetadata; \
           ReadDirectory cap path |-> Result FileError (List DirEntry); \
           CreateDirectory cap recursive path |-> Result FileError Unit; \
           RemoveFile cap path |-> Result FileError Unit; \
           RemoveDirectory cap recursive path |-> Result FileError Unit; \
           Rename cap from to |-> Result FileError Unit \
         }",
    )
    .map_err(|e| ElabError::Internal(format!("prelude fs_resp failed: {}", e)))?;

    // `FS : Auth -> Type -> Type = \a r. ITree (FSOp a) (fs_resp a) r` — the
    // file-I/O analog of `IO`, reusing the lifted, effect-generic `ITree` (no
    // second effect system). Auth-parameterized (build-level mechanical
    // consequence of D2's `Cap`/`FSOp` enrichment: `FSOp`/`fs_resp` are no
    // longer flat, already-applied `Type`s once Auth-indexed, so `ITree`'s
    // `E`/`Resp` type arguments need a concrete `a` supplied — threaded from
    // `read_bytes`'s own bound `a`). This does not reopen any locked
    // decision: `read_bytes` stays authority-polymorphic (α) with sufficiency
    // enforced only at the runtime `authorizes` gate; `FS`'s extra Auth
    // parameter is purely outer-ring plumbing, invisible to the driver
    // (fully erased at the `EvalVal` layer) and to every AC/BV.
    elab.elaborate_decl("fn FS (a : Auth) (r : Type) : Type = ITree (FSOp a) (fs_resp a) r")
        .map_err(|e| ElabError::Internal(format!("prelude FS failed: {}", e)))?;

    // `read_bytes : (a : Auth) -> Cap a -> Bytes -> FS a (Result FileError Bytes)`
    //   = \a cap path. Vis (FSOp a) (fs_resp a) (Result FileError Bytes) (ReadFile cap path)
    //                    (\r. Ret (FSOp a) (fs_resp a) (Result FileError Bytes) r)
    //
    // α (fs-read-file-lines-flip D2, forced by locked AC4 + SEAM-A, settled
    // by citation): `read_bytes` is authority-POLYMORPHIC — no static
    // sufficiency check. Sufficiency (`a ⊒ APartial`) is enforced ONLY at
    // the runtime driver `authorizes` gate (`ken-interp/src/eval.rs`).
    //
    // A **pure, total** constructor-application definition (D5/AC5): reduces
    // in the pure core to a `Vis (ReadFile cap path) (λr. Ret r)` `ITree`
    // value — no syscall, no partiality. Declaration elaboration records its
    // `[FS]` row in the final `ElabEnv.effect_rows`; L6's escape oracle reads
    // that declaration-backed entry directly.
    elab.elaborate_decl(
        "proc read_bytes (a : Auth) (cap : Cap a) (path : Bytes) : FS a (Result FileError Bytes) visits [FS] = \
         Vis (FSOp a) (fs_resp a) (Result FileError Bytes) (ReadFile a cap path) \
           (\\r. Ret (FSOp a) (fs_resp a) (Result FileError Bytes) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude read_bytes failed: {}", e)))?;
    elab.elaborate_decl(
        "proc write_file (a : Auth) (cap : Cap a) (path : Bytes) (policy : CreatePolicy) (contents : Bytes) : FS a (Result FileError Unit) visits [FS] = \
         Vis (FSOp a) (fs_resp a) (Result FileError Unit) (WriteFile a cap path policy contents) \
           (\\r. Ret (FSOp a) (fs_resp a) (Result FileError Unit) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude write_file failed: {}", e)))?;
    elab.elaborate_decl(
        "proc append_file (a : Auth) (cap : Cap a) (path : Bytes) (contents : Bytes) : FS a (Result FileError Unit) visits [FS] = \
         Vis (FSOp a) (fs_resp a) (Result FileError Unit) (AppendFile a cap path contents) \
           (\\r. Ret (FSOp a) (fs_resp a) (Result FileError Unit) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude append_file failed: {}", e)))?;
    elab.elaborate_decl(
        "proc file_metadata (a : Auth) (cap : Cap a) (path : Bytes) : FS a (Result FileError FileMetadata) visits [FS] = \
         Vis (FSOp a) (fs_resp a) (Result FileError FileMetadata) (Metadata a cap path) \
           (\\r. Ret (FSOp a) (fs_resp a) (Result FileError FileMetadata) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude file_metadata failed: {}", e)))?;
    elab.elaborate_decl(
        "proc read_directory (a : Auth) (cap : Cap a) (path : Bytes) : FS a (Result FileError (List DirEntry)) visits [FS] = \
         Vis (FSOp a) (fs_resp a) (Result FileError (List DirEntry)) (ReadDirectory a cap path) \
           (\\r. Ret (FSOp a) (fs_resp a) (Result FileError (List DirEntry)) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude read_directory failed: {}", e)))?;
    elab.elaborate_decl(
        "proc create_directory (a : Auth) (cap : Cap a) (recursive : Bool) (path : Bytes) : FS a (Result FileError Unit) visits [FS] = \
         Vis (FSOp a) (fs_resp a) (Result FileError Unit) (CreateDirectory a cap recursive path) \
           (\\r. Ret (FSOp a) (fs_resp a) (Result FileError Unit) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude create_directory failed: {}", e)))?;
    elab.elaborate_decl(
        "proc remove_file (a : Auth) (cap : Cap a) (path : Bytes) : FS a (Result FileError Unit) visits [FS] = \
         Vis (FSOp a) (fs_resp a) (Result FileError Unit) (RemoveFile a cap path) \
           (\\r. Ret (FSOp a) (fs_resp a) (Result FileError Unit) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude remove_file failed: {}", e)))?;
    elab.elaborate_decl(
        "proc remove_directory (a : Auth) (cap : Cap a) (recursive : Bool) (path : Bytes) : FS a (Result FileError Unit) visits [FS] = \
         Vis (FSOp a) (fs_resp a) (Result FileError Unit) (RemoveDirectory a cap recursive path) \
           (\\r. Ret (FSOp a) (fs_resp a) (Result FileError Unit) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude remove_directory failed: {}", e)))?;
    elab.elaborate_decl(
        "proc rename_file (a : Auth) (cap : Cap a) (from : Bytes) (to : Bytes) : FS a (Result FileError Unit) visits [FS] = \
         Vis (FSOp a) (fs_resp a) (Result FileError Unit) (Rename a cap from to) \
           (\\r. Ret (FSOp a) (fs_resp a) (Result FileError Unit) r)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude rename_file failed: {}", e)))?;

    // Program-I I-1 entrypoint ABI. These are ordinary, kernel-checked Ken
    // declarations: the host runner knows their fixed shape, but no kernel
    // rule or trusted primitive is added for them.
    elab.elaborate_decl("data ExitCode = Success | Failure UInt8")
        .map_err(|e| ElabError::Internal(format!("prelude ExitCode failed: {}", e)))?;
    elab.elaborate_decl(
        "data ProcessInput = MkProcessInput (List Bytes) (List (Prod Bytes Bytes)) Bytes",
    )
    .map_err(|e| ElabError::Internal(format!("prelude ProcessInput failed: {}", e)))?;

    // The surface `data` helper cannot currently express this constructor
    // field because `Cap` is indexed by the value-level `Auth` family. Use
    // the ordinary kernel inductive API, exactly as `FSOp` above does; this is
    // still a checked inductive and adds no primitive or trusted rule.
    let program_caps_id = declare_inductive(&mut elab.env, |_id| InductiveSpec {
        level_params: vec![],
        params: vec![auth_t.clone()],
        indices: vec![],
        level: Level::Zero,
        constructors: vec![CtorSpec {
            args: vec![Term::app(Term::const_(cap_id, vec![]), Term::var(0))],
            target_indices: vec![],
        }],
    })
    .map_err(|e| ElabError::Internal(format!("prelude ProgramCaps failed: {e}")))?;
    elab.globals.insert("ProgramCaps".into(), program_caps_id);
    let program_caps_ctor = elab
        .env
        .inductive(program_caps_id)
        .and_then(|ind| ind.constructors.first())
        .map(|ctor| ctor.id)
        .ok_or_else(|| ElabError::Internal("prelude: MkProgramCaps missing".into()))?;
    elab.globals
        .insert("MkProgramCaps".into(), program_caps_ctor);

    elab.elaborate_decl(
        "const HostIO (r : Type) : Type = \
         ITree (Coproduct (FSOp APartial) ConsoleOp) \
           (resp_coproduct (FSOp APartial) ConsoleOp (fs_resp APartial) console_resp) r",
    )
    .map_err(|e| ElabError::Internal(format!("prelude HostIO failed: {}", e)))?;
    elab.elaborate_decl(
        "fn host_exit (code : ExitCode) : HostIO ExitCode = \
         Ret (Coproduct (FSOp APartial) ConsoleOp) \
           (resp_coproduct (FSOp APartial) ConsoleOp (fs_resp APartial) console_resp) \
           ExitCode code",
    )
    .map_err(|e| ElabError::Internal(format!("prelude host_exit failed: {}", e)))?;
    elab.elaborate_decl(
        "proc host_program_then (action : IO Unit) (code : ExitCode) \
         : HostIO ExitCode visits [Console] = \
         bind (Coproduct (FSOp APartial) ConsoleOp) \
           (resp_coproduct (FSOp APartial) ConsoleOp (fs_resp APartial) console_resp) \
           Unit ExitCode \
           (inject_r (FSOp APartial) ConsoleOp (fs_resp APartial) console_resp Unit action) \
           (\\_. host_exit code)",
    )
    .map_err(|e| ElabError::Internal(format!("prelude host_program_then failed: {}", e)))?;
    elab.elaborate_decl(
        "proc host_program (action : IO Unit) : HostIO ExitCode visits [Console] = \
         host_program_then action Success",
    )
    .map_err(|e| ElabError::Internal(format!("prelude host_program failed: {}", e)))?;

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
        stream_id,
        stdin_id,
        stdout_id,
        stderr_id,
        console_op_id,
        read_id,
        write_id,
        flush_id,
        is_terminal_id,
        read_result_id,
        chunk_id,
        eof_id,
        itree_id,
        ret_id,
        vis_id,
        io_id,
        print_line_id,
        mkdecimalpair_id: decimal_char_env.mkdecimalpair_id,
        state_op_id,
        get_id,
        put_id,
        coproduct_id,
        inl_id,
        inr_id,
        resp_state_id,
        resp_coproduct_id,
        bind_id,
        run_state_id,
        get_fn_id,
        put_fn_id,
        inject_l_id,
        inject_r_id,
    })
}
