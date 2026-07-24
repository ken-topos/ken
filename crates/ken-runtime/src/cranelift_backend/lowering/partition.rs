//! Private frame codec for synchronous native process tail partitions.
//!
//! A partition descriptor may retain immutable runtime syntax, but never a
//! Cranelift value from its caller. Every dynamic leaf is written to the
//! caller-owned frame and loaded as a fresh value in the helper.

use super::*;

use cranelift_codegen::ir::{Type, Value};
use cranelift_module::FuncId;
use std::fmt::Write as _;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub(super) const PARTITION_FRAME_FIELD_BYTES: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PartitionStaticFingerprint {
    hash: u64,
    bytes: u64,
}

/// Exact, marker-aware identity for immutable lowering state.
///
/// The dense ID is assigned only after a collision-safe comparison of the
/// canonical bytes. State keys therefore compare a fixed-size exact identity
/// instead of retaining and re-walking a complete residual byte string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PartitionStaticDescriptor {
    id: u32,
}

#[derive(Default)]
struct PartitionStaticDescriptorInterner {
    by_bucket: BTreeMap<(u64, u64), Vec<u32>>,
    canonical: Vec<Arc<[u8]>>,
    bytes_constructed: usize,
    bytes_retained: usize,
    bucket_probes: usize,
    exact_comparisons: usize,
    exact_bytes_compared_upper_bound: usize,
}

impl PartitionStaticDescriptorInterner {
    fn intern(
        &mut self,
        fingerprint: PartitionStaticFingerprint,
        canonical: Vec<u8>,
    ) -> PartitionStaticDescriptor {
        self.bytes_constructed = self.bytes_constructed.saturating_add(canonical.len());
        let bucket = (fingerprint.hash, fingerprint.bytes);
        if let Some(candidates) = self.by_bucket.get(&bucket) {
            self.bucket_probes = self.bucket_probes.saturating_add(candidates.len());
            for candidate in candidates.iter().copied() {
                self.exact_comparisons = self.exact_comparisons.saturating_add(1);
                let retained = &self.canonical[candidate as usize];
                self.exact_bytes_compared_upper_bound = self
                    .exact_bytes_compared_upper_bound
                    .saturating_add(retained.len().max(canonical.len()));
                if retained.as_ref() == canonical.as_slice() {
                    return PartitionStaticDescriptor { id: candidate };
                }
            }
        }
        let id = u32::try_from(self.canonical.len())
            .expect("compiler-private partition descriptor identity exhausted");
        self.bytes_retained = self.bytes_retained.saturating_add(canonical.len());
        self.canonical.push(canonical.into());
        self.by_bucket.entry(bucket).or_default().push(id);
        PartitionStaticDescriptor { id }
    }
}

thread_local! {
    static PARTITION_STATIC_DESCRIPTORS:
        std::cell::RefCell<PartitionStaticDescriptorInterner> =
            std::cell::RefCell::new(PartitionStaticDescriptorInterner::default());
}

pub(super) fn reset_partition_static_descriptors() {
    PARTITION_STATIC_DESCRIPTORS.with(|interner| {
        *interner.borrow_mut() = PartitionStaticDescriptorInterner::default();
    });
    PARTITION_LOWERED_KEYS.with(|interner| {
        *interner.borrow_mut() = PartitionLoweredKeyInterner::default();
    });
}

pub(super) fn partition_static_descriptor_counts() -> (usize, usize, usize, usize, usize) {
    PARTITION_STATIC_DESCRIPTORS.with(|interner| {
        let interner = interner.borrow();
        (
            interner.bytes_constructed,
            interner.bytes_retained,
            interner.bucket_probes,
            interner.exact_comparisons,
            interner.exact_bytes_compared_upper_bound,
        )
    })
}

struct PartitionFingerprintWriter {
    hash: u64,
    bytes: u64,
    canonical: Vec<u8>,
}

struct PartitionBucketWriter {
    hash: u64,
    bytes: u64,
}

impl std::fmt::Write for PartitionBucketWriter {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        for byte in value.as_bytes() {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01b3);
            self.bytes = self.bytes.wrapping_add(1);
        }
        Ok(())
    }
}

struct PartitionFingerprintHasher(PartitionFingerprintWriter);

impl Hasher for PartitionFingerprintHasher {
    fn finish(&self) -> u64 {
        self.0.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.canonical.extend_from_slice(bytes);
        for byte in bytes {
            self.0.hash ^= u64::from(*byte);
            self.0.hash = self.0.hash.wrapping_mul(0x0000_0100_0000_01b3);
            self.0.bytes = self.0.bytes.wrapping_add(1);
        }
    }
}

impl std::fmt::Write for PartitionFingerprintWriter {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.canonical.extend_from_slice(value.as_bytes());
        for byte in value.as_bytes() {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01b3);
            self.bytes = self.bytes.wrapping_add(1);
        }
        Ok(())
    }
}

fn partition_static_bucket(value: &impl std::fmt::Debug) -> PartitionStaticFingerprint {
    let mut writer = PartitionBucketWriter {
        hash: 0xcbf2_9ce4_8422_2325,
        bytes: 0,
    };
    write!(&mut writer, "{value:?}").expect("fingerprint writer cannot fail");
    PartitionStaticFingerprint {
        hash: writer.hash,
        bytes: writer.bytes,
    }
}

fn partition_runtime_expr_descriptor(expr: &RuntimeExpr) -> PartitionStaticDescriptor {
    let mut hasher = PartitionFingerprintHasher(PartitionFingerprintWriter {
        hash: 0xcbf2_9ce4_8422_2325,
        bytes: 0,
        canonical: Vec::new(),
    });
    partition_hash_runtime_expr(expr, &mut hasher);
    let fingerprint = PartitionStaticFingerprint {
        hash: hasher.0.hash,
        bytes: hasher.0.bytes,
    };
    PARTITION_STATIC_DESCRIPTORS.with(|interner| {
        interner
            .borrow_mut()
            .intern(fingerprint, hasher.0.canonical)
    })
}

fn partition_hash_runtime_expr(expr: &RuntimeExpr, hasher: &mut impl Hasher) {
    match expr {
        RuntimeExpr::CheckedJoinSite { site_id, .. } => {
            0_u8.hash(hasher);
            site_id.hash(hasher);
        }
        RuntimeExpr::CheckedSubcontinuationFrame { frame_id, .. } => {
            1_u8.hash(hasher);
            frame_id.hash(hasher);
        }
        RuntimeExpr::CheckedRecursiveInvocation {
            call_template_id,
            checked_occurrence_path,
            ..
        } => {
            2_u8.hash(hasher);
            call_template_id.hash(hasher);
            checked_occurrence_path.hash(hasher);
        }
        RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids,
            checked_occurrence_paths,
            ..
        } => {
            3_u8.hash(hasher);
            slot_template_ids.hash(hasher);
            checked_occurrence_paths.hash(hasher);
        }
        RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id,
            checked_occurrence_path,
            ..
        } => {
            4_u8.hash(hasher);
            call_template_id.hash(hasher);
            checked_occurrence_path.hash(hasher);
        }
        RuntimeExpr::Value(value) => {
            5_u8.hash(hasher);
            value.hash(hasher);
        }
        RuntimeExpr::Var(index) => {
            6_u8.hash(hasher);
            index.hash(hasher);
        }
        RuntimeExpr::Let { value, body } => {
            7_u8.hash(hasher);
            partition_hash_runtime_expr(value, hasher);
            partition_hash_runtime_expr(body, hasher);
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            8_u8.hash(hasher);
            partition_hash_runtime_expr(scrutinee, hasher);
            partition_hash_runtime_expr(then_expr, hasher);
            partition_hash_runtime_expr(else_expr, hasher);
        }
        RuntimeExpr::PrimitiveCall { primitive, args } => {
            9_u8.hash(hasher);
            primitive.hash(hasher);
            partition_hash_runtime_exprs(args, hasher);
        }
        RuntimeExpr::Construct { constructor, args } => {
            10_u8.hash(hasher);
            constructor.hash(hasher);
            partition_hash_runtime_exprs(args, hasher);
        }
        RuntimeExpr::Match {
            scrutinee,
            cases,
            default,
        } => {
            11_u8.hash(hasher);
            partition_hash_runtime_expr(scrutinee, hasher);
            partition_hash_match_cases(cases, hasher);
            default.hash(hasher);
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee,
            cases,
            default,
        } => {
            12_u8.hash(hasher);
            partition_hash_runtime_expr(scrutinee, hasher);
            partition_hash_computational_cases(cases, hasher);
            default.hash(hasher);
        }
        RuntimeExpr::Record { fields } => {
            13_u8.hash(hasher);
            fields.len().hash(hasher);
            for (name, value) in fields {
                name.hash(hasher);
                partition_hash_runtime_expr(value, hasher);
            }
        }
        RuntimeExpr::Project { record, field } => {
            14_u8.hash(hasher);
            partition_hash_runtime_expr(record, hasher);
            field.hash(hasher);
        }
        RuntimeExpr::Closure {
            captures,
            params,
            body,
        } => {
            15_u8.hash(hasher);
            captures.hash(hasher);
            params.hash(hasher);
            partition_hash_runtime_expr(body, hasher);
        }
        RuntimeExpr::LexicalClosure {
            captures,
            params,
            body,
        } => {
            16_u8.hash(hasher);
            partition_hash_runtime_exprs(captures, hasher);
            params.hash(hasher);
            partition_hash_runtime_expr(body, hasher);
        }
        RuntimeExpr::DeclarationRef { symbol } => {
            17_u8.hash(hasher);
            symbol.hash(hasher);
        }
        RuntimeExpr::ImportedDeclarationRef {
            symbol,
            dependency,
            dependency_semantic_hash,
        } => {
            18_u8.hash(hasher);
            symbol.hash(hasher);
            dependency.hash(hasher);
            dependency_semantic_hash.hash(hasher);
        }
        RuntimeExpr::Call { callee, args } => {
            19_u8.hash(hasher);
            partition_hash_runtime_expr(callee, hasher);
            partition_hash_runtime_exprs(args, hasher);
        }
        RuntimeExpr::Effect {
            family,
            operation,
            capability,
            args,
        } => {
            20_u8.hash(hasher);
            family.hash(hasher);
            operation.hash(hasher);
            capability.is_some().hash(hasher);
            if let Some(capability) = capability {
                capability.identity.hash(hasher);
                partition_hash_runtime_expr(&capability.value, hasher);
            }
            partition_hash_runtime_exprs(args, hasher);
        }
        RuntimeExpr::Trap(trap) => {
            21_u8.hash(hasher);
            trap.hash(hasher);
        }
    }
}

fn partition_hash_runtime_exprs(values: &[RuntimeExpr], hasher: &mut impl Hasher) {
    values.len().hash(hasher);
    for value in values {
        partition_hash_runtime_expr(value, hasher);
    }
}

fn partition_runtime_exprs_descriptor(values: &[RuntimeExpr]) -> PartitionStaticDescriptor {
    let mut hasher = PartitionFingerprintHasher(PartitionFingerprintWriter {
        hash: 0xcbf2_9ce4_8422_2325,
        bytes: 0,
        canonical: Vec::new(),
    });
    partition_hash_runtime_exprs(values, &mut hasher);
    let fingerprint = PartitionStaticFingerprint {
        hash: hasher.0.hash,
        bytes: hasher.0.bytes,
    };
    PARTITION_STATIC_DESCRIPTORS.with(|interner| {
        interner
            .borrow_mut()
            .intern(fingerprint, hasher.0.canonical)
    })
}

fn partition_runtime_expr_pair_descriptor(
    first: &RuntimeExpr,
    second: &RuntimeExpr,
) -> PartitionStaticDescriptor {
    let mut hasher = PartitionFingerprintHasher(PartitionFingerprintWriter {
        hash: 0xcbf2_9ce4_8422_2325,
        bytes: 0,
        canonical: Vec::new(),
    });
    partition_hash_runtime_expr(first, &mut hasher);
    partition_hash_runtime_expr(second, &mut hasher);
    let fingerprint = PartitionStaticFingerprint {
        hash: hasher.0.hash,
        bytes: hasher.0.bytes,
    };
    PARTITION_STATIC_DESCRIPTORS.with(|interner| {
        interner
            .borrow_mut()
            .intern(fingerprint, hasher.0.canonical)
    })
}

fn partition_hash_match_cases(cases: &[crate::RuntimeMatchCase], hasher: &mut impl Hasher) {
    cases.len().hash(hasher);
    for case in cases {
        case.constructor.hash(hasher);
        case.binders.hash(hasher);
        partition_hash_runtime_expr(&case.body, hasher);
    }
}

fn partition_hash_computational_cases(
    cases: &[crate::RuntimeComputationalMatchCase],
    hasher: &mut impl Hasher,
) {
    cases.len().hash(hasher);
    for case in cases {
        case.constructor.hash(hasher);
        case.argument_binders.hash(hasher);
        case.recursive_positions.hash(hasher);
        partition_hash_runtime_expr(&case.body, hasher);
    }
}

fn partition_match_descriptor(
    cases: &[crate::RuntimeMatchCase],
    default: &RuntimeTrap,
) -> PartitionStaticDescriptor {
    let mut hasher = PartitionFingerprintHasher(PartitionFingerprintWriter {
        hash: 0xcbf2_9ce4_8422_2325,
        bytes: 0,
        canonical: Vec::new(),
    });
    partition_hash_match_cases(cases, &mut hasher);
    default.hash(&mut hasher);
    let fingerprint = PartitionStaticFingerprint {
        hash: hasher.0.hash,
        bytes: hasher.0.bytes,
    };
    PARTITION_STATIC_DESCRIPTORS.with(|interner| {
        interner
            .borrow_mut()
            .intern(fingerprint, hasher.0.canonical)
    })
}

fn partition_computational_match_descriptor(
    cases: &[crate::RuntimeComputationalMatchCase],
    default: &RuntimeTrap,
) -> PartitionStaticDescriptor {
    let mut hasher = PartitionFingerprintHasher(PartitionFingerprintWriter {
        hash: 0xcbf2_9ce4_8422_2325,
        bytes: 0,
        canonical: Vec::new(),
    });
    partition_hash_computational_cases(cases, &mut hasher);
    default.hash(&mut hasher);
    let fingerprint = PartitionStaticFingerprint {
        hash: hasher.0.hash,
        bytes: hasher.0.bytes,
    };
    PARTITION_STATIC_DESCRIPTORS.with(|interner| {
        interner
            .borrow_mut()
            .intern(fingerprint, hasher.0.canonical)
    })
}

/// IDs kept ahead of the coordinator's next assignment.
///
/// Lowering can encounter several sequential host-result fanouts in one
/// function before returning to the coordinator. Imports remain lazy, so this
/// reserve costs declarations only; the coordinator replenishes it between
/// functions and does not impose a total helper-count ceiling.
pub(super) const PARTITION_HELPER_ID_RESERVE: usize = 16;

/// Static descriptor bytes above which a process entry is armed for its first
/// admissible checked partition seam. Helpers do not use this heuristic:
/// after entry outlining they cut pre-emptively at every planned predecessor.
///
/// This is a timing guard, not a semantic identity or aggregate-growth bound.
/// Exact state interning and the function/aggregate budgets remain the bounds.
const PARTITION_ENTRY_STATIC_BYTES: usize = 4_096;

pub(super) fn partition_source_static_bytes<'a>(
    expr: &RuntimeExpr,
    declarations: impl Iterator<Item = &'a RuntimeDeclaration>,
) -> usize {
    let mut bytes = partition_static_bucket(expr).bytes as usize;
    for declaration in declarations {
        bytes = bytes.saturating_add(partition_static_bucket(declaration).bytes as usize);
    }
    bytes
}

