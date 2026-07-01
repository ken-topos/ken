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

use ken_kernel::{declare_primitive, env::PrimReduction, GlobalId, Level, Term};

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
        ordresult_id: z,
        lt_id: z,
        eqr_id: z,
        gt_id: z,
        equal_id: z,
        and_id: z,
        issorted_id: z,
        perm_id: z,
        byte_length_id: z,
        char_length_id: z,
        string_to_list_char_id: z,
        list_char_to_string_id: z,
        map_id: z,
        set_id: z,
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
    // `OrdResult` — a matchable comparison result (`Lt` / `Eq` / `Gt`). `Bool`
    // is an opaque primitive (not `data Bool = True | False`), so it is not
    // pattern-matchable; `sort` / `insert` branch on `OrdResult` instead.
    pub ordresult_id: GlobalId,
    pub lt_id: GlobalId,
    pub eqr_id: GlobalId,
    pub gt_id: GlobalId,
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
    // L3b: abstract collection types (`37 §6`).
    /// `Map : Type → Type → Type` (abstract; `DecEq K` gate via `where`).
    pub map_id: GlobalId,
    /// `Set : Type → Type` (abstract; `DecEq A` gate via `where`).
    pub set_id: GlobalId,
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
    elab.elaborate_decl("data OrdResult = Lt | Eq | Gt")
        .map_err(|e| ElabError::Internal(format!("prelude OrdResult failed: {}", e)))?;

    // VAL1-surface inductives — declared before `lookup` closure to avoid
    // conflicting borrows (elaborate_decl needs &mut elab).
    elab.elaborate_decl("data Unit = MkUnit")
        .map_err(|e| ElabError::Internal(format!("prelude Unit failed: {}", e)))?;
    elab.elaborate_decl("data ConsoleOp = Write String")
        .map_err(|e| ElabError::Internal(format!("prelude ConsoleOp failed: {}", e)))?;
    // Two-field Vis: effect op first, continuation second — matches run_io contract.
    elab.elaborate_decl("data ITree r = Ret r | Vis ConsoleOp (Unit -> ITree r)")
        .map_err(|e| ElabError::Internal(format!("prelude ITree failed: {}", e)))?;

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
    let ordresult_id = lookup("OrdResult")?;
    let lt_id = lookup("Lt")?;
    let eqr_id = lookup("Eq")?;
    let gt_id = lookup("Gt")?;
    // VAL1-surface inductives (declared before lookup closure above).
    let unit_id = lookup("Unit")?;
    let mkunit_id = lookup("MkUnit")?;
    let console_op_id = lookup("ConsoleOp")?;
    let write_id = lookup("Write")?;
    let itree_id = lookup("ITree")?;
    let ret_id = lookup("Ret")?;
    let vis_id = lookup("Vis")?;
    // `lookup` is last used above; NLL ends its borrow here.

    // ── Ω constants (postulates with Pi types, applied form) ───────────────
    // `Equal : Π(A:Type). Π(x:A). Π(y:A). Ω`  (the `≡`).
    // de Bruijn: Pi(Type, Pi(Var 0, Pi(Var 1, Ω₀)))  — A=Var0, x=Var0, y=Var1
    // under their binders.
    let equal_ty = Term::pi(
        type0.clone(),
        Term::pi(Term::var(0), Term::pi(Term::var(1), omega0.clone())),
    );
    let equal_id = elab
        .declare_postulate_raw("Equal", equal_ty)
        .map_err(|e| ElabError::Internal(format!("prelude Equal failed: {}", e)))?;

    // `And : Ω → Ω → Ω`  (the `∧`).
    let and_ty = Term::pi(omega0.clone(), Term::pi(omega0.clone(), omega0.clone()));
    let and_id = elab
        .declare_postulate_raw("And", and_ty)
        .map_err(|e| ElabError::Internal(format!("prelude And failed: {}", e)))?;

    // `isSorted : Π(A:Type). List A → Ω`.
    let list_a = |a: Term| Term::app(Term::indformer(list_id, vec![]), a);
    let issorted_ty = Term::pi(
        type0.clone(),
        Term::pi(list_a(Term::var(0)), omega0.clone()),
    );
    let issorted_id = elab
        .declare_postulate_raw("isSorted", issorted_ty)
        .map_err(|e| ElabError::Internal(format!("prelude isSorted failed: {}", e)))?;

    // `Perm : Π(A:Type). List A → List A → Ω`.
    let perm_ty = Term::pi(
        type0.clone(),
        Term::pi(
            list_a(Term::var(0)),
            Term::pi(list_a(Term::var(1)), omega0.clone()),
        ),
    );
    let perm_id = elab
        .declare_postulate_raw("Perm", perm_ty)
        .map_err(|e| ElabError::Internal(format!("prelude Perm failed: {}", e)))?;

    // ── L3b: abstract collection types (`37 §6`) ───────────────────────────
    // `Map : Type → Type → Type` — abstract; `DecEq K` gate enforced via
    // the `where` constraint mechanism in `elaborate_rdecl_v1` (L3b).
    let map_ty =
        Term::pi(type0.clone(), Term::pi(type0.clone(), type0.clone()));
    let map_id = elab
        .declare_postulate_raw("Map", map_ty)
        .map_err(|e| ElabError::Internal(format!("prelude Map failed: {}", e)))?;

    // `Set : Type → Type` — abstract; same `DecEq A` gate.
    let set_ty = Term::pi(type0.clone(), type0.clone());
    let set_id = elab
        .declare_postulate_raw("Set", set_ty)
        .map_err(|e| ElabError::Internal(format!("prelude Set failed: {}", e)))?;

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

    // `IO : Type → Type` — the Console-effect IO type (postulate; prim
    // reduction to `ITree ConsoleOp` held until `wp/VAL1-console-exec` lands).
    let io_ty = Term::pi(type0.clone(), type0.clone());
    let io_id = elab
        .declare_postulate_raw("IO", io_ty)
        .map_err(|e| ElabError::Internal(format!("prelude IO failed: {}", e)))?;

    // `print_line : String → IO Unit` — Console print primitive.
    // Declared as a primitive (not a postulate) so eval produces CtorPending
    // that is intercepted in `apply` once `store.print_line_id` is wired.
    // Symbol "print_line" is handled in `eval.rs::apply` using `store.console_ids`.
    let unit_t = Term::indformer(unit_id, vec![]);
    let io_unit = Term::app(Term::const_(io_id, vec![]), unit_t);
    let print_line_ty = Term::pi(string_t.clone(), io_unit);
    let print_line_id = declare_primitive(
        &mut elab.env,
        vec![],
        print_line_ty,
        PrimReduction::Op { symbol: "print_line" },
    )
    .map_err(|e| ElabError::Internal(format!("prelude print_line failed: {}", e)))?;
    elab.globals.insert("print_line".to_string(), print_line_id);

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
        ordresult_id,
        lt_id,
        eqr_id,
        gt_id,
        equal_id,
        and_id,
        issorted_id,
        perm_id,
        byte_length_id,
        char_length_id,
        string_to_list_char_id,
        list_char_to_string_id,
        map_id,
        set_id,
        unit_id,
        mkunit_id,
        console_op_id,
        write_id,
        itree_id,
        ret_id,
        vis_id,
        io_id,
        print_line_id,
    })
}
