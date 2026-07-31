//! **`RT-FNSPLIT-C3-ACTIVATION` `D2` — the small C ABI over an opaque
//! activation handle.**
//!
//! ⭐ **C stores an opaque pointer and status values, and nothing else.** That
//! is the whole point of `§3a`: the existing generated C already **duplicates**
//! `struct KenNativeIntArenaV1` twice and **constructs** an arena itself, and
//! the ruling says to **subsume** that precedent, ⛔ not repeat it. So no arena,
//! services or activation layout crosses into C — only handles.
//!
//! ## The two scopes survive the crossing
//!
//! `S2` measured that the persistent image is **store**-lifetime while the
//! arenas are **invocation**-lifetime. ⇒ The C ABI keeps both, rather than
//! flattening them into one call and re-introducing the bug at the boundary:
//!
//! ```text
//! ken_boundary_store_v1_open(profile, &store)      // once per store
//! ken_activation_v1_begin(store, &activation)      // once per activation
//! ken_activation_v1_services(activation, &services)
//! ken_activation_v1_finish(activation, store, ...)
//! ken_activation_v1_destroy(activation)
//! ken_boundary_store_v1_destroy(store)
//! ```
//!
//! ## ⛔ The profile is the ONE layout C is allowed to know
//!
//! `§4` bans a second copy of the **arena, services, native-`Int` or
//! activation** layouts in generated C. ⚠ The profile is not among them, and
//! `D5` explicitly contemplates the stub *"embedding those already-authorized
//! numbers"*. ⇒ [`KenBoundaryResourceProfileV1`] is deliberately the only
//! `#[repr(C)]` struct that crosses.
//!
//! ⭐ And it carries its own `version` and `size` so a C/Rust disagreement
//! **fails closed** rather than being read under a layout it does not have —
//! ⛔ eight bare positional `u64` parameters would have reintroduced exactly the
//! transposition hazard that `BoundaryRegionLimitsV1`'s named fields removed,
//! in the one language with no help against it.

use std::ffi::c_void;

use crate::boundary_activation::{BoundaryActivationV1, BoundaryStoreBindingV1};
use crate::boundary_resource_profile::{
    BoundaryRegionLimitsV1, BoundaryResourceProfileV1, BOUNDARY_RESOURCE_PROFILE_VERSION,
};
use crate::boundary_value::{BoundaryValueStore, BoundaryWord};

/// Success.
pub const KEN_ACTIVATION_OK: i64 = 0;
/// A required out-parameter or handle was null.
pub const KEN_ACTIVATION_ERR_NULL: i64 = -1;
/// The profile's `version`/`size` do not describe a layout this runtime
/// implements. ⛔ Fail closed rather than reinterpret.
pub const KEN_ACTIVATION_ERR_PROFILE: i64 = -2;
/// The activation is finished, so it has no services pointer to give.
pub const KEN_ACTIVATION_ERR_FINISHED: i64 = -3;
/// Adoption of the escaping result failed. ⚠ The boundary status is not
/// squashed into this one: it is returned through the out-parameter.
pub const KEN_ACTIVATION_ERR_ADOPT: i64 = -4;
/// ⛔ Generated code wrote a MALFORMED final export. ⚠ Distinct from "nothing
/// was exported", which succeeds and renders the raw entry result — collapsing
/// the two would turn a corrupt export into a printed number.
pub const KEN_ACTIVATION_ERR_EXPORT: i64 = -5;
/// The rendering does not fit the caller's buffer. ⛔ Nothing partial is
/// written: a truncated integer is a wrong answer, not a short one.
pub const KEN_ACTIVATION_ERR_BUFFER: i64 = -6;

/// **The deployment-authorized profile, as it crosses into C.**
///
/// ⛔ Eight named limits and no default — the C side supplies all eight or the
/// call is refused. ⭐ `version` and `size` make a layout disagreement a
/// **checked refusal** instead of a silent misread.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct KenBoundaryResourceProfileV1 {
    /// Must equal [`BOUNDARY_RESOURCE_PROFILE_VERSION`].
    pub version: u64,
    /// Must equal `size_of::<KenBoundaryResourceProfileV1>()`.
    pub size: u64,
    /// Invocation-arena node ceiling.
    pub invocation_nodes: u64,
    /// Invocation-arena child-word ceiling.
    pub invocation_words: u64,
    /// Invocation-arena data-byte ceiling.
    pub invocation_data_bytes: u64,
    /// Invocation-arena native-`Int` limb ceiling.
    pub invocation_native_int_limbs: u64,
    /// Persistent-image node ceiling.
    pub persistent_nodes: u64,
    /// Persistent-image child-word ceiling.
    pub persistent_words: u64,
    /// Persistent-image data-byte ceiling.
    pub persistent_data_bytes: u64,
    /// Persistent-image native-`Int` limb ceiling.
    pub persistent_native_int_limbs: u64,
}

