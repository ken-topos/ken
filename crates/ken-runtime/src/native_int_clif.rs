//! Compiler-private exact-`Int` support emitted into every native module.
//!
//! This module deliberately contains the only executable arena-layout
//! knowledge.  The JIT and linked starters own only a zeroed header and its
//! lifetime; all validation and integer semantics live in the local CLIF graph.

use cranelift_codegen::ir::{
    types, AbiParam, Function, InstBuilder, MemFlags, StackSlotData, StackSlotKind, UserFuncName,
};
use cranelift_codegen::verify_function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage, Module};

use crate::cranelift_backend::{backend_module, CraneliftBackendError};

#[cfg(test)]
thread_local! {
    static LOCAL_CLIF_CAPTURE: std::cell::RefCell<Option<Vec<String>>> = const {
        std::cell::RefCell::new(None)
    };
}

pub(crate) const ARENA_HEAD: i32 = 0;
pub(crate) const ARENA_NEXT_SLOT: i32 = 8;
pub(crate) const ARENA_FINAL_TAG: i32 = 16;
pub(crate) const ARENA_FINAL_PAYLOAD: i32 = 24;
pub(crate) const ARENA_FINAL_SIGN: i32 = 32;
pub(crate) const ARENA_FINAL_LEN: i32 = 40;
pub(crate) const ARENA_FINAL_LIMBS: i32 = 48;
pub(crate) const ARENA_FINAL_SMALL: i32 = 56;
#[cfg(test)]
pub(crate) const ARENA_BYTES: usize = 64;

const ENTRY_NEXT: i32 = 0;
const ENTRY_SLOT: i32 = 8;
const ENTRY_SIGN: i32 = 16;
const ENTRY_LEN: i32 = 24;
const ENTRY_LIMBS: i32 = 32;

const VIEW_SIGN: i32 = 0;
const VIEW_LEN: i32 = 8;
const VIEW_LIMBS: i32 = 16;
const VIEW_SMALL: i32 = 24;

#[derive(Clone, Copy)]
pub(crate) struct NativeIntLocalFuncs {
    pub binop: FuncId,
    pub compare: FuncId,
    pub intern: FuncId,
    pub narrow: FuncId,
    pub export: FuncId,
}

#[derive(Clone, Copy)]
struct NativeIntLocalGraph {
    malloc: FuncId,
    free: FuncId,
    resolve: FuncId,
    intern: FuncId,
    binop: FuncId,
    compare: FuncId,
    narrow: FuncId,
    export: FuncId,
    wrapping_mutation: bool,
}

pub(crate) fn emit_native_int_local_graph<M: Module>(
    module: &mut M,
    wrapping_mutation: bool,
) -> Result<NativeIntLocalFuncs, CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut malloc_sig = module.make_signature();
    malloc_sig.params.push(AbiParam::new(ptr));
    malloc_sig.returns.push(AbiParam::new(ptr));
    let malloc = module
        .declare_function("malloc", Linkage::Import, &malloc_sig)
        .map_err(|e| backend_module(e.to_string()))?;
    let mut free_sig = module.make_signature();
    free_sig.params.push(AbiParam::new(ptr));
    let free = module
        .declare_function("free", Linkage::Import, &free_sig)
        .map_err(|e| backend_module(e.to_string()))?;

    let resolve = declare(module, "ken_native_int_resolve_local", 4, 1)?;
    let intern = declare(module, "ken_native_int_intern_local", 5, 1)?;
    let binop = declare(module, "ken_native_int_binop_local", 7, 1)?;
    let compare = declare(module, "ken_native_int_compare_local", 6, 1)?;
    let narrow = declare(module, "ken_native_int_narrow_local", 4, 1)?;
    let export = declare(module, "ken_native_int_export_local", 3, 1)?;
    let graph = NativeIntLocalGraph {
        malloc,
        free,
        resolve,
        intern,
        binop,
        compare,
        narrow,
        export,
        wrapping_mutation,
    };
    define_resolve(module, graph)?;
    define_intern(module, graph)?;
    define_compare(module, graph)?;
    define_narrow(module, graph)?;
    define_export(module, graph)?;
    define_binop(module, graph)?;
    Ok(NativeIntLocalFuncs {
        binop,
        compare,
        intern,
        narrow,
        export,
    })
}

#[cfg(test)]
pub(crate) fn capture_native_int_local_graph<M: Module>(
    module: &mut M,
) -> Result<String, CraneliftBackendError> {
    LOCAL_CLIF_CAPTURE.with(|capture| *capture.borrow_mut() = Some(Vec::new()));
    emit_native_int_local_graph(module, false)?;
    Ok(LOCAL_CLIF_CAPTURE.with(|capture| {
        capture
            .borrow_mut()
            .take()
            .expect("capture was installed")
            .join("\n-- helper --\n")
    }))
}

fn declare<M: Module>(
    module: &mut M,
    name: &str,
    params: usize,
    returns: usize,
) -> Result<FuncId, CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut sig = module.make_signature();
    // Every helper argument/result is one native word on the supported target.
    // Pointer arguments are passed as words and used at pointer type below.
    for _ in 0..params {
        sig.params.push(AbiParam::new(ptr));
    }
    for _ in 0..returns {
        sig.returns.push(AbiParam::new(types::I64));
    }
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
    Function::with_name_signature(UserFuncName::user(1, id.as_u32()), sig)
}

