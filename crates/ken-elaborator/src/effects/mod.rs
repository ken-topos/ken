//! L5 effect discipline — row lattice, escape check, ITree denotation, row-poly.
//!
//! **L5-build** (`13fd2bf`): static row analysis, first-order.
//! - **`row`**: effect-row lattice (`§1.1`); also `RowVar`/`RowType`/`RowSubst`
//!   (row-polymorphism types, moved here as the foundation shared by all modules).
//! - **`algebra`**: set-level `⊕` row algebra (`§2.3`), `CapParam` (`§2.5`).
//! - **`infer`**: `infer_row` + `infer_all` (§1.2–1.3), `EffectDecl`.
//! - **`check`**: §1.4 escape, §2.5 capability, §4.4 cross-space, §5.2
//!   tail-resumptive, `check_higher_order_guard` (conservative guard).
//!
//! **L5-denotation** (this WP — K1.5 merged at `f037451`, gate lifted):
//! - **`itree`**: `ITree` (Ret/Vis), `perform`, `bind`, `handler_fold` (§2.1,
//!   §2.2, §5). K1.5 admitted ITree's W-style Vis + generated `elim_ITree`
//!   (`spec/10-kernel/14 §3.1`, AC5 in `k1p5_wstyle.rs`).
//! - **`row_poly`**: `infer_row_poly` + `check_row_poly_escape` (§1.2 row-poly).
//!   Propagates row variables symbolically for higher-order parameters, replacing
//!   the conservative `unknown_effectful_params` guard for new code.
//!
//! ## The §3.1 cross-workstream contract (now finalized)
//!
//! Locked interface for Sec1/Sec2/B1 downstream WPs:
//!
//! | Surface form | Kernel denotation | Read by |
//! |---|---|---|
//! | latent row `A →[ρ] B` | `Vis`-tags in `ITree ⟦ρ⟧ B` | escape check §1; B1 |
//! | capability `Cap E` | a value parameter (Π, §2.5) | Sec2 authority |
//! | IFC label `@ℓ` | label index on the `Vis` op/resp | Sec1 flow check |
//!
//! Three load-bearing guarantees (§3.1): manifest-in-the-type, every
//! authority-relevant act is a `Vis` node, discharge is visible (handlers only).
//!
//! ## Spec prose reconciled
//!
//! `36-effects.md §2.1` ("gated on K1.5") and `§7.0` ("L5 is gated on K1.5")
//! updated to "admitted as of K1.5 (`f037451`)". The `§6` deliverable note
//! updated to remove the K1.5-gating qualifier on ITree/bind/handlers.

pub mod algebra;
pub mod check;
pub mod extract;
pub mod infer;
pub mod itree;
pub mod lower;
pub mod row;
pub mod row_poly;

pub use algebra::{cap_set, row_join, CapParam};
pub use check::{
    check_capabilities, check_capabilities_no_handler, check_cross_space,
    check_decl, check_escape, check_higher_order_guard, check_tail_resumptive,
    CrossSpaceAccess, EffectError, ResumeKind, WitnessMap,
};
pub use infer::{infer_all, infer_row, EffectDecl};
pub use itree::{bind, handler_fold, perform, HandlerCase, ITree, Response, Value};
pub use row::{EffectName, EffectRow, RowSubst, RowType, RowVar};
pub use extract::{build_decl_with_extracted_params, extract_hof_params, RowVarAllocator};
pub use lower::{lower_bind, lower_elim_itree, lower_handler_fold_uniform};
pub use row_poly::{check_row_poly_escape, infer_row_poly};
