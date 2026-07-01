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

/// Size relation of `arg` to parameter `param_idx` (`17 §4.2`).
fn size_rel(param_idx: usize, arg: &Term, prov: &Provenances) -> SizeOrd {
    if let Term::Var(i) = arg {
        if let Some((p, ord)) = prov_get(prov, *i) {
            if p == param_idx {
                return ord;
            }
        }
    }
    SizeOrd::Unknown // constructor-wrapping, app, prim, cast are all ?
}

// ---------------------------------------------------------------------------
// Call extraction
// ---------------------------------------------------------------------------

struct CallEdge {
    caller: usize,
    callee: usize,
    matrix: ScMatrix,
}

/// Peel exactly `n` leading `Lam` binders, assigning provenances.
/// Fields (first `n_fields` binders) get `field_prov`; IHs get `None`.
fn enter_method<'a>(
    mut term: &'a Term,
    n_fields: usize,
    n_ihs: usize,
    field_prov: Option<(usize, SizeOrd)>,
    prov: &Provenances,
    caller_idx: usize,
    n_caller: usize,
    group: &[(GlobalId, usize)],
    env: &GlobalEnv,
    out: &mut Vec<CallEdge>,
) {
    let mut cur_prov = prov.clone();
    let n_total = n_fields + n_ihs;
    for i in 0..n_total {
        match term {
            Term::Lam(_, body) => {
                let entry = if i < n_fields { field_prov } else { None };
                cur_prov = prov_push(&cur_prov, entry);
                term = body;
            }
            _ => break,
        }
    }
    collect_calls(term, caller_idx, n_caller, group, &cur_prov, env, out);
}

/// Traverse `term` collecting edges for all `Const` calls to group members.
fn collect_calls(
    term: &Term,
    caller_idx: usize,
    n_caller: usize,
    group: &[(GlobalId, usize)],
    prov: &Provenances,
    env: &GlobalEnv,
    out: &mut Vec<CallEdge>,
) {
    match term {
        Term::App(_, _) => {
            let (head, args) = peel_app(term);
            if let Term::Const { id, .. } = &head {
                if let Some(callee_idx) = group.iter().position(|(gid, _)| gid == id) {
                    let n_callee = group[callee_idx].1;
                    let mut m = ScMatrix::zero(n_caller, n_callee);
                    for (j, arg) in args.iter().enumerate().take(n_callee) {
                        for i in 0..n_caller {
                            m.entries[i][j] = size_rel(i, arg, prov);
                        }
                    }
                    out.push(CallEdge {
                        caller: caller_idx,
                        callee: callee_idx,
                        matrix: m,
                    });
                }
            }
            // Recurse into head and all args.
            collect_calls(&head, caller_idx, n_caller, group, prov, env, out);
            for arg in &args {
                collect_calls(arg, caller_idx, n_caller, group, prov, env, out);
            }
        }
        Term::Lam(a, body) => {
            collect_calls(a, caller_idx, n_caller, group, prov, env, out);
            let p2 = prov_push(prov, None);
            collect_calls(body, caller_idx, n_caller, group, &p2, env, out);
        }
        Term::Pi(a, b) | Term::Sigma(a, b) => {
            collect_calls(a, caller_idx, n_caller, group, prov, env, out);
            let p2 = prov_push(prov, None);
            collect_calls(b, caller_idx, n_caller, group, &p2, env, out);
        }
        Term::Let { ty, val, body } => {
            collect_calls(ty, caller_idx, n_caller, group, prov, env, out);
            collect_calls(val, caller_idx, n_caller, group, prov, env, out);
            let p2 = prov_push(prov, None);
            collect_calls(body, caller_idx, n_caller, group, &p2, env, out);
        }
        Term::Elim {
            fam,
            params,
            motive,
            methods,
            indices,
            scrut,
            ..
        } => {
            for p in params {
                collect_calls(p, caller_idx, n_caller, group, prov, env, out);
            }
            collect_calls(motive, caller_idx, n_caller, group, prov, env, out);
            for ix in indices {
                collect_calls(ix, caller_idx, n_caller, group, prov, env, out);
            }
            collect_calls(scrut, caller_idx, n_caller, group, prov, env, out);

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
                    let n_ihs = c.recursive_positions.len();
                    enter_method(
                        method, n_fields, n_ihs, field_prov, prov, caller_idx, n_caller, group,
                        env, out,
                    );
                }
            } else {
                for method in methods {
                    collect_calls(method, caller_idx, n_caller, group, prov, env, out);
                }
            }
        }
        // Terms with no binders: recurse uniformly.
        Term::Pair(a, b) | Term::Ascript(a, b) | Term::Quot(a, b) => {
            collect_calls(a, caller_idx, n_caller, group, prov, env, out);
            collect_calls(b, caller_idx, n_caller, group, prov, env, out);
        }
        Term::Proj1(p)
        | Term::Proj2(p)
        | Term::Refl(p)
        | Term::QuotClass(p)
        | Term::Trunc(p)
        | Term::TruncProj(p) => {
            collect_calls(p, caller_idx, n_caller, group, prov, env, out);
        }
        Term::Eq(a, x, y) => {
            collect_calls(a, caller_idx, n_caller, group, prov, env, out);
            collect_calls(x, caller_idx, n_caller, group, prov, env, out);
            collect_calls(y, caller_idx, n_caller, group, prov, env, out);
        }
        Term::Cast(a, b, e, t) => {
            collect_calls(a, caller_idx, n_caller, group, prov, env, out);
            collect_calls(b, caller_idx, n_caller, group, prov, env, out);
            collect_calls(e, caller_idx, n_caller, group, prov, env, out);
            collect_calls(t, caller_idx, n_caller, group, prov, env, out);
        }
        Term::J(m, d, e) => {
            collect_calls(m, caller_idx, n_caller, group, prov, env, out);
            collect_calls(d, caller_idx, n_caller, group, prov, env, out);
            collect_calls(e, caller_idx, n_caller, group, prov, env, out);
        }
        Term::QuotElim {
            motive,
            method,
            respect,
            scrut,
        } => {
            collect_calls(motive, caller_idx, n_caller, group, prov, env, out);
            collect_calls(method, caller_idx, n_caller, group, prov, env, out);
            collect_calls(respect, caller_idx, n_caller, group, prov, env, out);
            collect_calls(scrut, caller_idx, n_caller, group, prov, env, out);
        }
        // Leaves: no sub-terms with calls.
        Term::Var(_)
        | Term::Type(_)
        | Term::Omega(_)
        | Term::Const { .. }
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
        collect_calls(inner, caller_idx, n, &group, &prov, env, &mut edges);
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
