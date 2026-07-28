//! **`RT-FNSPLIT-C3-ACTIVATION` `D3` — the Rust-owned activation and its
//! lifecycle.**
//!
//! ⭐ **The lifecycle verbs already existed; the OWNER did not.**
//! `reserve` / `bind_*` / `publish` / `seal_persistent` / `publish_persistent` /
//! `adopt` are landed definitions with **no production caller** — measured, and
//! it is why this node exists. ⇒ What this module supplies is the **caller and
//! its lifetime**, which is the part a definition cannot express: ordering,
//! ownership and teardown.
//!
//! ## The ruled order, across TWO scopes
//!
//! ⭐ The order spans two lifetimes, and splitting it that way is the thing this
//! module got wrong first — see [`BoundaryStoreBindingV1`].
//!
//! **Once per store** ([`BoundaryStoreBindingV1::open`]):
//!
//! 1. **reserve** the persistent image from the profile's persistent limits;
//! 2. **publish** the persistent image.
//!
//! **Once per activation** ([`BoundaryActivationV1::begin`]):
//!
//! 3. **bind** the published persistent base through [`ARENA_PERSISTENT`];
//! 4. **bind** this invocation's native-`Int` arena through
//!    [`ARENA_NATIVE_INT`] — ⭐ the *same* pointer the services record carries,
//!    so ordinary native lowering and boundary-`Int` decoding share one native
//!    authority while no code mistakes it for the boundary header;
//! 5. **reserve** invocation storage from the profile's invocation limits;
//! 6. **publish** the arena and put that base in the services record.
//!
//! ⛔ **No reservation or materialization after either published pointer
//! exists** — growth moves a region's tables under a pointer generated code
//! already holds. That is enforced structurally: each reservation happens inside
//! the one constructor for its scope, and neither type exposes a reserve
//! afterwards.
//!
//! ## ⛔ Why the owned objects are boxed
//!
//! ⚠ [`NativeIntArenaV1`] is a plain `#[repr(C)]` struct, so a pointer to it
//! points **into** the value. Handing that pointer out and then *moving* the
//! activation would leave generated code holding a dangling pointer, and nothing
//! would look wrong. ⭐ Boxing gives a heap address that survives every move of
//! the handle — which matters because the handle is a value until the C ABI
//! pins it. The same reasoning applies to the services record.
//!
//! ⚠ The boundary region's own published header is a `Vec<u64>`, so its base is
//! already heap-stable; ⛔ that is a property of that type, not a general one,
//! and is the reason the two cases are treated differently rather than
//! uniformly.
//!
//! ## ⛔ Not an artifact-static arena
//!
//! `§3b` closes that fork: a process-lifetime arena is shared by repeated,
//! concurrent and re-entrant activations, and its published table pointers and
//! counts are **mutable and invocation-specific**, so two executions would
//! **alias storage**. ⇒ The arena is invocation-owned, and
//! `two_activations_do_not_share_mutable_arena_state` is the control that would
//! have caught the alternative.

use std::ffi::c_void;

use crate::activation_services::GeneratedActivationServicesV1;
use crate::boundary_resource_profile::BoundaryResourceProfileV1;
use crate::boundary_value::{
    BoundaryArenaBuilder, BoundaryArenaV1, BoundaryValueStore, BoundaryWord, ARENA_DATA_CAPACITY,
    ARENA_LIMB_CAPACITY, ARENA_NATIVE_INT, ARENA_NODE_CAPACITY, ARENA_PERSISTENT,
    ARENA_WORD_CAPACITY,
};
use crate::native_int::NativeIntArenaV1;