fn finish<M: Module>(
    module: &mut M,
    id: FuncId,
    mut func: Function,
) -> Result<(), CraneliftBackendError> {
    verify_function(&func, module.isa())
        .map_err(|e| backend_module(format!("native Int local helper verification: {e}")))?;
    #[cfg(test)]
    LOCAL_CLIF_CAPTURE.with(|capture| {
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

fn define_resolve<M: Module>(
    module: &mut M,
    graph: NativeIntLocalGraph,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.resolve, 4);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let arena = p[0];
        let tag = p[1];
        let payload = p[2];
        let out = p[3];
        let small = b.create_block();
        let big = b.create_block();
        let bad = b.create_block();
        let is_small = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, tag, 0);
        b.ins().brif(is_small, small, &[], big, &[]);
        b.switch_to_block(small);
        let neg = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
            payload,
            0,
        );
        let mag_neg = b.ins().ineg(payload);
        let mag = b.ins().select(neg, mag_neg, payload);
        let sign = b.ins().uextend(types::I64, neg);
        b.ins().store(MemFlags::trusted(), sign, out, VIEW_SIGN);
        let one = b.ins().iconst(types::I64, 1);
        b.ins().store(MemFlags::trusted(), one, out, VIEW_LEN);
        b.ins().store(MemFlags::trusted(), mag, out, VIEW_SMALL);
        let limb_ptr = b.ins().iadd_imm(out, i64::from(VIEW_SMALL));
        b.ins()
            .store(MemFlags::trusted(), limb_ptr, out, VIEW_LIMBS);
        let ok = b.ins().iconst(types::I64, 0);
        b.ins().return_(&[ok]);
        b.switch_to_block(big);
        let is_big = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, tag, 1);
        let nonzero = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::NotEqual,
            payload,
            0,
        );
        let valid = b.ins().band(is_big, nonzero);
        let scan = b.create_block();
        b.append_block_param(scan, ptr);
        let head = b.ins().load(ptr, MemFlags::trusted(), arena, ARENA_HEAD);
        b.ins().brif(valid, scan, &[head.into()], bad, &[]);
        b.switch_to_block(scan);
        let node = b.block_params(scan)[0];
        let absent = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, node, 0);
        let check = b.create_block();
        b.ins().brif(absent, bad, &[], check, &[]);
        b.switch_to_block(check);
        let slot = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, ENTRY_SLOT);
        let found = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            slot,
            payload,
        );
        let hit = b.create_block();
        let next = b.create_block();
        b.ins().brif(found, hit, &[], next, &[]);
        b.switch_to_block(hit);
        let sign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, ENTRY_SIGN);
        let len = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, ENTRY_LEN);
        let limbs = b.ins().iadd_imm(node, i64::from(ENTRY_LIMBS));
        b.ins().store(MemFlags::trusted(), sign, out, VIEW_SIGN);
        b.ins().store(MemFlags::trusted(), len, out, VIEW_LEN);
        b.ins().store(MemFlags::trusted(), limbs, out, VIEW_LIMBS);
        let ok = b.ins().iconst(types::I64, 0);
        b.ins().return_(&[ok]);
        b.switch_to_block(next);
        let node = b.ins().load(ptr, MemFlags::trusted(), node, ENTRY_NEXT);
        b.ins().jump(scan, &[node.into()]);
        b.switch_to_block(bad);
        let err = b.ins().iconst(types::I64, -1);
        b.ins().return_(&[err]);
        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.resolve, func)
}

