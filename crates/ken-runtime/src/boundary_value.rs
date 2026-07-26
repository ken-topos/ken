//! `RT-FNSPLIT-B2V` — the executable boundary-value ABI.
//!
//! `RT-FNSPLIT-B2O` gave the static owner partition; `RT-FNSPLIT-B2R` gave the
//! slot order, width and declared ownership of an activation frame. **Neither
//! said what the bits of an `AbiCarrier::ValueWord` or `ResultWord` MEAN**, nor
//! how compiled code inspects a dynamic aggregate. Hard-stop `#10` measured the
//! consequence: a compiled-once callee cannot consume the `Constructor` and
//! `HostResult` values that actually cross a boundary, because `Lowered` is a
//! **compile-time specialization lattice** and the one aggregate path that works
//! today works only because *the consumer is Rust* (`ResultDecoder` +
//! `result_table` in `CompiledModule`).
//!
//! This module supplies the missing half: **one closed 64-bit tagged word** for
//! every source-valued boundary transfer, together with the flat tables emitted
//! code reads it out of and writes it into. The CLIF side lives in
//! [`crate::boundary_value_clif`]; the two are one deliverable and must not be
//! separated — a representation without an executable interface is exactly the
//! shape that produced `#9` and then `#10` one layer down.
//!
//! ## The word
//!
//! ```text
//! bits [0..8)   tag      — BoundaryTag, a closed repr(u8) enum
//! bits [8..64)  payload  — immediate scalar, or a node index in the REGION
//!                          the tag names
//! ```
//!
//! ## ⭐ Two regions, because a word's lifetime is part of its meaning
//!
//! A handle's index is meaningless without knowing which table it indexes, and
//! the tag is what says. There are exactly two:
//!
//! | tag band | region | lives as long as |
//! |---|---|---|
//! | `Persistent*` | [`BoundaryPersistentImage`], owned by [`BoundaryValueStore`] | the store |
//! | `Invocation*` | [`BoundaryArenaV1`] | the native invocation |
//!
//! ⛔ **A persistent word must not be an index into invocation storage.** The
//! escape check permits a persistent word to leave the invocation; if its
//! payload named an arena node, the word it permitted out would name freed
//! storage the moment the arena died. The region split is what makes the
//! permission and the lifetime agree, and it is why the arena carries a
//! *pointer to* persistent storage rather than containing any.
//!
//! **Immediate where lawful, opaque handle otherwise.** The split mirrors the
//! one `spec/40-runtime/41-values.md` already draws and `values.rs` already
//! implements — *"scalars are immediate; compounds are content-addressed"* — so
//! the boundary word does not invent a second value taxonomy.
//!
//! ## ⛔ The representation is never chosen by inspecting a value
//!
//! `AC-2`. A tag is a function of the **class** of a transfer and of magnitude
//! bounds that emitted code re-derives at runtime; it is never a function of a
//! particular JIT-time seed value or of caller depth. This is enforced
//! structurally rather than by assertion: [`BoundaryWord::immediate`] and
//! [`BoundaryArenaBuilder`] take a class and a payload and **nothing else** —
//! neither `NativeSeedEnvironment` nor any environment vector is in scope in
//! this module, and the module does not import one. The `B2R` seed-environment
//! discharge took exactly this form and was the strongest thing in that node.
//!
//! ## Two owners, and they are different questions
//!
//! `D2`/`AC-6`. *Who owns the frame slot that stores the word* is `B2R`'s
//! question and its answer is `AbiStorageOwner`. *Who owns the thing the word
//! points at* is this module's question and its answer is
//! [`BoundaryReferentOwner`]. ⛔ `AbiStorageOwner::ActivationFrame` must never
//! stand in for the second: a persistent referent outlives the frame whose slot
//! held the word, and a borrowed one dies with the invocation even though the
//! slot is frame-owned exactly as before.

use std::collections::BTreeMap;

use crate::ir::{RuntimeGroundValue, RuntimeSymbol};
use crate::store::{SlotId, Store, NULL_SLOT};
use crate::values::Value;

// ---------------------------------------------------------------------------
// The word
// ---------------------------------------------------------------------------

/// Width of the tag field, in bits. The payload occupies the remainder.
pub const BOUNDARY_TAG_BITS: u32 = 8;
/// Mask selecting the tag out of a boundary word.
pub const BOUNDARY_TAG_MASK: u64 = (1 << BOUNDARY_TAG_BITS) - 1;
/// Width of the payload field, in bits.
pub const BOUNDARY_PAYLOAD_BITS: u32 = 64 - BOUNDARY_TAG_BITS;

/// Inclusive lower bound of the immediate-`Int` range.
pub const BOUNDARY_IMMEDIATE_INT_MIN: i64 = -(1i64 << (BOUNDARY_PAYLOAD_BITS - 1));
/// Inclusive upper bound of the immediate-`Int` range.
pub const BOUNDARY_IMMEDIATE_INT_MAX: i64 = (1i64 << (BOUNDARY_PAYLOAD_BITS - 1)) - 1;

/// The closed tag of a boundary word.
///
/// ⛔ **Closed on purpose.** A new carrier or a new representable class is a
/// change *here*, which makes every exhaustive `match` on it a compile error
/// until it is dispositioned — never a value that silently defaults into
/// `ValueWord` (`AC-1`).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BoundaryTag {
    /// `false`/`true` in the payload.
    ImmediateBool = 0,
    /// A two's-complement `Int` inside [`BOUNDARY_IMMEDIATE_INT_MIN`] ..=
    /// [`BOUNDARY_IMMEDIATE_INT_MAX`]. Outside it the value is a
    /// [`BoundaryTag::PersistentGround`] handle — a **runtime** magnitude
    /// dispatch that emitted code performs, not a compile-time specialization.
    ImmediateInt = 1,
    /// A process exit status scalar.
    ImmediateExitStatus = 2,
    /// A host-reply-validated bounded `Nat`.
    ImmediateBoundedNat = 3,
    /// A structural `Nat` deforested to one native scalar.
    ImmediateStructuralNat = 4,
    /// Handle to a persistable Ken value. Payload indexes the **persistent
    /// image**, so the word outlives the invocation that minted it; a node the
    /// store materialized also names the [`SlotId`] that is the referent's
    /// **owner of record**.
    PersistentGround = 5,
    /// Handle to a retained closure: static origin plus captured words. Also
    /// persistent-region-indexed.
    PersistentClosure = 6,
    /// Handle to borrowed ingress — a host-owned buffer or option that is valid
    /// only for this native invocation. Payload indexes the invocation arena.
    InvocationBorrowed = 7,
    /// Handle to a `HostResult`: a runtime success discriminant plus the two
    /// payload words it selects between.
    InvocationHostResult = 8,
}

// ⛔ There is deliberately NO `ImmediateCapability` and no `ImmediateResource`.
//
// An earlier draft had both, and `Lowered::boundary_disposition` produced
// neither: a capability or resource token is an opaque 64-bit identity, and the
// immediate field is 56 bits, so both route to `InvocationBorrowed` handles
// whose node payload holds the full word. Tags that no disposition can produce
// are unreachable representation surface, and unreachable surface reads as
// "supported" to the next person who greps for it. The closed set is therefore
// exactly the set the disposition yields.

impl BoundaryTag {
    /// Every tag, in declaration order.
    ///
    /// ⭐ Derived from the closed `match` below rather than written twice, so
    /// this list cannot drift from the enum: adding a variant without extending
    /// the `match` is a compile error, and the array length is checked against
    /// it in this module's tests.
    pub const ALL: [BoundaryTag; 9] = [
        BoundaryTag::ImmediateBool,
        BoundaryTag::ImmediateInt,
        BoundaryTag::ImmediateExitStatus,
        BoundaryTag::ImmediateBoundedNat,
        BoundaryTag::ImmediateStructuralNat,
        BoundaryTag::PersistentGround,
        BoundaryTag::PersistentClosure,
        BoundaryTag::InvocationBorrowed,
        BoundaryTag::InvocationHostResult,
    ];

    /// Decode a tag byte. `None` for any byte outside the closed set — an
    /// unknown tag is a **third outcome that fails**, never a pass-through.
    pub fn from_bits(bits: u64) -> Option<Self> {
        Some(match bits {
            0 => BoundaryTag::ImmediateBool,
            1 => BoundaryTag::ImmediateInt,
            2 => BoundaryTag::ImmediateExitStatus,
            3 => BoundaryTag::ImmediateBoundedNat,
            4 => BoundaryTag::ImmediateStructuralNat,
            5 => BoundaryTag::PersistentGround,
            6 => BoundaryTag::PersistentClosure,
            7 => BoundaryTag::InvocationBorrowed,
            8 => BoundaryTag::InvocationHostResult,
            _ => return None,
        })
    }

    /// The owner of the thing this word denotes — **not** the owner of the slot
    /// the word sits in (`AC-6`).
    pub fn referent_owner(self) -> BoundaryReferentOwner {
        match self {
            BoundaryTag::ImmediateBool
            | BoundaryTag::ImmediateInt
            | BoundaryTag::ImmediateExitStatus
            | BoundaryTag::ImmediateBoundedNat
            | BoundaryTag::ImmediateStructuralNat => BoundaryReferentOwner::NoReferent,
            BoundaryTag::PersistentGround | BoundaryTag::PersistentClosure => {
                BoundaryReferentOwner::PersistentStore
            }
            BoundaryTag::InvocationBorrowed | BoundaryTag::InvocationHostResult => {
                BoundaryReferentOwner::InvocationArena
            }
        }
    }