/// **One activation: the per-invocation arenas, the services record, and the
/// published bases that generated code is given.**
///
/// ⛔ The `BoundaryValueStore` is **not** owned here — it outlives the
/// activation and may back several in sequence. It is passed to the operations
/// that need it, which keeps *"the store owns the persistent image for as long
/// as any adopted result may live"* a property of the caller's scope rather
/// than of this struct's drop order.
pub struct BoundaryActivationV1 {
    profile: BoundaryResourceProfileV1,
    /// ⛔ Boxed for address stability — see the module doc.
    native_int_arena: Box<NativeIntArenaV1>,
    arena: Box<BoundaryArenaV1>,
    /// ⛔ Boxed for the same reason: generated code receives its address.
    services: Box<GeneratedActivationServicesV1>,
    /// The base [`BoundaryArenaV1::publish`] returned, remembered so the
    /// services record can be checked against it without dereferencing
    /// anything.
    published_boundary_base: *mut u64,
    /// The base [`BoundaryValueStore::publish_persistent`] returned.
    published_persistent_base: *mut u64,
    finished: bool,
}

/// **The store-lifetime half of the ruled order: the persistent image is
/// reserved and published ONCE PER STORE, not once per activation.**
///
/// ⛔⛔ **This split is not stylistic — the first cut of this module got it
/// wrong and the landed guard caught it.** Reserving the persistent image
/// inside each activation panics on the second one with *"reserve before
/// publish: growing a table moves it under the pointer"*, which is
/// `BoundaryRegion::reserve` refusing exactly the thing `§3` forbids.
///
/// ⭐ The correct reading of `§3a`: the **store** owns the persistent image for
/// as long as any adopted persistent result may live, while **each invocation**
/// owns its `NativeIntArenaV1`, `BoundaryArenaV1` and services record. ⇒ Two
/// scopes, two lifetimes, and the profile spans both.
///
/// ⚠ It also makes *"an activation cannot widen its own limits"* structural:
/// the profile lives here, and [`BoundaryActivationV1::begin`] takes it from
/// this binding rather than accepting one from its caller.
pub struct BoundaryStoreBindingV1 {
    profile: BoundaryResourceProfileV1,
    published_persistent_base: *mut u64,
}

impl BoundaryStoreBindingV1 {
    /// Reserve and publish the store's persistent image from the authorized
    /// persistent limits. ⛔ Once per store.
    pub fn open(store: &mut BoundaryValueStore, profile: BoundaryResourceProfileV1) -> Self {
        // Routed through `as_reserve_arguments` so the named->positional
        // mapping is spelled once in the whole crate.
        let (nodes, words, data, limbs) = profile.persistent.as_reserve_arguments();
        store.reserve_persistent(nodes, words, data, limbs);
        let published_persistent_base = store.publish_persistent();
        BoundaryStoreBindingV1 {
            profile,
            published_persistent_base,
        }
    }

    /// The authorized profile. ⛔ Read-only.
    pub fn profile(&self) -> BoundaryResourceProfileV1 {
        self.profile
    }

    /// The published persistent-image base every activation binds.
    pub fn published_persistent_base(&self) -> *mut u64 {
        self.published_persistent_base
    }
}

impl BoundaryActivationV1 {
    /// **Begin an activation: bind, reserve this invocation's storage, publish,
    /// in the ruled order.**
    ///
    /// ⛔ Every *invocation* reservation happens here and nowhere else. There is
    /// no public reserve on the returned value, so *"no post-publication
    /// reservation"* is a property of the type rather than a rule someone has to
    /// remember. ⭐ The persistent half was already reserved and published by
    /// [`BoundaryStoreBindingV1::open`].
    pub fn begin(binding: &BoundaryStoreBindingV1) -> Self {
        let profile = binding.profile;
        let published_persistent_base = binding.published_persistent_base;

        let mut native_int_arena = Box::new(NativeIntArenaV1::default());
        let mut arena = Box::new(BoundaryArenaBuilder::new().finish());
        arena.bind_persistent(Some(published_persistent_base as *const u64));
        // 2 — bind the native arena. ⭐ The pointer bound here and the pointer
        //     the services record carries are THE SAME, by construction: it is
        //     read once into a local and used twice.
        let native_base: *mut u64 = (&mut *native_int_arena as *mut NativeIntArenaV1).cast();
        arena.bind_native_int(Some(native_base as *const u64));
        // 3 — reserve invocation storage from the AUTHORIZED invocation limits.
        let (nodes, words, data, limbs) = profile.invocation.as_reserve_arguments();
        arena.reserve(nodes, words, data, limbs);
        // 4 — publish, and only now build the services record.
        let published_boundary_base = arena.publish();
        let services = Box::new(GeneratedActivationServicesV1::new(
            native_base,
            published_boundary_base,
        ));

        BoundaryActivationV1 {
            profile,
            native_int_arena,
            arena,
            services,
            published_boundary_base,
            published_persistent_base,
            finished: false,
        }
    }

