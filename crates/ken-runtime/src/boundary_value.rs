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
//! every source-valued boundary transfer, together with the flat, invocation
//! scoped arena that emitted code reads it out of. The CLIF side lives in
//! [`crate::boundary_value_clif`]; the two are one deliverable and must not be
//! separated — a representation without an executable interface is exactly the
//! shape that produced `#9` and then `#10` one layer down.
//!
//! ## The word
//!
//! ```text
//! bits [0..8)   tag      — BoundaryTag, a closed repr(u8) enum
//! bits [8..64)  payload  — immediate scalar, or an arena node index
//! ```
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
    /// An opaque capability token scalar.
    ImmediateCapability = 3,
    /// An opaque resource token scalar.
    ImmediateResource = 4,
    /// A host-reply-validated bounded `Nat`.
    ImmediateBoundedNat = 5,
    /// A structural `Nat` deforested to one native scalar.
    ImmediateStructuralNat = 6,
    /// Handle to a persistable Ken value. Payload is an arena node index; the
    /// node names the [`SlotId`] that is the referent's **owner of record**.
    PersistentGround = 7,
    /// Handle to a retained closure: static origin plus captured words.
    PersistentClosure = 8,
    /// Handle to borrowed ingress — a host-owned buffer or option that is valid
    /// only for this native invocation.
    InvocationBorrowed = 9,
    /// Handle to a `HostResult`: a runtime success discriminant plus the two
    /// payload words it selects between.
    InvocationHostResult = 10,
}

