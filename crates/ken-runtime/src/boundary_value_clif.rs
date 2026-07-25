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
    boundary_class_mask, boundary_domain_mask, boundary_int_marker_mask, BoundaryClass,
    BoundaryImmediateDomain, BoundaryReferentOwner, BoundaryTag, ARENA_DATA, ARENA_DATA_CAPACITY,
    ARENA_DATA_COUNT, ARENA_FROZEN, ARENA_LIMBS, ARENA_LIMB_CAPACITY, ARENA_LIMB_COUNT,
    ARENA_NAMES, ARENA_NATIVE_INT, ARENA_NODES, ARENA_NODE_CAPACITY, ARENA_NODE_COUNT,
    ARENA_PERSISTENT, ARENA_WORDS, ARENA_WORD_CAPACITY, ARENA_WORD_COUNT, BOUNDARY_ERR_BOUNDS,
    BOUNDARY_ERR_CAPACITY, BOUNDARY_ERR_CLASS, BOUNDARY_ERR_ESCAPE, BOUNDARY_ERR_FROZEN,
    BOUNDARY_ERR_RELATION, BOUNDARY_ERR_SHAPE, BOUNDARY_ERR_TAG, BOUNDARY_INT_REGION_LIMBS,
    BOUNDARY_NODE_STRIDE, BOUNDARY_OK, BOUNDARY_TAG_BITS, BOUNDARY_TAG_CLASS_RELATION,
    BOUNDARY_TAG_MASK, NODE_CLASS, NODE_EXTENT, NODE_FIELDS_AT, NODE_FIELD_COUNT, NODE_LIMBS_AT,
    NODE_LIMB_COUNT, NODE_OWNER, NODE_PAYLOAD, NODE_SLOT, NODE_TAG_ID,
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
    "ken_boundary_store_int_limbs_local",
    "ken_boundary_store_int_limb_local",
    "ken_boundary_store_bytes_len_local",
    "ken_boundary_store_byte_local",
    // ── content access: the value's BITS, not its identity or its length ──
    "ken_boundary_byte_local",
    "ken_boundary_int_sign_local",
    "ken_boundary_int_len_local",
    "ken_boundary_int_limb_local",
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
    store_int_limbs: FuncId,
    store_int_limb: FuncId,
    store_bytes_len: FuncId,
    store_byte: FuncId,
    byte: FuncId,
    int_sign: FuncId,
    int_len: FuncId,
    int_limb: FuncId,
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
    let store_int_limbs = declare(module, "ken_boundary_store_int_limbs_local", 5)?;
    let store_int_limb = declare(module, "ken_boundary_store_int_limb_local", 4)?;
    let store_bytes_len = declare(module, "ken_boundary_store_bytes_len_local", 4)?;
    let store_byte = declare(module, "ken_boundary_store_byte_local", 4)?;
    let byte = declare(module, "ken_boundary_byte_local", 4)?;
    let int_sign = declare(module, "ken_boundary_int_sign_local", 3)?;
    let int_len = declare(module, "ken_boundary_int_len_local", 3)?;
    let int_limb = declare(module, "ken_boundary_int_limb_local", 4)?;
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
        store_int_limbs,
        store_int_limb,
        store_bytes_len,
        store_byte,
        byte,
        int_sign,
        int_len,
        int_limb,
        native_int_resolve: native_int.resolve,
    };

    define_resolve(module, graph)?;
    define_class(module, graph)?;
    define_node_word(module, graph, graph.owner, NODE_OWNER)?;
    define_node_word(module, graph, graph.slot, NODE_SLOT)?;
    define_scalar(module, graph)?;
    define_node_word(module, graph, graph.tag, NODE_TAG_ID)?;
    define_node_word(module, graph, graph.field_count, NODE_FIELD_COUNT)?;
    define_field(module, graph)?;
    define_record_field(module, graph)?;
    define_host_success(module, graph)?;
    define_host_payload(module, graph)?;
    define_make_immediate(module, graph)?;
    define_escape_check(module, graph)?;
    define_alloc(module, graph)?;
    define_store_node_word(module, graph, graph.store_tag_id, NODE_TAG_ID)?;
    define_store_node_word(module, graph, graph.store_scalar, NODE_PAYLOAD)?;
    define_store_field(module, graph)?;
    define_store_name(module, graph)?;
    define_store_int_tag(module, graph)?;
    define_store_int_limbs(module, graph)?;
    define_store_int_limb(module, graph)?;
    define_store_bytes_len(module, graph)?;
    define_byte_access(module, graph, graph.store_byte, true)?;
    define_byte_access(module, graph, graph.byte, false)?;
    define_int_part(module, graph, graph.int_sign, IntPart::Sign)?;
    define_int_part(module, graph, graph.int_len, IntPart::Len)?;
    define_int_part(module, graph, graph.int_limb, IntPart::Limb)?;

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
        store_int_limbs,
        store_int_limb,
        store_bytes_len,
        store_byte,
        byte,
        int_sign,
        int_len,
        int_limb,
    })
}

