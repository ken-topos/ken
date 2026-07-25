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
    BoundaryClass, BoundaryTag, ARENA_NAMES, ARENA_NODES, ARENA_NODE_COUNT, ARENA_WORDS,
    BOUNDARY_ERR_BOUNDS, BOUNDARY_ERR_CLASS, BOUNDARY_ERR_ESCAPE, BOUNDARY_ERR_SHAPE,
    BOUNDARY_ERR_TAG, BOUNDARY_NODE_STRIDE, BOUNDARY_OK, BOUNDARY_TAG_BITS, BOUNDARY_TAG_MASK,
    NODE_CLASS, NODE_FIELDS_AT, NODE_FIELD_COUNT, NODE_OWNER, NODE_PAYLOAD, NODE_SLOT, NODE_TAG_ID,
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
];

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
}

/// Emit the boundary-value helper graph into `module`.
///
/// ⛔ **`D6` — INERT.** This declares and defines helpers and nothing else. It
/// populates no generated function for any semantic origin, adds no cross-owner
/// call, and installs no second body-emission authority. `RT-FNSPLIT-B2F`
/// performs the switch-over that calls these.
pub(crate) fn emit_boundary_value_local_graph<M: Module>(
    module: &mut M,
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
    })
}

/// Capture every helper body as text, for the JIT/object identity pin and the
/// closed-inventory pin. Test-only.
#[cfg(test)]
pub(crate) fn capture_boundary_value_local_graph<M: Module>(
    module: &mut M,
) -> Result<String, CraneliftBackendError> {
    BOUNDARY_CLIF_CAPTURE.with(|capture| *capture.borrow_mut() = Some(Vec::new()));
    emit_boundary_value_local_graph(module)?;
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
/// The highest tag in the closed set.
const LAST_TAG: i64 = BoundaryTag::InvocationHostResult as i64;

/// `(arena, word, out) -> status`, writing the node's base address to `*out`.
///
/// ⭐ **The only place a word becomes an address.** Every projection routes
/// through it, so the bounds check below is not one of many — it is *the* one.
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
        let known = b.ins().icmp_imm(IntCC::UnsignedLessThanOrEqual, tag, LAST_TAG);
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

        b.switch_to_block(handle);
        let index = b.ins().ushr_imm(word, i64::from(BOUNDARY_TAG_BITS));
        let count = b
            .ins()
            .load(types::I64, MemFlags::trusted(), arena, ARENA_NODE_COUNT);
        let within = b.ins().icmp(IntCC::UnsignedLessThan, index, count);
        let ok = b.create_block();
        let oob = b.create_block();
        b.ins().brif(within, ok, &[], oob, &[]);

        b.switch_to_block(oob);
        let err = b.ins().iconst(types::I64, BOUNDARY_ERR_BOUNDS);
        b.ins().return_(&[err]);

        b.switch_to_block(ok);
        let nodes = b.ins().load(ptr, MemFlags::trusted(), arena, ARENA_NODES);
        let offset = b.ins().imul_imm(index, i64::from(BOUNDARY_NODE_STRIDE));
        let node = b.ins().iadd(nodes, offset);
        b.ins().store(MemFlags::trusted(), node, out, 0);
        let z = b.ins().iconst(types::I64, BOUNDARY_OK);
        b.ins().return_(&[z]);

        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.resolve, func)
}

/// Emit the shared prologue: resolve `word` to a node address, branching to
/// `bad` on any non-zero status with that status returned.
///
/// Returns the node address in the current (resolved) block.
fn resolve_prologue(
    b: &mut FunctionBuilder<'_>,
    ptr: cranelift_codegen::ir::Type,
    resolve: cranelift_codegen::ir::FuncRef,
    arena: cranelift_codegen::ir::Value,
    word: cranelift_codegen::ir::Value,
) -> cranelift_codegen::ir::Value {
    let slot = b.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
        cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
        8,
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
    b.ins().load(ptr, MemFlags::trusted(), cell, 0)
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
        let node = resolve_prologue(&mut b, ptr, resolve, arena, word);
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

        let cell_slot = b.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            8,
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
        let class = b.ins().load(types::I64, MemFlags::trusted(), node, NODE_CLASS);
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

        let cell_slot = b.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            8,
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
        let node = resolve_prologue(&mut b, ptr, resolve, arena, word);

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
        let words = b.ins().load(ptr, MemFlags::trusted(), arena, ARENA_WORDS);
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
        let node = resolve_prologue(&mut b, ptr, resolve, arena, word);

        // ⛔ Class-checked: a positional aggregate has a parallel name table of
        // zeroes, and a caller asking it for a named field is asking a question
        // it cannot answer. That is `ERR_CLASS`, not "not found".
        let class = b.ins().load(types::I64, MemFlags::trusted(), node, NODE_CLASS);
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
        let names = b.ins().load(ptr, MemFlags::trusted(), arena, ARENA_NAMES);
        let words = b.ins().load(ptr, MemFlags::trusted(), arena, ARENA_WORDS);

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
        let node = resolve_prologue(&mut b, ptr, resolve, arena, word);
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
        let node = resolve_prologue(&mut b, ptr, resolve, arena, word);
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
        let words = b.ins().load(ptr, MemFlags::trusted(), arena, ARENA_WORDS);
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
    let class = b.ins().load(types::I64, MemFlags::trusted(), node, NODE_CLASS);
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
        let shifted = b.ins().ishl_imm(payload, i64::from(BOUNDARY_TAG_BITS));
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
        let known = b.ins().icmp_imm(IntCC::UnsignedLessThanOrEqual, tag, LAST_TAG);
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