impl KenBoundaryResourceProfileV1 {
    /// Convert to the Rust profile, refusing a layout this runtime does not
    /// implement.
    ///
    /// ⛔ No default and no widening: a wrong `version` or `size` is
    /// [`KEN_ACTIVATION_ERR_PROFILE`], ⛔ never a fallback profile.
    fn to_rust(self) -> Option<BoundaryResourceProfileV1> {
        if self.version != u64::from(BOUNDARY_RESOURCE_PROFILE_VERSION)
            || self.size != std::mem::size_of::<KenBoundaryResourceProfileV1>() as u64
        {
            return None;
        }
        Some(BoundaryResourceProfileV1 {
            invocation: BoundaryRegionLimitsV1 {
                nodes: self.invocation_nodes as usize,
                words: self.invocation_words as usize,
                data_bytes: self.invocation_data_bytes as usize,
                native_int_limbs: self.invocation_native_int_limbs as usize,
            },
            persistent: BoundaryRegionLimitsV1 {
                nodes: self.persistent_nodes as usize,
                words: self.persistent_words as usize,
                data_bytes: self.persistent_data_bytes as usize,
                native_int_limbs: self.persistent_native_int_limbs as usize,
            },
        })
    }
}

/// The store-lifetime handle. ⛔ Opaque to C.
pub struct KenBoundaryStoreV1 {
    store: BoundaryValueStore,
    binding: BoundaryStoreBindingV1,
}

/// The activation handle. ⛔ Opaque to C.
pub struct KenActivationV1 {
    activation: BoundaryActivationV1,
}

/// Open a store and reserve/publish its persistent image from the authorized
/// profile. ⛔ Once per store.
///
/// # Safety
///
/// `profile` must point to a readable [`KenBoundaryResourceProfileV1`] and
/// `out_store` to a writable pointer slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_boundary_store_v1_open(
    profile: *const KenBoundaryResourceProfileV1,
    out_store: *mut *mut KenBoundaryStoreV1,
) -> i64 {
    if profile.is_null() || out_store.is_null() {
        return KEN_ACTIVATION_ERR_NULL;
    }
    let Some(profile) = (unsafe { *profile }).to_rust() else {
        return KEN_ACTIVATION_ERR_PROFILE;
    };
    let mut store = BoundaryValueStore::new();
    let binding = BoundaryStoreBindingV1::open(&mut store, profile);
    let handle = Box::into_raw(Box::new(KenBoundaryStoreV1 { store, binding }));
    unsafe { *out_store = handle };
    KEN_ACTIVATION_OK
}

/// Destroy a store handle.
///
/// # Safety
///
/// `store` must be a handle from [`ken_boundary_store_v1_open`] that has not
/// already been destroyed, and every activation opened against it must already
/// have been destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_boundary_store_v1_destroy(store: *mut KenBoundaryStoreV1) -> i64 {
    if store.is_null() {
        return KEN_ACTIVATION_ERR_NULL;
    }
    drop(unsafe { Box::from_raw(store) });
    KEN_ACTIVATION_OK
}

/// Begin one activation against an open store.
///
/// # Safety
///
/// `store` must be a live handle and `out_activation` a writable pointer slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_activation_v1_begin(
    store: *mut KenBoundaryStoreV1,
    out_activation: *mut *mut KenActivationV1,
) -> i64 {
    if store.is_null() || out_activation.is_null() {
        return KEN_ACTIVATION_ERR_NULL;
    }
    let store = unsafe { &*store };
    let activation = BoundaryActivationV1::begin(&store.binding);
    let handle = Box::into_raw(Box::new(KenActivationV1 { activation }));
    unsafe { *out_activation = handle };
    KEN_ACTIVATION_OK
}