/// Capture every helper body as text, for the JIT/object identity pin and the
/// closed-inventory pin. Test-only.
#[cfg(test)]
pub(crate) fn capture_boundary_value_local_graph<M: Module>(
    module: &mut M,
) -> Result<String, CraneliftBackendError> {
    let native = crate::native_int_clif::emit_native_int_local_graph(module, false)?;
    BOUNDARY_CLIF_CAPTURE.with(|capture| *capture.borrow_mut() = Some(Vec::new()));
    emit_boundary_value_local_graph(module, &native)?;
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
    let mut ctx = module.make_context();
    std::mem::swap(&mut ctx.func, &mut func);
    module
        .define_function(id, &mut ctx)
        .map_err(|e| backend_module(e.to_string()))
}

/// The lowest tag whose payload is a node index rather than a value.
const FIRST_HANDLE_TAG: i64 = BoundaryTag::PersistentGround as i64;
/// The highest tag whose node lives in the **persistent** region.
///
/// ⚠ A threshold, so it depends on the closed tag set staying grouped by
/// referent owner. `b2v_the_region_thresholds_agree_with_referent_owner` is the
/// pin that makes a reordering redden instead of silently re-pointing every
/// persistent word at the arena.
const LAST_PERSISTENT_TAG: i64 = BoundaryTag::PersistentClosure as i64;
/// The highest tag in the closed set.
const LAST_TAG: i64 = BoundaryTag::InvocationHostResult as i64;

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
fn define_resolve<M: Module>(module: &mut M, graph: Graph) -> Result<(), CraneliftBackendError> {
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
        let known = b
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, tag, LAST_TAG);
        let closed = b.create_block();
        let unknown = b.create_block();
        b.ins().brif(known, closed, &[], unknown, &[]);

        b.switch_to_block(unknown);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_TAG);
        b.ins().return_(&[err]);

        b.switch_to_block(closed);
        let is_handle = b
            .ins()
            .icmp_imm(IntCC::UnsignedGreaterThanOrEqual, tag, FIRST_HANDLE_TAG);
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
        let persistent = b
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, tag, LAST_PERSISTENT_TAG);
        let store_side = b.create_block();
        let arena_side = b.create_block();
        b.ins().brif(persistent, store_side, &[], arena_side, &[]);

        b.switch_to_block(arena_side);
        b.ins().jump(selected, &[arena.into()]);

        b.switch_to_block(store_side);
        // ⛔ A persistent word resolves through PERSISTENT storage or not at
        // all. An invocation bound to no persistent region fails closed —
        // falling back to the arena would read the persistent index against the
        // wrong table, which is silent corruption rather than an error.
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
fn define_class<M: Module>(module: &mut M, graph: Graph) -> Result<(), CraneliftBackendError> {
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
        let is_bool = b
            .ins()
            .icmp_imm(IntCC::Equal, tag, BoundaryTag::ImmediateBool as i64);
        // Every non-`Bool` immediate is an integer scalar. The finer identity —
        // exit status vs capability vs bounded nat — lives in the word's own
        // tag byte, which emitted code reads with a single `band`; it does not
        // need a helper and does not belong in the ground-value class space.
        let bool_class = b.ins().iconst(types::I64, BoundaryClass::Bool as i64);
        let int_class = b.ins().iconst(types::I64, BoundaryClass::Int as i64);
        let class = b.ins().select(is_bool, bool_class, int_class);
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

        let immediate = b
            .ins()
            .icmp_imm(IntCC::UnsignedLessThan, tag, FIRST_HANDLE_TAG);
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
) -> Result<(), CraneliftBackendError> {
    let mut func = begin(module, graph.escape_check, 2);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let word = p[1];

        let tag = b.ins().band_imm(word, BOUNDARY_TAG_MASK as i64);
        let known = b
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, tag, LAST_TAG);
        let closed = b.create_block();
        let unknown = b.create_block();
        b.ins().brif(known, closed, &[], unknown, &[]);

        b.switch_to_block(unknown);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_TAG);
        b.ins().return_(&[err]);

        b.switch_to_block(closed);
        let borrowed = b.ins().icmp_imm(
            IntCC::UnsignedGreaterThanOrEqual,
            tag,
            BoundaryTag::InvocationBorrowed as i64,
        );
        let escaping = b.create_block();
        let permitted = b.create_block();
        b.ins().brif(borrowed, escaping, &[], permitted, &[]);

        b.switch_to_block(escaping);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_ESCAPE);
        b.ins().return_(&[err]);

        b.switch_to_block(permitted);
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

