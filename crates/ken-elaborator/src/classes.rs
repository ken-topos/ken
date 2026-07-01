//! Typeclass environment — registry for class/instance declarations (`33 §5`).
//!
//! This module is a **pure data layer**: no kernel calls, no elaboration.
//! All elaboration logic lives in `elab.rs` which has access to the private
//! `ElabCtx` type. This module exports only the data structures that `elab.rs`
//! populates and `ElabEnv` carries.

use std::collections::HashMap;
use ken_kernel::{GlobalId, Term};

/// Whether a class is a property class (Ω-sorted Σ-chain, coherence-free via
/// Ω-PI) or a structure class (Type-sorted, canonical-one-per-head rule).
/// Determined at declaration time by the kernel's `sort_sigma` (`check.rs:192`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassKind {
    /// All fields are Ω-sorted — the class is a proposition.
    /// Multiple instances for the same head-type are fine (Ω-PI: all are
    /// definitionally equal), so the overlap check is skipped.
    Property,
    /// At least one field is Type-sorted — the class carries computational
    /// content. Exactly one canonical instance per `(class, head-type)` key is
    /// enforced by the overlap check.
    Structure,
}

/// Per-class metadata registered at declaration time.
pub struct ClassInfo {
    /// The optional single type-parameter name (e.g. `A` in `class Eq A`).
    pub param: Option<String>,
    /// Field names in declaration order.
    pub field_names: Vec<String>,
    /// Field types in declaration order — a real Σ-telescope (`33 §5.2`):
    /// `field_types[i]` is a kernel `Term` valid in context
    /// `[a?, field_types[0], …, field_types[i-1]]` (the class param, if
    /// any, then every EARLIER field's type, outermost/highest-index
    /// first). Used to compute each instance field's properly-substituted
    /// expected type (`ken_kernel::subst::subst_tel`) and to drive `.field`
    /// projection (`elab.rs`'s `RExpr::RProj`).
    pub field_types: Vec<Term>,
    /// Kernel `GlobalId` of the class's Σ-record type (`C : Type → sort`).
    pub type_id: GlobalId,
    /// Whether this is a property or structure class (`33 §5.1`).
    pub kind: ClassKind,
    /// Module where this class was declared (for orphan check, `33 §5.3`).
    pub module_id: u32,
}

/// Per-instance metadata.
pub struct InstanceInfo {
    /// Kernel `GlobalId` of the instance's Σ-record value.
    pub instance_id: GlobalId,
    /// Module where this instance was declared (for orphan check).
    pub module_id: u32,
}

/// The typeclass environment: class registry, canonical instance registry,
/// structural postulate IDs, and per-module tracking for the orphan check.
pub struct ClassEnv {
    pub classes: HashMap<String, ClassInfo>,
    /// Canonical instances: `(class_name, head_type_name)` → `InstanceInfo`.
    /// For property classes this may hold multiple under different keys, but
    /// only one per `(class, head)` pair (property instances on the same head
    /// are accepted by Ω-PI, so no duplicate-key registration occurs — each
    /// instance is still a distinct value; the property check just waives the
    /// overlap error at the *second* registration).
    pub instances: HashMap<(String, String), InstanceInfo>,
    /// `RecordNil : Omega 0` — the Σ-chain prop terminator.
    pub record_nil_id: GlobalId,
    /// `record_nil_val : RecordNil` — the unique inhabitant.
    pub record_nil_val_id: GlobalId,
    /// Current module counter (bumped at module boundaries).
    pub current_module: u32,
    /// Maps each `GlobalId` to the module where it was declared.
    pub global_modules: HashMap<GlobalId, u32>,
}

impl ClassEnv {
    /// Create a sentinel `ClassEnv` with placeholder IDs. Use only when the
    /// class machinery is not needed (non-class elaboration paths). The real
    /// `ClassEnv` is created by `elab::init_class_env` which pre-declares the
    /// structural postulates in the kernel.
    pub fn sentinel() -> Self {
        ClassEnv {
            classes: HashMap::new(),
            instances: HashMap::new(),
            record_nil_id: GlobalId(0),
            record_nil_val_id: GlobalId(0),
            current_module: 0,
            global_modules: HashMap::new(),
        }
    }

    /// Advance the module counter (call at module boundaries for orphan check).
    pub fn next_module(&mut self) {
        self.current_module += 1;
    }

    /// Look up the canonical instance for `(class_name, head_type_name)`.
    pub fn instance_search(
        &self,
        class_name: &str,
        head_name: &str,
    ) -> Option<GlobalId> {
        self.instances
            .get(&(class_name.to_string(), head_name.to_string()))
            .map(|i| i.instance_id)
    }
}
