//! Foundation crate — content-addressing design validation (F4).
//!
//! This is NOT the production content store (that is K3/X2). This is a
//! small-scale design validation that exercises the intern algorithm,
//! canonical encoding, FNV-1a hashing, and the store index patterns
//! described in `docs/design/content-addressing.md` and
//! `spec/40-runtime/41-values.md`, `spec/40-runtime/44-capacity.md`.
//!
//! The bench harness records: intern throughput, measured dedup rate vs
//! expected, memory per distinct value, O(1) equality (slot-id compare),
//! and loud-at-limit behaviour.

pub mod canonical;
pub mod hash;
pub mod store;
pub mod values;

pub use canonical::Canonical;
pub use hash::fnv1a_64;
pub use store::{InternResult, Store, StoreStats};
pub use values::Value;