// The remaining definitions are intentionally kept together: their bodies are
// the canonical executable arithmetic graph shared by JIT and object modules.
fn define_intern<M: Module>(
    module: &mut M,
    graph: NativeIntLocalGraph,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.intern, 5);
    let malloc = module.declare_func_in_func(graph.malloc, &mut func);
    let free = module.declare_func_in_func(graph.free, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let arena = p[0];
        let sign = p[1];
        let limbs = p[2];
        let len = p[3];
        let out = p[4];
        let bad = b.create_block();
        let trim = b.create_block();
        b.append_block_param(trim, types::I64);
        let arena_ok =
            b.ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, arena, 0);
        let limbs_ok =
            b.ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, limbs, 0);
        let out_ok = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, out, 0);
        let len_ok = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, len, 0);
        let sign_ok = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            sign,
            1,
        );
        let aok = b.ins().band(arena_ok, limbs_ok);
        let lok = b.ins().band(len_ok, sign_ok);
        let ook = b.ins().band(out_ok, lok);
        let ok = b.ins().band(aok, ook);
        b.ins().brif(ok, trim, &[len.into()], bad, &[]);
        b.switch_to_block(trim);
        let n = b.block_params(trim)[0];
        let gt1 = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            n,
            1,
        );
        let check = b.create_block();
        let canonical = b.create_block();
        b.append_block_param(canonical, types::I64);
        b.ins().brif(gt1, check, &[], canonical, &[n.into()]);
        b.switch_to_block(check);
        let last_index = b.ins().iadd_imm(n, -1);
        let last_offset = b.ins().ishl_imm(last_index, 3);
        let last = b.ins().iadd(limbs, last_offset);
        let word = b.ins().load(types::I64, MemFlags::trusted(), last, 0);
        let zero = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, word, 0);
        let dec = b.ins().iadd_imm(n, -1);
        b.ins()
            .brif(zero, trim, &[dec.into()], canonical, &[n.into()]);
        b.switch_to_block(canonical);
        let n = b.block_params(canonical)[0];
        let first = b.ins().load(types::I64, MemFlags::trusted(), limbs, 0);
        let one = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, n, 1);
        let first_zero = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, first, 0);
        let magnitude_zero = b.ins().band(one, first_zero);
        let zero_word = b.ins().iconst(types::I64, 0);
        let normalized_sign = b.ins().select(magnitude_zero, zero_word, sign);
        let positive_fit = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
            first,
            0,
        );
        let neg_limit = b.ins().iconst(types::I64, i64::MIN);
        let negative_fit = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            first,
            neg_limit,
        );
        let is_neg = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            normalized_sign,
            1,
        );
        let fits = b.ins().select(is_neg, negative_fit, positive_fit);
        let small = b.ins().band(one, fits);
        let emit_small = b.create_block();
        let scan = b.create_block();
        b.append_block_param(scan, ptr);
        let head = b.ins().load(ptr, MemFlags::trusted(), arena, ARENA_HEAD);
        b.ins().brif(small, emit_small, &[], scan, &[head.into()]);
        b.switch_to_block(emit_small);
        let neg_first = b.ins().ineg(first);
        let payload = b.ins().select(is_neg, neg_first, first);
        let z = b.ins().iconst(types::I64, 0);
        b.ins().store(MemFlags::trusted(), z, out, 0);
        b.ins().store(MemFlags::trusted(), payload, out, 8);
        b.ins().return_(&[z]);
        b.switch_to_block(scan);
        let node = b.block_params(scan)[0];
        let absent = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, node, 0);
        let examine = b.create_block();
        let allocate = b.create_block();
        b.ins().brif(absent, allocate, &[], examine, &[]);
        b.switch_to_block(examine);
        let esign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, ENTRY_SIGN);
        let elen = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, ENTRY_LEN);
        let same_sign = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            esign,
            normalized_sign,
        );
        let same_len = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, elen, n);
        let maybe = b.ins().band(same_sign, same_len);
        let compare = b.create_block();
        b.append_block_param(compare, types::I64);
        let advance = b.create_block();
        let zero_index = b.ins().iconst(types::I64, 0);
        b.ins()
            .brif(maybe, compare, &[zero_index.into()], advance, &[]);
        b.switch_to_block(compare);
        let i = b.block_params(compare)[0];
        let done = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, i, n);
        let hit = b.create_block();
        let wordcheck = b.create_block();
        b.ins().brif(done, hit, &[], wordcheck, &[]);
        b.switch_to_block(wordcheck);
        let off = b.ins().ishl_imm(i, 3);
        let ap = b.ins().iadd(limbs, off);
        let a = b.ins().load(types::I64, MemFlags::trusted(), ap, 0);
        let ebase = b.ins().iadd_imm(node, i64::from(ENTRY_LIMBS));
        let ep = b.ins().iadd(ebase, off);
        let c = b.ins().load(types::I64, MemFlags::trusted(), ep, 0);
        let eq = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, a, c);
        let next = b.ins().iadd_imm(i, 1);
        b.ins().brif(eq, compare, &[next.into()], advance, &[]);
        b.switch_to_block(hit);
        let slot = b
            .ins()
            .load(types::I64, MemFlags::trusted(), node, ENTRY_SLOT);
        let one_tag = b.ins().iconst(types::I64, 1);
        b.ins().store(MemFlags::trusted(), one_tag, out, 0);
        b.ins().store(MemFlags::trusted(), slot, out, 8);
        let z = b.ins().iconst(types::I64, 0);
        b.ins().return_(&[z]);
        b.switch_to_block(advance);
        let next = b.ins().load(ptr, MemFlags::trusted(), node, ENTRY_NEXT);
        b.ins().jump(scan, &[next.into()]);
        b.switch_to_block(allocate);
        let limb_bytes = b.ins().ishl_imm(n, 3);
        let bytes = b.ins().iadd_imm(limb_bytes, i64::from(ENTRY_LIMBS));
        let call = b.ins().call(malloc, &[bytes]);
        let node = b.inst_results(call)[0];
        let null = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, node, 0);
        let init = b.create_block();
        b.ins().brif(null, bad, &[], init, &[]);
        b.switch_to_block(init);
        let old_head = b.ins().load(ptr, MemFlags::trusted(), arena, ARENA_HEAD);
        let old_slot = b
            .ins()
            .load(types::I64, MemFlags::trusted(), arena, ARENA_NEXT_SLOT);
        let max = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, old_slot, -1);
        let init_ok = b.create_block();
        let free_bad = b.create_block();
        b.ins().brif(max, free_bad, &[], init_ok, &[]);
        b.switch_to_block(free_bad);
        b.ins().call(free, &[node]);
        b.ins().jump(bad, &[]);
        b.switch_to_block(init_ok);
        let slot = b.ins().iadd_imm(old_slot, 1);
        b.ins()
            .store(MemFlags::trusted(), old_head, node, ENTRY_NEXT);
        b.ins().store(MemFlags::trusted(), slot, node, ENTRY_SLOT);
        b.ins()
            .store(MemFlags::trusted(), normalized_sign, node, ENTRY_SIGN);
        b.ins().store(MemFlags::trusted(), n, node, ENTRY_LEN);
        let copy = b.create_block();
        b.append_block_param(copy, types::I64);
        let copied = b.create_block();
        let zero_index = b.ins().iconst(types::I64, 0);
        b.ins().jump(copy, &[zero_index.into()]);
        b.switch_to_block(copy);
        let i = b.block_params(copy)[0];
        let done = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, i, n);
        let copy_word = b.create_block();
        b.ins().brif(done, copied, &[], copy_word, &[]);
        b.switch_to_block(copy_word);
        let off = b.ins().ishl_imm(i, 3);
        let src = b.ins().iadd(limbs, off);
        let word = b.ins().load(types::I64, MemFlags::trusted(), src, 0);
        let base = b.ins().iadd_imm(node, i64::from(ENTRY_LIMBS));
        let dst = b.ins().iadd(base, off);
        b.ins().store(MemFlags::trusted(), word, dst, 0);
        let next_i = b.ins().iadd_imm(i, 1);
        b.ins().jump(copy, &[next_i.into()]);
        b.switch_to_block(copied);
        b.ins().store(MemFlags::trusted(), node, arena, ARENA_HEAD);
        b.ins()
            .store(MemFlags::trusted(), slot, arena, ARENA_NEXT_SLOT);
        let one_tag = b.ins().iconst(types::I64, 1);
        b.ins().store(MemFlags::trusted(), one_tag, out, 0);
        b.ins().store(MemFlags::trusted(), slot, out, 8);
        let z = b.ins().iconst(types::I64, 0);
        b.ins().return_(&[z]);
        b.switch_to_block(bad);
        let err = b.ins().iconst(types::I64, -1);
        b.ins().return_(&[err]);
        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.intern, func)
}
fn define_compare<M: Module>(
    module: &mut M,
    graph: NativeIntLocalGraph,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.compare, 6);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let arena = p[0];
        let op = p[1];
        let ls = b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 32, 0));
        let rs = b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 32, 0));
        let lv = b.ins().stack_addr(ptr, ls, 0);
        let rv = b.ins().stack_addr(ptr, rs, 0);
        let lc = b.ins().call(resolve, &[arena, p[2], p[3], lv]);
        let lstat = b.inst_results(lc)[0];
        let rc = b.ins().call(resolve, &[arena, p[4], p[5], rv]);
        let rstat = b.inst_results(rc)[0];
        let status = b.ins().bor(lstat, rstat);
        let valid = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, status, 0);
        let start = b.create_block();
        let bad = b.create_block();
        b.ins().brif(valid, start, &[], bad, &[]);
        b.switch_to_block(start);
        let lsign = b.ins().load(types::I64, MemFlags::trusted(), lv, VIEW_SIGN);
        let rsign = b.ins().load(types::I64, MemFlags::trusted(), rv, VIEW_SIGN);
        let signs_eq = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, lsign, rsign);
        let same_sign = b.create_block();
        let diff_sign = b.create_block();
        b.ins().brif(signs_eq, same_sign, &[], diff_sign, &[]);
        b.switch_to_block(diff_sign);
        let lneg = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, lsign, 1);
        let minus = b.ins().iconst(types::I64, -1);
        let plus = b.ins().iconst(types::I64, 1);
        let order = b.ins().select(lneg, minus, plus);
        let answer = b.create_block();
        b.append_block_param(answer, types::I64);
        b.ins().jump(answer, &[order.into()]);
        b.switch_to_block(same_sign);
        let llen = b.ins().load(types::I64, MemFlags::trusted(), lv, VIEW_LEN);
        let rlen = b.ins().load(types::I64, MemFlags::trusted(), rv, VIEW_LEN);
        let len_eq = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, llen, rlen);
        let lengths = b.create_block();
        let words = b.create_block();
        b.append_block_param(words, types::I64);
        b.ins().brif(len_eq, words, &[llen.into()], lengths, &[]);
        b.switch_to_block(lengths);
        let mag_less = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            llen,
            rlen,
        );
        let minus = b.ins().iconst(types::I64, -1);
        let plus = b.ins().iconst(types::I64, 1);
        let mag_order = b.ins().select(mag_less, minus, plus);
        let neg = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, lsign, 1);
        let neg_order = b.ins().ineg(mag_order);
        let order = b.ins().select(neg, neg_order, mag_order);
        b.ins().jump(answer, &[order.into()]);
        b.switch_to_block(words);
        let n = b.block_params(words)[0];
        let zero = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, n, 0);
        let equal = b.create_block();
        let word = b.create_block();
        b.ins().brif(zero, equal, &[], word, &[]);
        b.switch_to_block(word);
        let i = b.ins().iadd_imm(n, -1);
        let off = b.ins().ishl_imm(i, 3);
        let lbase = b.ins().load(ptr, MemFlags::trusted(), lv, VIEW_LIMBS);
        let rbase = b.ins().load(ptr, MemFlags::trusted(), rv, VIEW_LIMBS);
        let lp = b.ins().iadd(lbase, off);
        let rp = b.ins().iadd(rbase, off);
        let lw = b.ins().load(types::I64, MemFlags::trusted(), lp, 0);
        let rw = b.ins().load(types::I64, MemFlags::trusted(), rp, 0);
        let eq = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, lw, rw);
        let differ = b.create_block();
        b.ins().brif(eq, words, &[i.into()], differ, &[]);
        b.switch_to_block(differ);
        let less = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            lw,
            rw,
        );
        let minus = b.ins().iconst(types::I64, -1);
        let plus = b.ins().iconst(types::I64, 1);
        let mag_order = b.ins().select(less, minus, plus);
        let neg = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, lsign, 1);
        let neg_order = b.ins().ineg(mag_order);
        let order = b.ins().select(neg, neg_order, mag_order);
        b.ins().jump(answer, &[order.into()]);
        b.switch_to_block(equal);
        let zero_order = b.ins().iconst(types::I64, 0);
        b.ins().jump(answer, &[zero_order.into()]);
        b.switch_to_block(answer);
        let order = b.block_params(answer)[0];
        let is_eq = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, order, 0);
        let is_le = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
            order,
            0,
        );
        let op_eq = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, op, 0);
        let op_le = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, op, 1);
        let op_valid = b.ins().bor(op_eq, op_le);
        let result = b.ins().select(op_eq, is_eq, is_le);
        let result = b.ins().uextend(types::I64, result);
        let ret = b.create_block();
        b.ins().brif(op_valid, ret, &[], bad, &[]);
        b.switch_to_block(ret);
        b.ins().return_(&[result]);
        b.switch_to_block(bad);
        let err = b.ins().iconst(types::I64, -1);
        b.ins().return_(&[err]);
        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.compare, func)
}
fn define_narrow<M: Module>(
    module: &mut M,
    graph: NativeIntLocalGraph,
) -> Result<(), CraneliftBackendError> {
    define_view_consumer(module, graph, graph.narrow, true)
}
fn define_export<M: Module>(
    module: &mut M,
    graph: NativeIntLocalGraph,
) -> Result<(), CraneliftBackendError> {
    define_view_consumer(module, graph, graph.export, false)
}
fn define_view_consumer<M: Module>(
    module: &mut M,
    graph: NativeIntLocalGraph,
    id: FuncId,
    narrow: bool,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let params = if narrow { 4 } else { 3 };
    let mut func = begin(module, id, params);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let arena = p[0];
        let tag = p[1];
        let payload = p[2];
        let slot =
            b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 32, 0));
        let view = b.ins().stack_addr(ptr, slot, 0);
        let call = b.ins().call(resolve, &[arena, tag, payload, view]);
        let status = b.inst_results(call)[0];
        let valid = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, status, 0);
        let ok = b.create_block();
        let bad = b.create_block();
        b.ins().brif(valid, ok, &[], bad, &[]);
        b.switch_to_block(ok);
        if narrow {
            let sign = b
                .ins()
                .load(types::I64, MemFlags::trusted(), view, VIEW_SIGN);
            let len = b
                .ins()
                .load(types::I64, MemFlags::trusted(), view, VIEW_LEN);
            let sign_ok = b
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, sign, 0);
            let len_ok = b
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, len, 1);
            let fits = b.ins().band(sign_ok, len_ok);
            let write = b.create_block();
            let range = b.create_block();
            b.ins().brif(fits, write, &[], range, &[]);
            b.switch_to_block(write);
            let limbs = b.ins().load(ptr, MemFlags::trusted(), view, VIEW_LIMBS);
            let word = b.ins().load(types::I64, MemFlags::trusted(), limbs, 0);
            b.ins().store(MemFlags::trusted(), word, p[3], 0);
            let z = b.ins().iconst(types::I64, 0);
            b.ins().return_(&[z]);
            b.switch_to_block(range);
            let one = b.ins().iconst(types::I64, 1);
            b.ins().return_(&[one]);
        } else {
            b.ins()
                .store(MemFlags::trusted(), tag, arena, ARENA_FINAL_TAG);
            b.ins()
                .store(MemFlags::trusted(), payload, arena, ARENA_FINAL_PAYLOAD);
            let sign = b
                .ins()
                .load(types::I64, MemFlags::trusted(), view, VIEW_SIGN);
            let len = b
                .ins()
                .load(types::I64, MemFlags::trusted(), view, VIEW_LEN);
            let limbs = b.ins().load(ptr, MemFlags::trusted(), view, VIEW_LIMBS);
            let first = b.ins().load(types::I64, MemFlags::trusted(), limbs, 0);
            b.ins()
                .store(MemFlags::trusted(), first, arena, ARENA_FINAL_SMALL);
            let small_ptr = b.ins().iadd_imm(arena, i64::from(ARENA_FINAL_SMALL));
            let is_small = b
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, tag, 0);
            let stable_limbs = b.ins().select(is_small, small_ptr, limbs);
            b.ins()
                .store(MemFlags::trusted(), sign, arena, ARENA_FINAL_SIGN);
            b.ins()
                .store(MemFlags::trusted(), len, arena, ARENA_FINAL_LEN);
            b.ins()
                .store(MemFlags::trusted(), stable_limbs, arena, ARENA_FINAL_LIMBS);
            let z = b.ins().iconst(types::I64, 0);
            b.ins().return_(&[z]);
        }
        b.switch_to_block(bad);
        let err = b.ins().iconst(types::I64, -1);
        b.ins().return_(&[err]);
        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, id, func)
}
fn define_binop<M: Module>(
    module: &mut M,
    graph: NativeIntLocalGraph,
) -> Result<(), CraneliftBackendError> {
    let ptr = module.target_config().pointer_type();
    let mut func = begin(module, graph.binop, 7);
    let resolve = module.declare_func_in_func(graph.resolve, &mut func);
    let intern = module.declare_func_in_func(graph.intern, &mut func);
    let malloc = module.declare_func_in_func(graph.malloc, &mut func);
    let free = module.declare_func_in_func(graph.free, &mut func);
    let mut fctx = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut func, &mut fctx);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let p = b.block_params(entry).to_vec();
        let arena = p[0];
        let op = p[1];
        let output = p[6];
        if graph.wrapping_mutation {
            let lhs_small =
                b.ins()
                    .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, p[2], 0);
            let rhs_small =
                b.ins()
                    .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, p[4], 0);
            let small = b.ins().band(lhs_small, rhs_small);
            let wrap = b.create_block();
            let reject = b.create_block();
            b.ins().brif(small, wrap, &[], reject, &[]);
            b.switch_to_block(wrap);
            let is_add = b
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, op, 0);
            let is_sub = b
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, op, 1);
            let add = b.ins().iadd(p[3], p[5]);
            let sub = b.ins().isub(p[3], p[5]);
            let mul = b.ins().imul(p[3], p[5]);
            let non_add = b.ins().select(is_sub, sub, mul);
            let value = b.ins().select(is_add, add, non_add);
            let zero = b.ins().iconst(types::I64, 0);
            b.ins().store(MemFlags::trusted(), zero, output, 0);
            b.ins().store(MemFlags::trusted(), value, output, 8);
            b.ins().return_(&[zero]);
            b.switch_to_block(reject);
            let error = b.ins().iconst(types::I64, -1);
            b.ins().return_(&[error]);
            b.seal_all_blocks();
            b.finalize();
            return finish(module, graph.binop, func);
        }
        let lhs_small = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, p[2], 0);
        let rhs_small = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, p[4], 0);
        let op_valid = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            op,
            2,
        );
        let both_small = b.ins().band(lhs_small, rhs_small);
        let fast_eligible = b.ins().band(both_small, op_valid);
        let small_fast = b.create_block();
        let general = b.create_block();
        b.ins().brif(fast_eligible, small_fast, &[], general, &[]);
        b.switch_to_block(small_fast);
        let (add, add_overflow) = b.ins().sadd_overflow(p[3], p[5]);
        let (sub, sub_overflow) = b.ins().ssub_overflow(p[3], p[5]);
        let (mul, mul_overflow) = b.ins().smul_overflow(p[3], p[5]);
        let is_add = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, op, 0);
        let is_sub = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, op, 1);
        let non_add_value = b.ins().select(is_sub, sub, mul);
        let value = b.ins().select(is_add, add, non_add_value);
        let non_add_overflow = b.ins().select(is_sub, sub_overflow, mul_overflow);
        let overflow = b.ins().select(is_add, add_overflow, non_add_overflow);
        let emit_small = b.create_block();
        b.ins().brif(overflow, general, &[], emit_small, &[]);
        b.switch_to_block(emit_small);
        let zero = b.ins().iconst(types::I64, 0);
        b.ins().store(MemFlags::trusted(), zero, output, 0);
        b.ins().store(MemFlags::trusted(), value, output, 8);
        b.ins().return_(&[zero]);
        b.switch_to_block(general);
        let lhs_slot =
            b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 32, 0));
        let rhs_slot =
            b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 32, 0));
        let lhs = b.ins().stack_addr(ptr, lhs_slot, 0);
        let rhs = b.ins().stack_addr(ptr, rhs_slot, 0);
        let lhs_call = b.ins().call(resolve, &[arena, p[2], p[3], lhs]);
        let lhs_status = b.inst_results(lhs_call)[0];
        let rhs_call = b.ins().call(resolve, &[arena, p[4], p[5], rhs]);
        let rhs_status = b.inst_results(rhs_call)[0];
        let statuses = b.ins().bor(lhs_status, rhs_status);
        let values_valid =
            b.ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, statuses, 0);
        let op_valid = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            op,
            2,
        );
        let valid = b.ins().band(values_valid, op_valid);
        let prepare = b.create_block();
        let bad = b.create_block();
        b.ins().brif(valid, prepare, &[], bad, &[]);

        b.switch_to_block(prepare);
        let lhs_len = b.ins().load(types::I64, MemFlags::trusted(), lhs, VIEW_LEN);
        let rhs_len = b.ins().load(types::I64, MemFlags::trusted(), rhs, VIEW_LEN);
        let lhs_longer = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            lhs_len,
            rhs_len,
        );
        let max_len = b.ins().select(lhs_longer, lhs_len, rhs_len);
        let add_capacity = b.ins().iadd_imm(max_len, 1);
        let mul_capacity = b.ins().iadd(lhs_len, rhs_len);
        let is_mul = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, op, 2);
        let capacity = b.ins().select(is_mul, mul_capacity, add_capacity);
        let bytes = b.ins().ishl_imm(capacity, 3);
        let allocation = b.ins().call(malloc, &[bytes]);
        let result = b.inst_results(allocation)[0];
        let allocated =
            b.ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, result, 0);
        let zero_loop = b.create_block();
        b.append_block_param(zero_loop, types::I64);
        let initial_zero = b.ins().iconst(types::I64, 0);
        b.ins()
            .brif(allocated, zero_loop, &[initial_zero.into()], bad, &[]);
        b.switch_to_block(zero_loop);
        let zero_index = b.block_params(zero_loop)[0];
        let zero_done = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            zero_index,
            capacity,
        );
        let dispatch = b.create_block();
        let zero_word = b.create_block();
        b.ins().brif(zero_done, dispatch, &[], zero_word, &[]);
        b.switch_to_block(zero_word);
        let zero_offset = b.ins().ishl_imm(zero_index, 3);
        let zero_address = b.ins().iadd(result, zero_offset);
        let zero = b.ins().iconst(types::I64, 0);
        b.ins().store(MemFlags::trusted(), zero, zero_address, 0);
        let next_zero = b.ins().iadd_imm(zero_index, 1);
        b.ins().jump(zero_loop, &[next_zero.into()]);

        let finish = b.create_block();
        b.append_block_param(finish, types::I64);
        b.switch_to_block(dispatch);
        let multiply = b.create_block();
        let add_sub = b.create_block();
        b.ins().brif(is_mul, multiply, &[], add_sub, &[]);

        // Grade-school unsigned limb multiplication.  The sign is handled
        // independently and canonicalization happens in `intern`.
        b.switch_to_block(multiply);
        let lhs_sign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), lhs, VIEW_SIGN);
        let rhs_sign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), rhs, VIEW_SIGN);
        let product_sign = b.ins().bxor(lhs_sign, rhs_sign);
        let lhs_limbs = b.ins().load(ptr, MemFlags::trusted(), lhs, VIEW_LIMBS);
        let rhs_limbs = b.ins().load(ptr, MemFlags::trusted(), rhs, VIEW_LIMBS);
        let mul_outer = b.create_block();
        b.append_block_param(mul_outer, types::I64);
        let mul_done = b.create_block();
        let outer_zero = b.ins().iconst(types::I64, 0);
        b.ins().jump(mul_outer, &[outer_zero.into()]);
        b.switch_to_block(mul_outer);
        let i = b.block_params(mul_outer)[0];
        let outer_done = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, i, lhs_len);
        let mul_inner_start = b.create_block();
        b.ins()
            .brif(outer_done, mul_done, &[], mul_inner_start, &[]);
        b.switch_to_block(mul_inner_start);
        let i_offset = b.ins().ishl_imm(i, 3);
        let lhs_address = b.ins().iadd(lhs_limbs, i_offset);
        let lhs_word = b
            .ins()
            .load(types::I64, MemFlags::trusted(), lhs_address, 0);
        let mul_inner = b.create_block();
        b.append_block_param(mul_inner, types::I64);
        b.append_block_param(mul_inner, types::I64);
        let inner_zero = b.ins().iconst(types::I64, 0);
        let carry_zero = b.ins().iconst(types::I64, 0);
        b.ins()
            .jump(mul_inner, &[inner_zero.into(), carry_zero.into()]);
        b.switch_to_block(mul_inner);
        let j = b.block_params(mul_inner)[0];
        let carry = b.block_params(mul_inner)[1];
        let inner_done = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, j, rhs_len);
        let carry_start = b.create_block();
        b.append_block_param(carry_start, types::I64);
        let mul_word = b.create_block();
        b.ins()
            .brif(inner_done, carry_start, &[carry.into()], mul_word, &[]);
        b.switch_to_block(mul_word);
        let j_offset = b.ins().ishl_imm(j, 3);
        let rhs_address = b.ins().iadd(rhs_limbs, j_offset);
        let rhs_word = b
            .ins()
            .load(types::I64, MemFlags::trusted(), rhs_address, 0);
        let low = b.ins().imul(lhs_word, rhs_word);
        let high = b.ins().umulhi(lhs_word, rhs_word);
        let k = b.ins().iadd(i, j);
        let k_offset = b.ins().ishl_imm(k, 3);
        let result_address = b.ins().iadd(result, k_offset);
        let current = b
            .ins()
            .load(types::I64, MemFlags::trusted(), result_address, 0);
        let with_current = b.ins().iadd(low, current);
        let current_carry = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            with_current,
            low,
        );
        let sum = b.ins().iadd(with_current, carry);
        let input_carry = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            sum,
            with_current,
        );
        b.ins().store(MemFlags::trusted(), sum, result_address, 0);
        let current_carry = b.ins().uextend(types::I64, current_carry);
        let input_carry = b.ins().uextend(types::I64, input_carry);
        let high = b.ins().iadd(high, current_carry);
        let next_carry = b.ins().iadd(high, input_carry);
        let next_j = b.ins().iadd_imm(j, 1);
        b.ins().jump(mul_inner, &[next_j.into(), next_carry.into()]);

        b.switch_to_block(carry_start);
        let initial_carry = b.block_params(carry_start)[0];
        let carry_loop = b.create_block();
        b.append_block_param(carry_loop, types::I64);
        b.append_block_param(carry_loop, types::I64);
        let carry_index = b.ins().iadd(i, rhs_len);
        b.ins()
            .jump(carry_loop, &[carry_index.into(), initial_carry.into()]);
        b.switch_to_block(carry_loop);
        let k = b.block_params(carry_loop)[0];
        let carry = b.block_params(carry_loop)[1];
        let no_carry = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, carry, 0);
        let next_outer = b.create_block();
        let store_carry = b.create_block();
        b.ins().brif(no_carry, next_outer, &[], store_carry, &[]);
        b.switch_to_block(store_carry);
        let in_bounds = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            k,
            capacity,
        );
        let carry_word = b.create_block();
        let free_bad = b.create_block();
        b.ins().brif(in_bounds, carry_word, &[], free_bad, &[]);
        b.switch_to_block(carry_word);
        let offset = b.ins().ishl_imm(k, 3);
        let address = b.ins().iadd(result, offset);
        let current = b.ins().load(types::I64, MemFlags::trusted(), address, 0);
        let sum = b.ins().iadd(current, carry);
        let overflow = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            sum,
            current,
        );
        b.ins().store(MemFlags::trusted(), sum, address, 0);
        let next_carry = b.ins().uextend(types::I64, overflow);
        let next_k = b.ins().iadd_imm(k, 1);
        b.ins()
            .jump(carry_loop, &[next_k.into(), next_carry.into()]);
        b.switch_to_block(next_outer);
        let next_i = b.ins().iadd_imm(i, 1);
        b.ins().jump(mul_outer, &[next_i.into()]);
        b.switch_to_block(mul_done);
        b.ins().jump(finish, &[product_sign.into()]);

        // Addition/subtraction first chooses the effective RHS sign, then
        // performs magnitude add or ordered magnitude subtraction.
        b.switch_to_block(add_sub);
        let lhs_sign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), lhs, VIEW_SIGN);
        let rhs_sign = b
            .ins()
            .load(types::I64, MemFlags::trusted(), rhs, VIEW_SIGN);
        let lhs_limbs = b.ins().load(ptr, MemFlags::trusted(), lhs, VIEW_LIMBS);
        let rhs_limbs = b.ins().load(ptr, MemFlags::trusted(), rhs, VIEW_LIMBS);
        let rhs_first = b.ins().load(types::I64, MemFlags::trusted(), rhs_limbs, 0);
        let rhs_len_one =
            b.ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, rhs_len, 1);
        let rhs_first_zero =
            b.ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, rhs_first, 0);
        let rhs_zero = b.ins().band(rhs_len_one, rhs_first_zero);
        let is_sub = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, op, 1);
        let rhs_nonzero = b.ins().bnot(rhs_zero);
        let flip = b.ins().band(is_sub, rhs_nonzero);
        let flipped_sign = b.ins().bxor_imm(rhs_sign, 1);
        let effective_rhs_sign = b.ins().select(flip, flipped_sign, rhs_sign);
        let same_sign = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            lhs_sign,
            effective_rhs_sign,
        );
        let mag_add = b.create_block();
        let mag_compare = b.create_block();
        b.ins().brif(same_sign, mag_add, &[], mag_compare, &[]);

        b.switch_to_block(mag_add);
        let add_loop = b.create_block();
        b.append_block_param(add_loop, types::I64);
        b.append_block_param(add_loop, types::I64);
        let add_zero = b.ins().iconst(types::I64, 0);
        let carry_zero = b.ins().iconst(types::I64, 0);
        b.ins()
            .jump(add_loop, &[add_zero.into(), carry_zero.into()]);
        b.switch_to_block(add_loop);
        let i = b.block_params(add_loop)[0];
        let carry = b.block_params(add_loop)[1];
        let add_done = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, i, max_len);
        let add_finish = b.create_block();
        b.append_block_param(add_finish, types::I64);
        let add_word = b.create_block();
        b.ins()
            .brif(add_done, add_finish, &[carry.into()], add_word, &[]);
        b.switch_to_block(add_word);
        let lhs_present = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            i,
            lhs_len,
        );
        let rhs_present = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            i,
            rhs_len,
        );
        let off = b.ins().ishl_imm(i, 3);
        let lhs_candidate = b.ins().iadd(lhs_limbs, off);
        let rhs_candidate = b.ins().iadd(rhs_limbs, off);
        let lhs_address = b.ins().select(lhs_present, lhs_candidate, result);
        let rhs_address = b.ins().select(rhs_present, rhs_candidate, result);
        let lhs_loaded = b
            .ins()
            .load(types::I64, MemFlags::trusted(), lhs_address, 0);
        let rhs_loaded = b
            .ins()
            .load(types::I64, MemFlags::trusted(), rhs_address, 0);
        let zero = b.ins().iconst(types::I64, 0);
        let lhs_word = b.ins().select(lhs_present, lhs_loaded, zero);
        let rhs_word = b.ins().select(rhs_present, rhs_loaded, zero);
        let first_sum = b.ins().iadd(lhs_word, rhs_word);
        let first_carry = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            first_sum,
            lhs_word,
        );
        let sum = b.ins().iadd(first_sum, carry);
        let second_carry = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            sum,
            first_sum,
        );
        let first_carry = b.ins().uextend(types::I64, first_carry);
        let second_carry = b.ins().uextend(types::I64, second_carry);
        let next_carry = b.ins().bor(first_carry, second_carry);
        let result_address = b.ins().iadd(result, off);
        b.ins().store(MemFlags::trusted(), sum, result_address, 0);
        let next_i = b.ins().iadd_imm(i, 1);
        b.ins().jump(add_loop, &[next_i.into(), next_carry.into()]);
        b.switch_to_block(add_finish);
        let carry = b.block_params(add_finish)[0];
        let final_off = b.ins().ishl_imm(max_len, 3);
        let final_address = b.ins().iadd(result, final_off);
        b.ins().store(MemFlags::trusted(), carry, final_address, 0);
        b.ins().jump(finish, &[lhs_sign.into()]);

        // Magnitude comparison selects the large and small views.  The
        // selected pointers/lengths/sign are threaded explicitly to subtraction.
        b.switch_to_block(mag_compare);
        let lengths_equal = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            lhs_len,
            rhs_len,
        );
        let compare_words = b.create_block();
        b.append_block_param(compare_words, types::I64);
        let choose_lengths = b.create_block();
        b.ins().brif(
            lengths_equal,
            compare_words,
            &[lhs_len.into()],
            choose_lengths,
            &[],
        );
        let choose = b.create_block();
        b.append_block_param(choose, types::I64);
        b.switch_to_block(choose_lengths);
        let lhs_larger = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            lhs_len,
            rhs_len,
        );
        let lhs_larger = b.ins().uextend(types::I64, lhs_larger);
        b.ins().jump(choose, &[lhs_larger.into()]);
        b.switch_to_block(compare_words);
        let n = b.block_params(compare_words)[0];
        let none = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, n, 0);
        let equal_magnitude = b.create_block();
        let compare_word = b.create_block();
        b.ins().brif(none, equal_magnitude, &[], compare_word, &[]);
        b.switch_to_block(compare_word);
        let i = b.ins().iadd_imm(n, -1);
        let off = b.ins().ishl_imm(i, 3);
        let lhs_address = b.ins().iadd(lhs_limbs, off);
        let rhs_address = b.ins().iadd(rhs_limbs, off);
        let lhs_word = b
            .ins()
            .load(types::I64, MemFlags::trusted(), lhs_address, 0);
        let rhs_word = b
            .ins()
            .load(types::I64, MemFlags::trusted(), rhs_address, 0);
        let same_word = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            lhs_word,
            rhs_word,
        );
        let choose_word = b.create_block();
        b.ins()
            .brif(same_word, compare_words, &[i.into()], choose_word, &[]);
        b.switch_to_block(choose_word);
        let lhs_larger = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            lhs_word,
            rhs_word,
        );
        let lhs_larger = b.ins().uextend(types::I64, lhs_larger);
        b.ins().jump(choose, &[lhs_larger.into()]);
        b.switch_to_block(equal_magnitude);
        let zero = b.ins().iconst(types::I64, 0);
        b.ins().jump(finish, &[zero.into()]);

        b.switch_to_block(choose);
        let lhs_larger = b.block_params(choose)[0];
        let lhs_selected = b.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            lhs_larger,
            1,
        );
        let large_limbs = b.ins().select(lhs_selected, lhs_limbs, rhs_limbs);
        let small_limbs = b.ins().select(lhs_selected, rhs_limbs, lhs_limbs);
        let large_len = b.ins().select(lhs_selected, lhs_len, rhs_len);
        let result_sign = b.ins().select(lhs_selected, lhs_sign, effective_rhs_sign);
        let subtract = b.create_block();
        b.append_block_param(subtract, types::I64);
        b.append_block_param(subtract, types::I64);
        let zero_i = b.ins().iconst(types::I64, 0);
        let zero_borrow = b.ins().iconst(types::I64, 0);
        b.ins().jump(subtract, &[zero_i.into(), zero_borrow.into()]);
        b.switch_to_block(subtract);
        let i = b.block_params(subtract)[0];
        let borrow = b.block_params(subtract)[1];
        let done = b
            .ins()
            .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, i, large_len);
        let subtract_done = b.create_block();
        b.append_block_param(subtract_done, types::I64);
        let subtract_word = b.create_block();
        b.ins()
            .brif(done, subtract_done, &[borrow.into()], subtract_word, &[]);
        b.switch_to_block(subtract_word);
        let off = b.ins().ishl_imm(i, 3);
        let large_address = b.ins().iadd(large_limbs, off);
        let large_word = b
            .ins()
            .load(types::I64, MemFlags::trusted(), large_address, 0);
        let small_len = b.ins().select(lhs_selected, rhs_len, lhs_len);
        let small_present = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            i,
            small_len,
        );
        let small_candidate = b.ins().iadd(small_limbs, off);
        let small_address = b.ins().select(small_present, small_candidate, result);
        let small_loaded = b
            .ins()
            .load(types::I64, MemFlags::trusted(), small_address, 0);
        let zero = b.ins().iconst(types::I64, 0);
        let small_word = b.ins().select(small_present, small_loaded, zero);
        let right = b.ins().iadd(small_word, borrow);
        let borrow_add = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            right,
            small_word,
        );
        let value = b.ins().isub(large_word, right);
        let borrow_sub = b.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            large_word,
            right,
        );
        let borrow_add = b.ins().uextend(types::I64, borrow_add);
        let borrow_sub = b.ins().uextend(types::I64, borrow_sub);
        let next_borrow = b.ins().bor(borrow_add, borrow_sub);
        let address = b.ins().iadd(result, off);
        b.ins().store(MemFlags::trusted(), value, address, 0);
        let next_i = b.ins().iadd_imm(i, 1);
        b.ins().jump(subtract, &[next_i.into(), next_borrow.into()]);
        b.switch_to_block(subtract_done);
        let borrow = b.block_params(subtract_done)[0];
        let borrow_clear =
            b.ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, borrow, 0);
        b.ins()
            .brif(borrow_clear, finish, &[result_sign.into()], free_bad, &[]);

        b.switch_to_block(finish);
        let sign = b.block_params(finish)[0];
        let call = b
            .ins()
            .call(intern, &[arena, sign, result, capacity, output]);
        let status = b.inst_results(call)[0];
        b.ins().call(free, &[result]);
        b.ins().return_(&[status]);
        b.switch_to_block(free_bad);
        b.ins().call(free, &[result]);
        b.ins().jump(bad, &[]);
        b.switch_to_block(bad);
        let error = b.ins().iconst(types::I64, -1);
        b.ins().return_(&[error]);
        b.seal_all_blocks();
        b.finalize();
    }
    finish(module, graph.binop, func)
}
