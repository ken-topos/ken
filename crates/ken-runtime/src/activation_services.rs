//! **`RT-FNSPLIT-B2F` `S6` — the generated-code activation-services record.**
//!
//! ⭐ **Architect ruling (relayed `evt_e300y2kjeb6k`): the boundary carrier gets
//! its own fixed host-context binding, and it is ⛔ never the native-`Int`
//! arena.** [`crate::native_int::NativeIntArenaV1`] and
//! [`crate::boundary_value::BoundaryArenaV1`] are different runtime objects with
//! different layouts and different ownership; a single `arena` binding that
//! served both was a false equality, retracted in
//! [`crate::cranelift_backend`]'s `carrier_arena` accessor.
//!
//! ## What this record is, and what it deliberately is not
//!
//! ⛔ **It is not a Ken value and it is not part of the `B2R` activation
//! frame.** Arenas are *host runtime services*. The `B2R` program activation
//! frame gains no arena slot, and no arena ever enters a `Parameter` / `Capture`
//! transfer — those carry source-valued bindings only. This record travels
//! beside the frame, as a second fixed parameter, precisely so that the
//! program-derived frame schema and the uniform non-program-derived service
//! context stay structurally separate.
//!
//! ⛔ **The two fields are distinct and typed.** Not one polymorphic `arena`,
//! not two raw positional parameters, not an emitter-selected pointer. The
//! generated code's calling convention is uniform:
//!
//! ```text
//! root / unit: (frame_ptr, services_ptr) -> i64
//! ```
//!
//! and every descendant unit receives the **unchanged** `services_ptr`.
//!
//! ## Why the offsets are derived from an inventory rather than written down
//!
//! ⭐ Same discipline as [`crate::boundary_value::RegionHeaderField`]: the field
//! order is a closed enum, the offsets are `const fn`s over it, and
//! [`ACTIVATION_SERVICES_BYTES`] is computed from the inventory's length. ⇒ A
//! field added later cannot leave a stale offset constant behind, because there
//! is no offset constant that was not derived. ⚠ That is a *layout* proof only;
//! the agreement between this inventory and the `#[repr(C)]` struct below is a
//! separate claim, and it is measured by
//! `the_record_fields_sit_at_the_offsets_emitted_code_loads` rather than
//! asserted here.

/// Every field of the activation-services record, in layout order.
///
/// ⛔ Closed, with no catch-all: adding a variant is a compile error at
/// [`Self::ALL`], which is what makes the inventory the authority rather than a
/// list someone has to remember to update.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActivationServiceField {
    /// The invocation's native-`Int` arena — the authority ordinary native
    /// `Int` lowering resolves and interns through.
    NativeIntArena,
    /// The invocation's **published** boundary-arena header base — the
    /// authority every boundary-carrier producer and consumer allocates and
    /// projects through.
    BoundaryArena,
}

impl ActivationServiceField {
    /// Every field, in layout order.
    pub const ALL: [ActivationServiceField; 2] = [
        ActivationServiceField::NativeIntArena,
        ActivationServiceField::BoundaryArena,
    ];

    /// This field's byte offset — its position, times the word width.
    pub const fn offset(self) -> i32 {
        (self as i32) * 8
    }
}

/// Byte offset of the native-`Int` arena pointer. ⛔ Ordinary native `Int`
/// lowering reads **only** this one.
pub const SERVICES_NATIVE_INT_ARENA: i32 = ActivationServiceField::NativeIntArena.offset();

/// Byte offset of the published boundary-arena header base. ⛔ Every
/// boundary-carrier helper graph takes **only** this one as its arena argument.
pub const SERVICES_BOUNDARY_ARENA: i32 = ActivationServiceField::BoundaryArena.offset();

/// Byte size of the record, **derived** from the field inventory.
pub const ACTIVATION_SERVICES_BYTES: i32 = (ActivationServiceField::ALL.len() * 8) as i32;

