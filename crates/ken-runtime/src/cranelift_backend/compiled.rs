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
        let mut native_int_arena = crate::NativeIntArenaV1::default();
        let process_root = process_root
            .unwrap_or_else(|| (&mut native_int_arena as *mut crate::NativeIntArenaV1).cast());
        let native =
            unsafe { mem::transmute::<_, extern "C" fn(*const std::ffi::c_void) -> i64>(code) };
        let token = native(process_root);
        let decoder = self
            .decoder
            .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?;
        let ground = match decoder {
            ResultDecoder::Int => RuntimeGroundValue::Int(
                native_int_arena
                    .decode_final_export()
                    .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?,
            ),
            ResultDecoder::ProcessStatus => RuntimeGroundValue::Int(token.into()),
            ResultDecoder::Bool => RuntimeGroundValue::Bool(token != 0),
            ResultDecoder::Table => self
                .result_table
                .get(&token)
                .cloned()
                .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?,
        };
        Ok((RuntimeObservation::Returned(ground), Some(token)))
    }
}