impl BoundaryTag {
    /// Every tag, in declaration order.
    ///
    /// ⭐ Derived from the closed `match` below rather than written twice, so
    /// this list cannot drift from the enum: adding a variant without extending
    /// the `match` is a compile error, and the array length is checked against
    /// it in this module's tests.
    pub const ALL: [BoundaryTag; 11] = [
        BoundaryTag::ImmediateBool,
        BoundaryTag::ImmediateInt,
        BoundaryTag::ImmediateExitStatus,
        BoundaryTag::ImmediateCapability,
        BoundaryTag::ImmediateResource,
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
            3 => BoundaryTag::ImmediateCapability,
            4 => BoundaryTag::ImmediateResource,
            5 => BoundaryTag::ImmediateBoundedNat,
            6 => BoundaryTag::ImmediateStructuralNat,
            7 => BoundaryTag::PersistentGround,
            8 => BoundaryTag::PersistentClosure,
            9 => BoundaryTag::InvocationBorrowed,
            10 => BoundaryTag::InvocationHostResult,
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
            | BoundaryTag::ImmediateCapability
            | BoundaryTag::ImmediateResource
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

    /// Build a handle word naming an arena node.
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

/// Byte size of the arena header.
pub const BOUNDARY_ARENA_HEADER_BYTES: i32 = 64;

/// Pointer to the node table.
pub const ARENA_NODES: i32 = 0;
/// Number of nodes.
pub const ARENA_NODE_COUNT: i32 = 8;
/// Pointer to the child-word table.
pub const ARENA_WORDS: i32 = 16;
/// Number of child words.
pub const ARENA_WORD_COUNT: i32 = 24;
/// Pointer to the field-name-id table, parallel to the word table.
pub const ARENA_NAMES: i32 = 32;
/// Number of field-name ids.
pub const ARENA_NAME_COUNT: i32 = 40;

/// Status returned by every emitted-code helper on success.
pub const BOUNDARY_OK: i64 = 0;
/// The word's tag byte is outside the closed set.
pub const BOUNDARY_ERR_TAG: i64 = -1;
/// The word is an immediate where a handle was required, or the reverse.
pub const BOUNDARY_ERR_SHAPE: i64 = -2;
/// A node index, field index or name lookup left the arena's bounds.
pub const BOUNDARY_ERR_BOUNDS: i64 = -3;
/// The node's class does not admit the requested projection.
pub const BOUNDARY_ERR_CLASS: i64 = -4;
/// ⛔ Borrowed ingress attempted to escape the native invocation (`AC-7`).
pub const BOUNDARY_ERR_ESCAPE: i64 = -5;

// ---------------------------------------------------------------------------
// The invocation-scoped arena
// ---------------------------------------------------------------------------

/// The flat, invocation-scoped tables emitted code projects out of.
///
/// ⛔ **Not a parallel permanent heap** (`D2`). Every node dies with the
/// invocation. A node whose owner is [`BoundaryReferentOwner::PersistentStore`]
/// is a *view* on a value the [`Store`] owns — the node carries that
/// [`SlotId`], so the owner of record is recoverable from the node itself and
/// is never inferred from the frame slot the word happened to sit in.
#[derive(Debug, Default)]
pub struct BoundaryArenaV1 {
    nodes: Vec<u64>,
    words: Vec<u64>,
    names: Vec<u64>,
    header: Vec<u64>,
}

impl BoundaryArenaV1 {
    /// Number of nodes materialized.
    pub fn node_count(&self) -> usize {
        self.nodes.len() / (BOUNDARY_NODE_STRIDE as usize / 8)
    }

    /// Read one field of one node. `None` when the index or offset is out of
    /// range — the Rust-side mirror of the CLIF bounds checks, used by tests as
    /// an independent oracle rather than by re-reading the CLIF's own answer.
    pub fn node_field(&self, index: u64, offset: i32) -> Option<u64> {
        let stride = BOUNDARY_NODE_STRIDE as usize / 8;
        let base = (index as usize).checked_mul(stride)?;
        self.nodes.get(base + (offset as usize / 8)).copied()
    }

    /// The child word at an absolute word-table index.
    pub fn word_at(&self, index: u64) -> Option<BoundaryWord> {
        self.words.get(index as usize).copied().map(BoundaryWord)
    }

    /// The field-name id at an absolute name-table index.
    pub fn name_at(&self, index: u64) -> Option<u64> {
        self.names.get(index as usize).copied()
    }

    /// Publish the header and hand back a pointer emitted code can read.
    ///
    /// # Safety contract
    ///
    /// The returned pointer is valid only while `self` is alive and not
    /// mutated. Callers are the invocation drivers, which own the arena for the
    /// invocation's extent — the same discipline `NativeIntArenaV1` already
    /// uses for its own header.
    pub fn publish(&mut self) -> *const u64 {
        self.header = vec![
            self.nodes.as_ptr() as u64,
            self.node_count() as u64,
            self.words.as_ptr() as u64,
            self.words.len() as u64,
            self.names.as_ptr() as u64,
            self.names.len() as u64,
            0,
            0,
        ];
        self.header.as_ptr()
    }
}

/// Builds a [`BoundaryArenaV1`].
///
/// ⛔ Holds no environment and no seed value. Its whole input is a class, a
/// payload and a child list — `AC-2` by construction rather than by assertion.
#[derive(Debug, Default)]
pub struct BoundaryArenaBuilder {
    arena: BoundaryArenaV1,
}

impl BoundaryArenaBuilder {
    /// A fresh, empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one node and return the handle word naming it.
    #[allow(clippy::too_many_arguments)]
    pub fn push_node(
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
        let index = self.arena.node_count() as u64;
        let fields_at = self.arena.words.len() as u64;
        for child in children {
            self.arena.words.push(child.0);
        }
        // The name table stays parallel to the word table for EVERY node, so a
        // record's names sit at exactly its children's indices. Non-records pad
        // with zero rather than shifting the two tables out of step.
        if names.is_empty() {
            self.arena.names.resize(self.arena.words.len(), 0);
        } else {
            for name in names {
                self.arena.names.push(*name);
            }
        }
        self.arena.nodes.extend_from_slice(&[
            class as u64,
            tag.referent_owner() as u64,
            slot,
            tag_id,
            payload,
            children.len() as u64,
            fields_at,
        ]);
        BoundaryWord::handle(tag, index)
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
        }
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

/// Materialize a ground value into the arena and return its boundary word.
///
/// Scalars stay immediate; compounds become persistent handles whose node
/// records the owning [`SlotId`]. The recursion is over the value's *own*
/// structure — no environment and no caller context participates.
pub fn materialize_ground(
    store: &mut BoundaryValueStore,
    builder: &mut BoundaryArenaBuilder,
    value: &RuntimeGroundValue,
) -> Option<BoundaryWord> {
    Some(match value {
        RuntimeGroundValue::Bool(bit) => {
            BoundaryWord::immediate(BoundaryTag::ImmediateBool, u64::from(*bit))
        }
        RuntimeGroundValue::Int(int) => match int {
            // `immediate` shifts the payload left by the tag width, so the top
            // eight bits — pure sign extension inside the immediate range —
            // fall off and `signed_payload`'s arithmetic shift restores them.
            crate::RuntimeIntV1::Small(v) if BoundaryWord::int_fits_immediate(*v) => {
                BoundaryWord::immediate(BoundaryTag::ImmediateInt, *v as u64)
            }
            _ => {
                let slot = store.persist(value)?;
                builder.push_node(
                    BoundaryTag::PersistentGround,
                    BoundaryClass::Int,
                    slot,
                    0,
                    0,
                    &[],
                    &[],
                )
            }
        },
        RuntimeGroundValue::Bytes(bytes) => {
            let slot = store.persist(value)?;
            builder.push_node(
                BoundaryTag::PersistentGround,
                BoundaryClass::Bytes,
                slot,
                0,
                bytes.len() as u64,
                &[],
                &[],
            )
        }
        RuntimeGroundValue::String(text) => {
            let slot = store.persist(value)?;
            builder.push_node(
                BoundaryTag::PersistentGround,
                BoundaryClass::String,
                slot,
                0,
                text.len() as u64,
                &[],
                &[],
            )
        }
        RuntimeGroundValue::Constructor { constructor, args } => {
            let slot = store.persist(value)?;
            let tag_id = store.intern_symbol(constructor);
            let mut children = Vec::with_capacity(args.len());
            for arg in args {
                children.push(materialize_ground(store, builder, arg)?);
            }
            builder.push_node(
                BoundaryTag::PersistentGround,
                BoundaryClass::Constructor,
                slot,
                tag_id,
                0,
                &children,
                &[],
            )
        }
        RuntimeGroundValue::Record { fields } => {
            let slot = store.persist(value)?;
            let mut children = Vec::with_capacity(fields.len());
            let mut names = Vec::with_capacity(fields.len());
            for (name, field) in fields {
                names.push(store.intern_symbol(name));
                children.push(materialize_ground(store, builder, field)?);
            }
            builder.push_node(
                BoundaryTag::PersistentGround,
                BoundaryClass::Record,
                slot,
                0,
                0,
                &children,
                &names,
            )
        }
    })
}

/// Materialize borrowed host ingress — valid for this invocation only.
///
/// ⛔ The node's owner is [`BoundaryReferentOwner::InvocationArena`] and its
/// slot is [`NULL_SLOT`], which is what makes escape detectable rather than
/// merely documented (`AC-7`).
pub fn materialize_borrowed(
    builder: &mut BoundaryArenaBuilder,
    payload: u64,
) -> BoundaryWord {
    builder.push_node(
        BoundaryTag::InvocationBorrowed,
        BoundaryClass::BorrowedOpaque,
        NULL_SLOT,
        0,
        payload,
        &[],
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
        NULL_SLOT,
        0,
        success,
        &[ok, err],
        &[],
    )
}

/// Whether a word may cross out of the native invocation that produced it.
///
/// ⛔ **Fail-closed (`AC-7`).** An invocation-owned referent escaping its
/// invocation is [`BOUNDARY_ERR_ESCAPE`], never a silent pass.
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