    /// Whether the payload is the value itself rather than an arena index.
    pub fn is_immediate(self) -> bool {
        self.referent_owner() == BoundaryReferentOwner::NoReferent
    }
}

/// Who owns the **referent** a handle points at, and therefore how long it
/// lives.
///
/// ⛔ Deliberately a distinct type from `AbiStorageOwner`. `B2R`'s vocabulary
/// answers *who owns the frame slot*; this answers *who owns the thing the slot
/// points at*. Collapsing them is the substitution `AC-6`'s control must redden
/// on.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u64)]
pub enum BoundaryReferentOwner {
    /// An immediate: the word *is* the value, so there is nothing to own.
    NoReferent = 0,
    /// The content-addressed [`Store`]. The referent outlives the activation
    /// whose frame slot held the word, and outlives the invocation.
    PersistentStore = 1,
    /// The invocation-scoped arena. The referent dies when the native
    /// invocation ends; a word naming one **must not escape** (`AC-7`).
    InvocationArena = 2,
}

/// One closed 64-bit boundary value.
///
/// This is the meaning of `AbiCarrier::ValueWord` and of `AbiCarrier::
/// ResultWord`. Both carriers are 8 bytes in `B2R`'s declaration and both are
/// this type; `B2R` declared the width and this declares the content.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BoundaryWord(pub u64);

impl BoundaryWord {
    /// Build a word from a tag and a raw payload.
    ///
    /// ⛔ **`AC-2` is structural here:** the only inputs are a class and a
    /// payload. No seed environment, no caller environment and no activation
    /// depth is in scope in this module, so a representation *cannot* be
    /// specialized from one — there is nothing to specialize it from.
    ///
    /// ⛔ **Panics on a payload outside the tag's domain**, because the shift
    /// that builds the word is *total*: an out-of-range magnitude does not fail,
    /// it becomes a **different value**. The Rust builder and the emitted
    /// `ken_boundary_make_immediate_local` check the same
    /// [`BOUNDARY_IMMEDIATE_DOMAIN`] table — one relation, two enforcement
    /// points, exactly as `push_node` and the allocator share the tag × class
    /// relation. Use [`BoundaryWord::try_immediate`] where the payload is
    /// runtime data rather than a value you have already ranged.
    pub fn immediate(tag: BoundaryTag, payload: u64) -> Self {
        assert!(
            boundary_immediate_admits(tag, payload),
            "the ABI does not admit {payload} as a {tag:?} payload"
        );
        BoundaryWord((payload << BOUNDARY_TAG_BITS) | (tag as u64))
    }

    /// [`BoundaryWord::immediate`] as a fallible check — `None` for a payload
    /// outside the tag's domain, and for every handle tag.
    pub fn try_immediate(tag: BoundaryTag, payload: u64) -> Option<Self> {
        boundary_immediate_admits(tag, payload)
            .then(|| BoundaryWord((payload << BOUNDARY_TAG_BITS) | (tag as u64)))
    }

    /// Build a handle word naming a node **in the region its tag selects**.
    ///
    /// ⛔ **The index is region-relative, and the tag says which region.** A
    /// persistent tag's index names a node in the store-owned
    /// [`BoundaryPersistentImage`], which outlives every invocation; an
    /// invocation tag's index names a node in the [`BoundaryArenaV1`], which does
    /// not. Reading one index against the other region is the defect this split
    /// exists to make unrepresentable: a persistent word must not be a locator
    /// into storage that dies with the activation that minted it.
    pub fn handle(tag: BoundaryTag, node_index: u64) -> Self {
        BoundaryWord((node_index << BOUNDARY_TAG_BITS) | (tag as u64))
    }

    /// The word's tag, or `None` if the byte is outside the closed set.
    pub fn tag(self) -> Option<BoundaryTag> {
        BoundaryTag::from_bits(self.0 & BOUNDARY_TAG_MASK)
    }

    /// The raw payload bits.
    pub fn payload(self) -> u64 {
        self.0 >> BOUNDARY_TAG_BITS
    }

    /// The payload read as a two's-complement signed integer.
    pub fn signed_payload(self) -> i64 {
        ((self.0 as i64) >> BOUNDARY_TAG_BITS) as i64
    }

    /// Whether `value` fits the immediate-`Int` range.
    ///
    /// A **runtime** magnitude test. Emitted code performs the identical test
    /// in CLIF; nothing here inspects a JIT-time value to choose a layout.
    pub fn int_fits_immediate(value: i64) -> bool {
        (BOUNDARY_IMMEDIATE_INT_MIN..=BOUNDARY_IMMEDIATE_INT_MAX).contains(&value)
    }
}

// ---------------------------------------------------------------------------
// Arena layout — the contract the CLIF graph reads
// ---------------------------------------------------------------------------

/// The class of an arena node. Reconciled with `AbiCarrier::GroundValueCarrier`,
/// whose documented family is exactly `Bool`, `Int`, `Bytes`, `String`,
/// `Constructor`, `Record`; this adds the two classes that are **not** ground
/// values and therefore never had a `GroundValueCarrier` image — a retained
/// closure and borrowed host ingress.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u64)]
pub enum BoundaryClass {
    Bool = 0,
    Int = 1,
    Bytes = 2,
    String = 3,
    Constructor = 4,
    Record = 5,
    HostResult = 6,
    Closure = 7,
    BorrowedOpaque = 8,
}

impl BoundaryClass {
    /// Every class, in declaration order.
    pub const ALL: [BoundaryClass; 9] = [
        BoundaryClass::Bool,
        BoundaryClass::Int,
        BoundaryClass::Bytes,
        BoundaryClass::String,
        BoundaryClass::Constructor,
        BoundaryClass::Record,
        BoundaryClass::HostResult,
        BoundaryClass::Closure,
        BoundaryClass::BorrowedOpaque,
    ];
}

// ---------------------------------------------------------------------------
// The tag × class relation
// ---------------------------------------------------------------------------

/// ⛔ **The valid `(tag, class)` pairs — one authoritative relation.**
///
/// A closed set of tags and a closed set of classes do **not** make a closed
/// ABI: the tag decides *lifetime and region*, the class decides
/// *interpretation*, and their product contains pairs no disposition can ever
/// produce. `PersistentClosure + HostResult` and `InvocationHostResult +
/// Constructor` are representable in the product and meaningless in the ABI —
/// minting one succeeds and then fails much later at an unrelated projection,
/// which reports the wrong defect at the wrong place.
///
/// This table is derived from `Lowered::boundary_disposition` and is the single
/// source both the Rust builders and the emitted allocator check against:
/// [`boundary_class_mask`] compiles it to the bitmask the CLIF tests.
///
/// Immediate tags are absent by construction — they have no node, so they have
/// no class.
pub const BOUNDARY_TAG_CLASS_RELATION: &[(BoundaryTag, &[BoundaryClass])] = &[
    (
        BoundaryTag::PersistentGround,
        // The ground classes plus the spill arm: an `Int` too wide for the
        // immediate field becomes a persistent ground handle.
        &[
            BoundaryClass::Int,
            BoundaryClass::Bytes,
            BoundaryClass::String,
            BoundaryClass::Constructor,
            BoundaryClass::Record,
        ],
    ),
    (BoundaryTag::PersistentClosure, &[BoundaryClass::Closure]),
    (
        BoundaryTag::InvocationBorrowed,
        &[BoundaryClass::BorrowedOpaque],
    ),
    (
        BoundaryTag::InvocationHostResult,
        &[BoundaryClass::HostResult],
    ),
];

/// Whether the ABI admits this `(tag, class)` pair.
pub fn boundary_relation_admits(tag: BoundaryTag, class: BoundaryClass) -> bool {
    BOUNDARY_TAG_CLASS_RELATION
        .iter()
        .any(|(t, classes)| *t == tag && classes.contains(&class))
}

/// The relation for one tag, as a bitmask over [`BoundaryClass`] discriminants.
///
/// ⭐ This is what makes the emitted check Θ(1): the allocator selects one mask
/// with four comparisons and tests one bit, rather than walking a table. The
/// mask is *computed from* the relation above, so the CLIF cannot drift from
/// the declaration — there is one table and one derivation.
pub fn boundary_class_mask(tag: BoundaryTag) -> u64 {
    BOUNDARY_TAG_CLASS_RELATION
        .iter()
        .filter(|(t, _)| *t == tag)
        .flat_map(|(_, classes)| classes.iter())
        .fold(0u64, |mask, class| mask | (1u64 << (*class as u64)))
}

// ---------------------------------------------------------------------------
// The immediate payload domain
// ---------------------------------------------------------------------------

/// What an immediate tag's 56-bit payload field is allowed to hold.
///
/// ⛔ **A closed tag set does not close the immediate space either.** The tag
/// says how to *read* the payload; nothing in the tag says which payloads are
/// *values*. Without this, minting an immediate is a shift — and a shift is
/// total, so an out-of-range magnitude silently becomes a **different value**
/// and a `Bool` payload of `2` becomes a third boolean. Same defect shape as the
/// Cartesian `tag × class` product, one field down.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum BoundaryImmediateDomain {
    /// Exactly `{0, 1}`.
    Bit = 0,
    /// Two's complement, representable in [`BOUNDARY_PAYLOAD_BITS`].
    SignedPayload = 1,
    /// Non-negative, representable in [`BOUNDARY_PAYLOAD_BITS`].
    UnsignedPayload = 2,
}