/// **The generated-entry services view** — the `services_ptr` the internal
/// `(frame_ptr, services_ptr)` convention will take.
///
/// ⛔ `KEN_ACTIVATION_ERR_FINISHED` when the activation is finished or was never
/// published. ⭐ That is the permitted fail-closed shape: the launcher learns it
/// has nothing to call **with**, before calling — ⛔ never a starter that runs
/// and silently omits generated execution.
///
/// # Safety
///
/// `activation` must be a live handle and `out_services` a writable slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_activation_v1_services(
    activation: *const KenActivationV1,
    out_services: *mut *const c_void,
) -> i64 {
    if activation.is_null() || out_services.is_null() {
        return KEN_ACTIVATION_ERR_NULL;
    }
    let activation = unsafe { &*activation };
    match activation.activation.services_ptr() {
        Some(services) => {
            unsafe { *out_services = services };
            KEN_ACTIVATION_OK
        }
        None => KEN_ACTIVATION_ERR_FINISHED,
    }
}

/// **Bind process ingress and hand back the `frame_ptr` generated code takes as
/// its FIRST parameter.**
///
/// ⭐ `§3d`: the adapter obtains the root frame view **from the Rust owner**. ⇒
/// The generated C stub stops declaring `struct KenNativeInvocationV1`, stops
/// declaring `struct KenNativeIntArenaV1`, and stops constructing an arena on
/// its own stack — it passes `process_input`, `host_context` and `capability`
/// in and receives one opaque pointer back.
///
/// ⛔ `KEN_ACTIVATION_ERR_FINISHED` when the activation is finished or was never
/// published — the frame and the services pointer are withdrawn **together**, so
/// a caller cannot obtain half of a withdrawn pair.
///
/// # Safety
///
/// `activation` must be a live handle and `out_frame` a writable slot.
/// `process_input` and `host_context` are passed through unmodified and are the
/// caller's to keep alive across generated execution.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_activation_v1_bind_process_frame(
    activation: *mut KenActivationV1,
    process_input: *const c_void,
    host_context: *mut c_void,
    capability: u64,
    out_frame: *mut *const c_void,
) -> i64 {
    if activation.is_null() || out_frame.is_null() {
        return KEN_ACTIVATION_ERR_NULL;
    }
    let activation = unsafe { &mut *activation };
    match activation
        .activation
        .bind_process_frame(process_input, host_context, capability)
    {
        Some(frame) => {
            unsafe { *out_frame = frame };
            KEN_ACTIVATION_OK
        }
        None => KEN_ACTIVATION_ERR_FINISHED,
    }
}

/// **The `frame_ptr` for the non-process launch shape.**
///
/// ⭐ The generated root's single parameter is the native-`Int` arena there, so
/// the adapter hands that back — ⛔ still as one opaque pointer, with no layout
/// crossing into C.
///
/// # Safety
///
/// `activation` must be a live handle and `out_frame` a writable slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_activation_v1_native_frame(
    activation: *const KenActivationV1,
    out_frame: *mut *const c_void,
) -> i64 {
    if activation.is_null() || out_frame.is_null() {
        return KEN_ACTIVATION_ERR_NULL;
    }
    let activation = unsafe { &*activation };
    match activation.activation.native_frame_ptr() {
        Some(frame) => {
            unsafe { *out_frame = frame };
            KEN_ACTIVATION_OK
        }
        None => KEN_ACTIVATION_ERR_FINISHED,
    }
}

