//! # ken-kernel — the trusted kernel
//!
//! Ken's **trust root**: the small, permanent Rust core that decides whether a
//! fully-explicit core term is well-typed and whether a proof is valid
//! (`spec/10-kernel/`). Everything a user must trust to believe a Ken proof
//! lives here, and nowhere else (`README §2`).
//!
//! ## K1 scope — the core dependent type theory (`spec/10-kernel/11`–`14`)
//!
//! K1 delivers the set-level MLTT core: de Bruijn syntax + the global
//! environment (`11`), predicative non-cumulative checked universes with **no
//! `Type : Type`** (`12`), dependent Π/Σ with β and η (`13`), inductive families
//! with the strict-positivity admission and a dependent eliminator + ι (`14`),
//! and just enough structural conversion (β/η/ι/δ) to check them. The
//! `[K2]`-reserved formers (`Ω`, `Eq`, `cast`, `J`, quotients, truncation) are
//! carried in the grammar for forward compatibility but rejected by
//! `check`/`infer`; the full decidable conversion (NbE + SCT) is K2c; the
//! stable API surface is K-api.
//!
//! ## Design constraints
//!
//! - **Small and auditable** — the de Bruijn criterion (`PRINCIPLES §5`).
//!   Resist growth.
//! - **Correct from day one** — universes are checked, Σ is genuinely
//!   dependent, and conversion terminates on the K1 fragment (`README §6`).
//! - **Permanent host = Rust**, `#![forbid(unsafe_code)]`.

#![forbid(unsafe_code)]

pub mod check;
pub mod conv;
pub mod env;
pub mod error;
pub mod inductive;
pub mod subst;
pub mod term;

// --- re-exports (the provisional internal entry points; stable API is K-api) ---
pub use check::{
    check, declare_def, declare_inductive, declare_postulate, declare_primitive, infer,
    raw_well_formed, CtorSpec, InductiveSpec,
};
pub use conv::{convert, convert_type, level_eq, normalize, whnf};
pub use env::{ConstructorDecl, Context, Decl, GlobalEnv, InductiveDecl, PrimReduction};
pub use error::{KernelError, KernelResult};
pub use term::{GlobalId, Level, LevelVar, Term};

/// Crate version, surfaced for diagnostics.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