/// ⛔ **The payload domain of every immediate tag — one authoritative table.**
///
/// Handle tags are absent by construction: their payload is a node index the
/// allocator produced, and [`define_make_immediate`] refuses them outright
/// rather than admitting them with a domain.
///
/// ⚠ The three `Nat`-ish tags carry **`UnsignedPayload`**, which is the
/// *representational* bound and not a claim about their semantic range. A
/// process exit status is small in practice; the ABI does not know how small,
/// and inventing a tighter bound here would reject values the disposition
/// admits. Narrowing one is a contract decision, not a repair.
pub const BOUNDARY_IMMEDIATE_DOMAIN: &[(BoundaryTag, BoundaryImmediateDomain)] = &[
    (BoundaryTag::ImmediateBool, BoundaryImmediateDomain::Bit),
    (
        BoundaryTag::ImmediateInt,
        BoundaryImmediateDomain::SignedPayload,
    ),
    (
        BoundaryTag::ImmediateExitStatus,
        BoundaryImmediateDomain::UnsignedPayload,
    ),
    (
        BoundaryTag::ImmediateBoundedNat,
        BoundaryImmediateDomain::UnsignedPayload,
    ),
    (
        BoundaryTag::ImmediateStructuralNat,
        BoundaryImmediateDomain::UnsignedPayload,
    ),
];

/// The domain of an immediate tag, or `None` for a handle tag.
pub fn boundary_immediate_domain(tag: BoundaryTag) -> Option<BoundaryImmediateDomain> {
    BOUNDARY_IMMEDIATE_DOMAIN
        .iter()
        .find(|(t, _)| *t == tag)
        .map(|(_, domain)| *domain)
}

/// The tags in one domain, as a bitmask over [`BoundaryTag`] discriminants.
///
/// ⭐ What makes the emitted check Θ(1) and undriftable, exactly as
/// [`boundary_class_mask`] does for the relation: the CLIF evaluates all three
/// domain predicates and selects by a mask **computed from this table**, so
/// there is no second place to edit.
pub fn boundary_domain_mask(domain: BoundaryImmediateDomain) -> u64 {
    BOUNDARY_IMMEDIATE_DOMAIN
        .iter()
        .filter(|(_, d)| *d == domain)
        .fold(0u64, |mask, (tag, _)| mask | (1u64 << (*tag as u64)))
}

/// Whether `payload` is a value of `tag`'s immediate domain.
///
/// `false` for every handle tag — a handle's payload is an index the allocator
/// mints, never a caller's number.
pub fn boundary_immediate_admits(tag: BoundaryTag, payload: u64) -> bool {
    match boundary_immediate_domain(tag) {
        None => false,
        Some(BoundaryImmediateDomain::Bit) => payload <= 1,
        Some(BoundaryImmediateDomain::UnsignedPayload) => payload >> BOUNDARY_PAYLOAD_BITS == 0,
        Some(BoundaryImmediateDomain::SignedPayload) => {
            BoundaryWord::int_fits_immediate(payload as i64)
        }
    }
}

// ---------------------------------------------------------------------------
// How a spilled `Int` carries its magnitude
// ---------------------------------------------------------------------------

/// A spilled `Int` node's magnitude is in the **region's limb table**, at
/// [`NODE_LIMBS_AT`] for [`NODE_LIMB_COUNT`] limbs, sign in [`NODE_PAYLOAD`].
///
/// ⭐ **This is what makes an arbitrary-precision `Int` genuinely persistable,
/// and it is the SAME region-selection rule every other class already obeys.** A
/// `Bytes`'s content is in its region's data table, a `Constructor`'s children
/// are in its region's word table — a persistent value's content belongs to the
/// persistent region, not to a table that dies with an invocation. A
/// [`crate::native_int::NATIVE_INT_BIG_TAG_V1`] payload is a slot in the
/// *invocation's* `NativeIntArenaV1`, so it is correct for an invocation-scoped
/// result and can never be persistent; this marker is its persistent
/// counterpart, and the two are not interchangeable.
pub const BOUNDARY_INT_REGION_LIMBS: u64 = 2;

/// The closed set of magnitude markers a spilled `Int` node's [`NODE_EXTENT`]
/// may hold, with the region each one's storage lives in.
///
/// ⛔ Both enforcement points read this one table: the emitted
/// `ken_boundary_store_int_tag_local` admits a marker only for a node whose
/// referent owner matches, and the Rust builders assert the same. A marker with
/// no row admits nothing.
pub const BOUNDARY_INT_MARKER_OWNER: &[(u64, BoundaryReferentOwner)] = &[
    (
        crate::native_int::NATIVE_INT_SMALL_TAG_V1,
        // A `Small`'s magnitude IS the payload word — no storage, so it is
        // sound in either region.
        BoundaryReferentOwner::NoReferent,
    ),
    (
        crate::native_int::NATIVE_INT_BIG_TAG_V1,
        BoundaryReferentOwner::InvocationArena,
    ),
    (
        BOUNDARY_INT_REGION_LIMBS,
        BoundaryReferentOwner::PersistentStore,
    ),
];

/// Whether a node owned by `owner` may carry magnitude marker `marker`.
///
/// `NoReferent` in the table means "any region" — a `Small` carries its whole
/// magnitude in the node and names no storage at all.
pub fn boundary_int_marker_admits(marker: u64, owner: BoundaryReferentOwner) -> bool {
    BOUNDARY_INT_MARKER_OWNER.iter().any(|(m, required)| {
        *m == marker && (*required == BoundaryReferentOwner::NoReferent || *required == owner)
    })
}

/// The markers admitted for one owner, as a bitmask over marker values.
///
/// ⭐ Θ(1) in the emitted check and computed from the table above, so the CLIF
/// cannot drift from the declaration — the third instance of this pattern, after
/// [`boundary_class_mask`] and [`boundary_domain_mask`].
pub fn boundary_int_marker_mask(owner: BoundaryReferentOwner) -> u64 {
    BOUNDARY_INT_MARKER_OWNER
        .iter()
        .filter(|(_, required)| {
            *required == BoundaryReferentOwner::NoReferent || *required == owner
        })
        .fold(0u64, |mask, (marker, _)| mask | (1u64 << *marker))
}

/// Whether `(sign, limbs)` is a canonical exact-`Int` magnitude.
///
/// ⛔ **The one statement of the contract `RuntimeIntV1::canonical_sign_and_limbs`
/// produces**, so the Rust builder's assertion and the emitted seal are checking
/// the same thing rather than two hand-written approximations of it:
///
/// - **at least one limb** — an empty magnitude denotes no integer;
/// - **no leading zero limb** — least-significant first, so a zero in the top
///   position means the same value has two encodings;
/// - **zero is non-negative** — negative zero is a second encoding of zero.
///
/// ⚠ A one-limb `[0]` *is* canonical: that is the value zero. Rejecting it would
/// be an over-strengthening the contract does not entail.
pub fn boundary_int_magnitude_is_canonical(sign: u64, limbs: &[u64]) -> bool {
    if sign > 1 || limbs.is_empty() {
        return false;
    }
    let zero = limbs == [0];
    let top_ok = limbs.last() != Some(&0) || zero;
    top_ok && !(zero && sign == 1)
}

/// Byte stride of one arena node.
pub const BOUNDARY_NODE_STRIDE: i32 = 88;

/// `BoundaryClass` of this node.
pub const NODE_CLASS: i32 = 0;
/// `BoundaryReferentOwner` of this node's referent.
pub const NODE_OWNER: i32 = 8;
/// The `SlotId` that owns this node's value, or `NULL_SLOT` when the owner is
/// the invocation arena. **This field is what makes `AC-6` observable.**
pub const NODE_SLOT: i32 = 16;
/// Interned constructor symbol / record type identity, or `0`.
pub const NODE_TAG_ID: i32 = 24;
/// Scalar payload: bool bit, small-int value, `HostResult` success flag, or the
/// byte length of a `Bytes`/`String`.
pub const NODE_PAYLOAD: i32 = 32;
/// Number of child words this node has.
pub const NODE_FIELD_COUNT: i32 = 40;
/// Index into the word table of this node's first child word. Field *names*
/// live at the same index in the name table.
pub const NODE_FIELDS_AT: i32 = 48;
/// A second scalar whose meaning the **class** determines, exactly as
/// [`NODE_PAYLOAD`]'s already does:
///
/// | class | `NODE_PAYLOAD` | `NODE_EXTENT` |
/// |---|---|---|
/// | `Int` | the [`crate::native_int::NativeIntV1`] payload | its `tag` |
/// | `Bytes` / `String` | byte length | start index in the region's data table |
/// | everything else | as documented on `NODE_PAYLOAD` | `0`, unread |
///
/// ⚠ Every reader of this field is **class-guarded**, so a caller cannot read
/// one class's meaning out of another's node. A single un-guarded reader would
/// make the two meanings collide, which is why there is no generic accessor.
pub const NODE_EXTENT: i32 = 56;
/// Index into the region's **limb table** of a spilled `Int`'s first limb.
///
/// ⛔ A dedicated field and a dedicated table, deliberately — not a reuse of
/// [`NODE_FIELDS_AT`] and the word table. `ken_boundary_field_local` and
/// `ken_boundary_field_count_local` are **not** class-guarded, so limbs parked
/// in the word table would be readable as child *words*: a raw magnitude limb
/// returned where a tagged `BoundaryWord` is expected. Two meanings for one
/// table is exactly the collision `NODE_EXTENT`'s note warns about, and the
/// cheap fix is storage that cannot be reached by the wrong reader at all.
pub const NODE_LIMBS_AT: i32 = 64;
/// Number of limbs a spilled `Int` node's magnitude has. Zero for every other
/// class and for a `Small`.
pub const NODE_LIMB_COUNT: i32 = 72;
/// ⛔ **`1` once a region-limbed `Int`'s magnitude has been checked CANONICAL,
/// `0` while it is still being written.** Every reader of a region-limbed
/// magnitude requires it, so an unsealed node **denotes nothing**.
///
/// This exists because canonicity is not checkable when the span is claimed.
/// `store_int_limbs` runs before a single limb is written, so it can bound the
/// length and the sign and nothing else — it cannot see a leading zero limb, it
/// cannot see negative zero, and it cannot see a producer that claims three
/// limbs and writes two. Those are properties of the *finished* magnitude, and a
/// finished magnitude needs a completion step to be a thing the ABI can talk
/// about at all.
///
/// ⭐ **The seal is what makes "fails closed before publication" true rather
/// than aspirational.** The node exists and its word is in the producer's hand
/// the moment `alloc` returns; what a consumer can do with it is the only
/// meaningful sense of published, and until the seal a consumer can do nothing.
pub const NODE_INT_SEALED: i32 = 80;

