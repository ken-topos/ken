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
    /// Handle to a `Constructor`/`Record` aggregate **at least one of whose
    /// children is invocation-owned**, so the aggregate itself cannot outlive
    /// the invocation (`RT-DECL-CLOSURE-PORT` `D7`).
    ///
    /// ⭐ **This tag exists because the lifetime of an aggregate is a MEET over
    /// its children, and no per-value shape can compute it.** `Constructor` and
    /// `Record` are persistable *shapes*, so the value-shape disposition
    /// reaches for [`BoundaryTag::PersistentGround`]. That is right only when
    /// every child outlives the parent. When one child is an invocation-arena
    /// referent — a borrowed host buffer, a capability or resource token, a
    /// `HostResult` — a `PersistentGround` parent naming it is precisely the
    /// dangling relation `store_field`'s escape guard refuses.
    ///
    /// ⛔ **It is NOT a second persistable lane and NOT a widening.** The
    /// admitted classes are exactly `Constructor` and `Record`; its referent
    /// owner is the invocation arena, so every escape check that already keys
    /// on [`BoundaryReferentOwner::InvocationArena`] governs it unchanged, and
    /// a word carrying it may not be published past the invocation.
    ///
    /// ⚠ The alternative — keeping one aggregate tag and rejecting the mixed
    /// case — is what the tree did before, and it rejected a **sound**
    /// program: an invocation-scoped aggregate over invocation-scoped children
    /// dangles nothing. The defect was the missing lane, not the missing
    /// refusal.
    InvocationAggregate = 9,
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
    pub const ALL: [BoundaryTag; 10] = [
        BoundaryTag::ImmediateBool,
        BoundaryTag::ImmediateInt,
        BoundaryTag::ImmediateExitStatus,
        BoundaryTag::ImmediateBoundedNat,
        BoundaryTag::ImmediateStructuralNat,
        BoundaryTag::PersistentGround,
        BoundaryTag::PersistentClosure,
        BoundaryTag::InvocationBorrowed,
        BoundaryTag::InvocationHostResult,
        BoundaryTag::InvocationAggregate,
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
            9 => BoundaryTag::InvocationAggregate,
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
            BoundaryTag::InvocationBorrowed
            | BoundaryTag::InvocationHostResult
            | BoundaryTag::InvocationAggregate => BoundaryReferentOwner::InvocationArena,
        }
    }

    /// Whether the payload is the value itself rather than an arena index.
    pub fn is_immediate(self) -> bool {
        self.referent_owner() == BoundaryReferentOwner::NoReferent
    }

    /// The [`BoundaryClass`] the uniform `class` helper reports for a word
    /// carrying this tag as an **immediate**. `None` for a handle tag, whose
    /// class comes from its node.
    ///
    /// ⛔ **This is NOT an immediate node class, and it is deliberately kept
    /// apart from [`BOUNDARY_TAG_CLASS_RELATION`].** That relation governs what
    /// may be written into a node's `NODE_CLASS`, and it excludes immediate
    /// tags for a real reason — an immediate has no node, so there is nothing
    /// to give a class to. What this answers is a different question the ABI
    /// still has to answer: *when a consumer asks the `class` helper about an
    /// immediate word, what boundary-value classification comes back?* Merging
    /// the two would invent a fictional immediate node class and would make the
    /// node-legality relation admit tags it must keep refusing.
    ///
    /// ⭐ Total and wildcard-free, exactly like [`BoundaryTag::referent_owner`]:
    /// a new tag is a compile error here rather than a value that silently
    /// inherits whichever arm a hand-written branch left as its default. The
    /// emitted `class` helper used to answer this with
    /// `is_bool ? Bool : Int` written beside the helper body — a second mapping
    /// that could disagree with this one and nothing would notice.
    pub fn immediate_value_class(self) -> Option<BoundaryClass> {
        match self {
            BoundaryTag::ImmediateBool => Some(BoundaryClass::Bool),
            // Every remaining immediate is an integer scalar. The finer
            // identity — exit status vs bounded nat vs structural nat — lives
            // in the word's own tag byte and is not a ground-value class.
            BoundaryTag::ImmediateInt
            | BoundaryTag::ImmediateExitStatus
            | BoundaryTag::ImmediateBoundedNat
            | BoundaryTag::ImmediateStructuralNat => Some(BoundaryClass::Int),
            BoundaryTag::PersistentGround
            | BoundaryTag::PersistentClosure
            | BoundaryTag::InvocationBorrowed
            | BoundaryTag::InvocationHostResult
            | BoundaryTag::InvocationAggregate => None,
        }
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

impl BoundaryReferentOwner {
    /// Every owner, in declaration order.
    ///
    /// ⚠ **This list is not self-guarding — its consumers are.** The emitted
    /// marker-mask selection folds over it and maps each owner through a
    /// wildcard-free `match`, so a new variant is a compile error at that fold
    /// rather than a silently-absent entry here.
    pub const ALL: [BoundaryReferentOwner; 3] = [
        BoundaryReferentOwner::NoReferent,
        BoundaryReferentOwner::PersistentStore,
        BoundaryReferentOwner::InvocationArena,
    ];
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

    /// Decode a class word. `None` for any value outside the closed set — an
    /// unknown class is a **third outcome that fails**, exactly as
    /// [`BoundaryTag::from_bits`] treats an unknown tag. Derived from
    /// [`BoundaryClass::ALL`], so it cannot drift from the enum.
    pub fn from_bits(bits: u64) -> Option<Self> {
        BoundaryClass::ALL
            .into_iter()
            .find(|class| *class as u64 == bits)
    }
}

/// The in-node storage a class's payload occupies.
///
/// ⛔ **This is an authority, not a description.** `RECUT 2` requires the
/// emitted helper bodies to be generated *from* the representation authority
/// rather than to restate it; the class guards in `boundary_value_clif` used to
/// carry a literal class list beside each body, which is the hand-maintained
/// table the recut names. Those lists are now derived from this function.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BoundaryStorageShape {
    /// The payload rides in the node word itself.
    InlineWord,
    /// The payload is a region-owned magnitude — a limb span.
    IntMagnitude,
    /// The payload is a region-owned byte span.
    ByteSpan,
    /// The payload is references to child nodes.
    ChildNodes,
}

impl BoundaryClass {
    /// Which storage this class's payload occupies.
    ///
    /// ⛔ **Wildcard-free**, so a tenth class must declare its storage rather
    /// than inheriting one by default — the same discipline the disposition
    /// uses one layer up.
    pub fn storage_shape(self) -> BoundaryStorageShape {
        match self {
            BoundaryClass::Bool => BoundaryStorageShape::InlineWord,
            BoundaryClass::Int => BoundaryStorageShape::IntMagnitude,
            BoundaryClass::Bytes | BoundaryClass::String => BoundaryStorageShape::ByteSpan,
            BoundaryClass::Constructor
            | BoundaryClass::Record
            | BoundaryClass::HostResult
            | BoundaryClass::Closure => BoundaryStorageShape::ChildNodes,
            // A borrowed opaque names storage this ABI does not own, so it has
            // no in-node payload of its own.
            BoundaryClass::BorrowedOpaque => BoundaryStorageShape::InlineWord,
        }
    }
}

/// The emission plan — the representation authority, reduced to what the
/// emitter needs in order to *generate* the helper bodies.
///
/// ⛔ **Computed once at the `lowering/core` → `emit_boundary_value_local_graph`
/// seam and passed in.** It is deliberately data-only and crate-private: the
/// derivation lives with the `LoweredVariant`/`BoundaryInput` authority in
/// `cranelift_backend::lowering`, which is the only place that can see it, and
/// the emitter may **consume** the plan but cannot restate it.
///
/// ⚠ **It carries no seed value and no sampled runtime value** — only the
/// finite class sets the partition admits. A representation chosen by
/// inspecting a value describes a program that cannot be written (`D1`/`AC-2`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundaryEmissionPlan {
    int_magnitude_classes: Vec<BoundaryClass>,
    byte_span_classes: Vec<BoundaryClass>,
    tags: BoundaryTagAdmission,
}

/// The tag half of the plan: which tags the partition admits, split by the
/// distinctions the emitted helpers actually branch on.
///
/// ⛔ **Sets, never ordinal bands.** The emitter previously asked *"is this tag
/// numerically at or below `LAST_PERSISTENT_TAG`"*, which is a second authority
/// derived by hand from [`BoundaryTag`]'s declaration order: reordering the
/// enum leaves both constants well-formed and silently re-points every
/// persistent word at the invocation arena. A set has no such failure mode, and
/// it does not require the admitted tags to be contiguous in the first place.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundaryTagAdmission {
    admitted: Vec<BoundaryTag>,
    immediate: Vec<BoundaryTag>,
    handle: Vec<BoundaryTag>,
    owner_bands: Vec<(BoundaryReferentOwner, Vec<BoundaryTag>)>,
    immediate_value_classes: Vec<(BoundaryTag, BoundaryClass)>,
    handle_class_relation: Vec<(BoundaryTag, Vec<BoundaryClass>)>,
}

impl BoundaryTagAdmission {
    /// Build the tag admission from already-derived sets.
    pub(crate) fn new(
        admitted: Vec<BoundaryTag>,
        immediate: Vec<BoundaryTag>,
        handle: Vec<BoundaryTag>,
        owner_bands: Vec<(BoundaryReferentOwner, Vec<BoundaryTag>)>,
        immediate_value_classes: Vec<(BoundaryTag, BoundaryClass)>,
        handle_class_relation: Vec<(BoundaryTag, Vec<BoundaryClass>)>,
    ) -> Self {
        BoundaryTagAdmission {
            admitted,
            immediate,
            handle,
            owner_bands,
            immediate_value_classes,
            handle_class_relation,
        }
    }

    /// Every tag any admitted outcome can carry. A tag outside this set is the
    /// third outcome that fails, never a fall-through.
    pub(crate) fn admitted(&self) -> &[BoundaryTag] {
        &self.admitted
    }

    /// The tags whose payload is the value itself.
    pub(crate) fn immediate(&self) -> &[BoundaryTag] {
        &self.immediate
    }

    /// The tags whose payload indexes a node.
    pub(crate) fn handle(&self) -> &[BoundaryTag] {
        &self.handle
    }

