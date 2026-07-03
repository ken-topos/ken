//! Size-change termination (SCT) gate (`17 §4`).
//!
//! Admitted at definition time by [`sct_check`]. Three steps:
//! 1. Scan each body for `Const` calls to group members; compute size-change
//!    matrices (`sizeRel` per arg vs param, `17 §4.2`).
//! 2. Compute the idempotent closure of self-loop matrices via Floyd-Warshall.
//! 3. Accept iff every idempotent self-loop has ≥1 `↓` on the diagonal.

use crate::env::GlobalEnv;
use crate::inductive::peel_app;
use crate::term::{GlobalId, Term};

// ---------------------------------------------------------------------------
// Size ordering and entry composition
// ---------------------------------------------------------------------------

/// Size relation between caller argument and callee parameter (`17 §4.2`).
/// Ordered `? < ↓= < ↓`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SizeOrd {
    Unknown,
    DownEq,
    Down,
}

/// Compose two consecutive size-relation steps (`17 §4.3`).
///
/// `a` = relation from the caller's param to the intermediate param (first
/// call); `b` = relation from the intermediate to the final param (second
/// call).  A `?` at EITHER end breaks the thread.
pub fn compose_ord(a: SizeOrd, b: SizeOrd) -> SizeOrd {
    use SizeOrd::*;
    match (a, b) {
        (Down, Down) | (Down, DownEq) | (DownEq, Down) => Down,
        (DownEq, DownEq) => DownEq,
        _ => Unknown, // any Unknown step breaks the thread
    }
}

// ---------------------------------------------------------------------------
// Size-change matrices
// ---------------------------------------------------------------------------

/// Size-change matrix for one call edge.  `entries[i][j]` = `sizeRel` of
/// caller param `i` to callee param `j`.
#[derive(Clone, Debug, PartialEq)]
struct ScMatrix {
    entries: Vec<Vec<SizeOrd>>,
    nrows: usize, // caller params
    ncols: usize, // callee params
}

impl ScMatrix {
    fn zero(nrows: usize, ncols: usize) -> Self {
        Self {
            entries: vec![vec![SizeOrd::Unknown; ncols]; nrows],
            nrows,
            ncols,
        }
    }

    /// Matrix product `self ⊙ rhs`.  `self.ncols` must equal `rhs.nrows`.
    fn compose(&self, rhs: &ScMatrix) -> ScMatrix {
        assert_eq!(self.ncols, rhs.nrows);
        let mut out = ScMatrix::zero(self.nrows, rhs.ncols);
        for i in 0..self.nrows {
            for k in 0..rhs.ncols {
                let mut best = SizeOrd::Unknown;
                for j in 0..self.ncols {
                    let v = compose_ord(self.entries[i][j], rhs.entries[j][k]);
                    if v > best {
                        best = v;
                    }
                }
                out.entries[i][k] = best;
            }
        }
        out
    }

    fn is_square(&self) -> bool {
        self.nrows == self.ncols
    }

    fn is_idempotent(&self) -> bool {
        if !self.is_square() {
            return false;
        }
        self.compose(self) == *self
    }

    fn has_strict_diagonal(&self) -> bool {
        (0..self.nrows).any(|i| self.entries[i][i] == SizeOrd::Down)
    }
}

// ---------------------------------------------------------------------------
// Provenance tracking
// ---------------------------------------------------------------------------

/// Per-variable size relation to a root parameter.
/// Index 0 = Var(0) in the current scope.
type Provenances = Vec<Option<(usize, SizeOrd)>>;

fn prov_push(p: &Provenances, entry: Option<(usize, SizeOrd)>) -> Provenances {
    let mut v = vec![entry];
    v.extend_from_slice(p);
    v
}

fn prov_get(p: &Provenances, i: usize) -> Option<(usize, SizeOrd)> {
    p.get(i).and_then(|x| *x)
}

/// Tags a bound variable as *exactly* field `field_pos` (0-indexed, positional)
/// of an `n_fields`-arity destructuring of caller parameter `param_idx` via
/// constructor `ctor` — i.e. the variable this tag is attached to is the raw
/// binder the match compiler introduced for that field, nothing more.
///
/// `sct-reconstruction-descent` (shape (b)): this is the load-bearing
/// soundness surface. It is consulted only by [`is_exact_reconstruction`],
/// which requires *every* field of a reconstruction to match its tag
/// positionally and by constructor+param — any added structure, reorder, or
/// substitution loses the match and the reconstruction is `Unknown`, never a
/// false `DownEq`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReconTag {
    param_idx: usize,
    ctor: GlobalId,
    n_fields: usize,
    field_pos: usize,
}

