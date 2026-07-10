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
//! carried in the grammar for forward compatibility; the full decidable
//! conversion (NbE + SCT) is K2c; the stable API surface is K-api.
//!
//! ## K2 scope — the observational equality layer (`spec/10-kernel/15`–`16`)
//!
//! K2 makes the reserved formers type and compute (extending K1's
//! `check`/`infer`/`whnf`/`conv`, never rewriting them): the strict-proposition
//! universe **Ωₗ** (level-indexed, predicative, definitional proof-irrelevance,
//! `16 §1`), observational **`Eq`-by-type** (funext/propext definitional; the
//! inductive same-ctor conjunct with dependent-telescope `cast`s / diff-ctor ⇒
//! `Bottom`; `16 §2`), **`cast`-by-type** (regularity + by-type computation,
//! computing from endpoints never the proof; `16 §3`), the derived **`J`**
//! (J-β on `refl` **and** reduction on non-`refl` via `cast`; `15 §4`), set-
//! quotients `A/R` (`16 §5`), propositional truncation `‖A‖` (`16 §6`), and the
//! conversion extension — Ω-PI shortcut + propositional-argument-skip plugged
//! into K1's `convert` seam (`16 §8`). The reductions live in [`obs`]; the
//! typing rules in [`check`]; the seam in [`conv`].
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
pub mod obs;
pub mod sct;
pub mod subst;
pub mod term;

// --- re-exports (the provisional internal entry points; stable API is K-api) ---
pub use check::{
    check, declare_deceq_certificate, declare_def, declare_inductive, declare_postulate,
    declare_primitive, declare_recursive_group, infer, raw_well_formed, CtorSpec, InductiveSpec,
};
pub use conv::{convert, convert_type, level_eq, normalize, whnf};
pub use env::{ConstructorDecl, Context, Decl, DecEqCert, GlobalEnv, InductiveDecl, PrimReduction};
pub use error::{KernelError, KernelResult};
pub use term::{GlobalId, Level, LevelVar, Term};

/// Crate version, surfaced for diagnostics.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