    /// Each referent owner the partition publishes handles for, paired with
    /// exactly the tags it publishes under that owner.
    ///
    /// ⛔ **A relation, not a two-way split.** The emitter used to assume there
    /// are exactly two handle owners and discriminate them with one threshold;
    /// an owner the partition started admitting would have been silently folded
    /// into whichever side of that threshold its tag landed on.
    pub(crate) fn owner_bands(&self) -> &[(BoundaryReferentOwner, Vec<BoundaryTag>)] {
        &self.owner_bands
    }

    /// Each admitted immediate tag paired with the class the `class` helper
    /// must report for it.
    ///
    /// ⛔ **Not a node class.** See [`BoundaryTag::immediate_value_class`] for
    /// why this is a separate contract from [`BOUNDARY_TAG_CLASS_RELATION`]. A
    /// tag absent from this relation has no classification, and the emitted
    /// helper fails closed on it rather than defaulting.
    pub(crate) fn immediate_value_classes(&self) -> &[(BoundaryTag, BoundaryClass)] {
        &self.immediate_value_classes
    }

    /// The normalized handle `BoundaryTag → set<BoundaryClass>` relation — what
    /// may be written into a **node's** `NODE_CLASS`.
    ///
    /// ⛔ **This is the emitted allocator's sole authority for the relation**,
    /// and it is derived from `BoundaryOutcome::HandleWord` in the one
    /// partition sweep. `ImmediateWord` is excluded by construction: an
    /// immediate has no node, so it has no node class. A tag with no row here
    /// admits nothing, and the allocator's fold is seeded with the empty mask so
    /// that absence is `BOUNDARY_ERR_RELATION` on its own rather than something
    /// an earlier guard has to make unreachable.
    pub(crate) fn handle_class_relation(&self) -> &[(BoundaryTag, Vec<BoundaryClass>)] {
        &self.handle_class_relation
    }

    /// The tags published under one owner — empty if the partition publishes
    /// none, which is a legitimate answer and not a missing entry.
    pub(crate) fn tags_owned_by(&self, owner: BoundaryReferentOwner) -> &[BoundaryTag] {
        self.owner_bands
            .iter()
            .find(|(band, _)| *band == owner)
            .map(|(_, tags)| tags.as_slice())
            .unwrap_or(&[])
    }
}

impl BoundaryEmissionPlan {
    /// Build a plan from already-derived class and tag sets.
    ///
    /// ⛔ There is deliberately **no** whole-admitted-class set here. One was
    /// carried for a while and no emitted helper ever read it — `rustc` said so
    /// on every lib build (`method admitted_classes is never used`). A derived
    /// set with no production consumer is a declaration, and `RULING R3` is
    /// explicit that a declaration does not discharge the predicate. The
    /// per-storage-shape sets below are the ones the emitter actually uses.
    ///
    /// ⛔ Crate-private and unexported: the only caller is the derivation in
    /// `cranelift_backend::lowering`, so a second hand-written plan cannot
    /// appear beside the helper bodies.
    pub(crate) fn new(
        int_magnitude_classes: Vec<BoundaryClass>,
        byte_span_classes: Vec<BoundaryClass>,
        tags: BoundaryTagAdmission,
    ) -> Self {
        BoundaryEmissionPlan {
            int_magnitude_classes,
            byte_span_classes,
            tags,
        }
    }

    /// The tag sets the emitted helpers branch on.
    pub(crate) fn tags(&self) -> &BoundaryTagAdmission {
        &self.tags
    }

    /// The classes a limb-storage helper may touch.
    pub(crate) fn int_magnitude_classes(&self) -> &[BoundaryClass] {
        &self.int_magnitude_classes
    }

    /// The classes a byte-span helper may touch.
    pub(crate) fn byte_span_classes(&self) -> &[BoundaryClass] {
        &self.byte_span_classes
    }
}

// ---------------------------------------------------------------------------
// The tag × class relation
// ---------------------------------------------------------------------------

/// ⛔ **The valid `(tag, class)` pairs, as the RECONCILED RUST MIRROR — not the
/// authority.**
///
/// ⚠ This heading used to read *"one authoritative relation"*, which
/// contradicted the explanation a dozen lines below it. The heading is the part
/// a hurried reader takes away, so it is corrected here rather than only there.
///
/// A closed set of tags and a closed set of classes do **not** make a closed
/// ABI: the tag decides *lifetime and region*, the class decides
/// *interpretation*, and their product contains pairs no disposition can ever
/// produce. `PersistentClosure + HostResult` and `InvocationHostResult +
/// Constructor` are representable in the product and meaningless in the ABI —
/// minting one succeeds and then fails much later at an unrelated projection,
/// which reports the wrong defect at the wrong place.
///
/// ⛔ **NOT the authority, and NOT derived — a hand-written Rust MIRROR.** The
/// **sole** representation authority is the `BoundaryInput → BoundaryOutcome`
/// partition, and the emitted allocator consumes it through
/// `BoundaryEmissionPlan::handle_class_relation`.
///
/// ⚠ **This is ONE contract with TWO ENFORCEMENT PATHS, not two authorities**
/// (`R5` clause 3, as corrected by the Architect's erratum). The Rust builders
/// and the emitted allocator enforce the same tag × node-class legality at
/// different production sites; this slice exists only because the partition is
/// private to `cranelift_backend::lowering` where the builders cannot see it.
/// ⛔ **What is forbidden is an independently maintained or independently
/// authoritative mirror** — which is exactly what this was before the
/// reconciliation below existed.
///
/// ⚠ Its previous doc called it *"derived from `Lowered::boundary_disposition`"*
/// and *"the single source"*. Both were false: nothing derived it and nothing
/// checked it. It is now **mechanically reconciled** to the partition-derived
/// relation over the full finite `BoundaryTag::ALL × BoundaryClass::ALL` product,
/// **in both directions**, by
/// `b2v_the_rust_mirror_and_the_derived_relation_reconcile_over_the_product`.
/// Drift in either direction reddens.
///
/// Immediate tags are absent by construction — they have no node, so they have
/// no class.
pub(crate) const BOUNDARY_TAG_CLASS_RELATION: &[(BoundaryTag, &[BoundaryClass])] = &[
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
    // ⛔ Exactly the two aggregate classes, and nothing else. This lane exists
    // to carry a `Constructor`/`Record` whose lifetime meet is the invocation;
    // admitting `Bytes`/`String`/`Int` here would create a second, redundant
    // encoding of values that have no children to take a meet over.
    (
        BoundaryTag::InvocationAggregate,
        &[BoundaryClass::Constructor, BoundaryClass::Record],
    ),
];

/// ⛔ **The closed set of RETIRED lanes — reserved ABI metadata, NOT a
/// capability** (`RT-FNSPLIT-C1` `D5`).
///
/// ⭐ **Recognized, never admitted.** These pairs stay in the ABI's *vocabulary*
/// so a word naming one is refused **by name** with
/// [`BOUNDARY_ERR_RETIRED_LANE`], instead of collapsing into
/// [`BOUNDARY_ERR_TAG`] and becoming indistinguishable from a corrupt byte.
///
/// ⛔ **This is deliberately a written-down declaration and not derived**, and
/// that is the opposite of how the admitted sets work. `BoundaryEmissionPlan::
/// derive` sweeps the live representation authority, so a lane with no producer
/// contributes nothing — which is exactly how the closure vocabulary was lost
/// when its disposition became `FailClosedForbidden`. A tombstone has no
/// producer **by definition**, so it cannot be derived from producers and must
/// be stated.
///
/// ⛔ **Never consult this from a producer or a representation disposition.**
/// Its only readers are decode/classification and emitted-helper validation.
/// `Closure` / `DeclarationClosure` remain `FailClosedForbidden`, and no
/// `RepresentedHandle { PersistentClosure, Closure }` may be restored — the
/// point is a recognition/admission split, not a revived capability.
pub(crate) const BOUNDARY_RETIRED_LANES: &[(BoundaryTag, BoundaryClass)] =
    &[(BoundaryTag::PersistentClosure, BoundaryClass::Closure)];

/// Whether this `(tag, class)` pair names a retired lane.
///
/// ⚠ A `true` here means *"well-formed, and refused because the capability is
/// retired"* — it is **not** a malformed pair. `PersistentClosure + Bool` is
/// malformed and answers `false`, keeping its [`BOUNDARY_ERR_RELATION`]
/// diagnostic; only the exactly-paired lane reaches this.
pub(crate) fn boundary_lane_is_retired(tag: BoundaryTag, class: BoundaryClass) -> bool {
    BOUNDARY_RETIRED_LANES
        .iter()
        .any(|(retired_tag, retired_class)| *retired_tag == tag && *retired_class == class)
}

/// The tags that are **recognized but carry no admitted lane**, given the
/// partition's admitted tag set (`RT-FNSPLIT-C1` `D5`).
///
/// ⛔ **Derived from BOTH authorities at every call site, never written down.**
/// A tag is retired exactly when it names a retired lane *and* the live
/// partition admits it nowhere — so a tag that still has one surviving admitted
/// lane is **not** reported here, because such a tag is genuinely admitted and
/// refusing it by name would be the inverse error.
///
/// ⭐ **Why this is a function of the plan and not a seventh field on
/// [`BoundaryTagAdmission`].** Every emitted helper already holds the plan's
/// admitted set; taking it as an argument means each mutation fixture derives
/// its retired set from *its own* admitted set rather than from a hand-written
/// list that would drift into a second authority — which is the defect the
/// whole partition-derived plan exists to avoid.
pub(crate) fn boundary_retired_tags(admitted: &[BoundaryTag]) -> Vec<BoundaryTag> {
    let mut tags: Vec<BoundaryTag> = Vec::new();
    for (retired_tag, _) in BOUNDARY_RETIRED_LANES {
        if !admitted.contains(retired_tag) && !tags.contains(retired_tag) {
            tags.push(*retired_tag);
        }
    }
    tags
}