/// Per-variable reconstruction tag, threaded in lockstep with [`Provenances`]
/// (same push/pop sites, same indexing — `Var(i)`'s tag, if any, describes
/// the same binder `Provenances[i]` describes). Index 0 = Var(0).
type Reconstructions = Vec<Option<ReconTag>>;

fn recon_push(r: &Reconstructions, entry: Option<ReconTag>) -> Reconstructions {
    let mut v = vec![entry];
    v.extend_from_slice(r);
    v
}

fn recon_get(r: &Reconstructions, i: usize) -> Option<ReconTag> {
    r.get(i).and_then(|x| *x)
}

/// Does `arg` peel to `App*(Constructor{id}, args)` where `args` are *exactly*
/// the raw, positionally-ordered field binders of an `n_fields`-arity
/// destructuring of `param_idx` via constructor `id` — i.e. an exact,
/// same-size reconstruction of `param_idx`'s matched value (`Suc m2` where
/// `m2` was bound by matching `param_idx` against `Suc m2`)?
///
/// **The sole unsafe vector for this WP is over-firing this predicate** — see
/// `docs/program/wp/sct-reconstruction-descent.md` §"soundness boundary".
/// Every one of the following must hold or this returns `false` (⇒ caller
/// falls back to `Unknown`, the safe direction):
/// - the head is a bare `Term::Constructor` (not a `Const`, `IndFormer`, …);
/// - each argument is a **bare `Term::Var`**, never itself an application,
///   constant, or further constructor (rules out `badAck2`'s size-increasing
///   `Suc (Suc m2)` — its outer arg is `App(Ctor, App(Ctor, Var))`, not a raw
///   `Var`, so the inner `Var` check fails at position 0);
/// - `args.len()` equals the tag's recorded `n_fields` (rules out a partial
///   or over-applied reconstruction);
/// - each `args[j]`'s tag has the **same** `ctor` (rules out WRONG-CTOR),
///   the **same** `param_idx` (a reconstruction of a *different* param gives
///   no relation to this one), and **`field_pos == j`** (rules out REORDER —
///   a swapped positional match is not the identity reconstruction).
///
/// A tag lookup miss (an untagged var — an IH slot, a deferred continuation
/// slot with no known scrutinee provenance, or any ordinary binder) also
/// fails the check (rules out WRONG-FIELD: passing some other in-scope var).
fn is_exact_reconstruction(param_idx: usize, arg: &Term, recon: &Reconstructions) -> bool {
    let (head, args) = peel_app(arg);
    let Term::Constructor { id: ctor_id, .. } = head else {
        return false;
    };
    let n = args.len();
    for (j, a) in args.iter().enumerate() {
        let Term::Var(v) = a else {
            return false;
        };
        match recon_get(recon, *v) {
            Some(tag)
                if tag.param_idx == param_idx
                    && tag.ctor == ctor_id
                    && tag.n_fields == n
                    && tag.field_pos == j => {}
            _ => return false,
        }
    }
    true
}

/// Size relation of `arg` to parameter `param_idx` (`17 §4.2`).
///
/// Two ways an argument can relate to `param_idx`: it is exactly the bound
/// variable holding a known-related value (`prov`), or it is an **exact
/// reconstruction** of `param_idx`'s matched value via the same constructor
/// and positionally-raw fields (`recon`, `sct-reconstruction-descent`) — a
/// reconstruction is never strictly smaller, so it contributes `DownEq`
/// only, **never `Down`**.
fn size_rel(param_idx: usize, arg: &Term, prov: &Provenances, recon: &Reconstructions) -> SizeOrd {
    if let Term::Var(i) = arg {
        if let Some((p, ord)) = prov_get(prov, *i) {
            if p == param_idx {
                return ord;
            }
        }
    }
    if is_exact_reconstruction(param_idx, arg, recon) {
        return SizeOrd::DownEq;
    }
    SizeOrd::Unknown // constructor-wrapping (inexact), app, prim, cast are all ?
}

// ---------------------------------------------------------------------------
// Call extraction
// ---------------------------------------------------------------------------

struct CallEdge {
    caller: usize,
    callee: usize,
    matrix: ScMatrix,
}