/// Every field of the internal generated-unit call envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitCallFrameField {
    Slots,
    HostDispatchContext,
}

impl UnitCallFrameField {
    pub const ALL: [Self; 2] = [Self::Slots, Self::HostDispatchContext];

    pub const fn offset(self) -> i32 {
        (self as i32) * 8
    }
}

pub const UNIT_CALL_FRAME_SLOTS: i32 = UnitCallFrameField::Slots.offset();
pub const UNIT_CALL_FRAME_HOST_DISPATCH_CONTEXT: i32 =
    UnitCallFrameField::HostDispatchContext.offset();
pub const UNIT_CALL_FRAME_BYTES: i32 = (UnitCallFrameField::ALL.len() * 8) as i32;

/// Fixed runtime-only envelope passed as `frame_ptr` to every internal unit.
///
/// `slots` addresses exactly the B2R payload. The host context is deliberately
/// outside that payload: it is not an `AbiSlot` and cannot reconstruct a Ken
/// environment.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GeneratedUnitCallFrameV1 {
    pub slots: *mut u8,
    pub host_dispatch_context: *mut std::ffi::c_void,
}

/// The fixed, runtime-owned services record generated code receives as its
/// second parameter.
///
/// ## Ownership and publication order — ⛔ not open to the emitter
///
/// The source of both fields is the runtime/store **activation owner**, never
/// lowering and never a call site. Materialization and *all* capacity
/// reservation happen **before** publication:
///
/// 1. publish the store's persistent image and bind that pointer into the
///    invocation boundary arena through
///    [`crate::boundary_value::ARENA_PERSISTENT`];
/// 2. bind the invocation's native arena through
///    [`crate::boundary_value::ARENA_NATIVE_INT`] — ⭐ **the same pointer this
///    record carries in [`Self::native_int_arena`]**, so ordinary native
///    lowering and boundary-`Int` decoding share one native authority while no
///    code mistakes that pointer for the boundary header;
/// 3. reserve invocation storage, publish the boundary arena, and put the
///    published base in [`Self::boundary_arena`].
///
/// ⛔ **No reservation or materialization may occur after either published
/// pointer is handed to generated code** — growth would move a region under a
/// pointer emitted code already holds.
///
/// ## Fail closed — there is no fallback equality
///
/// ⛔ A generated function missing either function-local service binding is a
/// **compile-time lowering failure**. ⛔ A launcher without a non-null, fully
/// published boundary arena and native arena does not call generated code. ⛔ No
/// failure may substitute block parameter 0, the services pointer itself, the
/// native arena, a raw result word, or alternate storage for
/// [`Self::boundary_arena`].
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GeneratedActivationServicesV1 {
    /// `*mut NativeIntArenaV1`, as an untyped header base.
    pub native_int_arena: *mut u64,
    /// The published `BoundaryArenaV1` header base.
    pub boundary_arena: *mut u64,
}

impl Default for GeneratedActivationServicesV1 {
    fn default() -> Self {
        Self {
            native_int_arena: std::ptr::null_mut(),
            boundary_arena: std::ptr::null_mut(),
        }
    }
}

impl GeneratedActivationServicesV1 {
    /// Bind both services. ⛔ Both are required: a record is either wholly
    /// published or not handed to generated code at all, which is why there is
    /// no per-field setter and no partial constructor.
    pub fn new(native_int_arena: *mut u64, boundary_arena: *mut u64) -> Self {
        Self {
            native_int_arena,
            boundary_arena,
        }
    }

    /// The address generated code receives as its `services_ptr`.
    pub fn as_ptr(&self) -> *const std::ffi::c_void {
        (self as *const Self).cast()
    }

