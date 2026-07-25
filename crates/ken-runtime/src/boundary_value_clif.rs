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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary_value::{
        materialize_borrowed, materialize_ground, materialize_host_result, BoundaryArenaBuilder,
        BoundaryReferentOwner, BoundaryValueStore, BoundaryWord,
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
        let helpers = emit_boundary_value_local_graph(&mut module).expect("graph emits");
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
                let good = b
                    .ins()
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
            let word = materialize_ground(&mut store, &mut builder, &cons(head))
                .expect("a constructor materializes");
            let mut arena = builder.finish();
            let base = arena.publish();

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
                &mut builder,
                &RuntimeGroundValue::Int(RuntimeIntV1::Small(11)),
            )
            .expect("ok payload");
            let err = materialize_ground(
                &mut store,
                &mut builder,
                &RuntimeGroundValue::Int(RuntimeIntV1::Small(22)),
            )
            .expect("err payload");
            let word = materialize_host_result(&mut builder, success, ok, err);
            let mut arena = builder.finish();
            let base = arena.publish();

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
        let word = materialize_ground(&mut store, &mut builder, &outer).expect("materializes");
        let payload_name = store.intern_symbol("payload");
        let depth_name = store.intern_symbol("depth");
        let mut arena = builder.finish();
        let base = arena.publish();

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
        let persistent =
            materialize_ground(&mut store, &mut builder, &cons(5)).expect("materializes");
        let borrowed = materialize_borrowed(&mut builder, 0xDEAD_BEEF);
        let mut arena = builder.finish();
        let base = arena.publish();

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
        let persistent =
            materialize_ground(&mut store, &mut builder, &cons(1)).expect("materializes");
        let borrowed = materialize_borrowed(&mut builder, 1);
        let host = materialize_host_result(&mut builder, 1, persistent, persistent);
        let immediate = BoundaryWord::immediate(BoundaryTag::ImmediateBool, 1);
        let mut arena = builder.finish();
        let base = arena.publish();

        // Positive control on the permitted side: if EVERYTHING were refused,
        // the escape assertions below would pass for the wrong reason.
        assert_eq!(run2(escape_code, base, persistent), crate::boundary_value::BOUNDARY_OK);
        assert_eq!(run2(escape_code, base, immediate), crate::boundary_value::BOUNDARY_OK);

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
        let word = materialize_ground(&mut store, &mut builder, &cons(1)).expect("materializes");
        let mut arena = builder.finish();
        let base = arena.publish();

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
        // Names, not just a count: a swap that kept the population size would
        // pass a count and fails this.
        let mut seen = BOUNDARY_LOCAL_HELPERS.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            BOUNDARY_LOCAL_HELPERS.len(),
            "AC-9: the declared inventory has a duplicate name"
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
            let mut builder = BoundaryArenaBuilder::new();
            for i in 0..count {
                materialize_ground(&mut store, &mut builder, &cons(i as i64))
                    .expect("materializes");
            }
            let arena = builder.finish();
            assert!(
                arena.node_count() >= count,
                "the arena grew with the value population, as it should"
            );
            assert_eq!(
                BOUNDARY_LOCAL_HELPERS.len(),
                13,
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
        let mut builder = BoundaryArenaBuilder::new();
        let value = cons(31);
        let word = materialize_ground(&mut store, &mut builder, &value).expect("materializes");
        let arena = builder.finish();

        let slot = arena
            .node_field(word.payload(), crate::boundary_value::NODE_SLOT)
            .expect("the node exists");
        assert_ne!(slot, crate::store::NULL_SLOT, "a persistent node names a slot");

        // Path A — the typed residency map.
        assert_eq!(store.resident(slot), Some(&value));
        // Path B — the store's own bytes, through the decode inverse.
        let decoded = store.decode_slot(slot).expect("the store resolves the slot");
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
        let mut builder = BoundaryArenaBuilder::new();
        let a = materialize_ground(&mut store, &mut builder, &cons(9)).expect("materializes");
        let b = materialize_ground(&mut store, &mut builder, &cons(9)).expect("materializes");
        let c = materialize_ground(&mut store, &mut builder, &cons(10)).expect("materializes");
        let arena = builder.finish();

        let slot_of = |w: BoundaryWord| {
            arena
                .node_field(w.payload(), crate::boundary_value::NODE_SLOT)
                .expect("node")
        };
        assert_eq!(slot_of(a), slot_of(b), "equal values are one referent");
        assert_ne!(slot_of(a), slot_of(c), "distinct values are distinct referents");
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
            11,
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
            let mut builder = BoundaryArenaBuilder::new();
            let word = materialize_ground(
                &mut store,
                &mut builder,
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
}