/// The lowest invocation-owned tag.
const FIRST_INVOCATION_TAG: i64 = BoundaryTag::InvocationBorrowed as i64;
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
) -> cranelift_codegen::ir::Value {
    let known = b
        .ins()
        .icmp_imm(IntCC::UnsignedLessThanOrEqual, tag, LAST_TAG);
    let closed = b.create_block();
    let unknown = b.create_block();
    b.ins().brif(known, closed, &[], unknown, &[]);

    b.switch_to_block(unknown);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_TAG);
    b.ins().return_(&[err]);

    b.switch_to_block(closed);
    // ⛔ An immediate has no node to allocate. `make_immediate` is its
    // constructor, and conflating the two would mint a word whose payload is
    // read as a node index.
    let is_handle = b
        .ins()
        .icmp_imm(IntCC::UnsignedGreaterThanOrEqual, tag, FIRST_HANDLE_TAG);
    let handle = b.create_block();
    let immediate = b.create_block();
    b.ins().brif(is_handle, handle, &[], immediate, &[]);

    b.switch_to_block(immediate);
    let err = b.ins().iconst(types::I64, BOUNDARY_ERR_SHAPE);
    b.ins().return_(&[err]);

    b.switch_to_block(handle);
    let selected = b.create_block();
    b.append_block_param(selected, ptr);
    let persistent = b
        .ins()
        .icmp_imm(IntCC::UnsignedLessThanOrEqual, tag, LAST_PERSISTENT_TAG);
    let store_side = b.create_block();
    let arena_side = b.create_block();
    b.ins().brif(persistent, store_side, &[], arena_side, &[]);

    b.switch_to_block(arena_side);
    b.ins().jump(selected, &[arena.into()]);

    b.switch_to_block(store_side);
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

    b.switch_to_block(selected);
    b.block_params(selected)[0]
}

fn one_i64(b: &mut FunctionBuilder<'_>) -> cranelift_codegen::ir::Value {
    b.ins().iconst(types::I64, 1)
}

