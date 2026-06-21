//! # ken-kernel — the trusted kernel
//!
//! This crate is Ken's **trust root**: the small, permanent core that decides
//! whether a term is well-typed and whether a proof is valid. Everything a user
//! must trust to believe a Ken proof lives here, and nowhere else.
//!
//! Design constraints (see `../../02-strategy.md`, `../../docs/adr/`):
//! - **Small and auditable** (the de Bruijn criterion). Resist growth.
//! - **Correct from day one** — universe checking, dependent Sigma, and a
//!   decidable, termination-certified conversion. No `Type: Type`.
//! - **Permanent host = Rust.** The elaborator and codegen may self-host later;
//!   this kernel does not.
//!
//! Status: scaffold. The type theory (Pi, dependent Sigma, Id, J, universes) and
//! the proof checker land in work packages **K1** and **K2**.
#![forbid(unsafe_code)]

/// Crate version, surfaced for diagnostics.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
