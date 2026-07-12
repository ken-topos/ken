//! Typeclass environment — registry for class/instance declarations (`33 §5`).
//!
//! This module is a **pure data layer**: no kernel calls, no elaboration.
//! All elaboration logic lives in `elab.rs` which has access to the private
//! `ElabCtx` type. This module exports only the data structures that `elab.rs`
//! populates and `ElabEnv` carries.

use crate::ast::DefKeyword;
use crate::error::Span;
use crate::resolve::RType;
use ken_kernel::{GlobalId, Term};
use std::collections::HashMap;

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
    /// Elaborated class-parameter kind. Absent means the class is nullary.
    pub param_kind: Option<Term>,
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
    /// Optional SURF-2 purity marker per field. This metadata is erased before
    /// kernel admission; it must stay parallel to `field_names`/`field_types`.
    pub field_purities: Vec<Option<DefKeyword>>,
    /// Kernel `GlobalId` of the class's Σ-record type (`C : Type → sort`).
    pub type_id: GlobalId,
    /// Whether this is a property or structure class (`33 §5.1`).
    pub kind: ClassKind,
    /// Module where this class was declared (for orphan check, `33 §5.3`).
    pub module_id: u32,
}

/// Per-instance metadata.
#[derive(Clone)]
pub struct InstanceInfo {
    /// Kernel `GlobalId` of the instance's Σ-record value.
    pub instance_id: GlobalId,
    /// Class this instance inhabits. Used only by surface projection purity.
    pub class_name: String,
    /// Inferred effect row for each instance field, in class-field order.
    pub field_effect_rows: Vec<crate::effects::RowType>,
    /// Module where this instance was declared (for orphan check).
    pub module_id: u32,
    /// Type parameters abstracted by the instance head, in source order.
    pub head_param_count: usize,
    /// The resolved instance-head pattern.  It selects which concrete type
    /// arguments are applied at a use site (e.g. `Pair a Bool` abstracts only
    /// `a`, not the fixed `Bool` slot).
    pub head_type: Option<RType>,
    /// Prerequisite dictionaries, retained so use-site resolution can build
    /// their recursive applications rather than merely returning the head id.
    pub constraints: Vec<InstanceConstraintInfo>,
    /// Dotted source-package path that defined this instance (N4 provenance).
    /// Direct, non-loader elaboration uses the stable `<local>` sentinel.
    pub defining_package: String,
    /// Original declaration span, retained so overlap reports both sites.
    pub declaration_span: Span,
}

/// Structured provenance emitted by a successful implicit resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceResolution {
    pub instance_id: GlobalId,
    pub class_name: String,
    pub head_type: String,
    pub defining_package: String,
}

/// A prerequisite dictionary required by a polymorphic instance.
#[derive(Clone)]
pub struct InstanceConstraintInfo {
    pub class_name: String,
    /// Surface-resolved shape, used to select the recursively required head.
    pub head_type: RType,
    /// Kernel type in the instance-head parameter context, used to close the
    /// instance Pi/lambda telescope and to kernel-check applications.
    pub core_type: Term,
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
    /// Package of the source unit currently elaborating through the N2 loader.
    pub current_package: Option<String>,
    /// Explicit direct-use roots of the active `program`/`package` boundary.
    /// `None` keeps direct, non-loader elaboration backward compatible.
    pub direct_use_packages: Option<std::collections::HashSet<String>>,
    /// Whether a boundary-less source closure may use its sole provider
    /// package without an explicit admission declaration.
    pub implicit_single_provider: bool,
    /// Distinct source packages which registered instances in this closure.
    pub source_instance_packages: std::collections::HashSet<String>,
    /// Successful implicit-resolution provenance in source order.
    pub resolution_provenance: Vec<InstanceResolution>,
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
            current_package: None,
            direct_use_packages: None,
            implicit_single_provider: false,
            source_instance_packages: std::collections::HashSet::new(),
            resolution_provenance: Vec::new(),
        }
    }

    /// Advance the module counter (call at module boundaries for orphan check).
    pub fn next_module(&mut self) {
        self.current_module += 1;
    }

    /// Look up the canonical instance for `(class_name, head_type_name)`.
    pub fn instance_search(&self, class_name: &str, head_name: &str) -> Option<GlobalId> {
        self.instances
            .get(&(class_name.to_string(), head_name.to_string()))
            .map(|i| i.instance_id)
    }
}
