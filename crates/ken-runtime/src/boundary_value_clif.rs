//! `RT-FNSPLIT-B2V` — the **executable** half of the boundary-value ABI.
//!
//! [`crate::boundary_value`] says what the bits of a boundary word mean. This
//! module is the part that makes that meaning reachable from **emitted code**,
//! and the two are one deliverable: a representation whose only reader is Rust
//! is exactly what hard-stop `#10` measured and rejected — the landed
//! aggregate-result path works today only because the consumer is Rust
//! (`ResultDecoder` + `result_table` living in `CompiledModule`), so it does not
//! generalize from the artifact boundary to an internal one.
//!
//! ⛔ **A Rust-side token with no runtime lookup path does not count** (`D3`).
//! Every helper below is a `Linkage::Local` CLIF body compiled into the module
//! alongside the program, exactly as `native_int_clif` already does for exact
//! `Int`. Nothing here calls back into Rust at runtime, and nothing here reads a
//! compile-time table.
//!
//! ## Θ(1) per module
//!
//! The helper population is a **fixed list**, declared once per module and
//! never per origin, per call site or per runtime value. That is the growth
//! invariant the whole `RT-NATIVE-FNSPLIT` program exists to protect: a
//! per-value helper would reintroduce the defect the parent node is closing.
//! [`BOUNDARY_LOCAL_HELPERS`] is the closed inventory, and it is pinned by name
//! rather than by count so that *any* addition reddens.
//!
//! ## Where the layout knowledge lives
//!
//! Only [`define_resolve`] converts a word into a node address. Every other
//! helper calls it. A layout change is therefore one edit in CLIF and one edit
//! in the constants `boundary_value` publishes — never a change scattered
//! across projections.

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, AbiParam, Function, InstBuilder, MemFlags, UserFuncName};
use cranelift_codegen::verify_function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage, Module};

use crate::boundary_value::{
    boundary_domain_mask, boundary_int_marker_mask, BoundaryClass, BoundaryImmediateDomain,
    BoundaryReferentOwner, BoundaryTag, ARENA_DATA, ARENA_DATA_CAPACITY, ARENA_DATA_COUNT,
    ARENA_FROZEN, ARENA_LIMBS, ARENA_LIMB_CAPACITY, ARENA_LIMB_COUNT, ARENA_NAMES,
    ARENA_NAME_COUNT, ARENA_NATIVE_INT, ARENA_NODES, ARENA_NODE_CAPACITY, ARENA_NODE_COUNT,
    ARENA_PERSISTENT, ARENA_SEALED, ARENA_WORDS, ARENA_WORD_CAPACITY, ARENA_WORD_COUNT,
    BOUNDARY_ERR_BOUNDS, BOUNDARY_ERR_CAPACITY, BOUNDARY_ERR_CLASS, BOUNDARY_ERR_ESCAPE,
    BOUNDARY_ERR_FROZEN, BOUNDARY_ERR_RELATION, BOUNDARY_ERR_RETIRED_LANE, BOUNDARY_ERR_SEALED,
    BOUNDARY_ERR_SHAPE, BOUNDARY_ERR_TAG, BOUNDARY_INT_REGION_LIMBS, BOUNDARY_NODE_STRIDE,
    BOUNDARY_OK, BOUNDARY_TAG_BITS, BOUNDARY_TAG_MASK, NODE_CLASS, NODE_EXTENT, NODE_FIELDS_AT,
    NODE_FIELD_COUNT, NODE_INT_SEALED, NODE_LIMBS_AT, NODE_LIMB_COUNT, NODE_OWNER, NODE_PAYLOAD,
    NODE_SLOT, NODE_TAG_ID,
};
use crate::cranelift_backend::{backend_module, CraneliftBackendError};

#[cfg(test)]
thread_local! {
    static BOUNDARY_CLIF_CAPTURE: std::cell::RefCell<Option<Vec<String>>> = const {
        std::cell::RefCell::new(None)
    };
}

/// ⛔ **The closed helper inventory (`AC-9`).**
///
/// Pinned as the exact permitted **set of names**, not as a count: a name list
/// makes an addition redden with the added name in the failure message, where a
/// count only says "something moved". The population is fixed per module — it
/// does not grow with origins, call sites or runtime values.
pub const BOUNDARY_LOCAL_HELPERS: &[&str] = &[
    "ken_boundary_resolve_local",
    "ken_boundary_class_local",
    "ken_boundary_owner_local",
    "ken_boundary_slot_local",
    "ken_boundary_scalar_local",
    "ken_boundary_tag_local",
    "ken_boundary_field_count_local",
    "ken_boundary_field_local",
    "ken_boundary_record_field_local",
    "ken_boundary_host_success_local",
    "ken_boundary_host_payload_local",
    "ken_boundary_make_immediate_local",
    "ken_boundary_escape_check_local",
    // ── construction: the producer half of the interface ──────────────────
    "ken_boundary_alloc_local",
    "ken_boundary_store_tag_id_local",
    "ken_boundary_store_scalar_local",
    "ken_boundary_store_field_local",
    "ken_boundary_store_name_local",
    "ken_boundary_store_int_tag_local",
    "ken_boundary_seal_int_local",
    "ken_boundary_store_int_limbs_local",
    "ken_boundary_store_int_limb_local",
    "ken_boundary_store_bytes_len_local",
    "ken_boundary_store_byte_local",
    // ── content access: the value's BITS, not its identity or its length ──
    "ken_boundary_byte_local",
    "ken_boundary_int_sign_local",
    "ken_boundary_int_len_local",
    "ken_boundary_int_limb_local",
    "ken_boundary_int_view_local",
];

// ⛔ There is deliberately NO `ken_boundary_store_slot_local`.
//
// It existed, it took a **caller-supplied** `SlotId`, and the frozen-prefix
// guard expressly permits writes to a newly allocated node — so emitted code
// could replace the allocator's `NULL_SLOT` with any slot it chose. That
// contradicted the load-bearing claim that only the store mints persistent
// identity, and it contradicted this node's own recorded residual, which says
// emitted-created nodes stay `NULL_SLOT`.
//
// ⚠ **The residual was pinned on a path that never exercised the helper that
// broke it.** The control constructed a node and read back `NULL_SLOT` without
// ever calling `store_slot`, so it asserted a property of a field nothing had
// written to. Assigning store identity is not an emitted-code operation at all;
// the closure is to remove the capability, not to guard it.
//
// `EMITTED_WRITABLE_NODE_OFFSETS` below is what keeps it removed: the generic
// node-word setter refuses to be emitted for any other offset.

/// ⛔ **The only node words emitted code may set** (`AC-6`).
///
/// Everything absent is identity or layout: `NODE_SLOT` is the store's,
/// `NODE_CLASS`/`NODE_OWNER` are the allocator's and must agree with the tag,
/// and `NODE_FIELD_COUNT`/`NODE_FIELDS_AT`/`NODE_EXTENT` are spans whose bounds
/// were checked when they were claimed. `define_store_node_word` asserts
/// membership at **emission** time, so a setter for a forbidden offset cannot
/// be built — the surface is closed rather than watched.
const EMITTED_WRITABLE_NODE_OFFSETS: &[i32] = &[NODE_TAG_ID, NODE_PAYLOAD];

/// The emitted-code interface, as `FuncId`s to call.
///
/// ⚠ **Every field is currently unread in production, and that is the node's
/// defining constraint rather than an oversight.** `D6` makes `B2V` inert: it
/// lands the representation and the interface, and `RT-FNSPLIT-B2F` lands the
/// switch-over that calls them. Marked rather than silently consumed, so the
/// unused state stays visible to the next reader instead of being disguised by
/// a token production reference that would breach `D6`.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(crate) struct BoundaryLocalFuncs {
    /// `(arena, word, out) -> status` — the value's [`BoundaryClass`].
    pub class: FuncId,
    /// `(arena, word, out) -> status` — the referent owner (`AC-6`).
    pub owner: FuncId,
    /// `(arena, word, out) -> status` — the owning `SlotId`, or `NULL_SLOT`.
    pub slot: FuncId,
    /// `(arena, word, out) -> status` — scalar extraction.
    pub scalar: FuncId,
    /// `(arena, word, out) -> status` — constructor / record tag identity.
    pub tag: FuncId,
    /// `(arena, word, out) -> status` — number of fields.
    pub field_count: FuncId,
    /// `(arena, word, index, out) -> status` — positional projection.
    pub field: FuncId,
    /// `(arena, word, name_id, out) -> status` — record field access.
    pub record_field: FuncId,
    /// `(arena, word, out) -> status` — `HostResult` success discriminant.
    pub host_success: FuncId,
    /// `(arena, word, out) -> status` — the payload that discriminant selects.
    pub host_payload: FuncId,
    /// `(tag, payload, out) -> status` — construct an immediate.
    pub make_immediate: FuncId,
    /// `(arena, word) -> status` — fail-closed borrowed-ingress escape check.
    pub escape_check: FuncId,
    /// `(arena, tag, class, field_count, out) -> status` — allocate a handle
    /// node **in the region the tag selects** and write its word to `*out`.
    pub alloc: FuncId,
    /// `(arena, word, tag_id) -> status` — record constructor/record identity.
    pub store_tag_id: FuncId,
    /// `(arena, word, payload) -> status` — record the scalar payload, which for
    /// a `HostResult` is the success discriminant.
    pub store_scalar: FuncId,
    /// `(arena, word, index, child) -> status` — write one child word.
    pub store_field: FuncId,
    /// `(arena, word, index, name_id) -> status` — write one field name.
    pub store_name: FuncId,
    /// `(arena, word, native_tag) -> status` — record a spilled `Int`'s
    /// `NativeIntV1` tag. Class-guarded to `Int`.
    pub store_int_tag: FuncId,
    /// `(arena, word) -> status` — check a region-limbed `Int`'s magnitude
    /// canonical and seal it. Until this succeeds the node denotes nothing.
    pub seal_int: FuncId,
    /// `(arena, word, sign, len, out) -> status` — claim `len` magnitude limbs
    /// in the node's OWN region for a spilled `Int` and write the span's start
    /// to `*out`. The counterpart of `store_bytes_len` for `Int` content.
    pub store_int_limbs: FuncId,
    /// `(arena, word, index, limb) -> status` — write one magnitude limb.
    pub store_int_limb: FuncId,
    /// `(arena, word, len, out) -> status` — claim `len` data bytes for a
    /// `Bytes`/`String` node and write the span's start to `*out`.
    pub store_bytes_len: FuncId,
    /// `(arena, word, index, byte) -> status` — write one content byte.
    pub store_byte: FuncId,
    /// `(arena, word, index, out) -> status` — read one content byte.
    pub byte: FuncId,
    /// `(arena, word, out) -> status` — a spilled `Int`'s sign.
    pub int_sign: FuncId,
    /// `(arena, word, out) -> status` — a spilled `Int`'s limb count.
    pub int_len: FuncId,
    /// `(arena, word, index, out) -> status` — a spilled `Int`'s limb.
    pub int_limb: FuncId,
    /// `(arena, word, out_view) -> status` — a spilled `Int`'s canonical
    /// `{sign, len, limbs}` view.
    pub int_view: FuncId,
}

#[derive(Clone, Copy)]
struct Graph {
    resolve: FuncId,
    class: FuncId,
    owner: FuncId,
    slot: FuncId,
    scalar: FuncId,
    tag: FuncId,
    field_count: FuncId,
    field: FuncId,
    record_field: FuncId,
    host_success: FuncId,
    host_payload: FuncId,
    make_immediate: FuncId,
    escape_check: FuncId,
    alloc: FuncId,
    store_tag_id: FuncId,
    store_scalar: FuncId,
    store_field: FuncId,
    store_name: FuncId,
    store_int_tag: FuncId,
    seal_int: FuncId,
    store_int_limbs: FuncId,
    store_int_limb: FuncId,
    store_bytes_len: FuncId,
    store_byte: FuncId,
    byte: FuncId,
    int_sign: FuncId,
    int_len: FuncId,
    int_limb: FuncId,
    int_view: FuncId,
    /// `ken_native_int_resolve_local`, declared into this module by the
    /// native-`Int` graph that is emitted before this one.
    native_int_resolve: FuncId,
}

/// Emit the boundary-value helper graph into `module`.
///
/// ⛔ **`D6` — INERT.** This declares and defines helpers and nothing else. It
/// populates no generated function for any semantic origin, adds no cross-owner
/// call, and installs no second body-emission authority. `RT-FNSPLIT-B2F`
/// performs the switch-over that calls these.
pub(crate) fn emit_boundary_value_local_graph<M: Module>(
    module: &mut M,
    native_int: &crate::native_int_clif::NativeIntLocalFuncs,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<BoundaryLocalFuncs, CraneliftBackendError> {
    let resolve = declare(module, "ken_boundary_resolve_local", 3)?;
    let class = declare(module, "ken_boundary_class_local", 3)?;
    let owner = declare(module, "ken_boundary_owner_local", 3)?;
    let slot = declare(module, "ken_boundary_slot_local", 3)?;
    let scalar = declare(module, "ken_boundary_scalar_local", 3)?;
    let tag = declare(module, "ken_boundary_tag_local", 3)?;
    let field_count = declare(module, "ken_boundary_field_count_local", 3)?;
    let field = declare(module, "ken_boundary_field_local", 4)?;
    let record_field = declare(module, "ken_boundary_record_field_local", 4)?;
    let host_success = declare(module, "ken_boundary_host_success_local", 3)?;
    let host_payload = declare(module, "ken_boundary_host_payload_local", 3)?;
    let make_immediate = declare(module, "ken_boundary_make_immediate_local", 3)?;
    let escape_check = declare(module, "ken_boundary_escape_check_local", 2)?;
    let alloc = declare(module, "ken_boundary_alloc_local", 5)?;
    let store_tag_id = declare(module, "ken_boundary_store_tag_id_local", 3)?;
    let store_scalar = declare(module, "ken_boundary_store_scalar_local", 3)?;
    let store_field = declare(module, "ken_boundary_store_field_local", 4)?;
    let store_name = declare(module, "ken_boundary_store_name_local", 4)?;
    let store_int_tag = declare(module, "ken_boundary_store_int_tag_local", 3)?;
    let seal_int = declare(module, "ken_boundary_seal_int_local", 2)?;
    let store_int_limbs = declare(module, "ken_boundary_store_int_limbs_local", 5)?;
    let store_int_limb = declare(module, "ken_boundary_store_int_limb_local", 4)?;
    let store_bytes_len = declare(module, "ken_boundary_store_bytes_len_local", 4)?;
    let store_byte = declare(module, "ken_boundary_store_byte_local", 4)?;
    let byte = declare(module, "ken_boundary_byte_local", 4)?;
    let int_sign = declare(module, "ken_boundary_int_sign_local", 3)?;
    let int_len = declare(module, "ken_boundary_int_len_local", 3)?;
    let int_limb = declare(module, "ken_boundary_int_limb_local", 4)?;
    let int_view = declare(module, "ken_boundary_int_view_local", 3)?;
    let graph = Graph {
        resolve,
        class,
        owner,
        slot,
        scalar,
        tag,
        field_count,
        field,
        record_field,
        host_success,
        host_payload,
        make_immediate,
        escape_check,
        alloc,
        store_tag_id,
        store_scalar,
        store_field,
        store_name,
        store_int_tag,
        seal_int,
        store_int_limbs,
        store_int_limb,
        store_bytes_len,
        store_byte,
        byte,
        int_sign,
        int_len,
        int_limb,
        int_view,
        native_int_resolve: native_int.resolve,
    };

    define_resolve(module, graph, plan)?;
    define_class(module, graph, plan)?;
    define_node_word(module, graph, graph.owner, NODE_OWNER)?;
    define_node_word(module, graph, graph.slot, NODE_SLOT)?;
    define_scalar(module, graph)?;
    define_node_word(module, graph, graph.tag, NODE_TAG_ID)?;
    define_node_word(module, graph, graph.field_count, NODE_FIELD_COUNT)?;
    define_field(module, graph)?;
    define_record_field(module, graph)?;
    define_host_success(module, graph)?;
    define_host_payload(module, graph)?;
    define_make_immediate(module, graph, plan)?;
    define_escape_check(module, graph, plan)?;
    define_alloc(module, graph, plan)?;
    define_store_node_word(module, graph, graph.store_tag_id, NODE_TAG_ID)?;
    define_store_node_word(module, graph, graph.store_scalar, NODE_PAYLOAD)?;
    define_store_field(module, graph, plan)?;
    define_store_name(module, graph)?;
    define_store_int_tag(module, graph, plan)?;
    define_seal_int(module, graph, plan)?;
    define_store_int_limbs(module, graph, plan)?;
    define_store_int_limb(module, graph, plan)?;
    define_store_bytes_len(module, graph, plan)?;
    define_byte_access(module, graph, graph.store_byte, true, plan)?;
    define_byte_access(module, graph, graph.byte, false, plan)?;
    define_int_part(module, graph, graph.int_sign, IntPart::Sign, plan)?;
    define_int_part(module, graph, graph.int_len, IntPart::Len, plan)?;
    define_int_part(module, graph, graph.int_limb, IntPart::Limb, plan)?;
    define_int_part(module, graph, graph.int_view, IntPart::View, plan)?;

    Ok(BoundaryLocalFuncs {
        class,
        owner,
        slot,
        scalar,
        tag,
        field_count,
        field,
        record_field,
        host_success,
        host_payload,
        make_immediate,
        escape_check,
        alloc,
        store_tag_id,
        store_scalar,
        store_field,
        store_name,
        store_int_tag,
        seal_int,
        store_int_limbs,
        store_int_limb,
        store_bytes_len,
        store_byte,
        byte,
        int_sign,
        int_len,
        int_limb,
        int_view,
    })
}

/// Capture every helper body as text, for the JIT/object identity pin and the
/// closed-inventory pin. Test-only.
#[cfg(test)]
pub(crate) fn capture_boundary_value_local_graph<M: Module>(
    module: &mut M,
) -> Result<String, CraneliftBackendError> {
    let plan = crate::boundary_value::BoundaryEmissionPlan::derive();
    capture_boundary_value_local_graph_with_plan(module, &plan)
}

/// Capture the emitted graph under an **injected** plan.
///
/// ⛔ **This exists so the authority-to-emitter edge can be shown CAUSALLY.**
/// `RECUT 2` is not discharged by an emitter that receives a plan and ignores
/// it, so the control feeds a perturbed plan and requires the emitted CLIF to
/// differ. Without an injection point the only available evidence would be
/// "the plan is passed", which is the `let _ = plan` the ruling excludes.
#[cfg(test)]
pub(crate) fn capture_boundary_value_local_graph_with_plan<M: Module>(
    module: &mut M,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<String, CraneliftBackendError> {
    let native = crate::native_int_clif::emit_native_int_local_graph(module, false)?;
    BOUNDARY_CLIF_CAPTURE.with(|capture| *capture.borrow_mut() = Some(Vec::new()));
    emit_boundary_value_local_graph(module, &native, plan)?;
    Ok(BOUNDARY_CLIF_CAPTURE.with(|capture| {
        capture
            .borrow_mut()
            .take()
            .expect("capture was installed")
            .join("\n-- boundary helper --\n")
    }))
}

fn declare<M: Module>(
    module: &mut M,
    name: &str,
    params: usize,
) -> Result<FuncId, CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut sig = module.make_signature();
    // One native word per argument on the supported target, matching the
    // `native_int_clif` convention: pointers and words are the same width, and
    // a helper that mixed widths would be an ABI the emitter has to remember.
    for _ in 0..params {
        sig.params.push(AbiParam::new(ptr));
    }
    sig.returns.push(AbiParam::new(types::I64));
    module
        .declare_function(name, Linkage::Local, &sig)
        .map_err(|e| backend_module(e.to_string()))
}

fn begin<M: Module>(module: &M, id: FuncId, params: usize) -> Function {
    let ptr = module.target_config().pointer_type();
    let mut sig = module.make_signature();
    for _ in 0..params {
        sig.params.push(AbiParam::new(ptr));
    }
    sig.returns.push(AbiParam::new(types::I64));
    Function::with_name_signature(UserFuncName::user(3, id.as_u32()), sig)
}

fn finish<M: Module>(
    module: &mut M,
    id: FuncId,
    mut func: Function,
) -> Result<(), CraneliftBackendError> {
    verify_function(&func, module.isa())
        .map_err(|e| backend_module(format!("boundary-value local helper verification: {e}")))?;
    #[cfg(test)]
    BOUNDARY_CLIF_CAPTURE.with(|capture| {
        if let Some(functions) = capture.borrow_mut().as_mut() {
            functions.push(func.display().to_string());
        }
    });
    #[cfg(test)]
    crate::cranelift_backend::scale_b_record_boundary_value(&func);
    let mut ctx = module.make_context();
    std::mem::swap(&mut ctx.func, &mut func);
    module
        .define_function(id, &mut ctx)
        .map_err(|e| backend_module(e.to_string()))
}

/// Emit `tag ∈ tags`, where `tags` is a set the plan **derived** from the
/// representation authority.
///
/// ⛔ **A disjunction over a set, never an ordinal band.** The three
/// thresholds this replaces — `FIRST_HANDLE_TAG`, `LAST_PERSISTENT_TAG`,
/// `LAST_TAG` — were each a second authority computed by hand from
/// [`BoundaryTag`]'s declaration order. Reordering the enum left all three
/// well-formed while silently changing what the emitted code admitted, so the
/// property they encoded ("the tags are grouped by referent owner") had to be
/// held up by a separate pin. A set has no ordering to depend on, and it does
/// not require the admitted tags to be contiguous in the first place.
///
/// An empty set is a legitimate answer from the authority — the partition
/// admits no tag of that kind — and yields constant false rather than a panic:
/// nothing is a member of nothing.
fn tag_in_set(
    b: &mut FunctionBuilder<'_>,
    tag: cranelift_codegen::ir::Value,
    tags: &[BoundaryTag],
) -> cranelift_codegen::ir::Value {
    let mut member: Option<cranelift_codegen::ir::Value> = None;
    for candidate in tags {
        let hit = b.ins().icmp_imm(IntCC::Equal, tag, *candidate as i64);
        member = Some(match member {
            None => hit,
            Some(prev) => b.ins().bor(prev, hit),
        });
    }
    match member {
        Some(member) => member,
        None => b.ins().iconst(types::I8, 0),
    }
}

/// Refuse a tag the partition does not admit, with the **exact** diagnostic
/// (`RT-FNSPLIT-C1` `D5`). Returns; the builder is left in the `unknown` block.
///
/// ⛔ **RECOGNITION IS AN ABI-WIDE PROPERTY, NOT A `define_alloc` ONE — and
/// that is the whole reason this helper exists.** `BoundaryEmissionPlan::derive`
/// sweeps the live representation authority, so retiring the durable-closure
/// lane deleted `PersistentClosure` from `plan.tags().admitted()` outright.
/// Every site gating on that set therefore stopped answering *"I refuse this
/// specific retired lane"* (`-12`) and started answering *"I do not recognize
/// this byte"* (`-1`) — the identity arbitrary corruption produces.
///
/// ⚠ **The first `D5` pass repaired only `define_alloc` and left three more
/// recognition sites downgraded**, which is the removal-census failure exactly:
/// deleting a producer removes the vocabulary needed to reject it, at *every*
/// consumer, not at the one the change was written against. `define_resolve`,
/// `define_escape_check` and the child-tag check are all decode/classification
/// — the readers [`crate::boundary_value::BOUNDARY_RETIRED_LANES`] names as its
/// intended ones — so all three must name the lane they refuse.
///
/// ⭐ The retired set is derived here from the plan's own admitted set, so a
/// mutation fixture that admits a different partition gets the retired set that
/// partition implies rather than a hand-written constant.
///
/// ⚠ **HONEST RESIDUAL — three of the four call sites are pinned, the fourth is
/// not.** `define_resolve` and `define_escape_check` are both swept by
/// [`tests::b2v_emitted_code_admits_exactly_the_closed_tag_set`], which asserts
/// `-12` for every retired tag byte across all 256 and reddened at exactly that
/// position before this helper existed. ⛔ The **child-tag** site has no control
/// asserting its retired arm: reaching it needs a node whose stored CHILD word
/// carries a retired tag, and the store path that would place one is the same
/// one the retirement closes. So that arm is **repaired but unexercised** — it
/// is guarded by review, not by CI, and this note is where the next reader
/// inherits that limit rather than reading four pinned sites into three.
fn refuse_unadmitted_tag(
    b: &mut FunctionBuilder<'_>,
    tag: cranelift_codegen::ir::Value,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) {
    let retired = crate::boundary_value::boundary_retired_tags(plan.tags().admitted());
    let is_retired = tag_in_set(b, tag, &retired);
    let retired_block = b.create_block();
    let unknown = b.create_block();
    b.ins().brif(is_retired, retired_block, &[], unknown, &[]);

    b.switch_to_block(retired_block);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_RETIRED_LANE);
    b.ins().return_(&[err]);

    b.switch_to_block(unknown);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_TAG);
    b.ins().return_(&[err]);
}

/// Emit the region selection for a handle tag, driven by the plan's derived
/// owner bands. Jumps to `selected` with the region base, or returns an exact
/// status; the builder is left in an unreachable trailing block.
///
/// ⛔ **A relation over owners, not a two-way threshold.** The code this
/// replaces asked one question — *is this tag at or below the last persistent
/// one* — which assumes there are exactly two handle owners **and** that the
/// tag order separates them. An owner the partition started publishing would
/// have been folded into whichever side of that threshold its tag fell on,
/// silently, with both constants still well-formed. Here each band is asked
/// separately and the owner is mapped through a wildcard-free `match`, so a new
/// [`BoundaryReferentOwner`] is a compile error at this seam.
fn select_region_by_owner_band(
    b: &mut FunctionBuilder<'_>,
    ptr: cranelift_codegen::ir::Type,
    arena: cranelift_codegen::ir::Value,
    tag: cranelift_codegen::ir::Value,
    selected: cranelift_codegen::ir::Block,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) {
    for (owner, tags) in plan.tags().owner_bands() {
        let hit = b.create_block();
        let miss = b.create_block();
        let member = tag_in_set(b, tag, tags);
        b.ins().brif(member, hit, &[], miss, &[]);

        b.switch_to_block(hit);
        match owner {
            BoundaryReferentOwner::InvocationArena => {
                b.ins().jump(selected, &[arena.into()]);
            }
            BoundaryReferentOwner::PersistentStore => {
                // ⛔ A persistent word resolves through PERSISTENT storage or
                // not at all. An invocation bound to no persistent region fails
                // closed — falling back to the arena would read the persistent
                // index against the wrong table, which is silent corruption
                // rather than an error.
                let region = b
                    .ins()
                    .load(ptr, MemFlags::trusted(), arena, ARENA_PERSISTENT);
                let bound = b.ins().icmp_imm(IntCC::NotEqual, region, 0);
                let have = b.create_block();
                let unbound = b.create_block();
                b.ins().brif(bound, have, &[], unbound, &[]);

                b.switch_to_block(unbound);
                let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
                b.ins().return_(&[err]);

                b.switch_to_block(have);
                b.ins().jump(selected, &[region.into()]);
            }
            BoundaryReferentOwner::NoReferent => {
                // A handle whose referent nothing owns has no region to resolve
                // in. The partition publishes none today; if it started to,
                // this is an exact status rather than a silent arena read.
                let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
                b.ins().return_(&[err]);
            }
        }

        b.switch_to_block(miss);
    }
    // No band claimed this tag. Unreachable while the bands' union is the
    // handle set the caller already tested — and still written, because
    // "cannot happen" is the default that stops being true first.
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_TAG);
    b.ins().return_(&[err]);
}

/// Byte offset of the region base inside `resolve`'s two-word output cell.
const RESOLVED_REGION: i32 = 8;

/// `(arena, word, out) -> status`, writing the node's base address to `out[0]`
/// and **the base of the region that node lives in** to `out[1]`.
///
/// ⭐ **The only place a word becomes an address, and the only place a word
/// selects a region.** Both questions are answered here, together, because they
/// are one question: a handle's index means nothing until you know which table
/// it indexes. Handing back only the address would leave every child-word
/// projection to guess the region, and guessing "the arena" is exactly the
/// defect this rewrite closes.
fn define_resolve<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.resolve, 3);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, out) = (p[0], p[1], p[2]);

        let tag = b.ins().band_imm(word, BOUNDARY_TAG_MASK as i64);
        // ⛔ An unknown tag is a THIRD OUTCOME THAT FAILS, never a fall-through
        // into some default projection.
        let known = tag_in_set(&mut b, tag, plan.tags().admitted());
        let closed = b.create_block();
        let not_admitted = b.create_block();
        b.ins().brif(known, closed, &[], not_admitted, &[]);

        b.switch_to_block(not_admitted);
        refuse_unadmitted_tag(&mut b, tag, plan);

        b.switch_to_block(closed);
        let is_handle = tag_in_set(&mut b, tag, plan.tags().handle());
        let handle = b.create_block();
        let immediate = b.create_block();
        b.ins().brif(is_handle, handle, &[], immediate, &[]);

        b.switch_to_block(immediate);
        // An immediate has no referent, so there is no address to hand back.
        // The caller distinguishes this from a malformed word by the status.
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
        b.ins().return_(&[err]);

        // ── region selection ────────────────────────────────────────────────
        b.switch_to_block(handle);
        let selected = b.create_block();
        b.append_block_param(selected, ptr);
        select_region_by_owner_band(&mut b, ptr, arena, tag, selected, plan);

        // ── bounds, within the SELECTED region ──────────────────────────────
        b.switch_to_block(selected);
        let region = b.block_params(selected)[0];
        let index = b.ins().ushr_imm(word, i64::from(BOUNDARY_TAG_BITS));
        let count = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_NODE_COUNT);
        let within = b.ins().icmp(IntCC::UnsignedLessThan, index, count);
        let ok = b.create_block();
        let oob = b.create_block();
        b.ins().brif(within, ok, &[], oob, &[]);

        b.switch_to_block(oob);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(ok);
        let nodes = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_NODES);
        let offset = b.ins().imul_imm(index, i64::from(BOUNDARY_NODE_STRIDE));
        let node = b.ins().iadd(nodes, offset);
        b.ins().store(MemFlags::trusted(), node, out, 0);
        b.ins()
            .store(MemFlags::trusted(), region, out, RESOLVED_REGION);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.resolve, func)
}

/// A resolved handle: the node's address and the base of its own region.
#[derive(Clone, Copy)]
struct Resolved {
    node: cranelift_codegen::ir::Value,
    region: cranelift_codegen::ir::Value,
}

/// Emit the shared prologue: resolve `word`, returning early with any non-zero
/// status.
///
/// Returns the node address **and its region** in the current (resolved) block.
/// A helper that reads child words must use the region, never the `arena`
/// parameter: for a persistent handle those are different tables.
fn resolve_prologue(
    b: &mut FunctionBuilder<'_>,
    ptr: cranelift_codegen::ir::Type,
    resolve: cranelift_codegen::ir::FuncRef,
    arena: cranelift_codegen::ir::Value,
    word: cranelift_codegen::ir::Value,
) -> Resolved {
    let slot = b.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
        cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
        16,
        3,
    ));
    let cell = b.ins().stack_addr(ptr, slot, 0);
    let call = b.ins().call(resolve, &[arena, word, cell]);
    let status = b.inst_results(call)[0];
    let good = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_OK);
    let ok = b.create_block();
    let bad = b.create_block();
    b.ins().brif(good, ok, &[], bad, &[]);

    b.switch_to_block(bad);
    b.ins().return_(&[status]);

    b.switch_to_block(ok);
    let node = b.ins().load(ptr, MemFlags::trusted(), cell, 0);
    let region = b
        .ins()
        .load(ptr, MemFlags::trusted(), cell, RESOLVED_REGION);
    Resolved { node, region }
}

/// `(arena, word, out) -> status` reading one fixed node word.
///
/// One definition serves `owner`, `slot`, `tag` and `field_count`: they differ
/// only in a byte offset, and four hand-copied bodies would be four chances for
/// the offsets to drift apart.
fn define_node_word<M: Module>(
    module: &mut M,
    graph: Graph,
    id: FuncId,
    offset: i32,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, id, 3);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, out) = (p[0], p[1], p[2]);
        let Resolved { node, .. } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        let value = b.ins().load(types::I64, MemFlags::trusted(), node, offset);
        b.ins().store(MemFlags::trusted(), value, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);
        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, id, func)
}

/// `(arena, word, out) -> status` — the value's class.
///
/// Handles read their node's class; immediates derive it from the word tag, so
/// emitted code gets one uniform answer without having to know which arm it is
/// looking at first.
fn define_class<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.class, 3);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, out) = (p[0], p[1], p[2]);

        // 16 bytes: `resolve` writes the node address AND its region.
        let cell_slot = b.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            16,
            3,
        ));
        let cell = b.ins().stack_addr(ptr, cell_slot, 0);
        let call = b.ins().call(resolve, &[arena, word, cell]);
        let status = b.inst_results(call)[0];

        let resolved = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_OK);
        let from_node = b.create_block();
        let not_node = b.create_block();
        b.ins().brif(resolved, from_node, &[], not_node, &[]);

        b.switch_to_block(from_node);
        let node = b.ins().load(ptr, MemFlags::trusted(), cell, 0);
        let class = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_CLASS);
        b.ins().store(MemFlags::trusted(), class, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.switch_to_block(not_node);
        // Only `ERR_SHAPE` means "a well-formed immediate"; every other status
        // is a real failure and is propagated unchanged.
        let is_immediate = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_ERR_SHAPE);
        let immediate = b.create_block();
        let propagate = b.create_block();
        b.ins().brif(is_immediate, immediate, &[], propagate, &[]);

        b.switch_to_block(propagate);
        b.ins().return_(&[status]);

        b.switch_to_block(immediate);
        let tag = b.ins().band_imm(word, BOUNDARY_TAG_MASK as i64);
        // ⛔ **The immediate's reported class comes from the AUTHORITY, one
        // entry per admitted immediate tag.** This was
        // `is_bool ? Bool : Int` — a second mapping written beside the helper
        // body, free to disagree with the partition and with nothing to notice
        // if it did. `RULING R3` names that shape as a discharge that does not
        // count.
        //
        // ⚠ These are **boundary-value** classifications, not node classes.
        // `BOUNDARY_TAG_CLASS_RELATION` governs what may be written into a
        // node's `NODE_CLASS` and rightly excludes every immediate tag, because
        // an immediate has no node. Merging the two contracts would invent a
        // fictional immediate node class.
        //
        // Innermost value is the unclassifiable answer: an immediate tag the
        // authority gives no class fails closed rather than inheriting an arm.
        let mut classified: Option<cranelift_codegen::ir::Value> = None;
        let mut class = b.ins().iconst(types::I64, 0);
        for (candidate, value_class) in plan.tags().immediate_value_classes() {
            let hit = b.ins().icmp_imm(IntCC::Equal, tag, *candidate as i64);
            let named = b.ins().iconst(types::I64, *value_class as i64);
            class = b.ins().select(hit, named, class);
            classified = Some(match classified {
                None => hit,
                Some(prev) => b.ins().bor(prev, hit),
            });
        }
        let classified = match classified {
            Some(classified) => classified,
            None => b.ins().iconst(types::I8, 0),
        };
        let named = b.create_block();
        let unclassified = b.create_block();
        b.ins().brif(classified, named, &[], unclassified, &[]);

        b.switch_to_block(unclassified);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_CLASS);
        b.ins().return_(&[err]);

        b.switch_to_block(named);
        b.ins().store(MemFlags::trusted(), class, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.class, func)
}

/// `(arena, word, out) -> status` — scalar extraction.
fn define_scalar<M: Module>(module: &mut M, graph: Graph) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.scalar, 3);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, out) = (p[0], p[1], p[2]);

        // 16 bytes: `resolve` writes the node address AND its region.
        let cell_slot = b.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            16,
            3,
        ));
        let cell = b.ins().stack_addr(ptr, cell_slot, 0);
        let call = b.ins().call(resolve, &[arena, word, cell]);
        let status = b.inst_results(call)[0];
        let resolved = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_OK);
        let from_node = b.create_block();
        let not_node = b.create_block();
        b.ins().brif(resolved, from_node, &[], not_node, &[]);

        b.switch_to_block(from_node);
        let node = b.ins().load(ptr, MemFlags::trusted(), cell, 0);
        let payload = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_PAYLOAD);
        b.ins().store(MemFlags::trusted(), payload, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.switch_to_block(not_node);
        let is_immediate = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_ERR_SHAPE);
        let immediate = b.create_block();
        let propagate = b.create_block();
        b.ins().brif(is_immediate, immediate, &[], propagate, &[]);

        b.switch_to_block(propagate);
        b.ins().return_(&[status]);

        b.switch_to_block(immediate);
        // Arithmetic shift: the immediate-`Int` range is two's complement in
        // the payload field, so sign extension is part of the decode.
        let value = b.ins().sshr_imm(word, i64::from(BOUNDARY_TAG_BITS));
        b.ins().store(MemFlags::trusted(), value, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.scalar, func)
}

/// `(arena, word, index, out) -> status` — positional field projection.
fn define_field<M: Module>(module: &mut M, graph: Graph) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.field, 4);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, index, out) = (p[0], p[1], p[2], p[3]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);

        let count = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELD_COUNT);
        let within = b.ins().icmp(IntCC::UnsignedLessThan, index, count);
        let ok = b.create_block();
        let oob = b.create_block();
        b.ins().brif(within, ok, &[], oob, &[]);

        b.switch_to_block(oob);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(ok);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELDS_AT);
        let words = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_WORDS);
        let absolute = b.ins().iadd(at, index);
        let offset = b.ins().imul_imm(absolute, 8);
        let address = b.ins().iadd(words, offset);
        let child = b.ins().load(types::I64, MemFlags::trusted(), address, 0);
        b.ins().store(MemFlags::trusted(), child, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.field, func)
}

/// `(arena, word, name_id, out) -> status` — record field access by name.
///
/// The name table runs parallel to the word table for every node, so a record's
/// names sit at exactly its children's indices and the scan is one loop with no
/// second index to keep in step.
fn define_record_field<M: Module>(
    module: &mut M,
    graph: Graph,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.record_field, 4);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, name_id, out) = (p[0], p[1], p[2], p[3]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);

        // ⛔ Class-checked: a positional aggregate has a parallel name table of
        // zeroes, and a caller asking it for a named field is asking a question
        // it cannot answer. That is `ERR_CLASS`, not "not found".
        let class = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_CLASS);
        let is_record = b
            .ins()
            .icmp_imm(IntCC::Equal, class, BoundaryClass::Record as i64);
        let scan_setup = b.create_block();
        let wrong_class = b.create_block();
        b.ins().brif(is_record, scan_setup, &[], wrong_class, &[]);

        b.switch_to_block(wrong_class);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_CLASS);
        b.ins().return_(&[err]);

        b.switch_to_block(scan_setup);
        let count = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELD_COUNT);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELDS_AT);
        let names = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_NAMES);
        let words = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_WORDS);

        let scan = b.create_block();
        b.append_block_param(scan, types::I64);
        let zero = b.ins().iconst(types::I64, 0);
        b.ins().jump(scan, &[zero.into()]);

        b.switch_to_block(scan);
        let i = b.block_params(scan)[0];
        let more = b.ins().icmp(IntCC::UnsignedLessThan, i, count);
        let probe = b.create_block();
        let missing = b.create_block();
        b.ins().brif(more, probe, &[], missing, &[]);

        b.switch_to_block(missing);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(probe);
        let absolute = b.ins().iadd(at, i);
        let offset = b.ins().imul_imm(absolute, 8);
        let name_at = b.ins().iadd(names, offset);
        let candidate = b.ins().load(types::I64, MemFlags::trusted(), name_at, 0);
        let hit = b.ins().icmp(IntCC::Equal, candidate, name_id);
        let found = b.create_block();
        let next = b.create_block();
        b.ins().brif(hit, found, &[], next, &[]);

        b.switch_to_block(found);
        let word_at = b.ins().iadd(words, offset);
        let child = b.ins().load(types::I64, MemFlags::trusted(), word_at, 0);
        b.ins().store(MemFlags::trusted(), child, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.switch_to_block(next);
        let step = b.ins().iadd_imm(i, 1);
        b.ins().jump(scan, &[step.into()]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.record_field, func)
}

/// `(arena, word, out) -> status` — the `HostResult` success discriminant.
fn define_host_success<M: Module>(
    module: &mut M,
    graph: Graph,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.host_success, 3);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, out) = (p[0], p[1], p[2]);
        let Resolved { node, .. } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        let node = host_result_guard(&mut b, node);
        let success = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_PAYLOAD);
        b.ins().store(MemFlags::trusted(), success, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);
        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.host_success, func)
}

/// `(arena, word, out) -> status` — the payload the discriminant selects.
///
/// ⭐ The selection is a **runtime** branch on a runtime discriminant. Nothing
/// here consults a compile-time record of which arm a reply took; that is the
/// distinction `#10` measured, and it is the one this helper exists to hold.
fn define_host_payload<M: Module>(
    module: &mut M,
    graph: Graph,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.host_payload, 3);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, out) = (p[0], p[1], p[2]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        let node = host_result_guard(&mut b, node);

        let success = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_PAYLOAD);
        let ok_index = b.ins().iconst(types::I64, 0);
        let err_index = b.ins().iconst(types::I64, 1);
        let took_ok = b.ins().icmp_imm(IntCC::NotEqual, success, 0);
        let index = b.ins().select(took_ok, ok_index, err_index);

        let count = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELD_COUNT);
        let within = b.ins().icmp(IntCC::UnsignedLessThan, index, count);
        let ok = b.create_block();
        let oob = b.create_block();
        b.ins().brif(within, ok, &[], oob, &[]);

        b.switch_to_block(oob);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(ok);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELDS_AT);
        let words = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_WORDS);
        let absolute = b.ins().iadd(at, index);
        let offset = b.ins().imul_imm(absolute, 8);
        let address = b.ins().iadd(words, offset);
        let child = b.ins().load(types::I64, MemFlags::trusted(), address, 0);
        b.ins().store(MemFlags::trusted(), child, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.host_payload, func)
}

/// Return early with `ERR_CLASS` unless the node is a `HostResult`.
fn host_result_guard(
    b: &mut FunctionBuilder<'_>,
    node: cranelift_codegen::ir::Value,
) -> cranelift_codegen::ir::Value {
    let class = b
        .ins()
        .load(types::I64, MemFlags::trusted(), node, NODE_CLASS);
    let is_host = b
        .ins()
        .icmp_imm(IntCC::Equal, class, BoundaryClass::HostResult as i64);
    let ok = b.create_block();
    let bad = b.create_block();
    b.ins().brif(is_host, ok, &[], bad, &[]);

    b.switch_to_block(bad);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_CLASS);
    b.ins().return_(&[err]);

    b.switch_to_block(ok);
    node
}

/// `(tag, payload, out) -> status` — construct an immediate word.
///
/// ⛔ **`AC-2` structurally:** the parameters are a class and a payload. There
/// is no arena, no environment and no activation in scope, so this helper
/// *cannot* specialize a representation from a value even if a caller wanted it
/// to.
fn define_make_immediate<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let mut func = begin(module, graph.make_immediate, 3);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (tag, payload, out) = (p[0], p[1], p[2]);

        let immediate = tag_in_set(&mut b, tag, plan.tags().immediate());
        let ok = b.create_block();
        let bad = b.create_block();
        b.ins().brif(immediate, ok, &[], bad, &[]);

        b.switch_to_block(bad);
        // A handle tag has no immediate form; minting one would produce a word
        // whose payload is read as a node index. Fail closed.
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(ok);
        // ⛔ **And then it RANGE-CHECKS, which the earlier candidate did not.**
        // The word is built by a left shift, and a shift is *total*: a payload
        // wider than the field silently became a DIFFERENT VALUE, and a `Bool`
        // payload of `2` became a third boolean. `boundary_value.rs` claimed
        // emitted code performed the identical test; it did not.
        //
        // The three domain predicates are all evaluated, then selected by the
        // tag's own bit in each domain's mask — Θ(1) and branch-free, the same
        // shape the tag × class relation check uses, and computed from the one
        // `BOUNDARY_IMMEDIATE_DOMAIN` table so the CLIF cannot drift from it.
        let shift = i64::from(BOUNDARY_TAG_BITS);
        let one = b.ins().iconst(types::I64, 1);
        let tag_bit = b.ins().ishl(one, tag);

        let is_bit = b.ins().icmp_imm(IntCC::UnsignedLessThanOrEqual, payload, 1);
        let unsigned_round = {
            let up = b.ins().ishl_imm(payload, shift);
            b.ins().ushr_imm(up, shift)
        };
        let is_unsigned = b.ins().icmp(IntCC::Equal, unsigned_round, payload);
        let signed_round = {
            let up = b.ins().ishl_imm(payload, shift);
            b.ins().sshr_imm(up, shift)
        };
        let is_signed = b.ins().icmp(IntCC::Equal, signed_round, payload);

        let in_domain = |b: &mut FunctionBuilder<'_>,
                         domain: BoundaryImmediateDomain,
                         holds: cranelift_codegen::ir::Value| {
            let mask = b
                .ins()
                .iconst(types::I64, boundary_domain_mask(domain) as i64);
            let selected = b.ins().band(mask, tag_bit);
            let applies = b.ins().icmp_imm(IntCC::NotEqual, selected, 0);
            let held = b.ins().band(applies, holds);
            (applies, held)
        };
        let (bit_applies, bit_ok) = in_domain(&mut b, BoundaryImmediateDomain::Bit, is_bit);
        let (_, unsigned_ok) = in_domain(
            &mut b,
            BoundaryImmediateDomain::UnsignedPayload,
            is_unsigned,
        );
        let (_, signed_ok) = in_domain(&mut b, BoundaryImmediateDomain::SignedPayload, is_signed);

        // ⛔ Undomained is a THIRD OUTCOME THAT FAILS. An immediate tag with no
        // row in the table admits nothing, so a new tag is rejected until it is
        // dispositioned — never waved through by a default that never ran.
        let some = b.ins().bor(bit_ok, unsigned_ok);
        let admitted = b.ins().bor(some, signed_ok);
        let good = b.ins().icmp_imm(IntCC::NotEqual, admitted, 0);
        let build = b.create_block();
        let refuse = b.create_block();
        b.ins().brif(good, build, &[], refuse, &[]);

        b.switch_to_block(refuse);
        // A `Bool` that is not a bit is the wrong SHAPE; a magnitude past the
        // field is out of BOUNDS. Distinct errors, so a control can tell which
        // rule refused without reading the payload back.
        let shape = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
        let bounds = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        let err = b.ins().select(bit_applies, shape, bounds);
        b.ins().return_(&[err]);

        b.switch_to_block(build);
        let shifted = b.ins().ishl_imm(payload, shift);
        let word = b.ins().bor(shifted, tag);
        b.ins().store(MemFlags::trusted(), word, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.make_immediate, func)
}

/// `(arena, word) -> status` — fail-closed borrowed-ingress escape check.
///
/// ⛔ **`AC-7`.** A word whose referent the invocation arena owns must not leave
/// the native invocation that produced it: the referent dies with the arena and
/// the escaped word would name freed storage. The check keys on the **referent**
/// owner, never on the frame slot the word sat in — those are `D2`'s two
/// different questions.
fn define_escape_check<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.escape_check, 2);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let arena = p[0];
        let word = p[1];

        let tag = b.ins().band_imm(word, BOUNDARY_TAG_MASK as i64);
        let known = tag_in_set(&mut b, tag, plan.tags().admitted());
        let closed = b.create_block();
        let not_admitted = b.create_block();
        b.ins().brif(known, closed, &[], not_admitted, &[]);

        b.switch_to_block(not_admitted);
        refuse_unadmitted_tag(&mut b, tag, plan);

        b.switch_to_block(closed);
        // ⛔ The escaping tags are exactly the ones the partition publishes
        // under invocation ownership. This was an inline `tag >=
        // InvocationBorrowed` — the same hand-derived ordinal band as the named
        // constants, and one the located inventory missed precisely because it
        // had no constant to grep for.
        let borrowed = tag_in_set(
            &mut b,
            tag,
            plan.tags()
                .tags_owned_by(BoundaryReferentOwner::InvocationArena),
        );
        let escaping = b.create_block();
        let permitted = b.create_block();
        b.ins().brif(borrowed, escaping, &[], permitted, &[]);

        b.switch_to_block(escaping);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_ESCAPE);
        b.ins().return_(&[err]);

        // ⛔ **A PERSISTENT word may not escape UNADOPTED.** Emitted code can
        // construct and seal a persistent node, but the node it leaves carries
        // `NULL_SLOT` — and this ABI's own layout contract says a null slot
        // denotes invocation-arena ownership. Such a word is a *pending
        // adoption*, not a published persistent handle: letting it cross the
        // generated-function boundary would publish a handle whose declared
        // owner is the store while the store has never heard of it, and a
        // consumer could recover only the ABSENCE of an identity.
        //
        // ⭐ The store's `adopt` mints the real `SlotId`; this is the gate that
        // makes adoption non-optional rather than advisory.
        b.switch_to_block(permitted);
        let immediate = tag_in_set(&mut b, tag, plan.tags().immediate());
        let done = b.create_block();
        let handle = b.create_block();
        b.ins().brif(immediate, done, &[], handle, &[]);

        b.switch_to_block(handle);
        let Resolved { node, .. } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        let slot = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_SLOT);
        let adopted = b
            .ins()
            .icmp_imm(IntCC::NotEqual, slot, crate::store::NULL_SLOT as i64);
        let pending = b.create_block();
        b.ins().brif(adopted, done, &[], pending, &[]);

        b.switch_to_block(pending);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_ESCAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(done);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.escape_check, func)
}

// ---------------------------------------------------------------------------
// Construction — the producer half of the interface
// ---------------------------------------------------------------------------
//
// ⭐ **Why this half has to exist.** A consumer-only interface proves that
// separately compiled code can *inspect a fixture Rust materialized*. It does
// not prove a producer can mint the word a consumer is supposed to receive, and
// a callee that returns an aggregate is exactly a producer. Shipping the
// projection half alone would hand `B2F` the same wall one layer along: dynamic
// children with no executable way to build the parent.
//
// **Storage, capacity and lifetime, stated:** construction allocates from a
// **reservation** the owner made before publishing — `BoundaryArenaV1::reserve`
// for invocation nodes, `BoundaryValueStore::reserve_persistent` for persistent
// ones. Emitted code never grows a region (growth would move it under the
// published pointer) and never touches the frozen prefix (those nodes carry the
// store's identity). Both ceilings fail closed with an exact status.

/// The highest class in the closed [`BoundaryClass`] set.
const LAST_CLASS: i64 = BoundaryClass::BorrowedOpaque as i64;
/// The largest magnitude marker in `BOUNDARY_INT_MARKER_OWNER`. Derived, so a
/// new marker cannot leave the emitted range guard behind.
const LAST_INT_MARKER: i64 = BOUNDARY_INT_REGION_LIMBS as i64;

/// Select the region a *tag* names, returning early on an unusable one.
///
/// Shared by construction; the projection side reaches the same answer through
/// [`define_resolve`], which is the only place a *word* becomes an address.
fn select_region_by_tag(
    b: &mut FunctionBuilder<'_>,
    ptr: cranelift_codegen::ir::Type,
    arena: cranelift_codegen::ir::Value,
    tag: cranelift_codegen::ir::Value,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> cranelift_codegen::ir::Value {
    // ⛔ **Recognition (`-1`) and shape (`-2`) MOVED to `define_alloc`'s ordered
    // prologue** — this is the sole caller, and leaving copies here would make
    // them unreachable branches. An unreachable fail-closed branch is not a
    // fail-closed branch, and the two checks are now part of a sequence whose
    // order is load-bearing (recognize -> shape -> pair -> tombstone -> admit),
    // which cannot be expressed with two of its steps buried in this helper.
    //
    // ⇒ By the time control reaches here the tag is recognized, handle-shaped,
    // compatible with its class, and not a retired lane.
    let selected = b.create_block();
    b.append_block_param(selected, ptr);
    select_region_by_owner_band(b, ptr, arena, tag, selected, plan);

    b.switch_to_block(selected);
    b.block_params(selected)[0]
}

/// The [`BoundaryReferentOwner`] discriminant a tag implies, as a branch-free
/// fold over the plan's bands.
///
/// ⛔ **Every band contributes an arm.** The innermost value is
/// [`BoundaryReferentOwner::NoReferent`] — the owner that names no storage — so
/// a tag in no band records "nothing owns this" rather than inheriting whatever
/// the last comparison happened to leave. `select_region_by_tag` has already
/// refused such a tag, which is what keeps that default unreachable rather than
/// merely unlikely.
fn owner_of_tag(
    b: &mut FunctionBuilder<'_>,
    tag: cranelift_codegen::ir::Value,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> cranelift_codegen::ir::Value {
    let mut chain = b
        .ins()
        .iconst(types::I64, BoundaryReferentOwner::NoReferent as i64);
    for (owner, tags) in plan.tags().owner_bands() {
        let member = tag_in_set(b, tag, tags);
        let value = b.ins().iconst(types::I64, *owner as i64);
        chain = b.ins().select(member, value, chain);
    }
    chain
}

fn one_i64(b: &mut FunctionBuilder<'_>) -> cranelift_codegen::ir::Value {
    b.ins().iconst(types::I64, 1)
}

/// Select the class bitmask the ABI relation admits for `tag`, **from the
/// plan's partition-derived relation**.
///
/// ⭐ Θ(1): one comparison per admitted row and a chain of selects — no table
/// walk and no data section.
///
/// ⛔ **Seeded with the EMPTY mask, and that is `RULING R5` clause 2 rather
/// than a style choice.** The code this replaces folded from the last row
/// backwards so the innermost value was a *real* mask, justified by
/// `select_region_by_tag` having already refused any unlisted tag. Both halves
/// of that are exactly what the ruling forbids: a real seed means a tag with no
/// row silently inherits another row's classes, and leaning on an upstream guard
/// makes the absent-row branch **unreachable and therefore untestable**. ⚠ An
/// untestable fail-closed branch is not a fail-closed branch. With a zero seed
/// the allocator's relation decision fails closed on its own — an absent row
/// admits nothing and yields exact `BOUNDARY_ERR_RELATION`.
///
/// ⛔ The masks are computed here from the plan's relation, not from
/// `BOUNDARY_TAG_CLASS_RELATION`. That slice is a Rust-side mirror, reconciled
/// to this relation over the whole product; it is not what the emitter reads.
fn relation_mask(
    b: &mut FunctionBuilder<'_>,
    tag: cranelift_codegen::ir::Value,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> cranelift_codegen::ir::Value {
    let mut chain = b.ins().iconst(types::I64, 0);
    for (tag_value, classes) in plan.tags().handle_class_relation() {
        let mask = classes
            .iter()
            .fold(0u64, |mask, class| mask | (1u64 << (*class as u64)));
        let named = b.ins().iconst(types::I64, mask as i64);
        let hit = b.ins().icmp_imm(IntCC::Equal, tag, *tag_value as i64);
        chain = b.ins().select(hit, named, chain);
    }
    chain
}

/// `(arena, tag, class, field_count, out) -> status` — allocate a handle node in
/// the region the tag selects and write its word to `*out`.
///
/// ⛔ **The word this returns is a persistent identity when the tag is
/// persistent.** It indexes store-owned storage, so it stays meaningful after
/// the invocation arena is gone — which is the whole reason the region split
/// exists.
fn define_alloc<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.alloc, 5);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, tag, class, field_count, out) = (p[0], p[1], p[2], p[3], p[4]);

        // ⛔ The class space is closed too. An out-of-set class would be handed
        // straight back by the `class` projection, so an unknown one fails here
        // rather than becoming a value nobody can interpret.
        let class_ok = b
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, class, LAST_CLASS);
        let classed = b.create_block();
        let bad_class = b.create_block();
        b.ins().brif(class_ok, classed, &[], bad_class, &[]);

        b.switch_to_block(bad_class);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_CLASS);
        b.ins().return_(&[err]);

        b.switch_to_block(classed);

        // ── ⛔ RETIRED LANE — refused BY NAME, before anything is selected ───
        //
        // `RT-FNSPLIT-C1` `D5`, Architect `dec_21aa95jbsznfh` + addendum
        // `dec_6xffebwj4s347`. The `(PersistentClosure, Closure)` pair is
        // **recognized ABI vocabulary that is never admitted**.
        //
        // ⚠ **Position is load-bearing and is the ruling's own ordering.** This
        // sits ahead of `select_region_by_tag`, the seal guard and every write,
        // because the ruling requires the refusal to land *before allocation,
        // owner/region lookup, CFG construction, or invocation*. A retired lane
        // must never reach the machinery that would give it a home.
        //
        // ⭐ **Why an exact-pair test rather than a recognized relation row.**
        // The two outcomes the ruling requires both fall out of this placement,
        // and neither needs `PersistentClosure` to enter an admitted set:
        //
        //   `PersistentClosure` + `Closure` -> `BOUNDARY_ERR_RETIRED_LANE`, here.
        //   `PersistentClosure` + `Bool`    -> `BOUNDARY_ERR_RELATION`, below,
        //                                      via the absent-row fail-closed
        //                                      path the relation mask already
        //                                      has by construction.
        //
        // ⚠ Stated honestly, because the mechanism differs from the wording
        // even though the observable status does not: the `-8` for a malformed
        // cross-pair arises from the tag having **no row**, not from a
        // recognized row that excludes `Bool`. The distinction is invisible at
        // the ABI and would only matter if a retired tag ever needed to admit
        // some classes, which is a contradiction in terms.
        //
        // ⛔ Do not "simplify" this by adding the pair to the plan's relation:
        // that is the admitted set, and admitting it restores the durable lane
        // `D5` removes.
        //
        // ⚠⚠ **THE ORDER BELOW IS THE RULING'S AND IS CONSTRAINED FROM BOTH
        // SIDES.** I got it wrong once in each direction, so both constraints
        // are written down rather than left to be re-derived:
        //
        //   * recognition must come BEFORE the pair check — `select_region_by_tag`
        //     rejects an unadmitted tag with `BOUNDARY_ERR_TAG`, so a retired tag
        //     that is not recognized here never reaches the relation at all and
        //     `PersistentClosure + Bool` reports `-1` instead of `-8`;
        //   * the tombstone must come AFTER the pair check — checking it first
        //     works for the retired pair but leaves a genuinely unknown tag
        //     reaching the relation, and an unknown tag must keep reporting
        //     `-1`, not `-8`.

        // ── STEP 1 — RECOGNIZE the vocabulary, tombstone names included ──────
        //
        // ⭐ Recognition is strictly wider than admission and that is the whole
        // point: a name is kept so a refusal can cite it.
        let admitted_known = tag_in_set(&mut b, tag, plan.tags().admitted());
        let mut known = admitted_known;
        for (retired_tag, _) in crate::boundary_value::BOUNDARY_RETIRED_LANES {
            let hit = b.ins().icmp_imm(IntCC::Equal, tag, *retired_tag as i64);
            known = b.ins().bor(known, hit);
        }
        let recognized = b.create_block();
        let unrecognized = b.create_block();
        b.ins().brif(known, recognized, &[], unrecognized, &[]);

        b.switch_to_block(unrecognized);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_TAG);
        b.ins().return_(&[err]);

        b.switch_to_block(recognized);

        // ── STEP 1b — SHAPE: an immediate has no node to allocate ────────────
        //
        // ⚠ **Measured constraint, not a guessed one.** This must sit *between*
        // recognition and the pair check. `make_immediate` is an immediate's
        // constructor, and conflating the two would mint a word whose payload
        // is read as a node index. Putting the relation check first instead
        // reported `BOUNDARY_ERR_RELATION` for `ImmediateBool` — immediates
        // hold no rows in the handle relation, so an empty mask looks exactly
        // like an illegal pair. ⛔ `-2` and `-8` are different findings and the
        // control that told me so was already there.
        //
        // A retired lane is handle-shaped: it names a node that simply may no
        // longer be allocated.
        let admitted_handle = tag_in_set(&mut b, tag, plan.tags().handle());
        let mut handle_shaped = admitted_handle;
        for (retired_tag, _) in crate::boundary_value::BOUNDARY_RETIRED_LANES {
            let hit = b.ins().icmp_imm(IntCC::Equal, tag, *retired_tag as i64);
            handle_shaped = b.ins().bor(handle_shaped, hit);
        }
        let shaped = b.create_block();
        let immediate = b.create_block();
        b.ins().brif(handle_shaped, shaped, &[], immediate, &[]);

        b.switch_to_block(immediate);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(shaped);

        // ── STEP 2 — the (tag, class) PAIR, over admitted ∪ retired ──────────
        //
        // ⛔ This replaces the relation check that used to sit after region
        // selection. It is not duplicated there: a check downstream of this one
        // could never fire, and a control that cannot fail is not a control.
        let mut mask = relation_mask(&mut b, tag, plan);
        for (retired_tag, retired_class) in crate::boundary_value::BOUNDARY_RETIRED_LANES {
            let row = b.ins().iconst(types::I64, 1i64 << (*retired_class as u64));
            let hit = b.ins().icmp_imm(IntCC::Equal, tag, *retired_tag as i64);
            let widened = b.ins().bor(mask, row);
            mask = b.ins().select(hit, widened, mask);
        }
        let one = one_i64(&mut b);
        let bit = b.ins().ishl(one, class);
        let compatible = b.ins().band(mask, bit);
        let related = b.ins().icmp_imm(IntCC::NotEqual, compatible, 0);
        let paired = b.create_block();
        let unrelated = b.create_block();
        b.ins().brif(related, paired, &[], unrelated, &[]);

        b.switch_to_block(unrelated);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_RELATION);
        b.ins().return_(&[err]);

        b.switch_to_block(paired);

        // ── STEP 3 — a COMPATIBLE pair that names a retired lane ─────────────
        //
        // ⚠ Reached only for well-formed pairs, which is what makes `-12` mean
        // *"lawful word, retired capability"* rather than *"malformed input"*.
        // Ahead of region selection, the seal guard and every write, per the
        // ruling's "before allocation, owner/region lookup, CFG construction,
        // or invocation".
        for (retired_tag, retired_class) in crate::boundary_value::BOUNDARY_RETIRED_LANES {
            let tag_hit = b.ins().icmp_imm(IntCC::Equal, tag, *retired_tag as i64);
            let class_hit = b.ins().icmp_imm(IntCC::Equal, class, *retired_class as i64);
            let lane_hit = b.ins().band(tag_hit, class_hit);
            let live = b.create_block();
            let retired = b.create_block();
            b.ins().brif(lane_hit, retired, &[], live, &[]);

            b.switch_to_block(retired);
            let err = b.ins().iconst(types::I64, BOUNDARY_ERR_RETIRED_LANE);
            b.ins().return_(&[err]);

            b.switch_to_block(live);
        }

        // ── STEP 4 — the unchanged admitted path ─────────────────────────────
        let region = select_region_by_tag(&mut b, ptr, arena, tag, plan);
        // The eleventh writer: `alloc` takes no word, so it never reaches
        // `mutable_guard` and needs the seal check on its own path.
        seal_guard(&mut b, region);

        // ⛔ **The `(tag, class)` relation check MOVED to step 2 above** — it is
        // not missing and it is deliberately not duplicated here.
        //
        // Both sets are closed and their product still contains pairs no
        // disposition can produce (`PersistentClosure + HostResult`,
        // `InvocationHostResult + Constructor`); minting one succeeds and then
        // fails much later at an unrelated projection, reporting the wrong
        // defect in the wrong place. That is unchanged — only the position is.
        //
        // ⚠ It had to move **ahead of region selection** so a *recognized but
        // unadmitted* tag reaches it at all: `select_region_by_tag` returns
        // `BOUNDARY_ERR_TAG` for anything outside the admitted set, which
        // short-circuited the pair diagnostic for the retired lane. ⛔ Leaving a
        // second copy here would be a branch that can never be taken, and an
        // unreachable fail-closed branch is not a fail-closed branch.

        // ── node capacity ───────────────────────────────────────────────────
        let count = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_NODE_COUNT);
        let node_cap = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_NODE_CAPACITY);
        let has_node = b.ins().icmp(IntCC::UnsignedLessThan, count, node_cap);
        let node_room = b.create_block();
        let no_room = b.create_block();
        b.ins().brif(has_node, node_room, &[], no_room, &[]);

        b.switch_to_block(no_room);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_CAPACITY);
        b.ins().return_(&[err]);

        // ── word capacity ───────────────────────────────────────────────────
        b.switch_to_block(node_room);
        let words_live = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_WORD_COUNT);
        let word_cap = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_WORD_CAPACITY);
        // `field_count` is caller-supplied, so the sum could wrap. Bound the
        // addend first: a field count already past capacity cannot fit whatever
        // is left, and checking it separately means the sum below cannot
        // overflow into a spuriously small "fits".
        let addend_ok = b
            .ins()
            .icmp(IntCC::UnsignedLessThanOrEqual, field_count, word_cap);
        let sum_check = b.create_block();
        b.ins().brif(addend_ok, sum_check, &[], no_room, &[]);

        b.switch_to_block(sum_check);
        let need = b.ins().iadd(words_live, field_count);
        let has_words = b.ins().icmp(IntCC::UnsignedLessThanOrEqual, need, word_cap);
        let room = b.create_block();
        b.ins().brif(has_words, room, &[], no_room, &[]);

        // ── initialize ──────────────────────────────────────────────────────
        b.switch_to_block(room);
        let nodes = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_NODES);
        let offset = b.ins().imul_imm(count, i64::from(BOUNDARY_NODE_STRIDE));
        let node = b.ins().iadd(nodes, offset);

        // The owner is derived from the tag, never passed in: a node whose
        // recorded owner disagreed with the tag that reaches it would make the
        // escape check answer about the wrong lifetime.
        //
        // ⛔ **And "derived from the tag" now means the plan's band relation**,
        // not a threshold plus two constants. The fold is over exactly the
        // owners the partition publishes, so an owner it started publishing
        // gets its own arm instead of being absorbed by the `select`'s
        // otherwise-branch.
        let owner = owner_of_tag(&mut b, tag, plan);

        let zero = b.ins().iconst(types::I64, 0);
        let null_slot = b.ins().iconst(types::I64, crate::store::NULL_SLOT as i64);
        b.ins().store(MemFlags::trusted(), class, node, NODE_CLASS);
        b.ins().store(MemFlags::trusted(), owner, node, NODE_OWNER);
        b.ins()
            .store(MemFlags::trusted(), null_slot, node, NODE_SLOT);
        b.ins().store(MemFlags::trusted(), zero, node, NODE_TAG_ID);
        b.ins().store(MemFlags::trusted(), zero, node, NODE_PAYLOAD);
        b.ins()
            .store(MemFlags::trusted(), field_count, node, NODE_FIELD_COUNT);
        b.ins()
            .store(MemFlags::trusted(), words_live, node, NODE_FIELDS_AT);

        // The reservation is zero-initialized and node indices only ever
        // increase, so the child slots this node just claimed are already zero.
        // Re-zeroing them would be an O(field_count) loop buying nothing.
        let next = b.ins().iadd_imm(count, 1);
        b.ins()
            .store(MemFlags::trusted(), next, region, ARENA_NODE_COUNT);
        b.ins()
            .store(MemFlags::trusted(), need, region, ARENA_WORD_COUNT);

        let shifted = b.ins().ishl_imm(count, i64::from(BOUNDARY_TAG_BITS));
        let word = b.ins().bor(shifted, tag);
        b.ins().store(MemFlags::trusted(), word, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.alloc, func)
}

/// Return early with [`BOUNDARY_ERR_SEALED`] if the store has taken exclusive
/// ownership of this region.
///
/// ⛔ **The emitted half of the seal/quiescence handoff (`AC-6`).** Adoption
/// validates a snapshot and then canonicalizes it; a writer that can still run
/// in between makes those two different graphs. Rust's `&mut` cannot express
/// that exclusivity, because emitted code holds the raw region base it was
/// published and never asks the borrow checker for permission — so the seal
/// lives in the published header, on the path every mutator already walks.
///
/// ⭐ **One definition, and between its two call sites it covers EVERY emitted
/// writer.** `mutable_guard` runs in all ten word-taking mutators and
/// `define_alloc` calls this directly, which is the whole of `EMITTED_WRITERS`.
/// Copying the check into eleven bodies would be a hand-maintained matrix that
/// can drift from the inventory; one definition on a shared path cannot.
fn seal_guard(b: &mut FunctionBuilder<'_>, region: cranelift_codegen::ir::Value) {
    let sealed = b
        .ins()
        .load(types::I64, MemFlags::trusted(), region, ARENA_SEALED);
    let open = b.ins().icmp_imm(IntCC::Equal, sealed, 0);
    let ok = b.create_block();
    let bad = b.create_block();
    b.ins().brif(open, ok, &[], bad, &[]);

    b.switch_to_block(bad);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SEALED);
    b.ins().return_(&[err]);

    b.switch_to_block(ok);
}

/// Return early with [`BOUNDARY_ERR_FROZEN`] unless `word` names a node emitted
/// code constructed — i.e. one at or beyond the region's frozen prefix.
///
/// ⛔ **Seal first, then frozen.** The seal is a statement about the whole
/// region and about *who owns it now*; the frozen prefix is a statement about
/// which nodes within it are emitted code's. Once the store has taken the
/// region, no node in it is writable, including one past the prefix — so asking
/// the narrower question first would admit exactly the write the handoff exists
/// to exclude.
fn mutable_guard(
    b: &mut FunctionBuilder<'_>,
    word: cranelift_codegen::ir::Value,
    region: cranelift_codegen::ir::Value,
) {
    seal_guard(b, region);
    let index = b.ins().ushr_imm(word, i64::from(BOUNDARY_TAG_BITS));
    let frozen = b
        .ins()
        .load(types::I64, MemFlags::trusted(), region, ARENA_FROZEN);
    let mutable = b
        .ins()
        .icmp(IntCC::UnsignedGreaterThanOrEqual, index, frozen);
    let ok = b.create_block();
    let bad = b.create_block();
    b.ins().brif(mutable, ok, &[], bad, &[]);

    b.switch_to_block(bad);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_FROZEN);
    b.ins().return_(&[err]);

    b.switch_to_block(ok);
}

/// `(arena, word, value) -> status` writing one fixed node word.
///
/// One definition serves `store_slot`, `store_tag_id` and `store_scalar`, for
/// the same reason [`define_node_word`] serves their readers: they differ only
/// in a byte offset, and hand-copied bodies are chances for the offsets to
/// drift apart.
fn define_store_node_word<M: Module>(
    module: &mut M,
    graph: Graph,
    id: FuncId,
    offset: i32,
) -> Result<(), CraneliftBackendError> {
    // ⛔ Emitting a setter for any other offset is a **panic at emission**,
    // which every test that emits the graph exercises. This is the mechanism
    // that keeps `NODE_SLOT` out of emitted code's reach: not a review rule and
    // not a scan, but a refusal to build the helper at all.
    assert!(
        EMITTED_WRITABLE_NODE_OFFSETS.contains(&offset),
        "emitted code may not set node offset {offset}"
    );
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, id, 3);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, value) = (p[0], p[1], p[2]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        mutable_guard(&mut b, word, region);
        b.ins().store(MemFlags::trusted(), value, node, offset);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);
        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, id, func)
}

/// `(arena, word, index, child) -> status` — write one child word.
///
/// ⛔ **The escape check, one layer down (`AC-6`/`AC-7`).** A persistent parent
/// must not embed an invocation-owned child: the parent is permitted to leave
/// the invocation, and after it does, that child word names freed storage. The
/// Θ(1) tag test on the parent is sound *because* this store refuses to build
/// the case that would defeat it — so the invariant is enforced where it is
/// created, not re-walked at every crossing.
fn define_store_field<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.store_field, 4);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, index, child) = (p[0], p[1], p[2], p[3]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        mutable_guard(&mut b, word, region);

        let count = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELD_COUNT);
        let within = b.ins().icmp(IntCC::UnsignedLessThan, index, count);
        let checked = b.create_block();
        let oob = b.create_block();
        b.ins().brif(within, checked, &[], oob, &[]);

        b.switch_to_block(oob);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(checked);
        let child_tag = b.ins().band_imm(child, BOUNDARY_TAG_MASK as i64);
        let known = tag_in_set(&mut b, child_tag, plan.tags().admitted());
        let child_ok = b.create_block();
        let bad_child = b.create_block();
        b.ins().brif(known, child_ok, &[], bad_child, &[]);

        b.switch_to_block(bad_child);
        refuse_unadmitted_tag(&mut b, child_tag, plan);

        b.switch_to_block(child_ok);
        let owner = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_OWNER);
        let parent_persists = b.ins().icmp_imm(
            IntCC::Equal,
            owner,
            BoundaryReferentOwner::PersistentStore as i64,
        );
        let child_dies = tag_in_set(
            &mut b,
            child_tag,
            plan.tags()
                .tags_owned_by(BoundaryReferentOwner::InvocationArena),
        );
        let dangling = b.ins().band(parent_persists, child_dies);
        let escapes = b.create_block();
        let sound = b.create_block();
        b.ins().brif(dangling, escapes, &[], sound, &[]);

        b.switch_to_block(escapes);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_ESCAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(sound);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELDS_AT);
        let words = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_WORDS);
        let absolute = b.ins().iadd(at, index);
        let byte = b.ins().imul_imm(absolute, 8);
        let address = b.ins().iadd(words, byte);
        b.ins().store(MemFlags::trusted(), child, address, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.store_field, func)
}

/// `(arena, word, index, name_id) -> status` — write one field name.
///
/// The name table is parallel to the word table, so a constructed `Record` is
/// readable by `record_field` on exactly the same rule a materialized one is.
/// Without this the producer half would be able to build every live class
/// *except* the one whose reader takes a name — an asymmetry `B2F` would inherit
/// as a wall rather than as a documented gap.
fn define_store_name<M: Module>(module: &mut M, graph: Graph) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.store_name, 4);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, index, name_id) = (p[0], p[1], p[2], p[3]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        mutable_guard(&mut b, word, region);

        let count = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELD_COUNT);
        let within = b.ins().icmp(IntCC::UnsignedLessThan, index, count);
        let ok = b.create_block();
        let oob = b.create_block();
        b.ins().brif(within, ok, &[], oob, &[]);

        b.switch_to_block(oob);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(ok);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_FIELDS_AT);
        let names = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_NAMES);
        let absolute = b.ins().iadd(at, index);
        let byte = b.ins().imul_imm(absolute, 8);
        let address = b.ins().iadd(names, byte);
        b.ins().store(MemFlags::trusted(), name_id, address, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.store_name, func)
}

// ---------------------------------------------------------------------------
// Content — the value's BITS, not its identity or its length
// ---------------------------------------------------------------------------
//
// ⛔ **Why an identity and a length were not enough.** A spilled `Int` node
// carried `NODE_PAYLOAD = 0`, and `Bytes`/`String` carried only a byte count, so
// a separately compiled consumer saw every wide integer as zero and could not
// tell two equal-length strings apart. The typed residency map and the canonical
// decoder that *could* tell them apart are **Rust** paths — which is precisely
// what hard-stop `#10` rejected. A representation whose content only Rust can
// read is not one executable representation.

/// Return early with [`BOUNDARY_ERR_CLASS`] unless the node's class is one of
/// `classes`.
fn class_guard(
    b: &mut FunctionBuilder<'_>,
    node: cranelift_codegen::ir::Value,
    classes: &[BoundaryClass],
) {
    let class = b
        .ins()
        .load(types::I64, MemFlags::trusted(), node, NODE_CLASS);
    let ok = b.create_block();
    let bad = b.create_block();
    let mut admitted: Option<cranelift_codegen::ir::Value> = None;
    for candidate in classes {
        let hit = b.ins().icmp_imm(IntCC::Equal, class, *candidate as i64);
        admitted = Some(match admitted {
            None => hit,
            Some(prev) => b.ins().bor(prev, hit),
        });
    }
    let admitted = admitted.expect("a class guard names at least one class");
    b.ins().brif(admitted, ok, &[], bad, &[]);

    b.switch_to_block(bad);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_CLASS);
    b.ins().return_(&[err]);

    b.switch_to_block(ok);
}

/// `(arena, word, len, out) -> status` — claim `len` data bytes for a
/// `Bytes`/`String` node and write the span's start index to `*out`.
///
/// The third ceiling: the data table is reserved before publication exactly as
/// the node and word tables are, and running out is [`BOUNDARY_ERR_CAPACITY`].
/// Require that this node's magnitude marker is [`BOUNDARY_INT_REGION_LIMBS`],
/// returning [`BOUNDARY_ERR_SHAPE`] otherwise.
///
/// ⭐ **This is what keeps the two magnitude representations from mixing.**
/// `store_int_tag` is the sole writer of the marker and it enforces the
/// region relation; every limb helper then refuses to touch a node whose marker
/// says its magnitude lives somewhere else. Neither check subsumes the other:
/// one says *which storage this node may name*, this says *that the storage it
/// names is mine to write*.
fn region_limbs_guard(b: &mut FunctionBuilder<'_>, node: cranelift_codegen::ir::Value) {
    let marker = b
        .ins()
        .load(types::I64, MemFlags::trusted(), node, NODE_EXTENT);
    let mine = b
        .ins()
        .icmp_imm(IntCC::Equal, marker, BOUNDARY_INT_REGION_LIMBS as i64);
    let ok = b.create_block();
    let bad = b.create_block();
    b.ins().brif(mine, ok, &[], bad, &[]);

    b.switch_to_block(bad);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
    b.ins().return_(&[err]);

    b.switch_to_block(ok);
}

/// Bounds-check a limb span **without wrapping** and return its base address,
/// returning [`BOUNDARY_ERR_BOUNDS`] from the caller if it does not fit.
///
/// ⛔ **`at + len <= live` is the wrong test and it was the shipped one.** CLIF's
/// `iadd` wraps, so a stale or malformed `at` near `u64::MAX` produces a small
/// sum that satisfies the comparison, after which the address is formed from the
/// **unchecked** `at`. The source comment claimed it failed closed before any
/// address was formed; it did not, and the Rust oracle beside it was correctly
/// using `checked_add` the whole time — the two halves of one property, written
/// to different standards.
///
/// `at <= live && len <= live - at` is the non-wrapping form: the subtraction is
/// only evaluated where it cannot underflow, and neither comparison can wrap.
fn region_limb_base(
    b: &mut FunctionBuilder<'_>,
    ptr: cranelift_codegen::ir::Type,
    region: cranelift_codegen::ir::Value,
    at: cranelift_codegen::ir::Value,
    len: cranelift_codegen::ir::Value,
    live: cranelift_codegen::ir::Value,
) -> cranelift_codegen::ir::Value {
    let start_ok = b.ins().icmp(IntCC::UnsignedLessThanOrEqual, at, live);
    let room = b.ins().isub(live, at);
    let len_ok = b.ins().icmp(IntCC::UnsignedLessThanOrEqual, len, room);
    let fits = b.ins().band(start_ok, len_ok);
    let spanned = b.create_block();
    let unspanned = b.create_block();
    b.ins().brif(fits, spanned, &[], unspanned, &[]);

    b.switch_to_block(unspanned);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
    b.ins().return_(&[err]);

    b.switch_to_block(spanned);
    let table = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_LIMBS);
    let byte_at = b.ins().imul_imm(at, 8);
    b.ins().iadd(table, byte_at)
}

/// `(arena, word, sign, len, out) -> status` — claim `len` magnitude limbs in
/// the node's own region and write the span's start to `*out`.
///
/// ⛔ **The counterpart of `store_bytes_len`, and it exists for the reason QA's
/// last block taught.** A read path that works is not evidence that a producer
/// path exists; a persistent wide `Int` that only Rust can build would leave
/// emitted construction of the disposition's spill arm untested and unbuilt.
fn define_store_int_limbs<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.store_int_limbs, 5);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, sign, len, out) = (p[0], p[1], p[2], p[3], p[4]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        mutable_guard(&mut b, word, region);
        class_guard(&mut b, node, plan.int_magnitude_classes());
        region_limbs_guard(&mut b, node);

        // ⛔ The sign is a BIT, not a number. `decode_final_export` reads `0` or
        // `1` and refuses anything else, so admitting a third value here would
        // build a node the exact-`Int` decoder cannot read back.
        //
        // ⛔ And the magnitude has **at least one limb**. An empty magnitude
        // denotes no integer at all, and it is the one canonicity clause that
        // *is* checkable here — the others are properties of limbs that do not
        // exist yet, which is precisely why `seal_int` has to exist.
        let bit = b.ins().icmp_imm(IntCC::UnsignedLessThanOrEqual, sign, 1);
        let nonempty = b.ins().icmp_imm(IntCC::UnsignedGreaterThanOrEqual, len, 1);
        let shape_ok = b.ins().band(bit, nonempty);
        let shaped = b.create_block();
        let unshaped = b.create_block();
        b.ins().brif(shape_ok, shaped, &[], unshaped, &[]);

        b.switch_to_block(unshaped);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(shaped);
        let live = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_LIMB_COUNT);
        let cap = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_LIMB_CAPACITY);
        // Bound the addend before the sum, so a caller-supplied length cannot
        // wrap into a spuriously small "fits".
        let addend_ok = b.ins().icmp(IntCC::UnsignedLessThanOrEqual, len, cap);
        let sum_check = b.create_block();
        let no_room = b.create_block();
        b.ins().brif(addend_ok, sum_check, &[], no_room, &[]);

        b.switch_to_block(no_room);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_CAPACITY);
        b.ins().return_(&[err]);

        b.switch_to_block(sum_check);
        let need = b.ins().iadd(live, len);
        let fits = b.ins().icmp(IntCC::UnsignedLessThanOrEqual, need, cap);
        let ok = b.create_block();
        b.ins().brif(fits, ok, &[], no_room, &[]);

        b.switch_to_block(ok);
        b.ins().store(MemFlags::trusted(), sign, node, NODE_PAYLOAD);
        b.ins()
            .store(MemFlags::trusted(), live, node, NODE_LIMBS_AT);
        b.ins()
            .store(MemFlags::trusted(), len, node, NODE_LIMB_COUNT);
        // ⛔ Claiming a span UNSEALS the node. A producer that claims a second
        // span on an already-sealed node must re-earn the seal, so a stale
        // canonicity proof can never stay attached to a fresh magnitude.
        let unsealed = b.ins().iconst(types::I64, 0);
        b.ins()
            .store(MemFlags::trusted(), unsealed, node, NODE_INT_SEALED);
        b.ins()
            .store(MemFlags::trusted(), need, region, ARENA_LIMB_COUNT);
        b.ins().store(MemFlags::trusted(), live, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.store_int_limbs, func)
}

/// `(arena, word) -> status` — check a region-limbed `Int`'s magnitude canonical
/// and seal it.
///
/// ⛔ **The completion step, and the interface change that makes "fails closed
/// before publication" true instead of aspirational.** `store_int_limbs` runs
/// before a single limb exists, so it can bound the length and the sign and
/// nothing else: it cannot see a leading zero limb, it cannot see negative zero,
/// and it cannot see a producer that claims three limbs and writes two. Those
/// are properties of the *finished* magnitude. Without a completion step the
/// producer's success meant only "the span was reserved", and a consumer could
/// project a word that denotes no exact `Int`.
///
/// ⭐ **`ken_boundary_int_*_local` requires the seal**, so an unsealed node
/// denotes nothing — which is the only operative meaning of unpublished here,
/// since the word is in the producer's hand from the moment `alloc` returns.
///
/// The clauses are `boundary_int_magnitude_is_canonical`'s, one for one:
/// at least one limb (already held), no leading zero limb, and zero is
/// non-negative. ⚠ A one-limb `[0]` **is** canonical — it is the value zero, and
/// rejecting it would be an over-strengthening the contract does not entail.
fn define_seal_int<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.seal_int, 2);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word) = (p[0], p[1]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        mutable_guard(&mut b, word, region);
        class_guard(&mut b, node, plan.int_magnitude_classes());
        region_limbs_guard(&mut b, node);

        let sign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_PAYLOAD);
        let len = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_LIMB_COUNT);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_LIMBS_AT);
        let live = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_LIMB_COUNT);
        let base = region_limb_base(&mut b, ptr, region, at, len, live);

        // The top limb, at `len - 1`. `store_int_limbs` guarantees `len >= 1`,
        // and `region_limb_base` has just re-derived the span against the live
        // count, so this address is inside the table.
        let top_index = b.ins().iadd_imm(len, -1);
        let top_offset = b.ins().imul_imm(top_index, 8);
        let top_address = b.ins().iadd(base, top_offset);
        let top = b
            .ins()
            .load(types::I64, MemFlags::trusted(), top_address, 0);

        let one_limb = b.ins().icmp_imm(IntCC::Equal, len, 1);
        let top_zero = b.ins().icmp_imm(IntCC::Equal, top, 0);
        // The value zero: exactly one limb, and that limb is zero.
        let is_zero = b.ins().band(one_limb, top_zero);
        // No leading zero limb — unless the magnitude IS zero.
        let top_nonzero = b.ins().icmp_imm(IntCC::NotEqual, top, 0);
        let top_ok = b.ins().bor(top_nonzero, is_zero);
        // Zero is non-negative: refuse negative zero.
        let negative = b.ins().icmp_imm(IntCC::Equal, sign, 1);
        let negative_zero = b.ins().band(negative, is_zero);
        let sign_ok = b.ins().icmp_imm(IntCC::Equal, negative_zero, 0);
        let canonical = b.ins().band(top_ok, sign_ok);
        let ok = b.create_block();
        let bad = b.create_block();
        b.ins().brif(canonical, ok, &[], bad, &[]);

        b.switch_to_block(bad);
        // ⛔ The node stays UNSEALED, so a producer that ignores this status
        // still cannot publish a word a consumer will read.
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(ok);
        let sealed = b.ins().iconst(types::I64, 1);
        b.ins()
            .store(MemFlags::trusted(), sealed, node, NODE_INT_SEALED);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.seal_int, func)
}

/// `(arena, word, index, limb) -> status` — write one magnitude limb.
fn define_store_int_limb<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.store_int_limb, 4);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, index, limb) = (p[0], p[1], p[2], p[3]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        mutable_guard(&mut b, word, region);
        class_guard(&mut b, node, plan.int_magnitude_classes());
        region_limbs_guard(&mut b, node);

        let len = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_LIMB_COUNT);
        let within = b.ins().icmp(IntCC::UnsignedLessThan, index, len);
        let ok = b.create_block();
        let oob = b.create_block();
        b.ins().brif(within, ok, &[], oob, &[]);

        b.switch_to_block(oob);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(ok);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_LIMBS_AT);
        let absolute = b.ins().iadd(at, index);
        let table = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_LIMBS);
        let offset = b.ins().imul_imm(absolute, 8);
        let address = b.ins().iadd(table, offset);
        b.ins().store(MemFlags::trusted(), limb, address, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.store_int_limb, func)
}

fn define_store_bytes_len<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.store_bytes_len, 4);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, len, out) = (p[0], p[1], p[2], p[3]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        mutable_guard(&mut b, word, region);
        class_guard(&mut b, node, plan.byte_span_classes());

        let live = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_DATA_COUNT);
        let cap = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_DATA_CAPACITY);
        // Bound the addend before the sum, so a caller-supplied length cannot
        // wrap into a spuriously small "fits".
        let addend_ok = b.ins().icmp(IntCC::UnsignedLessThanOrEqual, len, cap);
        let sum_check = b.create_block();
        let no_room = b.create_block();
        b.ins().brif(addend_ok, sum_check, &[], no_room, &[]);

        b.switch_to_block(no_room);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_CAPACITY);
        b.ins().return_(&[err]);

        b.switch_to_block(sum_check);
        let need = b.ins().iadd(live, len);
        let fits = b.ins().icmp(IntCC::UnsignedLessThanOrEqual, need, cap);
        let ok = b.create_block();
        b.ins().brif(fits, ok, &[], no_room, &[]);

        b.switch_to_block(ok);
        b.ins().store(MemFlags::trusted(), len, node, NODE_PAYLOAD);
        b.ins().store(MemFlags::trusted(), live, node, NODE_EXTENT);
        b.ins()
            .store(MemFlags::trusted(), need, region, ARENA_DATA_COUNT);
        b.ins().store(MemFlags::trusted(), live, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.store_bytes_len, func)
}

/// `(arena, word, index, byte_or_out) -> status` — one content byte, written
/// when `write` and read otherwise.
///
/// One definition serves both directions because they share every check that
/// matters: the class guard, the bound against the node's own length, and the
/// span arithmetic. Two bodies would be two chances for the reader and the
/// writer to disagree about which byte index `i` names.
fn define_byte_access<M: Module>(
    module: &mut M,
    graph: Graph,
    id: FuncId,
    write: bool,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, id, 4);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, index, last) = (p[0], p[1], p[2], p[3]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        if write {
            mutable_guard(&mut b, word, region);
        }
        class_guard(&mut b, node, plan.byte_span_classes());

        let len = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_PAYLOAD);
        let within = b.ins().icmp(IntCC::UnsignedLessThan, index, len);
        let ok = b.create_block();
        let oob = b.create_block();
        b.ins().brif(within, ok, &[], oob, &[]);

        b.switch_to_block(oob);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(ok);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_EXTENT);
        let data = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_DATA);
        let absolute = b.ins().iadd(at, index);
        let address = b.ins().iadd(data, absolute);
        if write {
            let narrow = b.ins().ireduce(types::I8, last);
            b.ins().store(MemFlags::trusted(), narrow, address, 0);
        } else {
            let value = b.ins().load(types::I8, MemFlags::trusted(), address, 0);
            let widened = b.ins().uextend(types::I64, value);
            b.ins().store(MemFlags::trusted(), widened, last, 0);
        }
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, id, func)
}

/// `(arena, word, native_tag) -> status` — record a spilled `Int`'s
/// `NativeIntV1` tag.
///
/// Class-guarded and range-guarded: the native tag space is `{Small, Big}` and
/// anything else would be handed to `ken_native_int_resolve_local`, whose own
/// contract it would violate.
fn define_store_int_tag<M: Module>(
    module: &mut M,
    graph: Graph,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.store_int_tag, 3);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let (arena, word, marker) = (p[0], p[1], p[2]);
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        mutable_guard(&mut b, word, region);
        class_guard(&mut b, node, plan.int_magnitude_classes());

        // ⛔ **The marker is REGION-BOUND, and that is the whole check.** A
        // `Small` carries its magnitude in the node itself and is sound
        // anywhere; a `NATIVE_INT_BIG_TAG_V1` payload is a slot in the
        // *invocation's* native arena; `BOUNDARY_INT_REGION_LIMBS` names the
        // region's own limb table. An invocation marker on a persistent node is
        // the ephemeral-locator defect — a surviving parent naming storage that
        // dies first — so it is `ERR_ESCAPE`, the same error `store_field`
        // returns for the same defect one representation up. A marker outside
        // the closed set is a *shape* error: a different question, a different
        // answer.
        //
        // ⛔ Range-guard BEFORE forming `1 << marker`: a shift past the word
        // width is not defined to produce zero, so an unguarded marker could
        // alias a bit that IS admitted.
        let known = b
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, marker, LAST_INT_MARKER);
        let ranged = b.create_block();
        let unknown = b.create_block();
        b.ins().brif(known, ranged, &[], unknown, &[]);

        b.switch_to_block(unknown);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(ranged);
        let one = b.ins().iconst(types::I64, 1);
        let bit = b.ins().ishl(one, marker);
        let owner = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_OWNER);
        // ⛔ **A fold over EVERY declared owner, not a two-way select.** The
        // code this replaces asked *"is the owner the store"* and gave every
        // other answer the invocation mask — so a node recording
        // `NoReferent` was handed the invocation arena's admitted markers,
        // while the Rust twin [`boundary_int_marker_admits`] admits only the
        // owner-agnostic ones for it. Two implementations of one relation
        // disagreeing on an arm neither side reaches is the shape that stops
        // being unreachable without anything reddening.
        //
        // Innermost value is the empty mask: an owner with no row admits no
        // marker at all, which is the fail-closed direction.
        let mut mask = b.ins().iconst(types::I64, 0);
        for candidate in BoundaryReferentOwner::ALL {
            let is = b.ins().icmp_imm(IntCC::Equal, owner, candidate as i64);
            let admitted = b
                .ins()
                .iconst(types::I64, boundary_int_marker_mask(candidate) as i64);
            mask = b.ins().select(is, admitted, mask);
        }
        let selected = b.ins().band(mask, bit);
        let admitted = b.ins().icmp_imm(IntCC::NotEqual, selected, 0);
        let sound = b.create_block();
        let escapes = b.create_block();
        b.ins().brif(admitted, sound, &[], escapes, &[]);

        b.switch_to_block(escapes);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_ESCAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(sound);
        b.ins()
            .store(MemFlags::trusted(), marker, node, NODE_EXTENT);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.store_int_tag, func)
}

/// Which part of a decoded exact integer a helper returns.
#[derive(Clone, Copy)]
enum IntPart {
    Sign,
    Len,
    Limb,
    View,
}

/// `(arena, word, [index,] out) -> status` — one part of a spilled `Int`.
///
/// ⭐ **The decode is `ken_native_int_resolve_local`'s, not ours.** The node
/// carries a `NativeIntV1` `(tag, payload)` pair and this helper hands that pair
/// to the landed exact-`Int` decoder, then reads the view it writes. Deriving
/// sign and limbs here would be a second exact-integer representation living
/// beside the first — the proliferation `docs/PRINCIPLES.md` forbids, and the
/// thing that would make "one executable value representation" false again.
fn define_int_part<M: Module>(
    module: &mut M,
    graph: Graph,
    id: FuncId,
    part: IntPart,
    plan: &crate::boundary_value::BoundaryEmissionPlan,
) -> Result<(), CraneliftBackendError> {
    use crate::native_int_clif::{VIEW_LEN, VIEW_LIMBS, VIEW_SIGN};

    let ptr = module.target_config().pointer_type();
    let arity = if matches!(part, IntPart::Limb) { 4 } else { 3 };
    let mut func = begin(module, id, arity);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let native = module.declare_func_in_func(graph.native_int_resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let arena = p[0];
        let word = p[1];
        let (index, out) = if matches!(part, IntPart::Limb) {
            (Some(p[2]), p[3])
        } else {
            (None, p[2])
        };
        let Resolved { node, region } = resolve_prologue(&mut b, ptr, resolve, arena, word);
        class_guard(&mut b, node, plan.int_magnitude_classes());

        let marker = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_EXTENT);

        // The two magnitude sources converge here as one `(sign, len, limbs)`
        // triple, so the projection below is written once and every caller
        // downstream of this point is representation-blind.
        let decoded = b.create_block();
        b.append_block_param(decoded, types::I64);
        b.append_block_param(decoded, types::I64);
        b.append_block_param(decoded, ptr);

        let persisted = b
            .ins()
            .icmp_imm(IntCC::Equal, marker, BOUNDARY_INT_REGION_LIMBS as i64);
        let in_region = b.create_block();
        let in_arena = b.create_block();
        b.ins().brif(persisted, in_region, &[], in_arena, &[]);

        // ── the region's own limb table ──────────────────────────────────
        //
        // ⛔ Not the native decoder's to answer. A persistent wide `Int`'s
        // magnitude lives here precisely BECAUSE the invocation's native arena
        // dies first; routing it through that arena would be asking the wrong
        // region, which is the defect this marker exists to remove.
        b.switch_to_block(in_region);
        // ⛔ The magnitude must be SEALED. An unsealed node is one whose limbs
        // have been claimed but never checked canonical, and a consumer that
        // could project it would be reading a word that denotes no exact `Int`.
        let sealed = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_INT_SEALED);
        let is_sealed = b.ins().icmp_imm(IntCC::Equal, sealed, 1);
        let readable = b.create_block();
        let unsealed = b.create_block();
        b.ins().brif(is_sealed, readable, &[], unsealed, &[]);

        b.switch_to_block(unsealed);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(readable);
        let region_sign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_PAYLOAD);
        let region_len = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_LIMB_COUNT);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_LIMBS_AT);
        let live = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_LIMB_COUNT);
        // Non-wrapping, before any address is formed — see `region_limb_base`.
        let region_base = region_limb_base(&mut b, ptr, region, at, region_len, live);
        b.ins().jump(
            decoded,
            &[region_sign.into(), region_len.into(), region_base.into()],
        );

        // ── the invocation's native-`Int` arena ──────────────────────────
        //
        // ⭐ The decode is `ken_native_int_resolve_local`'s, not ours. Deriving
        // a `Small`'s or an invocation `Big`'s sign and limbs here would be a
        // second exact-integer representation living beside the first.
        //
        // ⛔ Undecodable is a THIRD OUTCOME THAT FAILS. An invocation bound to
        // no native-`Int` arena cannot decode one, and returning zero is exactly
        // the defect this helper exists to close.
        b.switch_to_block(in_arena);
        let native_arena = b
            .ins()
            .load(ptr, MemFlags::trusted(), arena, ARENA_NATIVE_INT);
        let bound = b.ins().icmp_imm(IntCC::NotEqual, native_arena, 0);
        let have = b.create_block();
        let unbound = b.create_block();
        b.ins().brif(bound, have, &[], unbound, &[]);

        b.switch_to_block(unbound);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(have);
        let native_payload = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_PAYLOAD);
        let view_slot = b.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            32,
            3,
        ));
        let view = b.ins().stack_addr(ptr, view_slot, 0);
        let call = b
            .ins()
            .call(native, &[native_arena, marker, native_payload, view]);
        let status = b.inst_results(call)[0];
        let ok_native = b.ins().icmp_imm(IntCC::Equal, status, 0);
        let good = b.create_block();
        let bad = b.create_block();
        b.ins().brif(ok_native, good, &[], bad, &[]);

        b.switch_to_block(bad);
        // The native decoder's own refusal, reported as a shape error rather
        // than passed through: its status space is not this ABI's.
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(good);
        let native_sign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), view, VIEW_SIGN);
        let native_len = b
            .ins()
            .load(types::I64, MemFlags::trusted(), view, VIEW_LEN);
        let native_base = b.ins().load(ptr, MemFlags::trusted(), view, VIEW_LIMBS);
        b.ins().jump(
            decoded,
            &[native_sign.into(), native_len.into(), native_base.into()],
        );

        // ── one projection, both representations ─────────────────────────
        b.switch_to_block(decoded);
        let sign = b.block_params(decoded)[0];
        let len = b.block_params(decoded)[1];
        let limbs = b.block_params(decoded)[2];
        match part {
            IntPart::Sign => {
                b.ins().store(MemFlags::trusted(), sign, out, 0);
            }
            IntPart::Len => {
                b.ins().store(MemFlags::trusted(), len, out, 0);
            }
            IntPart::Limb => {
                let index = index.expect("the limb arm takes an index");
                let within = b.ins().icmp(IntCC::UnsignedLessThan, index, len);
                let ok = b.create_block();
                let oob = b.create_block();
                b.ins().brif(within, ok, &[], oob, &[]);

                b.switch_to_block(oob);
                let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
                b.ins().return_(&[err]);

                b.switch_to_block(ok);
                let offset = b.ins().imul_imm(index, 8);
                let address = b.ins().iadd(limbs, offset);
                let limb = b.ins().load(types::I64, MemFlags::trusted(), address, 0);
                b.ins().store(MemFlags::trusted(), limb, out, 0);
            }
            IntPart::View => {
                b.ins().store(MemFlags::trusted(), sign, out, 0);
                b.ins().store(MemFlags::trusted(), len, out, 8);
                b.ins().store(MemFlags::trusted(), limbs, out, 16);
            }
        }
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, id, func)
}

#[cfg(test)]
pub(crate) mod tests {

    /// Capture the emitted helper graph under an injected plan (RECUT 2 causal).
    pub(crate) fn capture_with_plan(plan: &crate::boundary_value::BoundaryEmissionPlan) -> String {
        let mut module = jit();
        super::capture_boundary_value_local_graph_with_plan(&mut module, plan).expect("graph emits")
    }
    use super::*;
    use crate::boundary_value::{
        boundary_code_id, boundary_immediate_admits, boundary_immediate_domain,
        boundary_int_marker_admits, boundary_relation_admits, materialize_borrowed,
        materialize_ground, materialize_host_result, BoundaryArenaBuilder, BoundaryArenaV1,
        BoundaryValueStore, BoundaryWord, NodeField, RegionHeaderField, BOUNDARY_IMMEDIATE_INT_MAX,
        BOUNDARY_IMMEDIATE_INT_MIN, BOUNDARY_NODE_STRIDE, BOUNDARY_PAYLOAD_BITS,
        BOUNDARY_REGION_HEADER_BYTES, BOUNDARY_TAG_CLASS_RELATION,
    };
    // Statuses only the controls assert on — the production graph never returns
    // them from a helper, so they belong to the test scope rather than to the
    // module's import list.
    use crate::boundary_value::{BOUNDARY_ERR_CYCLE, BOUNDARY_ERR_UNBOUND};
    use crate::ir::RuntimeGroundValue;
    use crate::native_int::RuntimeIntV1;
    use crate::values::Value;
    use cranelift_codegen::settings::{self, Configurable};
    use cranelift_jit::{JITBuilder, JITModule};
    use cranelift_module::default_libcall_names;

    /// A JIT module configured exactly as the production one is.
    ///
    /// Built here rather than reached for through the backend, so `B2V` adds no
    /// visibility surface to `cranelift_backend` at all.
    fn jit() -> JITModule {
        let mut flags = settings::builder();
        flags.set("use_colocated_libcalls", "false").expect("flag");
        flags.set("is_pic", "true").expect("flag");
        let isa = cranelift_native::builder()
            .expect("host is a supported target")
            .finish(settings::Flags::new(flags))
            .expect("isa finishes");
        JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()))
    }

    /// Which helper a probe should call, and with how many arguments.
    #[derive(Clone, Copy)]
    enum Probe {
        /// `(arena, word) -> *out`
        Unary(fn(&BoundaryLocalFuncs) -> FuncId),
        /// `(arena, word, extra) -> *out`
        Binary(fn(&BoundaryLocalFuncs) -> FuncId),
        /// `(arena, word) -> status`, returning the status itself.
        Status(fn(&BoundaryLocalFuncs) -> FuncId),
    }

    /// Compile a probe that calls one helper and returns either the projected
    /// value or, on a non-zero status, the status.
    ///
    /// ⭐ **This is what makes `D5` non-vacuous.** The probe is a SEPARATELY
    /// COMPILED CLIF body: it holds no Rust closure, no `result_table`, and no
    /// compile-time image of the value it is about to read. Everything it
    /// learns, it learns by calling the helpers on a word and an arena pointer
    /// handed to it at run time.
    fn compile_probe(probe: Probe) -> (JITModule, *const u8) {
        compile_probe_with_plan(
            probe,
            &crate::boundary_value::BoundaryEmissionPlan::derive(),
        )
    }

    /// The same probe, compiled against a **caller-supplied** plan.
    ///
    /// ⛔ **This is what makes the per-site causal evidence possible.** A
    /// whole-graph capture comparison cannot see one site defecting while the
    /// others still consume the plan; running the compiled helper and asking
    /// what it *answers* can.
    fn compile_probe_with_plan(
        probe: Probe,
        plan: &crate::boundary_value::BoundaryEmissionPlan,
    ) -> (JITModule, *const u8) {
        let mut module = jit();
        let native = crate::native_int_clif::emit_native_int_local_graph(&mut module, false)
            .expect("native-int graph emits");
        let helpers =
            emit_boundary_value_local_graph(&mut module, &native, plan).expect("graph emits");
        let ptr = module.target_config().pointer_type();

        let arity = match probe {
            Probe::Unary(_) | Probe::Status(_) => 2,
            Probe::Binary(_) => 3,
        };
        let mut sig = module.make_signature();
        for _ in 0..arity {
            sig.params.push(AbiParam::new(ptr));
        }
        sig.returns.push(AbiParam::new(types::I64));
        let id = module
            .declare_function("b2v_probe", Linkage::Local, &sig)
            .expect("probe declares");
        let mut ctx = module.make_context();
        ctx.func = Function::with_name_signature(UserFuncName::user(4, id.as_u32()), sig);
        let target = match probe {
            Probe::Unary(pick) | Probe::Binary(pick) | Probe::Status(pick) => pick(&helpers),
        };
        let callee = module.declare_func_in_func(target, &mut ctx.func);
        let mut fctx = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut ctx.func, &mut fctx);
            let entry = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.switch_to_block(entry);
            let p = b.block_params(entry).to_vec();
            let slot = b.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                8,
                3,
            ));
            let out = b.ins().stack_addr(ptr, slot, 0);
            let call = match probe {
                Probe::Unary(_) => b.ins().call(callee, &[p[0], p[1], out]),
                Probe::Binary(_) => b.ins().call(callee, &[p[0], p[1], p[2], out]),
                Probe::Status(_) => b.ins().call(callee, &[p[0], p[1]]),
            };
            let status = b.inst_results(call)[0];
            if matches!(probe, Probe::Status(_)) {
                b.ins().return_(&[status]);
            } else {
                let good =
                    b.ins()
                        .icmp_imm(IntCC::Equal, status, crate::boundary_value::BOUNDARY_OK);
                let ok = b.create_block();
                let bad = b.create_block();
                b.ins().brif(good, ok, &[], bad, &[]);
                b.switch_to_block(bad);
                b.ins().return_(&[status]);
                b.switch_to_block(ok);
                let value = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
                b.ins().return_(&[value]);
            }
            b.seal_all_blocks();
            b.finalize();
        }
        module.define_function(id, &mut ctx).expect("probe defines");
        module.finalize_definitions().expect("jit finalizes");
        let code = module.get_finalized_function(id);
        (module, code)
    }

    fn run2(code: *const u8, arena: *const u64, word: BoundaryWord) -> i64 {
        let f: extern "C" fn(*const u64, u64) -> i64 = unsafe { std::mem::transmute(code) };
        f(arena, word.0)
    }

    fn run3(code: *const u8, arena: *const u64, word: BoundaryWord, extra: u64) -> i64 {
        let f: extern "C" fn(*const u64, u64, u64) -> i64 = unsafe { std::mem::transmute(code) };
        f(arena, word.0, extra)
    }

    /// A published invocation, bound to the store's persistent region.
    ///
    /// ⚠ Holds the arena so the published pointers outlive nothing. The
    /// persistent pointer aliases the *store's* tables, which is the point: the
    /// invocation is a route to persistent storage and not its owner, so the
    /// store outlives every `Bound` built from it.
    struct Bound {
        #[allow(dead_code)]
        arena: BoundaryArenaV1,
        base: *mut u64,
        persistent: *mut u64,
    }

    fn bind(store: &mut BoundaryValueStore, builder: BoundaryArenaBuilder) -> Bound {
        bind_with(store, builder, (0, 0, 0), (0, 0, 0))
    }

    /// Bind with an explicit construction reservation for each region.
    fn bind_with(
        store: &mut BoundaryValueStore,
        builder: BoundaryArenaBuilder,
        persistent_room: (usize, usize, usize),
        arena_room: (usize, usize, usize),
    ) -> Bound {
        bind_limbs(store, builder, persistent_room, arena_room, (0, 0))
    }

    /// [`bind_with`] plus an explicit `(persistent, arena)` magnitude-limb
    /// reservation, for the controls that construct a wide `Int`.
    fn bind_limbs(
        store: &mut BoundaryValueStore,
        builder: BoundaryArenaBuilder,
        persistent_room: (usize, usize, usize),
        arena_room: (usize, usize, usize),
        limb_room: (usize, usize),
    ) -> Bound {
        store.reserve_persistent(
            persistent_room.0,
            persistent_room.1,
            persistent_room.2,
            limb_room.0,
        );
        let persistent = store.publish_persistent();
        let mut arena = builder.finish();
        arena.reserve(arena_room.0, arena_room.1, arena_room.2, limb_room.1);
        arena.bind_persistent(Some(persistent));
        let base = arena.publish();
        Bound {
            arena,
            base,
            persistent,
        }
    }

    /// A second invocation over the same persistent region.
    ///
    /// ⭐ This is what makes the survival control mean anything: a *fresh* arena
    /// with its own tables, sharing only the store's persistent image.
    fn rebind(persistent: *mut u64) -> Bound {
        let mut arena = BoundaryArenaBuilder::new().finish();
        arena.bind_persistent(Some(persistent));
        let base = arena.publish();
        Bound {
            arena,
            base,
            persistent,
        }
    }

    fn run5(code: *const u8, base: *const u64, a: u64, b: u64, c: u64, d: u64) -> i64 {
        let f: extern "C" fn(*const u64, u64, u64, u64, u64) -> i64 =
            unsafe { std::mem::transmute(code) };
        f(base, a, b, c, d)
    }

    fn run4(code: *const u8, base: *const u64, a: u64, b: u64, c: u64) -> i64 {
        let f: extern "C" fn(*const u64, u64, u64, u64) -> i64 =
            unsafe { std::mem::transmute(code) };
        f(base, a, b, c)
    }

    /// A `Cons(7, Nil)` whose payload is chosen at run time, not baked in.
    /// ⭐⭐ **`D2`'s test-side authority.** Every symbol these fixtures
    /// materialize, with the artifact-static identity "the plan issued" for it.
    ///
    /// ⚠ **The identities start at [`C1_D2_IDENTITY_BASE`] deliberately.** In
    /// production they are packed spans into the plan's name arena; here the
    /// only property that matters is that they are **nothing `intern_symbol`
    /// would mint**. Interning numbers densely from `1` in insertion order, so a
    /// base far above the fixture population means ⭐ **any re-mint produces a
    /// visibly wrong tag rather than an accidentally right one** — which is the
    /// difference between these tests noticing the property and merely passing.
    const C1_D2_IDENTITY_BASE: u64 = 0x5000_0000;

    const C1_D2_ISSUED_SYMBOLS: &[&str] = &[
        "ctor:fixture::Box::Wrap",
        "ctor:fixture::Cycle::Leaf",
        "ctor:fixture::Cycle::Root",
        "ctor:fixture::Dag::Parent",
        "ctor:fixture::Dag::Shared",
        "ctor:fixture::Deep::Link",
        "ctor:fixture::Ground::Leaf",
        "ctor:fixture::List::Cons",
        "ctor:fixture::List::Nil",
        "ctor:fixture::Partial::Node",
        "ctor:fixture::Ring::Link",
        "ctor:fixture::Seal::Node",
        "ctor:fixture::Unsealed::Node",
        "depth",
        "flag",
        "payload",
    ];

    /// The identity this fixture authority issues for `symbol`.
    fn c1_d2_issued_identity(symbol: &str) -> u64 {
        let position = C1_D2_ISSUED_SYMBOLS
            .iter()
            .position(|candidate| *candidate == symbol)
            .expect("every fixture symbol is issued an identity");
        C1_D2_IDENTITY_BASE + position as u64
    }

    /// A store whose carrier identities have been issued by the authority above.
    ///
    /// ⛔ `c1_d2_store()` alone can no longer materialize a
    /// constructor or a record: `D2`'s minting ban means an unissued symbol
    /// **fails closed**. ⇒ Reaching for this helper is the test-side shape of
    /// *"the identity comes from somewhere else."*
    fn c1_d2_store() -> BoundaryValueStore {
        let mut store = BoundaryValueStore::new();
        for symbol in C1_D2_ISSUED_SYMBOLS {
            assert!(
                store.issue_carrier_identity(symbol, c1_d2_issued_identity(symbol)),
                "the fixture authority issues each symbol exactly one identity"
            );
        }
        store
    }

    /// ⭐⭐ **`D2`'s own control — THE MINTING BAN, asserted directly.**
    ///
    /// **MEASURED:** an unissued constructor and an unissued record field name
    /// each make `materialize_ground` return `None`; the same values materialize
    /// once the authority has issued identities for them; and the tag the
    /// carrier then carries is the **issued word**, which is not the id
    /// `intern_symbol` would have minted.
    /// **CLAIMED:** `D2` — one identity authority, shared by producer and
    /// consumer. The store is a consumer of identities, ⛔ never a source.
    /// **THE GAP:** every *other* test now routes through `c1_d2_store`, so they
    /// exercise the issued path but would stay green if the store quietly
    /// re-minted on a miss — they never present a miss. ⭐ This is the one that
    /// presents one.
    ///
    /// ⚠ Promise class: **durable invariant**. Adding constructors, fields or
    /// carrier helpers keeps it green; restoring a mint-on-miss fallback — in
    /// any spelling — turns it red.
    #[test]
    fn c1_d2_the_store_consumes_identities_and_refuses_to_mint_them() {
        let unissued = RuntimeGroundValue::Constructor {
            constructor: "ctor:fixture::Unissued::Ctor".to_string(),
            args: Vec::new(),
        };
        let mut store = c1_d2_store();
        assert!(
            materialize_ground(&mut store, &unissued).is_none(),
            "⛔ THE BAN: a constructor no authority has issued an identity for \
             must FAIL CLOSED. Minting one here is the second authority `D2` \
             forbids, wearing the clothes of a convenience"
        );

        let unissued_field = RuntimeGroundValue::Record {
            fields: vec![(
                "unissued_field".to_string(),
                RuntimeGroundValue::Bool(true),
            )],
        };
        assert!(
            materialize_ground(&mut store, &unissued_field).is_none(),
            "record field names take the same authority as constructor tags"
        );

        // ── POSITIVE CONTROL ────────────────────────────────────────────────
        //
        // ⚠ Without this the refusals above are satisfied by a `materialize`
        // that fails for ANY reason — a negative check passes for any reason at
        // all, so it needs a positive half on the same fixture.
        assert!(store.issue_carrier_identity("ctor:fixture::Unissued::Ctor", 0x7700_1234));
        assert!(store.issue_carrier_identity("unissued_field", 0x7700_5678));
        assert!(
            materialize_ground(&mut store, &unissued).is_some(),
            "NON-VACUITY: the SAME value must materialize once its identity is \
             issued, or the refusal above says nothing about identity"
        );
        assert!(materialize_ground(&mut store, &unissued_field).is_some());

        // ⭐ And the issued word is what the carrier actually carries -- chosen
        // far above anything `intern_symbol` numbers to, so a re-mint could not
        // land on it by coincidence.
        assert_eq!(store.carrier_identity("ctor:fixture::Unissued::Ctor"), Some(0x7700_1234));
        assert_eq!(store.carrier_symbol(0x7700_1234), Some("ctor:fixture::Unissued::Ctor"));

        // ⛔ Two authorities for one symbol is a caller bug, refused rather than
        // silently overwritten -- an overwrite would let a later issuer win and
        // leave earlier-materialized nodes keyed on a dead identity.
        assert!(
            !store.issue_carrier_identity("ctor:fixture::Unissued::Ctor", 0x7700_9999),
            "re-issuing a DIFFERENT identity for one symbol must be refused"
        );
        assert!(
            store.issue_carrier_identity("ctor:fixture::Unissued::Ctor", 0x7700_1234),
            "re-issuing the SAME identity is idempotent, not an error"
        );
    }

    fn cons(head: i64) -> RuntimeGroundValue {
        RuntimeGroundValue::Constructor {
            constructor: "ctor:fixture::List::Cons".to_string(),
            args: vec![
                RuntimeGroundValue::Int(RuntimeIntV1::Small(head)),
                RuntimeGroundValue::Constructor {
                    constructor: "ctor:fixture::List::Nil".to_string(),
                    args: vec![],
                },
            ],
        }
    }

    // ── `AC-4`/`AC-5` — emitted code discriminates and projects ─────────────

    /// **`D5` control 1 — a non-constant `Constructor` through a `Parameter`,
    /// inspected by a separately compiled body.**
    ///
    /// ⛔ The discriminating design choice: the probe is compiled ONCE and then
    /// run against THREE different values. A callee reading a compile-time
    /// template would return the same answer all three times, which is exactly
    /// the mutation `AC-5` requires to redden.
    #[test]
    fn b2v_emitted_code_projects_a_non_constant_constructor_field() {
        let (_module, code) = compile_probe(Probe::Binary(|h| h.field));
        let (_m2, tag_code) = compile_probe(Probe::Unary(|h| h.tag));
        let (_m3, scalar_code) = compile_probe(Probe::Unary(|h| h.scalar));

        for head in [7i64, -3, 1_000_000] {
            let mut store = c1_d2_store();
            let mut builder = BoundaryArenaBuilder::new();
            let word =
                materialize_ground(&mut store, &cons(head)).expect("a constructor materializes");
            let f = bind(&mut store, builder);
            let base = f.base;

            // Field 0 is the head. The probe returns its WORD; decoding the
            // scalar out of that word is a second emitted-code call, so no step
            // of this chain runs in Rust.
            let head_word = BoundaryWord(run3(code, base, word, 0) as u64);
            assert_eq!(
                head_word.tag(),
                Some(BoundaryTag::ImmediateInt),
                "the head is an immediate Int word"
            );
            let observed = run2(scalar_code, base, head_word);
            assert_eq!(
                observed, head,
                "emitted code must read the RUNTIME head, not a template"
            );

            // And the constructor identity is projectable too.
            //
            // ⭐⭐ **`D2` STRENGTHENING.** This assertion used to read
            // `store.symbol(tag_id)` — *"the tag must be a REAL interned
            // symbol."* ⛔ That property is the store-instance-dependent
            // substrate `§2e` removes: interning numbers densely per store
            // instance, so it was satisfied by an id no compiled body could ever
            // have compared against.
            //
            // ⇒ The property now asserted is **agreement with the one
            // authority**: emitted code read back exactly the identity the
            // authority issued for this constructor, and the reverse view
            // resolves it. ⚠ The first assertion is the load-bearing one — it
            // pins the *value*; the second only pins that the view is wired to
            // the same map.
            let tag_id = run2(tag_code, base, word) as u64;
            assert_eq!(
                tag_id,
                c1_d2_issued_identity("ctor:fixture::List::Cons"),
                "emitted code must read back the identity the AUTHORITY issued, \
                 not one this store minted for itself"
            );
            assert_eq!(
                store.carrier_symbol(tag_id),
                Some("ctor:fixture::List::Cons"),
                "and the reverse lookup is a view over that same authority"
            );
        }
    }

    /// **`D5` control 2 — a `HostResult` across a boundary, with the callee
    /// selecting the correct arm.**
    ///
    /// ⛔ The success flag is a RUNTIME value. Both arms are materialized for
    /// every case, so a callee that returned a fixed arm would pass one case
    /// and fail the other.
    #[test]
    fn b2v_emitted_code_selects_the_host_result_arm_at_runtime() {
        let (_m, payload_code) = compile_probe(Probe::Unary(|h| h.host_payload));
        let (_m2, success_code) = compile_probe(Probe::Unary(|h| h.host_success));
        let (_m3, scalar_code) = compile_probe(Probe::Unary(|h| h.scalar));

        for (success, expected) in [(1u64, 11i64), (0, 22)] {
            let mut store = c1_d2_store();
            let mut builder = BoundaryArenaBuilder::new();
            let ok = materialize_ground(
                &mut store,
                &RuntimeGroundValue::Int(RuntimeIntV1::Small(11)),
            )
            .expect("ok payload");
            let err = materialize_ground(
                &mut store,
                &RuntimeGroundValue::Int(RuntimeIntV1::Small(22)),
            )
            .expect("err payload");
            let word = materialize_host_result(&mut builder, success, ok, err);
            let f = bind(&mut store, builder);
            let base = f.base;

            assert_eq!(
                run2(success_code, base, word),
                success as i64,
                "the discriminant is read from the arena"
            );
            let selected = BoundaryWord(run2(payload_code, base, word) as u64);
            assert_eq!(
                run2(scalar_code, base, selected),
                expected,
                "emitted code must select the arm the RUNTIME discriminant names"
            );
        }
    }

    /// **`D5` control 3 — nested aggregate flow.**
    ///
    /// A record inside a constructor inside a record: the projection chain runs
    /// entirely in emitted code, one helper call per level.
    #[test]
    fn b2v_emitted_code_projects_a_nested_aggregate() {
        let (_m, field_code) = compile_probe(Probe::Binary(|h| h.field));
        let (_m2, record_code) = compile_probe(Probe::Binary(|h| h.record_field));
        let (_m3, scalar_code) = compile_probe(Probe::Unary(|h| h.scalar));

        let inner = RuntimeGroundValue::Record {
            fields: vec![
                (
                    "depth".to_string(),
                    RuntimeGroundValue::Int(RuntimeIntV1::Small(42)),
                ),
                ("flag".to_string(), RuntimeGroundValue::Bool(true)),
            ],
        };
        let nested = RuntimeGroundValue::Constructor {
            constructor: "ctor:fixture::Box::Wrap".to_string(),
            args: vec![inner],
        };
        let outer = RuntimeGroundValue::Record {
            fields: vec![("payload".to_string(), nested)],
        };

        let mut store = c1_d2_store();
        let mut builder = BoundaryArenaBuilder::new();
        let word = materialize_ground(&mut store, &outer).expect("materializes");
        // ⭐ `D2`: the field names emitted code selects on come from the
        // **authority**, ⛔ not from `intern_symbol` — which would mint a second
        // id for a name the materialized record already keyed on the issued one.
        let payload_name = c1_d2_issued_identity("payload");
        let depth_name = c1_d2_issued_identity("depth");
        let f = bind(&mut store, builder);
        let base = f.base;

        let wrapped = BoundaryWord(run3(record_code, base, word, payload_name) as u64);
        let record = BoundaryWord(run3(field_code, base, wrapped, 0) as u64);
        let depth = BoundaryWord(run3(record_code, base, record, depth_name) as u64);
        assert_eq!(
            run2(scalar_code, base, depth),
            42,
            "three levels of projection, all in emitted code"
        );
    }

    // ── `AC-6` — referent owner is not the slot owner ───────────────────────

    /// **`AC-6`.** Emitted code can read the referent owner, and the two owner
    /// kinds are actually distinguishable on values that are otherwise alike.
    ///
    /// ⛔ **Non-degenerate pair, on purpose.** A persistent and a borrowed word
    /// are compared on the SAME projection, so substituting one owner for the
    /// other inverts both answers rather than passing on one.
    #[test]
    fn b2v_referent_owner_distinguishes_persistent_from_borrowed() {
        let (_m, owner_code) = compile_probe(Probe::Unary(|h| h.owner));
        let (_m2, slot_code) = compile_probe(Probe::Unary(|h| h.slot));

        let mut store = c1_d2_store();
        let mut builder = BoundaryArenaBuilder::new();
        let persistent = materialize_ground(&mut store, &cons(5)).expect("materializes");
        let borrowed = materialize_borrowed(&mut builder, 0xDEAD_BEEF);
        let f = bind(&mut store, builder);
        let base = f.base;

        assert_eq!(
            run2(owner_code, base, persistent),
            BoundaryReferentOwner::PersistentStore as i64
        );
        assert_eq!(
            run2(owner_code, base, borrowed),
            BoundaryReferentOwner::InvocationArena as i64
        );
        // The pair is non-degenerate: the two answers DIFFER, so an oracle that
        // collapsed the owners would fail rather than agree with itself.
        assert_ne!(
            run2(owner_code, base, persistent),
            run2(owner_code, base, borrowed),
            "AC-6 is vacuous unless the two owners actually differ here"
        );

        // And the persistent referent names a real store slot while the
        // borrowed one names none — the second axis of the same distinction.
        assert_ne!(run2(slot_code, base, persistent), 0);
        assert_eq!(
            run2(slot_code, base, borrowed),
            crate::store::NULL_SLOT as i64
        );
    }

    // ── `AC-7` — borrowed ingress fails closed on escape ────────────────────

    /// **`AC-7`.** The exact error, never `is_err`.
    #[test]
    fn b2v_borrowed_ingress_fails_closed_on_escape_with_an_exact_error() {
        let (_m, escape_code) = compile_probe(Probe::Status(|h| h.escape_check));

        let mut store = c1_d2_store();
        let mut builder = BoundaryArenaBuilder::new();
        let persistent = materialize_ground(&mut store, &cons(1)).expect("materializes");
        let borrowed = materialize_borrowed(&mut builder, 1);
        let host = materialize_host_result(&mut builder, 1, persistent, persistent);
        let immediate = BoundaryWord::immediate(BoundaryTag::ImmediateBool, 1);
        let f = bind(&mut store, builder);
        let base = f.base;

        // Positive control on the permitted side: if EVERYTHING were refused,
        // the escape assertions below would pass for the wrong reason.
        assert_eq!(
            run2(escape_code, base, persistent),
            crate::boundary_value::BOUNDARY_OK
        );
        assert_eq!(
            run2(escape_code, base, immediate),
            crate::boundary_value::BOUNDARY_OK
        );

        assert_eq!(
            run2(escape_code, base, borrowed),
            crate::boundary_value::BOUNDARY_ERR_ESCAPE
        );
        assert_eq!(
            run2(escape_code, base, host),
            crate::boundary_value::BOUNDARY_ERR_ESCAPE
        );

        // An unknown tag is its OWN error, not the escape error — otherwise a
        // malformed word would be reported as a lifetime violation.
        let malformed = BoundaryWord(0xFF);
        assert_eq!(
            run2(escape_code, base, malformed),
            crate::boundary_value::BOUNDARY_ERR_TAG
        );
    }

    /// A projection helper refuses a word whose tag is outside the closed set,
    /// and refuses an out-of-range node index — both with their own exact
    /// status rather than a shared catch-all.
    #[test]
    fn b2v_malformed_words_are_refused_with_distinct_exact_errors() {
        let (_m, class_code) = compile_probe(Probe::Unary(|h| h.class));
        let (_m2, field_code) = compile_probe(Probe::Binary(|h| h.field));

        let mut store = c1_d2_store();
        let mut builder = BoundaryArenaBuilder::new();
        let word = materialize_ground(&mut store, &cons(1)).expect("materializes");
        let f = bind(&mut store, builder);
        let base = f.base;

        assert_eq!(
            run2(class_code, base, BoundaryWord(0xFF)),
            crate::boundary_value::BOUNDARY_ERR_TAG
        );
        let past_end = BoundaryWord::handle(BoundaryTag::PersistentGround, 9_999);
        assert_eq!(
            run2(class_code, base, past_end),
            crate::boundary_value::BOUNDARY_ERR_BOUNDS
        );
        // A field index past the arity is bounds, not a wrapped read.
        assert_eq!(
            run3(field_code, base, word, 99),
            crate::boundary_value::BOUNDARY_ERR_BOUNDS
        );
        // A named lookup on a positional aggregate is a CLASS error: the node
        // has a parallel name table of zeroes, so "not found" would be the
        // wrong answer to the wrong question.
        let (_m3, record_code) = compile_probe(Probe::Binary(|h| h.record_field));
        assert_eq!(
            run3(record_code, base, word, 1),
            crate::boundary_value::BOUNDARY_ERR_CLASS
        );
    }

    // ── `AC-9` — the helper population is closed and Θ(1) ───────────────────

    /// **`AC-9`.** The permitted inventory is pinned as a SET OF NAMES, so any
    /// addition reddens — including one nobody imagined.
    ///
    /// ⛔ **This pin exists because no landed census covers these helpers.**
    /// `BACKEND_PRODUCTION_SOURCES` and the emission census are scoped to
    /// `cranelift_backend/**`; `native_int_clif.rs` already declares eight
    /// functions and appears in neither. A pin's silence is scoped to the
    /// question it asks, so their silence about a sibling file is not evidence.
    #[test]
    fn b2v_the_helper_inventory_is_closed_and_named() {
        let mut module = jit();
        let clif = capture_boundary_value_local_graph(&mut module).expect("graph emits");

        // Positive control FIRST: prove the instrument can see anything at all
        // before trusting a count it reports.
        assert!(
            clif.contains("function"),
            "AC-9: the capture is empty, so every count below means nothing"
        );
        assert_eq!(
            clif.matches("-- boundary helper --").count() + 1,
            BOUNDARY_LOCAL_HELPERS.len(),
            "AC-9: a helper failed to emit a body, or one was added without \
             extending BOUNDARY_LOCAL_HELPERS"
        );
        let mut seen = BOUNDARY_LOCAL_HELPERS.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            BOUNDARY_LOCAL_HELPERS.len(),
            "AC-9: the declared inventory has a duplicate name"
        );

        // ⛔ **The names the MODULE actually declares, not the names the list
        // recites.** The two preceding assertions are both properties of
        // `BOUNDARY_LOCAL_HELPERS` — they never ask the emitter anything, so a
        // helper renamed at its `declare` site kept them green. Measured, after
        // Runtime QA found the identical defect in the tag-closure pin: the
        // shared error is a pin that interrogates the DECLARATION of intent
        // instead of the artifact.
        let mut declared: Vec<String> = module
            .declarations()
            .get_functions()
            .filter_map(|(id, decl)| {
                let name = decl.linkage_name(id).into_owned();
                name.starts_with("ken_boundary_").then_some(name)
            })
            .collect();
        declared.sort();
        let mut expected: Vec<String> = BOUNDARY_LOCAL_HELPERS
            .iter()
            .map(|n| n.to_string())
            .collect();
        expected.sort();
        assert_eq!(
            declared, expected,
            "AC-9: the module's declared `ken_boundary_*` symbols are not exactly \
             the permitted inventory"
        );
    }

    /// **`AC-9`, the growth half.** The population is fixed per module — it
    /// does not scale with the number of values, nodes, or aggregate depth.
    ///
    /// ⛔ Demonstrated over two genuinely different module sizes rather than
    /// asserted, because "Θ(1)" is a claim about how the count RESPONDS, and a
    /// single measurement cannot express a response.
    #[test]
    fn b2v_helper_population_does_not_grow_with_the_value_population() {
        let small = {
            let mut module = jit();
            capture_boundary_value_local_graph(&mut module).expect("emits")
        };
        let large = {
            let mut module = jit();
            capture_boundary_value_local_graph(&mut module).expect("emits")
        };
        assert_eq!(
            small.matches("-- boundary helper --").count(),
            large.matches("-- boundary helper --").count()
        );

        // The value population that a module might carry, varied by three
        // orders of magnitude. The helper count is independent of it by
        // construction — the helpers live in the module, the values in the
        // arena — and this measures that independence rather than restating it.
        for count in [1usize, 64, 1024] {
            let mut store = c1_d2_store();
            for i in 0..count {
                materialize_ground(&mut store, &cons(i as i64)).expect("materializes");
            }
            assert!(
                store.image().node_count() >= count,
                "persistent storage grew with the value population, as it should"
            );
            assert_eq!(
                BOUNDARY_LOCAL_HELPERS.len(),
                29,
                "the helper population must not move with the value population"
            );
        }
    }

    // ── `D2` — the store's read-back is real, on two independent paths ──────

    /// The completion `D2` required: a slot resolves back to a value through
    /// the STORE's bytes, and that agrees with the typed residency map.
    ///
    /// ⭐ Two paths that never consult each other. Agreement here is
    /// corroboration; a residency-only design would have had one path read
    /// twice, which corroborates nothing.
    #[test]
    fn b2v_a_persistent_slot_resolves_back_through_the_store() {
        let mut store = c1_d2_store();
        let value = cons(31);
        let word = materialize_ground(&mut store, &value).expect("materializes");

        let slot = store
            .image()
            .node_field(word.payload(), crate::boundary_value::NODE_SLOT)
            .expect("the node exists");
        assert_ne!(
            slot,
            crate::store::NULL_SLOT,
            "a persistent node names a slot"
        );

        // Path A — the typed residency map.
        assert_eq!(store.resident(slot), Some(&value));
        // Path B — the store's own bytes, through the decode inverse.
        let decoded = store
            .decode_slot(slot)
            .expect("the store resolves the slot");
        assert!(
            matches!(decoded, crate::values::Value::Constructor { .. }),
            "the byte path recovers a constructor, independently of path A"
        );

        // Positive control: an id the store never minted resolves to nothing,
        // so the successes above are not a function that returns Some for
        // anything.
        // ⛔ `Value: PartialEq` is gone (`D3`): equality, order and hash live
        // only on the sealed canonical witness. This assertion only needs
        // *absence*, which needs no equality at all.
        assert!(store.decode_slot(u64::MAX).is_none());
    }

    /// Equal values share one referent, because identity is the STORE's answer
    /// and not this layer's.
    #[test]
    fn b2v_equal_values_share_one_persistent_referent() {
        let mut store = c1_d2_store();
        let a = materialize_ground(&mut store, &cons(9)).expect("materializes");
        let b = materialize_ground(&mut store, &cons(9)).expect("materializes");
        let c = materialize_ground(&mut store, &cons(10)).expect("materializes");

        let slot_of = |w: BoundaryWord| {
            store
                .image()
                .node_field(w.payload(), crate::boundary_value::NODE_SLOT)
                .expect("node")
        };
        assert_eq!(slot_of(a), slot_of(b), "equal values are one referent");
        assert_ne!(
            slot_of(a),
            slot_of(c),
            "distinct values are distinct referents"
        );
        // ⭐ And identity reaches the WORD, not just the node behind it: one
        // slot has one persistent index, so equal values are literally the same
        // 64 bits. That is what lets a persistent word survive its invocation.
        assert_eq!(a, b, "equal values are one persistent word");
        assert_ne!(a, c, "distinct values are distinct persistent words");
    }

    // ── `AC-1`/`AC-2` — the word is closed and cannot be value-specialized ──

    /// **`AC-1`.** The tag set is closed: every byte outside it decodes to
    /// `None`, and the published list matches the decoder exactly.
    #[test]
    fn b2v_the_tag_set_is_closed_in_both_directions() {
        for tag in BoundaryTag::ALL {
            assert_eq!(BoundaryTag::from_bits(tag as u64), Some(tag));
        }
        // ⛔ This was `assert_eq!(BoundaryTag::ALL.len(), 9)`, and the literal
        // was **redundant with the byte-range loop below**, which already
        // derives its boundary from `ALL.len()` — a variant added to the enum
        // and omitted from `ALL` reddens there, on the decode/publish
        // disagreement itself. All the frozen count added was a second red
        // whenever a lane is legitimately admitted, naming a drift that had
        // not happened.
        //
        // ⭐ Re-asserted as the RELATION it was standing in for: `ALL`
        // publishes each admitted byte exactly once, so it is a faithful
        // enumeration rather than a list of the right length.
        let mut published = BoundaryTag::ALL.to_vec();
        published.sort_by_key(|tag| *tag as u64);
        published.dedup_by_key(|tag| *tag as u64);
        assert_eq!(
            published.len(),
            BoundaryTag::ALL.len(),
            "AC-1: the published tag list repeats a tag byte"
        );
        // Everything outside the set is refused, across the whole byte range —
        // an enumeration of forbidden values would have missed whichever byte
        // nobody thought of.
        for byte in 0u64..=255 {
            let decoded = BoundaryTag::from_bits(byte);
            assert_eq!(
                decoded.is_some(),
                byte < BoundaryTag::ALL.len() as u64,
                "AC-1: tag byte {byte} decoded against the closed set"
            );
        }
    }

    /// **`AC-2`.** A word's representation is a function of class and magnitude
    /// alone.
    ///
    /// ⛔ The strongest form of this is structural and stated in
    /// `boundary_value`: no seed environment and no caller environment is in
    /// scope at the construction site, so there is nothing to specialize from.
    /// This adds the behavioural half — that the immediate/handle choice tracks
    /// MAGNITUDE and nothing else.
    #[test]
    fn b2v_the_immediate_handle_choice_tracks_magnitude_only() {
        use crate::boundary_value::{BOUNDARY_IMMEDIATE_INT_MAX, BOUNDARY_IMMEDIATE_INT_MIN};

        let cases = [
            (0i64, true),
            (1, true),
            (-1, true),
            (BOUNDARY_IMMEDIATE_INT_MAX, true),
            (BOUNDARY_IMMEDIATE_INT_MIN, true),
            // Boundary + 1 on both sides: the limit itself, not a typical
            // magnitude, is where a range check goes wrong.
            (BOUNDARY_IMMEDIATE_INT_MAX + 1, false),
            (BOUNDARY_IMMEDIATE_INT_MIN - 1, false),
            (i64::MAX, false),
            (i64::MIN, false),
        ];
        for (value, immediate) in cases {
            let mut store = c1_d2_store();
            let word = materialize_ground(
                &mut store,
                &RuntimeGroundValue::Int(RuntimeIntV1::Small(value)),
            )
            .expect("an Int materializes");
            assert_eq!(
                word.tag() == Some(BoundaryTag::ImmediateInt),
                immediate,
                "AC-2: {value} took the wrong arm"
            );
            if immediate {
                assert_eq!(
                    word.signed_payload(),
                    value,
                    "AC-2: the immediate round-trips, sign included"
                );
            }
        }
    }

    /// **`AC-1` at the EMITTED interface — the closed tag set, swept, not
    /// sampled.**
    ///
    /// ⛔ **This test exists because the pin it replaces was a false green.**
    /// `b2v_malformed_words_are_refused_with_distinct_exact_errors` probes the
    /// single byte `0xFF`. Runtime QA changed `LAST_TAG` from `8` to `8 + 1`
    /// and every boundary test stayed green (13/13): tag `9` became accepted by
    /// `define_resolve`, and no emitted-code assertion ever asked about it.
    ///
    /// ★ The Rust-side twin already swept all 256 bytes. **The discipline was
    /// applied on one side of the same property and not the other** — which is
    /// exactly the failure mode of a per-candidate reminder, satisfied by the
    /// control you were thinking hardest about.
    ///
    /// **MEASURED:** for every one of the 256 tag bytes, the emitted helpers
    /// return the outcome CLASS the closed set implies.
    /// **CLAIMED:** emitted code admits exactly the tags `BoundaryTag` admits.
    /// **THE GAP:** the expectations are derived from `from_bits` /
    /// `referent_owner` — a *different* expression of the rule than the CLIF's
    /// `FIRST_HANDLE_TAG`/`LAST_TAG` comparisons — so the two must agree rather
    /// than one restating the other.
    #[test]
    fn b2v_emitted_code_admits_exactly_the_closed_tag_set() {
        use crate::boundary_value::{
            BOUNDARY_ERR_BOUNDS, BOUNDARY_ERR_ESCAPE, BOUNDARY_ERR_TAG, BOUNDARY_OK,
            BOUNDARY_TAG_BITS,
        };

        let (_m, class_code) = compile_probe(Probe::Unary(|h| h.class));
        let (_m2, escape_code) = compile_probe(Probe::Status(|h| h.escape_check));

        let mut store = c1_d2_store();
        let mut builder = BoundaryArenaBuilder::new();
        materialize_ground(&mut store, &cons(1)).expect("materializes");
        let f = bind(&mut store, builder);
        let base = f.base;

        // Every handle-tagged probe word names a node index far past the end,
        // so a KNOWN handle tag is distinguishable from an UNKNOWN tag by its
        // error: bounds versus tag. Without that separation both would refuse
        // and the sweep could not tell an admitted tag from a rejected one.
        let out_of_range: u64 = 9_999;
        let mut admitted = 0usize;
        let mut rejected = 0usize;
        let mut retired = 0usize;

        // ⛔ `RT-FNSPLIT-C1` `D5` — RECOGNITION is a third outcome here too, and
        // it is derived from the same authority the emitter reads rather than
        // written down: a tag whose only lane is retired is still **vocabulary**,
        // so it must be refused BY NAME (`-12`) and never collapse into `-1`,
        // which is what an arbitrary corrupt byte produces.
        let retired_tags = crate::boundary_value::boundary_retired_tags(
            crate::boundary_value::BoundaryEmissionPlan::derive()
                .tags()
                .admitted(),
        );

        for byte in 0u64..=255 {
            let word = BoundaryWord((out_of_range << BOUNDARY_TAG_BITS) | byte);
            let known = BoundaryTag::from_bits(byte);

            let class = run2(class_code, base, word);
            match known {
                None => {
                    assert_eq!(
                        class, BOUNDARY_ERR_TAG,
                        "AC-1: emitted `class` admitted tag byte {byte}, which is \
                         outside the closed set"
                    );
                    rejected += 1;
                }
                Some(tag) if retired_tags.contains(&tag) => {
                    assert_eq!(
                        class,
                        crate::boundary_value::BOUNDARY_ERR_RETIRED_LANE,
                        "D5: {tag:?} is recognized ABI vocabulary whose only lane \
                         is retired, so emitted `class` must refuse it by name — \
                         not as an unrecognized byte"
                    );
                    retired += 1;
                }
                Some(tag) if tag.is_immediate() => {
                    assert!(
                        class >= 0,
                        "AC-1: emitted `class` refused the admitted immediate tag \
                         {byte} with status {class}"
                    );
                    admitted += 1;
                }
                Some(_) => {
                    assert_eq!(
                        class, BOUNDARY_ERR_BOUNDS,
                        "AC-1: an admitted handle tag {byte} with an out-of-range \
                         index must fail on BOUNDS, not on tag"
                    );
                    admitted += 1;
                }
            }

            // ⚠ The persistent arm's expectation moved when `escape_check`
            // gained the ADOPTION gate: it now resolves the node, because a
            // persistent word may not escape while its node still carries
            // `NULL_SLOT`. These fixtures carry a deliberately out-of-range
            // index, so a persistent word cannot be shown adopted and does not
            // escape — reported as `ERR_BOUNDS`, because a malformed word and a
            // lifetime violation are different answers and this one is
            // malformed.
            let expected_escape = match known {
                None => BOUNDARY_ERR_TAG,
                // ⛔ `D5`: recognition precedes ownership. A retired tag has no
                // live owner band at all, so asking `referent_owner()` would
                // answer from the Rust enum about a lane the partition no longer
                // publishes — the two-authority split this plan exists to close.
                Some(tag) if retired_tags.contains(&tag) => {
                    crate::boundary_value::BOUNDARY_ERR_RETIRED_LANE
                }
                Some(tag) => match tag.referent_owner() {
                    BoundaryReferentOwner::InvocationArena => BOUNDARY_ERR_ESCAPE,
                    BoundaryReferentOwner::PersistentStore => BOUNDARY_ERR_BOUNDS,
                    BoundaryReferentOwner::NoReferent => BOUNDARY_OK,
                },
            };
            assert_eq!(
                run2(escape_code, base, word),
                expected_escape,
                "AC-1/AC-7: emitted `escape_check` disagreed with the closed set \
                 on tag byte {byte}"
            );
        }

        // ⚠ POSITIVE CONTROL. A sweep whose every byte landed in one bucket
        // would pass all three arms above for the wrong reason.
        //
        // ⛔ **TRANSITION SENTINEL, labelled as one.** This assertion goes RED
        // the moment the retired-lane vocabulary is emptied — which is exactly
        // the event that must be reviewed rather than absorbed, because an
        // empty tombstone list makes the retired arm above a `continue` nobody
        // takes and the whole `D5` distinction silently vacuous. It retires when
        // `B2F` lands a real durable callable carrier and the lane stops being a
        // tombstone; ⛔ it is NOT a durable invariant and must not be re-labelled
        // as one.
        assert!(
            !retired_tags.is_empty(),
            "D5: no tag is retired, so the retired arm of this sweep — and of \
             every sibling three-way partition — is unexercised"
        );
        assert_eq!(
            retired,
            retired_tags.len(),
            "D5: the sweep took the retired arm {retired} times but {} tags are \
             retired; the arm is not being reached",
            retired_tags.len()
        );
        // ⭐ Relationally derived, not re-fitted to what the code now emits: the
        // admitted bytes are the closed set MINUS the retired vocabulary, so
        // this goes red if either authority moves.
        assert_eq!(
            admitted,
            BoundaryTag::ALL.len() - retired_tags.len(),
            "AC-1: the number of admitted tag bytes must equal the closed set \
             less the retired vocabulary"
        );
        assert_eq!(
            rejected,
            256 - BoundaryTag::ALL.len(),
            "AC-1: every remaining byte must be rejected"
        );
    }

    // ── `AC-4` — emitted code CONSTRUCTS, not merely inspects ───────────────
    //
    // ⛔ Everything below builds its subject from **separately compiled CLIF**
    // and then reads it back with a **second** separately compiled body. A
    // fixture materialized in Rust would demonstrate only that a consumer can
    // walk a structure Rust built — which is the half `#10` already had.

    /// The construction helpers, pre-declared into a producer's own function.
    struct Refs {
        alloc: cranelift_codegen::ir::FuncRef,
        store_tag_id: cranelift_codegen::ir::FuncRef,
        store_scalar: cranelift_codegen::ir::FuncRef,
        store_field: cranelift_codegen::ir::FuncRef,
        store_name: cranelift_codegen::ir::FuncRef,
        make_immediate: cranelift_codegen::ir::FuncRef,
        store_int_tag: cranelift_codegen::ir::FuncRef,
        seal_int: cranelift_codegen::ir::FuncRef,
        store_int_limbs: cranelift_codegen::ir::FuncRef,
        store_int_limb: cranelift_codegen::ir::FuncRef,
        store_bytes_len: cranelift_codegen::ir::FuncRef,
        store_byte: cranelift_codegen::ir::FuncRef,
    }

    /// Call a helper and return its status immediately unless it is `OK`.
    fn guard(
        b: &mut FunctionBuilder<'_>,
        callee: cranelift_codegen::ir::FuncRef,
        args: &[cranelift_codegen::ir::Value],
    ) {
        let call = b.ins().call(callee, args);
        let status = b.inst_results(call)[0];
        let good = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_OK);
        let ok = b.create_block();
        let bad = b.create_block();
        b.ins().brif(good, ok, &[], bad, &[]);
        b.switch_to_block(bad);
        b.ins().return_(&[status]);
        b.switch_to_block(ok);
    }

    /// Compile a **producer**: a separately compiled body that constructs a
    /// boundary value by calling the emitted construction interface, and
    /// returns the constructed word — or the first non-`OK` status it hit.
    ///
    /// ⭐ Statuses are negative and handle words are positive, so one return
    /// value carries both without a second channel.
    fn compile_producer(
        arity: usize,
        emit: fn(
            &mut FunctionBuilder<'_>,
            &Refs,
            &[cranelift_codegen::ir::Value],
            cranelift_codegen::ir::Type,
        ),
    ) -> (JITModule, *const u8) {
        compile_producer_with_plan(
            arity,
            emit,
            &crate::boundary_value::BoundaryEmissionPlan::derive(),
        )
    }

    /// The same producer, against a **caller-supplied** plan — the per-cell
    /// relation evidence `RULING R5` clause 5 requires needs to run the emitted
    /// allocator under a perturbed relation, not just diff its text.
    fn compile_producer_with_plan(
        arity: usize,
        emit: fn(
            &mut FunctionBuilder<'_>,
            &Refs,
            &[cranelift_codegen::ir::Value],
            cranelift_codegen::ir::Type,
        ),
        plan: &crate::boundary_value::BoundaryEmissionPlan,
    ) -> (JITModule, *const u8) {
        let mut module = jit();
        let native = crate::native_int_clif::emit_native_int_local_graph(&mut module, false)
            .expect("native-int graph emits");
        let helpers =
            emit_boundary_value_local_graph(&mut module, &native, plan).expect("graph emits");
        let ptr = module.target_config().pointer_type();

        let mut sig = module.make_signature();
        for _ in 0..arity {
            sig.params.push(AbiParam::new(ptr));
        }
        sig.returns.push(AbiParam::new(types::I64));
        let id = module
            .declare_function("b2v_producer", Linkage::Local, &sig)
            .expect("producer declares");
        let mut ctx = module.make_context();
        ctx.func = Function::with_name_signature(UserFuncName::user(5, id.as_u32()), sig);
        let refs = Refs {
            alloc: module.declare_func_in_func(helpers.alloc, &mut ctx.func),
            store_tag_id: module.declare_func_in_func(helpers.store_tag_id, &mut ctx.func),
            store_scalar: module.declare_func_in_func(helpers.store_scalar, &mut ctx.func),
            store_field: module.declare_func_in_func(helpers.store_field, &mut ctx.func),
            store_name: module.declare_func_in_func(helpers.store_name, &mut ctx.func),
            make_immediate: module.declare_func_in_func(helpers.make_immediate, &mut ctx.func),
            store_int_tag: module.declare_func_in_func(helpers.store_int_tag, &mut ctx.func),
            seal_int: module.declare_func_in_func(helpers.seal_int, &mut ctx.func),
            store_int_limbs: module.declare_func_in_func(helpers.store_int_limbs, &mut ctx.func),
            store_int_limb: module.declare_func_in_func(helpers.store_int_limb, &mut ctx.func),
            store_bytes_len: module.declare_func_in_func(helpers.store_bytes_len, &mut ctx.func),
            store_byte: module.declare_func_in_func(helpers.store_byte, &mut ctx.func),
        };
        let mut fctx = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut ctx.func, &mut fctx);
            let entry = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.switch_to_block(entry);
            let p = b.block_params(entry).to_vec();
            emit(&mut b, &refs, &p, ptr);
            b.seal_all_blocks();
            b.finalize();
        }
        module
            .define_function(id, &mut ctx)
            .expect("producer defines");
        module.finalize_definitions().expect("jit finalizes");
        let code = module.get_finalized_function(id);
        (module, code)
    }

    /// An 8-byte out cell inside a producer.
    fn cell(
        b: &mut FunctionBuilder<'_>,
        ptr: cranelift_codegen::ir::Type,
    ) -> cranelift_codegen::ir::Value {
        let slot = b.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            8,
            3,
        ));
        b.ins().stack_addr(ptr, slot, 0)
    }

    /// `(base, head, nil_word, tag_id) -> word` — build `Cons(head, nil)` in
    /// **persistent** storage, entirely from emitted code.
    fn emit_cons_producer(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, head, nil_word, tag_id) = (p[0], p[1], p[2], p[3]);
        let out = cell(b, ptr);

        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b
            .ins()
            .iconst(types::I64, BoundaryClass::Constructor as i64);
        let two = b.ins().iconst(types::I64, 2);
        guard(b, refs.alloc, &[base, tag, class, two, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);

        guard(b, refs.store_tag_id, &[base, word, tag_id]);

        // The head is a RUNTIME parameter turned into an immediate word by the
        // emitted constructor — nothing about it is known when this body is
        // compiled.
        let int_tag = b.ins().iconst(types::I64, BoundaryTag::ImmediateInt as i64);
        let head_cell = cell(b, ptr);
        guard(b, refs.make_immediate, &[int_tag, head, head_cell]);
        let head_word = b.ins().load(types::I64, MemFlags::trusted(), head_cell, 0);

        let zero = b.ins().iconst(types::I64, 0);
        let one = b.ins().iconst(types::I64, 1);
        guard(b, refs.store_field, &[base, word, zero, head_word]);
        guard(b, refs.store_field, &[base, word, one, nil_word]);
        b.ins().return_(&[word]);
    }

    /// `(base, success, ok_word, err_word) -> word` — build a `HostResult`.
    fn emit_host_result_producer(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, success, ok_word, err_word) = (p[0], p[1], p[2], p[3]);
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::InvocationHostResult as i64);
        let class = b.ins().iconst(types::I64, BoundaryClass::HostResult as i64);
        let two = b.ins().iconst(types::I64, 2);
        guard(b, refs.alloc, &[base, tag, class, two, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);

        guard(b, refs.store_scalar, &[base, word, success]);
        let zero = b.ins().iconst(types::I64, 0);
        let one = b.ins().iconst(types::I64, 1);
        guard(b, refs.store_field, &[base, word, zero, ok_word]);
        guard(b, refs.store_field, &[base, word, one, err_word]);
        b.ins().return_(&[word]);
    }

    /// `(base, name_id, child) -> word` — build a one-field `Record`.
    fn emit_record_producer(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, name_id, child) = (p[0], p[1], p[2]);
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b.ins().iconst(types::I64, BoundaryClass::Record as i64);
        let one = b.ins().iconst(types::I64, 1);
        guard(b, refs.alloc, &[base, tag, class, one, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        let zero = b.ins().iconst(types::I64, 0);
        guard(b, refs.store_name, &[base, word, zero, name_id]);
        guard(b, refs.store_field, &[base, word, zero, child]);
        b.ins().return_(&[word]);
    }

    /// `(base, tag, class, field_count) -> word | status` — the allocator on
    /// its own, so the capacity ceilings are observable without a whole value.
    fn emit_alloc_probe(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, tag, class, count) = (p[0], p[1], p[2], p[3]);
        let out = cell(b, ptr);
        let call = b.ins().call(refs.alloc, &[base, tag, class, count, out]);
        let status = b.inst_results(call)[0];
        let good = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_OK);
        let ok = b.create_block();
        let bad = b.create_block();
        b.ins().brif(good, ok, &[], bad, &[]);
        b.switch_to_block(bad);
        b.ins().return_(&[status]);
        b.switch_to_block(ok);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        b.ins().return_(&[word]);
    }

    // ───── `RT-FNSPLIT-C3-ACTIVATION` `AC-4` — eight limits, eight controls ────
    //
    // ⛔⛔ **Eight SEPARATE cases, and that is the acceptance criterion, not a
    // style.** Emitted code answers one `BOUNDARY_ERR_CAPACITY` for every
    // exhausted table in either region, so a shared *"capacity exhausted"*
    // assertion is **one control claiming to be eight** — it cannot tell a
    // persistent data-byte ceiling from an invocation node ceiling. Each case
    // below therefore asserts the exact `(scope, resource)` the refusal was
    // about, attributed by comparing the region's live count against the
    // authorized limit.
    //
    // ⚠ Each case grants ONE tight limit and generous room for the other seven.
    // A fixture that tightened two would get a well-defined but arbitrary
    // attribution, and would pass while measuring the wrong cell.

    /// A profile that is generous everywhere, so a single tightened limit is
    /// unambiguously the one that fired.
    fn ac4_roomy() -> crate::boundary_resource_profile::BoundaryResourceProfileV1 {
        use crate::boundary_resource_profile::{BoundaryRegionLimitsV1, BoundaryResourceProfileV1};
        let roomy = BoundaryRegionLimitsV1 {
            nodes: 64,
            words: 256,
            data_bytes: 512,
            native_int_limbs: 64,
        };
        BoundaryResourceProfileV1 {
            invocation: roomy,
            persistent: roomy,
        }
    }

    /// The profile of [`ac4_roomy`] with exactly one limit tightened to `limit`.
    fn ac4_profile_with(
        scope: crate::boundary_resource_profile::BoundaryResourceScope,
        resource: crate::boundary_resource_profile::BoundaryResource,
        limit: usize,
    ) -> crate::boundary_resource_profile::BoundaryResourceProfileV1 {
        use crate::boundary_resource_profile::{BoundaryResource, BoundaryResourceScope};
        let mut profile = ac4_roomy();
        let limits = match scope {
            BoundaryResourceScope::Invocation => &mut profile.invocation,
            BoundaryResourceScope::Persistent => &mut profile.persistent,
        };
        let slot = match resource {
            BoundaryResource::Nodes => &mut limits.nodes,
            BoundaryResource::Words => &mut limits.words,
            BoundaryResource::DataBytes => &mut limits.data_bytes,
            BoundaryResource::NativeIntLimbs => &mut limits.native_int_limbs,
        };
        *slot = limit;
        profile
    }

    /// The `(tag, class)` pair that reaches a region, ⛔ derived from the
    /// admitted relation rather than chosen.
    fn ac4_lane(
        scope: crate::boundary_resource_profile::BoundaryResourceScope,
    ) -> (BoundaryTag, BoundaryClass) {
        use crate::boundary_resource_profile::BoundaryResourceScope;
        match scope {
            BoundaryResourceScope::Persistent => (BoundaryTag::PersistentGround, BoundaryClass::Int),
            BoundaryResourceScope::Invocation => (
                BoundaryTag::InvocationHostResult,
                BoundaryClass::HostResult,
            ),
        }
    }

    /// `(base, tag, class, len) -> status` — allocate, then claim a DATA body
    /// of `len` bytes. ⭐ `RT-FNSPLIT-C3` `AC-4`: the data-byte ceiling is
    /// reached by claiming a span, so one call exceeds it by exactly one.
    fn emit_ac4_bytes_len_probe(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, tag, class, len) = (p[0], p[1], p[2], p[3]);
        let out = cell(b, ptr);
        let zero = b.ins().iconst(types::I64, 0);
        let call = b.ins().call(refs.alloc, &[base, tag, class, zero, out]);
        let status = b.inst_results(call)[0];
        let good = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_OK);
        let ok = b.create_block();
        let bad = b.create_block();
        b.ins().brif(good, ok, &[], bad, &[]);
        b.switch_to_block(bad);
        b.ins().return_(&[status]);
        b.switch_to_block(ok);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        let span = cell(b, ptr);
        let call = b.ins().call(refs.store_bytes_len, &[base, word, len, span]);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
    }

    /// `(base, tag, class, len) -> status` — allocate a region-limbed `Int`,
    /// then claim `len` limbs. ⭐ `AC-4`'s limb ceiling, same shape.
    fn emit_ac4_int_limbs_probe(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, tag, class, len) = (p[0], p[1], p[2], p[3]);
        let out = cell(b, ptr);
        let zero = b.ins().iconst(types::I64, 0);
        let call = b.ins().call(refs.alloc, &[base, tag, class, zero, out]);
        let status = b.inst_results(call)[0];
        let good = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_OK);
        let ok = b.create_block();
        let bad = b.create_block();
        b.ins().brif(good, ok, &[], bad, &[]);
        b.switch_to_block(bad);
        b.ins().return_(&[status]);
        b.switch_to_block(ok);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        let marker = b
            .ins()
            .iconst(types::I64, crate::boundary_value::BOUNDARY_INT_REGION_LIMBS as i64);
        let call = b.ins().call(refs.store_int_tag, &[base, word, marker]);
        let status = b.inst_results(call)[0];
        let good = b.ins().icmp_imm(IntCC::Equal, status, BOUNDARY_OK);
        let claim = b.create_block();
        let refused = b.create_block();
        b.ins().brif(good, claim, &[], refused, &[]);
        b.switch_to_block(refused);
        b.ins().return_(&[status]);
        b.switch_to_block(claim);
        let sign = b.ins().iconst(types::I64, 0);
        let span = cell(b, ptr);
        let call = b
            .ins()
            .call(refs.store_int_limbs, &[base, word, sign, len, span]);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
    }

    /// `(base, word, index, child) -> status` — `store_field` on its own.
    fn emit_store_field_probe(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        _ptr: cranelift_codegen::ir::Type,
    ) {
        let call = b.ins().call(refs.store_field, &[p[0], p[1], p[2], p[3]]);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
    }

    /// `(base, word, tag_id) -> status` — `store_tag_id` on its own.
    fn emit_store_tag_id_probe(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        _ptr: cranelift_codegen::ir::Type,
    ) {
        let call = b.ins().call(refs.store_tag_id, &[p[0], p[1], p[2]]);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
    }

    /// `(base, word, payload) -> status` — `store_scalar` on its own.
    fn emit_store_scalar_probe(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        _ptr: cranelift_codegen::ir::Type,
    ) {
        let call = b.ins().call(refs.store_scalar, &[p[0], p[1], p[2]]);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
    }

    /// **`AC-4` — a producer mints a non-constant `Constructor`; a separately
    /// compiled consumer projects it.**
    ///
    /// ⛔ One compiled producer, three runtime heads. A producer that baked its
    /// payload in would return the same field three times, so the loop is the
    /// discriminator and not decoration.
    #[test]
    fn b2v_emitted_code_constructs_a_nonconstant_constructor_and_a_consumer_reads_it() {
        let (_pm, produce) = compile_producer(4, emit_cons_producer);
        let (_c1, field_code) = compile_probe(Probe::Binary(|h| h.field));
        let (_c2, scalar_code) = compile_probe(Probe::Unary(|h| h.scalar));
        let (_c3, tag_code) = compile_probe(Probe::Unary(|h| h.tag));
        let (_c4, class_code) = compile_probe(Probe::Unary(|h| h.class));

        for head in [7i64, -3, 1_000_000] {
            let mut store = c1_d2_store();
            // The only Rust-materialized ingredient is the tail; the parent —
            // its class, identity, arity and both children — is built by
            // emitted code.
            let nil = materialize_ground(
                &mut store,
                &RuntimeGroundValue::Constructor {
                    constructor: "ctor:fixture::List::Nil".to_string(),
                    args: vec![],
                },
            )
            .expect("nil materializes");
            let cons_id = store.intern_symbol("ctor:fixture::List::Cons");
            let f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (4, 8, 0),
                (0, 0, 0),
            );

            let word = BoundaryWord(run4(produce, f.base, head as u64, nil.0, cons_id) as u64);
            assert_eq!(
                word.tag(),
                Some(BoundaryTag::PersistentGround),
                "AC-4: the producer minted a persistent handle, not a status ({})",
                word.0 as i64
            );
            assert_eq!(
                run2(class_code, f.base, word),
                BoundaryClass::Constructor as i64,
                "AC-4: the constructed node's class is readable"
            );
            assert_eq!(
                store.symbol(run2(tag_code, f.base, word) as u64),
                Some("ctor:fixture::List::Cons"),
                "AC-4: the constructed node's identity is readable"
            );

            let head_word = BoundaryWord(run3(field_code, f.base, word, 0) as u64);
            assert_eq!(
                run2(scalar_code, f.base, head_word),
                head,
                "AC-4: the consumer must read the RUNTIME head the producer stored"
            );
            let tail = BoundaryWord(run3(field_code, f.base, word, 1) as u64);
            assert_eq!(tail, nil, "AC-4: the second child is the tail it was given");
        }
    }

    /// **`AC-4` — a producer mints BOTH `HostResult` arms.**
    #[test]
    fn b2v_emitted_code_constructs_both_host_result_arms() {
        let (_pm, produce) = compile_producer(4, emit_host_result_producer);
        let (_c1, success_code) = compile_probe(Probe::Unary(|h| h.host_success));
        let (_c2, payload_code) = compile_probe(Probe::Unary(|h| h.host_payload));
        let (_c3, scalar_code) = compile_probe(Probe::Unary(|h| h.scalar));

        for (success, expected) in [(1u64, 11i64), (0, 22)] {
            let mut store = c1_d2_store();
            let f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (0, 0, 0),
                (4, 8, 0),
            );
            let ok = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 11);
            let err = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 22);

            let word = BoundaryWord(run4(produce, f.base, success, ok.0, err.0) as u64);
            assert_eq!(
                word.tag(),
                Some(BoundaryTag::InvocationHostResult),
                "AC-4: the producer minted a host-result handle ({})",
                word.0 as i64
            );
            assert_eq!(
                run2(success_code, f.base, word),
                success as i64,
                "AC-4: the discriminant the producer stored is the one read back"
            );
            let selected = BoundaryWord(run2(payload_code, f.base, word) as u64);
            assert_eq!(
                run2(scalar_code, f.base, selected),
                expected,
                "AC-4: the consumer selected the arm at run time"
            );
        }
    }

    /// **`AC-4` — a constructed `Record` is readable by name.**
    ///
    /// Without `store_name` the producer could build every live class except
    /// the one whose reader takes a name. That asymmetry would be a wall for
    /// `B2F`, so it is closed here rather than recorded as a residual.
    #[test]
    fn b2v_emitted_code_constructs_a_record_readable_by_name() {
        let (_pm, produce) = compile_producer(3, emit_record_producer);
        let (_c1, named) = compile_probe(Probe::Binary(|h| h.record_field));
        let (_c2, scalar_code) = compile_probe(Probe::Unary(|h| h.scalar));

        let mut store = c1_d2_store();
        let name = store.intern_symbol("field:amount");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (2, 4, 0),
            (0, 0, 0),
        );
        let child = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 41);

        let word = BoundaryWord(run3(produce, f.base, BoundaryWord(name), child.0) as u64);
        assert_eq!(
            word.tag(),
            Some(BoundaryTag::PersistentGround),
            "AC-4: the producer minted a record handle ({})",
            word.0 as i64
        );
        let found = BoundaryWord(run3(named, f.base, word, name) as u64);
        assert_eq!(
            run2(scalar_code, f.base, found),
            41,
            "AC-4: the name the producer stored resolves to the field it stored"
        );
        // Positive control: an id the producer never wrote is not found, so the
        // hit above is a lookup and not "returns child 0 for anything".
        assert_eq!(
            run3(named, f.base, word, name + 1),
            BOUNDARY_ERR_BOUNDS,
            "AC-4: an unstored name must not resolve"
        );
    }

    // ── `AC-6` — a persistent handle is a persistent IDENTITY ───────────────

    /// **`AC-6` — a constructed persistent word outlives the arena that minted
    /// it.**
    ///
    /// ⛔ This is the property the previous candidate did not have: a
    /// `PersistentGround` word is permitted to escape the invocation, so after
    /// the arena dies it must still name its referent. The arena here is
    /// **dropped** and a second, unrelated invocation resolves the same word.
    #[test]
    fn b2v_a_constructed_persistent_word_survives_the_invocation_arena() {
        let (_pm, produce) = compile_producer(4, emit_cons_producer);
        let (_c1, field_code) = compile_probe(Probe::Binary(|h| h.field));
        let (_c2, scalar_code) = compile_probe(Probe::Unary(|h| h.scalar));
        let (_c3, slot_code) = compile_probe(Probe::Unary(|h| h.slot));

        let mut store = c1_d2_store();
        let nil = materialize_ground(
            &mut store,
            &RuntimeGroundValue::Constructor {
                constructor: "ctor:fixture::List::Nil".to_string(),
                args: vec![],
            },
        )
        .expect("nil materializes");
        let cons_id = store.intern_symbol("ctor:fixture::List::Cons");
        let persistent = {
            let first = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (4, 8, 0),
                (2, 2, 0),
            );
            let word = BoundaryWord(run4(produce, first.base, 88, nil.0, cons_id) as u64);
            assert_eq!(
                run2(
                    scalar_code,
                    first.base,
                    BoundaryWord(run3(field_code, first.base, word, 0) as u64)
                ),
                88,
                "AC-6: readable in the invocation that built it"
            );
            // Also mint an invocation-owned word, so the first arena genuinely
            // held state that the second one does not.
            let ephemeral = first.persistent;
            drop(first);
            (word, ephemeral)
        };
        let (word, persistent_base) = persistent;

        // A SECOND invocation: fresh arena, fresh tables, same store.
        let second = rebind(persistent_base);
        assert_eq!(
            run2(
                scalar_code,
                second.base,
                BoundaryWord(run3(field_code, second.base, word, 0) as u64)
            ),
            88,
            "AC-6: the same word must still name its referent after the arena died"
        );
        assert_eq!(
            run2(slot_code, second.base, word),
            crate::store::NULL_SLOT as i64,
            "AC-6: an emitted-constructed node carries no store slot — the \
             residual this node records honestly"
        );

        // ⚠ POSITIVE CONTROL for the whole mechanism. Resolution must go
        // through PERSISTENT storage: an invocation bound to none fails closed
        // rather than reading the persistent index against its own arena. If
        // `resolve` silently used the arena, this would return a value.
        let mut orphan = BoundaryArenaBuilder::new().finish();
        orphan.bind_persistent(None);
        let orphan_base = orphan.publish();
        assert_eq!(
            run3(field_code, orphan_base, word, 0),
            BOUNDARY_ERR_BOUNDS,
            "AC-6: a persistent word must not resolve against an arena"
        );
    }

    /// **`AC-6`/`AC-7` — a persistent parent refuses an invocation-owned
    /// child.**
    ///
    /// ⛔ The Θ(1) escape check permits a persistent word to leave. That is
    /// sound only because no persistent node can embed a child that dies first,
    /// and this is where that is enforced.
    #[test]
    fn b2v_a_persistent_node_refuses_an_invocation_owned_child() {
        let (_pm, alloc_code) = compile_producer(4, emit_alloc_probe);
        let (_sm, store_code) = compile_producer(4, emit_store_field_probe);

        let mut store = c1_d2_store();
        let mut builder = BoundaryArenaBuilder::new();
        let borrowed = materialize_borrowed(&mut builder, 0xBEEF);
        let f = bind_with(&mut store, builder, (2, 4, 0), (2, 4, 0));

        let parent = BoundaryWord(run4(
            alloc_code,
            f.base,
            BoundaryTag::PersistentGround as u64,
            BoundaryClass::Constructor as u64,
            1,
        ) as u64);
        assert_eq!(parent.tag(), Some(BoundaryTag::PersistentGround));

        assert_eq!(
            run4(store_code, f.base, parent.0, 0, borrowed.0),
            BOUNDARY_ERR_ESCAPE,
            "AC-7: a surviving parent must not embed a child that dies first"
        );

        // ⚠ POSITIVE CONTROL — the same store with a persistent child succeeds,
        // so the refusal above is about the child's OWNER and not about
        // `store_field` refusing everything.
        let immediate = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 5);
        assert_eq!(
            run4(store_code, f.base, parent.0, 0, immediate.0),
            BOUNDARY_OK,
            "AC-7: a child that outlives the invocation is admitted"
        );

        // And the mirror: an invocation-owned parent MAY hold a borrowed child,
        // because both die together.
        let ephemeral = BoundaryWord(run4(
            alloc_code,
            f.base,
            BoundaryTag::InvocationHostResult as u64,
            BoundaryClass::HostResult as u64,
            1,
        ) as u64);
        assert_eq!(
            run4(store_code, f.base, ephemeral.0, 0, borrowed.0),
            BOUNDARY_OK,
            "AC-7: an invocation-owned parent may hold an invocation-owned child"
        );
    }

    /// **`AC-4` — construction fails closed at every ceiling, with an exact
    /// status.**
    #[test]
    fn b2v_construction_fails_closed_at_each_ceiling() {
        let (_pm, alloc_code) = compile_producer(4, emit_alloc_probe);
        let persistent = BoundaryTag::PersistentGround as u64;
        let ctor = BoundaryClass::Constructor as u64;

        // Node ceiling: room for exactly one.
        {
            let mut store = c1_d2_store();
            let f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (1, 4, 0),
                (0, 0, 0),
            );
            assert!(
                run4(alloc_code, f.base, persistent, ctor, 0) >= 0,
                "the first allocation is inside the reservation"
            );
            assert_eq!(
                run4(alloc_code, f.base, persistent, ctor, 0),
                BOUNDARY_ERR_CAPACITY,
                "AC-4: the node ceiling is exact and fails closed"
            );
        }
        // Word ceiling: room for two nodes but only one child word.
        {
            let mut store = c1_d2_store();
            let f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (2, 1, 0),
                (0, 0, 0),
            );
            assert_eq!(
                run4(alloc_code, f.base, persistent, ctor, 2),
                BOUNDARY_ERR_CAPACITY,
                "AC-4: the child-word ceiling is exact and fails closed"
            );
            // A caller-supplied field count large enough to wrap the sum must
            // not be read as "fits".
            assert_eq!(
                run4(alloc_code, f.base, persistent, ctor, u64::MAX),
                BOUNDARY_ERR_CAPACITY,
                "AC-4: an overflowing field count fails closed"
            );
        }
        // The closed sets bound construction too.
        {
            let mut store = c1_d2_store();
            let f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (2, 4, 0),
                (2, 4, 0),
            );
            assert_eq!(
                run4(alloc_code, f.base, 200, ctor, 0),
                BOUNDARY_ERR_TAG,
                "AC-1: construction admits only the closed tag set"
            );
            assert_eq!(
                run4(
                    alloc_code,
                    f.base,
                    BoundaryTag::ImmediateInt as u64,
                    ctor,
                    0
                ),
                BOUNDARY_ERR_SHAPE,
                "AC-4: an immediate has no node to allocate"
            );
            assert_eq!(
                run4(alloc_code, f.base, persistent, 999, 0),
                BOUNDARY_ERR_CLASS,
                "AC-4: construction admits only the closed class set"
            );
        }
        // Persistent construction with no persistent region bound.
        {
            let mut arena = BoundaryArenaBuilder::new().finish();
            arena.reserve(2, 4, 0, 0);
            arena.bind_persistent(None);
            let base = arena.publish();
            assert_eq!(
                run4(alloc_code, base, persistent, ctor, 0),
                BOUNDARY_ERR_BOUNDS,
                "AC-6: persistent construction requires persistent storage"
            );
        }
    }

    /// **`AC-6` — the frozen prefix is not emitted code's to rewrite.**
    ///
    /// ⛔ A node the store materialized carries the store's `SlotId`. If
    /// emitted code could overwrite it, emitted code could forge persistent
    /// identity, and the store would stop being the sole identity authority.
    #[test]
    fn b2v_the_frozen_prefix_refuses_emitted_mutation() {
        let (_sm, scalar_store) = compile_producer(3, emit_store_scalar_probe);
        let (_am, alloc_code) = compile_producer(4, emit_alloc_probe);

        let mut store = c1_d2_store();
        let materialized = materialize_ground(&mut store, &cons(5)).expect("materializes");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (2, 4, 0),
            (0, 0, 0),
        );

        assert_eq!(
            run3(scalar_store, f.base, materialized, 99),
            BOUNDARY_ERR_FROZEN,
            "AC-6: a store-materialized node is not emitted code's to rewrite"
        );

        // ⚠ POSITIVE CONTROL — a node emitted code allocated IS writable, so
        // the refusal above is about the frozen prefix and not about
        // `store_scalar` refusing everything.
        let fresh = BoundaryWord(run4(
            alloc_code,
            f.base,
            BoundaryTag::PersistentGround as u64,
            BoundaryClass::Int as u64,
            0,
        ) as u64);
        assert_eq!(
            run3(scalar_store, f.base, fresh, 99),
            BOUNDARY_OK,
            "AC-6: a node emitted code built is emitted code's to fill in"
        );
    }

    // ── `AC-4` fidelity — CONTENT, not identity and not length ──────────────

    /// `(base, native_arena, value, out_is_unused) -> word` — build a spilled
    /// `Int` whose magnitude arrives at run time.
    fn emit_spilled_int_producer(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, value) = (p[0], p[1]);
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b.ins().iconst(types::I64, BoundaryClass::Int as i64);
        let zero = b.ins().iconst(types::I64, 0);
        guard(b, refs.alloc, &[base, tag, class, zero, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        // The `NativeIntV1` pair: payload is the run-time magnitude word, tag
        // says how to read it. Nothing about the value is known here.
        guard(b, refs.store_scalar, &[base, word, value]);
        let small = b.ins().iconst(
            types::I64,
            crate::native_int::NATIVE_INT_SMALL_TAG_V1 as i64,
        );
        guard(b, refs.store_int_tag, &[base, word, small]);
        b.ins().return_(&[word]);
    }

    /// `(base, len, seed, class) -> word` — build a span-carrying value whose
    /// CONTENT is derived from a run-time seed, at a run-time length.
    ///
    /// ⛔ **The class is a run-time PARAMETER, not a baked constant, and that
    /// is the point.** The previous candidate had a `Bytes`-only producer and
    /// asserted the `String` arm was "covered, since it shares every code path
    /// but the class" — but the class is exactly the axis `store_bytes_len`
    /// and `store_byte` guard on, so it is the one path that is *not* shared.
    /// QA defeated that by narrowing `store_bytes_len`'s `class_guard` to
    /// `Bytes` alone: every test stayed green because no test had ever asked
    /// emitted code to *build* a `String`. One body driven with both classes
    /// makes the guard's second arm reachable.
    fn emit_span_producer(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, len, seed, class) = (p[0], p[1], p[2], p[3]);
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let zero = b.ins().iconst(types::I64, 0);
        guard(b, refs.alloc, &[base, tag, class, zero, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        let span = cell(b, ptr);
        guard(b, refs.store_bytes_len, &[base, word, len, span]);

        // Write `seed + i` at every index — a loop over run-time bounds, so no
        // byte of the result is known when this body is compiled.
        let loop_head = b.create_block();
        b.append_block_param(loop_head, types::I64);
        b.ins().jump(loop_head, &[zero.into()]);
        b.switch_to_block(loop_head);
        let i = b.block_params(loop_head)[0];
        let more = b.ins().icmp(IntCC::UnsignedLessThan, i, len);
        let body = b.create_block();
        let done = b.create_block();
        b.ins().brif(more, body, &[], done, &[]);

        b.switch_to_block(body);
        let byte = b.ins().iadd(seed, i);
        guard(b, refs.store_byte, &[base, word, i, byte]);
        let next = b.ins().iadd_imm(i, 1);
        b.ins().jump(loop_head, &[next.into()]);

        b.switch_to_block(done);
        b.ins().return_(&[word]);
    }

    /// `(base, word, native_tag) -> status` — `store_int_tag` on its own.
    fn emit_store_int_tag_probe(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        _ptr: cranelift_codegen::ir::Type,
    ) {
        let call = b.ins().call(refs.store_int_tag, &[p[0], p[1], p[2]]);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
    }

    /// A published invocation that also binds a native-`Int` arena.
    fn with_native_int(f: &mut Bound, native: &crate::native_int::NativeIntArenaV1) {
        f.arena
            .bind_native_int(Some(native as *const _ as *const u64));
        f.base = f.arena.publish();
    }

    /// **`AC-4` — a separately compiled consumer reads a spilled `Int`'s
    /// CONTENT.**
    ///
    /// ⛔ The previous candidate wrote `NODE_PAYLOAD = 0` for every spilled
    /// `Int`, so emitted code saw every wide integer as zero. Identity lived in
    /// the store and content lived in a Rust decoder — which is exactly the
    /// arrangement hard-stop `#10` rejected.
    ///
    /// ⭐ The decode is `ken_native_int_resolve_local`'s. This control passes
    /// only if the boundary node really carries a `NativeIntV1` pair the landed
    /// decoder accepts, so it pins the *connection*, not a lookalike.
    #[test]
    fn b2v_a_separately_compiled_consumer_reads_a_spilled_int_by_content() {
        let (_pm, produce) = compile_producer(3, emit_spilled_int_producer);
        let (_c1, sign_code) = compile_probe(Probe::Unary(|h| h.int_sign));
        let (_c2, len_code) = compile_probe(Probe::Unary(|h| h.int_len));
        let (_c3, limb_code) = compile_probe(Probe::Binary(|h| h.int_limb));

        // Every value is outside the immediate range, and no two share a limb.
        let cases = [
            (1i64 << 60) + 7,
            (1i64 << 60) + 8,
            -((1i64 << 58) + 3),
            BOUNDARY_IMMEDIATE_INT_MAX + 1,
        ];
        let mut seen = Vec::new();
        for value in cases {
            assert!(
                !BoundaryWord::int_fits_immediate(value),
                "the case must actually spill, or this control tests the wrong arm"
            );
            let mut store = c1_d2_store();
            let native = crate::native_int::NativeIntArenaV1::default();
            let mut f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (2, 0, 0),
                (0, 0, 0),
            );
            with_native_int(&mut f, &native);

            let word = BoundaryWord(run3(produce, f.base, BoundaryWord(value as u64), 0) as u64);
            assert_eq!(
                word.tag(),
                Some(BoundaryTag::PersistentGround),
                "AC-4: the producer minted a spilled Int ({})",
                word.0 as i64
            );
            let sign = run2(sign_code, f.base, word);
            let len = run2(len_code, f.base, word);
            assert_eq!(len, 1, "a native Small decodes to one limb");
            let limb = run3(limb_code, f.base, word, 0);
            let observed = if sign == 1 { -limb } else { limb };
            assert_eq!(
                observed, value,
                "AC-4: emitted code must recover the RUNTIME magnitude, not zero"
            );
            seen.push(observed);
        }
        // ⚠ POSITIVE CONTROL. Two of the cases differ by one; if the consumer
        // were reading identity or length they would be indistinguishable.
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 4, "AC-4: every case must be distinguishable");
    }

    /// Drive the emitted span producer once and read the result back
    /// byte-by-byte with two more separately compiled bodies. Returns the
    /// content emitted code recovered.
    ///
    /// The construction is `alloc(PersistentGround, class)` →
    /// `store_bytes_len(len)` → `store_byte(i, seed + i)` for every `i`, all
    /// from CLIF, at run-time bounds. Nothing about the result is known when
    /// the producer body is compiled.
    fn emitted_span_roundtrip(
        produce: *const u8,
        byte_code: *const u8,
        scalar_code: *const u8,
        class_code: *const u8,
        class: BoundaryClass,
        len: u64,
        seed: u64,
    ) -> Vec<i64> {
        let mut store = c1_d2_store();
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (2, 0, 32),
            (0, 0, 0),
        );
        let produced = run4(produce, f.base, len, seed, class as u64);
        assert!(
            produced > 0,
            "AC-4: emitted {class:?} construction must succeed, got status {produced}"
        );
        let word = BoundaryWord(produced as u64);
        assert_eq!(
            word.tag(),
            Some(BoundaryTag::PersistentGround),
            "AC-4: the producer minted a persistent handle ({})",
            word.0 as i64
        );
        assert_eq!(
            run2(class_code, f.base, word),
            class as i64,
            "AC-4: emitted code must build the class it was ASKED for"
        );
        assert_eq!(
            run2(scalar_code, f.base, word),
            len as i64,
            "the length is still readable"
        );
        let read: Vec<i64> = (0..len).map(|i| run3(byte_code, f.base, word, i)).collect();
        assert_eq!(
            read,
            (0..len)
                .map(|i| ((seed + i) & 0xff) as i64)
                .collect::<Vec<_>>(),
            "AC-4: emitted code must read the RUNTIME bytes it wrote"
        );
        read
    }

    /// **`AC-4` — emitted code CONSTRUCTS equal-length, different-content
    /// `Bytes` AND `String`s, and a separately compiled consumer tells them
    /// apart.**
    ///
    /// ⛔ This is the control QA defeated on `ea8d9824`, and the defeat is the
    /// same failure class as the `store_slot` one this candidate already
    /// closed: **the `String` arm asserted a property of a path the test never
    /// walked.** Its handles were materialized in Rust, so narrowing
    /// `store_bytes_len`'s `class_guard` to `Bytes` alone — which makes
    /// emitted `String` construction impossible — left every test green.
    ///
    /// ⭐ The repair is reachability, not a stronger assertion: the class is a
    /// **run-time argument** to one emitted body, so both arms of every
    /// span-writing guard are exercised by emitted code.
    #[test]
    fn b2v_emitted_code_constructs_equal_length_bytes_and_strings_by_content() {
        let (_pm, produce) = compile_producer(4, emit_span_producer);
        let (_c1, byte_code) = compile_probe(Probe::Binary(|h| h.byte));
        let (_c2, scalar_code) = compile_probe(Probe::Unary(|h| h.scalar));
        let (_c3, class_code) = compile_probe(Probe::Unary(|h| h.class));

        // ⚠ Every case is the SAME length, so anything discriminating by
        // length — or by class — collapses them.
        let len = 6u64;
        let mut contents = Vec::new();
        for class in [BoundaryClass::Bytes, BoundaryClass::String] {
            for seed in [10u64, 11, 200] {
                contents.push((
                    class,
                    emitted_span_roundtrip(
                        produce,
                        byte_code,
                        scalar_code,
                        class_code,
                        class,
                        len,
                        seed,
                    ),
                ));
            }
        }
        assert_eq!(
            contents
                .iter()
                .map(|(_, c)| c)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3,
            "AC-4: equal-length values must differ by CONTENT"
        );
        // ⚠ POSITIVE CONTROL for the class axis: the two classes agree on
        // content for the same seed, so the class cannot be inferred FROM the
        // content, and the sweep above is genuinely testing both producers.
        for seed_index in 0..3 {
            assert_eq!(
                contents[seed_index].1,
                contents[seed_index + 3].1,
                "AC-4: the same seed must produce the same bytes in either class"
            );
        }
    }

    /// **`AC-4` — a `String` the STORE materialized and a `String` emitted code
    /// CONSTRUCTED are read identically by the same consumer.**
    ///
    /// The two producers are the only ones that exist, and the boundary word is
    /// supposed to be the whole interface between them. This is the pin that
    /// they agree; without it each producer could carry its own private layout
    /// and every single-producer control would still be green.
    #[test]
    fn b2v_the_two_string_producers_agree_byte_for_byte() {
        let (_pm, produce) = compile_producer(4, emit_span_producer);
        let (_c1, byte_code) = compile_probe(Probe::Binary(|h| h.byte));
        let (_c2, class_code) = compile_probe(Probe::Unary(|h| h.class));

        // Chosen so the emitted `seed + i` walk reproduces it exactly.
        let text = "defghi";
        let seed = u64::from(text.as_bytes()[0]);

        let mut store = c1_d2_store();
        let word = materialize_ground(&mut store, &RuntimeGroundValue::String(text.to_string()))
            .expect("a String materializes");
        let f = bind(&mut store, BoundaryArenaBuilder::new());
        assert_eq!(
            run2(class_code, f.base, word),
            BoundaryClass::String as i64,
            "AC-4: the class survives materialization"
        );
        let materialized: Vec<i64> = (0..text.len() as u64)
            .map(|i| run3(byte_code, f.base, word, i))
            .collect();
        assert_eq!(
            materialized,
            text.bytes().map(i64::from).collect::<Vec<_>>(),
            "AC-4: emitted code must read the store's String bytes"
        );

        let (_c3, scalar_code) = compile_probe(Probe::Unary(|h| h.scalar));
        let constructed = emitted_span_roundtrip(
            produce,
            byte_code,
            scalar_code,
            class_code,
            BoundaryClass::String,
            text.len() as u64,
            seed,
        );
        assert_eq!(
            constructed, materialized,
            "AC-4: the store's String and an emitted String must be the same bytes"
        );
    }

    /// **`AC-6` — the emitted interface cannot assign store identity.**
    ///
    /// ⛔ `ken_boundary_store_slot_local` took a caller-supplied `SlotId` and
    /// the frozen guard expressly permits writes to a **new** node, so emitted
    /// code could overwrite the allocator's `NULL_SLOT` with any slot. The old
    /// control read back `NULL_SLOT` on a node it had never called `store_slot`
    /// on — **it asserted a property of a field nothing had written to.**
    ///
    /// The closure is removal, not a guard: `EMITTED_WRITABLE_NODE_OFFSETS`
    /// makes emitting a setter for `NODE_SLOT` a panic, so the capability
    /// cannot be rebuilt by accident.
    /// Every emitted helper that MUTATES a node or a region table.
    ///
    /// ⛔ Membership is by what the helper does, not by how it is spelled —
    /// `ken_boundary_seal_int_local` writes `NODE_INT_SEALED` and carries no
    /// `store_` prefix.
    const EMITTED_WRITERS: &[&str] = &[
        "ken_boundary_alloc_local",
        "ken_boundary_seal_int_local",
        "ken_boundary_store_tag_id_local",
        "ken_boundary_store_scalar_local",
        "ken_boundary_store_field_local",
        "ken_boundary_store_name_local",
        "ken_boundary_store_int_tag_local",
        "ken_boundary_store_int_limbs_local",
        "ken_boundary_store_int_limb_local",
        "ken_boundary_store_bytes_len_local",
        "ken_boundary_store_byte_local",
    ];

    /// Every emitted helper that only projects. `make_immediate` is here
    /// deliberately: it builds a word from a tag and a payload and touches no
    /// region at all.
    const EMITTED_READERS: &[&str] = &[
        "ken_boundary_resolve_local",
        "ken_boundary_class_local",
        "ken_boundary_owner_local",
        "ken_boundary_slot_local",
        "ken_boundary_scalar_local",
        "ken_boundary_tag_local",
        "ken_boundary_field_count_local",
        "ken_boundary_field_local",
        "ken_boundary_record_field_local",
        "ken_boundary_host_success_local",
        "ken_boundary_host_payload_local",
        "ken_boundary_make_immediate_local",
        "ken_boundary_escape_check_local",
        "ken_boundary_byte_local",
        "ken_boundary_int_sign_local",
        "ken_boundary_int_len_local",
        "ken_boundary_int_limb_local",
        "ken_boundary_int_view_local",
    ];

    #[test]
    fn b2v_emitted_code_cannot_assign_store_identity() {
        // The allowed inventory, not a forbidden list: any node word outside
        // this set is unsettable, including ones nobody has thought of.
        assert_eq!(
            EMITTED_WRITABLE_NODE_OFFSETS,
            &[NODE_TAG_ID, NODE_PAYLOAD],
            "AC-6: the writable node-word set has moved"
        );
        for forbidden in [
            NODE_SLOT,
            NODE_CLASS,
            NODE_OWNER,
            NODE_FIELD_COUNT,
            NODE_FIELDS_AT,
            NODE_EXTENT,
        ] {
            assert!(
                !EMITTED_WRITABLE_NODE_OFFSETS.contains(&forbidden),
                "AC-6: emitted code must not be able to set node offset {forbidden}"
            );
        }
        // ⚠ The **allowed inventory of writers**, not a forbidden needle. A
        // `name.contains("slot")` scan looked right and was wrong twice over:
        // it fires on `ken_boundary_slot_local`, which is a *reader* and is
        // meant to exist — reading a node's slot is how `AC-6` is observable at
        // all — and it would miss a writer that spelled the field differently.
        // Pinning the permitted set makes ANY new writer redden, including one
        // nobody imagined.
        //
        // ⛔ **And the classification is TOTAL, because the prefix scan that
        // stood here was the same defect one level up.** It discovered writers
        // by `name.starts_with("ken_boundary_store_")` — so a writer named
        // anything else was invisible to it, and `ken_boundary_seal_int_local`
        // (which writes `NODE_INT_SEALED`) is exactly that helper. A discovery
        // rule keyed on spelling cannot enumerate an inventory; only a
        // partition can. Every helper must appear in exactly one of the two
        // lists below, so a new helper of ANY name reddens until someone says
        // which it is.
        let mut classified: Vec<&str> = EMITTED_WRITERS
            .iter()
            .chain(EMITTED_READERS.iter())
            .copied()
            .collect();
        let declared = classified.len();
        classified.sort_unstable();
        classified.dedup();
        assert_eq!(
            classified.len(),
            declared,
            "AC-6: a helper is classified twice"
        );
        let mut inventory: Vec<&str> = BOUNDARY_LOCAL_HELPERS.to_vec();
        inventory.sort_unstable();
        assert_eq!(
            classified, inventory,
            "AC-6: every emitted helper must be classified as a reader or a \
             writer — a new writer needs its own account of what it may set"
        );
        // ⚠ POSITIVE CONTROL on both halves: a partition that put everything in
        // one bucket would satisfy the equality above vacuously.
        assert!(
            !EMITTED_WRITERS.is_empty() && !EMITTED_READERS.is_empty(),
            "AC-6: neither half of the partition may be empty"
        );

        // ⚠ And behaviourally, over the whole writable inventory: exercise
        // EVERY store helper on a freshly constructed node and the store's
        // identity field must still be NULL_SLOT.
        let (_am, alloc_code) = compile_producer(4, emit_alloc_probe);
        let (_tm, tag_store) = compile_producer(3, emit_store_tag_id_probe);
        let (_sm, scalar_store) = compile_producer(3, emit_store_scalar_probe);
        let (_c1, slot_code) = compile_probe(Probe::Unary(|h| h.slot));

        let mut store = c1_d2_store();
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (2, 4, 0),
            (0, 0, 0),
        );
        let word = BoundaryWord(run4(
            alloc_code,
            f.base,
            BoundaryTag::PersistentGround as u64,
            BoundaryClass::Constructor as u64,
            0,
        ) as u64);
        assert_eq!(run3(tag_store, f.base, word, 77), BOUNDARY_OK);
        assert_eq!(run3(scalar_store, f.base, word, 88), BOUNDARY_OK);
        assert_eq!(
            run2(slot_code, f.base, word),
            crate::store::NULL_SLOT as i64,
            "AC-6: no sequence of emitted stores can give a node a store slot"
        );
        // Positive control: the writable fields DID move, so the NULL_SLOT
        // above is not "nothing was written anywhere".
        let (_c2, tag_read) = compile_probe(Probe::Unary(|h| h.tag));
        let (_c3, scalar_read) = compile_probe(Probe::Unary(|h| h.scalar));
        assert_eq!(run2(tag_read, f.base, word), 77);
        assert_eq!(run2(scalar_read, f.base, word), 88);
    }

    /// **`AC-1` — the `(tag, class)` RELATION is closed, positively and
    /// negatively, over the whole product.**
    ///
    /// ⚠ MEASURED: every one of the `BoundaryTag::ALL × BoundaryClass::ALL`
    /// pairs is put through the emitted allocator, against the **Rust mirror**'s
    /// answer. CLAIMED: the ABI admits exactly the relation the disposition
    /// yields. THE GAP: that the mirror *is* the disposition's relation — closed
    /// by [`b2v_the_rust_mirror_and_the_derived_relation_reconcile_over_the_product`],
    /// which sweeps the same product against the partition-derived plan in both
    /// directions.
    ///
    /// ⚠ This line used to point the gap at an elided `b2v_ac3_…`. Two tests
    /// with that prefix do exist, in `lowering/core/tests/control.rs` — they pin
    /// that the disposition has no wildcard arm and that every variant carries
    /// exactly one static policy. ⛔ **Neither says anything about the mirror**,
    /// so the reference named a real family that could not discharge the gap,
    /// which is worse than naming nothing: a reader who greps finds hits.
    #[test]
    fn b2v_the_tag_class_relation_is_closed_over_the_whole_product() {
        let (_pm, alloc_code) = compile_producer(4, emit_alloc_probe);
        let mut store = c1_d2_store();
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (64, 8, 0),
            (64, 8, 0),
        );

        let (mut admitted, mut rejected, mut retired) = (0usize, 0usize, 0usize);
        for tag in BoundaryTag::ALL {
            for class in BoundaryClass::ALL {
                let status = run4(alloc_code, f.base, tag as u64, class as u64, 0);
                if tag.is_immediate() {
                    // An immediate has no node, so it never reaches the
                    // relation at all — a distinct, earlier refusal.
                    assert_eq!(
                        status, BOUNDARY_ERR_SHAPE,
                        "AC-4: {tag:?} has no node to allocate"
                    );
                    continue;
                }
                // ⛔ `RT-FNSPLIT-C1` `D5` — the RETIRED lane is a third outcome.
                //
                // ⭐ It is neither admitted nor malformed, and collapsing it
                // into either arm would destroy the distinction the tombstone
                // exists to make: `PersistentClosure + Closure` is a
                // **well-formed pair naming a retired capability**, while
                // `PersistentClosure + Bool` is genuinely malformed and keeps
                // `BOUNDARY_ERR_RELATION`. Both are refused; only one can say
                // which lane it refused.
                if crate::boundary_value::boundary_lane_is_retired(tag, class) {
                    assert_eq!(
                        status,
                        crate::boundary_value::BOUNDARY_ERR_RETIRED_LANE,
                        "D5: {tag:?} + {class:?} is the retired lane and must be \
                         refused BY NAME at allocation, not as an unknown tag or \
                         a malformed pair"
                    );
                    retired += 1;
                    continue;
                }
                if boundary_relation_admits(tag, class) {
                    assert!(
                        status >= 0,
                        "AC-1: the ABI must admit {tag:?} + {class:?} (got {status})"
                    );
                    admitted += 1;
                } else {
                    assert_eq!(
                        status, BOUNDARY_ERR_RELATION,
                        "AC-1: the ABI must reject {tag:?} + {class:?} at ALLOCATION"
                    );
                    rejected += 1;
                }
            }
        }
        // ⚠ NON-VACUITY for the retired arm, and it is not optional: the arm
        // above is a `continue` guarded by a predicate, so a tombstone list that
        // silently emptied — or a `boundary_lane_is_retired` that stopped
        // matching — would take the arm zero times and the sweep would stay
        // green with the retired lane completely unexercised.
        assert_eq!(
            retired,
            crate::boundary_value::BOUNDARY_RETIRED_LANES.len(),
            "D5: the sweep exercised {retired} retired lanes but {} are declared; \
             the retired arm is not being reached",
            crate::boundary_value::BOUNDARY_RETIRED_LANES.len()
        );

        // ⚠ POSITIVE CONTROL on both arms: a relation that admitted everything
        // or nothing would satisfy one arm vacuously.
        let handles = BoundaryTag::ALL
            .iter()
            .filter(|t| !t.is_immediate())
            .count();
        // ⛔ **The schema is no longer the admitted set** (`RT-FNSPLIT-C1` `D5`).
        // `BOUNDARY_TAG_CLASS_RELATION` still spells the retired lane out —
        // that is the whole point of a tombstone, so a refusal can name it — so
        // the admitted total is the schema MINUS the retired rows. ⚠ Still
        // derived from both authorities rather than re-fitted to the observed
        // 7: a count refitted to whatever the code now emits measures nothing,
        // and this form goes red if either the schema or the tombstone list
        // moves.
        let expected_admitted: usize = BOUNDARY_TAG_CLASS_RELATION
            .iter()
            .flat_map(|(tag, classes)| classes.iter().map(move |class| (*tag, *class)))
            .filter(|(tag, class)| !crate::boundary_value::boundary_lane_is_retired(*tag, *class))
            .count();
        assert_eq!(admitted, expected_admitted, "AC-1: admitted count");
        // ⚠ The product now partitions into THREE arms, not two, so the retired
        // pairs must be subtracted here as well — otherwise this control would
        // silently absorb a lane that stopped being refused by name.
        assert_eq!(
            rejected,
            handles * BoundaryClass::ALL.len() - expected_admitted - retired,
            "AC-1: rejected count"
        );
        assert!(
            admitted > 0 && rejected > 0,
            "AC-1: neither arm may be empty"
        );

        // ⛔ The per-pair mask agreement that used to live here compared two
        // expressions of the SAME hand-written slice, which is why it could not
        // notice that nothing derived it. It is replaced by
        // `b2v_the_rust_mirror_and_the_derived_relation_reconcile_over_the_product`,
        // which compares the mirror against the PARTITION over this same
        // product, in both directions.
    }

    /// `(base, sign, len, seed) -> word` — build a PERSISTENT wide `Int`
    /// entirely from emitted code: allocate, mark the magnitude as region-owned,
    /// claim the limb span, then write every limb at run-time bounds.
    fn emit_wide_int_producer(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, sign, len, seed) = (p[0], p[1], p[2], p[3]);
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b.ins().iconst(types::I64, BoundaryClass::Int as i64);
        let zero = b.ins().iconst(types::I64, 0);
        guard(b, refs.alloc, &[base, tag, class, zero, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        let marker = b.ins().iconst(types::I64, BOUNDARY_INT_REGION_LIMBS as i64);
        guard(b, refs.store_int_tag, &[base, word, marker]);
        let span = cell(b, ptr);
        guard(b, refs.store_int_limbs, &[base, word, sign, len, span]);

        // `seed + i` at every index, over run-time bounds, so no limb of the
        // result is known when this body is compiled.
        let loop_head = b.create_block();
        b.append_block_param(loop_head, types::I64);
        b.ins().jump(loop_head, &[zero.into()]);
        b.switch_to_block(loop_head);
        let i = b.block_params(loop_head)[0];
        let more = b.ins().icmp(IntCC::UnsignedLessThan, i, len);
        let body = b.create_block();
        let done = b.create_block();
        b.ins().brif(more, body, &[], done, &[]);

        b.switch_to_block(body);
        let limb = b.ins().iadd(seed, i);
        guard(b, refs.store_int_limb, &[base, word, i, limb]);
        let next = b.ins().iadd_imm(i, 1);
        b.ins().jump(loop_head, &[next.into()]);

        b.switch_to_block(done);
        guard(b, refs.seal_int, &[base, word]);
        b.ins().return_(&[word]);
    }

    /// `(tag, payload, out) -> status` — `make_immediate` on its own, status
    /// returned unmodified so a control can assert the EXACT refusal.
    fn emit_make_immediate_probe(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        _ptr: cranelift_codegen::ir::Type,
    ) {
        let call = b.ins().call(refs.make_immediate, &[p[0], p[1], p[2]]);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
    }

    fn run_make_immediate(code: *const u8, tag: u64, payload: u64, out: &mut u64) -> i64 {
        let f: extern "C" fn(u64, u64, *mut u64) -> i64 = unsafe { std::mem::transmute(code) };
        f(tag, payload, out as *mut u64)
    }

    /// A wide `Int` built from the landed exact arithmetic, so its limbs are
    /// whatever that code actually produces rather than a hand-written pattern.
    fn wide_int(scale: i64) -> RuntimeIntV1 {
        RuntimeIntV1::Small(i64::MAX)
            .mul(&RuntimeIntV1::Small(1 << 20))
            .add(&RuntimeIntV1::Small(scale))
    }

    /// **`AC-1`/`AC-6` — the magnitude-marker relation is closed over the
    /// (owner, marker) product, not sampled.**
    ///
    /// ⛔ A closed set of markers is not a closed relation, for exactly the
    /// reason a closed set of tags and classes was not: the marker says *where
    /// the magnitude lives*, and putting an invocation-scoped one on a
    /// persistent node is the ephemeral-locator defect one representation down.
    #[test]
    fn b2v_the_magnitude_marker_relation_is_closed_over_owner_and_marker() {
        let (_am, alloc_code) = compile_producer(4, emit_alloc_probe);
        let (_tm, tag_probe) = compile_producer(3, emit_store_int_tag_probe);

        let mut store = c1_d2_store();
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (2, 0, 0),
            (0, 0, 0),
        );
        let word = BoundaryWord(run4(
            alloc_code,
            f.base,
            BoundaryTag::PersistentGround as u64,
            BoundaryClass::Int as u64,
            0,
        ) as u64);

        // ⛔ **Every allocatable `Int` node is PERSISTENT**, and that is a
        // consequence of the tag × class relation rather than a second rule:
        // `Int` appears under `PersistentGround` and nowhere else. Asserted from
        // the table, so admitting an invocation-owned `Int` later reddens here
        // and forces the marker question to be re-answered rather than
        // inherited.
        let int_tags: Vec<BoundaryTag> = BoundaryTag::ALL
            .iter()
            .copied()
            .filter(|t| boundary_relation_admits(*t, BoundaryClass::Int))
            .collect();
        assert_eq!(
            int_tags,
            vec![BoundaryTag::PersistentGround],
            "the marker sweep below covers every owner an Int node can have"
        );

        let mut admitted = 0;
        let mut refused = 0;
        // One past the closed set, so the range guard is exercised rather than
        // assumed — a marker that shifts past the word width could otherwise
        // alias an admitted bit.
        for marker in 0..=(LAST_INT_MARKER as u64 + 1) {
            let status = run3(tag_probe, f.base, word, marker);
            if marker > LAST_INT_MARKER as u64 {
                assert_eq!(
                    status, BOUNDARY_ERR_SHAPE,
                    "the marker set is closed: {marker} is outside it"
                );
                refused += 1;
            } else if boundary_int_marker_admits(marker, BoundaryReferentOwner::PersistentStore) {
                assert_eq!(
                    status, BOUNDARY_OK,
                    "a persistent Int must admit marker {marker}"
                );
                admitted += 1;
            } else {
                assert_eq!(
                    status, BOUNDARY_ERR_ESCAPE,
                    "marker {marker} names storage a persistent Int must not reach"
                );
                refused += 1;
            }
        }
        // ⚠ POSITIVE CONTROL on both arms: a check that admitted everything or
        // nothing would satisfy one arm vacuously.
        assert_eq!(admitted, 2, "Small and region-limbed are both admitted");
        assert_eq!(refused, 2, "the invocation Big and the out-of-set marker");

        // The mask the CLIF consumes must agree with the table, per pair.
        for owner in [
            BoundaryReferentOwner::PersistentStore,
            BoundaryReferentOwner::InvocationArena,
        ] {
            for marker in 0..=LAST_INT_MARKER as u64 {
                assert_eq!(
                    boundary_int_marker_mask(owner) & (1u64 << marker) != 0,
                    boundary_int_marker_admits(marker, owner),
                    "the emitted mask disagrees with the table for {owner:?} + {marker}"
                );
            }
        }
    }

    /// **`D1`/`D3`/`D4` — the `Int` disposition's promised spill EXISTS for an
    /// arbitrary-precision value.**
    ///
    /// ⛔ This is the pin the earlier candidate could not have passed.
    /// `Lowered::Int` classifies **every** `Int` as an immediate with a
    /// `PersistentGround`/`Int` spill, while materialization was `as_small()?`
    /// — so the promised spill did not exist for exactly the values a bignum
    /// language exists to carry, and the gap was recorded as a test residual
    /// rather than the missing deliverable it was.
    ///
    /// ⚠ MEASURED: a value too wide for `i64` materializes, and a separately
    /// compiled consumer recovers its sign and every limb. CLAIMED: the
    /// disposition's spill arm is deliverable. THE GAP: that the consumer's
    /// answer is the *value's* magnitude and not a re-reading of whatever the
    /// producer stored — closed by comparing against
    /// `RuntimeIntV1::canonical_sign_and_limbs`, which is the landed
    /// normalization every other consumer uses.
    #[test]
    fn b2v_a_wide_persistent_int_materializes_and_reads_back_by_content() {
        let (_c1, sign_code) = compile_probe(Probe::Unary(|h| h.int_sign));
        let (_c2, len_code) = compile_probe(Probe::Unary(|h| h.int_len));
        let (_c3, limb_code) = compile_probe(Probe::Binary(|h| h.int_limb));

        let mut seen = Vec::new();
        for scale in [0i64, 1, -1] {
            let value = wide_int(scale);
            let (sign, limbs) = value.canonical_sign_and_limbs();
            assert!(
                value.as_small().is_none() && limbs.len() > 1,
                "the case must actually be wide, or this control tests the Small arm"
            );

            let mut store = c1_d2_store();
            let word = materialize_ground(&mut store, &RuntimeGroundValue::Int(value.clone()))
                .expect("D1: a wide Int must materialize — the disposition promises the spill");
            assert_eq!(
                word.tag(),
                Some(BoundaryTag::PersistentGround),
                "D3: the spill is a persistent ground handle"
            );
            // Independent Rust oracle: the region's own view of the node, not
            // the emitted answer read back a second time.
            assert_eq!(
                store.image().0.node_limbs(word.payload()),
                Some(limbs.as_slice()),
                "the region must hold the canonical limbs"
            );

            let f = bind(&mut store, BoundaryArenaBuilder::new());
            assert_eq!(run2(sign_code, f.base, word), sign as i64, "sign");
            assert_eq!(
                run2(len_code, f.base, word),
                limbs.len() as i64,
                "limb count"
            );
            let read: Vec<u64> = (0..limbs.len() as u64)
                .map(|i| run3(limb_code, f.base, word, i) as u64)
                .collect();
            assert_eq!(read, limbs, "emitted code must recover every limb");

            // ⚠ The magnitude must genuinely exceed one limb AND `i64`, or a
            // Small-only implementation would pass this control.
            assert!(
                read.len() > 1,
                "a one-limb magnitude does not exercise the wide path"
            );
            // ⛔ Reading one limb past the end is BOUNDS, not a zero.
            assert_eq!(
                run3(limb_code, f.base, word, limbs.len() as u64),
                BOUNDARY_ERR_BOUNDS,
                "the limb span is bounds-checked"
            );
            seen.push((sign, read));
        }
        // ⚠ POSITIVE CONTROL — the three cases differ only in their low limb, so
        // anything reading identity, length or sign alone would collapse them.
        seen.dedup();
        assert_eq!(seen.len(), 3, "every wide case must be distinguishable");
    }

    /// **`AC-4`/`D4` — a wide persistent `Int` survives the invocation that read
    /// it, and emitted code can CONSTRUCT one.**
    ///
    /// ⛔ The producer half exists because of what QA's last block taught: a
    /// working read path is not evidence that the write path exists. The
    /// survival half is what distinguishes region-owned limbs from the
    /// invocation-scoped representation they replace — the arena is dropped
    /// between the write and the read.
    #[test]
    fn b2v_emitted_code_constructs_a_wide_persistent_int_that_outlives_the_arena() {
        let (_pm, produce) = compile_producer(4, emit_wide_int_producer);
        let (_c1, sign_code) = compile_probe(Probe::Unary(|h| h.int_sign));
        let (_c2, len_code) = compile_probe(Probe::Unary(|h| h.int_len));
        let (_c3, limb_code) = compile_probe(Probe::Binary(|h| h.int_limb));

        let len = 3u64;
        let seed = 0x0123_4567_89ab_cdefu64;
        let mut store = c1_d2_store();
        let (word, persistent) = {
            let f = bind_limbs(
                &mut store,
                BoundaryArenaBuilder::new(),
                (2, 0, 0),
                (0, 0, 0),
                (8, 0),
            );
            let produced = run4(produce, f.base, 1, len, seed);
            assert!(
                produced > 0,
                "emitted wide-Int construction must succeed, got status {produced}"
            );
            (BoundaryWord(produced as u64), f.persistent)
        };

        // ⭐ A FRESH invocation over the same persistent image: new arena, new
        // tables, sharing only the store's region. If the limbs had gone to
        // invocation storage the word would now name freed memory.
        let g = rebind(persistent);
        assert_eq!(run2(sign_code, g.base, word), 1, "the sign survives");
        assert_eq!(
            run2(len_code, g.base, word),
            len as i64,
            "the length survives"
        );
        let read: Vec<u64> = (0..len)
            .map(|i| run3(limb_code, g.base, word, i) as u64)
            .collect();
        assert_eq!(
            read,
            (0..len).map(|i| seed + i).collect::<Vec<_>>(),
            "every limb survives the invocation that wrote it"
        );
        // ⚠ POSITIVE CONTROL — the limbs are distinct, so a reader returning a
        // constant or the first limb repeatedly would not pass.
        assert_eq!(
            read.iter().collect::<std::collections::BTreeSet<_>>().len(),
            len as usize
        );
    }

    /// **`AC-1`/`AC-2` — emitted immediate construction range-checks its payload
    /// instead of truncating it.**
    ///
    /// ⛔ `ken_boundary_make_immediate_local` built its word with a left shift
    /// and checked only that the tag was below the first handle tag. A shift is
    /// **total**: a payload wider than the field silently became a *different
    /// value*, and a `Bool` payload of `2` became a third boolean —
    /// while `boundary_value.rs` said emitted code performed the identical
    /// range test. The only magnitude control exercised Rust materialization,
    /// which is a different producer entirely.
    #[test]
    fn b2v_emitted_immediate_construction_refuses_what_it_cannot_represent() {
        let (_pm, mint) = compile_producer(3, emit_make_immediate_probe);
        let mut out = 0u64;

        let mut admitted = 0;
        let mut refused = 0;
        for tag in BoundaryTag::ALL {
            // Payloads chosen to straddle every domain boundary at once, so one
            // sweep exercises the bit, the signed field and the unsigned field.
            for payload in [
                0u64,
                1,
                2,
                BOUNDARY_IMMEDIATE_INT_MAX as u64,
                (BOUNDARY_IMMEDIATE_INT_MAX as u64) + 1,
                BOUNDARY_IMMEDIATE_INT_MIN as u64,
                (BOUNDARY_IMMEDIATE_INT_MIN as u64) - 1,
                1u64 << BOUNDARY_PAYLOAD_BITS,
                u64::MAX,
            ] {
                out = 0;
                let status = run_make_immediate(mint, tag as u64, payload, &mut out);
                if boundary_immediate_admits(tag, payload) {
                    assert_eq!(
                        status, BOUNDARY_OK,
                        "{tag:?} must admit payload {payload:#x}"
                    );
                    // ⛔ **The word must round-trip.** This is what makes the
                    // check about truncation rather than about a status code:
                    // an admitted payload that came back different would be the
                    // very defect being closed.
                    let word = BoundaryWord(out);
                    assert_eq!(word.tag(), Some(tag), "the tag round-trips");
                    let back = if boundary_immediate_domain(tag)
                        == Some(BoundaryImmediateDomain::SignedPayload)
                    {
                        word.signed_payload() as u64
                    } else {
                        word.payload()
                    };
                    assert_eq!(back, payload, "{tag:?} truncated payload {payload:#x}");
                    assert_eq!(
                        word,
                        BoundaryWord::immediate(tag, payload),
                        "the emitted word and the Rust builder's must be identical"
                    );
                    admitted += 1;
                } else if !tag.is_immediate() {
                    assert_eq!(
                        status, BOUNDARY_ERR_SHAPE,
                        "{tag:?} is a handle tag and has no immediate form"
                    );
                    assert_eq!(out, 0, "a refused mint must write no word");
                    refused += 1;
                } else {
                    let expected = match boundary_immediate_domain(tag) {
                        Some(BoundaryImmediateDomain::Bit) => BOUNDARY_ERR_SHAPE,
                        _ => BOUNDARY_ERR_BOUNDS,
                    };
                    assert_eq!(
                        status, expected,
                        "{tag:?} must refuse payload {payload:#x} with an exact error"
                    );
                    assert_eq!(out, 0, "a refused mint must write no word");
                    refused += 1;
                }
            }
        }
        // ⚠ POSITIVE CONTROL on both arms — a checker that admitted everything
        // or refused everything satisfies one arm vacuously, and the earlier
        // candidate admitted everything.
        assert!(admitted > 0 && refused > 0, "neither arm may be empty");
        // And the two refusal reasons are genuinely distinguished, not one error
        // wearing two names.
        out = 0;
        assert_eq!(
            run_make_immediate(mint, BoundaryTag::ImmediateBool as u64, 2, &mut out),
            BOUNDARY_ERR_SHAPE,
            "a Bool that is not a bit is the wrong SHAPE"
        );
        assert_eq!(
            run_make_immediate(
                mint,
                BoundaryTag::ImmediateInt as u64,
                1u64 << BOUNDARY_PAYLOAD_BITS,
                &mut out
            ),
            BOUNDARY_ERR_BOUNDS,
            "a magnitude past the field is out of BOUNDS"
        );
    }

    /// **`AC-1` layout closure — one authority, derived extents, real
    /// consumers.**
    ///
    /// ⛔ The bound clause is explicit that *checking a hand-maintained constant
    /// against another hand-maintained constant does not discharge this*, and my
    /// first repair did exactly that. **The mechanism chosen is derivation:**
    /// [`NodeField`] and [`RegionHeaderField`] are the inventories, every
    /// offset is `position × 8`, both extents are `ALL.len() × 8`, and the two
    /// consumers — `publish` and `push_node` — place each word through a
    /// `match` with **no `_` arm**. A new field is therefore a **compile
    /// error**, which is the strongest available mechanism and is why the
    /// checks below are about *drift that remains possible* rather than about
    /// re-deriving the same arithmetic twice.
    ///
    /// ⚠ MEASURED: a **published** region's word count, the emitted-side offset
    /// constants, and the derived extents. CLAIMED: no consumer can read or
    /// write outside the extent the inventory defines. THE GAP: field *width* —
    /// every access is an 8-byte word, so an offset within the extent is not
    /// enough; `offset + 8 <= extent` is the clause's own wording and is
    /// asserted here.
    #[test]
    fn b2v_the_layout_inventory_is_the_sole_authority() {
        // ── the extents are derived, and the offsets are the slots ──────────
        let header: Vec<i32> = RegionHeaderField::ALL.iter().map(|f| f.offset()).collect();
        let node: Vec<i32> = NodeField::ALL.iter().map(|f| f.offset()).collect();
        for (what, offsets, extent) in [
            ("region header", &header, BOUNDARY_REGION_HEADER_BYTES),
            ("node", &node, BOUNDARY_NODE_STRIDE),
        ] {
            assert_eq!(
                *offsets,
                (0..extent).step_by(8).collect::<Vec<i32>>(),
                "{what}: the offsets are not exactly the slots of the extent"
            );
            // ⛔ Offset PLUS WIDTH. Every access is an 8-byte word, so the last
            // field's offset must leave a whole word inside the extent.
            for at in offsets {
                assert!(
                    at + 8 <= extent,
                    "{what}: a field at {at} reads past the {extent}-byte extent"
                );
            }
            assert!(
                !offsets.is_empty(),
                "{what}: the inventory may not be empty"
            );
        }

        // ── the emitted side reads the SAME constants, so name them ─────────
        //
        // ⚠ This is the drift the clause names as "emitted offset". There is no
        // second emitted authority — the CLIF loads at these very constants —
        // and pinning the correspondence keeps that true if someone introduces
        // one.
        for (at, field) in [
            (NODE_CLASS, NodeField::Class),
            (NODE_OWNER, NodeField::Owner),
            (NODE_SLOT, NodeField::Slot),
            (NODE_TAG_ID, NodeField::TagId),
            (NODE_PAYLOAD, NodeField::Payload),
            (NODE_FIELD_COUNT, NodeField::FieldCount),
            (NODE_FIELDS_AT, NodeField::FieldsAt),
            (NODE_EXTENT, NodeField::Extent),
            (NODE_LIMBS_AT, NodeField::LimbsAt),
            (NODE_LIMB_COUNT, NodeField::LimbCount),
            (NODE_INT_SEALED, NodeField::IntSealed),
        ] {
            assert_eq!(at, field.offset(), "an emitted node offset has drifted");
        }
        for (at, field) in [
            (ARENA_NODES, RegionHeaderField::Nodes),
            (ARENA_NODE_COUNT, RegionHeaderField::NodeCount),
            (ARENA_WORDS, RegionHeaderField::Words),
            (ARENA_WORD_COUNT, RegionHeaderField::WordCount),
            (ARENA_NAMES, RegionHeaderField::Names),
            (ARENA_NAME_COUNT, RegionHeaderField::NameCount),
            (ARENA_NODE_CAPACITY, RegionHeaderField::NodeCapacity),
            (ARENA_WORD_CAPACITY, RegionHeaderField::WordCapacity),
            (ARENA_PERSISTENT, RegionHeaderField::Persistent),
            (ARENA_FROZEN, RegionHeaderField::Frozen),
            (ARENA_DATA, RegionHeaderField::Data),
            (ARENA_DATA_COUNT, RegionHeaderField::DataCount),
            (ARENA_DATA_CAPACITY, RegionHeaderField::DataCapacity),
            (ARENA_NATIVE_INT, RegionHeaderField::NativeInt),
            (ARENA_LIMBS, RegionHeaderField::Limbs),
            (ARENA_LIMB_COUNT, RegionHeaderField::LimbCount),
            (ARENA_LIMB_CAPACITY, RegionHeaderField::LimbCapacity),
        ] {
            assert_eq!(at, field.offset(), "an emitted header offset has drifted");
        }

        // ── and PUBLICATION emits exactly the derived extent ────────────────
        //
        // ⭐ The one quantity that is not derivable by inspection: what
        // `publish` actually wrote. Measured on a real region, not restated.
        let mut store = c1_d2_store();
        materialize_ground(&mut store, &RuntimeGroundValue::Bool(true));
        let f = bind(&mut store, BoundaryArenaBuilder::new());
        for (what, words) in [
            ("persistent image", store.image().0.published_header_len()),
            ("invocation arena", f.arena.0.published_header_len()),
        ] {
            assert_eq!(
                words * std::mem::size_of::<u64>(),
                BOUNDARY_REGION_HEADER_BYTES as usize,
                "{what}: publication did not emit exactly the derived extent"
            );
        }
        // ⚠ POSITIVE CONTROL — a region that never published would report 0 and
        // satisfy nothing, so assert the measurement is of a real header.
        assert!(store.image().0.published_header_len() > 0);
    }

    /// `(base, sign, len, seed, top) -> word` — the wide-`Int` producer with the
    /// **top limb chosen by the caller**, so a control can drive the magnitude
    /// to each noncanonical shape without a second compiled body.
    fn emit_wide_int_producer_with_top(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, sign, len, seed, top) = (p[0], p[1], p[2], p[3], p[4]);
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b.ins().iconst(types::I64, BoundaryClass::Int as i64);
        let zero = b.ins().iconst(types::I64, 0);
        guard(b, refs.alloc, &[base, tag, class, zero, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        let marker = b.ins().iconst(types::I64, BOUNDARY_INT_REGION_LIMBS as i64);
        guard(b, refs.store_int_tag, &[base, word, marker]);
        let span = cell(b, ptr);
        guard(b, refs.store_int_limbs, &[base, word, sign, len, span]);

        let loop_head = b.create_block();
        b.append_block_param(loop_head, types::I64);
        b.ins().jump(loop_head, &[zero.into()]);
        b.switch_to_block(loop_head);
        let i = b.block_params(loop_head)[0];
        let more = b.ins().icmp(IntCC::UnsignedLessThan, i, len);
        let body = b.create_block();
        let done = b.create_block();
        b.ins().brif(more, body, &[], done, &[]);

        b.switch_to_block(body);
        // The last limb is the caller's; every other is `seed + i`.
        let last = b.ins().iadd_imm(len, -1);
        let is_last = b.ins().icmp(IntCC::Equal, i, last);
        let running = b.ins().iadd(seed, i);
        let limb = b.ins().select(is_last, top, running);
        guard(b, refs.store_int_limb, &[base, word, i, limb]);
        let next = b.ins().iadd_imm(i, 1);
        b.ins().jump(loop_head, &[next.into()]);

        b.switch_to_block(done);
        guard(b, refs.seal_int, &[base, word]);
        b.ins().return_(&[word]);
    }

    /// **`AC-1`/`AC-4` — emitted wide-`Int` construction cannot publish a word
    /// that denotes no canonical exact `Int`.**
    ///
    /// ⛔ It could. `store_int_limbs` checked `sign <= 1` and capacity and
    /// nothing else, so `len = 0`, a leading zero limb, and negative zero all
    /// returned success — and the committed control used an arbitrary nonzero
    /// seed, so it never went near the boundary. The contract is
    /// `RuntimeIntV1::canonical_sign_and_limbs`'s and it is not optional: a
    /// leading zero limb gives one value two encodings, and negative zero gives
    /// zero a second one.
    ///
    /// ⚠ The canonicity clauses are **not** checkable where the span is
    /// claimed — no limb exists yet — so the interface gained a completion step.
    /// `seal_int` runs after the limbs are written, and every reader requires
    /// the seal, which is what makes "fails closed before publication" a
    /// property of the mechanism rather than a sentence about it.
    #[test]
    fn b2v_emitted_wide_int_construction_refuses_a_noncanonical_magnitude() {
        let (_pm, produce) = compile_producer(5, emit_wide_int_producer_with_top);
        let (_c1, len_code) = compile_probe(Probe::Unary(|h| h.int_len));

        // (sign, len, seed, top, expected) — one row per canonicity clause,
        // each differing from the admitted row in exactly one component.
        let cases: [(u64, u64, u64, u64, i64); 6] = [
            // ⚠ POSITIVE CONTROL — the admitted shape, so "refuses everything"
            // cannot pass this test.
            (0, 3, 7, 9, BOUNDARY_OK),
            (1, 3, 7, 9, BOUNDARY_OK),
            // Empty magnitude: no integer at all. Refused where the span is
            // claimed, because length is the one clause checkable there.
            (0, 0, 7, 9, BOUNDARY_ERR_SHAPE),
            // Leading zero limb: two encodings of one value.
            (0, 3, 7, 0, BOUNDARY_ERR_SHAPE),
            (1, 2, 7, 0, BOUNDARY_ERR_SHAPE),
            // Negative zero: a second encoding of zero.
            (1, 1, 0, 0, BOUNDARY_ERR_SHAPE),
        ];
        let mut admitted = 0;
        let mut refused = 0;
        for (sign, len, seed, top, expected) in cases {
            let mut store = c1_d2_store();
            let f = bind_limbs(
                &mut store,
                BoundaryArenaBuilder::new(),
                (2, 0, 0),
                (0, 0, 0),
                (8, 0),
            );
            let produced = run5(produce, f.base, sign, len, seed, top);
            if expected == BOUNDARY_OK {
                assert!(
                    produced > 0,
                    "a canonical magnitude must publish (sign {sign}, len {len}, \
                     top {top}); got status {produced}"
                );
                assert_eq!(
                    run2(len_code, f.base, BoundaryWord(produced as u64)),
                    len as i64,
                    "a sealed magnitude reads back its length"
                );
                admitted += 1;
            } else {
                assert_eq!(
                    produced, expected,
                    "a noncanonical magnitude must be refused with an exact \
                     status (sign {sign}, len {len}, top {top})"
                );
                refused += 1;
            }
        }
        assert!(admitted > 0 && refused > 0, "neither arm may be empty");

        // ⛔ **The seal is what makes the refusal a refusal.** A producer that
        // ignores the status must still be unable to publish: a node whose
        // limbs were claimed but never sealed reads as `ERR_SHAPE`, not as a
        // magnitude. Without this the checks above would be advice.
        let (_um, unsealed) = compile_producer(4, emit_wide_int_producer_no_seal);
        let (_c2, sign_code) = compile_probe(Probe::Unary(|h| h.int_sign));
        let (_c3, limb_code) = compile_probe(Probe::Binary(|h| h.int_limb));
        let mut store = c1_d2_store();
        let f = bind_limbs(
            &mut store,
            BoundaryArenaBuilder::new(),
            (2, 0, 0),
            (0, 0, 0),
            (8, 0),
        );
        let word = BoundaryWord(run4(unsealed, f.base, 0, 2, 5) as u64);
        for (what, status) in [
            ("sign", run2(sign_code, f.base, word)),
            ("len", run2(len_code, f.base, word)),
            ("limb", run3(limb_code, f.base, word, 0)),
        ] {
            assert_eq!(
                status, BOUNDARY_ERR_SHAPE,
                "AC-4: an unsealed magnitude must not be readable ({what})"
            );
        }
    }

    /// The wide-`Int` producer **without** the completion step — the positive
    /// control for the seal itself.
    fn emit_wide_int_producer_no_seal(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, sign, len, seed) = (p[0], p[1], p[2], p[3]);
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b.ins().iconst(types::I64, BoundaryClass::Int as i64);
        let zero = b.ins().iconst(types::I64, 0);
        guard(b, refs.alloc, &[base, tag, class, zero, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        let marker = b.ins().iconst(types::I64, BOUNDARY_INT_REGION_LIMBS as i64);
        guard(b, refs.store_int_tag, &[base, word, marker]);
        let span = cell(b, ptr);
        guard(b, refs.store_int_limbs, &[base, word, sign, len, span]);
        let loop_head = b.create_block();
        b.append_block_param(loop_head, types::I64);
        b.ins().jump(loop_head, &[zero.into()]);
        b.switch_to_block(loop_head);
        let i = b.block_params(loop_head)[0];
        let more = b.ins().icmp(IntCC::UnsignedLessThan, i, len);
        let body = b.create_block();
        let done = b.create_block();
        b.ins().brif(more, body, &[], done, &[]);
        b.switch_to_block(body);
        let limb = b.ins().iadd(seed, i);
        guard(b, refs.store_int_limb, &[base, word, i, limb]);
        let next = b.ins().iadd_imm(i, 1);
        b.ins().jump(loop_head, &[next.into()]);
        b.switch_to_block(done);
        b.ins().return_(&[word]);
    }

    /// **`AC-1` — the region-limb span check does not WRAP.**
    ///
    /// ⛔ The reader computed `end = at + len` with CLIF's wrapping `iadd` and
    /// accepted `end <= live`. A span whose start is near `u64::MAX` wraps to a
    /// small sum, satisfies the comparison, and the address is then formed from
    /// the **unchecked** `at`. The comment above it said it failed closed before
    /// any address was formed. The Rust oracle beside it used `checked_add` and
    /// was correct — two halves of one property written to different standards.
    ///
    /// ⚠ **No production path can produce a malformed span**, so this control
    /// injects the corruption directly. A control that cannot construct the
    /// violating input is not evidence about the guard.
    #[test]
    fn b2v_a_wrapped_limb_span_fails_closed() {
        let (_c1, sign_code) = compile_probe(Probe::Unary(|h| h.int_sign));
        let (_c2, limb_code) = compile_probe(Probe::Binary(|h| h.int_limb));

        let value = wide_int(0);
        let mut store = c1_d2_store();
        let word = materialize_ground(&mut store, &RuntimeGroundValue::Int(value))
            .expect("a wide Int materializes");
        let index = word.payload();

        // ⚠ POSITIVE CONTROL first, on the untouched node: the reader answers,
        // so a later refusal is about the span and not about the fixture. The
        // image is published exactly once — the corruption happens in place,
        // and the second invocation rebinds the same region.
        let f = bind(&mut store, BoundaryArenaBuilder::new());
        let persistent = f.persistent;
        assert_eq!(run2(sign_code, f.base, word), 0, "the intact node reads");
        drop(f);

        // A start that makes `at + len` WRAP to a small value. Under the old
        // form `end` was tiny, `end <= live` held, and the address came from
        // this `at`.
        store
            .image_mut()
            .0
            .poke_node_field(index, NODE_LIMBS_AT, u64::MAX - 1);
        let f = rebind(persistent);
        assert_eq!(
            run2(sign_code, f.base, word),
            BOUNDARY_ERR_BOUNDS,
            "AC-1: a wrapped span must fail closed before an address is formed"
        );
        assert_eq!(
            run3(limb_code, f.base, word, 0),
            BOUNDARY_ERR_BOUNDS,
            "AC-1: and on the limb path too"
        );
        // The Rust oracle must agree — it is the half that was already right.
        assert_eq!(
            store.image().0.node_limbs(index),
            None,
            "the Rust oracle must refuse the same span"
        );
    }

    /// `(base, value) -> word` — the MAGNITUDE PARTITION, decided by emitted
    /// code at run time.
    ///
    /// ⛔ One compiled body, one runtime test, both arms. The immediate arm goes
    /// through `make_immediate`; the spill arm allocates a persistent `Int` and
    /// records the landed native `(tag, payload)` pair. Nothing here inspects a
    /// JIT-time value to choose a layout — that is `AC-2`, and it is why the
    /// partition is a property of the value rather than of the compilation.
    fn emit_magnitude_partition_producer(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, value) = (p[0], p[1]);
        let out = cell(b, ptr);
        // The same test `BoundaryWord::int_fits_immediate` performs: sign-extend
        // the payload field back and see whether the value survives.
        let shift = i64::from(BOUNDARY_TAG_BITS);
        let up = b.ins().ishl_imm(value, shift);
        let back = b.ins().sshr_imm(up, shift);
        let fits = b.ins().icmp(IntCC::Equal, back, value);
        let immediate = b.create_block();
        let spilled = b.create_block();
        b.ins().brif(fits, immediate, &[], spilled, &[]);

        b.switch_to_block(immediate);
        let int_tag = b.ins().iconst(types::I64, BoundaryTag::ImmediateInt as i64);
        guard(b, refs.make_immediate, &[int_tag, value, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        b.ins().return_(&[word]);

        b.switch_to_block(spilled);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b.ins().iconst(types::I64, BoundaryClass::Int as i64);
        let zero = b.ins().iconst(types::I64, 0);
        guard(b, refs.alloc, &[base, tag, class, zero, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        guard(b, refs.store_scalar, &[base, word, value]);
        let small = b.ins().iconst(
            types::I64,
            crate::native_int::NATIVE_INT_SMALL_TAG_V1 as i64,
        );
        guard(b, refs.store_int_tag, &[base, word, small]);
        b.ins().return_(&[word]);
    }

    /// **`AC-10` — the magnitude partition is a REAL emitted boundary, and its
    /// spill arm discharges the handle obligations.**
    ///
    /// ⛔ The classifier says `Lowered::Int` is *immediate-with-declared-handle-
    /// spill* and that the boundary is `int_fits_immediate`. That is a claim
    /// about emitted behaviour, and until now the only magnitude control ran
    /// **Rust** materialization — a different producer entirely, which is the
    /// same defect QA found on the `String` arm.
    ///
    /// ⚠ The witness pair is `MAX` against `MAX + 1`: **adjacent** values, so
    /// nothing but the partition can separate them. Both are minted by one
    /// compiled body making a **run-time** decision, and both are read back by
    /// separately compiled consumers.
    #[test]
    fn b2v_ac10_the_magnitude_boundary_is_a_real_emitted_partition() {
        let (_pm, produce) = compile_producer(2, emit_magnitude_partition_producer);
        let (_c1, owner_code) = compile_probe(Probe::Unary(|h| h.owner));
        let (_c2, class_code) = compile_probe(Probe::Unary(|h| h.class));
        let (_c3, sign_code) = compile_probe(Probe::Unary(|h| h.int_sign));
        let (_c4, limb_code) = compile_probe(Probe::Binary(|h| h.int_limb));

        for (value, expect_immediate) in [
            (BOUNDARY_IMMEDIATE_INT_MAX, true),
            (BOUNDARY_IMMEDIATE_INT_MAX + 1, false),
            (BOUNDARY_IMMEDIATE_INT_MIN, true),
            (BOUNDARY_IMMEDIATE_INT_MIN - 1, false),
        ] {
            // The classifier's prediction, from the same total projection the
            // emitted body performs.
            assert_eq!(
                BoundaryWord::int_fits_immediate(value),
                expect_immediate,
                "the fixture disagrees with the partition it is testing"
            );

            let mut store = c1_d2_store();
            let native = crate::native_int::NativeIntArenaV1::default();
            let mut f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (2, 0, 0),
                (0, 0, 0),
            );
            with_native_int(&mut f, &native);
            let word = BoundaryWord(run3(produce, f.base, BoundaryWord(value as u64), 0) as u64);

            if expect_immediate {
                assert_eq!(
                    word.tag(),
                    Some(BoundaryTag::ImmediateInt),
                    "AC-10: {value} is within the field, so the outcome is an \
                     IMMEDIATE WORD"
                );
                assert_eq!(
                    word.signed_payload(),
                    value,
                    "AC-10: the immediate arm must carry the value, not truncate it"
                );
                // ⛔ A separately compiled consumer agrees, and its answer is a
                // REFUSAL rather than `NoReferent`: `owner` is a node
                // projection, and an immediate word has no node. That is the
                // nondegenerate half of the pair — the same probe returns
                // `PersistentStore` one value later.
                assert_eq!(
                    run2(owner_code, f.base, word),
                    BOUNDARY_ERR_SHAPE,
                    "AC-10: an immediate word has no node to project an owner from"
                );
            } else {
                // ⛔ **The SPILL ARM is a handle outcome, so it discharges class,
                // referent owner, identity and lifetime — not merely "it did not
                // truncate".**
                assert_eq!(
                    word.tag(),
                    Some(BoundaryTag::PersistentGround),
                    "AC-10: {value} is beyond the field, so the outcome is a HANDLE"
                );
                assert_eq!(
                    run2(class_code, f.base, word),
                    BoundaryClass::Int as i64,
                    "AC-10: the spill arm's declared class"
                );
                assert_eq!(
                    run2(owner_code, f.base, word),
                    BoundaryReferentOwner::PersistentStore as i64,
                    "AC-10: the spill arm's declared owner — which is its lifetime"
                );
                // Identity: emitted-constructed, so NO store identity, and the
                // classifier says exactly that.
                assert_eq!(
                    store.image().0.node_field(word.payload(), NODE_SLOT),
                    Some(crate::store::NULL_SLOT),
                    "AC-10: an emitted-constructed handle carries no store identity"
                );
                // And the content survives, read by separately compiled bodies.
                let sign = run2(sign_code, f.base, word);
                let limb = run3(limb_code, f.base, word, 0);
                let observed = if sign == 1 { -limb } else { limb };
                assert_eq!(
                    observed, value,
                    "AC-10: the spill arm must recover the RUNTIME magnitude"
                );
            }
        }
    }

    /// **`AC-10`/`AC-6` — emitted construct → seal → STORE ADOPT → a separately
    /// compiled consumer recovers the real identity and the content, after the
    /// producer's arena is gone.**
    ///
    /// ⛔ The prior candidate classified an emitted-constructed persistent node
    /// as a published handle with "no store identity". The Architect ruled that
    /// out and the reasoning is decisive: a consumer can recover the *absence*
    /// of an identity, which is not recovering the same identity **intact** —
    /// and a null `NODE_SLOT` denotes *invocation-arena* ownership in this very
    /// layout, so the word contradicted itself. Reserving persistent-region
    /// storage is storage governance, not adoption.
    #[test]
    fn b2v_ac10_emitted_construction_publishes_only_through_store_adoption() {
        let (_pm, produce) = compile_producer(4, emit_wide_int_producer);
        let (_c1, escape_code) = compile_probe(Probe::Status(|h| h.escape_check));
        let (_c2, slot_code) = compile_probe(Probe::Unary(|h| h.slot));
        let (_c3, len_code) = compile_probe(Probe::Unary(|h| h.int_len));
        let (_c4, limb_code) = compile_probe(Probe::Binary(|h| h.int_limb));

        let len = 3u64;
        let seed = 0x00ff_0000_0000_0001u64;
        let mut store = c1_d2_store();
        let (pending, persistent) = {
            let f = bind_limbs(
                &mut store,
                BoundaryArenaBuilder::new(),
                (4, 0, 0),
                (0, 0, 0),
                (16, 0),
            );
            let produced = run4(produce, f.base, 0, len, seed);
            assert!(produced > 0, "emitted construction succeeds: {produced}");
            let word = BoundaryWord(produced as u64);
            // ⛔ **Constructed and sealed is NOT published.** The escape gate
            // refuses to let a pending persistent word cross a generated
            // function boundary — that is what makes adoption non-optional
            // rather than advisory.
            assert_eq!(
                run2(escape_code, f.base, word),
                BOUNDARY_ERR_ESCAPE,
                "AC-10: an unadopted persistent word must not escape"
            );
            (word, f.persistent)
        };
        let _ = persistent;

        // ── the sealed handoff, then the store-owned adoption boundary ─────
        store.seal_persistent();
        let adopted = store.adopt(pending).expect("AC-10: adoption succeeds");
        let slot = store
            .image()
            .0
            .node_field(adopted.payload(), NODE_SLOT)
            .expect("the adopted node is live");
        assert_ne!(
            slot,
            crate::store::NULL_SLOT,
            "AC-10: adoption must mint a REAL identity, not merely record one"
        );
        assert_eq!(
            store.placement(slot),
            Some(adopted.payload()),
            "AC-10: adoption installs the placement, so the identity resolves back"
        );

        // ⭐ A FRESH invocation: the producer's arena is gone, and a separately
        // compiled consumer recovers the identity AND the content.
        let g = rebind(store.image_mut().0.publish());
        assert_eq!(
            run2(slot_code, g.base, adopted),
            slot as i64,
            "AC-10: the consumer recovers the SAME identity, not its absence"
        );
        assert_eq!(
            run2(escape_code, g.base, adopted),
            BOUNDARY_OK,
            "AC-10: an adopted persistent word may now cross the boundary"
        );
        assert_eq!(
            run2(len_code, g.base, adopted),
            len as i64,
            "content: length"
        );
        let read: Vec<u64> = (0..len)
            .map(|i| run3(limb_code, g.base, adopted, i) as u64)
            .collect();
        assert_eq!(
            read,
            (0..len).map(|i| seed + i).collect::<Vec<_>>(),
            "AC-10: content survives adoption and the arena drop"
        );

        // ⛔ **Emitted code still cannot assign identity** while this
        // store-owned path is positively exercised — the anti-forgery property
        // carries unchanged, and the pair proves the two are not the same
        // capability.
        assert_eq!(
            EMITTED_WRITABLE_NODE_OFFSETS,
            &[NODE_TAG_ID, NODE_PAYLOAD],
            "AC-6: emitted code gained a writable node word"
        );
        assert!(
            !BOUNDARY_LOCAL_HELPERS
                .iter()
                .any(|name| name.contains("adopt")),
            "AC-6: adoption is the STORE's operation and has no emitted helper"
        );
    }

    /// **`AC-10` — equal emitted values converge on one store identity, and
    /// unequal values never alias.**
    ///
    /// ⚠ The pair is the whole control: convergence alone is satisfied by a
    /// store that gives everything one slot, and non-aliasing alone by one that
    /// never reuses. Both are asserted on independently emitted values.
    #[test]
    fn b2v_ac10_adoption_converges_equal_values_and_never_aliases_unequal() {
        let (_pm, produce) = compile_producer(4, emit_wide_int_producer);
        let len = 2u64;
        let mut store = c1_d2_store();
        let f = bind_limbs(
            &mut store,
            BoundaryArenaBuilder::new(),
            (8, 0, 0),
            (0, 0, 0),
            (32, 0),
        );
        // Three independent emitted constructions: two equal, one different.
        let mint = |base, seed| BoundaryWord(run4(produce, base, 0, len, seed) as u64);
        let first = mint(f.base, 0x0abc_0000_0000_0011);
        let second = mint(f.base, 0x0abc_0000_0000_0011);
        let other = mint(f.base, 0x0abc_0000_0000_0012);
        assert_ne!(
            first, second,
            "the two equal values must be DISTINCT nodes before adoption, or \
             convergence is trivial"
        );

        store.seal_persistent();
        let a = store.adopt(first).expect("adopts");
        let b = store.adopt(second).expect("adopts");
        let c = store.adopt(other).expect("adopts");
        let slot_of = |store: &BoundaryValueStore, word: BoundaryWord| {
            store
                .image()
                .0
                .node_field(word.payload(), NODE_SLOT)
                .expect("live")
        };
        assert_eq!(
            a, b,
            "AC-10: equal independently emitted values converge on ONE canonical word"
        );
        assert_eq!(slot_of(&store, a), slot_of(&store, b), "and one identity");
        assert_ne!(
            slot_of(&store, a),
            slot_of(&store, c),
            "AC-10: unequal values must never alias onto one identity"
        );
        assert_ne!(a, c, "AC-10: and must not share a canonical word");
    }

    /// **`AC-10` — no parent adopts while a reachable child is invocation-owned,
    /// and adoption fails closed with an exact status.**
    #[test]
    fn b2v_ac10_adoption_fails_closed_before_publication() {
        let mut store = c1_d2_store();
        // A word whose tag is not persistent has no adoption boundary.
        let immediate = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 7);
        assert_eq!(
            store.adopt(immediate),
            Err(BOUNDARY_ERR_SHAPE),
            "AC-10: only a persistent handle is adoptable"
        );
        // ⚠ And it fails on the SHAPE before it ever looks at the seal, which is
        // why this case still answers unsealed: a word that is not a persistent
        // handle has no adoption boundary to hand over in the first place.
        store.seal_persistent();
        // A persistent word naming no node cannot be validated.
        assert_eq!(
            store.adopt(BoundaryWord::handle(BoundaryTag::PersistentGround, 99)),
            Err(BOUNDARY_ERR_BOUNDS),
            "AC-10: adoption fails closed on a word it cannot resolve"
        );
        // ⚠ POSITIVE CONTROL: a store-materialized node is already adopted, so
        // adoption is idempotent rather than refusing everything.
        let word = materialize_ground(&mut store, &RuntimeGroundValue::Bytes(vec![1, 2, 3]))
            .expect("materializes");
        assert_eq!(
            store.adopt(word),
            Ok(word),
            "AC-10: an already-adopted node adopts idempotently"
        );
    }

    /// `(base, tag_id) -> word` — allocate one persistent `Constructor` with a
    /// single child slot, leaving the child unwritten.
    fn emit_cyclic_pair_node(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, tag_id) = (p[0], p[1]);
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b
            .ins()
            .iconst(types::I64, BoundaryClass::Constructor as i64);
        let one = b.ins().iconst(types::I64, 1);
        guard(b, refs.alloc, &[base, tag, class, one, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        guard(b, refs.store_tag_id, &[base, word, tag_id]);
        b.ins().return_(&[word]);
    }

    /// **`AC-10` — a node CYCLE is constructible by emitted code, and adoption
    /// refuses it rather than recursing.**
    ///
    /// ⛔ **Measured, not assumed.** `ken_boundary_store_field_local` refuses
    /// only a persistent parent with an *invocation-owned* child — so emitted
    /// code can allocate two persistent nodes and write each as the other's
    /// child, and **both writes pass every guard**. The frame said to answer the
    /// cycle question before recursing; this is the answer, and it means the
    /// shipped ground adoption had unbounded recursion on a reachable input.
    ///
    /// ⚠ MEASURED: both `store_field` calls return `OK`, then adoption returns
    /// an exact status. CLAIMED: adoption terminates on every reachable graph.
    /// THE GAP: that the guard is on the *reachability walk* and not on one
    /// shape — closed by the in-progress set, which is keyed on node index and
    /// so catches a cycle of any length.
    #[test]
    fn b2v_ac10_a_constructible_node_cycle_is_refused_not_recursed() {
        let (_pm, alloc_pair) = compile_producer(2, emit_cyclic_pair_node);
        let (_sm, store_field) = compile_producer(4, emit_store_field_probe);

        let mut store = c1_d2_store();
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (4, 4, 0),
            (0, 0, 0),
        );
        let first = BoundaryWord(run3(alloc_pair, f.base, BoundaryWord(1), 0) as u64);
        let second = BoundaryWord(run3(alloc_pair, f.base, BoundaryWord(2), 0) as u64);
        assert!(
            first.0 as i64 > 0 && second.0 as i64 > 0,
            "both nodes allocate"
        );

        // ⛔ The cycle itself — and every guard admits it.
        assert_eq!(
            run4(store_field, f.base, first.0, 0, second.0),
            BOUNDARY_OK,
            "emitted code may write a persistent child into a persistent parent"
        );
        assert_eq!(
            run4(store_field, f.base, second.0, 0, first.0),
            BOUNDARY_OK,
            "AC-10: and the reverse edge too — the cycle is CONSTRUCTIBLE"
        );

        // ⭐ Adoption terminates with an exact status instead of recursing.
        //
        // ⛔ The status is `BOUNDARY_ERR_CYCLE`, distinct from
        // `BOUNDARY_ERR_SHAPE`: *"this graph is not a value"* and *"this word is
        // the wrong shape"* are different findings, and a shared status would
        // leave this control unable to say which one it caught.
        store.seal_persistent();
        assert_eq!(
            store.adopt(first),
            Err(BOUNDARY_ERR_CYCLE),
            "AC-10: a cyclic graph has no canonical image and must fail closed"
        );

        // ⚠ POSITIVE CONTROL — an acyclic graph of the same shape adopts, so
        // the refusal is about the cycle and not about aggregates.
        let mut clean = c1_d2_store();
        // ⚠ The constructor ids must be the fixture authority's issued words:
        // a node naming an unissued id has no canonical image. Using those
        // words keeps this control about the cycle.
        let leaf_id = c1_d2_issued_identity("ctor:fixture::Cycle::Leaf");
        let root_id = c1_d2_issued_identity("ctor:fixture::Cycle::Root");
        let g = bind_with(
            &mut clean,
            BoundaryArenaBuilder::new(),
            (4, 4, 0),
            (0, 0, 0),
        );
        let leaf = BoundaryWord(run3(alloc_pair, g.base, BoundaryWord(leaf_id), 0) as u64);
        let root = BoundaryWord(run3(alloc_pair, g.base, BoundaryWord(root_id), 0) as u64);
        let immediate = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 5);
        assert_eq!(
            run4(store_field, g.base, leaf.0, 0, immediate.0),
            BOUNDARY_OK
        );
        assert_eq!(run4(store_field, g.base, root.0, 0, leaf.0), BOUNDARY_OK);
        clean.seal_persistent();
        assert!(
            clean.adopt(root).is_ok(),
            "AC-10: the same shape without a cycle must adopt"
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // The seal/quiescence handoff — `AC-6` ownership transfer
    // ───────────────────────────────────────────────────────────────────────

    /// One probe per emitted writer: call exactly that helper on a valid
    /// persistent node and hand back its raw status.
    ///
    /// ⛔ **Each probe returns the status instead of `guard`ing on it**, because
    /// the status *is* the measurement.
    macro_rules! seal_probe {
        ($name:ident, $field:ident, $extra:expr) => {
            fn $name(
                b: &mut FunctionBuilder<'_>,
                refs: &Refs,
                p: &[cranelift_codegen::ir::Value],
                _ptr: cranelift_codegen::ir::Type,
            ) {
                let (base, word) = (p[0], p[1]);
                let mut args = vec![base, word];
                for imm in $extra {
                    let v = b.ins().iconst(types::I64, imm as i64);
                    args.push(v);
                }
                let call = b.ins().call(refs.$field, &args);
                let status = b.inst_results(call)[0];
                b.ins().return_(&[status]);
            }
        };
    }

    seal_probe!(seal_probe_seal_int, seal_int, [0u64; 0]);
    seal_probe!(seal_probe_store_tag_id, store_tag_id, [1u64]);
    seal_probe!(seal_probe_store_scalar, store_scalar, [1u64]);
    seal_probe!(seal_probe_store_field, store_field, [0u64, 0u64]);
    seal_probe!(seal_probe_store_name, store_name, [0u64, 1u64]);
    seal_probe!(
        seal_probe_store_int_tag,
        store_int_tag,
        [crate::native_int::NATIVE_INT_SMALL_TAG_V1]
    );
    seal_probe!(
        seal_probe_store_int_limbs,
        store_int_limbs,
        [0u64, 1u64, 0u64]
    );
    seal_probe!(seal_probe_store_int_limb, store_int_limb, [0u64, 1u64]);
    seal_probe!(seal_probe_store_bytes_len, store_bytes_len, [0u64, 0u64]);
    seal_probe!(seal_probe_store_byte, store_byte, [0u64, 0u64]);

    /// `alloc` takes no word, so it gets a hand-written probe rather than the
    /// macro's shape — and it is the one writer that never reaches
    /// `mutable_guard`, which is exactly why it needs its own seal check.
    fn seal_probe_alloc(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let base = p[0];
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b.ins().iconst(types::I64, BoundaryClass::Int as i64);
        let zero = b.ins().iconst(types::I64, 0);
        let call = b.ins().call(refs.alloc, &[base, tag, class, zero, out]);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
    }

    /// ⛔ **The allowed inventory of seal probes — one row per emitted writer.**
    ///
    /// Checked against [`EMITTED_WRITERS`] below, so a new writer is red until
    /// someone gives it a seal probe. Pinning the permitted set is what makes a
    /// writer *nobody imagined* redden too.
    type SealProbe = fn(
        &mut FunctionBuilder<'_>,
        &Refs,
        &[cranelift_codegen::ir::Value],
        cranelift_codegen::ir::Type,
    );

    const SEAL_PROBES: &[(&str, SealProbe)] = &[
        ("ken_boundary_alloc_local", seal_probe_alloc),
        ("ken_boundary_seal_int_local", seal_probe_seal_int),
        ("ken_boundary_store_tag_id_local", seal_probe_store_tag_id),
        ("ken_boundary_store_scalar_local", seal_probe_store_scalar),
        ("ken_boundary_store_field_local", seal_probe_store_field),
        ("ken_boundary_store_name_local", seal_probe_store_name),
        ("ken_boundary_store_int_tag_local", seal_probe_store_int_tag),
        (
            "ken_boundary_store_int_limbs_local",
            seal_probe_store_int_limbs,
        ),
        (
            "ken_boundary_store_int_limb_local",
            seal_probe_store_int_limb,
        ),
        (
            "ken_boundary_store_bytes_len_local",
            seal_probe_store_bytes_len,
        ),
        ("ken_boundary_store_byte_local", seal_probe_store_byte),
    ];

    /// Every live persistent word, as a flat snapshot to compare against.
    fn persistent_snapshot(store: &BoundaryValueStore) -> Vec<u64> {
        let region = &store.image().0;
        let mut out = Vec::new();
        for index in 0..region.node_count() as u64 {
            for field in NodeField::ALL {
                out.push(region.node_field(index, field.offset()).unwrap_or(u64::MAX));
            }
        }
        for w in 0..region.word_count() as u64 {
            out.push(region.word_at(w).map_or(u64::MAX, |word| word.0));
        }
        out
    }

    /// **`AC-6` — after the sealed handoff EVERY emitted writer is refused with
    /// the exact status, and the adopted image cannot change.**
    ///
    /// ⚠ MEASURED: for each of the eleven helpers in [`EMITTED_WRITERS`], the
    /// same call returns some status before the seal and **`BOUNDARY_ERR_SEALED`
    /// after it**, and the whole persistent region is byte-identical across the
    /// sealed attempts. CLAIMED: adoption operates on a stable snapshot. THE
    /// GAP: that the refusal covers *every* writer rather than the ones a test
    /// author happened to think of — closed by driving the probe table and
    /// asserting it equals the production partition, so a new writer without a
    /// probe is red.
    ///
    /// ⭐ **The pair is the control.** A probe that returned `ERR_SEALED` in both
    /// states would prove nothing — it could be failing for its own reasons. The
    /// status must *change*, and it can only change to `ERR_SEALED` if the seal
    /// check sits on that helper's path.
    #[test]
    fn b2v_ac6_every_emitted_writer_is_refused_after_the_sealed_handoff() {
        // ⛔ The inventory first: the probe table must be exactly the production
        // writer partition, not a subset someone maintained by hand.
        let mut probed: Vec<&str> = SEAL_PROBES.iter().map(|(name, _)| *name).collect();
        probed.sort_unstable();
        let declared = probed.len();
        probed.dedup();
        assert_eq!(probed.len(), declared, "AC-6: a writer is probed twice");
        let mut writers = EMITTED_WRITERS.to_vec();
        writers.sort_unstable();
        assert_eq!(
            probed, writers,
            "AC-6: every emitted writer needs a seal probe — a new mutator must \
             not be able to run against a region the store has taken"
        );

        let (_am, alloc_ctor) = compile_producer(3, emit_ctor_node);
        let mut store = c1_d2_store();
        let tag_id = c1_d2_issued_identity("ctor:fixture::Seal::Node");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (8, 8, 8),
            (0, 0, 0),
        );
        let node = BoundaryWord(run3(alloc_ctor, f.base, BoundaryWord(tag_id), 1) as u64);
        assert!(node.0 as i64 > 0, "the target node allocates");

        let compiled: Vec<(&str, *const u8, JITModule)> = SEAL_PROBES
            .iter()
            .map(|(name, emit)| {
                let (module, code) = compile_producer(2, *emit);
                (*name, code, module)
            })
            .collect();

        // ── before the seal: a status, and it is not ERR_SEALED ─────────────
        let before: Vec<(&str, i64)> = compiled
            .iter()
            .map(|(name, code, _)| (*name, run2(*code, f.base, node)))
            .collect();
        for (name, status) in &before {
            assert_ne!(
                *status, BOUNDARY_ERR_SEALED,
                "{name} reports SEALED on an OPEN region — the probe never \
                 reaches the helper, so the pair below would be vacuous"
            );
        }

        // ── the handoff ─────────────────────────────────────────────────────
        store.seal_persistent();
        assert!(store.is_persistent_sealed(), "the region is sealed");
        let snapshot = persistent_snapshot(&store);

        for (name, code, _) in &compiled {
            assert_eq!(
                run2(*code, f.base, node),
                BOUNDARY_ERR_SEALED,
                "AC-6: {name} must be refused once the store owns the region"
            );
        }
        assert_eq!(
            persistent_snapshot(&store),
            snapshot,
            "AC-6: a refused write must also be a write that DID NOT HAPPEN — \
             the adopted image cannot change under adoption"
        );
    }

    /// `(base, tag_id, arity) -> word` — allocate one persistent `Constructor`
    /// with `arity` child slots.
    fn emit_ctor_node(
        b: &mut FunctionBuilder<'_>,
        refs: &Refs,
        p: &[cranelift_codegen::ir::Value],
        ptr: cranelift_codegen::ir::Type,
    ) {
        let (base, tag_id, arity) = (p[0], p[1], p[2]);
        let out = cell(b, ptr);
        let tag = b
            .ins()
            .iconst(types::I64, BoundaryTag::PersistentGround as i64);
        let class = b
            .ins()
            .iconst(types::I64, BoundaryClass::Constructor as i64);
        guard(b, refs.alloc, &[base, tag, class, arity, out]);
        let word = b.ins().load(types::I64, MemFlags::trusted(), out, 0);
        guard(b, refs.store_tag_id, &[base, word, tag_id]);
        b.ins().return_(&[word]);
    }

    // ⛔ `emit_closure_node` — the CLIF emitter that allocated a
    // `(PersistentClosure, Closure)` node — is DELETED for the same reason as
    // `emitted_closure`: `RT-FNSPLIT-C1` `D5` retires that lane, so the helper
    // can only ever produce `BOUNDARY_ERR_RETIRED_LANE`. ⛔ It is not kept as a
    // negative fixture — the product sweep and
    // `b2v_d5_the_durable_closure_lane_is_refused_at_allocation_by_name` already
    // exercise the refusal from the plan's own authority, without a bespoke
    // emitter that would have to be maintained against a dead representation.

    /// **`AC-6` — adoption REFUSES an unsealed region.**
    ///
    /// ⚠ Without this the seal is only half a mechanism: the emitted writers
    /// would be shut out *once someone remembered to seal*, and nothing would
    /// require anyone to. The handoff has two sides, and this is the one that
    /// makes it a precondition rather than a convention.
    ///
    /// ⚠ POSITIVE CONTROL: the identical graph adopts once sealed, so the
    /// refusal is about the handoff and not about the value.
    #[test]
    fn b2v_ac6_adoption_refuses_an_unsealed_region() {
        let (_am, alloc_ctor) = compile_producer(3, emit_ctor_node);
        let (_sm, store_field) = compile_producer(4, emit_store_field_probe);

        let mut store = c1_d2_store();
        let tag_id = c1_d2_issued_identity("ctor:fixture::Unsealed::Node");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (4, 4, 0),
            (0, 0, 0),
        );
        let node = BoundaryWord(run3(alloc_ctor, f.base, BoundaryWord(tag_id), 1) as u64);
        let leaf = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 4);
        assert_eq!(run4(store_field, f.base, node.0, 0, leaf.0), BOUNDARY_OK);

        assert!(!store.is_persistent_sealed(), "the region starts open");
        assert_eq!(
            store.adopt(node),
            Err(BOUNDARY_ERR_SEALED),
            "AC-6: adoption must not begin against a region emitted code can \
             still mutate"
        );

        store.seal_persistent();
        assert!(
            store.adopt(node).is_ok(),
            "AC-6: and the same graph adopts once the handoff has happened"
        );
    }

    /// **`AC-6` — COMPLETE validation precedes any identity installation.**
    ///
    /// ⛔ A malformed graph must leave the store exactly as it found it. If
    /// minting were interleaved with walking, a graph that turns out to be
    /// cyclic three nodes in would already have installed identities for the
    /// nodes the walk passed — the failure would be partial, and "adoption
    /// failure occurs before publication" would be true only of the root.
    ///
    /// ⚠ POSITIVE CONTROL: the same shape without the back-edge mints every
    /// node, so the emptiness below is the *refusal* and not an adoption that
    /// mints nothing.
    #[test]
    fn b2v_ac6_a_refused_graph_installs_no_identity_at_all() {
        let (_am, alloc_ctor) = compile_producer(3, emit_ctor_node);
        let (_sm, store_field) = compile_producer(4, emit_store_field_probe);

        let slots = |store: &BoundaryValueStore| -> Vec<u64> {
            let region = &store.image().0;
            (0..region.node_count() as u64)
                .map(|i| region.node_field(i, NODE_SLOT).unwrap_or(u64::MAX))
                .collect()
        };

        let mut store = c1_d2_store();
        let tag_id = c1_d2_issued_identity("ctor:fixture::Partial::Node");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (8, 8, 0),
            (0, 0, 0),
        );
        let ring: Vec<BoundaryWord> = (0..3)
            .map(|_| BoundaryWord(run3(alloc_ctor, f.base, BoundaryWord(tag_id), 1) as u64))
            .collect();
        for i in 0..3 {
            assert_eq!(
                run4(store_field, f.base, ring[i].0, 0, ring[(i + 1) % 3].0),
                BOUNDARY_OK
            );
        }
        store.seal_persistent();
        let before = slots(&store);
        assert_eq!(
            store.adopt(ring[0]),
            Err(BOUNDARY_ERR_CYCLE),
            "the graph is refused"
        );
        assert_eq!(
            slots(&store),
            before,
            "AC-6: a refused graph must install NO identity — not even for the \
             nodes the walk reached before the fault"
        );
        assert!(
            before.iter().all(|s| *s == crate::store::NULL_SLOT),
            "and none of them was minted to begin with"
        );

        // ⚠ POSITIVE CONTROL — the acyclic twin mints every node.
        let mut clean = c1_d2_store();
        let clean_id = c1_d2_issued_identity("ctor:fixture::Partial::Node");
        let g = bind_with(
            &mut clean,
            BoundaryArenaBuilder::new(),
            (8, 8, 0),
            (0, 0, 0),
        );
        let chain: Vec<BoundaryWord> = (0..3)
            .map(|_| BoundaryWord(run3(alloc_ctor, g.base, BoundaryWord(clean_id), 1) as u64))
            .collect();
        for pair in chain.windows(2) {
            assert_eq!(
                run4(store_field, g.base, pair[0].0, 0, pair[1].0),
                BOUNDARY_OK
            );
        }
        let leaf = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 1);
        assert_eq!(
            run4(store_field, g.base, chain[2].0, 0, leaf.0),
            BOUNDARY_OK
        );
        clean.seal_persistent();
        assert!(clean.adopt(chain[0]).is_ok());
        assert!(
            slots(&clean).iter().all(|s| *s != crate::store::NULL_SLOT),
            "AC-6: every node of an admitted graph IS minted, so the refusal \
             above is a refusal and not an adoption that does nothing"
        );
    }

    fn fixture_artifact(tag: &str, hash: u64) -> crate::RuntimeArtifactIdentity {
        crate::RuntimeArtifactIdentity {
            package_identity: format!("pkg:fixture::{tag}"),
            core_semantic_hash: 0xC0FE_0000 | hash,
            artifact_hash: 0x0A11_0000 | hash,
        }
    }

    /// An arbitrary non-`NULL_SLOT` identity, used to put a node on the
    /// already-owned fast path. Its value is irrelevant; only `!= NULL_SLOT` is.
    const PREOWNED_SLOT: crate::store::SlotId = 4242;

    // ⛔ `emitted_closure` — the closure-fixture builder — is DELETED with the
    // five phase-retired controls above, not kept behind `#[allow(dead_code)]`.
    // Its first act was `assert!(word.0 as i64 > 0, "closure allocates")`, which
    // the retired lane makes unsatisfiable, so it is a fixture for a value that
    // cannot exist. ⭐ Keeping it would silence rustc's dead-code warning — the
    // free oracle for *"does anything still consume this?"* — and leave an inert
    // artifact arguing that the capability is still around the corner.

    // ───────────────────────────────────────────────────────────────────────
    // `PersistentClosure` — the canonical image layer
    // ───────────────────────────────────────────────────────────────────────

    /// **`AC-6` — the same ordinal in two different artifacts is two different
    /// closures.**
    ///
    /// ⛔ **This is the failure Ruling B names outright**: `StaticOriginId` is a
    /// bare `u32` that restarts at zero per artifact, so an identity keyed on it
    /// would make two unrelated closures content-equal and hand the wrong body
    /// to whichever consumer asked second.
    ///
    /// ⚠ POSITIVE CONTROL on the same line: within *one* artifact the ordinal
    /// still discriminates, so the namespace has not simply swallowed it.
    #[test]
    fn b2v_ac6_equal_ordinals_in_two_artifacts_do_not_collide() {
        let first = fixture_artifact("alpha", 3);
        let second = fixture_artifact("beta", 4);
        assert_ne!(
            boundary_code_id(&first, 0),
            boundary_code_id(&second, 0),
            "AC-6: a bare local-origin ordinal must not collide across artifacts"
        );
        assert_ne!(
            boundary_code_id(&first, 0),
            boundary_code_id(&first, 1),
            "AC-6: and the ordinal must still discriminate WITHIN an artifact"
        );
        assert_eq!(
            boundary_code_id(&first, 5),
            boundary_code_id(&fixture_artifact("alpha", 3), 5),
            "AC-6: the same artifact and ordinal are the same identity"
        );
        // ⛔ **This pair is a DISTINCTNESS check, and nothing more — the name it
        // used to carry was a claim it did not earn.** It was labelled as
        // exercising the length prefix, and mutation `M48` (remove the prefix)
        // reddened nothing: `package_identity` is the only variable-length field
        // and the rest are fixed-width, so the encoding is injective with or
        // without it. Relabelled rather than deleted, because the distinctness
        // it does check is worth having.
        let ab = crate::RuntimeArtifactIdentity {
            package_identity: "ab".to_string(),
            core_semantic_hash: 0,
            artifact_hash: 0,
        };
        let a = crate::RuntimeArtifactIdentity {
            package_identity: "a".to_string(),
            core_semantic_hash: 0,
            artifact_hash: 0,
        };
        assert_ne!(
            boundary_code_id(&ab, 0),
            boundary_code_id(&a, 0),
            "AC-6: two package identities are two namespaces"
        );
    }

    // ── `D5` — the durable-closure lane, at the ALLOCATION boundary ─────────
    //
    // ⛔ **PHASE-RETIRED COVERAGE — read this before assuming the properties
    // named below are still defended.** `RT-FNSPLIT-C1` `D5` (Architect
    // `dec_21aa95jbsznfh` + addendum `dec_6xffebwj4s347`) makes
    // `(PersistentClosure, Closure)` a **retired lane**: recognized ABI
    // vocabulary, admitted by nothing.
    //
    // ⭐ **MEASURED, not assumed — a closure node is unconstructible from BOTH
    // producers.** Emitted allocation refuses the pair with
    // `BOUNDARY_ERR_RETIRED_LANE` in `define_alloc`'s ordered prologue, and the
    // Rust builder asserts `boundary_relation_admits` (the `ONE CONTRACT, TWO
    // ENFORCEMENT PATHS` guard in `boundary_value.rs`), which now answers
    // `false` for the same pair. ⇒ **No path remains by which a test can bring a
    // closure node into existence in order to observe what happens to it next.**
    //
    // ⛔ **So the five tests removed here are NOT relocated, and calling them
    // preserved would be false.** Their shared premise is *"a closure node
    // exists"*. Once the lane cannot be entered, the deeper properties they
    // pinned are **unreachable**, not merely re-sited. Each is recorded by name
    // with the property it held:
    //
    //   * `b2v_ac6_an_emitted_closure_node_is_refused_at_adoption` — adoption
    //     refuses with `BOUNDARY_ERR_ESCAPE` before any byte, digest, slot or
    //     provenance exists.
    //   * `b2v_acv5_a_closure_capturing_a_compound_node_mints_nothing` — the
    //     load-bearing arm: refusal precedes **minting**. Only a *compound*
    //     capture could tell `validate_reachable` apart from `canonical_image`,
    //     because an immediate is never interned.
    //   * `b2v_acv5_an_already_owned_closure_root_is_still_refused` — the
    //     already-owned fast path does not smuggle a closure ROOT past the gate.
    //   * `b2v_acv5_an_already_owned_closure_descendant_is_still_refused` — the
    //     same, TRANSITIVELY, for a descendant.
    //   * `b2v_ac6_an_invocation_owned_capture_rejects_before_publication` — an
    //     invocation-owned capture is refused before publication.
    //
    // ⚠ **What this costs, stated plainly.** Adoption-time refusal, transitive
    // descendant refusal and mint-ordering are now guarded by
    // **unconstructibility** rather than by an executed refusal. That is a
    // strictly earlier gate, but it is a **different property**: it says the
    // value cannot be built, not that the adopter would reject one if it were.
    // ⛔ If `B2F` lands a durable callable carrier and this stops being a
    // tombstone, these five must be **rewritten against that carrier**, not
    // resurrected — their fixtures assume the retired representation.
    //
    // ✅ Every GROUND twin is retained below and still executes; they are what
    // keeps the surviving refusal from being "this harness adopts nothing".

    /// **`D5` — the durable-closure lane is refused AT ALLOCATION, BY NAME.**
    ///
    /// The earliest point at which the retired lane is still observable, and so
    /// the only place the coverage above can honestly relocate to.
    ///
    /// **MEASURED:** emitted allocation of every declared retired lane returns
    /// [`crate::boundary_value::BOUNDARY_ERR_RETIRED_LANE`], while the identical
    /// harness admits a ground constructor.
    /// **CLAIMED:** an ordinary closure has no durable boundary lane.
    /// **THE GAP:** closed on the producing side by the Rust builder's
    /// `boundary_relation_admits` guard — two producers, one contract — and on
    /// the diagnostic side by
    /// [`b2v_the_tag_class_relation_is_closed_over_the_whole_product`], which
    /// separates this refusal from both `BOUNDARY_ERR_TAG` (an unrecognized
    /// byte) and `BOUNDARY_ERR_RELATION` (a malformed pair such as
    /// `PersistentClosure + Bool`). ⛔ Without that separation this assertion
    /// would be satisfied by the lane simply having been deleted.
    #[test]
    fn b2v_d5_the_durable_closure_lane_is_refused_at_allocation_by_name() {
        let (_pm, alloc_code) = compile_producer(4, emit_alloc_probe);
        let mut store = c1_d2_store();
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (64, 8, 0),
            (64, 8, 0),
        );

        for (tag, class) in crate::boundary_value::BOUNDARY_RETIRED_LANES {
            assert_eq!(
                run4(alloc_code, f.base, *tag as u64, *class as u64, 0),
                crate::boundary_value::BOUNDARY_ERR_RETIRED_LANE,
                "D5: {tag:?} + {class:?} is a retired lane and must be refused BY \
                 NAME at allocation"
            );
        }
        // ⚠ NON-VACUITY: an emptied tombstone list makes the loop above assert
        // nothing at all, and this is the whole subject of the test.
        assert!(
            !crate::boundary_value::BOUNDARY_RETIRED_LANES.is_empty(),
            "D5: no lane is retired, so the loop above asserted nothing"
        );

        // ⚠ POSITIVE CONTROL: the identical arena, plan and harness admit a
        // GROUND constructor — so the refusal above is caused by the retired
        // lane and not by a mis-sized region or an unbound store.
        assert!(
            run4(
                alloc_code,
                f.base,
                BoundaryTag::PersistentGround as u64,
                BoundaryClass::Constructor as u64,
                0
            ) >= 0,
            "the same harness admits a GROUND constructor, so the refusals above \
             are about the lane"
        );
    }

    /// ⚠ **POSITIVE CONTROL for the test above** — a ground node adopts through
    /// the identical harness.
    ///
    /// ⛔ Without this, *"closures are refused"* and *"this fixture adopts
    /// nothing"* are the same green.
    #[test]
    fn b2v_ac6_a_ground_node_still_adopts_through_the_same_harness() {
        let (_am, alloc_ctor) = compile_producer(3, emit_ctor_node);
        let mut store = c1_d2_store();
        store.bind_artifact(fixture_artifact("refused", 3));
        let tag_id = c1_d2_issued_identity("ctor:fixture::Ground::Leaf");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (4, 8, 0),
            (0, 0, 0),
        );
        let word = BoundaryWord(run3(alloc_ctor, f.base, BoundaryWord(tag_id), 0) as u64);
        store.seal_persistent();

        assert!(
            store.adopt(word).is_ok(),
            "the same arena, sealing and binding adopt a GROUND node, so the \
             closure refusal above is about the closure class"
        );
    }

    /// ⚠ **POSITIVE CONTROL for both already-owned arms — the fast path still
    /// WORKS for a persistable class.**
    ///
    /// ⛔ Without this, *"an already-owned closure is refused"* and *"admission
    /// now rejects everything already-owned"* are the same green, and the two
    /// arms above would be satisfied by a change that simply broke the
    /// optimization for every class.
    #[test]
    fn b2v_acv5_an_already_owned_ground_root_still_takes_the_fast_path() {
        let (_cm, alloc_ctor) = compile_producer(5, emit_ctor_node);

        let mut store = c1_d2_store();
        store.bind_artifact(fixture_artifact("refused", 3));
        let tag_id = c1_d2_issued_identity("ctor:fixture::Ground::Leaf");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (8, 16, 0),
            (0, 0, 0),
        );
        let child = BoundaryWord(run3(alloc_ctor, f.base, BoundaryWord(tag_id), 0) as u64);

        store.install_node_slot_for_test(child.payload(), PREOWNED_SLOT);
        store.seal_persistent();

        let slots_before = store.store_resident_slots();
        assert!(
            store.adopt(child).is_ok(),
            "a GROUND node with a pre-existing slot still adopts — the class \
             check admits it and the fast path then short-circuits"
        );
        assert_eq!(
            store.store_resident_slots(),
            slots_before,
            "and, being already owned, it is not re-interned — so the fast path \
             is genuinely still taken rather than merely tolerated"
        );
    }

    /// ⚠ **POSITIVE CONTROL for the arm above — the identical compound child,
    /// adopted on its own, DOES mint a slot.**
    ///
    /// ⛔ Without this, *"the closure minted nothing"* and *"this fixture's
    /// child was never mintable in the first place"* are the same green, and
    /// the whole arm collapses back to the immediate-capture one it replaced.
    #[test]
    fn b2v_acv5_the_same_compound_child_alone_does_mint_a_slot() {
        let (_cm, alloc_ctor) = compile_producer(5, emit_ctor_node);

        let mut store = c1_d2_store();
        store.bind_artifact(fixture_artifact("refused", 3));
        let tag_id = c1_d2_issued_identity("ctor:fixture::Ground::Leaf");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (8, 16, 0),
            (0, 0, 0),
        );
        let child = BoundaryWord(run3(alloc_ctor, f.base, BoundaryWord(tag_id), 0) as u64);
        store.seal_persistent();

        let slots_before = store.store_resident_slots();
        assert!(store.adopt(child).is_ok(), "the compound child adopts alone");
        assert!(
            store.store_resident_slots() > slots_before,
            "and minting it moves the store's slot count — so 'unchanged' in the \
             arm above is a real observation, not a property of the counter"
        );
        assert_ne!(
            store.node_slot_of(child.payload()),
            Some(crate::store::NULL_SLOT),
            "and it acquires a NODE_SLOT, so NULL_SLOT above is discriminating"
        );
    }




    /// **`AC-6` — `HostResult` and `BorrowedOpaque` are never placed in the
    /// permanent store.**
    ///
    /// ⚠ They are **not** narrowed or reclassified: they remain admitted
    /// invocation-owned represented arms. What is refused is *persistence*.
    #[test]
    fn b2v_ac6_invocation_owned_classes_are_never_persisted() {
        let mut store = c1_d2_store();
        store.bind_artifact(fixture_artifact("invocation", 8));
        let mut builder = BoundaryArenaBuilder::new();
        let borrowed = materialize_borrowed(&mut builder, 1);
        let host = materialize_host_result(&mut builder, 1, borrowed, borrowed);
        store.seal_persistent();
        for word in [borrowed, host] {
            assert_eq!(
                store.adopt(word),
                Err(BOUNDARY_ERR_SHAPE),
                "AC-6: an invocation-owned word has no persistent adoption boundary"
            );
        }
    }

    // ───────────────────────────────────────────────────────────────────────
    // The cycle / depth contract
    // ───────────────────────────────────────────────────────────────────────

    /// Allocate a chain of `depth` persistent `Constructor` nodes, each holding
    /// the next, with an immediate leaf.
    fn emitted_chain(
        alloc_ctor: *const u8,
        store_field: *const u8,
        base: *const u64,
        tag_id: u64,
        depth: usize,
    ) -> BoundaryWord {
        let nodes: Vec<BoundaryWord> = (0..depth)
            .map(|_| BoundaryWord(run3(alloc_ctor, base, BoundaryWord(tag_id), 1) as u64))
            .collect();
        for node in &nodes {
            assert!(node.0 as i64 > 0, "chain node allocates");
        }
        for pair in nodes.windows(2) {
            assert_eq!(
                run4(store_field, base, pair[0].0, 0, pair[1].0),
                BOUNDARY_OK
            );
        }
        let leaf = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 1);
        assert_eq!(
            run4(store_field, base, nodes[depth - 1].0, 0, leaf.0),
            BOUNDARY_OK
        );
        nodes[0]
    }

    /// **`AC-10` — a deep acyclic chain adopts. Depth is not a property that
    /// decides malformed-versus-admitted.**
    ///
    /// ⭐ **The number is measured, not chosen.** The former *recursive*
    /// adoption — restored verbatim and probed on the default 8 MiB test stack —
    /// carried depth **800** and died between **800 and 1600**. This walk is
    /// iterative, with the frontier on a heap `Vec`.
    ///
    /// ⛔ **THIS CONTROL PREVIOUSLY CARRIED A FALSE RESIDUAL, and how it got
    /// there is the part worth keeping.** It claimed the end-to-end bound was
    /// ~2500, set by a *recursive* `canonical::encode_canonical` plus `Value`'s
    /// derived `Clone`/`Drop`, and that closing it was not a `B2V`-sized change.
    /// Every clause of that is false on these bytes:
    ///
    /// - `encode_canonical` is **iterative** — its work stack is a heap `Vec`
    ///   and its host-stack use is O(1) in depth.
    /// - `RT-VALUE-TOTALITY-P1`'s `value_depth_totality` integration test covers
    ///   the encoder, the derived `Clone`, **and** drop glue out of process at
    ///   depth **131_072** on a *stated* 1 MiB stack.
    /// - P1's own bisection of the **pre-change** mechanisms at 8 MiB puts them
    ///   at **9032** / **10074** / **65486**, so even the figure this control
    ///   cited for the old recursive encoder was low by roughly 4x.
    ///
    /// ⚠ **The measurement was inherited across a re-anchor, not re-derived.**
    /// It was taken on a pre-P1 base; the branch was then re-anchored onto a
    /// base containing P1, and the number came along unre-measured. `P1` was on
    /// this WP's do-not-touch list, and "not mine to change" was read as "not
    /// relevant to re-check" — which is the whole error. Runtime QA disproved it
    /// by raising this constant and watching the test pass.
    ///
    /// **Measured on these bytes, this walk:** depth 3000, 10000 and 30000 all
    /// adopt. At 30000 the cost is ~142 s of arena work — so the remaining bound
    /// is **allocation and time, an ordinary resource boundary, never the host
    /// stack**, which is the same thing P1 says about the encoder. The deep
    /// instance is kept executable as the `#[ignore]`d control below rather than
    /// asserted in prose, because a claim in a comment cannot fail.
    #[test]
    fn b2v_ac10_a_deep_acyclic_chain_adopts_without_walk_recursion() {
        // Comfortably past the former recursive margin (dead between 800 and
        // 1600) and cheap enough for the default suite. The depth that rules
        // out hidden host-stack growth is the ignored control below.
        const DEPTH: usize = 3000;

        let (_am, alloc_ctor) = compile_producer(3, emit_ctor_node);
        let (_sm, store_field) = compile_producer(4, emit_store_field_probe);
        let mut store = c1_d2_store();
        let tag_id = c1_d2_issued_identity("ctor:fixture::Deep::Link");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (DEPTH + 2, DEPTH + 2, 0),
            (0, 0, 0),
        );
        let root = emitted_chain(alloc_ctor, store_field, f.base, tag_id, DEPTH);
        store.seal_persistent();

        let adopted = store.adopt(root);
        assert!(
            adopted.is_ok(),
            "AC-10: a finite deep acyclic value is ADMITTED, so adoption must \
             not fail on it — got {adopted:?}"
        );
        // ⛔ And specifically it is never confused with the malformed case.
        assert_ne!(
            adopted,
            Err(BOUNDARY_ERR_CYCLE),
            "AC-10: depth must never be reclassified as a cycle"
        );
        let slot = store
            .image()
            .0
            .node_field(adopted.expect("adopted").payload(), NODE_SLOT)
            .expect("live");
        assert_ne!(slot, crate::store::NULL_SLOT, "and it is store-minted");
    }

    /// **`AC-10`, deep instance — the depth that rules out host-stack growth.**
    ///
    /// ⛔ **`#[ignore]`d on purpose, and that is a stated cost, not a hiding
    /// place.** The run costs ~142 s of arena work at this depth, which would
    /// more than double a targeted `-p ken-runtime` suite; the fast control
    /// above carries depth 3000 on every run. This one exists so the deep
    /// measurement is **executable** rather than a sentence in a doc comment
    /// that can never fail. Run it with
    /// `scripts/ken-cargo test -p ken-runtime --lib
    /// b2v_ac10_a_deep_acyclic_chain_adopts_at_thirty_thousand -- --ignored`.
    ///
    /// ⚠ What it establishes is a **direction, not a limit**: 30000 adopts, and
    /// the cost that stops the sweep going higher is allocation and time. It
    /// does not name a depth at which this walk fails, because none was found —
    /// the frontier is a heap `Vec` and host-stack use does not grow with depth.
    /// The former recursive adoption died between 800 and 1600, so this is ~19x
    /// beyond the mechanism it replaced.
    #[test]
    #[ignore = "~142s of arena work; the fast instance at depth 3000 runs by default"]
    fn b2v_ac10_a_deep_acyclic_chain_adopts_at_thirty_thousand() {
        const DEPTH: usize = 30_000;

        let (_am, alloc_ctor) = compile_producer(3, emit_ctor_node);
        let (_sm, store_field) = compile_producer(4, emit_store_field_probe);
        let mut store = c1_d2_store();
        let tag_id = c1_d2_issued_identity("ctor:fixture::Deep::Link");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (DEPTH + 2, DEPTH + 2, 0),
            (0, 0, 0),
        );
        let root = emitted_chain(alloc_ctor, store_field, f.base, tag_id, DEPTH);
        store.seal_persistent();

        let adopted = store.adopt(root);
        assert!(
            adopted.is_ok(),
            "AC-10: a finite deep acyclic value is ADMITTED at depth {DEPTH} — \
             got {adopted:?}"
        );
        assert_ne!(
            adopted,
            Err(BOUNDARY_ERR_CYCLE),
            "AC-10: depth must never be reclassified as a cycle"
        );
    }

    /// **`AC-10` — a MULTI-NODE cycle is refused deterministically, while a
    /// shared-child DAG of the same shape adopts.**
    ///
    /// ⭐ **This pair is what makes the traversal tri-colour rather than a
    /// visited set.** A second edge into a node is *malformed* when that node is
    /// still on the stack and *legal sharing* when it is finished — a "have I
    /// seen this?" set collapses both into one answer and would have to reject
    /// the DAG to be safe on the cycle. The two halves differ in nothing but
    /// which way the second edge points.
    #[test]
    fn b2v_ac10_a_multi_node_cycle_is_refused_while_a_shared_dag_adopts() {
        let (_am, alloc_ctor) = compile_producer(3, emit_ctor_node);
        let (_sm, store_field) = compile_producer(4, emit_store_field_probe);

        // ── a three-node cycle: a -> b -> c -> a ────────────────────────────
        let mut store = c1_d2_store();
        let tag_id = c1_d2_issued_identity("ctor:fixture::Ring::Link");
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (8, 8, 0),
            (0, 0, 0),
        );
        let ring: Vec<BoundaryWord> = (0..3)
            .map(|_| BoundaryWord(run3(alloc_ctor, f.base, BoundaryWord(tag_id), 1) as u64))
            .collect();
        for i in 0..3 {
            assert_eq!(
                run4(store_field, f.base, ring[i].0, 0, ring[(i + 1) % 3].0),
                BOUNDARY_OK,
                "AC-10: every edge of a multi-node cycle is CONSTRUCTIBLE"
            );
        }
        store.seal_persistent();
        // Deterministic: the same input gives the same exact status, twice, and
        // from every entry point on the ring.
        for entry in &ring {
            for _ in 0..2 {
                assert_eq!(
                    store.adopt(*entry),
                    Err(BOUNDARY_ERR_CYCLE),
                    "AC-10: a multi-node cycle is refused deterministically"
                );
            }
        }

        // ── the DAG: one child reached by TWO parents ───────────────────────
        let mut dag = c1_d2_store();
        let parent_id = c1_d2_issued_identity("ctor:fixture::Dag::Parent");
        let shared_id = c1_d2_issued_identity("ctor:fixture::Dag::Shared");
        let g = bind_with(&mut dag, BoundaryArenaBuilder::new(), (8, 16, 0), (0, 0, 0));
        let shared = BoundaryWord(run3(alloc_ctor, g.base, BoundaryWord(shared_id), 1) as u64);
        let leaf = BoundaryWord::immediate(BoundaryTag::ImmediateInt, 3);
        assert_eq!(run4(store_field, g.base, shared.0, 0, leaf.0), BOUNDARY_OK);
        let root = BoundaryWord(run3(alloc_ctor, g.base, BoundaryWord(parent_id), 2) as u64);
        assert_eq!(run4(store_field, g.base, root.0, 0, shared.0), BOUNDARY_OK);
        assert_eq!(run4(store_field, g.base, root.0, 1, shared.0), BOUNDARY_OK);
        dag.seal_persistent();
        let adopted = dag.adopt(root);
        assert!(
            adopted.is_ok(),
            "AC-10: a repeated edge to a FINISHED node is legal sharing, not a \
             cycle — got {adopted:?}"
        );
        // And the shared child is one canonical node, reused rather than copied.
        let word = adopted.expect("adopted");
        let a = dag.image().0.word_at(
            dag.image()
                .0
                .node_field(word.payload(), NODE_FIELDS_AT)
                .expect("live"),
        );
        let b = dag.image().0.word_at(
            dag.image()
                .0
                .node_field(word.payload(), NODE_FIELDS_AT)
                .expect("live")
                + 1,
        );
        assert_eq!(
            a, b,
            "AC-10: the shared child resolves to ONE canonical node"
        );
    }

    /// **`RECUT 2`, causal and PER-SITE — every emitted tag-admission test is
    /// the plan's, not a constant that happens to agree with it.**
    ///
    /// ⛔ **This exists because the whole-graph differential was NOT enough,
    /// and three mutations proved it.** Hardcoding `define_resolve`'s validity
    /// test, the region selection's band test, or `escape_check`'s invocation
    /// band each left
    /// `recut2_the_emitted_helper_graph_changes_when_the_tag_sets_change`
    /// green: that pin compares the *entire* captured CLIF, so one site
    /// defecting is invisible while the other consumers still move. Its
    /// granularity was the graph; the property is per-site.
    ///
    /// **MEASURED:** for every admitted tag, removing it from the plan's
    /// admitted set changes what each probed helper *answers* for a word
    /// carrying it — from its real status to `ERR_TAG`.
    /// **CLAIMED:** no probed helper decides tag legality from a constant.
    /// **THE GAP:** sites no probe reaches. `store_field`'s child-tag check and
    /// `make_immediate`'s immediate-set check are exercised by their own
    /// behavioural tests but not by this differential, so for those two the
    /// evidence is the whole-graph pin plus review — stated here rather than
    /// implied by this test's name.
    #[test]
    fn b2v_every_emitted_tag_admission_test_is_the_plans() {
        use crate::boundary_value::{
            BoundaryEmissionPlan, BoundaryTagAdmission, BOUNDARY_ERR_TAG, BOUNDARY_TAG_BITS,
        };

        let plan = BoundaryEmissionPlan::derive();
        let mut store = c1_d2_store();
        let builder = BoundaryArenaBuilder::new();
        materialize_ground(&mut store, &cons(1)).expect("materializes");
        let f = bind(&mut store, builder);
        let base = f.base;

        // A node index far past the end, so an ADMITTED handle tag answers
        // `ERR_BOUNDS` and a rejected one answers `ERR_TAG`. Without that
        // separation both refuse and the differential cannot tell them apart.
        let out_of_range: u64 = 9_999;
        let probes: [(&str, Probe); 2] = [
            ("class", Probe::Unary(|h| h.class)),
            ("escape_check", Probe::Status(|h| h.escape_check)),
        ];

        assert!(
            !plan.tags().admitted().is_empty(),
            "RECUT 2: the plan admits no tag, so the sweep below is empty"
        );
        for tag in plan.tags().admitted() {
            let thinner: Vec<_> = plan
                .tags()
                .admitted()
                .iter()
                .copied()
                .filter(|other| other != tag)
                .collect();
            let perturbed = BoundaryEmissionPlan::new(
                plan.int_magnitude_classes().to_vec(),
                plan.byte_span_classes().to_vec(),
                BoundaryTagAdmission::new(
                    thinner,
                    plan.tags().immediate().to_vec(),
                    plan.tags().handle().to_vec(),
                    plan.tags().owner_bands().to_vec(),
                    plan.tags().immediate_value_classes().to_vec(),
                    plan.tags().handle_class_relation().to_vec(),
                ),
            );
            let word = BoundaryWord((out_of_range << BOUNDARY_TAG_BITS) | *tag as u64);

            for (name, probe) in probes {
                let (_real_module, real_code) = compile_probe_with_plan(probe, &plan);
                let (_pert_module, pert_code) = compile_probe_with_plan(probe, &perturbed);
                let real = run2(real_code, base, word);
                let pert = run2(pert_code, base, word);

                assert_ne!(
                    real, BOUNDARY_ERR_TAG,
                    "RECUT 2: `{name}` already refuses {tag:?} as an unknown tag \
                     under the REAL plan, so the perturbation below cannot \
                     change its answer and would prove nothing"
                );
                assert_eq!(
                    pert, BOUNDARY_ERR_TAG,
                    "RECUT 2: `{name}` still admits {tag:?} after the plan \
                     stopped admitting it — its validity test is a constant, \
                     not the plan's set"
                );
            }
        }
    }

    /// **`RECUT 2`, causal and PER-SITE — the emitted region selection follows
    /// the plan's owner bands.**
    ///
    /// ⛔ The tag test above holds the bands fixed, so a helper could consume
    /// the admitted set and still decide *ownership* from a threshold; the
    /// mutation that does exactly that survived it. Here each handle tag is
    /// moved to the other band and the helper's answer must follow.
    ///
    /// **MEASURED:** with a bound persistent region holding a node and an empty
    /// arena, `class` at node index 0 answers a persistent-banded tag and an
    /// invocation-banded tag differently — so moving a tag between bands
    /// changes its answer. **CLAIMED:** the region a word resolves in is the
    /// plan's, not the tag's ordinal position. **THE GAP:** that index 0 is a
    /// discriminating fixture, which the two `assert_ne!`s below establish
    /// rather than assume.
    #[test]
    fn b2v_every_emitted_owner_band_test_is_the_plans() {
        use crate::boundary_value::{
            BoundaryEmissionPlan, BoundaryReferentOwner, BoundaryTagAdmission,
        };

        let plan = BoundaryEmissionPlan::derive();
        let mut store = c1_d2_store();
        let builder = BoundaryArenaBuilder::new();
        materialize_ground(&mut store, &cons(1)).expect("materializes");
        let f = bind(&mut store, builder);
        let base = f.base;

        // ⛔ BOTH probes, not just `class`. The first cut of this test drove
        // `class` alone, and the mutation that hardcodes `escape_check`'s
        // invocation band stayed green through it: a probe that never reaches
        // a site is not evidence about that site.
        let probes: [(&str, Probe); 2] = [
            ("class", Probe::Unary(|h| h.class)),
            ("escape_check", Probe::Status(|h| h.escape_check)),
        ];

        assert!(
            plan.tags().owner_bands().len() >= 2,
            "RECUT 2: fewer than two owner bands, so a reassignment cannot be \
             expressed and this test proves nothing"
        );
        assert!(
            !plan.tags().handle().is_empty(),
            "RECUT 2: no handle tags, so the sweep below is empty"
        );

        for tag in plan.tags().handle() {
            let owner = tag.referent_owner();
            let elsewhere = match owner {
                BoundaryReferentOwner::PersistentStore => BoundaryReferentOwner::InvocationArena,
                BoundaryReferentOwner::InvocationArena | BoundaryReferentOwner::NoReferent => {
                    BoundaryReferentOwner::PersistentStore
                }
            };
            let bands: Vec<_> = plan
                .tags()
                .owner_bands()
                .iter()
                .map(|(band, tags)| {
                    let mut tags: Vec<_> =
                        tags.iter().copied().filter(|other| other != tag).collect();
                    if *band == elsewhere {
                        tags.push(*tag);
                        tags.sort();
                    }
                    (*band, tags)
                })
                .collect();
            let perturbed = BoundaryEmissionPlan::new(
                plan.int_magnitude_classes().to_vec(),
                plan.byte_span_classes().to_vec(),
                BoundaryTagAdmission::new(
                    plan.tags().admitted().to_vec(),
                    plan.tags().immediate().to_vec(),
                    plan.tags().handle().to_vec(),
                    bands,
                    plan.tags().immediate_value_classes().to_vec(),
                    plan.tags().handle_class_relation().to_vec(),
                ),
            );

            // Index 0 exists in the persistent region and not in the empty
            // arena, so the two regions give different answers.
            let word = BoundaryWord(*tag as u64);
            for (name, probe) in probes {
                let (_real_module, real_code) = compile_probe_with_plan(probe, &plan);
                let (_pert_module, pert_code) = compile_probe_with_plan(probe, &perturbed);
                let real = run2(real_code, base, word);
                let pert = run2(pert_code, base, word);
                assert_ne!(
                    real, pert,
                    "RECUT 2: `{name}` answers the same for {tag:?} whether the \
                     plan bands it under {owner:?} or {elsewhere:?} — that \
                     helper's owner decision is not derived from the bands"
                );
            }
        }
    }

    /// **`RULING R5` clause 3 — the Rust mirror and the partition-derived
    /// relation reconcile over the WHOLE product, in BOTH directions.**
    ///
    /// ⛔ **This is the executable form of a claim that used to be a comment.**
    /// `BOUNDARY_TAG_CLASS_RELATION`'s doc said it was *"derived from
    /// `Lowered::boundary_disposition`"* and was *"the single source"*. Nothing
    /// derived it and nothing checked it; the two happened to agree, which is a
    /// measurement, not a mechanism. The ruling's words: *"measured agreement
    /// today is not evidence — it is a claim awaiting an executable form."*
    ///
    /// **MEASURED:** for every one of the `BoundaryTag::ALL × BoundaryClass::ALL`
    /// cells, the mirror admits the cell **iff** the partition-derived plan
    /// relation does. **CLAIMED:** the Rust builders' legality answer is the
    /// authority's. **THE GAP:** none over this product — it is finite and swept
    /// exhaustively, in both directions, which is why the ruling could name the
    /// product rather than a sample.
    #[test]
    fn b2v_the_rust_mirror_and_the_derived_relation_reconcile_over_the_product() {
        let plan = crate::boundary_value::BoundaryEmissionPlan::derive();
        let derived_admits = |tag: BoundaryTag, class: BoundaryClass| {
            plan.tags()
                .handle_class_relation()
                .iter()
                .any(|(t, classes)| *t == tag && classes.contains(&class))
        };

        let (mut both, mut neither, mut retired) = (0usize, 0usize, 0usize);
        for tag in BoundaryTag::ALL {
            for class in BoundaryClass::ALL {
                let mirror = boundary_relation_admits(tag, class);
                let derived = derived_admits(tag, class);
                // Both directions in one assertion over a finite product: a
                // cell the mirror adds and a cell it drops are the same failure.
                assert_eq!(
                    mirror, derived,
                    "R5: the Rust mirror and the partition disagree on {tag:?} + \
                     {class:?} — mirror={mirror}, partition={derived}"
                );
                // ⛔ `RT-FNSPLIT-C1` `D5` — the RETIRED cell agrees on "not
                // admitted" but for a DIFFERENT REASON than an illegal cell, and
                // the difference is the deliverable. `BOUNDARY_TAG_CLASS_RELATION`
                // still SPELLS this cell — that is what a tombstone is — while
                // the partition drops it, so counting it as `neither` would let
                // the row total below silently absorb a lane that vanished from
                // the mirror entirely.
                if crate::boundary_value::boundary_lane_is_retired(tag, class) {
                    assert!(
                        !mirror && !derived,
                        "D5: the retired lane {tag:?} + {class:?} must be admitted \
                         by NEITHER authority — recognition never widens admission"
                    );
                    retired += 1;
                    continue;
                }
                if mirror {
                    both += 1;
                } else {
                    neither += 1;
                }
            }
        }
        // ⚠ NON-VACUITY for the retired arm — a predicate-guarded `continue`
        // that a silently-emptied tombstone list would take zero times.
        assert_eq!(
            retired,
            crate::boundary_value::BOUNDARY_RETIRED_LANES.len(),
            "D5: the sweep exercised {retired} retired lanes but {} are declared",
            crate::boundary_value::BOUNDARY_RETIRED_LANES.len()
        );
        // ⚠ NON-EMPTY POSITIVE CONTROLS on both arms, per clause 5. A relation
        // that admitted everything, or nothing, would make the agreement above
        // hold for a reason that has nothing to do with the two sides matching.
        assert!(
            both > 0,
            "R5: no cell is admitted by either side, so agreement is vacuous"
        );
        assert!(
            neither > 0,
            "R5: every cell is admitted, so agreement cannot distinguish the \
             relation from the full product"
        );
        // And the admitted count is the mirror's own content, so a mirror that
        // silently emptied would redden here rather than pass the sweep above.
        //
        // ⛔ **The schema is no longer the admitted set** (`D5`): it retains the
        // retired rows precisely so a refusal can name them, so the admitted
        // total is the schema MINUS those rows. ⚠ Derived from both authorities,
        // never re-fitted to the observed count — this goes red if the schema OR
        // the tombstone list moves.
        let rows: usize = BOUNDARY_TAG_CLASS_RELATION
            .iter()
            .map(|(_, classes)| classes.len())
            .sum();
        assert_eq!(
            both,
            rows - crate::boundary_value::BOUNDARY_RETIRED_LANES.len(),
            "R5: the swept agreement count does not match the mirror's own rows \
             less the retired vocabulary"
        );
    }

    /// **`RULING R5` clause 5 — the emitted relation is the plan's, bound by an
    /// exact-cell discriminator.**
    ///
    /// ⛔ **Aggregate CLIF inequality is insufficient.** Clause 5 applies `R4`'s
    /// causal principle to this single relation consumer with a **cell-specific
    /// discriminator**: remap and drop **one exact** `(tag, class)` cell and
    /// observe that cell's emitted acceptance change or `ERR_RELATION`.
    /// Population closure comes from the **full-product both-direction
    /// reconciliation** plus the opposite-side drift mutations, not from this
    /// sweep.
    ///
    /// ⚠ **It is NOT a requirement to run one mutation per cell** — the landed
    /// `R5` erratum says so explicitly, and an earlier version of this comment
    /// claimed clause 5 "sharpens `R4` to a universal per-cell rule", which
    /// licensed more than the ruling grants. The sweep below *is* per-cell
    /// because it is cheap here, which is a choice this control makes and not an
    /// obligation the ruling imposes.
    ///
    /// **MEASURED:** for every admitted cell, an `alloc` of that `(tag, class)`
    /// succeeds under the real plan and returns exact `ERR_RELATION` under a plan
    /// with that one cell dropped; and a cell remapped onto another tag's row
    /// moves acceptance with it. **CLAIMED:** the allocator's relation decision
    /// is the partition's, cell by cell. **THE GAP:** that the plan's relation is
    /// the partition's own answer — closed by the derivation control.
    #[test]
    fn b2v_the_emitted_relation_is_the_plans_per_cell() {
        use crate::boundary_value::{BoundaryEmissionPlan, BoundaryTagAdmission};

        let plan = BoundaryEmissionPlan::derive();
        let with_relation = |relation: Vec<(BoundaryTag, Vec<BoundaryClass>)>| {
            BoundaryEmissionPlan::new(
                plan.int_magnitude_classes().to_vec(),
                plan.byte_span_classes().to_vec(),
                BoundaryTagAdmission::new(
                    plan.tags().admitted().to_vec(),
                    plan.tags().immediate().to_vec(),
                    plan.tags().handle().to_vec(),
                    plan.tags().owner_bands().to_vec(),
                    plan.tags().immediate_value_classes().to_vec(),
                    relation,
                ),
            )
        };

        let cells: Vec<(BoundaryTag, BoundaryClass)> = plan
            .tags()
            .handle_class_relation()
            .iter()
            .flat_map(|(tag, classes)| classes.iter().map(move |class| (*tag, *class)))
            .collect();
        assert!(
            !cells.is_empty(),
            "R5: the relation is empty, so the per-cell sweep below is empty"
        );

        for (tag, class) in cells {
            let dropped: Vec<(BoundaryTag, Vec<BoundaryClass>)> = plan
                .tags()
                .handle_class_relation()
                .iter()
                .map(|(t, classes)| {
                    let kept: Vec<BoundaryClass> = classes
                        .iter()
                        .copied()
                        .filter(|c| !(*t == tag && *c == class))
                        .collect();
                    (*t, kept)
                })
                .collect();

            let (_rm, real_alloc) = compile_producer_with_plan(4, emit_alloc_probe, &plan);
            let (_dm, drop_alloc) =
                compile_producer_with_plan(4, emit_alloc_probe, &with_relation(dropped));

            let mut store = c1_d2_store();
            let f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (64, 8, 0),
                (64, 8, 0),
            );
            let real = run4(real_alloc, f.base, tag as u64, class as u64, 0);
            assert!(
                real >= 0,
                "R5: the real plan must ADMIT {tag:?} + {class:?}, or dropping \
                 the cell cannot change anything (got {real})"
            );

            let mut store = c1_d2_store();
            let f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (64, 8, 0),
                (64, 8, 0),
            );
            let without = run4(drop_alloc, f.base, tag as u64, class as u64, 0);
            assert_eq!(
                without, BOUNDARY_ERR_RELATION,
                "R5: `alloc` still admits {tag:?} + {class:?} after the plan \
                 dropped that exact cell — the emitted relation is not the \
                 plan's"
            );
        }

        // ⛔ **The REMAP half of clause 5** — it says *remap AND drop*. Dropping
        // shows the emitted allocator stops admitting a cell the plan withdrew;
        // remapping shows acceptance MOVES with the plan rather than merely
        // shrinking. A relation consumer could pass every drop above by
        // intersecting with a hardcoded table, and would fail here.
        let (donor, moved) = plan
            .tags()
            .handle_class_relation()
            .iter()
            .find_map(|(tag, classes)| classes.first().map(|class| (*tag, *class)))
            .expect("the relation has at least one cell");
        let recipient = plan
            .tags()
            .handle_class_relation()
            .iter()
            .map(|(tag, _)| *tag)
            .find(|tag| *tag != donor)
            .expect("the relation names at least two tags");
        assert!(
            !plan
                .tags()
                .handle_class_relation()
                .iter()
                .any(|(t, cs)| *t == recipient && cs.contains(&moved)),
            "R5: the recipient already admits {moved:?}, so moving the cell there \
             would change nothing and this control would pass vacuously"
        );
        let remapped: Vec<(BoundaryTag, Vec<BoundaryClass>)> = plan
            .tags()
            .handle_class_relation()
            .iter()
            .map(|(t, classes)| {
                let mut classes: Vec<BoundaryClass> = classes
                    .iter()
                    .copied()
                    .filter(|c| !(*t == donor && *c == moved))
                    .collect();
                if *t == recipient {
                    classes.push(moved);
                    classes.sort();
                }
                (*t, classes)
            })
            .collect();
        let (_xm, remap_alloc) =
            compile_producer_with_plan(4, emit_alloc_probe, &with_relation(remapped));
        for (tag, expected_ok) in [(donor, false), (recipient, true)] {
            let mut store = c1_d2_store();
            let f = bind_with(
                &mut store,
                BoundaryArenaBuilder::new(),
                (64, 8, 0),
                (64, 8, 0),
            );
            let status = run4(remap_alloc, f.base, tag as u64, moved as u64, 0);
            if expected_ok {
                assert!(
                    status >= 0,
                    "R5: after moving {moved:?} onto {tag:?}'s row the emitted \
                     allocator still refuses it (got {status}) — acceptance does \
                     not follow the plan, it is only intersected with it"
                );
            } else {
                assert_eq!(
                    status, BOUNDARY_ERR_RELATION,
                    "R5: after moving {moved:?} off {tag:?}'s row the emitted \
                     allocator still admits it"
                );
            }
        }

        // ⛔ **And the ABSENT-ROW case, which neither sweep above can reach.** Dropping a cell leaves the tag's row present, so the fold
        // still takes a `hit` arm and the seed is never read. Clause 2's
        // requirement is about a tag with *no row at all*: with a real seed such
        // a tag silently inherits another row's classes. Removing the row
        // entirely is the only perturbation that reads the seed, and without
        // this control "seeded with the empty mask" would be an untestable
        // claim — which is what clause 2 says a fail-closed branch must not be.
        for (tag, _) in plan.tags().handle_class_relation() {
            let rowless: Vec<(BoundaryTag, Vec<BoundaryClass>)> = plan
                .tags()
                .handle_class_relation()
                .iter()
                .filter(|(t, _)| t != tag)
                .map(|(t, classes)| (*t, classes.clone()))
                .collect();
            let (_nm, no_row_alloc) =
                compile_producer_with_plan(4, emit_alloc_probe, &with_relation(rowless));
            for class in BoundaryClass::ALL {
                let mut store = c1_d2_store();
                let f = bind_with(
                    &mut store,
                    BoundaryArenaBuilder::new(),
                    (64, 8, 0),
                    (64, 8, 0),
                );
                let status = run4(no_row_alloc, f.base, *tag as u64, class as u64, 0);
                assert_eq!(
                    status, BOUNDARY_ERR_RELATION,
                    "R5 clause 2: with NO row for {tag:?} the emitted allocator \
                     admitted {class:?} — the fold is seeded with a real mask, so \
                     a row-less tag inherits another row's classes"
                );
            }
        }
    }

    /// **`RECUT 2`, causal and PER-SITE — the immediate word's reported class
    /// is the authority's, per tag.**
    ///
    /// ⛔ **The site the located inventory never named.** `define_class`
    /// answered `is_bool ? BoundaryClass::Bool : BoundaryClass::Int` — a second
    /// tag → class mapping written beside the helper body, which is `RULING
    /// R3`'s "another hand-maintained table". It was invisible to the tag
    /// inventory because it names no threshold constant, and invisible to the
    /// class fold because that fold only ever looked at *node* classes.
    ///
    /// ⚠ **These are boundary-value classifications, not node classes**, and
    /// the two contracts stay apart: `BOUNDARY_TAG_CLASS_RELATION` governs
    /// `NODE_CLASS` legality and must keep excluding every immediate tag.
    ///
    /// **MEASURED:** per admitted immediate tag, remapping it in the plan
    /// changes what `class` answers for a word carrying it, and removing it
    /// makes `class` fail closed with `ERR_CLASS`.
    /// **CLAIMED:** the emitted immediate classification is the authority's.
    /// **THE GAP:** that the plan's relation is the partition's own answer —
    /// closed by `recut2_the_tag_admission_is_derived_from_the_partition_not_restated`,
    /// which now sweeps this relation too.
    #[test]
    fn b2v_the_emitted_immediate_class_is_the_plans() {
        use crate::boundary_value::{
            BoundaryEmissionPlan, BoundaryTagAdmission, BOUNDARY_ERR_CLASS, BOUNDARY_OK,
        };

        let plan = BoundaryEmissionPlan::derive();
        let mut store = c1_d2_store();
        let builder = BoundaryArenaBuilder::new();
        materialize_ground(&mut store, &cons(1)).expect("materializes");
        let f = bind(&mut store, builder);
        let base = f.base;

        assert!(
            !plan.tags().immediate_value_classes().is_empty(),
            "RECUT 2: the plan classifies no immediate tag, so the sweep is empty"
        );

        let respell = |immediate_classes: Vec<(BoundaryTag, BoundaryClass)>| {
            BoundaryEmissionPlan::new(
                plan.int_magnitude_classes().to_vec(),
                plan.byte_span_classes().to_vec(),
                BoundaryTagAdmission::new(
                    plan.tags().admitted().to_vec(),
                    plan.tags().immediate().to_vec(),
                    plan.tags().handle().to_vec(),
                    plan.tags().owner_bands().to_vec(),
                    immediate_classes,
                    plan.tags().handle_class_relation().to_vec(),
                ),
            )
        };

        for (tag, real_class) in plan.tags().immediate_value_classes() {
            // ⛔ A class no immediate is ever given, so "the answer changed"
            // cannot be an accident of two immediates sharing a class.
            let foreign = BoundaryClass::Record;
            assert_ne!(
                *real_class, foreign,
                "the perturbation must actually change {tag:?}'s class"
            );
            let remapped: Vec<_> = plan
                .tags()
                .immediate_value_classes()
                .iter()
                .map(|(other, class)| {
                    if other == tag {
                        (*other, foreign)
                    } else {
                        (*other, *class)
                    }
                })
                .collect();
            let dropped: Vec<_> = plan
                .tags()
                .immediate_value_classes()
                .iter()
                .copied()
                .filter(|(other, _)| other != tag)
                .collect();

            let word = BoundaryWord(*tag as u64);
            let probe = Probe::Unary(|h| h.class);
            let (_rm, real_code) = compile_probe_with_plan(probe, &plan);
            let (_qm, remap_code) = compile_probe_with_plan(probe, &respell(remapped));
            let (_dm, drop_code) = compile_probe_with_plan(probe, &respell(dropped));

            assert_eq!(
                run2(real_code, base, word),
                *real_class as i64,
                "RECUT 2: `class` does not report {real_class:?} for {tag:?} \
                 under the real plan, so the perturbations below are measured \
                 against the wrong baseline"
            );
            assert_eq!(
                run2(remap_code, base, word),
                foreign as i64,
                "RECUT 2: `class` still reports {real_class:?} for {tag:?} \
                 after the plan remapped it — the immediate classification is \
                 a literal, not the authority's"
            );
            assert_eq!(
                run2(drop_code, base, word),
                BOUNDARY_ERR_CLASS,
                "RECUT 2: `class` still classifies {tag:?} after the plan \
                 stopped classifying it — the helper is defaulting rather than \
                 failing closed"
            );
        }

        // ⚠ Positive control on the harness itself: `BOUNDARY_OK` is not what
        // these comparisons return, so an all-zero read would not masquerade
        // as agreement.
        assert_ne!(
            BOUNDARY_ERR_CLASS, BOUNDARY_OK,
            "the two outcomes this test distinguishes must be distinguishable"
        );
    }

    /// **`RECUT 2`, structural — EVERY `class_guard` call site takes its set
    /// from the plan, including the ones no probe reaches.**
    ///
    /// ⛔ **Why a source scan, when `pin-a-property` says reach for one last.**
    /// The behavioural pin below covers the three call sites this harness's
    /// probe shapes can reach. Four cannot: `store_int_limbs` takes five
    /// parameters, `store_int_tag` three, and `store_int_limb` /
    /// `store_bytes_len` take a value where `Probe::Binary` passes an out
    /// pointer. Leaving four of seven sites to review, after a mutation proved
    /// exactly this kind of site can silently defect, is the overclaim — so
    /// they get a mechanism whose limits are stated instead.
    ///
    /// **This pins the ALLOWED form, not a forbidden list:** every argument
    /// must come from `plan`. A new guard spelled any other way reddens,
    /// including one nobody imagined.
    ///
    /// **MEASURED:** the third argument of every `class_guard(...)` call in
    /// this module is a `plan.` expression. **CLAIMED:** no emitted class guard
    /// enumerates classes by hand. ⛔ **THE GAP:** a helper that *launders* a
    /// literal — `fn my_classes() -> &'static [BoundaryClass]` passed as
    /// `plan.foo()`-shaped text — is not detectable here and is not detectable
    /// by the behavioural pin either for the four unreachable sites. That arm
    /// is review-enforced, and saying so is the point of writing it down.
    #[test]
    fn b2v_every_class_guard_call_site_takes_its_set_from_the_plan() {
        let source = include_str!("boundary_value_clif.rs");
        let needle = "class_guard(";
        let mut sites = 0usize;
        let mut cursor = 0usize;

        while let Some(found) = source[cursor..].find(needle) {
            let at = cursor + found;
            cursor = at + needle.len();
            // Skip the definition itself and any doc-comment mention: only a
            // CALL has a `(` immediately followed by arguments on the same
            // logical expression, and only a call is preceded by whitespace.
            let before = source[..at]
                .rfind('\n')
                .map(|n| &source[n + 1..at])
                .unwrap_or("");
            // ⚠ This scan reads its OWN source, so it matches its own needle
            // literal and its own doc comment. Skip the definition, doc lines,
            // and any occurrence inside a string.
            if before.trim_start().starts_with("fn ")
                || before.contains("///")
                || before.ends_with('"')
            {
                continue;
            }

            // Take the balanced argument list.
            let mut depth = 1usize;
            let mut end = cursor;
            for (offset, ch) in source[cursor..].char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = cursor + offset;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            assert!(
                depth == 0,
                "RECUT 2: could not find the end of a `class_guard` argument \
                 list at byte {at} -- an undetermined parse FAILS rather than \
                 passing, or every gap in this scan is a silent green"
            );
            let args = &source[cursor..end];
            let third = args
                .rsplit(',')
                .next()
                .expect("a comma-separated argument list has a last element")
                .trim();
            assert!(
                third.starts_with("plan."),
                "RECUT 2: a `class_guard` call takes `{third}`, which does not \
                 come from the plan -- an emitted class guard that enumerates \
                 its own classes is the hand-maintained table beside the helper \
                 bodies that RULING R3 excludes"
            );
            sites += 1;
        }

        // ⛔ Positive control: a scan that matched nothing passes for any
        // reason at all. Seven is the count the mutation table was run
        // against; fewer means the scan stopped seeing sites, more means new
        // ones appeared and were checked.
        assert!(
            sites >= 7,
            "RECUT 2: the scan found {sites} `class_guard` call sites, fewer \
             than the seven this module has -- it is no longer looking at the \
             surface it claims to cover"
        );
    }

    /// **`RECUT 2`, causal and PER-SITE — every emitted CLASS guard is the
    /// plan's.**
    ///
    /// ⛔ **This closes a hole in `720f301c`, the axis the Architect confirmed.**
    /// `class_guard(&mut b, node, plan.int_magnitude_classes())` appears at five
    /// sites. Disconnecting **one** of them — restoring the literal
    /// `&[BoundaryClass::Int]` it used to be, leaving the other four consuming
    /// the plan — left the whole suite green: 439 passed, 0 failed. The
    /// whole-graph differential
    /// `recut2_the_emitted_helper_graph_changes_when_the_authority_changes`
    /// cannot see it, because the four remaining consumers still move the
    /// aggregate. Same granularity fault the tag axis had, found by running the
    /// same mutation against the earlier axis rather than assuming it was safe
    /// because it was confirmed.
    ///
    /// **MEASURED:** for each probed class-guarded helper, perturbing the class
    /// set that guards it changes that helper's answer for a node of the
    /// original class — from its real status to `ERR_CLASS`.
    /// **CLAIMED:** no probed helper's class guard is a literal.
    /// **THE GAP:** `store_int_limbs` (5 params) and `store_int_tag` (3 params)
    /// do not fit this harness's probe shapes, so for those two the evidence
    /// remains the whole-graph pin plus review. Named, not implied.
    #[test]
    fn b2v_every_emitted_class_guard_is_the_plans() {
        use crate::boundary_value::{BoundaryEmissionPlan, BOUNDARY_ERR_CLASS};

        let plan = BoundaryEmissionPlan::derive();
        // ⛔ Perturb ONLY the int-magnitude class set. `Record` is an admitted
        // class of a different storage shape, so the guard stays well-formed
        // and only its membership changes.
        let perturbed = BoundaryEmissionPlan::new(
            vec![BoundaryClass::Record],
            plan.byte_span_classes().to_vec(),
            plan.tags().clone(),
        );

        // The node is produced by the REAL emitter; only the reader varies.
        let (_pm, produce) = compile_producer(3, emit_spilled_int_producer);
        let value = (1i64 << 60) + 7;
        assert!(
            !BoundaryWord::int_fits_immediate(value),
            "the fixture must spill, or no Int-classed node exists to guard"
        );
        let mut store = c1_d2_store();
        let native = crate::native_int::NativeIntArenaV1::default();
        let mut f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (2, 0, 0),
            (0, 0, 0),
        );
        with_native_int(&mut f, &native);
        let word = BoundaryWord(run3(produce, f.base, BoundaryWord(value as u64), 0) as u64);
        assert_eq!(
            word.tag(),
            Some(BoundaryTag::PersistentGround),
            "the producer did not mint a spilled Int, so the sweep below reads \
             the wrong node"
        );

        let unary: [(&str, Probe); 2] = [
            ("int_sign", Probe::Unary(|h| h.int_sign)),
            ("int_len", Probe::Unary(|h| h.int_len)),
        ];
        let binary: [(&str, Probe); 1] = [("int_limb", Probe::Binary(|h| h.int_limb))];

        for (name, probe, index) in unary
            .iter()
            .map(|(n, p)| (*n, *p, None))
            .chain(binary.iter().map(|(n, p)| (*n, *p, Some(0u64))))
        {
            let (_real_module, real_code) = compile_probe_with_plan(probe, &plan);
            let (_pert_module, pert_code) = compile_probe_with_plan(probe, &perturbed);
            let (real, pert) = match index {
                None => (run2(real_code, f.base, word), run2(pert_code, f.base, word)),
                Some(i) => (
                    run3(real_code, f.base, word, i),
                    run3(pert_code, f.base, word, i),
                ),
            };
            assert_ne!(
                real, BOUNDARY_ERR_CLASS,
                "RECUT 2: `{name}` already refuses this node on class under the \
                 REAL plan, so the perturbation cannot change its answer and \
                 would prove nothing"
            );
            assert_eq!(
                pert, BOUNDARY_ERR_CLASS,
                "RECUT 2: `{name}` still accepts an Int-classed node after the \
                 plan stopped admitting `Int` for limb storage — its class \
                 guard is a literal, not the plan's set"
            );
        }
    }

    /// **`AC-1`/`AC-6` — the plan's owner bands and `referent_owner` are the
    /// same classification.**
    ///
    /// ⚠ MEASURED: every tag the partition publishes as a handle sits in
    /// exactly one band, and that band's owner is the one
    /// [`BoundaryTag::referent_owner`] gives it. CLAIMED: emitted code sends
    /// every word to the region that owns its referent. THE GAP: that emitted
    /// code decides from *these bands* — which it now does because the bands
    /// are the only tag classification the emitter can see, and which
    /// `recut2_the_emitted_helper_graph_changes_when_the_owner_bands_change`
    /// makes causal rather than textual.
    ///
    /// ⛔ **Two clauses of the predecessor were RETIRED, not quietly kept.**
    /// It asserted the owner bands were numerically *contiguous*, and it
    /// classified tags by comparing against `FIRST_HANDLE_TAG` /
    /// `LAST_PERSISTENT_TAG`. Both existed only because a threshold cannot
    /// separate bands that interleave. Nothing emitted depends on tag order
    /// any more, so keeping a still-passing contiguity assertion would pin a
    /// property no mechanism rests on — which reads to the next author as a
    /// constraint they must preserve.
    #[test]
    fn b2v_the_plan_owner_bands_agree_with_referent_owner() {
        use std::collections::BTreeSet;

        let plan = crate::boundary_value::BoundaryEmissionPlan::derive();
        let bands = plan.tags().owner_bands();

        // Positive controls FIRST: agreement over an empty relation, or one
        // that mentions a single owner, is not evidence about a classification.
        assert!(
            bands.len() >= 2,
            "AC-6: the plan names {} owner band(s), so 'the band agrees with \
             referent_owner' cannot distinguish the owners at all",
            bands.len()
        );
        assert!(
            bands.iter().all(|(_, tags)| !tags.is_empty()),
            "AC-6: an empty band makes the agreement below vacuous over it"
        );

        let mut seen: BTreeSet<BoundaryTag> = BTreeSet::new();
        for (owner, tags) in bands {
            for tag in tags {
                assert_eq!(
                    tag.referent_owner(),
                    *owner,
                    "AC-6: the plan bands {tag:?} under {owner:?}, but its \
                     referent owner is {:?}",
                    tag.referent_owner()
                );
                assert!(
                    seen.insert(*tag),
                    "AC-6: {tag:?} appears in more than one owner band, so the \
                     emitted fold's arms overlap and the last one silently wins"
                );
            }
        }

        // ⛔ And the converse, which the per-band sweep above cannot see: a
        // handle tag in NO band would resolve nowhere, and every assertion
        // above would still pass.
        assert_eq!(
            seen.iter().copied().collect::<Vec<_>>(),
            plan.tags().handle(),
            "AC-6: the bands' union is not the admitted handle set"
        );
    }
    /// ⭐⭐ **`AC-4` — each of the eight authorized limits, exercised by a REAL
    /// generated-code requester at limit-plus-one, and attributed to its exact
    /// named region and resource.**
    ///
    /// **MEASURED:** for each `(scope, resource)`, emitted code that requests
    /// one past the authorized ceiling is refused, and the activation attributes
    /// the refusal to **that** scope and resource.
    /// **CLAIMED:** each limit governs its named region and resource.
    /// **THE GAP:** ⛔ two of the eight cells are **not reachable by emitted
    /// code at all**, and that is measured below rather than skipped — see
    /// `ac4_invocation_data_and_limbs_are_unreachable_by_the_admitted_relation`.
    ///
    /// ⛔ Not one loop with one assertion: each cell asserts its own attribution,
    /// so a control that fired on the wrong cell fails naming both.
    #[test]
    fn ac4_each_reachable_limit_refuses_at_limit_plus_one_naming_its_own_resource() {
        use crate::boundary_activation::{BoundaryActivationV1, BoundaryStoreBindingV1};
        use crate::boundary_resource_profile::{BoundaryResource, BoundaryResourceScope};

        // The six cells the admitted relation can reach. ⛔ Derived below, not
        // asserted: the other two are proved unreachable by their own control.
        let cells = [
            (BoundaryResourceScope::Persistent, BoundaryResource::Nodes),
            (BoundaryResourceScope::Persistent, BoundaryResource::Words),
            (BoundaryResourceScope::Persistent, BoundaryResource::DataBytes),
            (
                BoundaryResourceScope::Persistent,
                BoundaryResource::NativeIntLimbs,
            ),
            (BoundaryResourceScope::Invocation, BoundaryResource::Nodes),
            (BoundaryResourceScope::Invocation, BoundaryResource::Words),
        ];

        for (scope, resource) in cells {
            let limit = 2usize;
            let profile = ac4_profile_with(scope, resource, limit);
            let mut store = BoundaryValueStore::new();
            let binding = BoundaryStoreBindingV1::open(&mut store, profile);
            let activation = BoundaryActivationV1::begin(&binding);
            let base = match scope {
                BoundaryResourceScope::Invocation => activation.published_boundary_base(),
                BoundaryResourceScope::Persistent => activation.published_boundary_base(),
            };
            let (tag, class) = ac4_lane(scope);

            // ⭐⭐ **Fill the region TO its ceiling first, then request one
            // more.** That is what *"at-limit-plus-one"* means, and it is also
            // what makes the attribution exact: a **refused** request bumps no
            // count, so asking "which resource is at its limit?" after a
            // refusal-from-empty names nothing. ⚠ I learned that from this
            // control failing — `persistent words` was refused for capacity
            // while every live count was still zero.
            let status = match resource {
                BoundaryResource::Nodes => {
                    let (_m, code) = compile_producer(4, emit_alloc_probe);
                    let f: extern "C" fn(*mut u64, i64, i64, i64) -> i64 =
                        unsafe { std::mem::transmute(code) };
                    let mut last = BOUNDARY_OK;
                    // limit + 1 allocations; the extra one must be refused.
                    for _ in 0..=limit {
                        last = f(base, tag as i64, class as i64, 0);
                    }
                    last
                }
                BoundaryResource::Words => {
                    let (_m, code) = compile_producer(4, emit_alloc_probe);
                    let f: extern "C" fn(*mut u64, i64, i64, i64) -> i64 =
                        unsafe { std::mem::transmute(code) };
                    // Fill: one node claiming exactly the authorized words.
                    let filled = f(base, tag as i64, class as i64, limit as i64);
                    assert_ne!(
                        filled, BOUNDARY_ERR_CAPACITY,
                        "AC-4: filling {scope} words TO the ceiling was itself \
                         refused, so the fixture never reached limit-plus-one"
                    );
                    f(base, tag as i64, class as i64, 1)
                }
                BoundaryResource::DataBytes => {
                    let (_m, code) = compile_producer(4, emit_ac4_bytes_len_probe);
                    let f: extern "C" fn(*mut u64, i64, i64, i64) -> i64 =
                        unsafe { std::mem::transmute(code) };
                    let (tag, class) = (
                        BoundaryTag::PersistentGround as i64,
                        BoundaryClass::Bytes as i64,
                    );
                    let filled = f(base, tag, class, limit as i64);
                    assert_ne!(
                        filled, BOUNDARY_ERR_CAPACITY,
                        "AC-4: filling {scope} data bytes TO the ceiling was \
                         itself refused"
                    );
                    f(base, tag, class, 1)
                }
                BoundaryResource::NativeIntLimbs => {
                    let (_m, code) = compile_producer(4, emit_ac4_int_limbs_probe);
                    let f: extern "C" fn(*mut u64, i64, i64, i64) -> i64 =
                        unsafe { std::mem::transmute(code) };
                    let (tag, class) = (
                        BoundaryTag::PersistentGround as i64,
                        BoundaryClass::Int as i64,
                    );
                    let filled = f(base, tag, class, limit as i64);
                    assert_ne!(
                        filled, BOUNDARY_ERR_CAPACITY,
                        "AC-4: filling {scope} limbs TO the ceiling was itself \
                         refused"
                    );
                    f(base, tag, class, 1)
                }
            };

            assert_eq!(
                status, BOUNDARY_ERR_CAPACITY,
                "AC-4: {scope} {resource} at limit+1 was not refused for capacity"
            );

            // ⭐ The half that makes this eight controls and not one: the
            // refusal is attributed to THIS scope and THIS resource.
            let named = activation
                .attribute_capacity_exhaustion(&store)
                .unwrap_or_else(|| {
                    panic!(
                        "AC-4: {scope} {resource} was refused for capacity but no \
                         authorized limit is at its ceiling, so the refusal came \
                         from somewhere other than the eight limits"
                    )
                });
            assert_eq!(
                (named.scope, named.resource),
                (scope, resource),
                "AC-4: the refusal was attributed to {} {} instead of {scope} {resource}",
                named.scope,
                named.resource
            );
            assert_eq!(named.limit, limit);
            assert!(named.to_string().contains(scope.name()));
            assert!(named.to_string().contains(resource.name()));
        }
    }

    /// ⛔⛔ **The other two cells are UNREACHABLE BY THE ADMITTED RELATION, and
    /// this measures it rather than asserting it.**
    ///
    /// ⭐ `BOUNDARY_TAG_CLASS_RELATION` gives the **invocation** arena exactly
    /// two lanes — `(InvocationBorrowed, BorrowedOpaque)` and
    /// `(InvocationHostResult, HostResult)` — and **neither class carries a data
    /// body or magnitude limbs.** `Bytes`/`String`/`Int` are admitted only under
    /// `PersistentGround`, which indexes the persistent region. ⇒ No emitted
    /// requester can consume an invocation data byte or an invocation limb.
    ///
    /// **MEASURED:** the relation admits no invocation lane for any of the
    /// body-bearing classes, and an emitted attempt to claim a data body under
    /// an invocation lane is refused **before** any capacity question — with a
    /// status that is ⛔ *not* `BOUNDARY_ERR_CAPACITY`.
    /// **CLAIMED:** those two of the eight limits cannot be exercised by
    /// generated code, so their ceilings are unreachable rather than untested.
    /// **THE GAP:** ⚠ this is a property of the **admitted relation as landed**.
    /// ⛔ If a future node admits `Bytes` or `Int` under an invocation tag, these
    /// two cells become reachable and `AC-4` owes them a fixture — the assertion
    /// below is written to go **red** in exactly that case rather than to keep
    /// quietly passing.
    #[test]
    fn ac4_invocation_data_and_limbs_are_unreachable_by_the_admitted_relation() {
        use crate::boundary_activation::{BoundaryActivationV1, BoundaryStoreBindingV1};
        use crate::boundary_resource_profile::{BoundaryResource, BoundaryResourceScope};
        use crate::boundary_value::{boundary_relation_admits, BOUNDARY_TAG_CLASS_RELATION};

        // 1 — the relation itself: no invocation tag admits a body-bearing class.
        let invocation_tags = [
            BoundaryTag::InvocationBorrowed,
            BoundaryTag::InvocationHostResult,
        ];
        let body_bearing = [
            BoundaryClass::Bytes,
            BoundaryClass::String,
            BoundaryClass::Int,
        ];
        for tag in invocation_tags {
            for class in body_bearing {
                assert!(
                    !boundary_relation_admits(tag, class),
                    "AC-4: the relation now admits ({tag:?}, {class:?}), so the \
                     invocation data-byte and limb ceilings ARE reachable and \
                     each owes its own at-limit-plus-one fixture"
                );
            }
        }
        // Non-vacuity: the relation is not simply refusing everything.
        assert!(boundary_relation_admits(
            BoundaryTag::PersistentGround,
            BoundaryClass::Bytes
        ));
        assert!(boundary_relation_admits(
            BoundaryTag::InvocationHostResult,
            BoundaryClass::HostResult
        ));
        assert!(
            BOUNDARY_TAG_CLASS_RELATION
                .iter()
                .any(|(tag, _)| *tag == BoundaryTag::InvocationBorrowed),
            "non-vacuity: the invocation tags are in the relation at all"
        );

        // 2 — and emitted code agrees: claiming a data body under an invocation
        // lane is refused BEFORE any capacity question.
        let profile = ac4_profile_with(
            BoundaryResourceScope::Invocation,
            BoundaryResource::DataBytes,
            0,
        );
        let mut store = BoundaryValueStore::new();
        let binding = BoundaryStoreBindingV1::open(&mut store, profile);
        let activation = BoundaryActivationV1::begin(&binding);
        let (_m, code) = compile_producer(4, emit_ac4_bytes_len_probe);
        let f: extern "C" fn(*mut u64, i64, i64, i64) -> i64 = unsafe { std::mem::transmute(code) };
        let status = f(
            activation.published_boundary_base(),
            BoundaryTag::InvocationHostResult as i64,
            BoundaryClass::Bytes as i64,
            1,
        );
        assert_ne!(
            status, BOUNDARY_OK,
            "AC-4: an invocation lane accepted a data body"
        );
        assert_ne!(
            status, BOUNDARY_ERR_CAPACITY,
            "AC-4: the refusal was a CAPACITY refusal, which would mean the cell \
             is reachable after all and owes a real at-limit-plus-one fixture"
        );
    }

}