/// **Render the final exported `Int`, or the raw entry result when nothing was
/// exported, into a caller-supplied buffer.**
///
/// ⭐⭐ **This is what lets the stub stop declaring the arena layout.** The C
/// side used to read `final_tag`/`final_payload`/`final_sign`/`final_len`/
/// `final_limbs` itself, re-derive the export's canonicality checks, and format
/// the digits — a second implementation of both. Now it asks the owner and
/// writes bytes.
///
/// `out_len` receives the number of bytes written. ⛔ `KEN_ACTIVATION_ERR_EXPORT`
/// when generated code wrote an export that is **malformed** — ⚠ a different
/// outcome from *"nothing was exported"*, which renders `fallback` and succeeds.
/// Collapsing the two would turn a corrupt export into a printed number.
///
/// ⛔ `KEN_ACTIVATION_ERR_BUFFER` when the rendering does not fit; ⚠ nothing
/// partial is written, because a truncated integer is a wrong answer rather than
/// a short one.
///
/// # Safety
///
/// `activation` must be live, `buffer` must be writable for `capacity` bytes,
/// and `out_len` must be a writable slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_activation_v1_write_final_export(
    activation: *const KenActivationV1,
    fallback: i64,
    buffer: *mut u8,
    capacity: usize,
    out_len: *mut usize,
) -> i64 {
    if activation.is_null() || buffer.is_null() || out_len.is_null() {
        return KEN_ACTIVATION_ERR_NULL;
    }
    let arena = unsafe { &*activation }.activation.native_int_arena();
    let rendered = if arena.has_final_export() {
        match arena.decode_final_export() {
            Some(export) => format_final_export(Some(export), fallback),
            None => return KEN_ACTIVATION_ERR_EXPORT,
        }
    } else {
        format_final_export(None, fallback)
    };
    if rendered.len() > capacity {
        return KEN_ACTIVATION_ERR_BUFFER;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(rendered.as_ptr(), buffer, rendered.len());
        *out_len = rendered.len();
    }
    KEN_ACTIVATION_OK
}

/// Seal the persistent image and adopt an escaping result.
///
/// `escaping_word` is `0` for *"nothing escapes"*; otherwise it is a boundary
/// word. On success `out_word` receives the adopted word.
///
/// # Safety
///
/// Both handles must be live, and `out_word` must be a writable slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_activation_v1_finish(
    activation: *mut KenActivationV1,
    store: *mut KenBoundaryStoreV1,
    escaping_word: u64,
    out_word: *mut u64,
) -> i64 {
    if activation.is_null() || store.is_null() || out_word.is_null() {
        return KEN_ACTIVATION_ERR_NULL;
    }
    let activation = unsafe { &mut *activation };
    let store = unsafe { &mut *store };
    let escaping = (escaping_word != 0).then(|| BoundaryWord(escaping_word));
    match activation.activation.finish(&mut store.store, escaping) {
        Ok(word) => {
            unsafe { *out_word = word.map_or(0, |word| word.0) };
            KEN_ACTIVATION_OK
        }
        Err(_) => KEN_ACTIVATION_ERR_ADOPT,
    }
}

/// Destroy an activation handle.
///
/// # Safety
///
/// `activation` must be a handle from [`ken_activation_v1_begin`] that has not
/// already been destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_activation_v1_destroy(activation: *mut KenActivationV1) -> i64 {
    if activation.is_null() {
        return KEN_ACTIVATION_ERR_NULL;
    }
    drop(unsafe { Box::from_raw(activation) });
    KEN_ACTIVATION_OK
}