    /// The `services_ptr` generated code receives as its second parameter.
    ///
    /// ⛔ `None` once the activation is finished: the persistent image has been
    /// sealed and handing generated code a pointer into a sealed world would be
    /// a write path the seal exists to close.
    pub fn services_ptr(&self) -> Option<*const c_void> {
        if self.finished || !self.is_published() {
            return None;
        }
        Some(self.services.as_ptr())
    }

    /// Whether both regions were actually published.
    ///
    /// ⛔⛔ **This exists because a mutation taught it.** Bypassing
    /// `BoundaryArenaV1::publish` left a null base, and every accessor that read
    /// the published header dereferenced it — so `AC-3`(b)'s substitution
    /// produced a **SIGSEGV that took the whole test binary with it**, not a
    /// red. ⚠ A crash is not a loud failure: it destroys the other tests'
    /// results in the same shard and names nothing.
    ///
    /// ⇒ Publication is now a checked state, the header accessors return
    /// `Option`, and an unpublished activation simply has no services pointer to
    /// give — ⭐ which is the permitted fail-closed shape: refusal **before**
    /// generated code is called, ⛔ never a launcher that runs and then quietly
    /// skips it.
    pub fn is_published(&self) -> bool {
        !self.published_boundary_base.is_null() && !self.published_persistent_base.is_null()
    }

    /// The published boundary-arena base, as the owner recorded it at publish
    /// time.
    ///
    /// ⭐ Exposed so the services record can be checked **by pointer equality**
    /// against the value [`BoundaryArenaV1::publish`] actually returned —
    /// ⛔ no dereference, so substituting a foreign pointer is caught without
    /// reading through it.
    pub fn published_boundary_base(&self) -> *mut u64 {
        self.published_boundary_base
    }

    /// The published persistent-image base.
    pub fn published_persistent_base(&self) -> *mut u64 {
        self.published_persistent_base
    }

    /// The native-`Int` arena pointer this activation bound and published, as
    /// the **services record** carries it.
    pub fn native_int_arena_ptr(&self) -> *mut u64 {
        self.services.native_int_arena
    }

    /// The address of the native-`Int` arena this activation **owns**, read
    /// from the box rather than from the services record.
    ///
    /// ⭐⭐ **A second, independent surface on purpose.** A `dead_code` warning
    /// said the owned box was never read — and it was right: every check went
    /// through the services record, so the record could have agreed with itself
    /// while pointing at something the activation does not own. ⇒ Comparing
    /// this against [`Self::native_int_arena_ptr`] is two surfaces agreeing,
    /// ⛔ not one surface read twice.
    ///
    /// ⚠ An **address**, deliberately, not a usable pointer: its only purpose
    /// is identity comparison, and returning `*mut` would invite a caller to
    /// dereference a second alias of the arena.
    pub fn owned_native_arena_address(&self) -> usize {
        (&*self.native_int_arena as *const NativeIntArenaV1) as usize
    }

    /// The profile this activation was authorized with. ⛔ Read-only: an
    /// activation cannot widen its own limits.
    pub fn profile(&self) -> BoundaryResourceProfileV1 {
        self.profile
    }

    /// Read-only view of the invocation arena, for inspecting what generated
    /// code constructed.
    pub fn arena(&self) -> &BoundaryArenaV1 {
        &self.arena
    }

