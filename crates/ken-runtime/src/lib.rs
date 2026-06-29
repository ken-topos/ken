//! Ken runtime — production content-addressed value store (K3).
//!
//! Implements `spec/40-runtime/41-values.md` + `44-capacity.md` and
//! `docs/design/content-addressing.md` at production resolution.
//!
//! Differences from the F4 design-validation crate (`ken-foundation`):
//! - NFC string normalization at construction time (`41 §3a`, design doc §1.4)
//! - Space-scoped arena separation + reclamation (`44 §3`)
//! - Arena page chaining beyond a single flat Vec (`44 §1b`)
//! - `unknown` propagation (Kleene/Heyting logic, `41 §6`)

pub mod canonical;
pub mod hash;
pub mod store;
pub mod unknown;
pub mod values;

pub use canonical::Canonical;
pub use hash::fnv1a_64;
pub use store::{InternResult, Space, Store, StoreStats};
pub use unknown::Unknown;
pub use values::{Sign, Value};
