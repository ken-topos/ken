//! L5 effect discipline — K1-buildable half (`36 §7.0`).
//!
//! Implements the static analyses over the surface AST that are independent of
//! `ITree` admittance (the K1.5-gated part). Specifically:
//!
//! - **`row`**: the effect-row lattice (`§1.1`): `EffectRow` as a finite set of
//!   named effects, with join (∪) and subset (⊆) operations.
//! - **`algebra`**: the set-level `⊕` row algebra (`§2.3`) and `CapParam` (`§2.5`).
//! - **`infer`**: `infer_row` (transitive inference, §1.2) and `infer_all`
//!   (call-graph least-fixpoint, §1.3).
//! - **`check`**: the `§1.4` escape gate (`ρ_inf ⊆ ρ_decl`), the `§2.5`
//!   capability-presence gate, the `§4.4` cross-space-alias gate, the `§5.2`
//!   tail-resumptive-handler gate, and the **higher-order row release guard**
//!   (`check_higher_order_guard`).
//!
//! ## Scope boundary: first-order only
//!
//! This module is **first-order**: `infer_row` resolves callee rows by name
//! (§1.2 `f a` clause where `f` is a named global). A higher-order parameter
//! `f : A →[ρ] B` carries a latent **row variable** `ρ` that cannot be resolved
//! without row-polymorphism (row unification + substitution). That machinery is
//! deferred to the K1.5-denotation follow-on WP.
//!
//! **Conservative safety valve:** `EffectDecl::unknown_effectful_params` lets a
//! caller declare the *candidate* effects a higher-order parameter might release.
//! `check_higher_order_guard` conservatively rejects if the declared row does not
//! cover them (§1.4 `⊆`). This prevents silent under-inference for higher-order
//! code until full row-polymorphism lands.
//!
//! **K1.5-deferred (NOT in this module):** the `ITree` datatype, `bind`,
//! `perform`, handlers/`runState`, the denotation `⟦·⟧`, the `§3.1` contract
//! realization, and row-polymorphism (row variables + latent-row propagation
//! across higher-order parameters). Those land after K1.5 admits Π-bound
//! recursive inductives (`§7.0`).

pub mod algebra;
pub mod check;
pub mod infer;
pub mod row;

pub use algebra::{cap_set, row_join, CapParam};
pub use check::{
    check_capabilities, check_capabilities_no_handler, check_cross_space,
    check_decl, check_escape, check_higher_order_guard, check_tail_resumptive,
    CrossSpaceAccess, EffectError, ResumeKind, WitnessMap,
};
pub use infer::{infer_all, infer_row, EffectDecl};
pub use row::{EffectName, EffectRow};