/// Is constructor field type `arg_ty` a (direct or W-style Π-bound)
/// recursive position for family `fam`? Peels leading `Term::Pi` domains
/// and checks whether the remaining codomain is headed by `fam`.
///
/// `ConstructorDecl.recursive_positions` (`env.rs`) is **not** populated by
/// `declare_inductive` (`check.rs`) for any user-declared inductive —
/// always `Vec::new()`. This mirrors `ken-interp::eval::is_recursive_arg`,
/// which independently re-derives the same information from each field's
/// declared type for the identical reason (see its doc comment). SCT needs
/// this to know how many trailing IH `Lam` binders a constructor's
/// compiled method carries (one per recursive field, `sct-completeness`
/// #12) — trusting `recursive_positions` directly would silently under-
/// count every constructor's IH slots as zero.
fn is_recursive_field(arg_ty: &Term, fam: GlobalId) -> bool {
    match arg_ty {
        Term::IndFormer { id, .. } => *id == fam,
        Term::App(f, _) => is_recursive_field(f, fam),
        Term::Pi(_, cod) => is_recursive_field(cod, fam),
        _ => false,
    }
}

/// One not-yet-bound binder's provenance + reconstruction tag, queued in the
/// order the match-compiler's `Lam`s bind them. Bundled (rather than two
/// parallel queues) so a slot's `prov` and `recon` can never drift out of
/// sync as `enter_method` consumes the queue one binder at a time.
#[derive(Clone, Copy, Debug, Default)]
struct PendingSlot {
    prov: Option<(usize, SizeOrd)>,
    recon: Option<ReconTag>,
}

/// Ordered pending queue for one constructor's own arity: `n_fields` field
/// slots (each `field_prov`, and — iff the scrutinee's own param is known —
/// a [`ReconTag`] positionally identifying it as field `j` of `ctor`)
/// followed by `n_ihs` IH slots (always empty) — the order the
/// match-compiler binds them in.
fn ctor_pending(
    n_fields: usize,
    n_ihs: usize,
    field_prov: Option<(usize, SizeOrd)>,
    ctor: GlobalId,
) -> Vec<PendingSlot> {
    let param_idx = field_prov.map(|(pi, _)| pi);
    let mut v = Vec::with_capacity(n_fields + n_ihs);
    v.extend((0..n_fields).map(|field_pos| PendingSlot {
        prov: field_prov,
        recon: param_idx.map(|param_idx| ReconTag {
            param_idx,
            ctor,
            n_fields,
            field_pos,
        }),
    }));
    v.extend(std::iter::repeat(PendingSlot::default()).take(n_ihs));
    v
}

/// Peel a method body's leading `Lam` binders against `pending` (front =
/// next binder's provenance).
///
/// `pending` is *not* assumed to be a flat run of exactly `pending.len()`
/// leading lambdas: the match-compiler can interleave a **nested**
/// sub-pattern-split `Elim` before all of the current constructor's own
/// fields/IHs are bound (`sct-completeness` #12 — e.g. `Node (Node ll lc
/// lr) c r`, whose `Node` method binds only `l` before splitting on it,
/// deferring `c, r` and both IH slots into each branch of that split).
/// When that happens, `dispatch_elim_methods` recurses into the nested
/// split with the *remaining* `pending` threaded through as a
/// continuation, so every branch still binds the deferred slots — with
/// their true `field_prov`/`None` — after its own fresh arity, regardless
/// of nesting depth.
///
/// Only a bare `Term::Var` scrutinee is recognized as a genuine nested
/// split of the fields currently being bound (the match-compiler only
/// ever splits on an already-bound variable this way); anything else
/// falls back to the original flat-peel behavior — stop and hand off to
/// `collect_calls` generically, which pushes `None` for any further
/// binders. That fallback is the pre-existing, already-sound
/// under-assignment (over-rejection is the safe direction) — this
/// function only ever *adds* correctly-scoped `Down`/`None` assignments
/// it can positively justify, never guesses.
fn enter_method<'a>(
    mut term: &'a Term,
    mut pending: &[PendingSlot],
    prov: &Provenances,
    recon: &Reconstructions,
    caller_idx: usize,
    n_caller: usize,
    group: &[(GlobalId, usize)],
    env: &GlobalEnv,
    out: &mut Vec<CallEdge>,
) {
    let mut cur_prov = prov.clone();
    let mut cur_recon = recon.clone();
    loop {
        if pending.is_empty() {
            collect_calls(term, caller_idx, n_caller, group, &cur_prov, &cur_recon, env, out);
            return;
        }
        match term {
            Term::Lam(_, body) => {
                cur_prov = prov_push(&cur_prov, pending[0].prov);
                cur_recon = recon_push(&cur_recon, pending[0].recon);
                pending = &pending[1..];
                term = body;
            }
            Term::Elim { scrut, .. } if matches!(scrut.as_ref(), Term::Var(_)) => {
                dispatch_elim_methods(
                    term, caller_idx, n_caller, group, &cur_prov, &cur_recon, env, out, pending,
                );
                return;
            }
            _ => break,
        }
    }
    collect_calls(term, caller_idx, n_caller, group, &cur_prov, &cur_recon, env, out);
}