/// Byte size of a **region header**.
///
/// ⭐ One layout serves both regions. The invocation arena and the persistent
/// image publish the *same* header shape, which is what lets a single
/// `resolve` select a region at run time and then read it with one set of
/// offsets. A second layout would be a second place for the offsets to drift.
pub const BOUNDARY_REGION_HEADER_BYTES: i32 = 136;

/// Pointer to the node table.
pub const ARENA_NODES: i32 = 0;
/// Number of **live** nodes. ⚠ Mutable: the emitted allocator bumps it.
pub const ARENA_NODE_COUNT: i32 = 8;
/// Pointer to the child-word table.
pub const ARENA_WORDS: i32 = 16;
/// Number of **live** child words. ⚠ Mutable: the emitted allocator bumps it.
pub const ARENA_WORD_COUNT: i32 = 24;
/// Pointer to the field-name-id table, parallel to the word table.
pub const ARENA_NAMES: i32 = 32;
/// Number of field-name ids.
pub const ARENA_NAME_COUNT: i32 = 40;
/// Node capacity — the ceiling the emitted allocator fails closed against.
pub const ARENA_NODE_CAPACITY: i32 = 48;
/// Child-word capacity — the other ceiling.
pub const ARENA_WORD_CAPACITY: i32 = 56;
/// Pointer to the **persistent region's** header, or `0` when this invocation
/// is bound to no persistent storage. Read from the *arena* header only.
pub const ARENA_PERSISTENT: i32 = 64;
/// Nodes present when the region was published.
///
/// ⛔ **The frozen prefix.** Emitted code may construct nodes at or beyond this
/// index and may mutate only those. A node the Rust side materialized carries
/// the store's [`SlotId`], and letting emitted code rewrite that field would let
/// it forge persistent identity — the store must remain the sole identity
/// authority, so the boundary is a bounds check rather than a convention.
pub const ARENA_FROZEN: i32 = 72;
/// Pointer to the region's **data table** — the byte span backing `Bytes` and
/// `String` contents.
pub const ARENA_DATA: i32 = 80;
/// Number of **live** data bytes. ⚠ Mutable: the emitted allocator bumps it.
pub const ARENA_DATA_COUNT: i32 = 88;
/// Data-table capacity — the third ceiling construction fails closed against.
pub const ARENA_DATA_CAPACITY: i32 = 96;
/// Pointer to the invocation's [`crate::native_int::NativeIntArenaV1`] header,
/// or `0`.
///
/// ⭐ **The connection to the landed exact-`Int` representation.** A spilled
/// `Int` node carries a native `(tag, payload)` pair and nothing else; emitted
/// code decodes it by calling `ken_native_int_resolve_local`, the *existing*
/// executable decoder. Re-deriving sign and limbs here would be a second exact
/// integer representation, which is the thing `docs/PRINCIPLES.md` calls
/// subsume-don't-proliferate. Read from the *arena* header only — the native
/// arena is invocation state.
pub const ARENA_NATIVE_INT: i32 = 104;
/// Pointer to the region's **limb table** — the `u64` magnitude storage backing
/// a spilled `Int` whose marker is [`BOUNDARY_INT_REGION_LIMBS`].
///
/// ⭐ Region-owned, which is the whole point: a persistent `Int`'s limbs outlive
/// every invocation because they live where the persistent nodes do.
pub const ARENA_LIMBS: i32 = 112;
/// Number of **live** limbs. ⚠ Mutable: the emitted allocator bumps it.
pub const ARENA_LIMB_COUNT: i32 = 120;
/// Limb-table capacity — the fourth ceiling construction fails closed against.
pub const ARENA_LIMB_CAPACITY: i32 = 128;

/// ⛔ **Every region-header field, named, in one list.**
///
/// The declared size and the published bytes were allowed to disagree once —
/// `publish` emitted 18 words against a 136-byte (17-word) constant that had **no
/// consumer anywhere in the tree**, so the layout claim was unenforced in both
/// directions at once. `publish` now writes *through* these offsets (a stale
/// constant is an out-of-bounds panic), and this list closes the other
/// direction: the pin asserts the offsets are exactly `0, 8, …` up to
/// [`BOUNDARY_REGION_HEADER_BYTES`], so a **new field without a size bump** and a
/// **size bump without a field** both redden.
///
/// ⭐ The allowed inventory, not a forbidden list: any addition must appear here
/// to pass, including one nobody imagined.
pub const BOUNDARY_REGION_HEADER_FIELDS: &[(&str, i32)] = &[
    ("ARENA_NODES", ARENA_NODES),
    ("ARENA_NODE_COUNT", ARENA_NODE_COUNT),
    ("ARENA_WORDS", ARENA_WORDS),
    ("ARENA_WORD_COUNT", ARENA_WORD_COUNT),
    ("ARENA_NAMES", ARENA_NAMES),
    ("ARENA_NAME_COUNT", ARENA_NAME_COUNT),
    ("ARENA_NODE_CAPACITY", ARENA_NODE_CAPACITY),
    ("ARENA_WORD_CAPACITY", ARENA_WORD_CAPACITY),
    ("ARENA_PERSISTENT", ARENA_PERSISTENT),
    ("ARENA_FROZEN", ARENA_FROZEN),
    ("ARENA_DATA", ARENA_DATA),
    ("ARENA_DATA_COUNT", ARENA_DATA_COUNT),
    ("ARENA_DATA_CAPACITY", ARENA_DATA_CAPACITY),
    ("ARENA_NATIVE_INT", ARENA_NATIVE_INT),
    ("ARENA_LIMBS", ARENA_LIMBS),
    ("ARENA_LIMB_COUNT", ARENA_LIMB_COUNT),
    ("ARENA_LIMB_CAPACITY", ARENA_LIMB_CAPACITY),
];

/// Every node field, named, in one list — the node's half of the same closure.
///
/// `push_node` writes exactly `NODE_WORDS` entries, and `NODE_WORDS` is derived
/// from [`BOUNDARY_NODE_STRIDE`]; the pin ties this list to both, so a field
/// added to the node without a stride bump is a compile-time length mismatch or
/// a red test rather than a silently truncated write.
pub const BOUNDARY_NODE_FIELDS: &[(&str, i32)] = &[
    ("NODE_CLASS", NODE_CLASS),
    ("NODE_OWNER", NODE_OWNER),
    ("NODE_SLOT", NODE_SLOT),
    ("NODE_TAG_ID", NODE_TAG_ID),
    ("NODE_PAYLOAD", NODE_PAYLOAD),
    ("NODE_FIELD_COUNT", NODE_FIELD_COUNT),
    ("NODE_FIELDS_AT", NODE_FIELDS_AT),
    ("NODE_EXTENT", NODE_EXTENT),
    ("NODE_LIMBS_AT", NODE_LIMBS_AT),
    ("NODE_LIMB_COUNT", NODE_LIMB_COUNT),
    ("NODE_INT_SEALED", NODE_INT_SEALED),
];

/// Status returned by every emitted-code helper on success.
pub const BOUNDARY_OK: i64 = 0;
/// The word's tag byte is outside the closed set.
pub const BOUNDARY_ERR_TAG: i64 = -1;
/// The word is an immediate where a handle was required, or the reverse.
pub const BOUNDARY_ERR_SHAPE: i64 = -2;
/// A node index, field index or name lookup left its region's bounds.
pub const BOUNDARY_ERR_BOUNDS: i64 = -3;
/// The node's class does not admit the requested projection.
pub const BOUNDARY_ERR_CLASS: i64 = -4;
/// ⛔ Borrowed ingress attempted to escape the native invocation (`AC-7`), or a
/// persistent node was handed an invocation-owned child (`AC-6`, one layer
/// down: a surviving structure must not embed a locator that dies first).
pub const BOUNDARY_ERR_ESCAPE: i64 = -5;
/// ⛔ Construction exhausted the region's reservation. Fail-closed: emitted code
/// never grows a region, because growth would move it under a published
/// pointer.
pub const BOUNDARY_ERR_CAPACITY: i64 = -6;
/// ⛔ The `(tag, class)` pair is outside the ABI's valid relation — a closed set
/// of tags and a closed set of classes do not make a closed relation.
pub const BOUNDARY_ERR_RELATION: i64 = -8;
/// ⛔ Construction targeted a node in the region's frozen prefix — a node the
/// Rust side materialized and whose store identity is not emitted code's to
/// rewrite.
pub const BOUNDARY_ERR_FROZEN: i64 = -7;