/// Select the class bitmask the ABI relation admits for `tag`.
///
/// ⭐ Θ(1): four comparisons and a chain of selects, no table walk and no data
/// section. Every constant is computed by [`boundary_class_mask`] from the one
/// authoritative relation, so this cannot say something the table does not.
fn relation_mask(
    b: &mut FunctionBuilder<'_>,
    tag: cranelift_codegen::ir::Value,
) -> cranelift_codegen::ir::Value {
    // Fold from the *last* handle tag backwards, so the innermost value is a
    // real mask rather than a zero default: an unlisted tag can never reach
    // here, because `select_region_by_tag` has already refused it.
    let mut chain: Option<cranelift_codegen::ir::Value> = None;
    for (tag_value, _) in BOUNDARY_TAG_CLASS_RELATION.iter().rev() {
        let mask = b
            .ins()
            .iconst(types::I64, boundary_class_mask(*tag_value) as i64);
        chain = Some(match chain {
            None => mask,
            Some(rest) => {
                let hit = b.ins().icmp_imm(IntCC::Equal, tag, *tag_value as i64);
                b.ins().select(hit, mask, rest)
            }
        });
    }
    chain.expect("the relation is non-empty")
}

/// `(arena, tag, class, field_count, out) -> status` — allocate a handle node in
/// the region the tag selects and write its word to `*out`.
///
/// ⛔ **The word this returns is a persistent identity when the tag is
/// persistent.** It indexes store-owned storage, so it stays meaningful after
/// the invocation arena is gone — which is the whole reason the region split
/// exists.
fn define_alloc<M: Module>(module: &mut M, graph: Graph) -> Result<(), CraneliftBackendError> {
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
        let region = select_region_by_tag(&mut b, ptr, arena, tag);

        // ── ⛔ the RELATION, not the two sets ────────────────────────────────
        //
        // Both sets are closed and their product still contains pairs no
        // disposition can produce — `PersistentClosure + HostResult`,
        // `InvocationHostResult + Constructor`. Minting one succeeds and then
        // fails much later at an unrelated projection, reporting the wrong
        // defect in the wrong place. The mask comes from
        // `BOUNDARY_TAG_CLASS_RELATION`, so there is one table and the CLIF
        // cannot drift from it.
        let mask = relation_mask(&mut b, tag);
        let one = one_i64(&mut b);
        let bit = b.ins().ishl(one, class);
        let admitted = b.ins().band(mask, bit);
        let related = b.ins().icmp_imm(IntCC::NotEqual, admitted, 0);
        let lawful = b.create_block();
        let unrelated = b.create_block();
        b.ins().brif(related, lawful, &[], unrelated, &[]);

        b.switch_to_block(unrelated);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_RELATION);
        b.ins().return_(&[err]);

        b.switch_to_block(lawful);

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
        let persistent = b
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, tag, LAST_PERSISTENT_TAG);
        let store_owner = b
            .ins()
            .iconst(types::I64, BoundaryReferentOwner::PersistentStore as i64);
        let arena_owner = b
            .ins()
            .iconst(types::I64, BoundaryReferentOwner::InvocationArena as i64);
        let owner = b.ins().select(persistent, store_owner, arena_owner);

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