/// Every symbol this ABI publishes, so a link-level check has one authority to
/// compare against.
///
/// ⭐ `D1`'s own warning is that a `crate-type` line is a **build-system**
/// claim and not a **link** one. ⇒ The archive is checked against *this* list.
/// ⛔ Pinned as the exact permitted set, so an addition reddens too.
pub const KEN_ACTIVATION_ABI_SYMBOLS: [&str; 9] = [
    "ken_boundary_store_v1_open",
    "ken_boundary_store_v1_destroy",
    "ken_activation_v1_begin",
    "ken_activation_v1_services",
    "ken_activation_v1_bind_process_frame",
    "ken_activation_v1_native_frame",
    "ken_activation_v1_write_final_export",
    "ken_activation_v1_finish",
    "ken_activation_v1_destroy",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn c_profile() -> KenBoundaryResourceProfileV1 {
        KenBoundaryResourceProfileV1 {
            version: u64::from(BOUNDARY_RESOURCE_PROFILE_VERSION),
            size: std::mem::size_of::<KenBoundaryResourceProfileV1>() as u64,
            invocation_nodes: 12,
            invocation_words: 24,
            invocation_data_bytes: 36,
            invocation_native_int_limbs: 48,
            persistent_nodes: 60,
            persistent_words: 72,
            persistent_data_bytes: 84,
            persistent_native_int_limbs: 96,
        }
    }

    /// ⭐ **The whole C lifecycle, driven exactly as the stub will drive it**,
    /// with only handles and status values crossing.
    #[test]
    fn the_c_abi_drives_the_whole_lifecycle_with_handles_and_statuses_only() {
        let profile = c_profile();
        let mut store = std::ptr::null_mut();
        assert_eq!(
            unsafe { ken_boundary_store_v1_open(&profile, &mut store) },
            KEN_ACTIVATION_OK
        );
        assert!(!store.is_null());

        let mut activation = std::ptr::null_mut();
        assert_eq!(
            unsafe { ken_activation_v1_begin(store, &mut activation) },
            KEN_ACTIVATION_OK
        );

        let mut services = std::ptr::null();
        assert_eq!(
            unsafe { ken_activation_v1_services(activation, &mut services) },
            KEN_ACTIVATION_OK
        );
        assert!(!services.is_null());

        // ⭐ `D7` — the generated root's frame comes from the OWNER, so the C
        // stub needs no `KenNativeInvocationV1` and no arena of its own.
        let mut frame = std::ptr::null();
        assert_eq!(
            unsafe {
                ken_activation_v1_bind_process_frame(
                    activation,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    3,
                    &mut frame,
                )
            },
            KEN_ACTIVATION_OK
        );
        assert!(!frame.is_null());
        assert_ne!(
            frame, services,
            "the frame and the services view are the two distinct parameters of \
             the internal convention; one pointer for both would collapse them"
        );

        let mut word = u64::MAX;
        assert_eq!(
            unsafe { ken_activation_v1_finish(activation, store, 0, &mut word) },
            KEN_ACTIVATION_OK
        );
        assert_eq!(word, 0);

        // ⛔ After finishing there is nothing to call generated code WITH, and
        // the launcher learns that before calling rather than after.
        assert_eq!(
            unsafe { ken_activation_v1_services(activation, &mut services) },
            KEN_ACTIVATION_ERR_FINISHED
        );
        assert_eq!(
            unsafe {
                ken_activation_v1_bind_process_frame(
                    activation,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    3,
                    &mut frame,
                )
            },
            KEN_ACTIVATION_ERR_FINISHED,
            "the frame must be withdrawn with the services view, not after it"
        );

        assert_eq!(
            unsafe { ken_activation_v1_destroy(activation) },
            KEN_ACTIVATION_OK
        );
        assert_eq!(
            unsafe { ken_boundary_store_v1_destroy(store) },
            KEN_ACTIVATION_OK
        );
    }

    /// ⭐⭐ **`AC-2` survives the C crossing** — two activations against one
    /// store get distinct services records, and the two scopes are not
    /// flattened by the ABI.
    #[test]
    fn two_activations_across_the_c_abi_get_distinct_services() {
        let profile = c_profile();
        let mut store = std::ptr::null_mut();
        assert_eq!(
            unsafe { ken_boundary_store_v1_open(&profile, &mut store) },
            KEN_ACTIVATION_OK
        );

        let mut first = std::ptr::null_mut();
        let mut second = std::ptr::null_mut();
        unsafe { ken_activation_v1_begin(store, &mut first) };
        unsafe { ken_activation_v1_begin(store, &mut second) };

        let (mut a, mut b) = (std::ptr::null(), std::ptr::null());
        unsafe { ken_activation_v1_services(first, &mut a) };
        unsafe { ken_activation_v1_services(second, &mut b) };
        assert!(!a.is_null() && !b.is_null());
        assert_ne!(
            a, b,
            "AC-2: two activations opened through the C ABI share one services \
             record, so the ABI flattened the two scopes"
        );

        unsafe { ken_activation_v1_destroy(first) };
        unsafe { ken_activation_v1_destroy(second) };
        unsafe { ken_boundary_store_v1_destroy(store) };
    }

    /// ⛔ **A profile whose layout this runtime does not implement is REFUSED**,
    /// not reinterpreted — and refused at *open*, which is before any activation
    /// exists.
    ///
    /// ⚠ Both discriminators are exercised separately: a wrong `version` and a
    /// wrong `size` are different disagreements, and a check that only looked at
    /// one would pass on the other.
    #[test]
    fn a_profile_from_a_layout_this_runtime_does_not_implement_is_refused() {
        let mut store = std::ptr::null_mut();

        let mut wrong_version = c_profile();
        wrong_version.version += 1;
        assert_eq!(
            unsafe { ken_boundary_store_v1_open(&wrong_version, &mut store) },
            KEN_ACTIVATION_ERR_PROFILE
        );
        assert!(
            store.is_null(),
            "a refused open must not hand back a handle"
        );

        let mut wrong_size = c_profile();
        wrong_size.size += 8;
        assert_eq!(
            unsafe { ken_boundary_store_v1_open(&wrong_size, &mut store) },
            KEN_ACTIVATION_ERR_PROFILE
        );
        assert!(store.is_null());
    }

    /// ⛔ Every entry point refuses a null handle or out-slot rather than
    /// dereferencing it. ⚠ `S2` already paid for the lesson that a crash is not
    /// a loud failure.
    #[test]
    fn every_entry_point_refuses_null_rather_than_dereferencing_it() {
        let profile = c_profile();
        let mut store_slot = std::ptr::null_mut();
        let mut services_slot = std::ptr::null();
        let mut word = 0u64;

        assert_eq!(
            unsafe { ken_boundary_store_v1_open(std::ptr::null(), &mut store_slot) },
            KEN_ACTIVATION_ERR_NULL
        );
        assert_eq!(
            unsafe { ken_boundary_store_v1_open(&profile, std::ptr::null_mut()) },
            KEN_ACTIVATION_ERR_NULL
        );
        assert_eq!(
            unsafe { ken_activation_v1_begin(std::ptr::null_mut(), &mut store_slot.cast()) },
            KEN_ACTIVATION_ERR_NULL
        );
        assert_eq!(
            unsafe { ken_activation_v1_services(std::ptr::null(), &mut services_slot) },
            KEN_ACTIVATION_ERR_NULL
        );
        assert_eq!(
            unsafe {
                ken_activation_v1_finish(std::ptr::null_mut(), std::ptr::null_mut(), 0, &mut word)
            },
            KEN_ACTIVATION_ERR_NULL
        );
        assert_eq!(
            unsafe { ken_activation_v1_destroy(std::ptr::null_mut()) },
            KEN_ACTIVATION_ERR_NULL
        );
        assert_eq!(
            unsafe { ken_boundary_store_v1_destroy(std::ptr::null_mut()) },
            KEN_ACTIVATION_ERR_NULL
        );
    }

    /// ⛔ **The six statuses are six** — a caller must be able to tell a null
    /// argument from a rejected profile from a finished activation.
    #[test]
    fn the_abi_statuses_are_distinct() {
        let statuses = [
            KEN_ACTIVATION_OK,
            KEN_ACTIVATION_ERR_NULL,
            KEN_ACTIVATION_ERR_PROFILE,
            KEN_ACTIVATION_ERR_FINISHED,
            KEN_ACTIVATION_ERR_ADOPT,
            KEN_ACTIVATION_ERR_EXPORT,
            KEN_ACTIVATION_ERR_BUFFER,
        ];
        let distinct = statuses.iter().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(distinct.len(), statuses.len());
        assert_eq!(
            KEN_ACTIVATION_ABI_SYMBOLS
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            KEN_ACTIVATION_ABI_SYMBOLS.len(),
            "the published symbol list names one symbol twice"
        );
    }
}

