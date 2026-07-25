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
    pub fn immediate(tag: BoundaryTag, payload: u64) -> Self {
        BoundaryWord((payload << BOUNDARY_TAG_BITS) | (tag as u64))
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

/// Byte stride of one arena node.
pub const BOUNDARY_NODE_STRIDE: i32 = 56;

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

/// Byte size of a **region header**.
///
/// ⭐ One layout serves both regions. The invocation arena and the persistent
/// image publish the *same* header shape, which is what lets a single
/// `resolve` select a region at run time and then read it with one set of
/// offsets. A second layout would be a second place for the offsets to drift.
pub const BOUNDARY_REGION_HEADER_BYTES: i32 = 80;

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
    live_nodes: usize,
    live_words: usize,
    header: Vec<u64>,
    /// Address of the persistent region's header, or `0`.
    persistent: u64,
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

    /// Nodes this region can still hold beyond the live count.
    pub fn node_capacity(&self) -> usize {
        self.nodes.len() / NODE_WORDS
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
    pub fn reserve(&mut self, nodes: usize, words: usize) {
        debug_assert!(
            self.header.is_empty(),
            "reserve before publish: growing a table moves it under the pointer"
        );
        let node_words = (self.live_nodes + nodes) * NODE_WORDS;
        self.nodes.resize(node_words, 0);
        self.words.resize(self.live_words + words, 0);
        self.names.resize(self.live_words + words, 0);
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
        children: &[BoundaryWord],
        names: &[u64],
    ) -> BoundaryWord {
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
    pub fn publish(&mut self) -> *mut u64 {
        self.header = vec![
            self.nodes.as_ptr() as u64,
            self.live_nodes as u64,
            self.words.as_ptr() as u64,
            self.live_words as u64,
            self.names.as_ptr() as u64,
            self.names.len() as u64,
            (self.nodes.len() / NODE_WORDS) as u64,
            self.words.len() as u64,
            self.persistent,
            // Everything materialized before publication is frozen; emitted code
            // constructs strictly beyond it.
            self.live_nodes as u64,
        ];
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
    pub fn reserve(&mut self, nodes: usize, words: usize) {
        self.0.reserve(nodes, words);
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
    pub fn reserve(&mut self, nodes: usize, words: usize) {
        self.0.reserve(nodes, words);
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
            .push_node(tag, class, NULL_SLOT, 0, payload, children, &[])
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
    pub fn reserve_persistent(&mut self, nodes: usize, words: usize) {
        self.image.reserve(nodes, words);
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

        let (class, tag_id, payload, children, names) = match value {
            // Both handled above; listed so this match stays exhaustive over the
            // value's own structure rather than falling through a wildcard.
            RuntimeGroundValue::Bool(_) => return None,
            RuntimeGroundValue::Int(_) => (BoundaryClass::Int, 0, 0, Vec::new(), Vec::new()),
            RuntimeGroundValue::Bytes(bytes) => (
                BoundaryClass::Bytes,
                0,
                bytes.len() as u64,
                Vec::new(),
                Vec::new(),
            ),
            RuntimeGroundValue::String(text) => (
                BoundaryClass::String,
                0,
                text.len() as u64,
                Vec::new(),
                Vec::new(),
            ),
            RuntimeGroundValue::Constructor { constructor, args } => {
                let tag_id = self.intern_symbol(constructor);
                let mut children = Vec::with_capacity(args.len());
                for arg in args {
                    children.push(self.materialize(arg)?);
                }
                (BoundaryClass::Constructor, tag_id, 0, children, Vec::new())
            }
            RuntimeGroundValue::Record { fields } => {
                let mut children = Vec::with_capacity(fields.len());
                let mut names = Vec::with_capacity(fields.len());
                for (name, field) in fields {
                    names.push(self.intern_symbol(name));
                    children.push(self.materialize(field)?);
                }
                (BoundaryClass::Record, 0, 0, children, names)
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
            &children,
            &names,
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