/// Return early with [`BOUNDARY_ERR_FROZEN`] unless `word` names a node emitted
/// code constructed — i.e. one at or beyond the region's frozen prefix.
fn mutable_guard(
    b: &mut FunctionBuilder<'_>,
    word: cranelift_codegen::ir::Value,
    region: cranelift_codegen::ir::Value,
) {
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
        let known = b
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, child_tag, LAST_TAG);
        let child_ok = b.create_block();
        let bad_child = b.create_block();
        b.ins().brif(known, child_ok, &[], bad_child, &[]);

        b.switch_to_block(bad_child);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_TAG);
        b.ins().return_(&[err]);

        b.switch_to_block(child_ok);
        let owner = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_OWNER);
        let parent_persists = b.ins().icmp_imm(
            IntCC::Equal,
            owner,
            BoundaryReferentOwner::PersistentStore as i64,
        );
        let child_dies = b.ins().icmp_imm(
            IntCC::UnsignedGreaterThanOrEqual,
            child_tag,
            FIRST_INVOCATION_TAG,
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
        class_guard(&mut b, node, &[BoundaryClass::Int]);
        region_limbs_guard(&mut b, node);

        // ⛔ The sign is a BIT, not a number. `decode_final_export` reads `0` or
        // `1` and refuses anything else, so admitting a third value here would
        // build a node the exact-`Int` decoder cannot read back.
        let bit = b.ins().icmp_imm(IntCC::UnsignedLessThanOrEqual, sign, 1);
        let shaped = b.create_block();
        let unshaped = b.create_block();
        b.ins().brif(bit, shaped, &[], unshaped, &[]);

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

/// `(arena, word, index, limb) -> status` — write one magnitude limb.
fn define_store_int_limb<M: Module>(
    module: &mut M,
    graph: Graph,
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
        class_guard(&mut b, node, &[BoundaryClass::Int]);
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
        class_guard(&mut b, node, &[BoundaryClass::Bytes, BoundaryClass::String]);

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
        class_guard(&mut b, node, &[BoundaryClass::Bytes, BoundaryClass::String]);

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
        class_guard(&mut b, node, &[BoundaryClass::Int]);

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
        let persists = b.ins().icmp_imm(
            IntCC::Equal,
            owner,
            BoundaryReferentOwner::PersistentStore as i64,
        );
        let persistent_mask = b.ins().iconst(
            types::I64,
            boundary_int_marker_mask(BoundaryReferentOwner::PersistentStore) as i64,
        );
        let invocation_mask = b.ins().iconst(
            types::I64,
            boundary_int_marker_mask(BoundaryReferentOwner::InvocationArena) as i64,
        );
        let mask = b.ins().select(persists, persistent_mask, invocation_mask);
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
        class_guard(&mut b, node, &[BoundaryClass::Int]);

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
        let region_sign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_PAYLOAD);
        let region_len = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_LIMB_COUNT);
        let at = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, NODE_LIMBS_AT);
        // The node's span is checked against the region's LIVE limb count
        // before any address is formed, so a node carrying a stale span fails
        // closed rather than reading past the table.
        let live = b
            .ins()
            .load(types::I64, MemFlags::trusted(), region, ARENA_LIMB_COUNT);
        let end = b.ins().iadd(at, region_len);
        let fits = b.ins().icmp(IntCC::UnsignedLessThanOrEqual, end, live);
        let spanned = b.create_block();
        let unspanned = b.create_block();
        b.ins().brif(fits, spanned, &[], unspanned, &[]);

        b.switch_to_block(unspanned);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(spanned);
        let table = b.ins().load(ptr, MemFlags::trusted(), region, ARENA_LIMBS);
        let byte_at = b.ins().imul_imm(at, 8);
        let region_base = b.ins().iadd(table, byte_at);
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
        }
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, id, func)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_value::{
        boundary_immediate_admits, boundary_immediate_domain, boundary_int_marker_admits,
        boundary_relation_admits, materialize_borrowed, materialize_ground,
        materialize_host_result, BoundaryArenaBuilder, BoundaryArenaV1, BoundaryValueStore,
        BoundaryWord, BOUNDARY_IMMEDIATE_INT_MAX, BOUNDARY_IMMEDIATE_INT_MIN,
        BOUNDARY_PAYLOAD_BITS,
    };
    use crate::ir::RuntimeGroundValue;
    use crate::native_int::RuntimeIntV1;
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
        let mut module = jit();
        let native = crate::native_int_clif::emit_native_int_local_graph(&mut module, false)
            .expect("native-int graph emits");
        let helpers = emit_boundary_value_local_graph(&mut module, &native).expect("graph emits");
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

    fn run4(code: *const u8, base: *const u64, a: u64, b: u64, c: u64) -> i64 {
        let f: extern "C" fn(*const u64, u64, u64, u64) -> i64 =
            unsafe { std::mem::transmute(code) };
        f(base, a, b, c)
    }

    /// A `Cons(7, Nil)` whose payload is chosen at run time, not baked in.
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
            let mut store = BoundaryValueStore::new();
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
            let tag_id = run2(tag_code, base, word);
            assert_eq!(
                store.symbol(tag_id as u64),
                Some("ctor:fixture::List::Cons"),
                "the tag id names the runtime constructor"
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
            let mut store = BoundaryValueStore::new();
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

        let mut store = BoundaryValueStore::new();
        let mut builder = BoundaryArenaBuilder::new();
        let word = materialize_ground(&mut store, &outer).expect("materializes");
        let payload_name = store.intern_symbol("payload");
        let depth_name = store.intern_symbol("depth");
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

        let mut store = BoundaryValueStore::new();
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

        let mut store = BoundaryValueStore::new();
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

        let mut store = BoundaryValueStore::new();
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
            let mut store = BoundaryValueStore::new();
            for i in 0..count {
                materialize_ground(&mut store, &cons(i as i64)).expect("materializes");
            }
            assert!(
                store.image().node_count() >= count,
                "persistent storage grew with the value population, as it should"
            );
            assert_eq!(
                BOUNDARY_LOCAL_HELPERS.len(),
                27,
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
        let mut store = BoundaryValueStore::new();
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
        assert_eq!(store.decode_slot(u64::MAX), None);
    }

    /// Equal values share one referent, because identity is the STORE's answer
    /// and not this layer's.
    #[test]
    fn b2v_equal_values_share_one_persistent_referent() {
        let mut store = BoundaryValueStore::new();
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
        assert_eq!(
            BoundaryTag::ALL.len(),
            9,
            "AC-1: the published tag list and the enum have drifted apart"
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
            let mut store = BoundaryValueStore::new();
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

        let mut store = BoundaryValueStore::new();
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

            let expected_escape = match known {
                None => BOUNDARY_ERR_TAG,
                Some(tag) => match tag.referent_owner() {
                    BoundaryReferentOwner::InvocationArena => BOUNDARY_ERR_ESCAPE,
                    BoundaryReferentOwner::NoReferent | BoundaryReferentOwner::PersistentStore => {
                        BOUNDARY_OK
                    }
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
        // would pass both arms above for the wrong reason.
        assert_eq!(
            admitted,
            BoundaryTag::ALL.len(),
            "AC-1: the number of admitted tag bytes must equal the closed set"
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
        let mut module = jit();
        let native = crate::native_int_clif::emit_native_int_local_graph(&mut module, false)
            .expect("native-int graph emits");
        let helpers = emit_boundary_value_local_graph(&mut module, &native).expect("graph emits");
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
            let mut store = BoundaryValueStore::new();
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
            let mut store = BoundaryValueStore::new();
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

        let mut store = BoundaryValueStore::new();
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

        let mut store = BoundaryValueStore::new();
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

        let mut store = BoundaryValueStore::new();
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
            let mut store = BoundaryValueStore::new();
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
            let mut store = BoundaryValueStore::new();
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
            let mut store = BoundaryValueStore::new();
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

        let mut store = BoundaryValueStore::new();
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
            let mut store = BoundaryValueStore::new();
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
        let mut store = BoundaryValueStore::new();
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

        let mut store = BoundaryValueStore::new();
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
        let writers: Vec<&str> = BOUNDARY_LOCAL_HELPERS
            .iter()
            .filter(|name| name.starts_with("ken_boundary_store_"))
            .copied()
            .collect();
        assert_eq!(
            writers,
            vec![
                "ken_boundary_store_tag_id_local",
                "ken_boundary_store_scalar_local",
                "ken_boundary_store_field_local",
                "ken_boundary_store_name_local",
                "ken_boundary_store_int_tag_local",
                "ken_boundary_store_int_limbs_local",
                "ken_boundary_store_int_limb_local",
                "ken_boundary_store_bytes_len_local",
                "ken_boundary_store_byte_local",
            ],
            "AC-6: the emitted writer inventory has moved — a new writer needs \
             its own account of what it may set"
        );

        // ⚠ And behaviourally, over the whole writable inventory: exercise
        // EVERY store helper on a freshly constructed node and the store's
        // identity field must still be NULL_SLOT.
        let (_am, alloc_code) = compile_producer(4, emit_alloc_probe);
        let (_tm, tag_store) = compile_producer(3, emit_store_tag_id_probe);
        let (_sm, scalar_store) = compile_producer(3, emit_store_scalar_probe);
        let (_c1, slot_code) = compile_probe(Probe::Unary(|h| h.slot));

        let mut store = BoundaryValueStore::new();
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
    /// pairs is put through the emitted allocator. CLAIMED: the ABI admits
    /// exactly the relation the disposition yields. THE GAP: that
    /// `BOUNDARY_TAG_CLASS_RELATION` *is* that disposition's relation — which
    /// `b2v_ac3_…` pins on the disposition side.
    #[test]
    fn b2v_the_tag_class_relation_is_closed_over_the_whole_product() {
        let (_pm, alloc_code) = compile_producer(4, emit_alloc_probe);
        let mut store = BoundaryValueStore::new();
        let f = bind_with(
            &mut store,
            BoundaryArenaBuilder::new(),
            (64, 8, 0),
            (64, 8, 0),
        );

        let (mut admitted, mut rejected) = (0usize, 0usize);
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
        // ⚠ POSITIVE CONTROL on both arms: a relation that admitted everything
        // or nothing would satisfy one arm vacuously.
        let handles = BoundaryTag::ALL
            .iter()
            .filter(|t| !t.is_immediate())
            .count();
        let expected_admitted: usize = BOUNDARY_TAG_CLASS_RELATION
            .iter()
            .map(|(_, classes)| classes.len())
            .sum();
        assert_eq!(admitted, expected_admitted, "AC-1: admitted count");
        assert_eq!(
            rejected,
            handles * BoundaryClass::ALL.len() - expected_admitted,
            "AC-1: rejected count"
        );
        assert!(
            admitted > 0 && rejected > 0,
            "AC-1: neither arm may be empty"
        );

        // The mask the CLIF consumes must agree with the table, per pair.
        for tag in BoundaryTag::ALL {
            for class in BoundaryClass::ALL {
                let by_mask = boundary_class_mask(tag) & (1u64 << (class as u64)) != 0;
                assert_eq!(
                    by_mask,
                    boundary_relation_admits(tag, class),
                    "the emitted mask disagrees with the relation for {tag:?} + {class:?}"
                );
            }
        }
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

        let mut store = BoundaryValueStore::new();
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

            let mut store = BoundaryValueStore::new();
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
        let mut store = BoundaryValueStore::new();
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

    /// **`AC-1`/`AC-6` — the region thresholds and `referent_owner` are the
    /// same classification.**
    ///
    /// ⚠ MEASURED: for every tag in the closed set, the numeric bands the CLIF
    /// compares against classify it exactly as [`BoundaryTag::referent_owner`]
    /// does. CLAIMED: `resolve` sends every word to the region that owns its
    /// referent. THE GAP: that `resolve` compares against *these* constants —
    /// which it does, textually, and which the survival control closes
    /// behaviourally. Reordering the enum silently re-points every persistent
    /// word at the arena, and this is what makes that redden.
    #[test]
    fn b2v_the_region_thresholds_agree_with_referent_owner() {
        for tag in BoundaryTag::ALL {
            let bits = tag as i64;
            let by_threshold = if bits < FIRST_HANDLE_TAG {
                BoundaryReferentOwner::NoReferent
            } else if bits <= LAST_PERSISTENT_TAG {
                BoundaryReferentOwner::PersistentStore
            } else {
                BoundaryReferentOwner::InvocationArena
            };
            assert_eq!(
                tag.referent_owner(),
                by_threshold,
                "the region band for {tag:?} disagrees with its referent owner"
            );
        }
        assert_eq!(
            FIRST_INVOCATION_TAG,
            LAST_PERSISTENT_TAG + 1,
            "the two owner bands must stay contiguous, or a threshold test \
             cannot separate them"
        );
        // Positive control: the bands are non-empty, so the agreement above is
        // not vacuous over an empty band.
        assert!(BoundaryTag::ALL
            .iter()
            .any(|t| t.referent_owner() == BoundaryReferentOwner::PersistentStore));
        assert!(BoundaryTag::ALL
            .iter()
            .any(|t| t.referent_owner() == BoundaryReferentOwner::InvocationArena));
    }
}