// ---------------------------------------------------------------------------
// The invocation-scoped arena
// ---------------------------------------------------------------------------

/// The flat node/word/name tables emitted code projects out of.
///
/// ⭐ **A container, not a lifetime.** The same layout backs both regions; what
/// differs is *who owns the storage and how long it lives*, and that is carried
/// by the two newtypes below rather than by a flag on this struct. A word's tag
/// selects the region, so the layout must be identical and the ownership must
/// not be.
#[derive(Debug, Default)]
pub struct BoundaryRegion {
    nodes: Vec<u64>,
    words: Vec<u64>,
    names: Vec<u64>,
    /// Backing bytes for `Bytes` / `String` contents.
    data: Vec<u8>,
    /// Backing `u64` magnitude limbs for spilled `Int` contents.
    limbs: Vec<u64>,
    live_nodes: usize,
    live_words: usize,
    live_data: usize,
    live_limbs: usize,
    header: Vec<u64>,
    /// Address of the persistent region's header, or `0`.
    persistent: u64,
    /// Address of the invocation's native-`Int` arena header, or `0`.
    native_int: u64,
}

const NODE_WORDS: usize = BOUNDARY_NODE_STRIDE as usize / 8;

impl BoundaryRegion {
    /// Number of **live** nodes.
    ///
    /// ⭐ Reads the published header once published, because the emitted
    /// allocator bumps that field directly. A Rust-side mirror would answer a
    /// stale count for exactly the nodes this node exists to let emitted code
    /// build.
    pub fn node_count(&self) -> usize {
        match self.header.first() {
            None => self.live_nodes,
            Some(_) => self.header[(ARENA_NODE_COUNT / 8) as usize] as usize,
        }
    }

    /// Number of live child words, on the same published-header rule.
    pub fn word_count(&self) -> usize {
        match self.header.first() {
            None => self.live_words,
            Some(_) => self.header[(ARENA_WORD_COUNT / 8) as usize] as usize,
        }
    }

    /// Number of live data bytes, on the same published-header rule.
    pub fn data_count(&self) -> usize {
        match self.header.first() {
            None => self.live_data,
            Some(_) => self.header[(ARENA_DATA_COUNT / 8) as usize] as usize,
        }
    }

    /// Words in the published header, or `0` before publication. The layout
    /// pin measures this rather than re-deriving the constant it is checking.
    pub fn published_header_len(&self) -> usize {
        self.header.len()
    }

    /// Number of live magnitude limbs, on the same published-header rule.
    pub fn limb_count(&self) -> usize {
        match self.header.first() {
            None => self.live_limbs,
            Some(_) => self.header[(ARENA_LIMB_COUNT / 8) as usize] as usize,
        }
    }

    /// Nodes this region can still hold beyond the live count.
    pub fn node_capacity(&self) -> usize {
        self.nodes.len() / NODE_WORDS
    }

    /// The live data bytes of one node's span, or `None` when the node is not
    /// a `Bytes`/`String` or its span leaves the table.
    ///
    /// The Rust-side mirror of the CLIF bounds checks, used by tests as an
    /// independent oracle rather than by re-reading the CLIF's own answer.
    pub fn node_data(&self, index: u64) -> Option<&[u8]> {
        let class = self.node_field(index, NODE_CLASS)?;
        if class != BoundaryClass::Bytes as u64 && class != BoundaryClass::String as u64 {
            return None;
        }
        let at = self.node_field(index, NODE_EXTENT)? as usize;
        let len = self.node_field(index, NODE_PAYLOAD)? as usize;
        let end = at.checked_add(len)?;
        (end <= self.data_count()).then(|| &self.data[at..end])
    }

    /// Read one field of one live node. `None` when the index or offset is out
    /// of range — the Rust-side mirror of the CLIF bounds checks, used by tests
    /// as an independent oracle rather than by re-reading the CLIF's own answer.
    pub fn node_field(&self, index: u64, offset: i32) -> Option<u64> {
        if index as usize >= self.node_count() {
            return None;
        }
        let base = (index as usize).checked_mul(NODE_WORDS)?;
        self.nodes.get(base + (offset as usize / 8)).copied()
    }

    /// The child word at an absolute word-table index.
    pub fn word_at(&self, index: u64) -> Option<BoundaryWord> {
        if index as usize >= self.word_count() {
            return None;
        }
        self.words.get(index as usize).copied().map(BoundaryWord)
    }

    /// The field-name id at an absolute name-table index.
    pub fn name_at(&self, index: u64) -> Option<u64> {
        self.names.get(index as usize).copied()
    }

    /// Reserve room for `nodes` further nodes and `words` further child words.
    ///
    /// ⛔ **This is the whole storage grant emitted code gets.** The allocator
    /// bumps the live counts within the reservation and returns
    /// [`BOUNDARY_ERR_CAPACITY`] past it; it never grows a table, because
    /// growing one would move it out from under the published pointer. Reserving
    /// is therefore the caller's explicit, auditable decision about how much
    /// storage an invocation may take.
    pub fn reserve(&mut self, nodes: usize, words: usize, data: usize, limbs: usize) {
        debug_assert!(
            self.header.is_empty(),
            "reserve before publish: growing a table moves it under the pointer"
        );
        let node_words = (self.live_nodes + nodes) * NODE_WORDS;
        self.nodes.resize(node_words, 0);
        self.words.resize(self.live_words + words, 0);
        self.names.resize(self.live_words + words, 0);
        self.data.resize(self.live_data + data, 0);
        self.limbs.resize(self.live_limbs + limbs, 0);
    }

    /// The live magnitude limbs of one spilled `Int` node, or `None` when the
    /// node is not a region-limbed `Int` or its span leaves the table.
    ///
    /// The Rust-side mirror of the CLIF bounds checks, used by tests as an
    /// independent oracle rather than by re-reading the CLIF's own answer.
    pub fn node_limbs(&self, index: u64) -> Option<&[u64]> {
        if self.node_field(index, NODE_CLASS)? != BoundaryClass::Int as u64
            || self.node_field(index, NODE_EXTENT)? != BOUNDARY_INT_REGION_LIMBS
        {
            return None;
        }
        if self.node_field(index, NODE_INT_SEALED)? != 1 {
            return None;
        }
        let at = self.node_field(index, NODE_LIMBS_AT)? as usize;
        let len = self.node_field(index, NODE_LIMB_COUNT)? as usize;
        let end = at.checked_add(len)?;
        (end <= self.limb_count()).then(|| &self.limbs[at..end])
    }

    /// Overwrite one raw field of one node — **fault injection, tests only**.
    ///
    /// ⛔ There is no production path that can produce a *stale or malformed*
    /// node span: the Rust builder computes spans from its own live counts and
    /// the emitted helpers bounds-check every write. So the reader's
    /// wraparound guard has **no reachable producer to exercise it**, and a
    /// control that cannot construct the malformed input is not evidence about
    /// the guard — it is the "pin that never exercises the violating mechanism"
    /// shape again. This injects the corruption directly, which is the only way
    /// to ask the question at all.
    #[cfg(test)]
    pub fn poke_node_field(&mut self, index: u64, offset: i32, value: u64) {
        let base = index as usize * NODE_WORDS;
        self.nodes[base + (offset as usize / 8)] = value;
    }

    /// Append `limbs` to the limb table, returning its start index.
    fn push_limbs(&mut self, limbs: &[u64]) -> u64 {
        let at = self.live_limbs as u64;
        let end = self.live_limbs + limbs.len();
        if self.limbs.len() < end {
            self.limbs.resize(end, 0);
        }
        self.limbs[self.live_limbs..end].copy_from_slice(limbs);
        self.live_limbs = end;
        at
    }

    /// Append `bytes` to the data table, returning its start index.
    fn push_data(&mut self, bytes: &[u8]) -> u64 {
        let at = self.live_data as u64;
        let end = self.live_data + bytes.len();
        if self.data.len() < end {
            self.data.resize(end, 0);
        }
        self.data[self.live_data..end].copy_from_slice(bytes);
        self.live_data = end;
        at
    }

