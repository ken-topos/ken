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

pub mod artifact_validation;
pub mod canonical;
pub mod cranelift_backend;
pub mod executable_artifact_contract;
pub mod executable_entrypoint_packaging;
pub mod hash;
pub mod ir;
pub mod native_execution_differential;
pub mod object_linker_packaging;
pub mod platform_runtime_support;
pub mod runtime_ir_evaluator;
pub mod store;
pub mod target_abi;
pub mod unknown;
pub mod values;

pub use artifact_validation::*;
pub use canonical::Canonical;
pub use cranelift_backend::*;
pub use executable_artifact_contract::*;
pub use executable_entrypoint_packaging::*;
pub use hash::fnv1a_64;
pub use ir::*;
pub use native_execution_differential::*;
pub use object_linker_packaging::*;
pub use platform_runtime_support::*;
pub use runtime_ir_evaluator::*;
pub use store::{InternResult, Space, Store, StoreStats};
pub use target_abi::*;
pub use unknown::Unknown;
pub use values::{Sign, Value};
