//! Exact JIT/object/ISA tests (RT-SPLIT §10.2a — "exact JIT/object/ISA tests
//! -> `artifact/tests.rs`").
//!
//! Moved verbatim from the residual facade's `mod tests` in slice 7. `super`
//! is `artifact`, so these reach `new_jit_module`/`new_object_module` as
//! ancestor-privates with zero widening. `verify_cranelift_function` is
//! LOWERING-owned (§10.2, Architect `evt_3tgaw9ws44fqg`), so it arrives
//! through the owner-adjacent adapter slice 5 landed beside its original --
//! the one adapter in this series that points lowering -> artifact rather than
//! artifact -> lowering. Test-only adapter reach is not a production DAG edge
//! in either direction (§10.3), which is why both can coexist without a cycle.

use super::*;

use std::mem;

// Named directly from the cranelift crates rather than through `lowering`'s
// `pub(in crate::cranelift_backend)` re-exports. The facade supplied these to
// the moved bodies through a `use lowering::core::*` GLOB, which hid the edge;
// naming them at their origin keeps this file off `lowering` support entirely
// and so introduces no `artifact -> lowering` production edge (§10.3).
use cranelift_codegen::ir::{
    types, AbiParam, Function, InstBuilder, StackSlotData, StackSlotKind, UserFuncName,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};

// The two lowering-owned items these artifact-subject tests reach. Both are
// test-only adapter reach, which §10.3 states does NOT enter the production
// DAG in either direction -- the same sanctioned crossing as the three
// artifact adapters the lowering tests use, pointing the other way.
// Aliasing preserves the moved bodies' call tokens (AC-3).
use crate::cranelift_backend::lowering::require_i64_for_artifact_tests;
use crate::cranelift_backend::lowering::verify_cranelift_function_for_artifact_tests as verify_cranelift_function;

#[test]
fn px8i_jit_and_object_construct_identical_local_helper_clif() {
    let mut jit = new_jit_module().expect("JIT module constructs");
    let jit_clif = crate::native_int_clif::capture_native_int_local_graph(&mut jit)
        .expect("JIT local helper graph emits");
    let mut object =
        new_object_module("px8i-local-helper-identity").expect("object module constructs");
    let object_clif = crate::native_int_clif::capture_native_int_local_graph(&mut object)
        .expect("object local helper graph emits");
    assert_eq!(jit_clif, object_clif);
    assert!(!jit_clif.is_empty());
    // Rework (Q-RESIDUE, 2026-07-21): the bare `5` was unverified
    // provenance. Grounded against `emit_native_int_local_graph`, which
    // calls exactly six `define_*` helpers (resolve, intern, compare,
    // narrow, export, binop); `capture_native_int_local_graph` joins
    // their captured CLIF bodies with "-- helper --", so N helpers yield
    // N-1 separators. This is a fixed property of the compiler's own
    // small, deliberately-enumerated local-helper set, not an external or
    // growable corpus -- pinning it here catches a helper silently
    // failing to emit a body.
    const LOCAL_HELPER_COUNT: usize = 6;
    assert_eq!(
        jit_clif.matches("-- helper --").count(),
        LOCAL_HELPER_COUNT - 1,
        "expected all {LOCAL_HELPER_COUNT} native-Int local helpers (resolve, intern, compare, narrow, export, binop) to emit a captured CLIF body"
    );
}

#[test]
fn px8i_local_helpers_reject_invalid_zero_stale_and_wrong_arena_slots() {
    let mut module = new_jit_module().expect("JIT module constructs");
    let helpers = crate::native_int_clif::emit_native_int_local_graph(&mut module, false)
        .expect("local helper graph emits");
    let pointer = module.target_config().pointer_type();

    let mut mint_signature = module.make_signature();
    mint_signature.params.push(AbiParam::new(pointer));
    mint_signature.returns.push(AbiParam::new(types::I64));
    let mint_id = module
        .declare_function("px8i_mint_probe", Linkage::Local, &mint_signature)
        .expect("mint probe declares");
    let mut mint_context = module.make_context();
    mint_context.func =
        Function::with_name_signature(UserFuncName::user(2, mint_id.as_u32()), mint_signature);
    let intern = module.declare_func_in_func(helpers.intern, &mut mint_context.func);
    let mut frontend = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut mint_context.func, &mut frontend);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let arena = builder.block_params(entry)[0];
        let limbs =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 3));
        let zero = builder.ins().iconst(types::I64, 0);
        let one = builder.ins().iconst(types::I64, 1);
        builder.ins().stack_store(zero, limbs, 0);
        builder.ins().stack_store(one, limbs, 8);
        let output =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 3));
        let limbs = builder.ins().stack_addr(pointer, limbs, 0);
        let output_pointer = builder.ins().stack_addr(pointer, output, 0);
        let two = builder.ins().iconst(types::I64, 2);
        let call = builder
            .ins()
            .call(intern, &[arena, zero, limbs, two, output_pointer]);
        let status = builder.inst_results(call)[0];
        require_i64_for_artifact_tests(&mut builder, status, 0);
        let slot = builder.ins().stack_load(types::I64, output, 8);
        builder.ins().return_(&[slot]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    verify_cranelift_function(&mint_context.func, module.isa()).expect("mint verifies");
    module
        .define_function(mint_id, &mut mint_context)
        .expect("mint defines");

    let mut check_signature = module.make_signature();
    check_signature.params.push(AbiParam::new(pointer));
    check_signature.params.push(AbiParam::new(types::I64));
    check_signature.params.push(AbiParam::new(types::I64));
    check_signature.returns.push(AbiParam::new(types::I64));
    let check_id = module
        .declare_function("px8i_slot_probe", Linkage::Local, &check_signature)
        .expect("slot probe declares");
    let mut check_context = module.make_context();
    check_context.func =
        Function::with_name_signature(UserFuncName::user(2, check_id.as_u32()), check_signature);
    let compare = module.declare_func_in_func(helpers.compare, &mut check_context.func);
    let mut frontend = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut check_context.func, &mut frontend);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let params = builder.block_params(entry).to_vec();
        let eq = builder.ins().iconst(types::I64, 0);
        let call = builder.ins().call(
            compare,
            &[params[0], eq, params[1], params[2], params[1], params[2]],
        );
        let status = builder.inst_results(call)[0];
        builder.ins().return_(&[status]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    verify_cranelift_function(&check_context.func, module.isa()).expect("check verifies");
    module
        .define_function(check_id, &mut check_context)
        .expect("check defines");
    module
        .finalize_definitions()
        .expect("probe module finalizes");

    let mint = module.get_finalized_function(mint_id);
    let check = module.get_finalized_function(check_id);
    let mint =
        unsafe { mem::transmute::<_, extern "C" fn(*mut crate::NativeIntArenaV1) -> u64>(mint) };
    let check = unsafe {
        mem::transmute::<_, extern "C" fn(*mut crate::NativeIntArenaV1, u64, u64) -> i64>(check)
    };
    let mut first = crate::NativeIntArenaV1::default();
    let mut second = crate::NativeIntArenaV1::default();
    let slot = mint(&mut first);
    assert_ne!(slot, 0);
    assert_eq!(check(&mut first, crate::NATIVE_INT_BIG_TAG_V1, slot), 1);
    assert_eq!(check(&mut first, crate::NATIVE_INT_BIG_TAG_V1, 0), -1);
    assert_eq!(check(&mut second, crate::NATIVE_INT_BIG_TAG_V1, slot), -1);
    assert_eq!(check(&mut first, 9, slot), -1);
}
