//! The compiled container: `CompiledModule`, its JIT specialization
//! `CompiledExpr`, the result decoder, result-table ownership, and JIT
//! execution with result decoding.
//!
//! RT-SPLIT slice 3 of 7. Pure move out of the flat `cranelift_backend`
//! module. This module does NOT own compilation policy -- it owns the
//! artifact of compilation and how its result is read back. Depends only on
//! `surface`.

use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use cranelift_jit::JITModule;
use cranelift_module::FuncId;

use super::surface::{backend, backend_module, BackendFailure, CraneliftBackendError};
use crate::{RuntimeGroundValue, RuntimeObservation, RuntimeTrap};

pub(super) struct CompiledModule<M> {
    pub(super) module: M,
    func_id: FuncId,
    decoder: Option<ResultDecoder>,
    result_table: BTreeMap<i64, RuntimeGroundValue>,
    trap: Option<RuntimeTrap>,
    pub(super) verifier_passed: bool,
    pub(super) assumptions: BTreeSet<String>,
    pub(super) unsupported: Vec<String>,
}

pub(super) type CompiledExpr = CompiledModule<JITModule>;

#[derive(Clone, Copy)]
pub(super) enum ResultDecoder {
    Int,
    ProcessStatus,
    Bool,
    Boundary,
    Table,
}

impl<M> CompiledModule<M> {
    /// Transparent one-to-one packing seam (RT-SPLIT §10.4a). Exists so the
    /// four construction-only fields (`func_id`, `decoder`, `result_table`,
    /// `trap`) can stay private to this module while the three existing
    /// construction sites live outside it. No validation, no defaults, no
    /// clones, no reordering, no policy -- adding any would make this a
    /// behavior change rather than wiring.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_parts(
        module: M,
        func_id: FuncId,
        decoder: Option<ResultDecoder>,
        result_table: BTreeMap<i64, RuntimeGroundValue>,
        trap: Option<RuntimeTrap>,
        verifier_passed: bool,
        assumptions: BTreeSet<String>,
        unsupported: Vec<String>,
    ) -> Self {
        Self {
            module,
            func_id,
            decoder,
            result_table,
            trap,
            verifier_passed,
            assumptions,
            unsupported,
        }
    }
}

impl CompiledModule<JITModule> {
    pub(super) fn run(
        mut self,
        process_root: Option<*const std::ffi::c_void>,
    ) -> Result<(RuntimeObservation, Option<i64>), CraneliftBackendError> {
        if let Some(trap) = self.trap {
            return Ok((RuntimeObservation::Trapped(trap), None));
        }

        self.module
            .finalize_definitions()
            .map_err(|err| backend_module(err.to_string()))?;
        let code = self.module.get_finalized_function(self.func_id);
        // Named native-code-execution boundary. This is tested/validated JIT
        // execution, never a proof and never a host-ABI syscall boundary.
        let mut store = crate::boundary_value::BoundaryValueStore::default();
        let binding = crate::boundary_activation::BoundaryStoreBindingV1::open(
            &mut store,
            crate::boundary_resource_profile::starter_smoke_profile(),
        );
        let activation = crate::boundary_activation::BoundaryActivationV1::begin(&binding);
        let process_root = process_root
            .or_else(|| activation.native_frame_ptr())
            .ok_or_else(|| {
                backend_module("activation did not publish a launch pointer".to_string())
            })?;
        let services = activation
            .services_ptr()
            .ok_or_else(|| {
                backend_module("activation did not publish its services".to_string())
            })?;
        let native = unsafe {
            mem::transmute::<
                _,
                extern "C" fn(
                    *const std::ffi::c_void,
                    *const std::ffi::c_void,
                ) -> i64,
            >(code)
        };
        let token = native(process_root, services);
        let decoder = self
            .decoder
            .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?;
        let ground = match decoder {
            ResultDecoder::Int => RuntimeGroundValue::Int(
                activation
                    .native_int_arena()
                    .decode_final_export()
                    .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?,
            ),
            ResultDecoder::ProcessStatus => RuntimeGroundValue::Int(token.into()),
            ResultDecoder::Bool => RuntimeGroundValue::Bool(token != 0),
            ResultDecoder::Boundary => match crate::boundary_value::BoundaryWord(token as u64)
                .tag()
            {
                Some(crate::boundary_value::BoundaryTag::ImmediateBool) => {
                    RuntimeGroundValue::Bool(
                        crate::boundary_value::BoundaryWord(token as u64).payload() != 0,
                    )
                }
                Some(crate::boundary_value::BoundaryTag::ImmediateInt) => {
                    RuntimeGroundValue::Int(
                        crate::boundary_value::BoundaryWord(token as u64)
                            .signed_payload()
                            .into(),
                    )
                }
                _ => {
                    return Err(backend(BackendFailure::NativeResultDecode { token }));
                }
            },
            ResultDecoder::Table => self
                .result_table
                .get(&token)
                .cloned()
                .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?,
        };
        Ok((RuntimeObservation::Returned(ground), Some(token)))
    }
}