/// **`RT-FNSPLIT-C3-ACTIVATION` `D7` — rendering the final exported `Int`, in
/// Rust.**
///
/// ⭐⭐ **A finding, and it is why the stub rewrite is bigger than a deletion.**
/// The generated C stub's `ken_print_exported_int` is not merely a *reader* of
/// the native-`Int` arena layout — it is a **second implementation of `Int`
/// rendering and of the export's canonicality checks**, re-deriving in C what
/// `NativeIntArenaV1::decode_final_export` already decides in Rust. ⇒ Removing
/// the C layout copy is not possible without moving the rendering too, and
/// `docs/PRINCIPLES.md`'s subsume-don't-proliferate says that is the right
/// direction rather than a cost.
///
/// ⛔ **The output is byte-for-byte the C stub's**, because the pre-existing
/// smoke positives assert exact stdout and `AC-6` is discharged by *those*
/// tests, ⛔ not by new ones. The three shapes:
///
/// | case | rendering |
/// |---|---|
/// | no export (`final_tag` untouched) | the raw entry result, `"%lld\n"` |
/// | `Small` | decimal, `"%PRId64\n"` |
/// | `Big` | optional `-`, `0x`, top limb `"%PRIx64"`, each lower limb `"%016PRIx64"`, `\n` |
///
/// ⚠ **The `Big` padding asymmetry is the part that would be got wrong** — the
/// most-significant limb is printed *unpadded* and every lower limb is padded to
/// 16 hex digits. Padding the top limb too would emit leading zeroes that the C
/// stub never emitted.
pub fn format_final_export(
    export: Option<crate::native_int::RuntimeIntV1>,
    fallback: i64,
) -> String {
    use crate::native_int::RuntimeIntV1;
    use crate::values::Sign;
    match export {
        None => format!("{fallback}\n"),
        Some(RuntimeIntV1::Small(value)) => format!("{value}\n"),
        Some(RuntimeIntV1::Big { sign, limbs }) => {
            let mut rendered = String::new();
            if sign == Sign::Negative {
                rendered.push('-');
            }
            rendered.push_str("0x");
            let (top, lower) = limbs.split_last().expect("a Big export has limbs");
            rendered.push_str(&format!("{top:x}"));
            for limb in lower.iter().rev() {
                rendered.push_str(&format!("{limb:016x}"));
            }
            rendered.push('\n');
            rendered
        }
    }
}