    /// Append one node and return the handle word naming it.
    #[allow(clippy::too_many_arguments)]
    fn push_node(
        &mut self,
        tag: BoundaryTag,
        class: BoundaryClass,
        slot: SlotId,
        tag_id: u64,
        payload: u64,
        extent: u64,
        children: &[BoundaryWord],
        names: &[u64],
        limbs: &[u64],
    ) -> BoundaryWord {
        // ⛔ The Rust builders check the SAME relation the emitted allocator
        // does. One table, two enforcement points — a pair no disposition can
        // produce must be unbuildable from either side.
        assert!(
            boundary_relation_admits(tag, class),
            "the ABI does not admit {tag:?} + {class:?}"
        );
        // ⛔ And the SAME magnitude-marker table, for the same reason: a marker
        // whose storage the node's region does not own is the ephemeral-locator
        // defect, and it must be unbuildable from Rust exactly as it is from
        // emitted code.
        debug_assert!(
            class != BoundaryClass::Int || boundary_int_marker_admits(extent, tag.referent_owner()),
            "a {:?} Int may not carry magnitude marker {extent}",
            tag.referent_owner()
        );
        debug_assert!(
            limbs.is_empty() || extent == BOUNDARY_INT_REGION_LIMBS,
            "limbs belong only to a region-limbed Int"
        );
        // ⛔ The SAME canonicity contract the emitted seal enforces, asserted at
        // the other producer. `RuntimeIntV1::canonical_sign_and_limbs` is the
        // authority: at least one limb, least-significant first with no leading
        // zero limb, and zero is non-negative.
        debug_assert!(
            extent != BOUNDARY_INT_REGION_LIMBS
                || boundary_int_magnitude_is_canonical(payload, limbs),
            "a region-limbed Int must carry a canonical magnitude"
        );
        debug_assert!(
            names.is_empty() || names.len() == children.len(),
            "a name table, when present, is parallel to the word table"
        );
        debug_assert!(
            self.header.is_empty(),
            "Rust-side materialization happens before publish"
        );
        let index = self.live_nodes as u64;
        let fields_at = self.live_words as u64;
        // The name table stays parallel to the word table for EVERY node, so a
        // record's names sit at exactly its children's indices. Non-records pad
        // with zero rather than shifting the two tables out of step.
        let end = self.live_words + children.len();
        if self.words.len() < end {
            self.words.resize(end, 0);
            self.names.resize(end, 0);
        }
        for (offset, child) in children.iter().enumerate() {
            self.words[self.live_words + offset] = child.0;
            self.names[self.live_words + offset] = names.get(offset).copied().unwrap_or(0);
        }
        self.live_words = end;
        let limbs_at = self.push_limbs(limbs);

        let base = index as usize * NODE_WORDS;
        if self.nodes.len() < base + NODE_WORDS {
            self.nodes.resize(base + NODE_WORDS, 0);
        }
        self.nodes[base..base + NODE_WORDS].copy_from_slice(&[
            class as u64,
            tag.referent_owner() as u64,
            slot,
            tag_id,
            payload,
            children.len() as u64,
            fields_at,
            extent,
            limbs_at,
            limbs.len() as u64,
            // Rust-materialized magnitudes come from `canonical_sign_and_limbs`
            // and are asserted canonical above, so they are born sealed. Emitted
            // construction earns the seal from `ken_boundary_seal_int_local`.
            u64::from(extent == BOUNDARY_INT_REGION_LIMBS),
        ]);
        self.live_nodes = index as usize + 1;
        BoundaryWord::handle(tag, index)
    }

    /// Publish the header and hand back the pointer emitted code reads.
    ///
    /// # Safety contract
    ///
    /// The returned pointer is valid only while `self` is alive and neither
    /// re-materialized into nor re-reserved. Emitted code **writes** through it
    /// — the live counts and the reserved node/word storage are mutable — so the
    /// pointer is `*mut`, and the region must be held mutably for the extent it
    /// is published, exactly as `NativeIntArenaV1` holds its own header.
    /// ⛔ **Sized from [`BOUNDARY_REGION_HEADER_BYTES`] and written through the
    /// offset constants, so the declared layout and the published bytes cannot
    /// disagree.** The previous form was a positional `vec![…]` whose length
    /// nobody derived and nobody checked: it published **18** words where the
    /// constant declared **17**, the constant had no consumer anywhere in the
    /// tree, and the reviewed "112 → 136" layout claim was therefore *false and
    /// unenforced*. A positional literal makes the constant decorative — the
    /// bytes are correct only if a reader counted the lines. Indexing by the
    /// offsets makes a stale constant an out-of-bounds panic, and
    /// [`BOUNDARY_REGION_HEADER_FIELDS`] closes the other direction.
    pub fn publish(&mut self) -> *mut u64 {
        let mut header = vec![0u64; (BOUNDARY_REGION_HEADER_BYTES / 8) as usize];
        header[(ARENA_NODES / 8) as usize] = self.nodes.as_ptr() as u64;
        header[(ARENA_NODE_COUNT / 8) as usize] = self.live_nodes as u64;
        header[(ARENA_WORDS / 8) as usize] = self.words.as_ptr() as u64;
        header[(ARENA_WORD_COUNT / 8) as usize] = self.live_words as u64;
        header[(ARENA_NAMES / 8) as usize] = self.names.as_ptr() as u64;
        header[(ARENA_NAME_COUNT / 8) as usize] = self.names.len() as u64;
        header[(ARENA_NODE_CAPACITY / 8) as usize] = (self.nodes.len() / NODE_WORDS) as u64;
        header[(ARENA_WORD_CAPACITY / 8) as usize] = self.words.len() as u64;
        header[(ARENA_PERSISTENT / 8) as usize] = self.persistent;
        // Everything materialized before publication is frozen; emitted code
        // constructs strictly beyond it.
        header[(ARENA_FROZEN / 8) as usize] = self.live_nodes as u64;
        header[(ARENA_DATA / 8) as usize] = self.data.as_ptr() as u64;
        header[(ARENA_DATA_COUNT / 8) as usize] = self.live_data as u64;
        header[(ARENA_DATA_CAPACITY / 8) as usize] = self.data.len() as u64;
        header[(ARENA_NATIVE_INT / 8) as usize] = self.native_int;
        header[(ARENA_LIMBS / 8) as usize] = self.limbs.as_ptr() as u64;
        header[(ARENA_LIMB_COUNT / 8) as usize] = self.live_limbs as u64;
        header[(ARENA_LIMB_CAPACITY / 8) as usize] = self.limbs.len() as u64;
        debug_assert_eq!(
            header.len() * std::mem::size_of::<u64>(),
            BOUNDARY_REGION_HEADER_BYTES as usize,
            "the published header must be exactly the declared layout"
        );
        self.header = header;
        self.header.as_mut_ptr()
    }
}

/// The **invocation-scoped** region.
///
/// ⛔ **Not a parallel permanent heap** (`D2`), and now structurally so: every
/// node here dies with the invocation, and no persistent word names one. A
/// persistent aggregate lives in [`BoundaryPersistentImage`] and is reached
/// through the persistent pointer this arena carries — so the arena is a *route*
/// to persistent storage, never its owner.
#[derive(Debug, Default)]
pub struct BoundaryArenaV1(pub BoundaryRegion);

impl BoundaryArenaV1 {
    /// Bind the persistent region this invocation resolves persistent words
    /// through. `None` leaves the invocation bound to no persistent storage, in
    /// which case every persistent word fails closed with
    /// [`BOUNDARY_ERR_BOUNDS`] rather than being read against the arena.
    pub fn bind_persistent(&mut self, region: Option<*const u64>) {
        self.0.persistent = region.map_or(0, |p| p as u64);
    }

    /// Bind the invocation's native-`Int` arena, through which emitted code
    /// decodes a spilled `Int`'s `(tag, payload)` pair. `None` leaves spilled
    /// integers undecodable, failing closed rather than reading zero.
    pub fn bind_native_int(&mut self, arena: Option<*const u64>) {
        self.0.native_int = arena.map_or(0, |p| p as u64);
    }

    /// Number of live invocation nodes.
    pub fn node_count(&self) -> usize {
        self.0.node_count()
    }

    /// Read one field of one live invocation node.
    pub fn node_field(&self, index: u64, offset: i32) -> Option<u64> {
        self.0.node_field(index, offset)
    }

    /// The child word at an absolute word-table index.
    pub fn word_at(&self, index: u64) -> Option<BoundaryWord> {
        self.0.word_at(index)
    }

    /// The field-name id at an absolute name-table index.
    pub fn name_at(&self, index: u64) -> Option<u64> {
        self.0.name_at(index)
    }

    /// Grant emitted code room to construct invocation-owned nodes.
    pub fn reserve(&mut self, nodes: usize, words: usize, data: usize, limbs: usize) {
        self.0.reserve(nodes, words, data, limbs);
    }

    /// Publish the arena header. See [`BoundaryRegion::publish`].
    pub fn publish(&mut self) -> *mut u64 {
        self.0.publish()
    }
}

/// The **store-owned** region: persistent aggregates, outliving every
/// invocation.
///
/// ⭐ **This is what makes a persistent word a persistent identity.** A
/// `PersistentGround` / `PersistentClosure` word's payload indexes *this* table,
/// which the [`BoundaryValueStore`] owns for the store's whole life. The word
/// survives the arena that minted it, and resolving it after that arena is gone
/// reaches the same node with the same [`SlotId`]. A persistent tag on an
/// invocation-arena index would be the contradiction the Architect measured: a
/// word permitted to escape that names storage which is already freed.
///
/// ## ⚠ Residual — emitted-constructed nodes are not content-addressed
///
/// A node the **store** materialized carries its [`SlotId`], and equal values
/// are one node because the store says so. A node **emitted code** constructs
/// carries [`NULL_SLOT`]: it survives the invocation and is fully readable, but
/// two structurally equal constructions are two nodes. Interning is a
/// content-addressing operation over a whole value; it is not Θ(1) at a
/// construction site, so it is not something the emitted allocator can do.
///
/// Recorded rather than papered over. It does **not** make this a second
/// identity authority — the store still mints every [`SlotId`], still governs
/// how much space construction may take, and can walk the nodes past
/// [`ARENA_FROZEN`] to adopt them. Closing the gap means deciding *when* an
/// invocation's constructed nodes are interned, which is a `B2F` question about
/// the invocation lifecycle rather than a `B2V` question about representation.
/// `b2v_a_constructed_persistent_word_survives_the_invocation_arena` asserts the
/// `NULL_SLOT`, so the limit is pinned instead of merely written down.
#[derive(Debug, Default)]
pub struct BoundaryPersistentImage(pub BoundaryRegion);