/// Whether the ABI admits this `(tag, class)` pair, per the Rust mirror.
///
/// The Rust builders' fail-before-publication check. Kept because they cannot
/// see the private lowering partition; reconciled to it over the full product.
///
/// ⚠ **A retired lane is RECOGNIZED but NOT admitted**, so this still answers
/// `false` for `(PersistentClosure, Closure)`. Recognition governs which
/// *diagnostic* a refusal carries; it never widens what is admitted.
pub(crate) fn boundary_relation_admits(tag: BoundaryTag, class: BoundaryClass) -> bool {
    // ⛔ Recognition first, admission second — the retired lane is in the
    // schema below **precisely so it can be named**, and reading the schema
    // alone would therefore report it as admitted. That is the inversion this
    // whole split exists to avoid: the pair stays spelled out so a refusal can
    // say which lane it refused, not so the lane works.
    if boundary_lane_is_retired(tag, class) {
        return false;
    }
    BOUNDARY_TAG_CLASS_RELATION
        .iter()
        .any(|(t, classes)| *t == tag && classes.contains(&class))
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
/// ⭐ What makes the emitted check Θ(1) and undriftable: the CLIF evaluates all
/// three domain predicates and selects by a mask **computed from this table**,
/// so there is no second place to edit.
///
/// ⚠ The `(tag, class)` relation used to have a twin of this function. It is
/// gone: that relation's mask is now folded from the plan's partition-derived
/// rows inside `relation_mask`, because "computed from this table" was the wrong
/// property when the table itself was the hand-maintained thing.
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
/// cannot drift from the declaration — the same pattern as
/// [`boundary_domain_mask`].
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

// ---------------------------------------------------------------------------
// Closure code identity — artifact-bound, never a bare ordinal
// ---------------------------------------------------------------------------

/// The artifact-bound `code_id` of a callable unit.
///
/// ⛔ **A bare local-origin ordinal is not an identity.** `StaticOriginId` is a
/// `u32` counter that restarts at zero in every artifact, so two independently
/// compiled artifacts both number their first callable unit `0`. A closure
/// keyed on that ordinal would make two *different* closures content-equal —
/// the store would intern them to one slot and hand the wrong body to whichever
/// consumer asked second. The ruling names this failure explicitly, and the
/// repair is a **namespace**, not a wider integer.
///
/// ⚠ **The length prefix is defensive, NOT load-bearing today — and I claimed
/// otherwise until a mutation said so.** The doc here used to argue that without
/// it `("ab", …)` and `("a", …)` could collide. That is false as this function
/// stands: `package_identity` is the *only* variable-length field and every
/// other one is fixed-width, so the total length already determines where the
/// string ends and the concatenation is injective either way. ⛔ **Mutation
/// `M48` removes the prefix and reddens nothing, which is the correct result,
/// not a missing control.** It is kept so that adding a *second*
/// variable-length field cannot silently make the encoding ambiguous — but that
/// is a future-proofing claim, and it is the only one it earns.
///
/// Uses the landed [`crate::hash::fnv1a_64`] rather than a second hash: it is
/// this crate's declared content-addressing hash, deterministic across runs and
/// documented as such. A `DefaultHasher` would be neither.
pub fn boundary_code_id(identity: &crate::RuntimeArtifactIdentity, local_origin: u64) -> u64 {
    let mut bytes = Vec::with_capacity(identity.package_identity.len() + 32);
    bytes.extend_from_slice(&(identity.package_identity.len() as u64).to_le_bytes());
    bytes.extend_from_slice(identity.package_identity.as_bytes());
    bytes.extend_from_slice(&identity.core_semantic_hash.to_le_bytes());
    bytes.extend_from_slice(&identity.artifact_hash.to_le_bytes());
    bytes.extend_from_slice(&local_origin.to_le_bytes());
    crate::hash::fnv1a_64(&bytes)
}

// ---------------------------------------------------------------------------
// Layout: ONE authority, derived extents, real consumers
// ---------------------------------------------------------------------------

/// ⛔ **The node's field inventory — the sole authority for node layout.**
///
/// Every offset, the stride, and the words `push_node` writes are **derived
/// from this enum**; nothing about node layout is stated twice. Adding a variant
/// moves every derived quantity at once and is a **compile error** in
/// `push_node`, whose `match` has no `_` arm — so a field cannot be half-added.
///
/// ⚠ **A hand-maintained constant checked against a hand-maintained list does
/// not close this**, and that is not hypothetical: the shipped candidate
/// declared a 136-byte header, published 144 bytes, and had **no consumer of
/// the constant anywhere in the tree**. Two authorities cannot check each
/// other; the fix is to have one. Each variant's meaning is documented on the
/// offset constant derived from it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum NodeField {
    Class,
    Owner,
    Slot,
    TagId,
    Payload,
    FieldCount,
    FieldsAt,
    Extent,
    LimbsAt,
    LimbCount,
    IntSealed,
}

impl NodeField {
    /// Every field, in layout order.
    pub const ALL: [NodeField; 11] = [
        NodeField::Class,
        NodeField::Owner,
        NodeField::Slot,
        NodeField::TagId,
        NodeField::Payload,
        NodeField::FieldCount,
        NodeField::FieldsAt,
        NodeField::Extent,
        NodeField::LimbsAt,
        NodeField::LimbCount,
        NodeField::IntSealed,
    ];

    /// This field's byte offset — its position, times the word width.
    pub const fn offset(self) -> i32 {
        (self as i32) * 8
    }
}

/// Byte stride of one arena node, **derived** from the field inventory.
pub const BOUNDARY_NODE_STRIDE: i32 = (NodeField::ALL.len() * 8) as i32;

/// `BoundaryClass` of this node.
pub const NODE_CLASS: i32 = NodeField::Class.offset();
/// `BoundaryReferentOwner` of this node's referent.
pub const NODE_OWNER: i32 = NodeField::Owner.offset();
/// The `SlotId` that owns this node's value, or `NULL_SLOT` when the owner is
/// the invocation arena. **This field is what makes `AC-6` observable.**
pub const NODE_SLOT: i32 = NodeField::Slot.offset();
/// Interned constructor symbol / record type identity, or `0`.
pub const NODE_TAG_ID: i32 = NodeField::TagId.offset();
/// Scalar payload: bool bit, small-int value, `HostResult` success flag, or the
/// byte length of a `Bytes`/`String`.
pub const NODE_PAYLOAD: i32 = NodeField::Payload.offset();
/// Number of child words this node has.
pub const NODE_FIELD_COUNT: i32 = NodeField::FieldCount.offset();
/// Index into the word table of this node's first child word. Field *names*
/// live at the same index in the name table.
pub const NODE_FIELDS_AT: i32 = NodeField::FieldsAt.offset();
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
pub const NODE_EXTENT: i32 = NodeField::Extent.offset();
/// Index into the region's **limb table** of a spilled `Int`'s first limb.
///
/// ⛔ A dedicated field and a dedicated table, deliberately — not a reuse of
/// [`NODE_FIELDS_AT`] and the word table. `ken_boundary_field_local` and
/// `ken_boundary_field_count_local` are **not** class-guarded, so limbs parked
/// in the word table would be readable as child *words*: a raw magnitude limb
/// returned where a tagged `BoundaryWord` is expected. Two meanings for one
/// table is exactly the collision `NODE_EXTENT`'s note warns about, and the
/// cheap fix is storage that cannot be reached by the wrong reader at all.
pub const NODE_LIMBS_AT: i32 = NodeField::LimbsAt.offset();
/// Number of limbs a spilled `Int` node's magnitude has. Zero for every other
/// class and for a `Small`.
pub const NODE_LIMB_COUNT: i32 = NodeField::LimbCount.offset();
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
pub const NODE_INT_SEALED: i32 = NodeField::IntSealed.offset();

/// Byte size of a **region header**.
///
/// ⭐ One layout serves both regions. The invocation arena and the persistent
/// image publish the *same* header shape, which is what lets a single
/// `resolve` select a region at run time and then read it with one set of
/// offsets. A second layout would be a second place for the offsets to drift.
/// ⛔ **The region header's field inventory — the sole authority for header
/// layout.** Same closure as [`NodeField`]: every offset and the header extent
/// derive from it, and `BoundaryRegion::header_value`'s `match` has no `_` arm,
/// so a new field is a compile error until it is given a value.
///
/// ⭐ One layout serves both regions, which is what lets a single `resolve`
/// select a region at run time and then read it with one set of offsets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum RegionHeaderField {
    Nodes,
    NodeCount,
    Words,
    WordCount,
    Names,
    NameCount,
    NodeCapacity,
    WordCapacity,
    Persistent,
    Frozen,
    Data,
    DataCount,
    DataCapacity,
    NativeInt,
    Limbs,
    LimbCount,
    LimbCapacity,
    Sealed,
}

impl RegionHeaderField {
    /// Every field, in layout order.
    pub const ALL: [RegionHeaderField; 18] = [
        RegionHeaderField::Nodes,
        RegionHeaderField::NodeCount,
        RegionHeaderField::Words,
        RegionHeaderField::WordCount,
        RegionHeaderField::Names,
        RegionHeaderField::NameCount,
        RegionHeaderField::NodeCapacity,
        RegionHeaderField::WordCapacity,
        RegionHeaderField::Persistent,
        RegionHeaderField::Frozen,
        RegionHeaderField::Data,
        RegionHeaderField::DataCount,
        RegionHeaderField::DataCapacity,
        RegionHeaderField::NativeInt,
        RegionHeaderField::Limbs,
        RegionHeaderField::LimbCount,
        RegionHeaderField::LimbCapacity,
        RegionHeaderField::Sealed,
    ];

    /// This field's byte offset — its position, times the word width.
    pub const fn offset(self) -> i32 {
        (self as i32) * 8
    }
}

/// Byte size of a region header, **derived** from the field inventory.
pub const BOUNDARY_REGION_HEADER_BYTES: i32 = (RegionHeaderField::ALL.len() * 8) as i32;