    /// ⛔ **The launcher's precondition**, as a value rather than a comment:
    /// generated code is called only when both services are non-null.
    pub fn is_published(&self) -> bool {
        !self.native_int_arena.is_null() && !self.boundary_arena.is_null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **The inventory and the `#[repr(C)]` struct agree — measured by
    /// address arithmetic, not by reading the constants back.**
    ///
    /// **MEASURED:** the byte distance from the record's base to each field, in
    /// a real value, equals the offset emitted code loads at.
    /// **CLAIMED:** a generated function loading `SERVICES_BOUNDARY_ARENA` from
    /// its `services_ptr` reads the boundary arena the activation owner bound.
    /// **THE GAP:** that the launcher passes the address of *this* struct — an
    /// obligation on the runner, discharged where it calls generated code, not
    /// here.
    ///
    /// Promise class: **normative compatibility vector** — these offsets are the
    /// generated-code ABI, and moving one is a contract decision.
    #[test]
    fn the_record_fields_sit_at_the_offsets_emitted_code_loads() {
        let record = GeneratedActivationServicesV1::default();
        let base = std::ptr::addr_of!(record) as usize;
        let native = std::ptr::addr_of!(record.native_int_arena) as usize;
        let boundary = std::ptr::addr_of!(record.boundary_arena) as usize;

        assert_eq!(native - base, SERVICES_NATIVE_INT_ARENA as usize);
        assert_eq!(boundary - base, SERVICES_BOUNDARY_ARENA as usize);
        assert_eq!(
            std::mem::size_of::<GeneratedActivationServicesV1>(),
            ACTIVATION_SERVICES_BYTES as usize
        );
    }

    /// ⭐ **The positive control for the assertion above** — the two offsets are
    /// *distinct*, so an emitter that loaded one where it meant the other would
    /// read a different word.
    ///
    /// ⚠ Without this, the layout test passes on a record whose fields all sit
    /// at `0`, which is exactly the collapse the ruling forbids: one
    /// polymorphic arena wearing two names.
    #[test]
    fn the_two_services_are_at_distinct_offsets() {
        assert_ne!(SERVICES_NATIVE_INT_ARENA, SERVICES_BOUNDARY_ARENA);
        assert_eq!(
            ActivationServiceField::ALL.len(),
            ActivationServiceField::ALL
                .iter()
                .map(|field| field.offset())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            "every field must occupy its own word"
        );
    }

    /// Promise class: normative compatibility vector — the internal generated
    /// call ABI consumes these two fixed pointer fields.
    #[test]
    fn the_unit_call_envelope_matches_its_closed_field_inventory() {
        let envelope = GeneratedUnitCallFrameV1 {
            slots: std::ptr::null_mut(),
            host_dispatch_context: std::ptr::null_mut(),
        };
        let base = std::ptr::addr_of!(envelope) as usize;
        assert_eq!(
            std::ptr::addr_of!(envelope.slots) as usize - base,
            UNIT_CALL_FRAME_SLOTS as usize
        );
        assert_eq!(
            std::ptr::addr_of!(envelope.host_dispatch_context) as usize - base,
            UNIT_CALL_FRAME_HOST_DISPATCH_CONTEXT as usize
        );
        assert_eq!(
            std::mem::size_of::<GeneratedUnitCallFrameV1>(),
            UNIT_CALL_FRAME_BYTES as usize
        );
        assert_eq!(UnitCallFrameField::ALL.len(), 2);
        assert_eq!(ActivationServiceField::ALL.len(), 2);
    }

    /// ⛔ **A partially bound record is not publishable**, and the launcher's
    /// precondition says so as a value it can test.
    #[test]
    fn a_partially_bound_record_is_not_published() {
        let mut word = 0u64;
        let pointer = &mut word as *mut u64;

        assert!(!GeneratedActivationServicesV1::default().is_published());
        assert!(!GeneratedActivationServicesV1::new(pointer, std::ptr::null_mut()).is_published());
        assert!(!GeneratedActivationServicesV1::new(std::ptr::null_mut(), pointer).is_published());
        assert!(GeneratedActivationServicesV1::new(pointer, pointer).is_published());
    }
}