    /// The four capacity ceilings **as generated code will read them**, from
    /// the published header rather than from a Rust-side mirror.
    ///
    /// ⭐ This is deliberately the same read the emitted allocator performs, so
    /// the pin over it measures what generated code will see. ⚠ A Rust-side
    /// accessor would measure a parallel copy and could agree while the header
    /// disagreed.
    ///
    /// Order: nodes, words, data bytes, limbs — the same order as
    /// `reserve`, so it can be compared against `as_reserve_arguments` directly.
    ///
    /// # Safety
    ///
    /// Reads the header this activation itself published and still owns. The
    /// allocation is alive for `&self`, and the offsets are the region's own
    /// derived field offsets.
    pub fn published_capacities(&self) -> Option<(u64, u64, u64, u64)> {
        let base = self.published_boundary_base;
        if base.is_null() {
            return None;
        }
        unsafe {
            Some((
                header_word(base, ARENA_NODE_CAPACITY),
                header_word(base, ARENA_WORD_CAPACITY),
                header_word(base, ARENA_DATA_CAPACITY),
                header_word(base, ARENA_LIMB_CAPACITY),
            ))
        }
    }

    /// The two pointers the published boundary header carries — the persistent
    /// image and the native-`Int` arena.
    ///
    /// # Safety
    ///
    /// As [`Self::published_capacities`].
    pub fn published_bindings(&self) -> Option<(u64, u64)> {
        let base = self.published_boundary_base;
        if base.is_null() {
            return None;
        }
        unsafe {
            Some((
                header_word(base, ARENA_PERSISTENT),
                header_word(base, ARENA_NATIVE_INT),
            ))
        }
    }

    /// **Finish the activation: seal the persistent image, then adopt an
    /// escaping result.**
    ///
    /// ⛔ Seal first. Adoption absorbs the published counts, validates the
    /// reachable graph, mints identity and publishes — every step reading a
    /// snapshot it assumes is stable. A writer that can still run makes that
    /// snapshot a fiction.
    ///
    /// ⛔ After this the services pointer is withdrawn.
    pub fn finish(
        &mut self,
        store: &mut BoundaryValueStore,
        escaping: Option<BoundaryWord>,
    ) -> Result<Option<BoundaryWord>, i64> {
        self.finished = true;
        store.seal_persistent();
        match escaping {
            None => Ok(None),
            Some(word) => store.adopt(word).map(Some),
        }
    }

