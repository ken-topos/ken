//! ITree denotation — the pure `Ret`/`Vis` tree (`36 §2.1`–`§2.2`, `§5`).
//!
//! Now buildable: K1.5 admitted `ITree`'s W-style `Vis` constructor and
//! generated `elim_ITree` (`kernel/tests/k1p5_wstyle.rs` AC5, `14 §3.1`).
//!
//! **Model.** The elaborator-level `ITree` mirrors the kernel inductive
//! `data ITree R where Ret : R → ITree R ; Vis : (Nat → ITree R) → ITree R`
//! (simplified: effect name as metadata, Nat-typed responses). The elaborator
//! uses this to verify denotation structure before emitting kernel terms; it is
//! not itself a kernel value.
//!
//! **Continuation representation.** `Vis`'s continuation `Nat → ITree R` is an
//! `Rc<dyn Fn(Response) → ITree>` — cloneable, capture-friendly, and sufficient
//! for the static analysis and conformance tests. Full kernel term generation
//! (lowering to `Term::Elim`) is the downstream WP.
//!
//! **Totality.** `bind` and `handler_fold` recurse on `cont(r)`, a strict sub-
//! tree (`elim_ITree` structural pattern, `14 §3`). Ken is total; this mirrors
//! the termination argument the kernel would check.

use std::rc::Rc;

use crate::ifc::{Label, BOTTOM};

use super::row::EffectName;

/// The response value type (models `Nat` from the kernel's simplified ITree).
pub type Response = u64;

/// The return value type (models the result type `R` of `ITree R`).
pub type Value = i64;

/// A shareable continuation `Response → ITree`.
pub type Cont = Rc<dyn Fn(Response) -> ITree>;

/// An interaction tree node (`36 §2.1`).
///
/// `ITree R` = `Ret (r : R)` | `Vis (e : E) (k : Nat → ITree R)`.
#[derive(Clone)]
pub enum ITree {
    /// `Ret r` — finished with value `r` (no effects performed).
    Ret(Value),
    /// `Vis e k` — perform effect `e`, then continue with `k` applied to the
    /// runtime's response. The continuation is a pure function into the tree.
    Vis {
        effect: EffectName,
        /// IFC index on the operation/response boundary.
        label: Label,
        cont: Cont,
    },
}

impl ITree {
    /// Construct `Ret r`.
    pub fn ret(v: Value) -> Self {
        Self::Ret(v)
    }

    /// Construct `Vis e k` from a closure.
    pub fn vis(
        effect: impl Into<EffectName>,
        cont: impl Fn(Response) -> ITree + 'static,
    ) -> Self {
        Self::vis_labeled(effect, BOTTOM, cont)
    }

    /// Construct a `Vis` carrying an IFC label index.
    pub fn vis_labeled(
        effect: impl Into<EffectName>,
        label: Label,
        cont: impl Fn(Response) -> ITree + 'static,
    ) -> Self {
        Self::Vis {
            effect: effect.into(),
            label,
            cont: Rc::new(cont),
        }
    }

    /// True iff this is a `Ret` node.
    pub fn is_ret(&self) -> bool {
        matches!(self, Self::Ret(_))
    }

    /// True iff this is a `Vis` node.
    pub fn is_vis(&self) -> bool {
        matches!(self, Self::Vis { .. })
    }

    /// The return value, if this is `Ret`.
    pub fn ret_value(&self) -> Option<Value> {
        match self {
            Self::Ret(v) => Some(*v),
            _ => None,
        }
    }

    /// The effect name, if this is `Vis`.
    pub fn effect_name(&self) -> Option<&EffectName> {
        match self {
            Self::Vis { effect, .. } => Some(effect),
            _ => None,
        }
    }

    /// The IFC label index, if this is a `Vis`.
    pub fn label(&self) -> Option<Label> {
        match self {
            Self::Vis { label, .. } => Some(*label),
            _ => None,
        }
    }