#[cfg(test)]
mod export_rendering_tests {
    use super::*;
    use crate::native_int::RuntimeIntV1;
    use crate::values::Sign;

    /// ⭐ **The three shapes render exactly as the C stub rendered them.**
    ///
    /// **MEASURED:** the bytes this function produces for each shape.
    /// **CLAIMED:** replacing the stub's `ken_print_exported_int` with a call
    /// into Rust changes no observable output.
    /// **THE GAP:** ⛔ that the stub actually calls it. That is `S4b`, and
    /// `AC-6`'s discharge is the **pre-existing** smoke positives going green
    /// after the swap — ⛔ not this test.
    ///
    /// Promise class: **normative compatibility vector** — these bytes are a
    /// linked executable's observable output and a starter's contract.
    #[test]
    fn the_rendered_export_matches_the_c_stubs_bytes_for_every_shape() {
        assert_eq!(format_final_export(None, 42), "42\n");
        assert_eq!(format_final_export(None, -7), "-7\n");
        assert_eq!(
            format_final_export(Some(RuntimeIntV1::Small(42)), 0),
            "42\n"
        );
        assert_eq!(
            format_final_export(Some(RuntimeIntV1::Small(-42)), 0),
            "-42\n"
        );

        // One limb: the top limb is UNPADDED.
        assert_eq!(
            format_final_export(
                Some(RuntimeIntV1::Big {
                    sign: Sign::NonNegative,
                    limbs: vec![0x8000_0000_0000_0001],
                }),
                0
            ),
            "0x8000000000000001\n"
        );
        // Two limbs: top unpadded, lower padded to 16 — the asymmetry.
        assert_eq!(
            format_final_export(
                Some(RuntimeIntV1::Big {
                    sign: Sign::NonNegative,
                    limbs: vec![0x0000_0000_0000_00ab, 0x1],
                }),
                0
            ),
            "0x100000000000000ab\n"
        );
        assert_eq!(
            format_final_export(
                Some(RuntimeIntV1::Big {
                    sign: Sign::Negative,
                    limbs: vec![0x0000_0000_0000_00ab, 0x1],
                }),
                0
            ),
            "-0x100000000000000ab\n"
        );
    }

    /// ⛔ **The positive control for the padding asymmetry.** Without it, a
    /// renderer that padded *every* limb would pass the single-limb case and
    /// the sign cases, and fail only on a multi-limb value — the shape least
    /// likely to appear in a smoke fixture.
    #[test]
    fn padding_the_top_limb_too_would_change_the_bytes() {
        let rendered = format_final_export(
            Some(RuntimeIntV1::Big {
                sign: Sign::NonNegative,
                limbs: vec![0xffff_ffff_ffff_ffff, 0x1],
            }),
            0,
        );
        assert_eq!(rendered, "0x1ffffffffffffffff\n");
        assert!(
            !rendered.starts_with("0x0"),
            "the most-significant limb was zero-padded, which the C stub never did"
        );
    }
}