impl BoundaryPersistentImage {
    /// Number of live persistent nodes.
    pub fn node_count(&self) -> usize {
        self.0.node_count()
    }

    /// Read one field of one live persistent node.
    pub fn node_field(&self, index: u64, offset: i32) -> Option<u64> {
        self.0.node_field(index, offset)
    }

    /// The child word at an absolute word-table index.
    pub fn word_at(&self, index: u64) -> Option<BoundaryWord> {
        self.0.word_at(index)
    }

    /// The field-name id at an absolute name-table index.
    pub fn name_at(&self, index: u64) -> Option<u64> {
        self.0.name_at(index)
    }

    /// Grant emitted code room to construct persistent nodes.
    pub fn reserve(&mut self, nodes: usize, words: usize, data: usize, limbs: usize) {
        self.0.reserve(nodes, words, data, limbs);
    }

    /// The live data bytes of one node's span.
    pub fn node_data(&self, index: u64) -> Option<&[u8]> {
        self.0.node_data(index)
    }

    /// Publish the persistent header. See [`BoundaryRegion::publish`].
    pub fn publish(&mut self) -> *mut u64 {
        self.0.publish()
    }
}

/// Builds the invocation-scoped [`BoundaryArenaV1`].
///
/// ⛔ Holds no environment and no seed value. Its whole input is a class, a
/// payload and a child list — `AC-2` by construction rather than by assertion.
///
/// ⛔ **Invocation-owned nodes only.** Ground values are persistent and are
/// materialized through [`BoundaryValueStore`]; this builder cannot mint a
/// persistent word, so the arena cannot become the referent of one.
#[derive(Debug, Default)]
pub struct BoundaryArenaBuilder {
    arena: BoundaryArenaV1,
}

impl BoundaryArenaBuilder {
    /// A fresh, empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one invocation-owned node and return the handle word naming it.
    ///
    /// # Panics
    ///
    /// If `tag` is not invocation-owned. That is a programming error in this
    /// crate, not a runtime input: the persistent arm has its own path, and
    /// silently accepting a persistent tag here would rebuild the exact defect
    /// the region split closes.
    pub fn push_node(
        &mut self,
        tag: BoundaryTag,
        class: BoundaryClass,
        payload: u64,
        children: &[BoundaryWord],
    ) -> BoundaryWord {
        assert_eq!(
            tag.referent_owner(),
            BoundaryReferentOwner::InvocationArena,
            "the invocation arena is never the referent of a persistent word"
        );
        self.arena
            .0
            .push_node(tag, class, NULL_SLOT, 0, payload, 0, children, &[], &[])
    }

    /// Finish, yielding the arena.
    pub fn finish(self) -> BoundaryArenaV1 {
        self.arena
    }
}

// ---------------------------------------------------------------------------
// The persistent side — completing `store.rs`, not replacing it
// ---------------------------------------------------------------------------

/// The persistent half of the boundary ABI.
///
/// ⭐ **What this is and is not.** The content-addressed [`Store`] assigns and
/// owns persistent **identity**: a `SlotId` here is the store's own id, so two
/// equal values are one referent because the store says so, not because this
/// layer decided. What the store cannot do — measured at `aecdb001`, and
/// reported as a false fixed input in the frame's `D2` — is answer `slot ->
/// value`: it has `encode_canonical` and no inverse, `slot_id` is a monotonic
/// counter with no reverse index, and `intern` types over [`Value`] rather than
/// [`RuntimeGroundValue`], with no landed symbol bridge.
///
/// ⚠ **Scoped honestly:** the typed residency below is the read-back half, not
/// a second addressing scheme. It is keyed by the store's ids, lives exactly as
/// long as the store, and is never consulted for identity. A `RuntimeGroundValue`
/// image is retained because the landed canonical encoding is one-way; when a
/// decoder lands, this table is what it replaces.
pub struct BoundaryValueStore {
    store: Store,
    resident: BTreeMap<SlotId, RuntimeGroundValue>,
    symbols: Vec<RuntimeSymbol>,
    symbol_ids: BTreeMap<RuntimeSymbol, u64>,
    /// The persistent region every persistent word indexes.
    image: BoundaryPersistentImage,
    /// `SlotId -> persistent node index`. ⭐ **This is what makes the word an
    /// identity rather than a locator:** the store's slot decides the index, so
    /// materializing one value in two different invocations yields the *same*
    /// word, and the store stays the sole identity authority.
    placement: BTreeMap<SlotId, u64>,
}

// `Store` derives neither, and widening its derives is outside this node's
// scope — `B2R`'s guardrail against reopening landed surface applies to the
// substrate too. Both impls are therefore local.
impl Default for BoundaryValueStore {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for BoundaryValueStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundaryValueStore")
            .field("resident", &self.resident.len())
            .field("symbols", &self.symbols.len())
            .finish()
    }
}

impl BoundaryValueStore {
    /// A fresh store.
    pub fn new() -> Self {
        BoundaryValueStore {
            store: Store::new(),
            resident: BTreeMap::new(),
            symbols: Vec::new(),
            symbol_ids: BTreeMap::new(),
            image: BoundaryPersistentImage::default(),
            placement: BTreeMap::new(),
        }
    }

    /// The persistent region, for read-back and reservation.
    pub fn image(&self) -> &BoundaryPersistentImage {
        &self.image
    }

    /// Grant emitted code room to construct persistent nodes.
    ///
    /// ⚠ **The store governs persistent storage, including the part emitted code
    /// writes.** There is no path by which emitted code takes persistent space
    /// the store did not grant, which is what keeps this from being a second,
    /// unaccountable heap.
    /// The persistent image, mutably — **fault injection, tests only.**
    #[cfg(test)]
    pub fn image_mut(&mut self) -> &mut BoundaryPersistentImage {
        &mut self.image
    }

    pub fn reserve_persistent(&mut self, nodes: usize, words: usize, data: usize, limbs: usize) {
        self.image.reserve(nodes, words, data, limbs);
    }

    /// Publish the persistent header emitted code resolves persistent words
    /// through.
    ///
    /// ⚠ Invalidated by any later materialization or reservation — those can
    /// move the tables. Materialize, reserve, then publish.
    pub fn publish_persistent(&mut self) -> *mut u64 {
        self.image.publish()
    }

    /// The persistent node index a slot occupies, if it has been materialized.
    pub fn placement(&self, slot: SlotId) -> Option<u64> {
        self.placement.get(&slot).copied()
    }

    /// Intern a symbol — a constructor name, a record type identity, or a
    /// record field name — to a dense id emitted code can compare.
    ///
    /// Ids start at `1`; `0` is reserved as "no symbol" so a zeroed node field
    /// is never mistaken for a real identity.
    pub fn intern_symbol(&mut self, symbol: &str) -> u64 {
        if let Some(id) = self.symbol_ids.get(symbol) {
            return *id;
        }
        let id = self.symbols.len() as u64 + 1;
        self.symbols.push(symbol.to_string());
        self.symbol_ids.insert(symbol.to_string(), id);
        id
    }

    /// The symbol an id names, if any.
    pub fn symbol(&self, id: u64) -> Option<&str> {
        if id == 0 {
            return None;
        }
        self.symbols.get((id - 1) as usize).map(String::as_str)
    }

    /// The value a persistent slot owns, if this store owns that slot.
    pub fn resident(&self, slot: SlotId) -> Option<&RuntimeGroundValue> {
        self.resident.get(&slot)
    }

    /// Resolve a slot through the **store's own** read-back path:
    /// `slot -> canonical bytes -> Value`.
    ///
    /// ⭐ **Deliberately a second, independent path.** [`Self::resident`]
    /// answers from the typed map this layer keeps; this answers from bytes the
    /// store owns and a decoder that never saw the typed map. Two paths that
    /// agree corroborate each other; one path read twice corroborates nothing,
    /// which is the shape a residency-only design would have shipped.
    pub fn decode_slot(&self, slot: SlotId) -> Option<Value> {
        let bytes = self.store.canonical_bytes(slot)?;
        let (value, used) = crate::canonical::decode_canonical(bytes)?;
        // Trailing bytes mean encoder and decoder disagree about the shape.
        // That is a failure, not a partial success.
        (used == bytes.len()).then_some(value)
    }

    /// Number of slots the underlying store can resolve back to bytes.
    pub fn store_resident_slots(&self) -> usize {
        self.store.resident_slots()
    }

    /// Number of distinct persistent referents.
    pub fn resident_count(&self) -> usize {
        self.resident.len()
    }

    /// Take persistent ownership of a ground value, returning its slot id.
    ///
    /// Identity comes from the [`Store`]: equal values intern to one slot, so
    /// the residency map holds one image per referent rather than one per
    /// materialization.
    pub fn persist(&mut self, value: &RuntimeGroundValue) -> Option<SlotId> {
        let image = self.store_image(value)?;
        let slot = self.store.intern(&image).slot_id();
        self.resident.entry(slot).or_insert_with(|| value.clone());
        Some(slot)
    }