    /// Whether [`Self::finish`] has run.
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

/// One word of a published region header.
///
/// # Safety
///
/// `base` must be a header published by this crate and still alive, and
/// `offset` must be one of the region's derived field offsets.
unsafe fn header_word(base: *mut u64, offset: i32) -> u64 {
    unsafe { *base.byte_offset(offset as isize) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_resource_profile::{
        BoundaryRegionLimitsV1, BoundaryResource, BoundaryResourceScope,
    };

    /// Eight **distinct** limits, so a transposition anywhere in `begin` is
    /// visible. ⛔ Equal limits would let every assertion below pass on a
    /// crossed wiring.
    fn distinct_profile() -> BoundaryResourceProfileV1 {
        BoundaryResourceProfileV1 {
            invocation: BoundaryRegionLimitsV1 {
                nodes: 12,
                words: 24,
                data_bytes: 36,
                native_int_limbs: 48,
            },
            persistent: BoundaryRegionLimitsV1 {
                nodes: 60,
                words: 72,
                data_bytes: 84,
                native_int_limbs: 96,
            },
        }
    }

    /// ⭐⭐ **`AC-3`(c) — Finding 8's exact defect, caught by pointer identity
    /// and ⛔ without dereferencing anything.**
    ///
    /// **MEASURED:** the services record's `boundary_arena` is the pointer
    /// `BoundaryArenaV1::publish` returned, and its `native_int_arena` is the
    /// pointer bound into the header at `ARENA_NATIVE_INT` — and the two are
    /// **different**.
    /// **CLAIMED:** generated code loading `SERVICES_BOUNDARY_ARENA` reaches the
    /// boundary arena and not the native one.
    /// **THE GAP:** ⛔ that generated code loads that field at all. The
    /// per-function binder is held on `B2F`; ⚠ this is the owner's half of the
    /// contract, not the emitter's.
    ///
    /// ⚠ **Why equality and not a header read:** substituting the native arena
    /// makes every header offset past 64 bytes an out-of-bounds read, so a
    /// "check the header looks wrong" control would be undefined behaviour
    /// rather than a red. Pointer identity fails cleanly.
    #[test]
    fn the_services_record_carries_the_boundary_base_and_not_the_native_one() {
        let mut store = BoundaryValueStore::new();
        let binding = BoundaryStoreBindingV1::open(&mut store, distinct_profile());
        let activation = BoundaryActivationV1::begin(&binding);

        let services = activation.services_ptr().expect("live activation");
        assert!(!services.is_null());

        assert_eq!(
            activation.services.boundary_arena,
            activation.published_boundary_base(),
            "AC-3(c): the services record's boundary arena is not the pointer \
             `publish` returned"
        );
        assert_eq!(
            activation.services.native_int_arena,
            activation.native_int_arena_ptr()
        );
        assert_ne!(
            activation.services.boundary_arena, activation.services.native_int_arena,
            "AC-3(c): the boundary and native arenas are the same pointer — this \
             is Finding 8 and it must be impossible, not merely absent"
        );

        // The header agrees with the record about the native arena, which is
        // the ruling's "one native authority" property.
        let (persistent, native) = activation
            .published_bindings()
            .expect("AC-3(b): a published activation must expose its header bindings");
        assert_eq!(native, activation.services.native_int_arena as u64);
        assert_eq!(persistent, activation.published_persistent_base() as u64);
        assert_ne!(persistent, 0, "ARENA_PERSISTENT was left unbound");
        assert_ne!(native, 0, "ARENA_NATIVE_INT was left unbound");
    }

    /// ⭐⭐ **`AC-2` — two activations get distinct mutable arena state.**
    ///
    /// ⚠ **This is the control that would have caught the artifact-static
    /// fork**, and it is written to fail if storage is aliased. ⛔ A fixture
    /// that runs one activation twice and checks it did not crash is *not*
    /// this: the two activations are alive **simultaneously** and their
    /// published bases, native arenas and services records are all required to
    /// be pairwise distinct.
    ///
    /// ⭐ They share exactly one thing, and it is the one the ruling permits:
    /// the store-owned persistent image.
    #[test]
    fn two_activations_do_not_share_mutable_arena_state() {
        let mut store = BoundaryValueStore::new();
        let binding = BoundaryStoreBindingV1::open(&mut store, distinct_profile());
        let first = BoundaryActivationV1::begin(&binding);
        let second = BoundaryActivationV1::begin(&binding);

        assert_ne!(
            first.published_boundary_base(),
            second.published_boundary_base(),
            "AC-2: two activations published the SAME arena base — their \
             mutable node/word counts alias"
        );
        assert_ne!(
            first.native_int_arena_ptr(),
            second.native_int_arena_ptr(),
            "AC-2: two activations share one native-Int arena"
        );
        assert_ne!(
            first.owned_native_arena_address(),
            second.owned_native_arena_address(),
            "AC-2: the two OWNED native arenas are one allocation, so the \
             records above differ while the storage behind them does not"
        );
        // The record and the owned box agree, per activation — two surfaces,
        // not one read twice.
        assert_eq!(
            first.native_int_arena_ptr() as usize,
            first.owned_native_arena_address()
        );
        assert_eq!(
            second.native_int_arena_ptr() as usize,
            second.owned_native_arena_address()
        );
        assert_ne!(
            first.services_ptr().expect("live"),
            second.services_ptr().expect("live"),
            "AC-2: two activations share one services record"
        );

        // ⭐ And what they DO share is exactly the permitted thing.
        assert_eq!(
            first.published_persistent_base(),
            second.published_persistent_base(),
            "the store-owned persistent image is explicitly shared; if this \
             diverges the store is no longer the single persistent authority"
        );
    }

    /// ⛔ **Moving the activation does not move the pointers generated code
    /// holds.**
    ///
    /// ⚠ The failure this prevents is silent: `NativeIntArenaV1` is a plain
    /// `#[repr(C)]` struct, so an unboxed field would hand out a pointer *into*
    /// the value, and moving the handle — returning it, pushing it into a
    /// `Vec`, storing it in a C-owned box — would dangle it with nothing
    /// looking wrong.
    #[test]
    fn moving_the_activation_does_not_move_what_generated_code_was_given() {
        let mut store = BoundaryValueStore::new();
        let binding = BoundaryStoreBindingV1::open(&mut store, distinct_profile());
        let activation = BoundaryActivationV1::begin(&binding);
        let (services, native, boundary) = (
            activation.services_ptr().expect("live"),
            activation.native_int_arena_ptr(),
            activation.published_boundary_base(),
        );

        // Move it twice, through a container, exactly as the C ABI will.
        let boxed = Box::new(activation);
        let mut moved = vec![*boxed];
        let activation = moved.pop().expect("one activation");

        assert_eq!(activation.services_ptr().expect("live"), services);
        assert_eq!(activation.native_int_arena_ptr(), native);
        assert_eq!(activation.published_boundary_base(), boundary);
    }

    /// ⭐ **`AC-4`'s WIRING half — each of the eight authorized limits reaches
    /// its own ceiling, read as generated code reads it.**
    ///
    /// **MEASURED:** the four capacity words in the *published header* equal the
    /// profile's four invocation limits, in order; and the persistent image's
    /// node ceiling equals the profile's persistent node limit.
    /// **CLAIMED:** each limit governs its named region and resource.
    /// **THE GAP:** ⛔⛔ **at-limit-plus-one is NOT measured here.** `AC-4`
    /// requires a request one past each ceiling to fail loudly naming that
    /// exact scope, and the requester is *generated code*, which this node does
    /// not make live. ⇒ That half is `S4`'s, against a real linked run.
    /// ⛔ Do not read this test as `AC-4` discharged.
    ///
    /// ⚠ Not circular: it crosses from the **profile** the deployment wrote to
    /// the **published header** generated code will read — two different
    /// objects — so a transposition inside `begin` reddens it.
    #[test]
    fn the_eight_authorized_limits_reach_their_own_published_ceilings() {
        let profile = distinct_profile();
        let mut store = BoundaryValueStore::new();
        let binding = BoundaryStoreBindingV1::open(&mut store, profile);
        let activation = BoundaryActivationV1::begin(&binding);

        let (nodes, words, data, limbs) = activation
            .published_capacities()
            .expect("AC-3(b): a published activation must expose its ceilings");
        let expected = profile.invocation.as_reserve_arguments();
        assert_eq!(
            (
                nodes as usize,
                words as usize,
                data as usize,
                limbs as usize
            ),
            expected,
            "AC-4 wiring: the invocation ceilings in the published header are \
             not the four authorized invocation limits, in order"
        );

        // The persistent side, through the store's own accessor.
        assert_eq!(
            store.image().0.node_capacity(),
            profile.persistent.nodes,
            "AC-4 wiring: the persistent node ceiling is not the authorized one"
        );

        // Non-vacuity: the eight authorized numbers are pairwise distinct, so
        // agreeing with them is a real constraint rather than a coincidence.
        let mut seen = std::collections::BTreeSet::new();
        for scope in BoundaryResourceScope::ALL {
            for resource in BoundaryResource::ALL {
                assert!(seen.insert(profile.limit(scope, resource)));
            }
        }
        assert_eq!(seen.len(), 8);
    }

    /// ⛔ **`AC-3`(b) — the services pointer is withdrawn once the activation is
    /// finished**, so a caller cannot hand generated code a way into a sealed
    /// world.
    #[test]
    fn finishing_withdraws_the_services_pointer_and_seals_the_image() {
        let mut store = BoundaryValueStore::new();
        let binding = BoundaryStoreBindingV1::open(&mut store, distinct_profile());
        let mut activation = BoundaryActivationV1::begin(&binding);
        assert!(activation.services_ptr().is_some());
        assert!(!store.is_persistent_sealed());

        activation
            .finish(&mut store, None)
            .expect("finishing with nothing escaping cannot fail");

        assert!(activation.is_finished());
        assert!(activation.services_ptr().is_none());
        assert!(
            store.is_persistent_sealed(),
            "AC-3(b): finishing did not seal the persistent image, so adoption \
             would read a snapshot writers can still change"
        );
    }
}