/// Shared `Term::Elim` dispatch: scan `params`/`motive`/`indices`/`scrut`
/// for calls, compute the scrutinee's field provenance, then recurse into
/// each constructor method with its own fresh arity queue followed by
/// `continuation` (the enclosing constructor's still-unbound fields/IHs —
/// empty at the top level, from `collect_calls`'s own `Elim` arm; nonempty
/// only when `enter_method` threads a nested split's remainder through).
fn dispatch_elim_methods(
    term: &Term,
    caller_idx: usize,
    n_caller: usize,
    group: &[(GlobalId, usize)],
    prov: &Provenances,
    recon: &Reconstructions,
    env: &GlobalEnv,
    out: &mut Vec<CallEdge>,
    continuation: &[PendingSlot],
) {
    let Term::Elim {
        fam,
        params,
        motive,
        methods,
        indices,
        scrut,
        ..
    } = term
    else {
        return;
    };
    for p in params {
        collect_calls(p, caller_idx, n_caller, group, prov, recon, env, out);
    }
    collect_calls(motive, caller_idx, n_caller, group, prov, recon, env, out);
    for ix in indices {
        collect_calls(ix, caller_idx, n_caller, group, prov, recon, env, out);
    }
    collect_calls(scrut, caller_idx, n_caller, group, prov, recon, env, out);

    let scrut_prov = match scrut.as_ref() {
        Term::Var(i) => prov_get(prov, *i),
        _ => None,
    };
    let field_prov = scrut_prov.map(|(pi, _)| (pi, SizeOrd::Down));

    if let Some(ind) = env.inductive(*fam) {
        for (k, method) in methods.iter().enumerate() {
            if k >= ind.constructors.len() {
                break;
            }
            let c = &ind.constructors[k];
            let n_fields = c.args.len();
            let n_ihs = c.args.iter().filter(|a| is_recursive_field(a, *fam)).count();
            let mut own_pending = ctor_pending(n_fields, n_ihs, field_prov, c.id);
            own_pending.extend_from_slice(continuation);
            enter_method(
                method, &own_pending, prov, recon, caller_idx, n_caller, group, env, out,
            );
        }
    } else {
        for method in methods {
            collect_calls(method, caller_idx, n_caller, group, prov, recon, env, out);
        }
    }
}