    /// The `values::Value` image used for content-addressed identity.
    ///
    /// `None` for a value the landed store cannot intern — `intern` asserts
    /// `is_compound`, so an immediate scalar has no store image and must never
    /// be routed here. That is not a defect: immediates never become handles.
    fn store_image(&mut self, value: &RuntimeGroundValue) -> Option<Value> {
        Some(match value {
            RuntimeGroundValue::Bool(_) => return None,
            RuntimeGroundValue::Int(int) => {
                // `canonical_big_image` is total over both arms and always
                // yields the compound `Value::BigInt`, which is interable; a
                // `Value::SmallInt` is an immediate and would trip the store's
                // `is_compound` assertion.
                int.canonical_big_image()
            }
            RuntimeGroundValue::Bytes(bytes) => Value::Bytes(bytes.clone()),
            RuntimeGroundValue::String(text) => Value::String(text.clone()),
            RuntimeGroundValue::Constructor { constructor, args } => {
                let constructor_id = self.intern_symbol(constructor) as u32;
                let mut encoded = Vec::with_capacity(args.len());
                for arg in args {
                    encoded.push(self.identity_leaf(arg)?);
                }
                Value::Constructor {
                    constructor_id,
                    args: encoded,
                }
            }
            RuntimeGroundValue::Record { fields } => {
                // `Value::Record` carries a `type_id` and positional fields —
                // it drops names — so the record's ordered field-name list IS
                // its type identity here, interned as one symbol.
                let identity = fields
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                let type_id = self.intern_symbol(&format!("record:{identity}")) as u32;
                let mut encoded = Vec::with_capacity(fields.len());
                for (_, field) in fields {
                    encoded.push(self.identity_leaf(field)?);
                }
                Value::Record {
                    type_id,
                    fields: encoded,
                }
            }
        })
    }

    /// The identity image of a nested value, where scalars stay immediate.
    fn identity_leaf(&mut self, value: &RuntimeGroundValue) -> Option<Value> {
        Some(match value {
            RuntimeGroundValue::Bool(bit) => Value::Bool(*bit),
            other => self.store_image(other)?,
        })
    }
}

// ---------------------------------------------------------------------------
// Materialization
// ---------------------------------------------------------------------------

/// Materialize a ground value into **persistent storage** and return its word.
///
/// Scalars stay immediate; compounds become persistent handles indexing the
/// store's own region. The recursion is over the value's *own* structure — no
/// environment and no caller context participates.
///
/// ⭐ **Takes no arena.** A ground value's referent outlives the invocation, so
/// the invocation arena is not a place it can go. That is the region split
/// expressed as a signature rather than as a comment.
pub fn materialize_ground(
    store: &mut BoundaryValueStore,
    value: &RuntimeGroundValue,
) -> Option<BoundaryWord> {
    store.materialize(value)
}

impl BoundaryValueStore {
    /// See [`materialize_ground`].
    fn materialize(&mut self, value: &RuntimeGroundValue) -> Option<BoundaryWord> {
        // `immediate` shifts the payload left by the tag width, so the top eight
        // bits — pure sign extension inside the immediate range — fall off and
        // `signed_payload`'s arithmetic shift restores them.
        if let RuntimeGroundValue::Bool(bit) = value {
            return Some(BoundaryWord::immediate(
                BoundaryTag::ImmediateBool,
                u64::from(*bit),
            ));
        }
        if let RuntimeGroundValue::Int(crate::RuntimeIntV1::Small(v)) = value {
            if BoundaryWord::int_fits_immediate(*v) {
                return Some(BoundaryWord::immediate(
                    BoundaryTag::ImmediateInt,
                    *v as u64,
                ));
            }
        }

        let slot = self.persist(value)?;
        // ⭐ One slot, one node, forever. A repeat materialization — in this
        // invocation or a later one — returns the identical word, so the word is
        // the store's identity and not a per-invocation locator.
        if let Some(index) = self.placement.get(&slot) {
            return Some(BoundaryWord::handle(BoundaryTag::PersistentGround, *index));
        }

        let (class, tag_id, payload, extent, children, names, limbs) = match value {
            // Handled above; listed so this match stays exhaustive over the
            // value's own structure rather than falling through a wildcard.
            RuntimeGroundValue::Bool(_) => return None,
            RuntimeGroundValue::Int(int) => match int.as_small() {
                // ⭐ A `Small`'s magnitude IS the node payload, and emitted
                // code decodes it with `ken_native_int_resolve_local` — the
                // landed exact-`Int` decoder, not a second one.
                Some(small) => (
                    BoundaryClass::Int,
                    0,
                    small as u64,
                    crate::native_int::NATIVE_INT_SMALL_TAG_V1,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
                // ⛔ **A wide `Int`'s limbs go in the PERSISTENT REGION, not
                // in the invocation's native arena.** The earlier candidate
                // returned `None` here, which made `Lowered::Int`'s promised
                // spill unreachable for exactly the values a bignum language
                // exists to carry. The reason it could not use
                // `NATIVE_INT_BIG_TAG_V1` still holds — that payload is a slot
                // in an arena that dies with the invocation — but the fix is to
                // put the magnitude where every other persistent content
                // already lives, beside the node that names it.
                None => {
                    let (sign, magnitude) = int.canonical_sign_and_limbs();
                    (
                        BoundaryClass::Int,
                        0,
                        sign,
                        BOUNDARY_INT_REGION_LIMBS,
                        Vec::new(),
                        Vec::new(),
                        magnitude,
                    )
                }
            },
            RuntimeGroundValue::Bytes(bytes) => {
                let at = self.image.0.push_data(bytes);
                (
                    BoundaryClass::Bytes,
                    0,
                    bytes.len() as u64,
                    at,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                )
            }
            RuntimeGroundValue::String(text) => {
                let at = self.image.0.push_data(text.as_bytes());
                (
                    BoundaryClass::String,
                    0,
                    text.len() as u64,
                    at,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                )
            }
            RuntimeGroundValue::Constructor { constructor, args } => {
                let tag_id = self.intern_symbol(constructor);
                let mut children = Vec::with_capacity(args.len());
                for arg in args {
                    children.push(self.materialize(arg)?);
                }
                (
                    BoundaryClass::Constructor,
                    tag_id,
                    0,
                    0,
                    children,
                    Vec::new(),
                    Vec::new(),
                )
            }
            RuntimeGroundValue::Record { fields } => {
                let mut children = Vec::with_capacity(fields.len());
                let mut names = Vec::with_capacity(fields.len());
                for (name, field) in fields {
                    names.push(self.intern_symbol(name));
                    children.push(self.materialize(field)?);
                }
                (BoundaryClass::Record, 0, 0, 0, children, names, Vec::new())
            }
        };

        // ⛔ A persistent node must not embed an invocation-owned child: the
        // parent survives the invocation and the child does not, so the escape
        // check on the parent's own tag would permit a word that reaches freed
        // storage. Unreachable from here — every child above is an immediate or
        // a persistent handle — and asserted so it stays unreachable.
        debug_assert!(
            children.iter().all(|c| c
                .tag()
                .is_some_and(|t| t.referent_owner() != BoundaryReferentOwner::InvocationArena)),
            "a persistent node never embeds an invocation-owned child"
        );

        let word = self.image.0.push_node(
            BoundaryTag::PersistentGround,
            class,
            slot,
            tag_id,
            payload,
            extent,
            &children,
            &names,
            &limbs,
        );
        self.placement.insert(slot, word.payload());
        Some(word)
    }
}

/// Materialize borrowed host ingress — valid for this invocation only.
///
/// ⛔ The node's owner is [`BoundaryReferentOwner::InvocationArena`] and its
/// slot is [`NULL_SLOT`], which is what makes escape detectable rather than
/// merely documented (`AC-7`).
pub fn materialize_borrowed(builder: &mut BoundaryArenaBuilder, payload: u64) -> BoundaryWord {
    builder.push_node(
        BoundaryTag::InvocationBorrowed,
        BoundaryClass::BorrowedOpaque,
        payload,
        &[],
    )
}

/// Materialize a `HostResult` — a runtime success discriminant selecting
/// between two already-materialized payload words.
///
/// ⛔ Borrowed ingress: the node is invocation-owned. `success` is a **runtime**
/// value; nothing here inspects which arm a particular reply took.
pub fn materialize_host_result(
    builder: &mut BoundaryArenaBuilder,
    success: u64,
    ok: BoundaryWord,
    err: BoundaryWord,
) -> BoundaryWord {
    builder.push_node(
        BoundaryTag::InvocationHostResult,
        BoundaryClass::HostResult,
        success,
        &[ok, err],
    )
}

/// Whether a word may cross out of the native invocation that produced it.
///
/// ⛔ **Fail-closed (`AC-7`).** An invocation-owned referent escaping its
/// invocation is [`BOUNDARY_ERR_ESCAPE`], never a silent pass.
///
/// ⚠ **What the Θ(1) tag test rests on, stated rather than assumed.** Permitting
/// a persistent word to leave is sound only because a persistent node's referent
/// is store-owned *and* no persistent node embeds an invocation-owned child.
/// The second half is a **construction-time** invariant, held at both paths that
/// can build one — [`BoundaryValueStore::materialize`] on the Rust side and
/// `ken_boundary_store_field_local` on the emitted side, which returns
/// [`BOUNDARY_ERR_ESCAPE`] for exactly that store. This check does not walk the
/// structure; walking would be O(size) and would re-answer at every crossing a
/// question already settled once at construction.
pub fn check_escape(word: BoundaryWord) -> i64 {
    match word.tag() {
        None => BOUNDARY_ERR_TAG,
        Some(tag) => match tag.referent_owner() {
            BoundaryReferentOwner::InvocationArena => BOUNDARY_ERR_ESCAPE,
            BoundaryReferentOwner::NoReferent | BoundaryReferentOwner::PersistentStore => {
                BOUNDARY_OK
            }
        },
    }
}