/// Pointer to the node table.
pub const ARENA_NODES: i32 = RegionHeaderField::Nodes.offset();
/// Number of **live** nodes. ⚠ Mutable: the emitted allocator bumps it.
pub const ARENA_NODE_COUNT: i32 = RegionHeaderField::NodeCount.offset();
/// Pointer to the child-word table.
pub const ARENA_WORDS: i32 = RegionHeaderField::Words.offset();
/// Number of **live** child words. ⚠ Mutable: the emitted allocator bumps it.
pub const ARENA_WORD_COUNT: i32 = RegionHeaderField::WordCount.offset();
/// Pointer to the field-name-id table, parallel to the word table.
pub const ARENA_NAMES: i32 = RegionHeaderField::Names.offset();
/// Number of field-name ids.
pub const ARENA_NAME_COUNT: i32 = RegionHeaderField::NameCount.offset();
/// Node capacity — the ceiling the emitted allocator fails closed against.
pub const ARENA_NODE_CAPACITY: i32 = RegionHeaderField::NodeCapacity.offset();
/// Child-word capacity — the other ceiling.
pub const ARENA_WORD_CAPACITY: i32 = RegionHeaderField::WordCapacity.offset();
/// Pointer to the **persistent region's** header, or `0` when this invocation
/// is bound to no persistent storage. Read from the *arena* header only.
pub const ARENA_PERSISTENT: i32 = RegionHeaderField::Persistent.offset();
/// Nodes present when the region was published.
///
/// ⛔ **The frozen prefix.** Emitted code may construct nodes at or beyond this
/// index and may mutate only those. A node the Rust side materialized carries
/// the store's [`SlotId`], and letting emitted code rewrite that field would let
/// it forge persistent identity — the store must remain the sole identity
/// authority, so the boundary is a bounds check rather than a convention.
pub const ARENA_FROZEN: i32 = RegionHeaderField::Frozen.offset();
/// Pointer to the region's **data table** — the byte span backing `Bytes` and
/// `String` contents.
pub const ARENA_DATA: i32 = RegionHeaderField::Data.offset();
/// Number of **live** data bytes. ⚠ Mutable: the emitted allocator bumps it.
pub const ARENA_DATA_COUNT: i32 = RegionHeaderField::DataCount.offset();
/// Data-table capacity — the third ceiling construction fails closed against.
pub const ARENA_DATA_CAPACITY: i32 = RegionHeaderField::DataCapacity.offset();
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
pub const ARENA_NATIVE_INT: i32 = RegionHeaderField::NativeInt.offset();
/// Pointer to the region's **limb table** — the `u64` magnitude storage backing
/// a spilled `Int` whose marker is [`BOUNDARY_INT_REGION_LIMBS`].
///
/// ⭐ Region-owned, which is the whole point: a persistent `Int`'s limbs outlive
/// every invocation because they live where the persistent nodes do.
pub const ARENA_LIMBS: i32 = RegionHeaderField::Limbs.offset();
/// Number of **live** limbs. ⚠ Mutable: the emitted allocator bumps it.
pub const ARENA_LIMB_COUNT: i32 = RegionHeaderField::LimbCount.offset();
/// Limb-table capacity — the fourth ceiling construction fails closed against.
pub const ARENA_LIMB_CAPACITY: i32 = RegionHeaderField::LimbCapacity.offset();
/// ⛔ **`1` once the store has taken exclusive ownership of this region, `0`
/// while emitted code may still write it** — the seal/quiescence handoff
/// adoption begins from.
///
/// ⭐ **Why this is a published header field and not a Rust-side flag.** The
/// property that has to hold is *"emitted writers can no longer mutate the
/// snapshot"*, and emitted writers do not consult Rust state — they hold the
/// raw region base they were published. A flag the mutators cannot see would
/// document the handoff instead of enforcing it. Here it sits on the one path
/// every mutator already walks, so [`ARENA_SEALED`] is checked in
/// `mutable_guard` and in the allocator, which between them are **every**
/// emitted writer.
pub const ARENA_SEALED: i32 = RegionHeaderField::Sealed.offset();

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
/// ⛔ **A mutation reached a region the store has SEALED.**
///
/// The seal is the *ownership-transfer* half of adoption (`AC-6`). Adoption
/// absorbs the published counts, validates the reachable graph, mints identity
/// and publishes — and every one of those steps reads a snapshot it assumes is
/// stable. A writer that can still run concurrently makes the snapshot a
/// fiction: the validated graph and the canonicalized graph would be different
/// graphs. ⛔ **Rust's `&mut` does not close this**, because emitted code holds
/// a raw region base captured at publication and does not go through the borrow
/// checker to use it. The seal is a field in the *published header* for exactly
/// that reason — it is on the path emitted code actually reads.
pub const BOUNDARY_ERR_SEALED: i64 = -9;
/// ⛔ **A persistent graph contains a cycle, so it has no canonical image.**
///
/// Ken's persistent `Value` is finite and well-founded: children and captures
/// are encoded *inline* as full canonical `Value`s, and store identity is the
/// hash+memcmp of that finite byte image. A back-edge has no such image, so a
/// cyclic staging graph is **malformed**, not an admitted value the ABI happens
/// to reject (Architect, cycle contract). ⚠ Distinct from
/// [`BOUNDARY_ERR_SHAPE`] on purpose: *"this graph is not a value"* and *"this
/// word is the wrong shape"* are different findings, and a shared status makes
/// the cycle control unable to say which one it caught.
pub const BOUNDARY_ERR_CYCLE: i64 = -10;
/// ⛔ **A `Closure` reached adoption while the store carries no artifact
/// binding.**
///
/// A closure's persistent identity includes *which code* it closes over, and a
/// bare local-origin ordinal collides across artifacts — two independently
/// compiled artifacts both number their first callable unit `0`. Binding the
/// ordinal into an artifact-scoped namespace is what makes the identity
/// meaningful, so an unbound store cannot mint one and must **fail closed**
/// rather than mint an ordinal that aliases.
pub const BOUNDARY_ERR_UNBOUND: i64 = -11;
/// ⛔ **The word names the RETIRED durable-closure lane** (`RT-FNSPLIT-C1`
/// `D5`, Architect `dec_21aa95jbsznfh` + addendum `dec_6xffebwj4s347`).
///
/// The `(PersistentClosure, Closure)` pair is **recognized ABI vocabulary and
/// is never admitted**. An ordinary closure is runtime-local and live-domain
/// only; it has no durable lane, and a callable cross-owner carrier is `B2F`'s
/// design rather than this node's.
///
/// ⭐ **Why this is its own status and not [`BOUNDARY_ERR_TAG`], which is the
/// whole point of the code existing.** Deleting the pair from the vocabulary
/// would have been the smaller change and it silently downgrades *"I refuse
/// this specific retired lane"* into *"I do not recognize this byte"* — the
/// same status arbitrary corruption produces. A refusal that cannot say **what**
/// it refused is the same failure class as a negative check that passes for any
/// reason. ⇒ The lane keeps its name **so that it can be refused by name.**
///
/// ⚠ Distinct from [`BOUNDARY_ERR_RELATION`] on the other side, too:
/// `PersistentClosure + Bool` is a **malformed pair** and still returns `-8`,
/// while `PersistentClosure + Closure` is a **well-formed pair naming a retired
/// capability** and returns this. Collapsing the two would lose the ability to
/// tell a corrupt word from a lawful word the ABI no longer honours.
pub const BOUNDARY_ERR_RETIRED_LANE: i64 = -12;

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
    /// Whether the store has taken exclusive ownership (see [`ARENA_SEALED`]).
    sealed: bool,
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

    /// One header field's published value.
    ///
    /// ⛔ **Exhaustive, no `_` arm** — that is the mechanism, not a style
    /// choice. A new [`RegionHeaderField`] variant fails to compile here, which
    /// is strictly stronger than any test could be.
    fn header_value(&self, field: RegionHeaderField) -> u64 {
        match field {
            RegionHeaderField::Nodes => self.nodes.as_ptr() as u64,
            RegionHeaderField::NodeCount => self.live_nodes as u64,
            RegionHeaderField::Words => self.words.as_ptr() as u64,
            RegionHeaderField::WordCount => self.live_words as u64,
            RegionHeaderField::Names => self.names.as_ptr() as u64,
            RegionHeaderField::NameCount => self.names.len() as u64,
            RegionHeaderField::NodeCapacity => (self.nodes.len() / NODE_WORDS) as u64,
            RegionHeaderField::WordCapacity => self.words.len() as u64,
            RegionHeaderField::Persistent => self.persistent,
            // Everything materialized before publication is frozen; emitted
            // code constructs strictly beyond it.
            RegionHeaderField::Frozen => self.live_nodes as u64,
            RegionHeaderField::Data => self.data.as_ptr() as u64,
            RegionHeaderField::DataCount => self.live_data as u64,
            RegionHeaderField::DataCapacity => self.data.len() as u64,
            RegionHeaderField::NativeInt => self.native_int,
            RegionHeaderField::Limbs => self.limbs.as_ptr() as u64,
            RegionHeaderField::LimbCount => self.live_limbs as u64,
            RegionHeaderField::LimbCapacity => self.limbs.len() as u64,
            RegionHeaderField::Sealed => u64::from(self.sealed),
        }
    }

    /// ⛔ **Take exclusive ownership: the seal/quiescence handoff.**
    ///
    /// Writes the flag into the **published** header, which is the copy emitted
    /// code reads, and not merely into the Rust-side field — otherwise the
    /// mutators this is supposed to stop would never see it. Idempotent.
    ///
    /// ⚠ Sealing a region that was never published is still meaningful: there
    /// is no published base for emitted code to hold, so the region is already
    /// quiescent and the flag simply records it.
    fn seal(&mut self) {
        self.sealed = true;
        if !self.header.is_empty() {
            self.header[(ARENA_SEALED / 8) as usize] = 1;
        }
    }

    /// Whether the store holds exclusive ownership of this region.
    ///
    /// Reads the **published** header when there is one, on the same rule as
    /// [`Self::node_count`]: the published copy is the one a writer consults,
    /// so it is the one that decides whether a write would have been refused.
    pub fn is_sealed(&self) -> bool {
        match self.header.first() {
            None => self.sealed,
            Some(_) => self.header[(ARENA_SEALED / 8) as usize] != 0,
        }
    }

    /// Words in the published header, or `0` before publication. The layout
    /// control measures this rather than re-deriving the constant it checks.
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

    /// Take Rust-side ownership of everything emitted code appended.
    ///
    /// ⛔ **Part of adoption, and not optional.** The emitted allocator bumps the
    /// *published header's* live counts; the Rust-side counts still describe the
    /// region as it was at publication. Re-publishing without this would reset
    /// the counts and truncate exactly the nodes the store is adopting — the
    /// referent would vanish from under its own new identity.
    fn absorb_published_counts(&mut self) {
        if self.header.is_empty() {
            return;
        }
        self.live_nodes = self.header[(ARENA_NODE_COUNT / 8) as usize] as usize;
        self.live_words = self.header[(ARENA_WORD_COUNT / 8) as usize] as usize;
        self.live_data = self.header[(ARENA_DATA_COUNT / 8) as usize] as usize;
        self.live_limbs = self.header[(ARENA_LIMB_COUNT / 8) as usize] as usize;
    }

    /// Install a node's `SlotId`. ⛔ **Store-owned.** This is the only writer
    /// of [`NODE_SLOT`] outside `push_node`, it is private to this module, and
    /// the only caller is [`BoundaryValueStore::adopt`]. Emitted code has no
    /// setter for this field and gains none —
    /// `EMITTED_WRITABLE_NODE_OFFSETS` makes emitting one a panic.
    fn set_node_slot(&mut self, index: u64, slot: SlotId) {
        let base = index as usize * NODE_WORDS;
        self.nodes[base + (NodeField::Slot as usize)] = slot;
    }

    /// Repoint one child word — used when adoption canonicalizes a child onto
    /// an existing store-owned node.
    fn set_word_at(&mut self, index: u64, word: BoundaryWord) {
        self.words[index as usize] = word.0;
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
        // ⛔ The Rust builders enforce the SAME tag x node-class legality
        // contract the emitted allocator does — ONE CONTRACT, TWO ENFORCEMENT
        // PATHS, and deliberately not one table. Emitted allocation reads the
        // partition-derived plan relation, which is the sole authority; Rust
        // reads `BOUNDARY_TAG_CLASS_RELATION`, the crate-private mirror
        // reconciled to that authority over the full finite product. A pair no
        // disposition can produce must be unbuildable from either side, and the
        // reconciliation is what makes "either side" mean one contract rather
        // than two independently maintained answers.
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
        // ⛔ Placed by field, not by position, through a `match` with no `_`
        // arm: a new `NodeField` is a compile error here until it is given a
        // value, so a node word can never be silently written as a zero.
        for field in NodeField::ALL {
            self.nodes[base + (field as usize)] = match field {
                NodeField::Class => class as u64,
                NodeField::Owner => tag.referent_owner() as u64,
                NodeField::Slot => slot,
                NodeField::TagId => tag_id,
                NodeField::Payload => payload,
                NodeField::FieldCount => children.len() as u64,
                NodeField::FieldsAt => fields_at,
                NodeField::Extent => extent,
                NodeField::LimbsAt => limbs_at,
                NodeField::LimbCount => limbs.len() as u64,
                // Rust-materialized magnitudes come from
                // `canonical_sign_and_limbs` and are asserted canonical above,
                // so they are born sealed. Emitted construction earns the seal
                // from `ken_boundary_seal_int_local`.
                NodeField::IntSealed => u64::from(extent == BOUNDARY_INT_REGION_LIMBS),
            };
        }
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
    /// ⛔ **The inventory is the sole authority and this is its consumer.** The
    /// vector is sized from [`RegionHeaderField::ALL`], each word is placed at
    /// its own field's offset, and [`BoundaryRegion::header_value`]'s `match`
    /// has **no `_` arm** — so a field added to the inventory is a compile error
    /// here until it is given a value, and can never be silently published as a
    /// zero.
    ///
    /// ⚠ The previous form was a positional `vec![…]` whose length nobody
    /// derived: it published **18** words against a **17**-word declared extent,
    /// and the constant had no consumer anywhere in the tree. Checking one
    /// hand-maintained number against another cannot detect that — there has to
    /// be one number.
    pub fn publish(&mut self) -> *mut u64 {
        let mut header = vec![0u64; RegionHeaderField::ALL.len()];
        for field in RegionHeaderField::ALL {
            header[field as usize] = self.header_value(field);
        }
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
/// ## Emitted construction is content-addressed by ADOPTION, not by the allocator
///
/// A node the **store** materialized carries its [`SlotId`] on arrival. A node
/// **emitted code** constructs carries [`NULL_SLOT`], because interning is a
/// content-addressing operation over a whole value and is not Θ(1) at a
/// construction site — the allocator cannot do it.
///
/// ⛔ **That state is `PendingStoreAdoption`, not a published outcome.** This
/// module's own layout contract says a null [`NODE_SLOT`] *denotes*
/// invocation-arena ownership, so a persistent-tagged node still carrying one is
/// internally inconsistent and must not escape. [`BoundaryValueStore::adopt`] is
/// the store-owned boundary that resolves it: seal, validate the reachable
/// graph, canonicalize, intern, and mint or reuse the real slot. Reserving
/// persistent-region storage is storage *governance*; only adoption is identity.
///
/// ⚠ **This is therefore not a second identity authority and never was one.**
/// The store mints every [`SlotId`], grants every byte of space, and is the only
/// writer of [`NODE_SLOT`] — emitted code has no setter for that field and
/// `define_store_node_word` refuses to build one.
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

    /// Whether the store holds exclusive ownership (see [`ARENA_SEALED`]).
    pub fn is_sealed(&self) -> bool {
        self.0.is_sealed()
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
    /// ⭐⭐ **`D2` — the ONE identity authority, as this store sees it.**
    ///
    /// A carrier `TagId` / record-field name id is an **artifact-static
    /// identity**: a packed span into the plan's own name arena, issued by
    /// `constructor_symbol_identity` / `record_field_identity`. ⛔ It is **not
    /// computable from the symbol string** — no formula here could reproduce
    /// it — so this store does not derive one. It is **told**, and it refuses
    /// symbols it was never told about.
    ///
    /// ⚠ **Why this is not `symbol_ids` under another name.** `intern_symbol`
    /// *mints*: dense insertion-order numbering, per store instance, minting on
    /// miss. ⇒ The same constructor gets a different id in a different store or
    /// a different insertion order, and a compiled-once body cannot compare
    /// against that (`§2e`). These two maps are a **view over an authority that
    /// lives elsewhere**, ⛔ never a source.
    carrier_identities: BTreeMap<RuntimeSymbol, u64>,
    /// The reverse view — `D2` rules that the reverse lookup survives *as a
    /// view over the one authority*, ⛔ never as a second source. Written only
    /// by [`BoundaryValueStore::issue_carrier_identity`], so it cannot disagree
    /// with the forward map.
    carrier_symbols: BTreeMap<u64, RuntimeSymbol>,
    /// The persistent region every persistent word indexes.
    image: BoundaryPersistentImage,
    /// `SlotId -> persistent node index`. ⭐ **This is what makes the word an
    /// identity rather than a locator:** the store's slot decides the index, so
    /// materializing one value in two different invocations yields the *same*
    /// word, and the store stays the sole identity authority.
    placement: BTreeMap<SlotId, u64>,
    /// The artifact whose callable-unit ordinals this store's closures name.
    ///
    /// ⛔ **`None` is a real state and it fails closed.** A closure's identity is
    /// artifact-scoped (see [`boundary_code_id`]); an unbound store has no
    /// namespace to scope it in, so `Closure` adoption returns
    /// [`BOUNDARY_ERR_UNBOUND`] rather than falling back to the bare ordinal.
    /// Ground adoption is unaffected — a ground value's identity is its content.
    artifact: Option<crate::RuntimeArtifactIdentity>,
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
            carrier_identities: BTreeMap::new(),
            carrier_symbols: BTreeMap::new(),
            image: BoundaryPersistentImage::default(),
            placement: BTreeMap::new(),
            artifact: None,
        }
    }

    /// Bind the artifact whose callable-unit ordinals this store's closures
    /// name. Until this is called, `Closure` adoption fails closed with
    /// [`BOUNDARY_ERR_UNBOUND`].
    pub fn bind_artifact(&mut self, identity: crate::RuntimeArtifactIdentity) {
        self.artifact = Some(identity);
    }

    /// The bound artifact, if any.
    pub fn artifact(&self) -> Option<&crate::RuntimeArtifactIdentity> {
        self.artifact.as_ref()
    }

    /// ⛔ **The seal/quiescence handoff — take exclusive ownership of the
    /// persistent region before adopting anything out of it.**
    ///
    /// [`Self::adopt`] refuses to run against an unsealed region, because every
    /// phase after it reads a snapshot: absorbing the published counts, walking
    /// the reachable graph, minting identity and publishing all assume the graph
    /// they are looking at is the graph that was validated. A writer still able
    /// to run makes those two different graphs.
    ///
    /// ⚠ **`&mut self` is not the proof.** Emitted code holds the raw region
    /// base it was published and never consults the borrow checker to use it, so
    /// exclusivity has to be expressed somewhere the mutators actually read —
    /// which is [`ARENA_SEALED`], in the published header.
    pub fn seal_persistent(&mut self) {
        self.image.0.seal();
    }

    /// Whether the persistent region is sealed.
    pub fn is_persistent_sealed(&self) -> bool {
        self.image.0.is_sealed()
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

    /// ⛔ **The store-owned ADOPTION boundary — the only way an emitted-
    /// constructed node becomes a published persistent handle.**
    ///
    /// Emitted code can construct and seal a persistent node, but the node it
    /// leaves behind carries [`NULL_SLOT`], and **this module's own layout
    /// contract says a null slot denotes invocation-arena ownership.** So such a
    /// node is not a persistent `HandleWord` at all — it is an internal
    /// *pending-adoption* state, and classifying it as a published outcome with
    /// "no store identity" was the defect this closes: a consumer can recover
    /// the *absence* of an identity, which is not recovering the same identity
    /// intact. Reserving persistent-region storage is storage governance, never
    /// adoption.
    ///
    /// What this does, in order:
    ///
    /// 1. **Bottom-up over the reachable graph**, so no parent is adopted while
    ///    a reachable child is still pending. An invocation-owned child is
    ///    [`BOUNDARY_ERR_ESCAPE`] — the same rule, re-checked at the boundary
    ///    that matters rather than inherited from construction.
    /// 2. **Canonicalize and intern** through [`BoundaryValueStore::persist`],
    ///    the landed content-addressed path — so equal values independently
    ///    emitted converge on one `SlotId` and unequal values cannot alias,
    ///    because that is what `Store::intern` already guarantees.
    /// 3. **Mint or reuse.** A slot already placed returns the *existing*
    ///    store-owned word and the pending node is abandoned; otherwise the slot
    ///    is installed on this node and its placement recorded.
    ///
    /// ⭐ **Mint authority stays the store's.** `set_node_slot` is private to
    /// this module and this is its only caller; emitted code has no setter for
    /// `NODE_SLOT` and the emission latch keeps it that way.
    ///
    /// ⚠ Fails closed with an exact status, **before** the word can be
    /// published or escape, for anything it cannot validate — including a class
    /// this decoder does not read back (`Closure`, `HostResult`,
    /// `BorrowedOpaque`), which is a conservative reject rather than a silent
    /// admission.
    pub fn adopt(&mut self, word: BoundaryWord) -> Result<BoundaryWord, i64> {
        let tag = word.tag().ok_or(BOUNDARY_ERR_TAG)?;
        if tag.referent_owner() != BoundaryReferentOwner::PersistentStore {
            return Err(BOUNDARY_ERR_SHAPE);
        }
        // ── phase 1: the sealed handoff ─────────────────────────────────────
        // Everything below reads a snapshot. Refuse to start unless emitted
        // writers have already been shut out of it.
        if !self.image.0.is_sealed() {
            return Err(BOUNDARY_ERR_SEALED);
        }
        // The store takes ownership of what emitted code appended before it
        // validates any of it.
        self.image.0.absorb_published_counts();

        // ── phase 2: validate the COMPLETE reachable graph, before anything
        //             is canonicalized, interned, minted or published ────────
        let order = self.validate_reachable(word.payload())?;

        // ── phase 3: canonicalize in postorder, then mint ───────────────────
        let canonical = self.canonicalize(&order)?;
        let root = canonical
            .get(&word.payload())
            .copied()
            .unwrap_or_else(|| word.payload());
        Ok(BoundaryWord::handle(tag, root))
    }

    /// Whether this node already carries a store-minted identity.
    ///
    /// Such a node is **black on arrival**: it is canonical already and its
    /// reachable subtree was validated when it was adopted, so the walk neither
    /// descends into it nor re-interns it.
    fn already_owned(&self, index: u64) -> Result<bool, i64> {
        let slot = self
            .image
            .0
            .node_field(index, NODE_SLOT)
            .ok_or(BOUNDARY_ERR_BOUNDS)?;
        Ok(slot != NULL_SLOT)
    }

    /// Whether a node of this class may become store-resident at all.
    ///
    /// ⭐ **The single source of truth for the two phases that must agree.**
    /// `validate_reachable` (phase 2) consults it to refuse *before* anything is
    /// interned; `canonical_image` (phase 3) consults it as defence in depth. ⛔
    /// Two hand-written lists would be free to drift, and the drift would be
    /// silent — the phase-3 list alone was the `AC-V5` defect (below).
    ///
    /// ⚠ **This sentence was once false on one path, and the correction is the
    /// point.** An earlier revision placed the phase-2 call *after* both
    /// `already_owned` fast paths, so on the already-owned path phase 2 did not
    /// read this predicate at all and a closure with a pre-existing `NODE_SLOT`
    /// was admitted. ⇒ *"Both phases read one predicate"* is a claim about
    /// **every** path through each phase, not about the predicate existing.
    ///
    /// ⛔ Exhaustive with no `_` arm on purpose: a new [`BoundaryClass`] must
    /// decide here whether it is persistable rather than inheriting an answer.
    fn class_is_persistable(class: BoundaryClass) -> bool {
        match class {
            BoundaryClass::Bool
            | BoundaryClass::Int
            | BoundaryClass::Bytes
            | BoundaryClass::String
            | BoundaryClass::Constructor
            | BoundaryClass::Record => true,
            // ⛔ Transitively non-persistable. `41 §2.1` denies an ordinary
            // closure canonical bytes, slot identity and persistence outright;
            // `HostResult` and `BorrowedOpaque` die with the invocation.
            BoundaryClass::Closure
            | BoundaryClass::HostResult
            | BoundaryClass::BorrowedOpaque => false,
        }
    }

    /// Refuse a non-persistable node **at admission to the walk**, which is what
    /// makes `AC-V5`'s "before any byte, hash, slot or provenance exists" true
    /// of the mechanism rather than merely asserted about it.
    fn admit_persistable(&self, index: u64) -> Result<(), i64> {
        let bits = self
            .image
            .0
            .node_field(index, NODE_CLASS)
            .ok_or(BOUNDARY_ERR_BOUNDS)?;
        let class = BoundaryClass::from_bits(bits).ok_or(BOUNDARY_ERR_CLASS)?;
        if Self::class_is_persistable(class) {
            Ok(())
        } else {
            Err(BOUNDARY_ERR_ESCAPE)
        }
    }

    /// ⛔ **Validate the complete reachable graph — iteratively — and return it
    /// in postorder.**
    ///
    /// ⛔ **This is also where a closure is refused, and the position is the
    /// deliverable — not an implementation detail.** `canonicalize` is
    /// **postorder and minting**: it builds each node's image and immediately
    /// `persist_image`s it, so by the time a closure node is reached its
    /// captured *compound* children already hold canonical bytes and
    /// store-minted slots. Refusing there would satisfy "the adoption returns
    /// `Err`" while violating `AC-V5`'s actual requirement, that refusal precede
    /// **any** byte, hash, slot or provenance. ⚠ A control whose closure carries
    /// only *immediate* captures cannot see that difference — which is exactly
    /// why the first version of this passed.
    ///
    /// ⭐ **Tri-colour, with an explicit heap stack, and both halves matter.**
    ///
    /// *Tri-colour* is what tells a **cycle** from a **shared DAG**, which a
    /// visited-set cannot: a second edge into a node is legal when that node is
    /// already finished (**black** — the sub-value is simply shared, and it
    /// reuses one canonical child) and malformed when the node is still on the
    /// stack (**grey** — the edge points back into the path that reached it).
    /// A plain "have I seen this?" set collapses those two into one answer and
    /// would have to reject every shared child to be safe.
    ///
    /// *Iterative* is what makes **depth** a non-property. The former recursive
    /// walk spent one host frame per node, so a finite, perfectly well-formed
    /// deep chain died of stack exhaustion — ⛔ **and `AC-10` admits those
    /// values, so crashing on one is as wrong as rejecting one.** Cycle-safety
    /// and depth-safety are different properties; a cycle guard closes only the
    /// first. Here the frontier is a `Vec` on the heap, so the bound is the
    /// region's own capacity rather than the thread's stack.
    ///
    /// ⚠ **The node-index key is sufficient only because one walk is scoped to
    /// one persistent image** — this store's own. If a walk could ever span
    /// images, the key would have to carry image identity too, or two nodes at
    /// the same index in different images would read as one.
    fn validate_reachable(&self, root: u64) -> Result<Vec<u64>, i64> {
        #[derive(Clone, Copy, Eq, PartialEq)]
        enum Colour {
            /// On the stack: an edge back to one of these is a cycle.
            Grey,
            /// Finished: a further edge to one of these is legal sharing.
            Black,
        }

        let mut colour: BTreeMap<u64, Colour> = BTreeMap::new();
        let mut order: Vec<u64> = Vec::new();
        // `(node index, next child offset)` — the frontier, on the heap.
        let mut stack: Vec<(u64, u64)> = Vec::new();

        // Admission site 1 of 2 — the root.
        //
        // ⛔ **BEFORE the already-owned fast path, and the order is the
        // deliverable.** `already_owned` skips a node on the grounds that "its
        // reachable subtree was validated when it was adopted" — an *inductive*
        // argument that presupposes adoption always refused this class. For the
        // closure class that premise is exactly what was broken, so trusting a
        // pre-existing `NODE_SLOT` here is circular: a closure-classed node
        // carrying a non-`NULL_SLOT` would be waved through as already
        // canonical, preserving a store-resident ordinary closure — the outcome
        // `41 §2.1` forbids outright.
        self.admit_persistable(root)?;
        if self.already_owned(root)? {
            return Ok(order);
        }
        colour.insert(root, Colour::Grey);
        stack.push((root, 0));

        while let Some(&(index, cursor)) = stack.last() {
            let count = self
                .image
                .0
                .node_field(index, NODE_FIELD_COUNT)
                .ok_or(BOUNDARY_ERR_BOUNDS)?;
            if cursor >= count {
                stack.pop();
                colour.insert(index, Colour::Black);
                order.push(index);
                continue;
            }
            // Advance before descending, so the frame resumes at the next child
            // rather than re-walking this one.
            stack
                .last_mut()
                .expect("the frame was just read from the top of the stack")
                .1 = cursor + 1;

            let at = self
                .image
                .0
                .node_field(index, NODE_FIELDS_AT)
                .ok_or(BOUNDARY_ERR_BOUNDS)?;
            let child = self
                .image
                .0
                .word_at(at + cursor)
                .ok_or(BOUNDARY_ERR_BOUNDS)?;
            match child.tag().ok_or(BOUNDARY_ERR_TAG)?.referent_owner() {
                BoundaryReferentOwner::NoReferent => {}
                // ⛔ A persistent parent must not publish reaching a referent
                // that dies with the invocation. Checked here, at the boundary
                // that matters, rather than inherited from construction.
                BoundaryReferentOwner::InvocationArena => return Err(BOUNDARY_ERR_ESCAPE),
                BoundaryReferentOwner::PersistentStore => {
                    let next = child.payload();
                    match colour.get(&next) {
                        // Back-edge into the path that reached here.
                        Some(Colour::Grey) => return Err(BOUNDARY_ERR_CYCLE),
                        // Shared child: legal, already finished, reused.
                        Some(Colour::Black) => {}
                        None => {
                            // Admission site 2 of 2 — every descendant. ⭐ The
                            // transitive half: a closure nested at any depth is
                            // refused here, while the walk is still read-only
                            // and nothing has been interned or minted.
                            //
                            // ⛔ Again BEFORE `already_owned`, for the reason
                            // given at the root: the fast path's soundness is
                            // inductive on adoption having refused this class,
                            // so it cannot be the thing that decides whether
                            // this class is admitted.
                            self.admit_persistable(next)?;
                            if self.already_owned(next)? {
                                colour.insert(next, Colour::Black);
                            } else {
                                colour.insert(next, Colour::Grey);
                                stack.push((next, 0));
                            }
                        }
                    }
                }
            }
        }
        Ok(order)
    }

    /// Canonicalize a validated postorder into store-owned identities.
    ///
    /// ⛔ **Runs only after validation is complete**, so no identity is minted
    /// and no root is published for a graph that turns out to be malformed
    /// further in. Postorder means every child is canonical before its parent is
    /// read, which is what lets a parent's image be assembled from images that
    /// already exist instead of re-walking the subtree.
    ///
    /// Returns `original index -> canonical index`.
    fn canonicalize(&mut self, order: &[u64]) -> Result<BTreeMap<u64, u64>, i64> {
        let mut canonical: BTreeMap<u64, u64> = BTreeMap::new();
        // One image per node, dropped as soon as its last parent has consumed
        // it: a parent's image *contains* its children's, so retaining every
        // entry would make a deep chain quadratic in memory for no benefit.
        let mut images: BTreeMap<u64, Value> = BTreeMap::new();
        let mut pending_parents = self.in_degrees(order)?;

        for &index in order {
            let count = self
                .image
                .0
                .node_field(index, NODE_FIELD_COUNT)
                .ok_or(BOUNDARY_ERR_BOUNDS)?;
            let at = self
                .image
                .0
                .node_field(index, NODE_FIELDS_AT)
                .ok_or(BOUNDARY_ERR_BOUNDS)?;

            let value = self.canonical_image(index, at, count, &images)?;

            // Release each child's image now that this parent has consumed it.
            for offset in 0..count {
                let child = self
                    .image
                    .0
                    .word_at(at + offset)
                    .ok_or(BOUNDARY_ERR_BOUNDS)?;
                if child.tag().ok_or(BOUNDARY_ERR_TAG)?.referent_owner()
                    != BoundaryReferentOwner::PersistentStore
                {
                    continue;
                }
                if let Some(remaining) = pending_parents.get_mut(&child.payload()) {
                    *remaining -= 1;
                    if *remaining == 0 {
                        images.remove(&child.payload());
                    }
                }
                // ⛔ **Preserve the child's OWN tag.** Rewriting it as
                // `PersistentGround` would silently change what the child *is*;
                // the tag is the child's, and only the index is this step's to
                // move.
                //
                // ⚠ This previously justified itself by the nested-closure case
                // — "retagging a `PersistentClosure` into a ground handle." That
                // example is now **unreachable**: `validate_reachable` refuses a
                // closure-classed node at any depth before this loop runs. The
                // rule is kept because it is right for every tag, ⛔ not because
                // the closure it cited can still arrive.
                if let Some(&target) = canonical.get(&child.payload()) {
                    if target != child.payload() {
                        let tag = child.tag().ok_or(BOUNDARY_ERR_TAG)?;
                        self.image
                            .0
                            .set_word_at(at + offset, BoundaryWord::handle(tag, target));
                    }
                }
            }

            let slot = self.persist_image(&value).ok_or(BOUNDARY_ERR_SHAPE)?;
            // Mint, or reuse an identity this store already owns.
            let placed = match self.placement.get(&slot) {
                Some(&existing) => existing,
                None => {
                    self.image.0.set_node_slot(index, slot);
                    self.placement.insert(slot, index);
                    index
                }
            };
            canonical.insert(index, placed);
            images.insert(index, value);
        }
        Ok(canonical)
    }

    /// How many parents inside this walk reference each node, so an image can be
    /// released the moment its last parent has read it.
    fn in_degrees(&self, order: &[u64]) -> Result<BTreeMap<u64, usize>, i64> {
        let mut degrees: BTreeMap<u64, usize> = BTreeMap::new();
        for &index in order {
            let count = self
                .image
                .0
                .node_field(index, NODE_FIELD_COUNT)
                .ok_or(BOUNDARY_ERR_BOUNDS)?;
            let at = self
                .image
                .0
                .node_field(index, NODE_FIELDS_AT)
                .ok_or(BOUNDARY_ERR_BOUNDS)?;
            for offset in 0..count {
                let child = self
                    .image
                    .0
                    .word_at(at + offset)
                    .ok_or(BOUNDARY_ERR_BOUNDS)?;
                if child.tag().ok_or(BOUNDARY_ERR_TAG)?.referent_owner()
                    == BoundaryReferentOwner::PersistentStore
                {
                    *degrees.entry(child.payload()).or_insert(0) += 1;
                }
            }
        }
        Ok(degrees)
    }

    /// ⛔ **The closed canonical-image layer: one persistent node → the [`Value`]
    /// that IS its content-addressed image.**
    ///
    /// ⭐ **Total over [`BoundaryClass`] with no `_` arm**, which is the point: a
    /// new class is a compile error here, not a value that silently has no
    /// canonical form. This is the *canonicalizer* phase, and it is the sole
    /// authority for what a persistent node's canonical image **is**.
    ///
    /// ⛔ **It is NOT the authority on what may be persisted.** That decision
    /// belongs to `validate_reachable` (phase 2), because this phase mints:
    /// every arm below runs inside a postorder loop that interns each image and
    /// writes a `NODE_SLOT` immediately. A refusal issued *here* would therefore
    /// arrive after the node's descendants already had canonical bytes and
    /// store-minted slots. The non-persistable arm below is retained as
    /// **defence in depth** and for exhaustiveness; both phases read one
    /// predicate, [`BoundaryValueStore::class_is_persistable`], so they cannot
    /// disagree.
    ///
    /// ⚠ **This block previously said `Value::Closure` "already *is* the
    /// normative image — authoritative code identity plus the full ordered
    /// canonical captured environment," and that the layer "serves ground
    /// values and closures."** Both statements described the retired mechanism:
    /// `41 §2.1` grants an ordinary closure no canonical image at all, and `D1`
    /// deleted the variant. ⛔ Replaced rather than annotated — an appended
    /// correction leaves the confident false sentence in the position a reader
    /// believes.
    fn canonical_image(
        &mut self,
        index: u64,
        at: u64,
        count: u64,
        images: &BTreeMap<u64, Value>,
    ) -> Result<Value, i64> {
        let class_bits = self
            .image
            .0
            .node_field(index, NODE_CLASS)
            .ok_or(BOUNDARY_ERR_BOUNDS)?;
        let class = BoundaryClass::from_bits(class_bits).ok_or(BOUNDARY_ERR_CLASS)?;
        let payload = self
            .image
            .0
            .node_field(index, NODE_PAYLOAD)
            .ok_or(BOUNDARY_ERR_BOUNDS)?;

        match class {
            // No handle tag admits `Bool` in `BOUNDARY_TAG_CLASS_RELATION`, so
            // no node is ever this class; a booolean crosses as an immediate.
            // Refused rather than given an image it could never be asked for.
            BoundaryClass::Bool => Err(BOUNDARY_ERR_CLASS),
            BoundaryClass::Int => {
                let marker = self
                    .image
                    .0
                    .node_field(index, NODE_EXTENT)
                    .ok_or(BOUNDARY_ERR_BOUNDS)?;
                if marker == crate::native_int::NATIVE_INT_SMALL_TAG_V1 {
                    // The store interns compounds only, and `canonical_big_image`
                    // is total over both arms — a `Value::SmallInt` would trip
                    // the store's `is_compound` assertion.
                    return Ok(crate::RuntimeIntV1::Small(payload as i64).canonical_big_image());
                }
                if marker == BOUNDARY_INT_REGION_LIMBS {
                    let limbs = self
                        .image
                        .0
                        .node_limbs(index)
                        .ok_or(BOUNDARY_ERR_BOUNDS)?
                        .to_vec();
                    // The seal is what makes this readable at all, and the
                    // canonicity law is the same one the emitted seal checks.
                    if !boundary_int_magnitude_is_canonical(payload, &limbs) {
                        return Err(BOUNDARY_ERR_SHAPE);
                    }
                    let sign = match payload {
                        0 => crate::Sign::NonNegative,
                        1 => crate::Sign::Negative,
                        _ => return Err(BOUNDARY_ERR_SHAPE),
                    };
                    return Ok(crate::RuntimeIntV1::from_canonical_parts(sign, limbs)
                        .canonical_big_image());
                }
                // A marker whose storage is invocation-scoped cannot back a
                // persistent value; a marker with no row backs nothing at all.
                Err(BOUNDARY_ERR_SHAPE)
            }
            BoundaryClass::Bytes => Ok(Value::Bytes(
                self.image
                    .0
                    .node_data(index)
                    .ok_or(BOUNDARY_ERR_BOUNDS)?
                    .to_vec(),
            )),
            BoundaryClass::String => {
                let bytes = self
                    .image
                    .0
                    .node_data(index)
                    .ok_or(BOUNDARY_ERR_BOUNDS)?
                    .to_vec();
                let text = String::from_utf8(bytes).map_err(|_| BOUNDARY_ERR_SHAPE)?;
                Ok(Value::String(text))
            }
            BoundaryClass::Constructor => {
                // The node stores the planner's artifact-local carrier
                // identity, not this store's dense canonical-value symbol id.
                // Resolve through the planner-issued reverse view, then intern
                // only into the store's distinct canonical-value namespace.
                let identity = self
                    .image
                    .0
                    .node_field(index, NODE_TAG_ID)
                    .ok_or(BOUNDARY_ERR_BOUNDS)?;
                let constructor = self
                    .carrier_symbol(identity)
                    .ok_or(BOUNDARY_ERR_SHAPE)?
                    .to_string();
                let constructor_id = self.intern_symbol(&constructor) as u32;
                let args = self.child_images(at, count, images)?;
                Ok(Value::Constructor {
                    constructor_id,
                    args,
                })
            }
            BoundaryClass::Record => {
                // `Value::Record` carries a `type_id` and positional fields — it
                // drops names — so the record's ordered field-name list IS its
                // type identity, interned as one symbol. Same rule the Rust-side
                // materialization uses, so the two converge on one slot.
                let mut names = Vec::with_capacity(count as usize);
                for offset in 0..count {
                    let id = self
                        .image
                        .0
                        .name_at(at + offset)
                        .ok_or(BOUNDARY_ERR_BOUNDS)?;
                    names.push(
                        self.carrier_symbol(id)
                            .ok_or(BOUNDARY_ERR_SHAPE)?
                            .to_string(),
                    );
                }
                let type_id = self.intern_symbol(&format!("record:{}", names.join(","))) as u32;
                let fields = self.child_images(at, count, images)?;
                Ok(Value::Record { type_id, fields })
            }
            // ⛔ Invocation-owned represented arms. They are not narrowed and not
            // reclassified — they are simply never placed in the permanent
            // store, and their transfer is governed by the invocation arena's
            // escape paths.
            //
            // ⛔ **`Closure` joined this arm when the canonical carrier lost its
            // closure variant.** An ordinary closure is runtime-local, so a
            // persistent node handed one is precisely the case
            // `BOUNDARY_ERR_ESCAPE` names. This arm previously bound an artifact
            // identity and built a `Value::Closure` from the full ordered
            // captures; `41 §2.1` forbids that outcome — a closure is
            // transitively non-persistable.
            //
            // ⛔ **UNREACHABLE in `adopt`, and deliberately kept.**
            // `validate_reachable` refuses these three classes at admission, so
            // no such node survives into this phase. ⚠ An earlier revision said
            // refusal "must happen before bytes, digest, or slot exist, which is
            // why the refusal is here at canonicalization" — the requirement was
            // right and the position was wrong, since this phase mints as it
            // goes. The arm stays because the match is total over the class
            // taxonomy and because a future caller could reach this function by
            // another route.
            //
            // ⚠ The tag/class taxonomy, the `(tag, class)` relation, the storage
            // shape and the CLIF emitters are deliberately UNCHANGED: B2V's
            // representation lane is a named residual owned by the `FNSPLIT`
            // re-cut (`SPEC-STORE-SPLIT` §7 item 1). Only the arm that
            // *constructed a canonical closure image* is gone.
            BoundaryClass::Closure
            | BoundaryClass::HostResult
            | BoundaryClass::BorrowedOpaque => Err(BOUNDARY_ERR_ESCAPE),
        }
    }

    /// The ordered canonical images of one node's children.
    fn child_images(
        &self,
        at: u64,
        count: u64,
        images: &BTreeMap<u64, Value>,
    ) -> Result<Vec<Value>, i64> {
        let mut out = Vec::with_capacity(count as usize);
        for offset in 0..count {
            let child = self
                .image
                .0
                .word_at(at + offset)
                .ok_or(BOUNDARY_ERR_BOUNDS)?;
            out.push(self.child_image(child, images)?);
        }
        Ok(out)
    }

    /// One child word, as the canonical image it denotes.
    ///
    /// ⛔ Exhaustive over [`BoundaryTag`] with no `_` arm — a new tag is a
    /// compile error here rather than a child that silently has no image.
    fn child_image(&self, word: BoundaryWord, images: &BTreeMap<u64, Value>) -> Result<Value, i64> {
        match word.tag().ok_or(BOUNDARY_ERR_TAG)? {
            BoundaryTag::ImmediateBool => Ok(Value::Bool(word.payload() == 1)),
            BoundaryTag::ImmediateInt => Ok(Value::SmallInt(word.signed_payload())),
            // ⚠ **A measured gap, refused rather than guessed.** These three
            // tags are constructible as children of a persistent aggregate, and
            // `Value` has no arm that means "exit status" or "bounded Nat". The
            // landed reader refused them too; picking a `Value` arm here would
            // be inventing the semantics rather than encoding them, so the
            // refusal is exact and reported instead of papered over.
            BoundaryTag::ImmediateExitStatus
            | BoundaryTag::ImmediateBoundedNat
            | BoundaryTag::ImmediateStructuralNat => Err(BOUNDARY_ERR_CLASS),
            // ⛔ An invocation-owned aggregate under a persistent parent is the
            // dangling relation itself, so this reports `ERR_ESCAPE` rather
            // than `ERR_CLASS`. The word is well-formed and its class is a
            // perfectly good aggregate class; what is wrong is the edge.
            // Reporting the class error here would name the wrong defect at a
            // reader that is otherwise the clearest place to see it.
            BoundaryTag::InvocationAggregate => Err(BOUNDARY_ERR_ESCAPE),
            BoundaryTag::PersistentGround | BoundaryTag::PersistentClosure => {
                if let Some(image) = images.get(&word.payload()) {
                    return Ok(image.clone());
                }
                // A node an earlier walk already adopted. ⭐ Recovered through
                // the **store's own** `slot -> bytes -> Value` path rather than
                // by re-reading the region, so the two paths corroborate each
                // other instead of one path being read twice.
                let slot = self
                    .image
                    .0
                    .node_field(word.payload(), NODE_SLOT)
                    .ok_or(BOUNDARY_ERR_BOUNDS)?;
                if slot == NULL_SLOT {
                    return Err(BOUNDARY_ERR_SHAPE);
                }
                self.decode_slot(slot).ok_or(BOUNDARY_ERR_SHAPE)
            }
            BoundaryTag::InvocationBorrowed | BoundaryTag::InvocationHostResult => {
                Err(BOUNDARY_ERR_ESCAPE)
            }
        }
    }

    /// Intern an already-canonical [`Value`] image, returning its slot.
    ///
    /// ⭐ **The `persist_image` seam.** [`Self::persist`] types over
    /// [`RuntimeGroundValue`], which has no closure arm; adoption produces
    /// [`Value`] directly, so it interns directly. Both end at the same
    /// `Store::intern`, which is what keeps identity single-authority.
    fn persist_image(&mut self, image: &Value) -> Option<SlotId> {
        image
            .is_compound()
            .then(|| self.store.intern(image).slot_id())
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

    /// ⭐ **`D2` — record the artifact-static identity the plan issued for
    /// `symbol`.** The caller holds the authority (`StaticTransitionPlan`); this
    /// store only remembers what it was handed.
    ///
    /// ⚠ Re-issuing the same pair is idempotent. Re-issuing a **different**
    /// identity for one symbol is a caller bug — two authorities by definition —
    /// so it is refused rather than silently overwritten.
    pub fn issue_carrier_identity(&mut self, symbol: &str, identity: u64) -> bool {
        match self.carrier_identities.get(symbol) {
            Some(existing) => *existing == identity,
            None => {
                self.carrier_identities
                    .insert(symbol.to_string(), identity);
                self.carrier_symbols.insert(identity, symbol.to_string());
                true
            }
        }
    }

    /// The artifact-static identity issued for `symbol`, if any.
    ///
    /// ⛔⛔ **THE MINTING BAN.** `None` means *"no authority has issued an
    /// identity for this symbol"*, and every carrier caller must **fail closed**
    /// on it. ⚠ Minting here on a miss would discharge `D2`'s sentence while
    /// preserving the exact defect it forbids: the mint *is* the second
    /// authority.
    pub fn carrier_identity(&self, symbol: &str) -> Option<u64> {
        self.carrier_identities.get(symbol).copied()
    }

    /// The symbol an artifact-static identity names — the reverse **view**.
    pub fn carrier_symbol(&self, identity: u64) -> Option<&str> {
        self.carrier_symbols.get(&identity).map(String::as_str)
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

    /// Recover the exact ground value of a store-owned boundary result after
    /// adoption. This is the native-observation inverse of materialization,
    /// not a lowering conversion: generated code still never converts a
    /// carried word back into a specialized value.
    pub(crate) fn observe_adopted_ground(
        &self,
        word: BoundaryWord,
    ) -> Option<RuntimeGroundValue> {
        match word.tag()? {
            BoundaryTag::ImmediateBool => {
                Some(RuntimeGroundValue::Bool(word.payload() != 0))
            }
            BoundaryTag::ImmediateInt => Some(RuntimeGroundValue::Int(
                word.signed_payload().into(),
            )),
            BoundaryTag::PersistentGround => {
                let slot = self.image.0.node_field(word.payload(), NODE_SLOT)?;
                if slot == NULL_SLOT {
                    return None;
                }
                self.ground_from_canonical(&self.decode_slot(slot)?)
            }
            BoundaryTag::ImmediateExitStatus
            | BoundaryTag::ImmediateBoundedNat
            | BoundaryTag::ImmediateStructuralNat
            | BoundaryTag::PersistentClosure
            | BoundaryTag::InvocationBorrowed
            | BoundaryTag::InvocationHostResult
            // An invocation aggregate is never store-adopted, so there is no
            // adopted ground value to observe. It joins the other two
            // invocation-owned tags rather than getting an arm of its own.
            | BoundaryTag::InvocationAggregate => None,
        }
    }

    fn ground_from_canonical(&self, value: &Value) -> Option<RuntimeGroundValue> {
        Some(match value {
            Value::Bool(value) => RuntimeGroundValue::Bool(*value),
            Value::SmallInt(value) => RuntimeGroundValue::Int((*value).into()),
            Value::BigInt { sign, limbs } => RuntimeGroundValue::Int(
                crate::RuntimeIntV1::from_canonical_parts(*sign, limbs.clone()),
            ),
            Value::Bytes(value) => RuntimeGroundValue::Bytes(value.clone()),
            Value::String(value) => RuntimeGroundValue::String(value.clone()),
            Value::Constructor {
                constructor_id,
                args,
            } => RuntimeGroundValue::Constructor {
                constructor: self.symbol(u64::from(*constructor_id))?.to_string(),
                args: args
                    .iter()
                    .map(|arg| self.ground_from_canonical(arg))
                    .collect::<Option<Vec<_>>>()?,
            },
            Value::Record { type_id, fields } => {
                let identity = self.symbol(u64::from(*type_id))?;
                let names = identity.strip_prefix("record:")?;
                let names = if names.is_empty() {
                    Vec::new()
                } else {
                    names.split(',').map(str::to_string).collect::<Vec<_>>()
                };
                if names.len() != fields.len() {
                    return None;
                }
                RuntimeGroundValue::Record {
                    fields: names
                        .into_iter()
                        .zip(fields.iter())
                        .map(|(name, value)| {
                            self.ground_from_canonical(value)
                                .map(|value| (name, value))
                        })
                        .collect::<Option<Vec<_>>>()?,
                }
            }
            Value::Char(_)
            | Value::Float(_)
            | Value::Float32(_)
            | Value::Int8(_)
            | Value::Int16(_)
            | Value::Int32(_)
            | Value::Int64(_)
            | Value::UInt8(_)
            | Value::UInt16(_)
            | Value::UInt32(_)
            | Value::UInt64(_)
            | Value::SmallDecimal { .. }
            | Value::BigDecimal { .. }
            | Value::Array { .. }
            | Value::Map { .. }
            | Value::Set { .. }
            | Value::Unknown => return None,
        })
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

    /// One node's `NODE_SLOT`, for `AC-V5`'s "nothing was minted" control.
    ///
    /// ⚠ Test-only and deliberately so: production has no business reading a
    /// node's minted identity back out of the image. It exists because
    /// *"adoption returned `Err`"* and *"adoption left no slot behind"* are
    /// different claims, and only the second is what `AC-V5` requires.
    #[cfg(test)]
    pub(crate) fn node_slot_of(&self, index: u64) -> Option<u64> {
        self.image.0.node_field(index, NODE_SLOT)
    }

    /// Install a `NODE_SLOT` on a node, so a control can exercise the
    /// **already-owned fast path** in `validate_reachable`.
    ///
    /// ⚠ Test-only. ⛔ There is no production route to this: a slot is minted
    /// by `canonicalize` and by nothing else. It exists because the fast path
    /// is reachable *by construction* — a node's `NODE_SLOT` is an ordinary
    /// field of the emitted image — and "no producer sets it today" is a claim
    /// about callers, not about reachability, which is the distinction `AC-V4`
    /// already turns on.
    #[cfg(test)]
    pub(crate) fn install_node_slot_for_test(&mut self, index: u64, slot: SlotId) {
        self.image.0.set_node_slot(index, slot);
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
                // ⭐⭐ `D2`: the carrier tag is the identity the PLAN issued, and
                // ⛔ an unissued constructor fails closed rather than minting one.
                let tag_id = self.carrier_identity(constructor)?;
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
                    // ⭐⭐ `D2`: record field identity, same one authority.
                    names.push(self.carrier_identity(name)?);
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