/// Traverse `term` collecting edges for all `Const` calls to group members.
fn collect_calls(
    term: &Term,
    caller_idx: usize,
    n_caller: usize,
    group: &[(GlobalId, usize)],
    prov: &Provenances,
    recon: &Reconstructions,
    env: &GlobalEnv,
    out: &mut Vec<CallEdge>,
) {
    match term {
        Term::App(_, _) => {
            let (head, args) = peel_app(term);
            let mut head_is_applied_group_call = false;
            if let Term::Const { id, .. } = &head {
                if let Some(callee_idx) = group.iter().position(|(gid, _)| gid == id) {
                    head_is_applied_group_call = true;
                    let n_callee = group[callee_idx].1;
                    let mut m = ScMatrix::zero(n_caller, n_callee);
                    for (j, arg) in args.iter().enumerate().take(n_callee) {
                        for i in 0..n_caller {
                            m.entries[i][j] = size_rel(i, arg, prov, recon);
                        }
                    }
                    out.push(CallEdge {
                        caller: caller_idx,
                        callee: callee_idx,
                        matrix: m,
                    });
                }
            }
            // Recurse into head — unless it's a group-member Const already
            // handled above as the applied call above: re-collecting it would
            // hit the bare-occurrence arm and push a second, spurious
            // ?-everywhere self-loop edge alongside the real one just computed
            // from `args`. Args always recurse (an argument position is never
            // "the same application").
            if !head_is_applied_group_call {
                collect_calls(&head, caller_idx, n_caller, group, prov, recon, env, out);
            }
            for arg in &args {
                collect_calls(arg, caller_idx, n_caller, group, prov, recon, env, out);
            }
        }
        Term::Lam(a, body) => {
            collect_calls(a, caller_idx, n_caller, group, prov, recon, env, out);
            let p2 = prov_push(prov, None);
            let r2 = recon_push(recon, None);
            collect_calls(body, caller_idx, n_caller, group, &p2, &r2, env, out);
        }
        Term::Pi(a, b) | Term::Sigma(a, b) => {
            collect_calls(a, caller_idx, n_caller, group, prov, recon, env, out);
            let p2 = prov_push(prov, None);
            let r2 = recon_push(recon, None);
            collect_calls(b, caller_idx, n_caller, group, &p2, &r2, env, out);
        }
        Term::Let { ty, val, body } => {
            collect_calls(ty, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(val, caller_idx, n_caller, group, prov, recon, env, out);
            let p2 = prov_push(prov, None);
            let r2 = recon_push(recon, None);
            collect_calls(body, caller_idx, n_caller, group, &p2, &r2, env, out);
        }
        Term::Elim { .. } => {
            dispatch_elim_methods(term, caller_idx, n_caller, group, prov, recon, env, out, &[]);
        }
        // Terms with no binders: recurse uniformly.
        Term::Pair(a, b) | Term::Ascript(a, b) | Term::Quot(a, b) => {
            collect_calls(a, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(b, caller_idx, n_caller, group, prov, recon, env, out);
        }
        Term::Proj1(p)
        | Term::Proj2(p)
        | Term::Refl(p)
        | Term::QuotClass(p)
        | Term::Trunc(p)
        | Term::TruncProj(p) => {
            collect_calls(p, caller_idx, n_caller, group, prov, recon, env, out);
        }
        Term::Eq(a, x, y) => {
            collect_calls(a, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(x, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(y, caller_idx, n_caller, group, prov, recon, env, out);
        }
        Term::Cast(a, b, e, t) => {
            collect_calls(a, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(b, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(e, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(t, caller_idx, n_caller, group, prov, recon, env, out);
        }
        Term::J(m, d, e) => {
            collect_calls(m, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(d, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(e, caller_idx, n_caller, group, prov, recon, env, out);
        }
        Term::QuotElim {
            motive,
            method,
            respect,
            scrut,
        } => {
            collect_calls(motive, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(method, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(respect, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(scrut, caller_idx, n_caller, group, prov, recon, env, out);
        }
        Term::Absurd(motive, proof) => {
            collect_calls(motive, caller_idx, n_caller, group, prov, recon, env, out);
            collect_calls(proof, caller_idx, n_caller, group, prov, recon, env, out);
        }
        Term::Const { id, .. } => {
            // A bare (unapplied) group-member occurrence: model as a
            // ?-everywhere self-loop. ScMatrix::zero = all SizeOrd::Unknown;
            // that matrix is idempotent with no strict diagonal, so
            // sct_check rejects. Bare self-references cannot be certified
            // terminating — use an eliminator.
            if let Some(callee_idx) = group.iter().position(|(gid, _)| gid == id) {
                let n_callee = group[callee_idx].1;
                out.push(CallEdge {
                    caller: caller_idx,
                    callee: callee_idx,
                    matrix: ScMatrix::zero(n_caller, n_callee),
                });
            }
        }
        // Leaves: no sub-terms with calls.
        Term::Var(_)
        | Term::Type(_)
        | Term::Omega(_)
        | Term::IndFormer { .. }
        | Term::Constructor { .. } => {}
    }
}

// ---------------------------------------------------------------------------
// Composition-set closure (correct SCT algorithm)
// ---------------------------------------------------------------------------

/// Compute the composition closure of the call-edge set and return every
/// distinct idempotent self-loop matrix.
///
/// The size-change principle (Ben-Amram / Lee–Jones) requires that **every**
/// idempotent matrix in the reachable composition set has a strict diagonal —
/// not just the element-wise max (union) over all paths.  The union
/// over-approximates: two distinct loops `M_A = [[↓]]` and `M_B = [[↓=]]`
/// union to `[[↓]]`, making the gate miss `M_B`'s lack of strict decrease and
/// wrongly admit a non-terminating definition.
///
/// This function keeps each distinct `(caller, callee, matrix)` triple
/// separately; the closure is closed under composition until no new triple
/// appears.  Idempotent self-loops are then collected without merging.
fn composition_closure_self_loops(edges: &[CallEdge]) -> Vec<ScMatrix> {
    // G* = full reachable set of (caller, callee, matrix) triples.
    let mut closure: Vec<(usize, usize, ScMatrix)> = Vec::new();

    // Seed from direct edges.
    for e in edges {
        let triple = (e.caller, e.callee, e.matrix.clone());
        if !closure.contains(&triple) {
            closure.push(triple);
        }
    }

    // Close under composition: each round works from a snapshot so that new
    // entries discovered this round are composed in the next round.
    loop {
        let snap = closure.clone();
        let mut added = false;
        for &(ci, cj, ref mi) in &snap {
            for &(cj2, ck, ref mj) in &snap {
                if cj != cj2 || mi.ncols != mj.nrows {
                    continue;
                }
                let composed = mi.compose(mj);
                let triple = (ci, ck, composed);
                if !closure.contains(&triple) {
                    closure.push(triple);
                    added = true;
                }
            }
        }
        if !added {
            break;
        }
    }

    // Collect all distinct idempotent self-loop matrices.
    closure
        .into_iter()
        .filter_map(|(i, k, m)| {
            if i == k && m.is_idempotent() {
                Some(m)
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Count the number of leading `Lam` binders.
pub fn count_params(body: &Term) -> usize {
    let mut n = 0;
    let mut cur = body;
    while let Term::Lam(_, b) = cur {
        n += 1;
        cur = b;
    }
    n
}

/// Skip `n` leading `Lam` binders, returning the inner body.
fn skip_lams(body: &Term, n: usize) -> &Term {
    let mut cur = body;
    for _ in 0..n {
        if let Term::Lam(_, b) = cur {
            cur = b;
        } else {
            break;
        }
    }
    cur
}

/// Build provenance for the outermost `n` lambda parameters.
/// `Var(0)` = innermost param = `param (n-1)`.
fn initial_prov(n: usize) -> Provenances {
    (0..n).map(|k| Some((n - 1 - k, SizeOrd::DownEq))).collect()
}

/// Build the (empty) reconstruction tags for the outermost `n` lambda
/// parameters — a group's own formal parameters are never reconstructions of
/// anything, only fields bound inside a match arm can be.
fn initial_recon(n: usize) -> Reconstructions {
    vec![None; n]
}

/// SCT gate: accept iff every idempotent self-loop has ≥1 `↓` on the diagonal.
///
/// `group_bodies` = `(id, body)` for each member of the mutually-recursive
/// group. Bodies must include their leading parameter lambdas. `env` must have
/// all group members pre-admitted (as opaque) so their IDs are visible.
pub fn sct_check(
    env: &GlobalEnv,
    group_bodies: &[(GlobalId, Term)],
) -> crate::error::KernelResult<()> {
    if group_bodies.is_empty() {
        return Ok(());
    }

    let group: Vec<(GlobalId, usize)> = group_bodies
        .iter()
        .map(|(id, body)| (*id, count_params(body)))
        .collect();

    let mut edges: Vec<CallEdge> = Vec::new();
    for (caller_idx, (_id, body)) in group_bodies.iter().enumerate() {
        let n = group[caller_idx].1;
        let inner = skip_lams(body, n);
        let prov = initial_prov(n);
        let recon = initial_recon(n);
        collect_calls(inner, caller_idx, n, &group, &prov, &recon, env, &mut edges);
    }

    if edges.is_empty() {
        return Ok(());
    } // non-recursive

    let self_loops = composition_closure_self_loops(&edges);

    for m in &self_loops {
        if m.is_idempotent() && !m.has_strict_diagonal() {
            return Err(crate::error::KernelError::NotTerminating(
                "SCT: idempotent self-loop has no strictly-decreasing parameter".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // `sct-reconstruction-descent` — direct unit tests of the load-bearing
    // `is_exact_reconstruction` predicate.
    //
    // AC3 requires the `DownEq`-only / exact-reconstruction boundary to be
    // *tested*, not just asserted. The REORDER/WRONG-FIELD/WRONG-CTOR near-
    // misses are pinned here directly against the predicate rather than as
    // full elaborated `.ken` programs: constructing a surface program that is
    // (a) genuinely non-terminating and (b) would *only* be wrongly accepted
    // by a positionally-blind version of this exact predicate turns out to be
    // nontrivial — a naive "swap two fields on the recursive call" surface
    // program (`f (m,n) = f (n-derived, m-derived)`) is easy to accidentally
    // make into a *real*, differently-shaped terminating descent (a
    // decreasing potential function on `m+n`), which is a legitimate accept,
    // not a discriminator at all. Testing the predicate directly is strictly
    // more precise for exactly this shape of question — it isolates the
    // mechanism from the surrounding SCT composition machinery, per
    // [[isolate-mechanism-from-orthogonal-fail-closed-gates]] — while
    // `badAck`/`badAck2`/`ack` (below, via the elaborator integration tests)
    // cover the end-to-end accept/reject behavior for the shapes that ARE
    // unambiguous at the surface level.
    // -----------------------------------------------------------------------

    const CTOR_A: GlobalId = GlobalId(9001);
    const CTOR_B: GlobalId = GlobalId(9002);

    /// `recon[i]` = tag for `Var(i)` directly (no reordering) — these tests
    /// exercise the predicate in isolation, not the real field-to-Var
    /// assignment `ctor_pending`/`enter_method` produce (that end-to-end
    /// wiring is covered by the elaborator-level `ack`/`badAck`/`badAck2`
    /// tests instead).
    fn recons(entries: Vec<Option<ReconTag>>) -> Reconstructions {
        entries
    }

    fn tag(param_idx: usize, ctor: GlobalId, n_fields: usize, field_pos: usize) -> ReconTag {
        ReconTag { param_idx, ctor, n_fields, field_pos }
    }

    #[test]
    fn is_exact_reconstruction_positive() {
        // `C x0 x1` reconstructing param 0's C-destructuring, exact position.
        let recon = recons(vec![Some(tag(0, CTOR_A, 2, 0)), Some(tag(0, CTOR_A, 2, 1))]);
        let arg = Term::app(Term::app(Term::constructor(CTOR_A, vec![]), Term::var(0)), Term::var(1));
        assert!(is_exact_reconstruction(0, &arg, &recon));
        // A different param's check must fail (recon targets param 0, not 1).
        assert!(!is_exact_reconstruction(1, &arg, &recon));
    }

    #[test]
    fn is_exact_reconstruction_reorder_rejected() {
        // `C x1 x0` — same fields, same ctor, POSITIONALLY SWAPPED.
        let recon = recons(vec![Some(tag(0, CTOR_A, 2, 0)), Some(tag(0, CTOR_A, 2, 1))]);
        let arg = Term::app(Term::app(Term::constructor(CTOR_A, vec![]), Term::var(1)), Term::var(0));
        assert!(
            !is_exact_reconstruction(0, &arg, &recon),
            "positional swap must not be treated as an exact reconstruction"
        );
    }

    #[test]
    fn is_exact_reconstruction_wrong_field_rejected() {
        // `C x0 z` — z is an in-scope var with NO tag (not a field of this
        // destructuring at all).
        let recon = recons(vec![Some(tag(0, CTOR_A, 2, 0)), Some(tag(0, CTOR_A, 2, 1)), None]);
        let arg = Term::app(Term::app(Term::constructor(CTOR_A, vec![]), Term::var(0)), Term::var(2));
        assert!(
            !is_exact_reconstruction(0, &arg, &recon),
            "substituting an untagged var must not be treated as an exact reconstruction"
        );
    }

    #[test]
    fn is_exact_reconstruction_wrong_ctor_rejected() {
        // `D x0 x1` — fields tagged for ctor A, reconstructed via ctor B.
        let recon = recons(vec![Some(tag(0, CTOR_A, 2, 0)), Some(tag(0, CTOR_A, 2, 1))]);
        let arg = Term::app(Term::app(Term::constructor(CTOR_B, vec![]), Term::var(0)), Term::var(1));
        assert!(
            !is_exact_reconstruction(0, &arg, &recon),
            "reconstructing via a different constructor must not be treated as an exact reconstruction"
        );
    }

    #[test]
    fn is_exact_reconstruction_structural_increase_rejected() {
        // `C (C x0 x1) x1` — arg 0 is itself a further application, not a
        // bare Var (badAck2's `Suc (Suc m2)` shape, generalized).
        let recon = recons(vec![Some(tag(0, CTOR_A, 2, 0)), Some(tag(0, CTOR_A, 2, 1))]);
        let inner = Term::app(Term::app(Term::constructor(CTOR_A, vec![]), Term::var(0)), Term::var(1));
        let arg = Term::app(Term::app(Term::constructor(CTOR_A, vec![]), inner), Term::var(1));
        assert!(
            !is_exact_reconstruction(0, &arg, &recon),
            "a net structural increase (non-bare-Var field) must not be treated as an exact reconstruction"
        );
    }

    #[test]
    fn is_exact_reconstruction_non_constructor_head_rejected() {
        // Applying a non-constructor head (e.g. a Const) is never a
        // reconstruction, regardless of its args' tags.
        let recon = recons(vec![Some(tag(0, CTOR_A, 1, 0))]);
        let arg = Term::app(Term::const_(GlobalId(1), vec![]), Term::var(0));
        assert!(!is_exact_reconstruction(0, &arg, &recon));
    }

    #[test]
    fn compose_table() {
        use SizeOrd::*;
        // Strict decrease dominates only when second step is not Unknown.
        assert_eq!(compose_ord(Down, Down), Down);
        assert_eq!(compose_ord(Down, DownEq), Down);
        assert_eq!(compose_ord(Down, Unknown), Unknown); // ? breaks thread
        assert_eq!(compose_ord(DownEq, Down), Down);
        assert_eq!(compose_ord(DownEq, DownEq), DownEq);
        assert_eq!(compose_ord(DownEq, Unknown), Unknown);
        assert_eq!(compose_ord(Unknown, Down), Unknown);
        assert_eq!(compose_ord(Unknown, DownEq), Unknown);
        assert_eq!(compose_ord(Unknown, Unknown), Unknown);
    }

    #[test]
    fn loop_self_loop_rejected() {
        // M = [[↓=]] — self-loop, no strict decrease.
        let m = ScMatrix {
            entries: vec![vec![SizeOrd::DownEq]],
            nrows: 1,
            ncols: 1,
        };
        assert!(m.is_idempotent());
        assert!(!m.has_strict_diagonal());
    }

    #[test]
    fn strict_self_loop_accepted() {
        let m = ScMatrix {
            entries: vec![vec![SizeOrd::Down]],
            nrows: 1,
            ncols: 1,
        };
        assert!(m.is_idempotent());
        assert!(m.has_strict_diagonal());
    }

    #[test]
    fn g_matrix_accepts() {
        // g(acc, n) → g(suc acc, n') where n' < n.
        // M = [[?, ?], [?, ↓]] — second param strictly decreases.
        use SizeOrd::*;
        let m = ScMatrix {
            entries: vec![vec![Unknown, Unknown], vec![Unknown, Down]],
            nrows: 2,
            ncols: 2,
        };
        assert!(m.is_idempotent());
        assert!(m.has_strict_diagonal());
    }

    #[test]
    fn ctor_wrap_compose_rejected() {
        // p→q: [[↓]], q→p: [[?]].
        // compose(↓, ?) = ? (per conformance seed) → self-loop [[?]] → REJECT.
        use SizeOrd::*;
        let m_pq = ScMatrix {
            entries: vec![vec![Down]],
            nrows: 1,
            ncols: 1,
        };
        let m_qp = ScMatrix {
            entries: vec![vec![Unknown]],
            nrows: 1,
            ncols: 1,
        };
        let composed = m_pq.compose(&m_qp); // p→q then q→p = p self-loop
        assert_eq!(composed.entries[0][0], Unknown);
        assert!(composed.is_idempotent());
        assert!(!composed.has_strict_diagonal());
    }

    #[test]
    fn union_masking_correctly_rejected() {
        // Architect counterexample: f has two distinct self-loops.
        //   M_A = [[↓]]  — structural call: strictly decreasing.
        //   M_B = [[↓=]] — stationary call: no strict decrease → idempotent, must REJECT.
        //
        // A union-based gate merges to [[↓]] and wrongly accepts.
        // The composition-set gate keeps M_B separate and rejects.
        use SizeOrd::*;
        let m_a = ScMatrix {
            entries: vec![vec![Down]],
            nrows: 1,
            ncols: 1,
        };
        let m_b = ScMatrix {
            entries: vec![vec![DownEq]],
            nrows: 1,
            ncols: 1,
        };
        assert!(m_a.is_idempotent() && m_b.is_idempotent());
        assert!(!m_b.has_strict_diagonal());

        // Verify the union would incorrectly mask M_B.
        assert_eq!(
            if Down > DownEq { Down } else { DownEq },
            Down // union = [[↓]], looks OK but hides M_B
        );

        let edges = vec![
            CallEdge {
                caller: 0,
                callee: 0,
                matrix: m_a,
            },
            CallEdge {
                caller: 0,
                callee: 0,
                matrix: m_b,
            },
        ];
        let loops = composition_closure_self_loops(&edges);
        assert!(
            loops.iter().any(|m| !m.has_strict_diagonal()),
            "M_B = [[↓=]] must survive as a distinct idempotent loop"
        );
    }
}