    /// Apply the continuation to a response, if this is `Vis`.
    pub fn apply_cont(&self, r: Response) -> Option<ITree> {
        match self {
            Self::Vis { cont, .. } => Some(cont(r)),
            _ => None,
        }
    }
}

// ----- §2.2 ret, perform, bind -----

/// `perform e = Vis e (λr. Ret r)` — the one-operation tree (§2.2).
pub fn perform(effect: impl Into<EffectName>) -> ITree {
    let e = effect.into();
    ITree::vis(e, |r| ITree::ret(r as Value))
}

/// `bind t f` — tree-grafting fold via `elim_ITree` (§2.2).
///
/// ```text
/// bind (Ret a)   f = f a
/// bind (Vis e k) f = Vis e (λr. bind (k r) f)
/// ```
///
/// Terminates: every recursive call is on `k r`, a strict sub-tree (`14 §3`).
pub fn bind(tree: ITree, f: Rc<dyn Fn(Value) -> ITree>) -> ITree {
    match tree {
        ITree::Ret(v) => f(v),
        ITree::Vis { effect, label, cont } => {
            let f2 = Rc::clone(&f);
            ITree::vis_labeled(effect, label, move |r| bind(cont(r), Rc::clone(&f2)))
        }
    }
}

/// Re-tag a tree along an effect-signature inclusion, preserving `Vis` labels.
pub fn incl(tree: ITree, inject: Rc<dyn Fn(EffectName) -> EffectName>) -> ITree {
    match tree {
        ITree::Ret(v) => ITree::ret(v),
        ITree::Vis { effect, label, cont } => {
            let inject2 = Rc::clone(&inject);
            ITree::vis_labeled(inject(effect), label, move |r| {
                incl(cont(r), Rc::clone(&inject2))
            })
        }
    }
}

// ----- §5 handlers — structural folds -----

/// A handler case: maps an effect to the response it provides (§5.1).
///
/// K1-buildable model: the handler provides a single `Response` value to the
/// continuation (tail-resumptive: continuation invoked at most once, §5.2).
/// The full `ops : E.Op → (E.Resp e → ITree F R') → ITree F R'` version
/// follows the same shape with an `Op`-dispatch step.
#[derive(Clone)]
pub struct HandlerCase {
    /// Which effect this case handles.
    pub effect: EffectName,
    /// The response the handler provides to the continuation.
    pub response: Response,
}

impl HandlerCase {
    pub fn new(effect: impl Into<EffectName>, response: Response) -> Self {
        Self { effect: effect.into(), response }
    }
}

/// Tail-resumptive handler fold over an ITree (§5.1).
///
/// ```text
/// handle ret ops (Ret r)         = ret r               -- Ret leaf
/// handle ret ops (Vis (inl e) k) = ops e k             -- handled E-op
/// handle ret ops (Vis (inr o) k) = Vis o (λr. handle ret ops (k r))  -- pass through
/// ```
///
/// Simplified: `ret = Ret`, `ops` = provide `case.response` to `k`.
/// Terminates: recurses only on `k(response)`, a strict subtree (`elim_ITree`
/// structural argument, `14 §3`). Tail-resumptive: `k` invoked at most once per
/// `Vis` node, in tail position (§5.2, OQ-9).
pub fn handler_fold(tree: ITree, cases: Rc<[HandlerCase]>) -> ITree {
    match tree {
        ITree::Ret(v) => ITree::ret(v),
        ITree::Vis { effect, label, cont } => {
            if let Some(case) = cases.iter().find(|c| c.effect == effect) {
                // Tail-resumptive: provide the response, continue the fold.
                let resp = case.response;
                handler_fold(cont(resp), Rc::clone(&cases))
            } else {
                // Unhandled effect: re-emit the Vis node; fold recursively.
                let cases2 = Rc::clone(&cases);
                ITree::vis_labeled(effect, label, move |r| {
                    handler_fold(cont(r), Rc::clone(&cases2))
                })
            }
        }
    }
}
