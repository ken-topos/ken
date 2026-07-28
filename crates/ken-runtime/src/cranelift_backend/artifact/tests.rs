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

/// **`RT-FNSPLIT-B2F` `AC-8`(b) — the native-`Int` module's DECLARED inventory
/// is exactly eight functions, pinned as a SET rather than as a count.**
///
/// ⭐ **`AC-8` records the measurement as *"6 definitions / 8 declarations,
/// Θ(1) per native module"* and says the definitions are already pinned
/// (`LOCAL_HELPER_COUNT`, above) while ⛔ **the declarations are genuinely
/// unpinned.** This is that pin.
///
/// ## Why a set and not the number 8
///
/// ⛔ A count is satisfied by any eight names, so swapping a helper for a
/// different import keeps it green. `pin-a-property` §5: assert the **exact
/// permitted inventory**, so that *any* addition reddens — including one nobody
/// imagined — and so that a removal reddens as the same failure rather than as a
/// different one.
///
/// ⚠ **And the two linkages are asserted separately on purpose.** `malloc` and
/// `free` are `Import`s the host supplies; the six helpers are `Local`
/// definitions this module owns. Collapsing them into one name set would accept
/// a helper silently demoted to an import — which is precisely how a body that
/// stopped being emitted would look from a name census.
///
/// **MEASURED:** the `(name, linkage)` pairs `ModuleDeclarations::get_functions`
/// reports for a module that has had the native-`Int` local graph emitted into
/// it, and nothing else.
/// **CLAIMED:** `emit_native_int_local_graph` declares exactly this inventory.
/// **THE GAP:** ⚠ that the population is **program-independent** is *not*
/// measured here — it is the structural guarantee `AC-8`(c) records, namely that
/// `emit_native_int_local_graph` takes no program-derived parameter, so the
/// compiler forbids that growth mode. ⛔ `AC-8` explicitly says to add no test
/// for it, and this is not one.
///
/// Promise class: **normative compatibility vector** — these are the symbols the
/// native-`Int` ABI publishes into every module, and changing one is a contract
/// decision, not a refactor.
#[test]
fn b2f_ac8_the_native_int_module_declares_exactly_its_eight_functions() {
    let mut module = new_jit_module().expect("JIT module constructs");
    crate::native_int_clif::emit_native_int_local_graph(&mut module, false)
        .expect("local helper graph emits");

    let declared = native_int_declared_inventory(&module);

    assert_eq!(
        declared,
        vec![
            ("free".to_string(), Linkage::Import),
            ("ken_native_int_binop_local".to_string(), Linkage::Local),
            ("ken_native_int_compare_local".to_string(), Linkage::Local),
            ("ken_native_int_export_local".to_string(), Linkage::Local),
            ("ken_native_int_intern_local".to_string(), Linkage::Local),
            ("ken_native_int_narrow_local".to_string(), Linkage::Local),
            ("ken_native_int_resolve_local".to_string(), Linkage::Local),
            ("malloc".to_string(), Linkage::Import),
        ],
        "the native-Int module's declared inventory moved; AC-8's `8 declarations` \
         is a claim about THIS set, not about the number"
    );
}

/// ⭐ **The positive control for the inventory pin — it proves the enumeration
/// OBSERVES declarations rather than returning a constant.**
///
/// ⚠ **A negative check passes for any reason** (`pin-a-property` §6), and an
/// inventory assertion is a negative check: "nothing else is declared" is green
/// on a harness that reports nothing at all. So feed the same enumerator a
/// module that genuinely has more in it — the boundary-value graph emitted
/// beside the native-`Int` one — and require it to **see the difference**.
///
/// ⛔ This is deliberately not a second copy of the expected list: it asserts a
/// **relation** (strict superset, and strictly larger) so it stays true when
/// either emitter legitimately changes its own population. ⚠ It therefore says
/// nothing about *which* extra symbols appeared, which is the sibling emitter's
/// obligation and not this file's.
#[test]
fn b2f_ac8_the_inventory_enumerator_sees_a_second_emitters_declarations() {
    let mut native_only = new_jit_module().expect("JIT module constructs");
    crate::native_int_clif::emit_native_int_local_graph(&mut native_only, false)
        .expect("local helper graph emits");
    let native_inventory = native_int_declared_inventory(&native_only);

    let mut both = new_jit_module().expect("JIT module constructs");
    let graph = crate::native_int_clif::emit_native_int_local_graph(&mut both, false)
        .expect("local helper graph emits");
    let plan = crate::boundary_value::BoundaryEmissionPlan::derive();
    crate::boundary_value_clif::emit_boundary_value_local_graph(&mut both, &graph, &plan)
        .expect("boundary-value graph emits");
    let both_inventory = native_int_declared_inventory(&both);

    // Non-vacuity, stated before the comparison: the instrument reported
    // something at all, and the two runs are not the same run.
    assert!(!native_inventory.is_empty());
    assert!(
        both_inventory.len() > native_inventory.len(),
        "the enumerator did not see the second emitter's declarations, so the \
         inventory pin above is green for an unknown reason"
    );
    for entry in &native_inventory {
        assert!(
            both_inventory.contains(entry),
            "emitting a second graph dropped `{}` from the declared inventory",
            entry.0
        );
    }
}

/// Every function a module declares, as sorted `(name, linkage)` pairs.
///
/// ⛔ Read from `ModuleDeclarations`, which is what the module actually holds —
/// ⛔ **not** from the source text of the emitter, and not from the `FuncId`s
/// the emitter happened to return. An emitter that declared a ninth function and
/// forgot to mention it in its return type is exactly the case this must catch.
fn native_int_declared_inventory<M: Module>(module: &M) -> Vec<(String, Linkage)> {
    let mut declared = module
        .declarations()
        .get_functions()
        .map(|(id, decl)| (decl.linkage_name(id).into_owned(), decl.linkage))
        .collect::<Vec<_>>();
    // ⚠ Sorted by NAME only — `Linkage` is not `Ord`, and sorting on it would
    // make the expected order depend on a foreign enum's declaration order.
    declared.sort_by(|left, right| left.0.cmp(&right.0));
    declared
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