pub(super) fn partition_entry_cut_should_arm(static_bytes: usize) -> bool {
    static_bytes >= PARTITION_ENTRY_STATIC_BYTES
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PartitionFunctionMeasure {
    pub(super) values: usize,
    pub(super) instructions: usize,
    pub(super) blocks: usize,
}

#[derive(Clone, Debug, Default)]
pub(super) struct PartitionCompilationMetrics {
    pub(super) frame_fields_total: usize,
    pub(super) frame_fields_max: usize,
    pub(super) frame_stores: usize,
    pub(super) frame_loads: usize,
    pub(super) cleanup_states: usize,
    pub(super) cleanup_edges: usize,
    pub(super) cleanup_frame_fields_total: usize,
    pub(super) cleanup_frame_fields_max: usize,
    pub(super) cleanup_frame_stores: usize,
    pub(super) cleanup_frame_loads: usize,
    pub(super) source_env_fields_total: usize,
    pub(super) source_env_fields_max: usize,
    pub(super) source_prefix_fields_total: usize,
    pub(super) source_prefix_fields_max: usize,
    pub(super) source_scope_fields_total: usize,
    pub(super) source_scope_fields_max: usize,
    pub(super) source_lineage_fields_total: usize,
    pub(super) source_lineage_fields_max: usize,
}

impl PartitionCompilationMetrics {
    pub(super) fn record_call_frame(&mut self, fields: usize) {
        self.frame_fields_total = self.frame_fields_total.saturating_add(fields);
        self.frame_fields_max = self.frame_fields_max.max(fields);
        self.frame_stores = self.frame_stores.saturating_add(fields);
    }

    pub(super) fn record_helper_frame_loads(&mut self, fields: usize) {
        self.frame_loads = self.frame_loads.saturating_add(fields);
    }

    pub(super) fn record_cleanup_call_frame(&mut self, fields: usize) {
        self.cleanup_edges = self.cleanup_edges.saturating_add(1);
        self.cleanup_frame_fields_total = self.cleanup_frame_fields_total.saturating_add(fields);
        self.cleanup_frame_fields_max = self.cleanup_frame_fields_max.max(fields);
        self.cleanup_frame_stores = self.cleanup_frame_stores.saturating_add(fields);
        self.record_call_frame(fields);
    }

    pub(super) fn record_cleanup_helper_frame_loads(&mut self, fields: usize) {
        self.cleanup_frame_loads = self.cleanup_frame_loads.saturating_add(fields);
        self.record_helper_frame_loads(fields);
    }

    pub(super) fn record_source_frame_components(
        &mut self,
        env: usize,
        prefix: usize,
        scope: usize,
        lineage: usize,
    ) {
        self.source_env_fields_total = self.source_env_fields_total.saturating_add(env);
        self.source_env_fields_max = self.source_env_fields_max.max(env);
        self.source_prefix_fields_total = self.source_prefix_fields_total.saturating_add(prefix);
        self.source_prefix_fields_max = self.source_prefix_fields_max.max(prefix);
        self.source_scope_fields_total = self.source_scope_fields_total.saturating_add(scope);
        self.source_scope_fields_max = self.source_scope_fields_max.max(scope);
        self.source_lineage_fields_total = self.source_lineage_fields_total.saturating_add(lineage);
        self.source_lineage_fields_max = self.source_lineage_fields_max.max(lineage);
    }
}

impl PartitionFunctionMeasure {
    pub(super) fn from_function(function: &Function) -> Self {
        Self {
            values: function.dfg.num_values(),
            instructions: function
                .layout
                .blocks()
                .map(|block| function.layout.block_insts(block).count())
                .sum(),
            blocks: function.layout.blocks().count(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PartitionBudget {
    pub(super) max_values: usize,
    pub(super) max_instructions: usize,
    pub(super) max_blocks: usize,
}

impl PartitionBudget {
    pub(super) const PRODUCTION: Self = Self {
        max_values: 20_000,
        max_instructions: 35_000,
        max_blocks: 12_000,
    };

    pub(super) fn check(
        self,
        measure: PartitionFunctionMeasure,
    ) -> Result<(), CraneliftBackendError> {
        if measure.values > self.max_values
            || measure.instructions > self.max_instructions
            || measure.blocks > self.max_blocks
        {
            return Err(unsupported(
                "NativeFunctionPartition",
                format!(
                    "indivisible lowering quantum exceeds the native function budget: \
                     actual values/instructions/blocks = {}/{}/{}, limits = {}/{}/{}",
                    measure.values,
                    measure.instructions,
                    measure.blocks,
                    self.max_values,
                    self.max_instructions,
                    self.max_blocks,
                ),
            ));
        }
        Ok(())
    }

    pub(super) fn should_partition(self, measure: PartitionFunctionMeasure) -> bool {
        #[cfg(test)]
        if PARTITION_TEST_FORCE_ARM.with(std::cell::Cell::get) {
            return true;
        }
        measure.values >= self.max_values.saturating_mul(2) / 3
            || measure.instructions >= self.max_instructions.saturating_mul(2) / 3
            || measure.blocks >= self.max_blocks.saturating_mul(2) / 3
    }
}

#[cfg(test)]
thread_local! {
    pub(super) static PARTITION_TEST_BUDGET:
        std::cell::Cell<Option<PartitionBudget>> = const { std::cell::Cell::new(None) };
    pub(super) static PARTITION_TEST_FORCE_ARM:
        std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(super) static PARTITION_TEST_MEASURES:
        std::cell::RefCell<Vec<PartitionFunctionMeasure>> =
            const { std::cell::RefCell::new(Vec::new()) };
}

pub(super) fn active_partition_budget() -> PartitionBudget {
    #[cfg(test)]
    {
        return PARTITION_TEST_BUDGET
            .with(std::cell::Cell::get)
            .unwrap_or(PartitionBudget::PRODUCTION);
    }
    #[cfg(not(test))]
    {
        PartitionBudget::PRODUCTION
    }
}

pub(super) fn record_partition_measure(measure: PartitionFunctionMeasure) {
    #[cfg(test)]
    PARTITION_TEST_MEASURES.with(|measures| measures.borrow_mut().push(measure));
    #[cfg(not(test))]
    let _ = measure;
}

pub(super) enum PartitionWorkItem {
    SourceArm(SourceArmPartitionWorkItem),
    SourceKont(SourceKontPartitionWorkItem),
    ProducerKont(ProducerKontPartitionWorkItem),
    Arm(ArmPartitionWorkItem),
    CleanupStep(CleanupStepPartitionWorkItem),
}

#[derive(Clone, Copy)]
pub(super) struct PartitionProducerKontCursor {
    pub(super) site_id: usize,
    pub(super) capture_pointer: Value,
}

#[derive(Clone)]
pub(super) struct PartitionProducerKontSitePlan {
    pub(super) action: ProducerKontAction,
    pub(super) successor: Option<PartitionProducerKontCursor>,
    pub(super) checked_join: PartitionCheckedJoinIdentity,
    pub(super) return_kind: ScalarMergeKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PartitionStateLifecycle {
    Reserved,
    Emitting,
    Defined,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartitionCheckedJoinIdentity {
    site_id: u64,
    declaration: String,
    checked_occurrence_path: Vec<u64>,
    checked_result_type_fingerprint: u64,
    occurrence_binding_fingerprint: u64,
    runtime_frame_fingerprint: u64,
    answer_kind: crate::NativeJoinAnswerKindV1,
}

impl From<&crate::NativeJoinPlanSiteV1> for PartitionCheckedJoinIdentity {
    fn from(site: &crate::NativeJoinPlanSiteV1) -> Self {
        Self {
            site_id: site.site_id,
            declaration: site.declaration.clone(),
            checked_occurrence_path: site.checked_occurrence_path.clone(),
            checked_result_type_fingerprint: site.checked_result_type_fingerprint,
            occurrence_binding_fingerprint: site.occurrence_binding_fingerprint,
            runtime_frame_fingerprint: site.runtime_frame_fingerprint,
            answer_kind: site.answer_kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartitionInvocationTemplateKey {
    SameSccCall(u64),
    ComputationalIHCall(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartitionCheckedParentKey {
    Selection,
    Unwind(PartitionRecursorNodeId),
}

fn partition_checked_parent_key(
    parent: Option<PartitionCheckedParent>,
) -> Option<PartitionCheckedParentKey> {
    parent.map(|parent| match parent {
        PartitionCheckedParent::Selection => PartitionCheckedParentKey::Selection,
        PartitionCheckedParent::Unwind(cursor) => PartitionCheckedParentKey::Unwind(cursor.node),
    })
}

impl From<InvocationTemplateRef> for PartitionInvocationTemplateKey {
    fn from(value: InvocationTemplateRef) -> Self {
        match value {
            InvocationTemplateRef::SameSccCall(id) => Self::SameSccCall(id),
            InvocationTemplateRef::ComputationalIHCall(id) => Self::ComputationalIHCall(id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartitionLayerRoleKey {
    SelectsOccurrence {
        origin: RecursorProducerOriginId,
    },
    ExitsScope {
        origin: RecursorProducerOriginId,
        scope_origin: RecursorProducerOriginId,
        parent_scope: Option<RecursorProducerOriginId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionLayerKey {
    eliminator_descriptor: PartitionStaticDescriptor,
    outer_env: Vec<PartitionLoweredKey>,
    role: PartitionLayerRoleKey,
    provenance: RecursorFrameProvenance,
    checked_frame_id: Option<u64>,
    checked_invocation_id: Option<u64>,
    checked_invocation_source: Option<PartitionInvocationTemplateKey>,
    checked_invocation_depth: usize,
    semantic_pending: bool,
}

fn partition_layer_key(layer: &ComputationalRecursorLayer) -> PartitionLayerKey {
    PartitionLayerKey {
        eliminator_descriptor: partition_computational_match_descriptor(
            &layer.cases,
            &layer.default,
        ),
        outer_env: layer.outer_env.iter().map(partition_lowered_key).collect(),
        role: match layer.role {
            RecursorLayerRole::SelectsOccurrence { origin, .. } => {
                PartitionLayerRoleKey::SelectsOccurrence { origin }
            }
            RecursorLayerRole::ExitsScope {
                origin,
                scope_origin,
                parent_scope,
                ..
            } => PartitionLayerRoleKey::ExitsScope {
                origin,
                scope_origin,
                parent_scope,
            },
        },
        provenance: layer.provenance,
        checked_frame_id: layer.checked_frame_id,
        checked_invocation_id: layer.checked_invocation_id,
        checked_invocation_source: layer.checked_invocation_source.map(Into::into),
        checked_invocation_depth: layer.checked_invocation_depth,
        semantic_pending: layer.semantic_pending,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PartitionLoweredKey(u32);

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartitionLoweredShape {
    Int,
    Bool,
    ProcessExitStatus,
    CapabilityToken,
    ResourceToken,
    BoundedNat,
    StructuralNat,
    ResponseBytes,
    HostResult {
        error: PartitionLoweredKey,
        ok: PartitionLoweredKey,
        err_constructor: String,
        ok_constructor: String,
    },
    DynamicConstructor(Vec<(i64, RuntimeSymbol, Vec<PartitionLoweredKey>)>),
    Bytes(Vec<u8>),
    BorrowedNativeValue,
    BorrowedOption {
        none: String,
        some: String,
    },
    String(String),
    Constructor {
        constructor: String,
        args: Vec<PartitionLoweredKey>,
    },
    Record(Vec<(String, PartitionLoweredKey)>),
    Closure {
        captures: Vec<PartitionLoweredKey>,
        params: Vec<String>,
        body_descriptor: PartitionStaticDescriptor,
    },
    DeclarationClosure {
        symbol: RuntimeSymbol,
        captures: Vec<PartitionLoweredKey>,
        params: Vec<String>,
        body_descriptor: PartitionStaticDescriptor,
    },
    ComputationalRecursorClosure {
        residual: PartitionLoweredKey,
        sibling_position: usize,
        selection: PartitionLayerKey,
        unwind_head: Option<PartitionRecursorNodeId>,
        qualification_head: Option<PartitionRecursorQualificationNodeId>,
        open_obligation_head: Option<PartitionOpenControlObligationNodeId>,
        checked_parent: Option<PartitionCheckedParentKey>,
        computational_ih_slot_template_id: Option<u64>,
        checked_invocation_source: Option<PartitionInvocationTemplateKey>,
    },
    Trap(RuntimeTrap),
    RecursiveBackedge,
}

#[derive(Default)]
struct PartitionLoweredKeyInterner {
    by_bucket: BTreeMap<(u64, u64), Vec<u32>>,
    nodes: Vec<PartitionLoweredShape>,
}

impl PartitionLoweredKeyInterner {
    fn intern(&mut self, shape: PartitionLoweredShape) -> PartitionLoweredKey {
        let bucket = partition_static_bucket(&shape);
        let bucket = (bucket.hash, bucket.bytes);
        if let Some(candidates) = self.by_bucket.get(&bucket) {
            for candidate in candidates.iter().copied() {
                if self.nodes[candidate as usize] == shape {
                    return PartitionLoweredKey(candidate);
                }
            }
        }
        let id = u32::try_from(self.nodes.len())
            .expect("compiler-private partition lowered-key identity exhausted");
        self.nodes.push(shape);
        self.by_bucket.entry(bucket).or_default().push(id);
        PartitionLoweredKey(id)
    }
}

thread_local! {
    static PARTITION_LOWERED_KEYS: std::cell::RefCell<PartitionLoweredKeyInterner> =
        std::cell::RefCell::new(PartitionLoweredKeyInterner::default());
}

fn partition_lowered_shape_key(shape: PartitionLoweredShape) -> PartitionLoweredKey {
    PARTITION_LOWERED_KEYS.with(|interner| interner.borrow_mut().intern(shape))
}

fn partition_lowered_key(value: &Lowered) -> PartitionLoweredKey {
    let shape = match value {
        Lowered::Int { .. } => PartitionLoweredShape::Int,
        Lowered::Bool { .. } => PartitionLoweredShape::Bool,
        Lowered::ProcessExitStatus { .. } => PartitionLoweredShape::ProcessExitStatus,
        Lowered::CapabilityToken { .. } => PartitionLoweredShape::CapabilityToken,
        Lowered::ResourceToken { .. } => PartitionLoweredShape::ResourceToken,
        Lowered::BoundedNat(_) => PartitionLoweredShape::BoundedNat,
        Lowered::StructuralNat(_) => PartitionLoweredShape::StructuralNat,
        Lowered::ResponseBytes { .. } => PartitionLoweredShape::ResponseBytes,
        Lowered::HostResult {
            error,
            ok,
            err_constructor,
            ok_constructor,
            ..
        } => PartitionLoweredShape::HostResult {
            error: partition_lowered_key(error),
            ok: partition_lowered_key(ok),
            err_constructor: err_constructor.clone(),
            ok_constructor: ok_constructor.clone(),
        },
        Lowered::DynamicConstructor(dynamic) => PartitionLoweredShape::DynamicConstructor(
            dynamic
                .alternatives
                .iter()
                .map(|alternative| {
                    (
                        alternative.tag,
                        alternative.constructor.clone(),
                        alternative
                            .fields
                            .iter()
                            .map(partition_lowered_key)
                            .collect(),
                    )
                })
                .collect(),
        ),
        Lowered::Bytes(bytes) => PartitionLoweredShape::Bytes(bytes.clone()),
        Lowered::BorrowedNativeValue { .. } => PartitionLoweredShape::BorrowedNativeValue,
        Lowered::BorrowedOption { none, some, .. } => PartitionLoweredShape::BorrowedOption {
            none: none.clone(),
            some: some.clone(),
        },
        Lowered::String(value) => PartitionLoweredShape::String(value.clone()),
        Lowered::Constructor { constructor, args } => PartitionLoweredShape::Constructor {
            constructor: constructor.clone(),
            args: args.iter().map(partition_lowered_key).collect(),
        },
        Lowered::Record { fields } => PartitionLoweredShape::Record(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), partition_lowered_key(value)))
                .collect(),
        ),
        Lowered::Closure {
            captures,
            params,
            body,
        } => PartitionLoweredShape::Closure {
            captures: captures.iter().map(partition_lowered_key).collect(),
            params: params.clone(),
            body_descriptor: partition_runtime_expr_descriptor(body),
        },
        Lowered::DeclarationClosure {
            symbol,
            captures,
            params,
            body,
        } => PartitionLoweredShape::DeclarationClosure {
            symbol: symbol.clone(),
            captures: captures.iter().map(partition_lowered_key).collect(),
            params: params.clone(),
            body_descriptor: partition_runtime_expr_descriptor(body),
        },
        Lowered::ComputationalRecursorClosure {
            residual,
            invocation,
            ..
        } => PartitionLoweredShape::ComputationalRecursorClosure {
            residual: partition_lowered_key(residual),
            sibling_position: invocation.sibling_position,
            selection: partition_layer_key(&invocation.selection),
            unwind_head: invocation.unwind.partition_cursor.map(|cursor| cursor.node),
            qualification_head: invocation
                .unwind
                .partition_qualification
                .map(|cursor| cursor.node),
            open_obligation_head: invocation
                .unwind
                .partition_open_obligation
                .map(|cursor| cursor.node),
            checked_parent: partition_checked_parent_key(invocation.checked_parent),
            computational_ih_slot_template_id: invocation.computational_ih_slot_template_id,
            checked_invocation_source: invocation
                .checked_invocation
                .map(|checked| checked.source.into()),
        },
        Lowered::Trap(trap) => PartitionLoweredShape::Trap(trap.clone()),
        Lowered::RecursiveBackedge => PartitionLoweredShape::RecursiveBackedge,
    };
    partition_lowered_shape_key(shape)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartitionEliminatorKey {
    Computational {
        eliminator_descriptor: PartitionStaticDescriptor,
        env: Vec<PartitionLoweredKey>,
        retained_scrutinee_index: Option<usize>,
        checked_frame_id: Option<u64>,
        checked_invocation_source: Option<PartitionInvocationTemplateKey>,
    },
    Ordinary {
        eliminator_descriptor: PartitionStaticDescriptor,
        env: Vec<PartitionLoweredKey>,
        retained_scrutinee_index: Option<usize>,
    },
    InvocationReturn,
}

fn partition_eliminator_key(value: &OwnedPartitionEliminator) -> PartitionEliminatorKey {
    match value {
        OwnedPartitionEliminator::Computational {
            cases,
            default,
            env,
            retained_scrutinee_index,
            checked_frame_id,
            checked_invocation_source,
            ..
        } => PartitionEliminatorKey::Computational {
            eliminator_descriptor: partition_computational_match_descriptor(cases, default),
            env: env.iter().map(partition_lowered_key).collect(),
            retained_scrutinee_index: *retained_scrutinee_index,
            checked_frame_id: *checked_frame_id,
            checked_invocation_source: checked_invocation_source.map(Into::into),
        },
        OwnedPartitionEliminator::Ordinary {
            cases,
            default,
            env,
            retained_scrutinee_index,
        } => PartitionEliminatorKey::Ordinary {
            eliminator_descriptor: partition_match_descriptor(cases, default),
            env: env.iter().map(partition_lowered_key).collect(),
            retained_scrutinee_index: *retained_scrutinee_index,
        },
        OwnedPartitionEliminator::InvocationReturn => PartitionEliminatorKey::InvocationReturn,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PartitionCleanupSuffixId(pub(super) u32);

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionCleanupSuffixKey {
    checked_join_site_id: u64,
    terminal_distance: usize,
    current: PartitionEliminatorKey,
    capture_field_types: Vec<Type>,
    successor: Option<PartitionCleanupSuffixId>,
}

#[derive(Clone)]
pub(super) struct PartitionCleanupSuffixDefinition {
    pub(super) current: OwnedPartitionEliminator,
    pub(super) capture_field_types: Vec<Type>,
    pub(super) successor: Option<PartitionCleanupSuffixId>,
}

#[derive(Default)]
pub(super) struct PartitionCleanupSuffixInterner {
    by_bucket: BTreeMap<(u64, u64), Vec<PartitionCleanupSuffixId>>,
    keys: Vec<PartitionCleanupSuffixKey>,
    definitions: Vec<PartitionCleanupSuffixDefinition>,
    bytes_constructed: usize,
    bytes_retained: usize,
    bucket_probes: usize,
    exact_comparisons: usize,
}

impl PartitionCleanupSuffixInterner {
    pub(super) fn intern_step(
        &mut self,
        checked_join_site_id: u64,
        terminal_distance: usize,
        current: &OwnedPartitionEliminator,
        capture_field_types: Vec<Type>,
        successor: Option<PartitionCleanupSuffixId>,
    ) -> PartitionCleanupSuffixId {
        let key = PartitionCleanupSuffixKey {
            checked_join_site_id,
            terminal_distance,
            current: partition_eliminator_key(current),
            capture_field_types: capture_field_types.clone(),
            successor,
        };
        let bucket = partition_static_bucket(&key);
        self.bytes_constructed = self.bytes_constructed.saturating_add(bucket.bytes as usize);
        let bucket_key = (bucket.hash, bucket.bytes);
        if let Some(candidates) = self.by_bucket.get(&bucket_key) {
            self.bucket_probes = self.bucket_probes.saturating_add(candidates.len());
            for candidate in candidates.iter().copied() {
                self.exact_comparisons = self.exact_comparisons.saturating_add(1);
                if self.keys[candidate.0 as usize] == key {
                    return candidate;
                }
            }
        }
        let id = PartitionCleanupSuffixId(
            u32::try_from(self.keys.len())
                .expect("compiler-private cleanup suffix identity exhausted"),
        );
        self.bytes_retained = self.bytes_retained.saturating_add(bucket.bytes as usize);
        self.keys.push(key);
        self.definitions.push(PartitionCleanupSuffixDefinition {
            current: current.clone(),
            capture_field_types,
            successor,
        });
        self.by_bucket.entry(bucket_key).or_default().push(id);
        id
    }

    pub(super) fn definition(
        &self,
        id: PartitionCleanupSuffixId,
    ) -> Result<PartitionCleanupSuffixDefinition, CraneliftBackendError> {
        self.definitions.get(id.0 as usize).cloned().ok_or_else(|| {
            unsupported(
                "NativeCleanupStepV1",
                "cleanup suffix identity is out of bounds",
            )
        })
    }

    pub(super) fn counts(&self) -> (usize, usize, usize, usize) {
        (
            self.definitions.len(),
            self.bytes_constructed,
            self.bytes_retained,
            self.exact_comparisons,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionSelectedScopeKey {
    has_parent_scope: bool,
    eliminator_descriptor: PartitionStaticDescriptor,
    outer_env: Vec<PartitionLoweredKey>,
    checked_frame_id: Option<u64>,
    checked_invocation_source: Option<PartitionInvocationTemplateKey>,
}

fn partition_scope_key(scope: &Option<OwnedSelectedScope>) -> Option<PartitionSelectedScopeKey> {
    scope.as_ref().map(|scope| PartitionSelectedScopeKey {
        has_parent_scope: scope.parent_scope.is_some(),
        eliminator_descriptor: partition_computational_match_descriptor(
            &scope.frame.cases,
            &scope.frame.default,
        ),
        outer_env: scope
            .frame
            .outer_env
            .iter()
            .map(partition_lowered_key)
            .collect(),
        checked_frame_id: scope.frame.checked_frame_id,
        checked_invocation_source: scope.frame.checked_invocation_source.map(Into::into),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionSelectedContinuationKey {
    pending: Vec<PartitionEliminatorKey>,
    selected_has_ancestry: bool,
    selected_scope: Option<PartitionSelectedScopeKey>,
}

fn partition_selected_lineage_key(
    lineage: &[OwnedSourceSelectedContinuation],
) -> Vec<PartitionSelectedContinuationKey> {
    lineage
        .iter()
        .map(|selected| PartitionSelectedContinuationKey {
            pending: selected
                .pending
                .iter()
                .map(partition_eliminator_key)
                .collect(),
            selected_has_ancestry: !selected.selected_ancestry.is_empty(),
            selected_scope: partition_scope_key(&selected.selected_scope),
        })
        .collect()
}

fn partition_selected_key(
    selected: &OwnedSourceSelectedContinuation,
) -> PartitionSelectedContinuationKey {
    PartitionSelectedContinuationKey {
        pending: selected
            .pending
            .iter()
            .map(partition_eliminator_key)
            .collect(),
        selected_has_ancestry: !selected.selected_ancestry.is_empty(),
        selected_scope: partition_scope_key(&selected.selected_scope),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionContinuationStaticKey {
    action: PartitionProducerKontActionKey,
    successor: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartitionProducerKontActionKey {
    Done {
        value: PartitionLoweredKey,
        terminal: PartitionProducerKontTerminalIdentity,
    },
    ApplyActiveEliminators {
        value: PartitionLoweredKey,
        pending: Vec<PartitionEliminatorKey>,
        selected_has_ancestry: bool,
        selected_scope: Option<PartitionSelectedScopeKey>,
        selected_has_parent: bool,
    },
    ApplyEliminators {
        value: PartitionLoweredKey,
        eliminators: Vec<PartitionEliminatorKey>,
    },
    OrientedInvocationReturn {
        value: PartitionLoweredKey,
        checked: bool,
    },
    CheckedComputationalIHReturn {
        value: PartitionLoweredKey,
        call_template_id: u64,
    },
    ScopeBodyReturn {
        value: PartitionLoweredKey,
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
        source_successor: PartitionSourceResumeSite,
    },
    ExitScopeStart {
        value: PartitionLoweredKey,
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
    },
    ExitScopeComplete {
        value: PartitionLoweredKey,
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
        obligation_successor: Option<PartitionOpenControlObligationNodeId>,
        source_successor: PartitionSourceResumeSite,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PartitionProducerKontTerminalIdentity {
    CheckedJoin,
    ScalarArmReturn {
        partition_site_id: u64,
        edge_index: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartitionContinuationKey {
    checked_join: PartitionCheckedJoinIdentity,
    input_kind: ScalarMergeKind,
    outer_return_kind: ScalarMergeKind,
    static_bucket: PartitionStaticFingerprint,
    static_key: Arc<PartitionContinuationStaticKey>,
    field_types: Vec<Type>,
    field_map: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartitionSourcePrefixKey {
    Terminal,
    CheckedRecursiveInvocationReturn {
        source: PartitionInvocationTemplateKey,
        has_dynamic_splice_edge: bool,
        next: Box<Self>,
    },
    CheckedComputationalIHInvocationReturn {
        call_template_id: u64,
        next: Box<Self>,
    },
    ReturnFromSelectedCase {
        frame_id: Option<u64>,
        parent: PartitionSelectedContinuationKey,
        exit: Option<(
            PartitionRecursorNodeId,
            PartitionOpenControlObligationNodeId,
            Option<PartitionOpenControlObligationNodeId>,
        )>,
        next: Box<Self>,
    },
    LetBody {
        body_descriptor: PartitionStaticDescriptor,
        env: Vec<PartitionLoweredKey>,
        next: Box<Self>,
    },
    ApplyRecursorSelection {
        layer: PartitionLayerKey,
        next: Box<Self>,
    },
    UnwindRecursorSegment {
        stack_head: Option<PartitionRecursorNodeId>,
        qualification_head: Option<PartitionRecursorQualificationNodeId>,
        open_obligation_head: Option<PartitionOpenControlObligationNodeId>,
        next: Box<Self>,
    },
    IfScrutinee {
        branch_descriptor: PartitionStaticDescriptor,
        env: Vec<PartitionLoweredKey>,
        next: Box<Self>,
    },
    ConstructArgument {
        constructor: RuntimeSymbol,
        remaining_descriptor: PartitionStaticDescriptor,
        lowered: Vec<PartitionLoweredKey>,
        env: Vec<PartitionLoweredKey>,
        next: Box<Self>,
    },
    MatchScrutinee {
        eliminator_descriptor: PartitionStaticDescriptor,
        env: Vec<PartitionLoweredKey>,
        next: Box<Self>,
    },
    ComputationalMatchScrutinee {
        eliminator_descriptor: PartitionStaticDescriptor,
        env: Vec<PartitionLoweredKey>,
        checked_frame_id: Option<u64>,
        answer_route: SourceComputationalAnswerRoute,
        next: Box<Self>,
    },
    ProjectRecord {
        field: String,
        next: Box<Self>,
    },
    CallCallee {
        args_descriptor: PartitionStaticDescriptor,
        env: Vec<PartitionLoweredKey>,
        next: Box<Self>,
    },
    CallArgument {
        callee: PartitionLoweredKey,
        remaining_descriptor: PartitionStaticDescriptor,
        lowered: Vec<PartitionLoweredKey>,
        env: Vec<PartitionLoweredKey>,
        next: Box<Self>,
    },
}

fn partition_source_prefix_key(prefix: &SourcePrefixTemplate) -> PartitionSourcePrefixKey {
    let lowered = |values: &[Lowered]| values.iter().map(partition_lowered_key).collect();
    match prefix {
        SourcePrefixTemplate::Terminal { expected_outer } => {
            let _ = expected_outer;
            PartitionSourcePrefixKey::Terminal
        }
        SourcePrefixTemplate::CheckedRecursiveInvocationReturn { instance, next } => {
            PartitionSourcePrefixKey::CheckedRecursiveInvocationReturn {
                source: instance.source.into(),
                has_dynamic_splice_edge: instance.dynamic_splice_edge.is_some(),
                next: Box::new(partition_source_prefix_key(next)),
            }
        }
        SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
            call_template_id,
            next,
        } => PartitionSourcePrefixKey::CheckedComputationalIHInvocationReturn {
            call_template_id: *call_template_id,
            next: Box::new(partition_source_prefix_key(next)),
        },
        SourcePrefixTemplate::ReturnFromSelectedCase {
            delimiter,
            parent_capture,
            exit_transition,
            next,
        } => {
            let parent = parent_capture
                .as_ref()
                .map(partition_selected_key)
                .expect("planned selected return owns one immediate parent capture");
            PartitionSourcePrefixKey::ReturnFromSelectedCase {
                frame_id: delimiter.frame_id,
                parent,
                exit: exit_transition.as_ref().map(|transition| {
                    (
                        transition
                            .target
                            .expect("planned source exit owns its recursor cell")
                            .node,
                        transition
                            .exit_obligation
                            .expect("planned source exit owns its obligation cell")
                            .node,
                        transition
                            .exit_obligation_successor
                            .map(|cursor| cursor.node),
                    )
                }),
                next: Box::new(partition_source_prefix_key(next)),
            }
        }
        SourcePrefixTemplate::LetBody { body, env, next } => PartitionSourcePrefixKey::LetBody {
            body_descriptor: partition_runtime_expr_descriptor(body),
            env: lowered(env),
            next: Box::new(partition_source_prefix_key(next)),
        },
        SourcePrefixTemplate::ApplyRecursorSelection { layer, next } => {
            PartitionSourcePrefixKey::ApplyRecursorSelection {
                layer: partition_layer_key(layer),
                next: Box::new(partition_source_prefix_key(next)),
            }
        }
        SourcePrefixTemplate::UnwindRecursorSegment { stack, next, .. } => {
            PartitionSourcePrefixKey::UnwindRecursorSegment {
                stack_head: stack.partition_cursor.map(|cursor| cursor.node),
                qualification_head: stack.partition_qualification.map(|cursor| cursor.node),
                open_obligation_head: stack.partition_open_obligation.map(|cursor| cursor.node),
                next: Box::new(partition_source_prefix_key(next)),
            }
        }
        SourcePrefixTemplate::IfScrutinee {
            then_expr,
            else_expr,
            env,
            next,
        } => PartitionSourcePrefixKey::IfScrutinee {
            branch_descriptor: partition_runtime_expr_pair_descriptor(then_expr, else_expr),
            env: lowered(env),
            next: Box::new(partition_source_prefix_key(next)),
        },
        SourcePrefixTemplate::ConstructArgument {
            constructor,
            remaining,
            lowered: values,
            env,
            next,
        } => PartitionSourcePrefixKey::ConstructArgument {
            constructor: constructor.clone(),
            remaining_descriptor: partition_runtime_exprs_descriptor(remaining),
            lowered: lowered(values),
            env: lowered(env),
            next: Box::new(partition_source_prefix_key(next)),
        },
        SourcePrefixTemplate::MatchScrutinee {
            cases,
            default,
            env,
            next,
        } => PartitionSourcePrefixKey::MatchScrutinee {
            eliminator_descriptor: partition_match_descriptor(cases, default),
            env: lowered(env),
            next: Box::new(partition_source_prefix_key(next)),
        },
        SourcePrefixTemplate::ComputationalMatchScrutinee {
            cases,
            default,
            env,
            checked_frame_id,
            answer_route,
            next,
            ..
        } => PartitionSourcePrefixKey::ComputationalMatchScrutinee {
            eliminator_descriptor: partition_computational_match_descriptor(cases, default),
            env: lowered(env),
            checked_frame_id: *checked_frame_id,
            answer_route: *answer_route,
            next: Box::new(partition_source_prefix_key(next)),
        },
        SourcePrefixTemplate::ProjectRecord { field, next } => {
            PartitionSourcePrefixKey::ProjectRecord {
                field: field.clone(),
                next: Box::new(partition_source_prefix_key(next)),
            }
        }
        SourcePrefixTemplate::CallCallee { args, env, next } => {
            PartitionSourcePrefixKey::CallCallee {
                args_descriptor: partition_runtime_exprs_descriptor(args),
                env: lowered(env),
                next: Box::new(partition_source_prefix_key(next)),
            }
        }
        SourcePrefixTemplate::CallArgument {
            callee,
            remaining,
            lowered: values,
            env,
            next,
        } => PartitionSourcePrefixKey::CallArgument {
            callee: partition_lowered_key(callee),
            remaining_descriptor: partition_runtime_exprs_descriptor(remaining),
            lowered: lowered(values),
            env: lowered(env),
            next: Box::new(partition_source_prefix_key(next)),
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PartitionSourceNodeId(pub(super) u32);

#[derive(Clone, Copy)]
pub(super) struct PartitionSourceCursor {
    pub(super) node: PartitionSourceNodeId,
    pub(super) capture_pointer: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PartitionSourceResumeTarget {
    Kont(PartitionSourceNodeId),
    Terminal,
}

#[derive(Clone, Copy)]
pub(super) enum PartitionSourceResumeCursor {
    Kont(PartitionSourceCursor),
    Terminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PartitionProducerResumeTarget {
    Producer(usize),
    Terminal,
}

#[derive(Clone, Copy)]
pub(super) enum PartitionProducerResumeCursor {
    Producer(PartitionProducerKontCursor),
    Terminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PartitionSourceResumeSite {
    pub(super) target: PartitionSourceResumeTarget,
    pub(super) parent_return: PartitionSourceNodeId,
    pub(super) producer: PartitionProducerResumeTarget,
    pub(super) terminal_outer: ContinuationCursorId,
    pub(super) pending_exit_head: Option<PartitionRecursorNodeId>,
    pub(super) pending_qualification_head: Option<PartitionRecursorQualificationNodeId>,
    pub(super) pending_obligation_head: Option<PartitionOpenControlObligationNodeId>,
}

#[derive(Clone)]
pub(super) struct PartitionSourceResume {
    pub(super) site: PartitionSourceResumeSite,
    pub(super) cursor: PartitionSourceResumeCursor,
    pub(super) parent_return_capture_pointer: Value,
    pub(super) producer: PartitionProducerResumeCursor,
    pub(super) pending_exit_stack: Option<RecursorUnwindStack>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionSourceNodeKey {
    current: PartitionSourcePrefixKey,
    capture_field_types: Vec<Type>,
    successor: Option<PartitionSourceNodeId>,
}

#[derive(Clone)]
pub(super) struct PartitionSourceNodeDefinition {
    pub(super) current: SourcePrefixTemplate,
    pub(super) capture_field_types: Vec<Type>,
    pub(super) successor: Option<PartitionSourceNodeId>,
}

#[derive(Default)]
pub(super) struct PartitionSourceNodeInterner {
    by_bucket: BTreeMap<(u64, u64), Vec<PartitionSourceNodeId>>,
    keys: Vec<PartitionSourceNodeKey>,
    definitions: Vec<PartitionSourceNodeDefinition>,
    bytes_constructed: usize,
    bytes_retained: usize,
    exact_comparisons: usize,
}

impl PartitionSourceNodeInterner {
    pub(super) fn intern(
        &mut self,
        current: SourcePrefixTemplate,
        capture_field_types: Vec<Type>,
        successor: Option<PartitionSourceNodeId>,
    ) -> PartitionSourceNodeId {
        let key = PartitionSourceNodeKey {
            current: partition_source_prefix_key(&current),
            capture_field_types: capture_field_types.clone(),
            successor,
        };
        let bucket = partition_static_bucket(&key);
        self.bytes_constructed = self.bytes_constructed.saturating_add(bucket.bytes as usize);
        let bucket_key = (bucket.hash, bucket.bytes);
        if let Some(candidates) = self.by_bucket.get(&bucket_key) {
            for candidate in candidates.iter().copied() {
                self.exact_comparisons = self.exact_comparisons.saturating_add(1);
                if self.keys[candidate.0 as usize] == key {
                    return candidate;
                }
            }
        }
        let id = PartitionSourceNodeId(
            u32::try_from(self.keys.len())
                .expect("compiler-private source-continuation node identity exhausted"),
        );
        self.bytes_retained = self.bytes_retained.saturating_add(bucket.bytes as usize);
        self.keys.push(key);
        self.definitions.push(PartitionSourceNodeDefinition {
            current,
            capture_field_types,
            successor,
        });
        self.by_bucket.entry(bucket_key).or_default().push(id);
        id
    }

    pub(super) fn definition(
        &self,
        id: PartitionSourceNodeId,
    ) -> Result<PartitionSourceNodeDefinition, CraneliftBackendError> {
        self.definitions.get(id.0 as usize).cloned().ok_or_else(|| {
            unsupported(
                "NativeSourceContinuationStepV1",
                "source-continuation node identity is out of bounds",
            )
        })
    }

    pub(super) fn counts(&self) -> (usize, usize, usize, usize) {
        (
            self.definitions.len(),
            self.bytes_constructed,
            self.bytes_retained,
            self.exact_comparisons,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PartitionRecursorNodeId(pub(super) u32);

#[derive(Clone, Copy)]
pub(super) struct PartitionRecursorCursor {
    pub(super) node: PartitionRecursorNodeId,
    pub(super) capture_pointer: Value,
}

#[derive(Clone, Copy)]
pub(super) struct PartitionRecursorQualificationCursor {
    pub(super) node: PartitionRecursorQualificationNodeId,
    pub(super) capture_pointer: Value,
}

#[derive(Clone, Copy)]
pub(super) struct PartitionOpenControlObligationCursor {
    pub(super) node: PartitionOpenControlObligationNodeId,
    pub(super) capture_pointer: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionRecursorNodeKey {
    current: PartitionLayerKey,
    capture_field_types: Vec<Type>,
    successor: Option<PartitionRecursorNodeId>,
}

#[derive(Clone)]
pub(super) struct PartitionRecursorNodeDefinition {
    pub(super) current: ComputationalRecursorLayer,
    pub(super) capture_field_types: Vec<Type>,
    pub(super) successor: Option<PartitionRecursorNodeId>,
}

#[derive(Default)]
pub(super) struct PartitionRecursorNodeInterner {
    by_bucket: BTreeMap<(u64, u64), Vec<PartitionRecursorNodeId>>,
    keys: Vec<PartitionRecursorNodeKey>,
    definitions: Vec<PartitionRecursorNodeDefinition>,
}

impl PartitionRecursorNodeInterner {
    pub(super) fn intern(
        &mut self,
        current: ComputationalRecursorLayer,
        capture_field_types: Vec<Type>,
        successor: Option<PartitionRecursorNodeId>,
    ) -> PartitionRecursorNodeId {
        let key = PartitionRecursorNodeKey {
            current: partition_layer_key(&current),
            capture_field_types: capture_field_types.clone(),
            successor,
        };
        let bucket = partition_static_bucket(&key);
        let bucket_key = (bucket.hash, bucket.bytes);
        if let Some(candidates) = self.by_bucket.get(&bucket_key) {
            for candidate in candidates.iter().copied() {
                if self.keys[candidate.0 as usize] == key {
                    return candidate;
                }
            }
        }
        let id = PartitionRecursorNodeId(
            u32::try_from(self.keys.len())
                .expect("compiler-private recursor-continuation node identity exhausted"),
        );
        self.keys.push(key);
        self.definitions.push(PartitionRecursorNodeDefinition {
            current,
            capture_field_types,
            successor,
        });
        self.by_bucket.entry(bucket_key).or_default().push(id);
        id
    }

    pub(super) fn definition(
        &self,
        id: PartitionRecursorNodeId,
    ) -> Result<PartitionRecursorNodeDefinition, CraneliftBackendError> {
        self.definitions.get(id.0 as usize).cloned().ok_or_else(|| {
            unsupported(
                "NativeRecursorContinuationStepV1",
                "recursor-continuation node identity is out of bounds",
            )
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PartitionRecursorQualificationNodeId(pub(super) u32);

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionRecursorQualificationNodeKey {
    target: PartitionRecursorNodeId,
    source: PartitionInvocationTemplateKey,
    successor: Option<PartitionRecursorQualificationNodeId>,
}

#[derive(Clone)]
pub(super) struct PartitionRecursorQualificationNodeDefinition {
    pub(super) target: PartitionRecursorNodeId,
    pub(super) source: InvocationTemplateRef,
    pub(super) successor: Option<PartitionRecursorQualificationNodeId>,
}

#[derive(Default)]
pub(super) struct PartitionRecursorQualificationNodeInterner {
    by_bucket: BTreeMap<(u64, u64), Vec<PartitionRecursorQualificationNodeId>>,
    keys: Vec<PartitionRecursorQualificationNodeKey>,
    definitions: Vec<PartitionRecursorQualificationNodeDefinition>,
}

impl PartitionRecursorQualificationNodeInterner {
    pub(super) fn intern(
        &mut self,
        target: PartitionRecursorNodeId,
        source: InvocationTemplateRef,
        successor: Option<PartitionRecursorQualificationNodeId>,
    ) -> PartitionRecursorQualificationNodeId {
        let key = PartitionRecursorQualificationNodeKey {
            target,
            source: source.into(),
            successor,
        };
        let bucket = partition_static_bucket(&key);
        let bucket_key = (bucket.hash, bucket.bytes);
        if let Some(candidates) = self.by_bucket.get(&bucket_key) {
            for candidate in candidates.iter().copied() {
                if self.keys[candidate.0 as usize] == key {
                    return candidate;
                }
            }
        }
        let id = PartitionRecursorQualificationNodeId(
            u32::try_from(self.keys.len())
                .expect("compiler-private recursor-qualification node identity exhausted"),
        );
        self.keys.push(key);
        self.definitions
            .push(PartitionRecursorQualificationNodeDefinition {
                target,
                source,
                successor,
            });
        self.by_bucket.entry(bucket_key).or_default().push(id);
        id
    }

    pub(super) fn definition(
        &self,
        id: PartitionRecursorQualificationNodeId,
    ) -> Result<PartitionRecursorQualificationNodeDefinition, CraneliftBackendError> {
        self.definitions.get(id.0 as usize).cloned().ok_or_else(|| {
            unsupported(
                "NativeRecursorContinuationStepV1",
                "recursor-qualification node identity is out of bounds",
            )
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PartitionOpenControlObligationNodeId(pub(super) u32);

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionOpenControlObligationNodeKey {
    target: PartitionRecursorNodeId,
    checked_frame_id: Option<u64>,
    semantic_pending: bool,
    has_parent_scope: bool,
    successor: Option<PartitionOpenControlObligationNodeId>,
}

#[derive(Clone)]
pub(super) struct PartitionOpenControlObligationNodeDefinition {
    pub(super) target: PartitionRecursorNodeId,
    pub(super) checked_frame_id: Option<u64>,
    pub(super) semantic_pending: bool,
    pub(super) has_parent_scope: bool,
    pub(super) successor: Option<PartitionOpenControlObligationNodeId>,
}

#[derive(Default)]
pub(super) struct PartitionOpenControlObligationNodeInterner {
    by_bucket: BTreeMap<(u64, u64), Vec<PartitionOpenControlObligationNodeId>>,
    keys: Vec<PartitionOpenControlObligationNodeKey>,
    definitions: Vec<PartitionOpenControlObligationNodeDefinition>,
}

impl PartitionOpenControlObligationNodeInterner {
    pub(super) fn intern(
        &mut self,
        target: PartitionRecursorNodeId,
        checked_frame_id: Option<u64>,
        semantic_pending: bool,
        has_parent_scope: bool,
        successor: Option<PartitionOpenControlObligationNodeId>,
    ) -> PartitionOpenControlObligationNodeId {
        let key = PartitionOpenControlObligationNodeKey {
            target,
            checked_frame_id,
            semantic_pending,
            has_parent_scope,
            successor,
        };
        let bucket = partition_static_bucket(&key);
        let bucket_key = (bucket.hash, bucket.bytes);
        if let Some(candidates) = self.by_bucket.get(&bucket_key) {
            for candidate in candidates.iter().copied() {
                if self.keys[candidate.0 as usize] == key {
                    return candidate;
                }
            }
        }
        let id = PartitionOpenControlObligationNodeId(
            u32::try_from(self.keys.len())
                .expect("compiler-private open-control-obligation identity exhausted"),
        );
        self.keys.push(key);
        self.definitions
            .push(PartitionOpenControlObligationNodeDefinition {
                target,
                checked_frame_id,
                semantic_pending,
                has_parent_scope,
                successor,
            });
        self.by_bucket.entry(bucket_key).or_default().push(id);
        id
    }

    pub(super) fn definition(
        &self,
        id: PartitionOpenControlObligationNodeId,
    ) -> Result<PartitionOpenControlObligationNodeDefinition, CraneliftBackendError> {
        self.definitions.get(id.0 as usize).cloned().ok_or_else(|| {
            unsupported(
                "NativeOpenControlObligationStepV1",
                "open-control-obligation node identity is out of bounds",
            )
        })
    }
}

pub(super) fn partition_source_head_template(
    continuation: &SourceContinuation<'_>,
    expected_outer: ContinuationCursorId,
) -> Result<SourcePrefixTemplate, CraneliftBackendError> {
    let terminal = || SourcePrefixTemplate::Terminal { expected_outer };
    Ok(match continuation {
        SourceContinuation::CheckedRecursiveInvocationReturn { instance, .. } => {
            SourcePrefixTemplate::CheckedRecursiveInvocationReturn {
                instance: instance.clone(),
                next: Box::new(terminal()),
            }
        }
        SourceContinuation::CheckedComputationalIHInvocationReturn {
            call_template_id, ..
        } => SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
            call_template_id: *call_template_id,
            next: Box::new(terminal()),
        },
        SourceContinuation::ReturnFromSelectedCase { delimiter, .. } => {
            SourcePrefixTemplate::ReturnFromSelectedCase {
                delimiter: *delimiter,
                parent_capture: None,
                exit_transition: None,
                next: Box::new(terminal()),
            }
        }
        SourceContinuation::LetBody { body, env, .. } => SourcePrefixTemplate::LetBody {
            body: body.clone(),
            env: env.clone(),
            next: Box::new(terminal()),
        },
        SourceContinuation::ApplyRecursorSelection { layer, .. } => {
            SourcePrefixTemplate::ApplyRecursorSelection {
                layer: layer.clone(),
                next: Box::new(terminal()),
            }
        }
        SourceContinuation::UnwindRecursorSegment {
            stack,
            resume_cursor,
            resume_cursor_instance,
            ..
        } => SourcePrefixTemplate::UnwindRecursorSegment {
            stack: stack.clone(),
            resume_cursor: *resume_cursor,
            resume_cursor_instance: *resume_cursor_instance,
            next: Box::new(terminal()),
        },
        SourceContinuation::IfScrutinee {
            then_expr,
            else_expr,
            env,
            ..
        } => SourcePrefixTemplate::IfScrutinee {
            then_expr: then_expr.clone(),
            else_expr: else_expr.clone(),
            env: env.clone(),
            next: Box::new(terminal()),
        },
        SourceContinuation::ConstructArgument {
            constructor,
            remaining,
            lowered,
            env,
            ..
        } => SourcePrefixTemplate::ConstructArgument {
            constructor: constructor.clone(),
            remaining: remaining.clone(),
            lowered: lowered.clone(),
            env: env.clone(),
            next: Box::new(terminal()),
        },
        SourceContinuation::MatchScrutinee {
            cases,
            default,
            env,
            ..
        } => SourcePrefixTemplate::MatchScrutinee {
            cases: cases.clone(),
            default: default.clone(),
            env: env.clone(),
            next: Box::new(terminal()),
        },
        SourceContinuation::ComputationalMatchScrutinee {
            cases,
            default,
            env,
            provenance,
            checked_frame_id,
            answer_route,
            ..
        } => SourcePrefixTemplate::ComputationalMatchScrutinee {
            cases: cases.clone(),
            default: default.clone(),
            env: env.clone(),
            provenance: *provenance,
            checked_frame_id: *checked_frame_id,
            answer_route: *answer_route,
            next: Box::new(terminal()),
        },
        SourceContinuation::ProjectRecord { field, .. } => SourcePrefixTemplate::ProjectRecord {
            field: field.clone(),
            next: Box::new(terminal()),
        },
        SourceContinuation::CallCallee { args, env, .. } => SourcePrefixTemplate::CallCallee {
            args: args.clone(),
            env: env.clone(),
            next: Box::new(terminal()),
        },
        SourceContinuation::CallArgument {
            callee,
            remaining,
            lowered,
            env,
            ..
        } => SourcePrefixTemplate::CallArgument {
            callee: callee.clone(),
            remaining: remaining.clone(),
            lowered: lowered.clone(),
            env: env.clone(),
            next: Box::new(terminal()),
        },
        SourceContinuation::Terminal(_) | SourceContinuation::Partitioned { .. } => {
            return Err(unsupported(
                "NativeSourceContinuationStepV1",
                "planning completeness: terminal/partitioned continuation cannot be pushed",
            ));
        }
    })
}

pub(super) fn partition_source_continuation_next<'a, 'b>(
    continuation: &'a SourceContinuation<'b>,
) -> Option<&'a SourceContinuation<'b>> {
    match continuation {
        SourceContinuation::CheckedRecursiveInvocationReturn { next, .. }
        | SourceContinuation::CheckedComputationalIHInvocationReturn { next, .. }
        | SourceContinuation::ReturnFromSelectedCase { next, .. }
        | SourceContinuation::LetBody { next, .. }
        | SourceContinuation::ApplyRecursorSelection { next, .. }
        | SourceContinuation::UnwindRecursorSegment { next, .. }
        | SourceContinuation::IfScrutinee { next, .. }
        | SourceContinuation::ConstructArgument { next, .. }
        | SourceContinuation::MatchScrutinee { next, .. }
        | SourceContinuation::ComputationalMatchScrutinee { next, .. }
        | SourceContinuation::ProjectRecord { next, .. }
        | SourceContinuation::CallCallee { next, .. }
        | SourceContinuation::CallArgument { next, .. } => Some(next.as_ref()),
        SourceContinuation::Terminal(_) | SourceContinuation::Partitioned { .. } => None,
    }
}

pub(super) fn partition_source_prefix_head_template<'a>(
    prefix: &'a SourcePrefixTemplate,
    expected_outer: ContinuationCursorId,
) -> Result<Option<(SourcePrefixTemplate, &'a SourcePrefixTemplate)>, CraneliftBackendError> {
    let terminal = || SourcePrefixTemplate::Terminal { expected_outer };
    Ok(Some(match prefix {
        SourcePrefixTemplate::Terminal {
            expected_outer: actual,
        } => {
            if *actual != expected_outer {
                return Err(unsupported(
                    "NativeSourceContinuationStepV1",
                    "source-prefix terminal cursor does not match its persistent-cell contract",
                ));
            }
            return Ok(None);
        }
        SourcePrefixTemplate::CheckedRecursiveInvocationReturn { instance, next } => (
            SourcePrefixTemplate::CheckedRecursiveInvocationReturn {
                instance: instance.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
            call_template_id,
            next,
        } => (
            SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
                call_template_id: *call_template_id,
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::ReturnFromSelectedCase {
            delimiter,
            parent_capture,
            exit_transition,
            next,
        } => (
            SourcePrefixTemplate::ReturnFromSelectedCase {
                delimiter: *delimiter,
                parent_capture: parent_capture.clone(),
                exit_transition: exit_transition.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::LetBody { body, env, next } => (
            SourcePrefixTemplate::LetBody {
                body: body.clone(),
                env: env.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::ApplyRecursorSelection { layer, next } => (
            SourcePrefixTemplate::ApplyRecursorSelection {
                layer: layer.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::UnwindRecursorSegment {
            stack,
            resume_cursor,
            resume_cursor_instance,
            next,
        } => (
            SourcePrefixTemplate::UnwindRecursorSegment {
                stack: stack.clone(),
                resume_cursor: *resume_cursor,
                resume_cursor_instance: *resume_cursor_instance,
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::IfScrutinee {
            then_expr,
            else_expr,
            env,
            next,
        } => (
            SourcePrefixTemplate::IfScrutinee {
                then_expr: then_expr.clone(),
                else_expr: else_expr.clone(),
                env: env.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::ConstructArgument {
            constructor,
            remaining,
            lowered,
            env,
            next,
        } => (
            SourcePrefixTemplate::ConstructArgument {
                constructor: constructor.clone(),
                remaining: remaining.clone(),
                lowered: lowered.clone(),
                env: env.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::MatchScrutinee {
            cases,
            default,
            env,
            next,
        } => (
            SourcePrefixTemplate::MatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::ComputationalMatchScrutinee {
            cases,
            default,
            env,
            provenance,
            checked_frame_id,
            answer_route,
            next,
        } => (
            SourcePrefixTemplate::ComputationalMatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                provenance: *provenance,
                checked_frame_id: *checked_frame_id,
                answer_route: *answer_route,
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::ProjectRecord { field, next } => (
            SourcePrefixTemplate::ProjectRecord {
                field: field.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::CallCallee { args, env, next } => (
            SourcePrefixTemplate::CallCallee {
                args: args.clone(),
                env: env.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
        SourcePrefixTemplate::CallArgument {
            callee,
            remaining,
            lowered,
            env,
            next,
        } => (
            SourcePrefixTemplate::CallArgument {
                callee: callee.clone(),
                remaining: remaining.clone(),
                lowered: lowered.clone(),
                env: env.clone(),
                next: Box::new(terminal()),
            },
            next.as_ref(),
        ),
    }))
}

pub(super) fn instantiate_partition_source_node<'a>(
    current: SourcePrefixTemplate,
    successor: SourceContinuation<'a>,
) -> Result<SourceContinuation<'a>, CraneliftBackendError> {
    Ok(match current {
        SourcePrefixTemplate::Terminal { .. } => {
            return Err(unsupported(
                "NativeSourceContinuationStepV1",
                "planning completeness: a Kont node cannot be terminal",
            ));
        }
        SourcePrefixTemplate::CheckedRecursiveInvocationReturn { instance, .. } => {
            SourceContinuation::CheckedRecursiveInvocationReturn {
                instance,
                next: Box::new(successor),
            }
        }
        SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
            call_template_id, ..
        } => SourceContinuation::CheckedComputationalIHInvocationReturn {
            call_template_id,
            next: Box::new(successor),
        },
        SourcePrefixTemplate::ReturnFromSelectedCase { delimiter, .. } => {
            SourceContinuation::ReturnFromSelectedCase {
                delimiter,
                next: Box::new(successor),
            }
        }
        SourcePrefixTemplate::LetBody { body, env, .. } => SourceContinuation::LetBody {
            body,
            env,
            next: Box::new(successor),
        },
        SourcePrefixTemplate::ApplyRecursorSelection { layer, .. } => {
            SourceContinuation::ApplyRecursorSelection {
                layer,
                next: Box::new(successor),
            }
        }
        SourcePrefixTemplate::UnwindRecursorSegment {
            stack,
            resume_cursor,
            resume_cursor_instance,
            ..
        } => SourceContinuation::UnwindRecursorSegment {
            stack,
            resume_cursor,
            resume_cursor_instance,
            next: Box::new(successor),
        },
        SourcePrefixTemplate::IfScrutinee {
            then_expr,
            else_expr,
            env,
            ..
        } => SourceContinuation::IfScrutinee {
            then_expr,
            else_expr,
            env,
            next: Box::new(successor),
        },
        SourcePrefixTemplate::ConstructArgument {
            constructor,
            remaining,
            lowered,
            env,
            ..
        } => SourceContinuation::ConstructArgument {
            constructor,
            remaining,
            lowered,
            env,
            next: Box::new(successor),
        },
        SourcePrefixTemplate::MatchScrutinee {
            cases,
            default,
            env,
            ..
        } => SourceContinuation::MatchScrutinee {
            cases,
            default,
            env,
            next: Box::new(successor),
        },
        SourcePrefixTemplate::ComputationalMatchScrutinee {
            cases,
            default,
            env,
            provenance,
            checked_frame_id,
            answer_route,
            ..
        } => SourceContinuation::ComputationalMatchScrutinee {
            cases,
            default,
            env,
            provenance,
            checked_frame_id,
            answer_route,
            next: Box::new(successor),
        },
        SourcePrefixTemplate::ProjectRecord { field, .. } => SourceContinuation::ProjectRecord {
            field,
            next: Box::new(successor),
        },
        SourcePrefixTemplate::CallCallee { args, env, .. } => SourceContinuation::CallCallee {
            args,
            env,
            next: Box::new(successor),
        },
        SourcePrefixTemplate::CallArgument {
            callee,
            remaining,
            lowered,
            env,
            ..
        } => SourceContinuation::CallArgument {
            callee,
            remaining,
            lowered,
            env,
            next: Box::new(successor),
        },
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionSourceArmStaticKey {
    consume_checked_entry_marker: bool,
    pending_computational_ih_call: Option<u64>,
    body: PartitionStaticDescriptor,
    env: Vec<PartitionLoweredKey>,
    declaration_stack: Vec<RuntimeSymbol>,
    active_recursive_sources: Vec<(PartitionInvocationTemplateKey, bool)>,
    source_head: Option<PartitionSourceNodeId>,
    pending_exit_head: Option<PartitionRecursorNodeId>,
    pending_qualification_head: Option<PartitionRecursorQualificationNodeId>,
    pending_obligation_head: Option<PartitionOpenControlObligationNodeId>,
    producer_head: Option<usize>,
    selected_has_ancestry: bool,
    selected_pending: Vec<PartitionEliminatorKey>,
    selected_scope: Option<PartitionSelectedScopeKey>,
    selected_has_parent: bool,
    cleanup_head: Option<PartitionCleanupSuffixId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartitionSourceArmKey {
    checked_join: PartitionCheckedJoinIdentity,
    required_kind: ScalarMergeKind,
    static_bucket: PartitionStaticFingerprint,
    static_key: Arc<PartitionSourceArmStaticKey>,
    field_types: Vec<Type>,
    field_map: Vec<usize>,
}

impl PartitionSourceArmKey {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        checked_join: PartitionCheckedJoinIdentity,
        required_kind: ScalarMergeKind,
        consume_checked_entry_marker: bool,
        pending_computational_ih_call: Option<u64>,
        body: &RuntimeExpr,
        env: &[Lowered],
        declaration_stack: &[RuntimeSymbol],
        active_recursive_invocations: &[CheckedRecursiveInvocationInstance],
        source_head: Option<PartitionSourceNodeId>,
        pending_exit_head: Option<PartitionRecursorNodeId>,
        pending_qualification_head: Option<PartitionRecursorQualificationNodeId>,
        pending_obligation_head: Option<PartitionOpenControlObligationNodeId>,
        producer_head: Option<usize>,
        _selected_activation: ContinuationActivationId,
        _selected_cursor: ContinuationCursorId,
        selected_ancestry: &[RecursorFrameProvenance],
        selected_pending: &[OwnedPartitionEliminator],
        selected_scope: &Option<OwnedSelectedScope>,
        selected_lineage: &[OwnedSourceSelectedContinuation],
        _terminal_outer: ContinuationCursorId,
        cleanup_head: Option<PartitionCleanupSuffixId>,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionSourceArmStaticKey {
            consume_checked_entry_marker,
            pending_computational_ih_call,
            body: partition_runtime_expr_descriptor(body),
            env: env.iter().map(partition_lowered_key).collect(),
            declaration_stack: declaration_stack.to_vec(),
            active_recursive_sources: active_recursive_invocations
                .iter()
                .map(|instance| {
                    (
                        instance.source.into(),
                        instance.dynamic_splice_edge.is_some(),
                    )
                })
                .collect(),
            source_head,
            pending_exit_head,
            pending_qualification_head,
            pending_obligation_head,
            producer_head,
            selected_has_ancestry: !selected_ancestry.is_empty(),
            selected_pending: selected_pending
                .iter()
                .map(partition_eliminator_key)
                .collect(),
            selected_scope: partition_scope_key(selected_scope),
            selected_has_parent: !selected_lineage.is_empty(),
            cleanup_head,
        };
        Self {
            checked_join,
            required_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionCleanupStepStaticKey {
    suffix: PartitionCleanupSuffixId,
    input: PartitionLoweredKey,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionSourceKontStaticKey {
    node: PartitionSourceNodeId,
    resume_parent_return: Option<PartitionSourceNodeId>,
    pending_exit_head: Option<PartitionRecursorNodeId>,
    pending_qualification_head: Option<PartitionRecursorQualificationNodeId>,
    pending_obligation_head: Option<PartitionOpenControlObligationNodeId>,
    producer_head: Option<usize>,
    pending_computational_ih_call: Option<u64>,
    input: PartitionLoweredKey,
    declaration_stack: Vec<RuntimeSymbol>,
    active_recursive_sources: Vec<(PartitionInvocationTemplateKey, bool)>,
    selected_has_ancestry: bool,
    selected_pending: Vec<PartitionEliminatorKey>,
    selected_scope: Option<PartitionSelectedScopeKey>,
    selected_has_parent: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartitionSourceKontKey {
    checked_join: PartitionCheckedJoinIdentity,
    required_kind: ScalarMergeKind,
    static_bucket: PartitionStaticFingerprint,
    static_key: Arc<PartitionSourceKontStaticKey>,
    field_types: Vec<Type>,
    field_map: Vec<usize>,
}

impl PartitionSourceKontKey {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        checked_join: PartitionCheckedJoinIdentity,
        required_kind: ScalarMergeKind,
        node: PartitionSourceNodeId,
        pending_exit_head: Option<PartitionRecursorNodeId>,
        pending_qualification_head: Option<PartitionRecursorQualificationNodeId>,
        pending_obligation_head: Option<PartitionOpenControlObligationNodeId>,
        producer_head: Option<usize>,
        pending_computational_ih_call: Option<u64>,
        input: &Lowered,
        declaration_stack: &[RuntimeSymbol],
        active_recursive_invocations: &[CheckedRecursiveInvocationInstance],
        _selected_activation: ContinuationActivationId,
        _selected_cursor: ContinuationCursorId,
        selected_ancestry: &[RecursorFrameProvenance],
        selected_pending: &[OwnedPartitionEliminator],
        selected_scope: &Option<OwnedSelectedScope>,
        selected_lineage: &[OwnedSourceSelectedContinuation],
        _terminal_outer: ContinuationCursorId,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionSourceKontStaticKey {
            node,
            resume_parent_return: None,
            pending_exit_head,
            pending_qualification_head,
            pending_obligation_head,
            producer_head,
            pending_computational_ih_call,
            input: partition_lowered_key(input),
            declaration_stack: declaration_stack.to_vec(),
            active_recursive_sources: active_recursive_invocations
                .iter()
                .map(|instance| {
                    (
                        instance.source.into(),
                        instance.dynamic_splice_edge.is_some(),
                    )
                })
                .collect(),
            selected_has_ancestry: !selected_ancestry.is_empty(),
            selected_pending: selected_pending
                .iter()
                .map(partition_eliminator_key)
                .collect(),
            selected_scope: partition_scope_key(selected_scope),
            selected_has_parent: !selected_lineage.is_empty(),
        };
        Self {
            checked_join,
            required_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn new_resume(
        checked_join: PartitionCheckedJoinIdentity,
        required_kind: ScalarMergeKind,
        node: PartitionSourceNodeId,
        parent_return: PartitionSourceNodeId,
        pending_exit_head: Option<PartitionRecursorNodeId>,
        pending_qualification_head: Option<PartitionRecursorQualificationNodeId>,
        pending_obligation_head: Option<PartitionOpenControlObligationNodeId>,
        producer_head: Option<usize>,
        pending_computational_ih_call: Option<u64>,
        input: &Lowered,
        declaration_stack: &[RuntimeSymbol],
        active_recursive_invocations: &[CheckedRecursiveInvocationInstance],
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionSourceKontStaticKey {
            node,
            resume_parent_return: Some(parent_return),
            pending_exit_head,
            pending_qualification_head,
            pending_obligation_head,
            producer_head,
            pending_computational_ih_call,
            input: partition_lowered_key(input),
            declaration_stack: declaration_stack.to_vec(),
            active_recursive_sources: active_recursive_invocations
                .iter()
                .map(|instance| {
                    (
                        instance.source.into(),
                        instance.dynamic_splice_edge.is_some(),
                    )
                })
                .collect(),
            selected_has_ancestry: false,
            selected_pending: Vec::new(),
            selected_scope: None,
            selected_has_parent: false,
        };
        Self {
            checked_join,
            required_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartitionCleanupStepKey {
    checked_join: PartitionCheckedJoinIdentity,
    required_kind: ScalarMergeKind,
    static_bucket: PartitionStaticFingerprint,
    static_key: PartitionCleanupStepStaticKey,
    field_types: Vec<Type>,
    field_map: Vec<usize>,
}

impl PartitionCleanupStepKey {
    pub(super) fn new(
        checked_join: PartitionCheckedJoinIdentity,
        required_kind: ScalarMergeKind,
        suffix: PartitionCleanupSuffixId,
        input: &Lowered,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionCleanupStepStaticKey {
            suffix,
            input: partition_lowered_key(input),
        };
        Self {
            checked_join,
            required_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key,
            field_types,
            field_map,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PartitionSemanticStateKey {
    ProducerKont(PartitionContinuationKey),
    SourceArm(PartitionSourceArmKey),
    SourceKont(PartitionSourceKontKey),
    CleanupStep(PartitionCleanupStepKey),
}

impl PartitionSemanticStateKey {
    fn site_id(&self) -> u64 {
        match self {
            Self::ProducerKont(key) => key.site_id(),
            Self::SourceArm(key) => key.checked_join.site_id,
            Self::SourceKont(key) => key.checked_join.site_id,
            Self::CleanupStep(key) => key.checked_join.site_id,
        }
    }

    fn bucket(&self) -> (u64, u64, u64) {
        let bucket = match self {
            Self::ProducerKont(key) => key.static_bucket,
            Self::SourceArm(key) => key.static_bucket,
            Self::SourceKont(key) => key.static_bucket,
            Self::CleanupStep(key) => key.static_bucket,
        };
        (self.site_id(), bucket.hash, bucket.bytes)
    }

    pub(super) fn return_contract(&self) -> PartitionStateReturnContract {
        match self {
            Self::ProducerKont(key) => PartitionStateReturnContract {
                checked_join: key.checked_join.clone(),
                required_kind: key.outer_return_kind,
                field_types: key.field_types.clone(),
                field_map: key.field_map.clone(),
            },
            Self::SourceArm(key) => PartitionStateReturnContract {
                checked_join: key.checked_join.clone(),
                required_kind: key.required_kind,
                field_types: key.field_types.clone(),
                field_map: key.field_map.clone(),
            },
            Self::SourceKont(key) => PartitionStateReturnContract {
                checked_join: key.checked_join.clone(),
                required_kind: key.required_kind,
                field_types: key.field_types.clone(),
                field_map: key.field_map.clone(),
            },
            Self::CleanupStep(key) => PartitionStateReturnContract {
                checked_join: key.checked_join.clone(),
                required_kind: key.required_kind,
                field_types: key.field_types.clone(),
                field_map: key.field_map.clone(),
            },
        }
    }
}

impl PartitionContinuationKey {
    pub(super) fn done(
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
        value: &Lowered,
        terminal: PartitionProducerKontTerminalIdentity,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionContinuationStaticKey {
            action: PartitionProducerKontActionKey::Done {
                value: partition_lowered_key(value),
                terminal,
            },
            successor: None,
        };
        Self {
            checked_join,
            input_kind: return_kind,
            outer_return_kind: return_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        checked_join: PartitionCheckedJoinIdentity,
        input_kind: ScalarMergeKind,
        outer_return_kind: ScalarMergeKind,
        value: &Lowered,
        _activation: ContinuationActivationId,
        _cursor: ContinuationCursorId,
        pending: &[OwnedPartitionEliminator],
        selected_ancestry: &[RecursorFrameProvenance],
        selected_scope: &Option<OwnedSelectedScope>,
        selected_lineage: &[OwnedSourceSelectedContinuation],
        successor: Option<usize>,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionContinuationStaticKey {
            action: PartitionProducerKontActionKey::ApplyActiveEliminators {
                value: partition_lowered_key(value),
                pending: pending.iter().map(partition_eliminator_key).collect(),
                selected_has_ancestry: !selected_ancestry.is_empty(),
                selected_scope: partition_scope_key(selected_scope),
                selected_has_parent: !selected_lineage.is_empty(),
            },
            successor,
        };
        Self {
            checked_join,
            input_kind,
            outer_return_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }

    pub(super) fn oriented_invocation_return(
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
        value: &Lowered,
        checked: bool,
        successor: Option<usize>,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionContinuationStaticKey {
            action: PartitionProducerKontActionKey::OrientedInvocationReturn {
                value: partition_lowered_key(value),
                checked,
            },
            successor,
        };
        Self {
            checked_join,
            input_kind: return_kind,
            outer_return_kind: return_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }

    pub(super) fn apply_eliminators(
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
        value: &Lowered,
        eliminators: &[OwnedPartitionEliminator],
        successor: Option<usize>,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionContinuationStaticKey {
            action: PartitionProducerKontActionKey::ApplyEliminators {
                value: partition_lowered_key(value),
                eliminators: eliminators.iter().map(partition_eliminator_key).collect(),
            },
            successor,
        };
        Self {
            checked_join,
            input_kind: return_kind,
            outer_return_kind: return_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }

    pub(super) fn checked_computational_ih_return(
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
        value: &Lowered,
        call_template_id: u64,
        successor: Option<usize>,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionContinuationStaticKey {
            action: PartitionProducerKontActionKey::CheckedComputationalIHReturn {
                value: partition_lowered_key(value),
                call_template_id,
            },
            successor,
        };
        Self {
            checked_join,
            input_kind: return_kind,
            outer_return_kind: return_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }

    pub(super) fn scope_body_return(
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
        value: &Lowered,
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
        source_successor: PartitionSourceResumeSite,
        successor: usize,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionContinuationStaticKey {
            action: PartitionProducerKontActionKey::ScopeBodyReturn {
                value: partition_lowered_key(value),
                target,
                obligation,
                source_successor,
            },
            successor: Some(successor),
        };
        Self {
            checked_join,
            input_kind: return_kind,
            outer_return_kind: return_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }

    pub(super) fn exit_scope_start(
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
        value: &Lowered,
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
        successor: Option<usize>,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionContinuationStaticKey {
            action: PartitionProducerKontActionKey::ExitScopeStart {
                value: partition_lowered_key(value),
                target,
                obligation,
            },
            successor,
        };
        Self {
            checked_join,
            input_kind: return_kind,
            outer_return_kind: return_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }

    pub(super) fn exit_scope_complete(
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
        value: &Lowered,
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
        obligation_successor: Option<PartitionOpenControlObligationNodeId>,
        source_successor: PartitionSourceResumeSite,
        field_types: Vec<Type>,
        field_map: Vec<usize>,
    ) -> Self {
        let static_key = PartitionContinuationStaticKey {
            action: PartitionProducerKontActionKey::ExitScopeComplete {
                value: partition_lowered_key(value),
                target,
                obligation,
                obligation_successor,
                source_successor,
            },
            successor: None,
        };
        Self {
            checked_join,
            input_kind: return_kind,
            outer_return_kind: return_kind,
            static_bucket: partition_static_bucket(&static_key),
            static_key: Arc::new(static_key),
            field_types,
            field_map,
        }
    }

    fn site_id(&self) -> u64 {
        self.checked_join.site_id
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PartitionContinuationState {
    pub(super) function: FuncId,
    pub(super) helper_index: usize,
    pub(super) lifecycle: PartitionStateLifecycle,
}

/// Reusable semantic promise made by one interned state definition.
///
/// This is intentionally not affine: any number of independently-authorized
/// call edges may target the same definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartitionStateReturnContract {
    pub(super) checked_join: PartitionCheckedJoinIdentity,
    pub(super) required_kind: ScalarMergeKind,
    field_types: Vec<Type>,
    field_map: Vec<usize>,
}

impl PartitionStateReturnContract {
    pub(super) fn producer_terminal(
        checked_join: PartitionCheckedJoinIdentity,
        required_kind: ScalarMergeKind,
    ) -> Self {
        Self {
            checked_join,
            required_kind,
            field_types: Vec::new(),
            field_map: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PartitionAggregateBudget {
    pub(super) max_states: usize,
    pub(super) max_edges: usize,
    pub(super) max_helpers: usize,
}

impl PartitionAggregateBudget {
    pub(super) const PRODUCTION: Self = Self {
        max_states: 4_096,
        max_edges: 16_384,
        max_helpers: 16_384,
    };
}

#[derive(Default)]
pub(super) struct PartitionContinuationInterner {
    by_bucket: BTreeMap<(u64, u64, u64), Vec<usize>>,
    keys: Vec<PartitionSemanticStateKey>,
    states: Vec<PartitionContinuationState>,
    contracts: Vec<PartitionStateReturnContract>,
    edges: usize,
    emitted: usize,
    descriptor_bytes_constructed: usize,
    descriptor_bytes_retained: usize,
    bucket_probes: usize,
    exact_key_comparisons: usize,
    exact_key_bytes_compared_upper_bound: usize,
}

impl PartitionContinuationInterner {
    pub(super) fn lookup(
        &mut self,
        key: &PartitionSemanticStateKey,
        budget: PartitionAggregateBudget,
    ) -> Result<Option<(usize, PartitionContinuationState)>, CraneliftBackendError> {
        self.edges = self.edges.checked_add(1).ok_or_else(|| {
            unsupported(
                "NativeFunctionPartition",
                "aggregate partition edge accounting overflowed",
            )
        })?;
        if self.edges > budget.max_edges {
            return Err(unsupported(
                "NativeFunctionPartition",
                format!(
                    "aggregate continuation graph exceeds its edge ceiling: actual {}, limit {}",
                    self.edges, budget.max_edges
                ),
            ));
        }
        let bucket = key.bucket();
        self.descriptor_bytes_constructed = self
            .descriptor_bytes_constructed
            .saturating_add(bucket.2 as usize);
        let mut found = None;
        if let Some(candidates) = self.by_bucket.get(&bucket) {
            self.bucket_probes = self.bucket_probes.saturating_add(candidates.len());
            for state_id in candidates.iter().copied() {
                self.exact_key_comparisons = self.exact_key_comparisons.saturating_add(1);
                let candidate_bytes = self.keys[state_id].bucket().2 as usize;
                self.exact_key_bytes_compared_upper_bound = self
                    .exact_key_bytes_compared_upper_bound
                    .saturating_add(candidate_bytes.max(bucket.2 as usize));
                if self.keys.get(state_id) == Some(key) {
                    found = Some(state_id);
                    break;
                }
            }
        }
        Ok(found.map(|state_id| (state_id, self.states[state_id])))
    }

    pub(super) fn reserve(
        &mut self,
        key: PartitionSemanticStateKey,
        function: FuncId,
        helper_index: usize,
        budget: PartitionAggregateBudget,
    ) -> Result<(usize, PartitionContinuationState), CraneliftBackendError> {
        if self.by_bucket.get(&key.bucket()).is_some_and(|candidates| {
            candidates
                .iter()
                .any(|state_id| self.keys.get(*state_id) == Some(&key))
        }) {
            return Err(unsupported(
                "NativeFunctionPartition",
                "one semantic continuation key was reserved more than once",
            ));
        }
        if self.states.len() >= budget.max_states {
            return Err(unsupported(
                "NativeFunctionPartition",
                format!(
                    "aggregate continuation graph exceeds its state ceiling: actual {}, limit {}",
                    self.states.len() + 1,
                    budget.max_states
                ),
            ));
        }
        if helper_index >= budget.max_helpers {
            return Err(unsupported(
                "NativeFunctionPartition",
                format!(
                    "aggregate native partition graph exceeds its helper ceiling: actual {}, limit {}",
                    helper_index + 1,
                    budget.max_helpers
                ),
            ));
        }
        let state_id = self.states.len();
        if std::env::var_os("KEN_NATIVE_PARTITION_METRICS").is_some() && state_id % 64 == 0 {
            let kind = match &key {
                PartitionSemanticStateKey::ProducerKont(_) => "producer",
                PartitionSemanticStateKey::SourceArm(_) => "eval",
                PartitionSemanticStateKey::SourceKont(_) => "kont",
                PartitionSemanticStateKey::CleanupStep(_) => "cleanup",
            };
            eprintln!(
                "KEN_NATIVE_PARTITION_PROGRESS states={} edges={} kind={} bucket={:?}",
                state_id + 1,
                self.edges,
                kind,
                key.bucket(),
            );
        }
        self.descriptor_bytes_retained = self
            .descriptor_bytes_retained
            .saturating_add(key.bucket().2 as usize);
        let state = PartitionContinuationState {
            function,
            helper_index,
            lifecycle: PartitionStateLifecycle::Reserved,
        };
        self.by_bucket
            .entry(key.bucket())
            .or_default()
            .push(state_id);
        self.keys.push(key);
        self.states.push(state);
        self.contracts.push(self.keys[state_id].return_contract());
        Ok((state_id, state))
    }

    pub(super) fn validate_call_contract(
        &self,
        state_id: usize,
        expected: &PartitionStateReturnContract,
    ) -> Result<(), CraneliftBackendError> {
        if self.contracts.get(state_id) != Some(expected) {
            return Err(unsupported(
                "NativeFunctionPartition",
                "interned predecessor call does not match the state return contract",
            ));
        }
        Ok(())
    }

    pub(super) fn begin_emitting(&mut self, state_id: usize) -> Result<(), CraneliftBackendError> {
        let state = self.states.get_mut(state_id).ok_or_else(|| {
            unsupported(
                "NativeFunctionPartition",
                "continuation state identity is out of bounds",
            )
        })?;
        if state.lifecycle != PartitionStateLifecycle::Reserved {
            return Err(unsupported(
                "NativeFunctionPartition",
                "continuation state was scheduled for definition more than once",
            ));
        }
        state.lifecycle = PartitionStateLifecycle::Emitting;
        Ok(())
    }

    pub(super) fn finish_definition(
        &mut self,
        state_id: usize,
    ) -> Result<(), CraneliftBackendError> {
        let state = self.states.get_mut(state_id).ok_or_else(|| {
            unsupported(
                "NativeFunctionPartition",
                "continuation state identity is out of bounds",
            )
        })?;
        if state.lifecycle != PartitionStateLifecycle::Emitting {
            return Err(unsupported(
                "NativeFunctionPartition",
                "continuation state definition did not own its reserved state",
            ));
        }
        state.lifecycle = PartitionStateLifecycle::Defined;
        self.emitted += 1;
        Ok(())
    }

    pub(super) fn require_complete(&self) -> Result<(), CraneliftBackendError> {
        if self.emitted != self.states.len()
            || self
                .states
                .iter()
                .any(|state| state.lifecycle != PartitionStateLifecycle::Defined)
        {
            return Err(unsupported(
                "NativeFunctionPartition",
                format!(
                    "continuation state accounting mismatch: planned {}, emitted {}",
                    self.states.len(),
                    self.emitted
                ),
            ));
        }
        Ok(())
    }

    pub(super) fn counts(&self) -> (usize, usize, usize) {
        (self.states.len(), self.edges, self.emitted)
    }

    pub(super) fn representation_counts(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.descriptor_bytes_constructed,
            self.descriptor_bytes_retained,
            self.bucket_probes,
            self.exact_key_comparisons,
            self.exact_key_bytes_compared_upper_bound,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartitionLedgerBaseline {
    pub(super) join_sites: BTreeSet<u64>,
    pub(super) subcontinuation_frames: BTreeSet<(u64, u64)>,
    pub(super) recursive_call_templates: BTreeSet<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PartitionBranchReturnDescriptor {
    pub(super) authority_id: u64,
    pub(super) partition_site_id: u64,
    pub(super) edge_index: u64,
    pub(super) helper_index: usize,
    pub(super) required_kind: ScalarMergeKind,
}

/// Move-only semantic authority for one outlined call edge.
///
/// This is deliberately independent of a caller-local Cranelift block. The
/// caller consumes it at call emission after validating the reusable state
/// return contract. The callee definition owns no call-edge authority.
pub(super) struct PartitionBranchReturnAuthority {
    pub(super) descriptor: PartitionBranchReturnDescriptor,
}

#[derive(Default)]
pub(super) struct PartitionBranchReturnLedger {
    next_authority: u64,
    plans: BTreeMap<u64, PartitionBranchReturnDescriptor>,
    consumed: BTreeSet<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PartitionCleanupTransitionDescriptor {
    authority_id: u64,
    from: Option<PartitionCleanupSuffixId>,
    to: PartitionCleanupSuffixId,
    helper_index: usize,
}

pub(super) struct PartitionCleanupTransitionAuthority {
    descriptor: PartitionCleanupTransitionDescriptor,
}

#[derive(Default)]
pub(super) struct PartitionCleanupTransitionLedger {
    next_authority: u64,
    plans: BTreeMap<u64, PartitionCleanupTransitionDescriptor>,
    consumed: BTreeSet<u64>,
}

impl PartitionCleanupTransitionLedger {
    pub(super) fn mint(
        &mut self,
        from: Option<PartitionCleanupSuffixId>,
        to: PartitionCleanupSuffixId,
        helper_index: usize,
    ) -> Result<PartitionCleanupTransitionAuthority, CraneliftBackendError> {
        let authority_id = self.next_authority;
        self.next_authority = self.next_authority.checked_add(1).ok_or_else(|| {
            unsupported(
                "NativeCleanupStepV1",
                "cleanup-transition authority identity exhausted",
            )
        })?;
        let descriptor = PartitionCleanupTransitionDescriptor {
            authority_id,
            from,
            to,
            helper_index,
        };
        if self.plans.insert(authority_id, descriptor).is_some() {
            return Err(unsupported(
                "NativeCleanupStepV1",
                "cleanup-transition authority was planned more than once",
            ));
        }
        Ok(PartitionCleanupTransitionAuthority { descriptor })
    }

    pub(super) fn consume(
        &mut self,
        authority: PartitionCleanupTransitionAuthority,
        from: Option<PartitionCleanupSuffixId>,
        to: PartitionCleanupSuffixId,
        helper_index: usize,
    ) -> Result<(), CraneliftBackendError> {
        let descriptor = authority.descriptor;
        if self.plans.get(&descriptor.authority_id) != Some(&descriptor)
            || descriptor.from != from
            || descriptor.to != to
            || descriptor.helper_index != helper_index
        {
            return Err(unsupported(
                "NativeCleanupStepV1",
                "cleanup-transition authority was swapped across an edge",
            ));
        }
        if !self.consumed.insert(descriptor.authority_id) {
            return Err(unsupported(
                "NativeCleanupStepV1",
                "cleanup-transition authority was replayed",
            ));
        }
        Ok(())
    }

    pub(super) fn require_complete(&self) -> Result<(), CraneliftBackendError> {
        let planned = self.plans.keys().copied().collect::<BTreeSet<_>>();
        if planned != self.consumed {
            return Err(unsupported(
                "NativeCleanupStepV1",
                format!(
                    "cleanup-transition authority mismatch: planned {planned:?}, consumed {:?}",
                    self.consumed
                ),
            ));
        }
        Ok(())
    }
}

impl PartitionBranchReturnLedger {
    pub(super) fn mint(
        &mut self,
        partition_site_id: u64,
        edge_index: u64,
        helper_index: usize,
        required_kind: ScalarMergeKind,
    ) -> Result<PartitionBranchReturnAuthority, CraneliftBackendError> {
        let authority_id = self.next_authority;
        self.next_authority = self.next_authority.checked_add(1).ok_or_else(|| {
            unsupported(
                "NativeFunctionPartition",
                "branch-return authority identity exhausted",
            )
        })?;
        let descriptor = PartitionBranchReturnDescriptor {
            authority_id,
            partition_site_id,
            edge_index,
            helper_index,
            required_kind,
        };
        if self.plans.insert(authority_id, descriptor).is_some() {
            return Err(unsupported(
                "NativeFunctionPartition",
                "branch-return authority was planned more than once",
            ));
        }
        Ok(PartitionBranchReturnAuthority { descriptor })
    }

    pub(super) fn consume(
        &mut self,
        authority: PartitionBranchReturnAuthority,
        helper_index: usize,
        actual_kind: ScalarMergeKind,
    ) -> Result<(), CraneliftBackendError> {
        let descriptor = authority.descriptor;
        if self.plans.get(&descriptor.authority_id) != Some(&descriptor)
            || descriptor.helper_index != helper_index
            || descriptor.required_kind != actual_kind
        {
            return Err(unsupported(
                "NativeFunctionPartition",
                "call-edge authority was swapped across helper, edge, or scalar kind",
            ));
        }
        if !self.consumed.insert(descriptor.authority_id) {
            return Err(unsupported(
                "NativeFunctionPartition",
                "branch-return authority was replayed",
            ));
        }
        Ok(())
    }

    pub(super) fn require_complete(&self) -> Result<(), CraneliftBackendError> {
        let planned = self.plans.keys().copied().collect::<BTreeSet<_>>();
        if planned != self.consumed {
            return Err(unsupported(
                "NativeFunctionPartition",
                format!(
                    "branch-return authority mismatch: planned {planned:?}, consumed {:?}",
                    self.consumed
                ),
            ));
        }
        Ok(())
    }
}

pub(super) fn partition_helper_return_kind_is_admissible(kind: ScalarMergeKind) -> bool {
    !matches!(kind, ScalarMergeKind::RecursiveBackedge)
}

pub(super) struct SourceArmPartitionWorkItem {
    pub(super) state_id: usize,
    pub(super) function: FuncId,
    pub(super) field_types: Vec<Type>,
    pub(super) field_map: Vec<usize>,
    pub(super) body: RuntimeExpr,
    pub(super) consume_checked_entry_marker: bool,
    pub(super) pending_computational_ih_call: Option<u64>,
    pub(super) env: Vec<Lowered>,
    pub(super) declaration_stack: Vec<RuntimeSymbol>,
    pub(super) active_recursive_invocations: Vec<CheckedRecursiveInvocationInstance>,
    pub(super) source_head: Option<PartitionSourceNodeId>,
    pub(super) source_capture_pointer: Option<Value>,
    pub(super) pending_partition_exit_stack: Option<RecursorUnwindStack>,
    pub(super) producer_kont: Option<PartitionProducerKontCursor>,
    pub(super) selected_activation: ContinuationActivationId,
    pub(super) selected_activation_instance: ActivationInstanceRef,
    pub(super) selected_cursor: ContinuationCursorId,
    pub(super) selected_cursor_instance: ControlCursorRef,
    pub(super) selected_ancestry: Vec<RecursorFrameProvenance>,
    pub(super) selected_pending: Vec<OwnedPartitionEliminator>,
    pub(super) selected_scope: Option<OwnedSelectedScope>,
    pub(super) selected_lineage: Vec<OwnedSourceSelectedContinuation>,
    pub(super) terminal_outer: ContinuationCursorId,
    pub(super) cleanup_head: Option<PartitionCleanupSuffixId>,
    pub(super) cleanup_capture_pointer: Option<Value>,
    pub(super) ledger_baseline: PartitionLedgerBaseline,
    pub(super) return_contract: PartitionStateReturnContract,
}

pub(super) struct CleanupStepPartitionWorkItem {
    pub(super) state_id: usize,
    pub(super) function: FuncId,
    pub(super) helper_index: usize,
    pub(super) field_types: Vec<Type>,
    pub(super) field_map: Vec<usize>,
    pub(super) input: Lowered,
    pub(super) suffix: PartitionCleanupSuffixId,
    pub(super) checked_join: PartitionCheckedJoinIdentity,
    pub(super) required_kind: ScalarMergeKind,
    pub(super) ledger_baseline: PartitionLedgerBaseline,
}

pub(super) struct SourceKontPartitionWorkItem {
    pub(super) state_id: usize,
    pub(super) function: FuncId,
    pub(super) helper_index: usize,
    pub(super) field_types: Vec<Type>,
    pub(super) field_map: Vec<usize>,
    pub(super) input: Lowered,
    pub(super) node: PartitionSourceNodeId,
    pub(super) capture_pointer: Value,
    pub(super) resume_parent: Option<PartitionSourceCursor>,
    pub(super) pending_partition_exit_stack: Option<RecursorUnwindStack>,
    pub(super) producer_kont: Option<PartitionProducerKontCursor>,
    pub(super) pending_computational_ih_call: Option<u64>,
    pub(super) declaration_stack: Vec<RuntimeSymbol>,
    pub(super) active_recursive_invocations: Vec<CheckedRecursiveInvocationInstance>,
    pub(super) selected_activation: ContinuationActivationId,
    pub(super) selected_activation_instance: ActivationInstanceRef,
    pub(super) selected_cursor: ContinuationCursorId,
    pub(super) selected_cursor_instance: ControlCursorRef,
    pub(super) selected_ancestry: Vec<RecursorFrameProvenance>,
    pub(super) selected_pending: Vec<OwnedPartitionEliminator>,
    pub(super) selected_scope: Option<OwnedSelectedScope>,
    pub(super) selected_lineage: Vec<OwnedSourceSelectedContinuation>,
    pub(super) terminal_outer: ContinuationCursorId,
    pub(super) ledger_baseline: PartitionLedgerBaseline,
    pub(super) return_contract: PartitionStateReturnContract,
}

#[derive(Clone)]
pub(super) enum ProducerKontAction {
    Done {
        terminal: PartitionProducerKontTerminalIdentity,
    },
    ApplyActiveEliminators {
        activation: ContinuationActivationId,
        activation_instance: ActivationInstanceRef,
        cursor: ContinuationCursorId,
        cursor_instance: ControlCursorRef,
        pending: Vec<OwnedPartitionEliminator>,
        selected_ancestry: Vec<RecursorFrameProvenance>,
        selected_scope: Option<OwnedSelectedScope>,
        selected_lineage: Vec<OwnedSourceSelectedContinuation>,
        capture_field_types: Vec<Type>,
    },
    ApplyEliminators {
        eliminators: Vec<OwnedPartitionEliminator>,
        capture_field_types: Vec<Type>,
    },
    OrientedInvocationReturn {
        checked: bool,
        capture_field_types: Vec<Type>,
    },
    CheckedComputationalIHReturn {
        call_template_id: u64,
        capture_field_types: Vec<Type>,
    },
    ScopeBodyReturn {
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
        source_successor: PartitionSourceResumeSite,
    },
    ExitScopeStart {
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
    },
    ExitScopeComplete {
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
        obligation_successor: Option<PartitionOpenControlObligationNodeId>,
        source_successor: PartitionSourceResumeSite,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartitionProducerKontSiteActionKey {
    Done {
        terminal: PartitionProducerKontTerminalIdentity,
    },
    ApplyActiveEliminators {
        pending: Vec<PartitionEliminatorKey>,
        selected_has_ancestry: bool,
        selected_scope: Option<PartitionSelectedScopeKey>,
        selected_has_parent: bool,
        capture_field_types: Vec<Type>,
    },
    ApplyEliminators {
        eliminators: Vec<PartitionEliminatorKey>,
        capture_field_types: Vec<Type>,
    },
    OrientedInvocationReturn {
        checked: bool,
        capture_field_types: Vec<Type>,
    },
    CheckedComputationalIHReturn {
        call_template_id: u64,
        capture_field_types: Vec<Type>,
    },
    ScopeBodyReturn {
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
        source_successor: PartitionSourceResumeSite,
    },
    ExitScopeStart {
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
    },
    ExitScopeComplete {
        target: PartitionRecursorNodeId,
        obligation: PartitionOpenControlObligationNodeId,
        obligation_successor: Option<PartitionOpenControlObligationNodeId>,
        source_successor: PartitionSourceResumeSite,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartitionProducerKontSiteKey {
    checked_join: PartitionCheckedJoinIdentity,
    return_kind: ScalarMergeKind,
    successor: Option<usize>,
    action: PartitionProducerKontSiteActionKey,
    static_bucket: PartitionStaticFingerprint,
}

impl PartitionProducerKontSiteKey {
    pub(super) fn new(
        action: &ProducerKontAction,
        successor: Option<PartitionProducerKontCursor>,
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
    ) -> Self {
        let action = match action {
            ProducerKontAction::Done { terminal } => PartitionProducerKontSiteActionKey::Done {
                terminal: *terminal,
            },
            ProducerKontAction::ApplyActiveEliminators {
                pending,
                selected_ancestry,
                selected_scope,
                selected_lineage,
                capture_field_types,
                ..
            } => PartitionProducerKontSiteActionKey::ApplyActiveEliminators {
                pending: pending.iter().map(partition_eliminator_key).collect(),
                selected_has_ancestry: !selected_ancestry.is_empty(),
                selected_scope: partition_scope_key(selected_scope),
                selected_has_parent: !selected_lineage.is_empty(),
                capture_field_types: capture_field_types.clone(),
            },
            ProducerKontAction::ApplyEliminators {
                eliminators,
                capture_field_types,
            } => PartitionProducerKontSiteActionKey::ApplyEliminators {
                eliminators: eliminators.iter().map(partition_eliminator_key).collect(),
                capture_field_types: capture_field_types.clone(),
            },
            ProducerKontAction::OrientedInvocationReturn {
                checked,
                capture_field_types,
            } => PartitionProducerKontSiteActionKey::OrientedInvocationReturn {
                checked: *checked,
                capture_field_types: capture_field_types.clone(),
            },
            ProducerKontAction::CheckedComputationalIHReturn {
                call_template_id,
                capture_field_types,
            } => PartitionProducerKontSiteActionKey::CheckedComputationalIHReturn {
                call_template_id: *call_template_id,
                capture_field_types: capture_field_types.clone(),
            },
            ProducerKontAction::ScopeBodyReturn {
                target,
                obligation,
                source_successor,
            } => PartitionProducerKontSiteActionKey::ScopeBodyReturn {
                target: *target,
                obligation: *obligation,
                source_successor: *source_successor,
            },
            ProducerKontAction::ExitScopeStart { target, obligation } => {
                PartitionProducerKontSiteActionKey::ExitScopeStart {
                    target: *target,
                    obligation: *obligation,
                }
            }
            ProducerKontAction::ExitScopeComplete {
                target,
                obligation,
                obligation_successor,
                source_successor,
            } => PartitionProducerKontSiteActionKey::ExitScopeComplete {
                target: *target,
                obligation: *obligation,
                obligation_successor: *obligation_successor,
                source_successor: *source_successor,
            },
        };
        let successor = successor.map(|cursor| cursor.site_id);
        let static_bucket =
            partition_static_bucket(&(&checked_join, return_kind, successor, &action));
        Self {
            checked_join,
            return_kind,
            successor,
            action,
            static_bucket,
        }
    }
}

#[derive(Default)]
pub(super) struct PartitionProducerKontSiteInterner {
    by_bucket: BTreeMap<(u64, u64), Vec<usize>>,
    keys: Vec<PartitionProducerKontSiteKey>,
}

impl PartitionProducerKontSiteInterner {
    pub(super) fn lookup(&self, key: &PartitionProducerKontSiteKey) -> Option<usize> {
        self.by_bucket
            .get(&(key.static_bucket.hash, key.static_bucket.bytes))
            .and_then(|candidates| {
                candidates
                    .iter()
                    .copied()
                    .find(|site_id| self.keys.get(*site_id) == Some(key))
            })
    }

    pub(super) fn insert(
        &mut self,
        site_id: usize,
        key: PartitionProducerKontSiteKey,
    ) -> Result<(), CraneliftBackendError> {
        if site_id != self.keys.len() || self.lookup(&key).is_some() {
            return Err(unsupported(
                "NativeProducerContinuationStepV1",
                "producer continuation site identity was reserved twice",
            ));
        }
        self.by_bucket
            .entry((key.static_bucket.hash, key.static_bucket.bytes))
            .or_default()
            .push(site_id);
        self.keys.push(key);
        Ok(())
    }
}

pub(super) struct ProducerKontPartitionWorkItem {
    pub(super) state_id: usize,
    pub(super) site_id: usize,
    pub(super) function: FuncId,
    pub(super) field_types: Vec<Type>,
    pub(super) field_map: Vec<usize>,
    pub(super) value: Lowered,
    pub(super) action: ProducerKontAction,
    pub(super) capture_pointer: Option<Value>,
    pub(super) successor: Option<PartitionProducerKontCursor>,
    pub(super) ledger_baseline: PartitionLedgerBaseline,
    pub(super) declaration_stack: Vec<RuntimeSymbol>,
    pub(super) active_recursive_invocations: Vec<CheckedRecursiveInvocationInstance>,
    pub(super) checked_join: PartitionCheckedJoinIdentity,
    pub(super) return_kind: ScalarMergeKind,
}

#[derive(Clone)]
pub(super) struct OwnedSourceSelectedContinuation {
    pub(super) activation: ContinuationActivationId,
    pub(super) activation_instance: ActivationInstanceRef,
    pub(super) cursor: ContinuationCursorId,
    pub(super) cursor_instance: ControlCursorRef,
    pub(super) pending: Vec<OwnedPartitionEliminator>,
    pub(super) selected_ancestry: Vec<RecursorFrameProvenance>,
    pub(super) selected_scope: Option<OwnedSelectedScope>,
}

pub(super) fn own_partition_selected(
    selected: &SourceSelectedContinuation<'_>,
) -> Option<OwnedSourceSelectedContinuation> {
    if selected.parent.is_some() || !partition_scope_is_admissible(&selected.selected_scope) {
        return None;
    }
    let pending = own_partition_eliminators(&selected.pending)?;
    if !partition_eliminators_are_admissible(&pending) {
        return None;
    }
    Some(OwnedSourceSelectedContinuation {
        activation: selected.activation,
        activation_instance: selected.activation_instance,
        cursor: selected.cursor,
        cursor_instance: selected.cursor_instance,
        pending,
        selected_ancestry: selected.selected_ancestry.clone(),
        selected_scope: selected.selected_scope.clone(),
    })
}

pub(super) fn own_partition_selected_lineage(
    lineage: &[SourceSelectedContinuation<'_>],
) -> Option<Vec<OwnedSourceSelectedContinuation>> {
    lineage
        .iter()
        .map(|selected| own_partition_selected(selected))
        .collect()
}

pub(super) fn append_partition_selected_lineage_values(
    lowering: &mut Lowering<'_>,
    builder: &mut FunctionBuilder<'_>,
    lineage: &[OwnedSourceSelectedContinuation],
    output: &mut Vec<Value>,
) -> Result<(), CraneliftBackendError> {
    if let Some(selected) = lineage.last() {
        output.push(selected.activation_instance.0);
        output.push(selected.cursor_instance.0);
        append_partition_eliminator_values(lowering, builder, &selected.pending, output)?;
        append_partition_scope_values(lowering, builder, &selected.selected_scope, output)?;
    }
    Ok(())
}

pub(super) fn rebuild_partition_selected_lineage(
    lineage: &mut [OwnedSourceSelectedContinuation],
    values: &mut impl Iterator<Item = Value>,
    native_int_tags: &mut BTreeMap<Value, Value>,
) -> Result<(), CraneliftBackendError> {
    if let Some(selected) = lineage.last_mut() {
        selected.activation_instance = ActivationInstanceRef(values.next().ok_or_else(|| {
            unsupported(
                "NativeControlCellV1",
                "selected lineage lost its activation-instance reference",
            )
        })?);
        selected.cursor_instance = ControlCursorRef(values.next().ok_or_else(|| {
            unsupported(
                "NativeControlCellV1",
                "selected lineage lost its cursor-instance reference",
            )
        })?);
        rebuild_partition_eliminators(&mut selected.pending, values, native_int_tags)?;
        rebuild_partition_scope(&mut selected.selected_scope, values, native_int_tags)?;
    }
    Ok(())
}

#[derive(Clone)]
pub(super) enum OwnedPartitionEliminator {
    Computational {
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        retained_scrutinee_index: Option<usize>,
        provenance: RecursorFrameProvenance,
        checked_frame_id: Option<u64>,
        checked_invocation_id: Option<u64>,
        checked_invocation_source: Option<InvocationTemplateRef>,
        checked_invocation_depth: usize,
    },
    Ordinary {
        cases: Vec<crate::RuntimeMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        retained_scrutinee_index: Option<usize>,
    },
    InvocationReturn,
}

pub(super) struct ArmPartitionWorkItem {
    pub(super) function: FuncId,
    pub(super) helper_index: usize,
    pub(super) field_types: Vec<Type>,
    pub(super) field_map: Vec<usize>,
    pub(super) body: RuntimeExpr,
    pub(super) env: Vec<Lowered>,
    pub(super) eliminators: Vec<OwnedPartitionEliminator>,
    pub(super) producer_kont: Option<PartitionProducerKontCursor>,
    pub(super) ledger_baseline: PartitionLedgerBaseline,
    pub(super) return_authority: PartitionBranchReturnAuthority,
}

pub(super) fn own_partition_eliminators(
    eliminators: &[EliminatorFrame<'_>],
) -> Option<Vec<OwnedPartitionEliminator>> {
    eliminators
        .iter()
        .map(|frame| match frame {
            EliminatorFrame::Computational(frame) if frame.deferred_constructor_case.is_none() => {
                Some(OwnedPartitionEliminator::Computational {
                    cases: frame.cases.to_vec(),
                    default: frame.default.clone(),
                    env: frame.env.to_vec(),
                    retained_scrutinee_index: frame.retained_scrutinee_index,
                    provenance: frame.provenance,
                    checked_frame_id: frame.checked_frame_id,
                    checked_invocation_id: frame.checked_invocation_id,
                    checked_invocation_source: frame.checked_invocation_source,
                    checked_invocation_depth: frame.checked_invocation_depth,
                })
            }
            EliminatorFrame::Ordinary(frame) if frame.deferred_constructor_case.is_none() => {
                Some(OwnedPartitionEliminator::Ordinary {
                    cases: frame.cases.to_vec(),
                    default: frame.default.clone(),
                    env: frame.env.to_vec(),
                    retained_scrutinee_index: frame.retained_scrutinee_index,
                })
            }
            EliminatorFrame::InvocationReturn => Some(OwnedPartitionEliminator::InvocationReturn),
            EliminatorFrame::Computational(_)
            | EliminatorFrame::Ordinary(_)
            | EliminatorFrame::PendingLet(_)
            | EliminatorFrame::Active(_) => None,
        })
        .collect()
}

pub(super) fn partition_eliminators_are_admissible(
    eliminators: &[OwnedPartitionEliminator],
) -> bool {
    eliminators.iter().all(|frame| match frame {
        OwnedPartitionEliminator::Computational { env, .. }
        | OwnedPartitionEliminator::Ordinary { env, .. } => {
            env.iter().all(partition_lowered_is_admissible)
        }
        OwnedPartitionEliminator::InvocationReturn => true,
    })
}

pub(super) fn append_partition_eliminator_values(
    lowering: &mut Lowering<'_>,
    builder: &mut FunctionBuilder<'_>,
    eliminators: &[OwnedPartitionEliminator],
    output: &mut Vec<Value>,
) -> Result<(), CraneliftBackendError> {
    for frame in eliminators {
        match frame {
            OwnedPartitionEliminator::Computational { env, .. }
            | OwnedPartitionEliminator::Ordinary { env, .. } => {
                for value in env {
                    append_partition_lowered_values(lowering, builder, value, output)?;
                }
            }
            OwnedPartitionEliminator::InvocationReturn => {}
        }
    }
    Ok(())
}

pub(super) fn rebuild_partition_eliminators(
    eliminators: &mut [OwnedPartitionEliminator],
    values: &mut impl Iterator<Item = Value>,
    native_int_tags: &mut BTreeMap<Value, Value>,
) -> Result<(), CraneliftBackendError> {
    for frame in eliminators {
        match frame {
            OwnedPartitionEliminator::Computational { env, .. }
            | OwnedPartitionEliminator::Ordinary { env, .. } => {
                for value in env {
                    rebuild_partition_lowered(value, values, native_int_tags)?;
                }
            }
            OwnedPartitionEliminator::InvocationReturn => {}
        }
    }
    Ok(())
}

pub(super) fn borrow_partition_eliminators<'a>(
    eliminators: &'a [OwnedPartitionEliminator],
) -> Vec<EliminatorFrame<'a>> {
    eliminators
        .iter()
        .map(|frame| match frame {
            OwnedPartitionEliminator::Computational {
                cases,
                default,
                env,
                retained_scrutinee_index,
                provenance,
                checked_frame_id,
                checked_invocation_id,
                checked_invocation_source,
                checked_invocation_depth,
            } => EliminatorFrame::Computational(ComputationalEliminatorFrame {
                cases,
                default,
                env,
                retained_scrutinee_index: *retained_scrutinee_index,
                deferred_constructor_case: None,
                provenance: *provenance,
                checked_frame_id: *checked_frame_id,
                checked_invocation_id: *checked_invocation_id,
                checked_invocation_source: *checked_invocation_source,
                checked_invocation_depth: *checked_invocation_depth,
            }),
            OwnedPartitionEliminator::Ordinary {
                cases,
                default,
                env,
                retained_scrutinee_index,
            } => EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                cases,
                default,
                env,
                retained_scrutinee_index: *retained_scrutinee_index,
                deferred_constructor_case: None,
            }),
            OwnedPartitionEliminator::InvocationReturn => EliminatorFrame::InvocationReturn,
        })
        .collect()
}

pub(super) fn partition_frame_size(field_count: usize) -> Result<u32, CraneliftBackendError> {
    let count = u32::try_from(field_count).map_err(|_| {
        unsupported(
            "NativeFunctionPartition",
            "private partition frame has too many fields",
        )
    })?;
    count
        .checked_mul(PARTITION_FRAME_FIELD_BYTES)
        .map(|size| size.max(PARTITION_FRAME_FIELD_BYTES))
        .ok_or_else(|| {
            unsupported(
                "NativeFunctionPartition",
                "private partition frame size overflowed",
            )
        })
}

pub(super) fn partition_frame_layout(
    builder: &FunctionBuilder<'_>,
    logical_fields: &[Value],
) -> (Vec<Value>, Vec<Type>, Vec<usize>) {
    let mut index_by_value = BTreeMap::new();
    let mut values = Vec::new();
    let mut field_types = Vec::new();
    let mut field_map = Vec::with_capacity(logical_fields.len());
    for value in logical_fields.iter().copied() {
        let index = if let Some(index) = index_by_value.get(&value).copied() {
            index
        } else {
            let index = values.len();
            index_by_value.insert(value, index);
            values.push(value);
            field_types.push(builder.func.dfg.value_type(value));
            index
        };
        field_map.push(index);
    }
    (values, field_types, field_map)
}

pub(super) fn expand_partition_frame_values(
    values: &[Value],
    field_map: &[usize],
) -> Result<Vec<Value>, CraneliftBackendError> {
    field_map
        .iter()
        .map(|index| {
            values.get(*index).copied().ok_or_else(|| {
                unsupported(
                    "NativeFunctionPartition",
                    "private frame occurrence map is out of bounds",
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_descriptor_bucket_collision_keeps_exact_identity_distinct() {
        let mut interner = PartitionStaticDescriptorInterner::default();
        let fingerprint = PartitionStaticFingerprint { hash: 7, bytes: 3 };
        let first = interner.intern(fingerprint, b"one".to_vec());
        let colliding = interner.intern(fingerprint, b"two".to_vec());
        let repeated = interner.intern(fingerprint, b"one".to_vec());

        assert_ne!(first, colliding);
        assert_eq!(first, repeated);
        assert_eq!(interner.canonical.len(), 2);
        assert_eq!(interner.exact_comparisons, 2);
    }

    #[test]
    fn lowered_key_bucket_collision_keeps_exact_shape_distinct() {
        let mut interner = PartitionLoweredKeyInterner::default();
        let int = interner.intern(PartitionLoweredShape::Int);
        let bool_bucket = partition_static_bucket(&PartitionLoweredShape::Bool);
        interner.by_bucket.clear();
        interner
            .by_bucket
            .insert((bool_bucket.hash, bool_bucket.bytes), vec![int.0]);

        let boolean = interner.intern(PartitionLoweredShape::Bool);
        assert_ne!(int, boolean);
        assert_eq!(interner.nodes.len(), 2);
    }

    fn cleanup_eliminator(label: &str) -> OwnedPartitionEliminator {
        OwnedPartitionEliminator::Ordinary {
            cases: vec![crate::RuntimeMatchCase {
                constructor: label.to_string(),
                binders: 0,
                body: RuntimeExpr::Value(RuntimeValue::Int(0.into())),
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: format!("{label} default"),
            },
            env: Vec::new(),
            retained_scrutinee_index: None,
        }
    }

    #[test]
    fn cleanup_suffix_is_terminal_to_head_and_reuses_an_exact_shared_tail() {
        let mut interner = PartitionCleanupSuffixInterner::default();
        let tail = cleanup_eliminator("Tail");
        let tail_id = interner.intern_step(7, 0, &tail, vec![types::I64], None);
        let repeated_tail = interner.intern_step(7, 0, &tail, vec![types::I64], None);
        assert_eq!(tail_id, repeated_tail);

        let first_head = interner.intern_step(
            7,
            1,
            &cleanup_eliminator("First"),
            vec![types::I64],
            Some(tail_id),
        );
        let second_head = interner.intern_step(
            7,
            1,
            &cleanup_eliminator("Second"),
            vec![types::I64],
            Some(tail_id),
        );
        assert_ne!(first_head, second_head);
        assert_eq!(interner.counts().0, 3);
    }

    #[test]
    fn cleanup_suffix_rejects_occurrence_schema_and_successor_aliases() {
        let mut interner = PartitionCleanupSuffixInterner::default();
        let current = cleanup_eliminator("Node");
        let tail = interner.intern_step(7, 0, &cleanup_eliminator("Tail"), vec![types::I64], None);
        let original = interner.intern_step(7, 1, &current, vec![types::I64], Some(tail));
        for incompatible in [
            interner.intern_step(8, 1, &current, vec![types::I64], Some(tail)),
            interner.intern_step(7, 2, &current, vec![types::I64], Some(tail)),
            interner.intern_step(7, 1, &current, vec![types::I32], Some(tail)),
            interner.intern_step(7, 1, &current, vec![types::I64], None),
        ] {
            assert_ne!(original, incompatible);
        }
    }

    #[test]
    fn cleanup_suffix_bucket_collision_keeps_exact_states_distinct() {
        let mut interner = PartitionCleanupSuffixInterner::default();
        let first = cleanup_eliminator("First");
        let first_id = interner.intern_step(7, 0, &first, vec![types::I64], None);
        let second = cleanup_eliminator("Second");
        let second_key = PartitionCleanupSuffixKey {
            checked_join_site_id: 7,
            terminal_distance: 0,
            current: partition_eliminator_key(&second),
            capture_field_types: vec![types::I64],
            successor: None,
        };
        let second_bucket = partition_static_bucket(&second_key);
        interner.by_bucket.clear();
        interner
            .by_bucket
            .insert((second_bucket.hash, second_bucket.bytes), vec![first_id]);
        let second_id = interner.intern_step(7, 0, &second, vec![types::I64], None);
        assert_ne!(first_id, second_id);
        assert_eq!(interner.definitions.len(), 2);
    }

    #[test]
    fn cleanup_transition_ledger_rejects_drop_replay_and_swap() {
        let first = PartitionCleanupSuffixId(1);
        let second = PartitionCleanupSuffixId(2);

        let mut valid = PartitionCleanupTransitionLedger::default();
        let authority = valid.mint(None, first, 3).unwrap();
        valid.consume(authority, None, first, 3).unwrap();
        valid.require_complete().unwrap();

        let mut dropped = PartitionCleanupTransitionLedger::default();
        let _authority = dropped.mint(None, first, 3).unwrap();
        assert!(dropped.require_complete().is_err());

        let mut replayed = PartitionCleanupTransitionLedger::default();
        let authority = replayed.mint(None, first, 3).unwrap();
        let duplicate = PartitionCleanupTransitionAuthority {
            descriptor: authority.descriptor,
        };
        replayed.consume(authority, None, first, 3).unwrap();
        assert!(replayed.consume(duplicate, None, first, 3).is_err());

        let mut swapped = PartitionCleanupTransitionLedger::default();
        let authority = swapped.mint(Some(first), second, 4).unwrap();
        assert!(swapped.consume(authority, Some(second), first, 4).is_err());
    }

    fn continuation_key(
        site_id: u64,
        input_kind: ScalarMergeKind,
        logical_field_types: Vec<Type>,
    ) -> PartitionSemanticStateKey {
        let field_map = (0..logical_field_types.len()).collect();
        PartitionSemanticStateKey::ProducerKont(PartitionContinuationKey {
            checked_join: PartitionCheckedJoinIdentity {
                site_id,
                declaration: "pkg::main".to_string(),
                checked_occurrence_path: vec![site_id],
                checked_result_type_fingerprint: 17,
                occurrence_binding_fingerprint: 19,
                runtime_frame_fingerprint: 23,
                answer_kind: crate::NativeJoinAnswerKindV1::ExitCode,
            },
            input_kind,
            outer_return_kind: ScalarMergeKind::ExitCode,
            static_bucket: partition_static_bucket(&("process-exit", site_id)),
            static_key: Arc::new(PartitionContinuationStaticKey {
                action: PartitionProducerKontActionKey::ApplyActiveEliminators {
                    value: partition_lowered_shape_key(PartitionLoweredShape::ProcessExitStatus),
                    activation: ContinuationActivationId(1),
                    cursor: ContinuationCursorId(1),
                    pending: Vec::new(),
                    selected_ancestry: Vec::new(),
                    selected_scope: None,
                    selected_lineage: Vec::new(),
                },
                successor: None,
            }),
            field_types: logical_field_types,
            field_map,
        })
    }

    #[test]
    fn continuation_diamond_and_payload_variants_share_one_definition() {
        let mut interner = PartitionContinuationInterner::default();
        let budget = PartitionAggregateBudget::PRODUCTION;
        let key = continuation_key(7, ScalarMergeKind::Int, vec![types::I64, types::I64]);
        assert!(interner.lookup(&key, budget).unwrap().is_none());
        let (state_id, state) = interner
            .reserve(key.clone(), FuncId::from_u32(3), 3, budget)
            .unwrap();
        let (reused_id, reused) = interner
            .lookup(&key, budget)
            .unwrap()
            .expect("second predecessor reuses the reserved state");
        assert_eq!(reused_id, state_id);
        assert_eq!(reused.function, state.function);
        interner.begin_emitting(state_id).unwrap();
        interner.finish_definition(state_id).unwrap();
        interner.require_complete().unwrap();
        assert_eq!(interner.counts(), (1, 2, 1));
    }

    #[test]
    fn continuation_bucket_collision_keeps_exact_descriptors_distinct() {
        let mut interner = PartitionContinuationInterner::default();
        let budget = PartitionAggregateBudget::PRODUCTION;
        let first = continuation_key(7, ScalarMergeKind::Int, vec![types::I64]);
        let mut colliding = first.clone();
        let (first_bucket, colliding_bucket, colliding_key) = match (&first, &mut colliding) {
            (
                PartitionSemanticStateKey::ProducerKont(first),
                PartitionSemanticStateKey::ProducerKont(colliding),
            ) => (
                first.static_bucket,
                &mut colliding.static_bucket,
                &mut colliding.static_key,
            ),
            _ => unreachable!("test keys are resume states"),
        };
        *colliding_bucket = first_bucket;
        *colliding_key = Arc::new(PartitionContinuationStaticKey {
            action: PartitionProducerKontActionKey::ApplyActiveEliminators {
                value: partition_lowered_shape_key(PartitionLoweredShape::Bool),
                activation: ContinuationActivationId(1),
                cursor: ContinuationCursorId(1),
                pending: Vec::new(),
                selected_ancestry: Vec::new(),
                selected_scope: None,
                selected_lineage: Vec::new(),
            },
            successor: None,
        });

        interner.lookup(&first, budget).unwrap();
        interner
            .reserve(first, FuncId::from_u32(3), 3, budget)
            .unwrap();
        assert!(
            interner.lookup(&colliding, budget).unwrap().is_none(),
            "a bucket collision must not alias exact semantic descriptors"
        );
        interner
            .reserve(colliding, FuncId::from_u32(4), 4, budget)
            .unwrap();
        assert_eq!(interner.counts().0, 2);
    }

    #[test]
    fn reusable_state_contract_accepts_two_affine_call_edges_one_definition() {
        let mut interner = PartitionContinuationInterner::default();
        let budget = PartitionAggregateBudget::PRODUCTION;
        let key = continuation_key(7, ScalarMergeKind::Int, vec![types::I64]);
        let contract = key.return_contract();
        interner.lookup(&key, budget).unwrap();
        let (state_id, state) = interner
            .reserve(key, FuncId::from_u32(3), 3, budget)
            .unwrap();
        interner
            .validate_call_contract(state_id, &contract)
            .unwrap();
        interner
            .validate_call_contract(state_id, &contract)
            .unwrap();

        let mut edges = PartitionBranchReturnLedger::default();
        for edge_index in 0..2 {
            let authority = edges
                .mint(7, edge_index, state.helper_index, contract.required_kind)
                .unwrap();
            edges
                .consume(authority, state.helper_index, contract.required_kind)
                .unwrap();
        }
        edges.require_complete().unwrap();
        interner.begin_emitting(state_id).unwrap();
        interner.finish_definition(state_id).unwrap();
        interner.require_complete().unwrap();
        assert_eq!(interner.counts(), (1, 1, 1));
    }

    #[test]
    fn continuation_key_rejects_kind_schema_and_occurrence_aliases() {
        let mut interner = PartitionContinuationInterner::default();
        let budget = PartitionAggregateBudget::PRODUCTION;
        let original = continuation_key(7, ScalarMergeKind::Int, vec![types::I64, types::I64]);
        interner.lookup(&original, budget).expect("edge accounts");
        interner
            .reserve(original, FuncId::from_u32(3), 3, budget)
            .unwrap();
        for incompatible in [
            continuation_key(8, ScalarMergeKind::Int, vec![types::I64, types::I64]),
            continuation_key(7, ScalarMergeKind::Bool, vec![types::I64, types::I64]),
            continuation_key(7, ScalarMergeKind::Int, vec![types::I64]),
        ] {
            assert!(interner.lookup(&incompatible, budget).unwrap().is_none());
        }
    }

    #[test]
    fn continuation_cycle_reuses_emitting_state_and_duplicate_reservation_rejects() {
        let mut interner = PartitionContinuationInterner::default();
        let budget = PartitionAggregateBudget::PRODUCTION;
        let key = continuation_key(7, ScalarMergeKind::Int, vec![types::I64, types::I64]);
        interner.lookup(&key, budget).unwrap();
        let (state_id, _) = interner
            .reserve(key.clone(), FuncId::from_u32(3), 3, budget)
            .unwrap();
        interner.begin_emitting(state_id).unwrap();
        let (_, backedge) = interner
            .lookup(&key, budget)
            .unwrap()
            .expect("cycle resolves to the emitting state");
        assert_eq!(backedge.lifecycle, PartitionStateLifecycle::Emitting);
        assert!(
            interner
                .reserve(key, FuncId::from_u32(4), 4, budget)
                .is_err(),
            "bypassing lookup must not define a duplicate helper"
        );
        interner.finish_definition(state_id).unwrap();
        interner.require_complete().unwrap();
    }

    #[test]
    fn continuation_aggregate_ceiling_fails_before_unbounded_planning() {
        let mut interner = PartitionContinuationInterner::default();
        let budget = PartitionAggregateBudget {
            max_states: 1,
            max_edges: 1,
            max_helpers: 1,
        };
        let first = continuation_key(7, ScalarMergeKind::Int, vec![types::I64]);
        interner.lookup(&first, budget).unwrap();
        interner
            .reserve(first, FuncId::from_u32(0), 0, budget)
            .unwrap();
        let second = continuation_key(8, ScalarMergeKind::Int, vec![types::I64]);
        assert!(interner.lookup(&second, budget).is_err());
        assert!(interner
            .reserve(second, FuncId::from_u32(1), 1, budget)
            .is_err());
    }

    fn exit_authority(
        ledger: &mut PartitionBranchReturnLedger,
        edge_index: u64,
        helper_index: usize,
    ) -> PartitionBranchReturnAuthority {
        ledger
            .mint(7, edge_index, helper_index, ScalarMergeKind::ExitCode)
            .expect("test authority mints")
    }

    #[test]
    fn partition_budget_accepts_boundary_and_rejects_each_excess() {
        let budget = PartitionBudget {
            max_values: 3,
            max_instructions: 5,
            max_blocks: 2,
        };
        assert!(budget
            .check(PartitionFunctionMeasure {
                values: 3,
                instructions: 5,
                blocks: 2,
            })
            .is_ok());
        for measure in [
            PartitionFunctionMeasure {
                values: 4,
                instructions: 5,
                blocks: 2,
            },
            PartitionFunctionMeasure {
                values: 3,
                instructions: 6,
                blocks: 2,
            },
            PartitionFunctionMeasure {
                values: 3,
                instructions: 5,
                blocks: 3,
            },
        ] {
            assert!(budget.check(measure).is_err());
        }
    }

    #[test]
    fn partition_helper_abi_accepts_checked_scalar_pairs_only() {
        for kind in [
            ScalarMergeKind::Int,
            ScalarMergeKind::Bool,
            ScalarMergeKind::StructuralNat,
            ScalarMergeKind::ExitCode,
        ] {
            assert!(partition_helper_return_kind_is_admissible(kind));
        }
        assert!(!partition_helper_return_kind_is_admissible(
            ScalarMergeKind::RecursiveBackedge
        ));
    }

    #[test]
    fn partition_frame_layout_stores_each_ssa_value_once() {
        let mut function = Function::new();
        let mut context = FunctionBuilderContext::new();
        let (first, second, values, field_types, field_map) = {
            let mut builder = FunctionBuilder::new(&mut function, &mut context);
            let entry = builder.create_block();
            builder.switch_to_block(entry);
            let first = builder.ins().iconst(types::I64, 7);
            let second = builder.ins().iconst(types::I64, 9);
            let logical = [first, first, second, first];
            let (values, field_types, field_map) = partition_frame_layout(&builder, &logical);
            builder.ins().return_(&[]);
            builder.seal_all_blocks();
            builder.finalize();
            (first, second, values, field_types, field_map)
        };
        assert_eq!(values, vec![first, second]);
        assert_eq!(field_types, vec![types::I64, types::I64]);
        assert_eq!(field_map, vec![0, 0, 1, 0]);
        assert_eq!(
            expand_partition_frame_values(&values, &field_map).expect("map is exact"),
            vec![first, first, second, first]
        );
    }

    #[test]
    fn partition_branch_return_ledger_rejects_drop_replay_and_swap() {
        let mut dropped = PartitionBranchReturnLedger::default();
        let _dropped = exit_authority(&mut dropped, 0, 3);
        assert!(dropped.require_complete().is_err());

        let mut replayed = PartitionBranchReturnLedger::default();
        let authority = exit_authority(&mut replayed, 0, 3);
        let replay = PartitionBranchReturnAuthority {
            descriptor: authority.descriptor,
        };
        replayed
            .consume(authority, 3, ScalarMergeKind::ExitCode)
            .expect("first affine return is consumed");
        assert!(replayed
            .consume(replay, 3, ScalarMergeKind::ExitCode)
            .is_err());

        let mut swapped_helper = PartitionBranchReturnLedger::default();
        let authority = exit_authority(&mut swapped_helper, 0, 3);
        assert!(swapped_helper
            .consume(authority, 4, ScalarMergeKind::ExitCode)
            .is_err());

        let mut swapped_edge = PartitionBranchReturnLedger::default();
        let authority = exit_authority(&mut swapped_edge, 0, 3);
        let swapped = PartitionBranchReturnAuthority {
            descriptor: PartitionBranchReturnDescriptor {
                edge_index: 1,
                ..authority.descriptor
            },
        };
        assert!(swapped_edge
            .consume(swapped, 3, ScalarMergeKind::ExitCode)
            .is_err());

        let mut swapped_kind = PartitionBranchReturnLedger::default();
        let authority = exit_authority(&mut swapped_kind, 0, 3);
        assert!(swapped_kind
            .consume(authority, 3, ScalarMergeKind::Int)
            .is_err());
    }
}

pub(super) fn partition_lowered_is_admissible(value: &Lowered) -> bool {
    match value {
        Lowered::HostResult { error, ok, .. } => {
            partition_lowered_is_admissible(error) && partition_lowered_is_admissible(ok)
        }
        Lowered::DynamicConstructor(dynamic) => dynamic
            .alternatives
            .iter()
            .flat_map(|alternative| &alternative.fields)
            .all(partition_lowered_is_admissible),
        Lowered::Constructor { args, .. } => args.iter().all(partition_lowered_is_admissible),
        Lowered::Record { fields } => fields
            .iter()
            .all(|(_, value)| partition_lowered_is_admissible(value)),
        Lowered::Closure { captures, .. } | Lowered::DeclarationClosure { captures, .. } => {
            captures.iter().all(partition_lowered_is_admissible)
        }
        Lowered::ComputationalRecursorClosure {
            residual,
            invocation,
            ..
        } => {
            partition_lowered_is_admissible(residual)
                && partition_layer_is_admissible(&invocation.selection)
                && invocation
                    .unwind
                    .later_wrappers_in_construction_order
                    .iter()
                    .all(partition_layer_is_admissible)
        }
        Lowered::RecursiveBackedge => false,
        Lowered::Int { .. }
        | Lowered::Bool { .. }
        | Lowered::ProcessExitStatus { .. }
        | Lowered::CapabilityToken { .. }
        | Lowered::ResourceToken { .. }
        | Lowered::BoundedNat(_)
        | Lowered::StructuralNat(_)
        | Lowered::ResponseBytes { .. }
        | Lowered::Bytes(_)
        | Lowered::BorrowedNativeValue { .. }
        | Lowered::BorrowedOption { .. }
        | Lowered::String(_)
        | Lowered::Trap(_) => true,
    }
}

fn partition_layer_is_admissible(layer: &ComputationalRecursorLayer) -> bool {
    layer.outer_env.iter().all(partition_lowered_is_admissible)
}

pub(super) fn partition_prefix_is_admissible(prefix: &SourcePrefixTemplate) -> bool {
    let local = match prefix {
        SourcePrefixTemplate::Terminal { .. }
        | SourcePrefixTemplate::CheckedComputationalIHInvocationReturn { .. }
        | SourcePrefixTemplate::ProjectRecord { .. } => true,
        SourcePrefixTemplate::CheckedRecursiveInvocationReturn { .. } => false,
        SourcePrefixTemplate::ReturnFromSelectedCase { .. } => true,
        SourcePrefixTemplate::LetBody { env, .. }
        | SourcePrefixTemplate::IfScrutinee { env, .. }
        | SourcePrefixTemplate::MatchScrutinee { env, .. }
        | SourcePrefixTemplate::ComputationalMatchScrutinee { env, .. }
        | SourcePrefixTemplate::CallCallee { env, .. } => {
            env.iter().all(partition_lowered_is_admissible)
        }
        SourcePrefixTemplate::ApplyRecursorSelection { layer, .. } => {
            partition_layer_is_admissible(layer)
        }
        SourcePrefixTemplate::UnwindRecursorSegment { stack, .. } => {
            stack.partition_cursor.is_some()
                || stack
                    .later_wrappers_in_construction_order
                    .iter()
                    .all(partition_layer_is_admissible)
        }
        SourcePrefixTemplate::ConstructArgument { lowered, env, .. } => lowered
            .iter()
            .chain(env)
            .all(partition_lowered_is_admissible),
        SourcePrefixTemplate::CallArgument {
            callee,
            lowered,
            env,
            ..
        } => {
            partition_lowered_is_admissible(callee)
                && lowered
                    .iter()
                    .chain(env)
                    .all(partition_lowered_is_admissible)
        }
    };
    local
        && match prefix {
            SourcePrefixTemplate::Terminal { .. } => true,
            SourcePrefixTemplate::CheckedRecursiveInvocationReturn { next, .. }
            | SourcePrefixTemplate::CheckedComputationalIHInvocationReturn { next, .. }
            | SourcePrefixTemplate::ReturnFromSelectedCase { next, .. }
            | SourcePrefixTemplate::LetBody { next, .. }
            | SourcePrefixTemplate::ApplyRecursorSelection { next, .. }
            | SourcePrefixTemplate::UnwindRecursorSegment { next, .. }
            | SourcePrefixTemplate::IfScrutinee { next, .. }
            | SourcePrefixTemplate::ConstructArgument { next, .. }
            | SourcePrefixTemplate::MatchScrutinee { next, .. }
            | SourcePrefixTemplate::ComputationalMatchScrutinee { next, .. }
            | SourcePrefixTemplate::ProjectRecord { next, .. }
            | SourcePrefixTemplate::CallCallee { next, .. }
            | SourcePrefixTemplate::CallArgument { next, .. } => {
                partition_prefix_is_admissible(next)
            }
        }
}

pub(super) fn partition_scope_is_admissible(scope: &Option<OwnedSelectedScope>) -> bool {
    scope.as_ref().is_none_or(|scope| {
        scope
            .frame
            .outer_env
            .iter()
            .all(partition_lowered_is_admissible)
    })
}

pub(super) fn append_partition_lowered_values(
    lowering: &mut Lowering<'_>,
    builder: &mut FunctionBuilder<'_>,
    value: &Lowered,
    output: &mut Vec<Value>,
) -> Result<(), CraneliftBackendError> {
    match value {
        Lowered::Int { value, known } => {
            output.push(lowering.native_int_tag(builder, *value, *known)?);
            output.push(*value);
        }
        Lowered::Bool { value, .. }
        | Lowered::ProcessExitStatus { value }
        | Lowered::CapabilityToken { value }
        | Lowered::ResourceToken { value } => output.push(*value),
        Lowered::BoundedNat(value) => output.push(value.value),
        Lowered::StructuralNat(value) => output.push(value.value),
        Lowered::ResponseBytes { pointer, len } => {
            output.push(*pointer);
            output.push(*len);
        }
        Lowered::HostResult {
            success, error, ok, ..
        } => {
            output.push(*success);
            append_partition_lowered_values(lowering, builder, error, output)?;
            append_partition_lowered_values(lowering, builder, ok, output)?;
        }
        Lowered::DynamicConstructor(dynamic) => {
            output.push(dynamic.discriminator);
            for alternative in &dynamic.alternatives {
                for field in &alternative.fields {
                    append_partition_lowered_values(lowering, builder, field, output)?;
                }
            }
        }
        Lowered::Bytes(_) | Lowered::String(_) | Lowered::Trap(_) => {}
        Lowered::BorrowedNativeValue { pointer } => output.push(*pointer),
        Lowered::BorrowedOption { present, value, .. } => {
            output.push(*present);
            output.push(*value);
        }
        Lowered::Constructor { args, .. } => {
            for argument in args {
                append_partition_lowered_values(lowering, builder, argument, output)?;
            }
        }
        Lowered::Record { fields } => {
            for (_, field) in fields {
                append_partition_lowered_values(lowering, builder, field, output)?;
            }
        }
        Lowered::Closure { captures, .. } | Lowered::DeclarationClosure { captures, .. } => {
            for capture in captures {
                append_partition_lowered_values(lowering, builder, capture, output)?;
            }
        }
        Lowered::ComputationalRecursorClosure {
            residual,
            invocation,
            ..
        } => {
            append_partition_lowered_values(lowering, builder, residual, output)?;
            output.push(invocation.resume_cursor_instance.0);
            append_partition_layer_values(lowering, builder, &invocation.selection, output)?;
            if let Some(cursor) = invocation.unwind.partition_cursor {
                output.push(cursor.capture_pointer);
            }
            if let Some(cursor) = invocation.unwind.partition_qualification {
                output.push(cursor.capture_pointer);
            }
            if let Some(cursor) = invocation.unwind.partition_open_obligation {
                output.push(cursor.capture_pointer);
            }
            if let Some(PartitionCheckedParent::Unwind(cursor)) = invocation.checked_parent {
                output.push(cursor.capture_pointer);
            }
        }
        Lowered::RecursiveBackedge => {
            return Err(unsupported(
                "NativeFunctionPartition",
                "caller-local recursive backedge is not frame-codec admissible",
            ));
        }
    }
    Ok(())
}

pub(super) fn append_partition_layer_values(
    lowering: &mut Lowering<'_>,
    builder: &mut FunctionBuilder<'_>,
    layer: &ComputationalRecursorLayer,
    output: &mut Vec<Value>,
) -> Result<(), CraneliftBackendError> {
    match layer.role {
        RecursorLayerRole::SelectsOccurrence { origin_scope, .. } => {
            output.push(origin_scope.0);
        }
        RecursorLayerRole::ExitsScope {
            origin_scope,
            scope_instance,
            parent_scope_instance,
            ..
        } => {
            output.push(origin_scope.0);
            output.push(scope_instance.0);
            if let Some(parent) = parent_scope_instance {
                output.push(parent.0);
            }
        }
    }
    for value in &layer.outer_env {
        append_partition_lowered_values(lowering, builder, value, output)?;
    }
    Ok(())
}

pub(super) fn append_partition_prefix_values(
    lowering: &mut Lowering<'_>,
    builder: &mut FunctionBuilder<'_>,
    prefix: &SourcePrefixTemplate,
    output: &mut Vec<Value>,
) -> Result<(), CraneliftBackendError> {
    match prefix {
        SourcePrefixTemplate::Terminal { .. }
        | SourcePrefixTemplate::CheckedRecursiveInvocationReturn { .. }
        | SourcePrefixTemplate::CheckedComputationalIHInvocationReturn { .. }
        | SourcePrefixTemplate::ProjectRecord { .. } => {}
        SourcePrefixTemplate::ReturnFromSelectedCase {
            delimiter,
            parent_capture,
            exit_transition,
            ..
        } => {
            output.push(delimiter.activation_instance.0);
            output.push(delimiter.cursor_instance.0);
            output.push(delimiter.scope_instance.0);
            let parent = parent_capture.as_ref().ok_or_else(|| {
                unsupported(
                    "NativeControlCellV1",
                    "selected return has no immediate parent capture",
                )
            })?;
            output.push(parent.activation_instance.0);
            output.push(parent.cursor_instance.0);
            append_partition_eliminator_values(lowering, builder, &parent.pending, output)?;
            append_partition_scope_values(lowering, builder, &parent.selected_scope, output)?;
            if let Some(transition) = exit_transition {
                output.push(
                    transition
                        .target
                        .expect("source exit owns its recursor cell")
                        .capture_pointer,
                );
                output.push(
                    transition
                        .exit_obligation
                        .expect("source exit owns its obligation cell")
                        .capture_pointer,
                );
                let invocation = lowering
                    .invocation_pointer
                    .expect("source exit capture owns an invocation pointer");
                let pointer_type = builder.func.dfg.value_type(invocation);
                output.push(transition.exit_obligation_successor.map_or_else(
                    || builder.ins().iconst(pointer_type, 0),
                    |cursor| cursor.capture_pointer,
                ));
            }
        }
        SourcePrefixTemplate::LetBody { env, .. }
        | SourcePrefixTemplate::IfScrutinee { env, .. }
        | SourcePrefixTemplate::MatchScrutinee { env, .. }
        | SourcePrefixTemplate::ComputationalMatchScrutinee { env, .. }
        | SourcePrefixTemplate::CallCallee { env, .. } => {
            for value in env {
                append_partition_lowered_values(lowering, builder, value, output)?;
            }
        }
        SourcePrefixTemplate::ApplyRecursorSelection { layer, .. } => {
            append_partition_layer_values(lowering, builder, layer, output)?;
        }
        SourcePrefixTemplate::UnwindRecursorSegment {
            stack,
            resume_cursor_instance,
            ..
        } => {
            output.push(resume_cursor_instance.0);
            if let Some(cursor) = stack.partition_cursor {
                output.push(cursor.capture_pointer);
            } else {
                for layer in &stack.later_wrappers_in_construction_order {
                    append_partition_layer_values(lowering, builder, layer, output)?;
                }
            }
            if let Some(cursor) = stack.partition_qualification {
                output.push(cursor.capture_pointer);
            }
            if let Some(cursor) = stack.partition_open_obligation {
                output.push(cursor.capture_pointer);
            }
        }
        SourcePrefixTemplate::ConstructArgument { lowered, env, .. } => {
            for value in lowered.iter().chain(env) {
                append_partition_lowered_values(lowering, builder, value, output)?;
            }
        }
        SourcePrefixTemplate::CallArgument {
            callee,
            lowered,
            env,
            ..
        } => {
            append_partition_lowered_values(lowering, builder, callee, output)?;
            for value in lowered.iter().chain(env) {
                append_partition_lowered_values(lowering, builder, value, output)?;
            }
        }
    }
    match prefix {
        SourcePrefixTemplate::Terminal { .. } => Ok(()),
        SourcePrefixTemplate::CheckedRecursiveInvocationReturn { next, .. }
        | SourcePrefixTemplate::CheckedComputationalIHInvocationReturn { next, .. }
        | SourcePrefixTemplate::ReturnFromSelectedCase { next, .. }
        | SourcePrefixTemplate::LetBody { next, .. }
        | SourcePrefixTemplate::ApplyRecursorSelection { next, .. }
        | SourcePrefixTemplate::UnwindRecursorSegment { next, .. }
        | SourcePrefixTemplate::IfScrutinee { next, .. }
        | SourcePrefixTemplate::ConstructArgument { next, .. }
        | SourcePrefixTemplate::MatchScrutinee { next, .. }
        | SourcePrefixTemplate::ComputationalMatchScrutinee { next, .. }
        | SourcePrefixTemplate::ProjectRecord { next, .. }
        | SourcePrefixTemplate::CallCallee { next, .. }
        | SourcePrefixTemplate::CallArgument { next, .. } => {
            append_partition_prefix_values(lowering, builder, next, output)
        }
    }
}

pub(super) fn append_partition_scope_values(
    lowering: &mut Lowering<'_>,
    builder: &mut FunctionBuilder<'_>,
    scope: &Option<OwnedSelectedScope>,
    output: &mut Vec<Value>,
) -> Result<(), CraneliftBackendError> {
    if let Some(scope) = scope {
        output.push(scope.scope_instance.0);
        if let Some(parent) = scope.parent_scope_instance {
            output.push(parent.0);
        }
        for value in &scope.frame.outer_env {
            append_partition_lowered_values(lowering, builder, value, output)?;
        }
    }
    Ok(())
}

fn next_partition_value(
    values: &mut impl Iterator<Item = Value>,
) -> Result<Value, CraneliftBackendError> {
    values.next().ok_or_else(|| {
        unsupported(
            "NativeFunctionPartition",
            "private partition frame is truncated",
        )
    })
}

pub(super) fn rebuild_partition_lowered(
    value: &mut Lowered,
    values: &mut impl Iterator<Item = Value>,
    native_int_tags: &mut BTreeMap<Value, Value>,
) -> Result<(), CraneliftBackendError> {
    match value {
        Lowered::Int { value, known } => {
            let tag = next_partition_value(values)?;
            *value = next_partition_value(values)?;
            native_int_tags.insert(*value, tag);
            if known.is_none() {
                *known = None;
            }
        }
        Lowered::Bool { value, known } => {
            *value = next_partition_value(values)?;
            *known = None;
        }
        Lowered::ProcessExitStatus { value }
        | Lowered::CapabilityToken { value }
        | Lowered::ResourceToken { value } => *value = next_partition_value(values)?,
        Lowered::BoundedNat(value) => {
            value.value = next_partition_value(values)?;
        }
        Lowered::StructuralNat(value) => value.value = next_partition_value(values)?,
        Lowered::ResponseBytes { pointer, len } => {
            *pointer = next_partition_value(values)?;
            *len = next_partition_value(values)?;
        }
        Lowered::HostResult {
            success, error, ok, ..
        } => {
            *success = next_partition_value(values)?;
            rebuild_partition_lowered(error, values, native_int_tags)?;
            rebuild_partition_lowered(ok, values, native_int_tags)?;
        }
        Lowered::DynamicConstructor(dynamic) => {
            dynamic.discriminator = next_partition_value(values)?;
            for alternative in &mut dynamic.alternatives {
                for field in &mut alternative.fields {
                    rebuild_partition_lowered(field, values, native_int_tags)?;
                }
            }
        }
        Lowered::Bytes(_) | Lowered::String(_) | Lowered::Trap(_) => {}
        Lowered::BorrowedNativeValue { pointer } => {
            *pointer = next_partition_value(values)?;
        }
        Lowered::BorrowedOption { present, value, .. } => {
            *present = next_partition_value(values)?;
            *value = next_partition_value(values)?;
        }
        Lowered::Constructor { args, .. } => {
            for argument in args {
                rebuild_partition_lowered(argument, values, native_int_tags)?;
            }
        }
        Lowered::Record { fields } => {
            for (_, field) in fields {
                rebuild_partition_lowered(field, values, native_int_tags)?;
            }
        }
        Lowered::Closure { captures, .. } | Lowered::DeclarationClosure { captures, .. } => {
            for capture in captures {
                rebuild_partition_lowered(capture, values, native_int_tags)?;
            }
        }
        Lowered::ComputationalRecursorClosure {
            residual,
            invocation,
            ..
        } => {
            rebuild_partition_lowered(residual, values, native_int_tags)?;
            invocation.resume_cursor_instance = ControlCursorRef(next_partition_value(values)?);
            rebuild_partition_layer(&mut invocation.selection, values, native_int_tags)?;
            if let Some(cursor) = &mut invocation.unwind.partition_cursor {
                cursor.capture_pointer = next_partition_value(values)?;
            }
            if let Some(cursor) = &mut invocation.unwind.partition_qualification {
                cursor.capture_pointer = next_partition_value(values)?;
            }
            if let Some(cursor) = &mut invocation.unwind.partition_open_obligation {
                cursor.capture_pointer = next_partition_value(values)?;
            }
            if let Some(PartitionCheckedParent::Unwind(cursor)) = &mut invocation.checked_parent {
                cursor.capture_pointer = next_partition_value(values)?;
            }
        }
        Lowered::RecursiveBackedge => {
            return Err(unsupported(
                "NativeFunctionPartition",
                "ineligible value reached the private partition frame decoder",
            ));
        }
    }
    Ok(())
}

pub(super) fn rebuild_partition_layer(
    layer: &mut ComputationalRecursorLayer,
    values: &mut impl Iterator<Item = Value>,
    native_int_tags: &mut BTreeMap<Value, Value>,
) -> Result<(), CraneliftBackendError> {
    match &mut layer.role {
        RecursorLayerRole::SelectsOccurrence { origin_scope, .. } => {
            *origin_scope = ScopeInstanceRef(next_partition_value(values)?);
        }
        RecursorLayerRole::ExitsScope {
            origin_scope,
            scope_instance,
            parent_scope_instance,
            ..
        } => {
            *origin_scope = ScopeInstanceRef(next_partition_value(values)?);
            *scope_instance = ScopeInstanceRef(next_partition_value(values)?);
            if parent_scope_instance.is_some() {
                *parent_scope_instance = Some(ScopeInstanceRef(next_partition_value(values)?));
            }
        }
    }
    for value in &mut layer.outer_env {
        rebuild_partition_lowered(value, values, native_int_tags)?;
    }
    Ok(())
}

pub(super) fn rebuild_partition_prefix(
    prefix: &mut SourcePrefixTemplate,
    values: &mut impl Iterator<Item = Value>,
    native_int_tags: &mut BTreeMap<Value, Value>,
) -> Result<(), CraneliftBackendError> {
    match prefix {
        SourcePrefixTemplate::Terminal { .. }
        | SourcePrefixTemplate::CheckedRecursiveInvocationReturn { .. }
        | SourcePrefixTemplate::CheckedComputationalIHInvocationReturn { .. }
        | SourcePrefixTemplate::ProjectRecord { .. } => {}
        SourcePrefixTemplate::ReturnFromSelectedCase {
            delimiter,
            parent_capture,
            exit_transition,
            ..
        } => {
            delimiter.activation_instance = ActivationInstanceRef(next_partition_value(values)?);
            delimiter.cursor_instance = ControlCursorRef(next_partition_value(values)?);
            delimiter.scope_instance = ScopeInstanceRef(next_partition_value(values)?);
            let parent = parent_capture.as_mut().ok_or_else(|| {
                unsupported(
                    "NativeControlCellV1",
                    "selected return has no immediate parent capture",
                )
            })?;
            parent.activation_instance = ActivationInstanceRef(next_partition_value(values)?);
            parent.cursor_instance = ControlCursorRef(next_partition_value(values)?);
            rebuild_partition_eliminators(&mut parent.pending, values, native_int_tags)?;
            rebuild_partition_scope(&mut parent.selected_scope, values, native_int_tags)?;
            if let Some(transition) = exit_transition {
                transition
                    .target
                    .as_mut()
                    .expect("source exit owns its recursor cell")
                    .capture_pointer = next_partition_value(values)?;
                transition
                    .exit_obligation
                    .as_mut()
                    .expect("source exit owns its obligation cell")
                    .capture_pointer = next_partition_value(values)?;
                let successor_pointer = next_partition_value(values)?;
                if let Some(successor) = &mut transition.exit_obligation_successor {
                    successor.capture_pointer = successor_pointer;
                }
            }
        }
        SourcePrefixTemplate::LetBody { env, .. }
        | SourcePrefixTemplate::IfScrutinee { env, .. }
        | SourcePrefixTemplate::MatchScrutinee { env, .. }
        | SourcePrefixTemplate::ComputationalMatchScrutinee { env, .. }
        | SourcePrefixTemplate::CallCallee { env, .. } => {
            for value in env {
                rebuild_partition_lowered(value, values, native_int_tags)?;
            }
        }
        SourcePrefixTemplate::ApplyRecursorSelection { layer, .. } => {
            rebuild_partition_layer(layer, values, native_int_tags)?;
        }
        SourcePrefixTemplate::UnwindRecursorSegment {
            stack,
            resume_cursor_instance,
            ..
        } => {
            *resume_cursor_instance = ControlCursorRef(next_partition_value(values)?);
            if let Some(cursor) = &mut stack.partition_cursor {
                cursor.capture_pointer = next_partition_value(values)?;
            } else {
                for layer in &mut stack.later_wrappers_in_construction_order {
                    rebuild_partition_layer(layer, values, native_int_tags)?;
                }
            }
            if let Some(cursor) = &mut stack.partition_qualification {
                cursor.capture_pointer = next_partition_value(values)?;
            }
            if let Some(cursor) = &mut stack.partition_open_obligation {
                cursor.capture_pointer = next_partition_value(values)?;
            }
        }
        SourcePrefixTemplate::ConstructArgument { lowered, env, .. } => {
            for value in lowered.iter_mut().chain(env) {
                rebuild_partition_lowered(value, values, native_int_tags)?;
            }
        }
        SourcePrefixTemplate::CallArgument {
            callee,
            lowered,
            env,
            ..
        } => {
            rebuild_partition_lowered(callee, values, native_int_tags)?;
            for value in lowered.iter_mut().chain(env) {
                rebuild_partition_lowered(value, values, native_int_tags)?;
            }
        }
    }
    match prefix {
        SourcePrefixTemplate::Terminal { .. } => Ok(()),
        SourcePrefixTemplate::CheckedRecursiveInvocationReturn { next, .. }
        | SourcePrefixTemplate::CheckedComputationalIHInvocationReturn { next, .. }
        | SourcePrefixTemplate::ReturnFromSelectedCase { next, .. }
        | SourcePrefixTemplate::LetBody { next, .. }
        | SourcePrefixTemplate::ApplyRecursorSelection { next, .. }
        | SourcePrefixTemplate::UnwindRecursorSegment { next, .. }
        | SourcePrefixTemplate::IfScrutinee { next, .. }
        | SourcePrefixTemplate::ConstructArgument { next, .. }
        | SourcePrefixTemplate::MatchScrutinee { next, .. }
        | SourcePrefixTemplate::ComputationalMatchScrutinee { next, .. }
        | SourcePrefixTemplate::ProjectRecord { next, .. }
        | SourcePrefixTemplate::CallCallee { next, .. }
        | SourcePrefixTemplate::CallArgument { next, .. } => {
            rebuild_partition_prefix(next, values, native_int_tags)
        }
    }
}

pub(super) fn rebuild_partition_scope(
    scope: &mut Option<OwnedSelectedScope>,
    values: &mut impl Iterator<Item = Value>,
    native_int_tags: &mut BTreeMap<Value, Value>,
) -> Result<(), CraneliftBackendError> {
    if let Some(scope) = scope {
        scope.scope_instance = ScopeInstanceRef(next_partition_value(values)?);
        if scope.parent_scope_instance.is_some() {
            scope.parent_scope_instance = Some(ScopeInstanceRef(next_partition_value(values)?));
        }
        for value in &mut scope.frame.outer_env {
            rebuild_partition_lowered(value, values, native_int_tags)?;
        }
    }
    Ok(())
}

pub(super) fn instantiate_partition_prefix<'a>(
    prefix: SourcePrefixTemplate,
    terminal: SourceContinuationTerminal<'a>,
) -> SourceContinuation<'a> {
    match prefix {
        SourcePrefixTemplate::Terminal { .. } => SourceContinuation::Terminal(terminal),
        SourcePrefixTemplate::CheckedRecursiveInvocationReturn { instance, next } => {
            SourceContinuation::CheckedRecursiveInvocationReturn {
                instance,
                next: Box::new(instantiate_partition_prefix(*next, terminal)),
            }
        }
        SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
            call_template_id,
            next,
        } => SourceContinuation::CheckedComputationalIHInvocationReturn {
            call_template_id,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::ReturnFromSelectedCase {
            delimiter, next, ..
        } => SourceContinuation::ReturnFromSelectedCase {
            delimiter,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::LetBody { body, env, next } => SourceContinuation::LetBody {
            body,
            env,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::ApplyRecursorSelection { layer, next } => {
            SourceContinuation::ApplyRecursorSelection {
                layer,
                next: Box::new(instantiate_partition_prefix(*next, terminal)),
            }
        }
        SourcePrefixTemplate::UnwindRecursorSegment {
            stack,
            resume_cursor,
            resume_cursor_instance,
            next,
        } => SourceContinuation::UnwindRecursorSegment {
            stack,
            resume_cursor,
            resume_cursor_instance,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::IfScrutinee {
            then_expr,
            else_expr,
            env,
            next,
        } => SourceContinuation::IfScrutinee {
            then_expr,
            else_expr,
            env,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::ConstructArgument {
            constructor,
            remaining,
            lowered,
            env,
            next,
        } => SourceContinuation::ConstructArgument {
            constructor,
            remaining,
            lowered,
            env,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::MatchScrutinee {
            cases,
            default,
            env,
            next,
        } => SourceContinuation::MatchScrutinee {
            cases,
            default,
            env,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::ComputationalMatchScrutinee {
            cases,
            default,
            env,
            provenance,
            checked_frame_id,
            answer_route,
            next,
        } => SourceContinuation::ComputationalMatchScrutinee {
            cases,
            default,
            env,
            provenance,
            checked_frame_id,
            answer_route,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::ProjectRecord { field, next } => SourceContinuation::ProjectRecord {
            field,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::CallCallee { args, env, next } => SourceContinuation::CallCallee {
            args,
            env,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
        SourcePrefixTemplate::CallArgument {
            callee,
            remaining,
            lowered,
            env,
            next,
        } => SourceContinuation::CallArgument {
            callee,
            remaining,
            lowered,
            env,
            next: Box::new(instantiate_partition_prefix(*next, terminal)),
        },
    }
}
