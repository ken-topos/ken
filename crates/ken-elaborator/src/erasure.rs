//! Erasure boundary from `CheckedCorePackage v0` to Ken runtime IR.
//!
//! This module consumes only the checked-core package artifact. Source identity
//! may remain in the package envelope for diagnostics and provenance, but it is
//! never an input to runtime meaning here.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ken_runtime::*;

use crate::checked_core::{
    self, consume_checked_core_package_for_target, validate_checked_core_package,
    CheckedCoreBodyTerm, CheckedCoreBodyViewError, CheckedCoreBodyViewSelection,
    CheckedCoreLevelView, CheckedCorePackage, CheckedCorePackageError, ClassInstanceKind,
    ClassInstanceMetadata, DataMetadata, EffectBoundary, EffectsForeignMetadata,
    LowerabilityStatus, PartialityMetadata, PrimitiveMetadata, RecordSigmaMetadata,
    RecursionMetadata, StableSymbol,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErasureError {
    InvalidPackage(CheckedCorePackageError),
    ProofErasureBoundaryWitness(ProofErasureBoundaryWitnessError),
    ExpressionLowering {
        symbol: StableSymbol,
        lane: &'static str,
        reason: String,
    },
    UnsupportedErasure {
        symbol: StableSymbol,
        reason: String,
    },
    MissingRuntimeMetadata {
        symbol: StableSymbol,
        section: &'static str,
    },
}

impl fmt::Display for ErasureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErasureError::InvalidPackage(err) => err.fmt(f),
            ErasureError::ProofErasureBoundaryWitness(err) => err.fmt(f),
            ErasureError::ExpressionLowering {
                symbol,
                lane,
                reason,
            } => write!(
                f,
                "unsupported checked-core expression lowering for {symbol} [{lane}]: {reason}"
            ),
            ErasureError::UnsupportedErasure { symbol, reason } => {
                write!(f, "unsupported erasure for {symbol}: {reason}")
            }
            ErasureError::MissingRuntimeMetadata { symbol, section } => {
                write!(f, "{symbol} is missing runtime metadata section {section}")
            }
        }
    }
}

impl std::error::Error for ErasureError {}

impl From<CheckedCorePackageError> for ErasureError {
    fn from(value: CheckedCorePackageError) -> Self {
        ErasureError::InvalidPackage(value)
    }
}

impl From<ProofErasureBoundaryWitnessError> for ErasureError {
    fn from(value: ProofErasureBoundaryWitnessError) -> Self {
        ErasureError::ProofErasureBoundaryWitness(value)
    }
}

pub fn erase_checked_core_package_for_target<'a>(
    package: &CheckedCorePackage,
    target_closure: impl IntoIterator<Item = &'a StableSymbol>,
) -> Result<RuntimeProgram, ErasureError> {
    let targets: Vec<StableSymbol> = target_closure.into_iter().cloned().collect();
    erase_checked_package_with_host_root(package, targets, None, None)
}

fn erase_checked_package_with_host_root(
    package: &CheckedCorePackage,
    mut targets: Vec<StableSymbol>,
    host_root: Option<(&StableSymbol, &CheckedHostSpineV1)>,
    mut native_plans: Option<&mut NativeLoweringPlanCollector>,
) -> Result<RuntimeProgram, ErasureError> {
    validate_checked_core_package(package)?;
    let requested_targets = targets.clone();
    let mut prelowered = BTreeMap::new();
    if let Some((root, spine)) = host_root {
        let root_kind = lower_checked_host_root(
            package,
            &requested_targets,
            root,
            spine,
            native_plans.as_deref_mut(),
        )?;
        let mut executable = BTreeSet::from([root.clone()]);
        let mut queue = runtime_declaration_refs_in_kind(&root_kind)
            .into_iter()
            .filter_map(|reference| {
                requested_targets
                    .iter()
                    .find(|symbol| {
                        symbol.to_string() == reference
                            && admitted_recursive_member(&package.artifact.semantic, symbol)
                    })
                    .cloned()
            })
            .collect::<Vec<_>>();
        prelowered.insert(root.clone(), root_kind);
        while let Some(symbol) = queue.pop() {
            if !executable.insert(symbol.clone()) {
                continue;
            }
            let declaration = match lower_checked_host_declaration(
                package,
                &requested_targets,
                &symbol,
                spine,
                native_plans.as_deref_mut(),
            ) {
                Ok(declaration) => declaration,
                Err(error)
                    if matches!(
                        &error,
                        ErasureError::ExpressionLowering { lane, .. }
                            if *lane == "unrecognized_checked_host_computation"
                    ) =>
                {
                    lower_symbol(package, &requested_targets, &symbol)?
                }
                Err(error) => return Err(error),
            };
            queue.extend(
                runtime_declaration_refs_in_kind(&declaration.kind)
                    .into_iter()
                    .filter_map(|reference| {
                        requested_targets
                            .iter()
                            .find(|symbol| {
                                symbol.to_string() == reference
                                    && admitted_recursive_member(&package.artifact.semantic, symbol)
                            })
                            .cloned()
                    }),
            );
            prelowered.insert(symbol, declaration.kind);
        }
        targets = executable.into_iter().collect();
    }
    consume_checked_core_package_for_target(package, targets.iter())?;
    reject_reachable_unsupported(package, &targets)?;

    let semantic = &package.artifact.semantic;
    let metadata = RuntimeMetadata {
        obligations: symbol_bytes_map(&semantic.obligations),
        obligation_metadata: obligation_metadata_map(&semantic.obligation_metadata),
        assumptions: symbol_bytes_map(&semantic.assumptions),
        assumption_trust_metadata: assumption_trust_metadata_map(
            &semantic.assumption_trust_metadata,
        ),
        trusted_base_delta: symbol_bytes_map(&semantic.trusted_base_delta),
        dependency_semantic_hashes: semantic
            .dependency_semantic_hashes
            .iter()
            .map(|(symbol, hash)| (symbol.to_string(), hash.clone()))
            .collect(),
        lowerability: lowerability_map(&semantic.lowerability),
        unsupported: symbol_bytes_map(&semantic.unsupported),
        runtime_declaration_targets: targets.iter().map(ToString::to_string).collect(),
        checked_core: checked_core_metadata(semantic),
        runtime_checks: runtime_checks_for_targets(package, &targets),
        capabilities: capabilities_for_targets(package, &targets),
        effects: effects_for_targets(package, &targets),
    };

    let mut declarations = Vec::new();
    for target in &targets {
        if let Some(kind) = prelowered.remove(target) {
            declarations.push(RuntimeDeclaration {
                symbol: target.to_string(),
                kind,
                metadata: metadata_for_symbol(package, target),
            });
        } else {
            declarations.push(lower_symbol(package, &targets, target)?);
        }
    }

    Ok(RuntimeProgram {
        package_identity: package.header.package_identity.to_string(),
        core_semantic_hash: package.core_semantic_hash,
        artifact_hash: package.artifact_hash,
        erased_core: ErasedExecutableCore {
            symbols: semantic
                .symbols
                .iter()
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>(),
            metadata,
        },
        declarations,
        examples: nc5_seed_examples(),
    })
}

fn lower_checked_host_declaration(
    package: &CheckedCorePackage,
    target_closure: &[StableSymbol],
    symbol: &StableSymbol,
    spine: &CheckedHostSpineV1,
    native_plans: Option<&mut NativeLoweringPlanCollector>,
) -> Result<RuntimeDeclaration, ErasureError> {
    let semantic = &package.artifact.semantic;
    let reachable_declarations = checked_host_body_view_symbols(semantic, target_closure);
    let selection = CheckedCoreBodyViewSelection {
        package_identity: package.header.package_identity.clone(),
        package_core_semantic_hash: package.core_semantic_hash,
        package_artifact_hash: package.artifact_hash,
        target_symbol: symbol.clone(),
        reachable_declarations,
        external_symbols: external_declaration_symbols(semantic),
        dependency_semantic_hashes: semantic.dependency_semantic_hashes.clone(),
    };
    let declarations = checked_host_declaration_closure(package, &selection, symbol)?;
    let declaration = declarations.get(symbol).ok_or_else(|| {
        expression_lowering_error(
            symbol,
            "missing_expression_body_view",
            "body view did not return the recursive checked HostIO declaration",
        )
    })?;
    let mut parameter_count = 0usize;
    let mut body = &declaration.body;
    while let CheckedCoreBodyTerm::Lambda { body: inner, .. } = body {
        parameter_count += 1;
        body = inner;
    }
    if parameter_count == 0 {
        return Err(expression_lowering_error(
            symbol,
            "recursive_host_abi_shape",
            "recursive checked HostIO declaration must be a function",
        ));
    }
    let mut stack = vec![symbol.clone()];
    let body = lower_checked_host_computation(
        body,
        &declarations,
        semantic,
        &mut stack,
        symbol,
        parameter_count,
        spine,
        None,
        &[1],
        native_plans,
        None,
    )?;
    Ok(RuntimeDeclaration {
        symbol: symbol.to_string(),
        kind: RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::Closure {
                captures: Vec::new(),
                params: (0..parameter_count)
                    .map(|index| format!("arg{index}"))
                    .collect(),
                body: Box::new(body),
            },
        },
        metadata: metadata_for_symbol(package, symbol),
    })
}

fn checked_host_declaration_closure(
    package: &CheckedCorePackage,
    selection: &CheckedCoreBodyViewSelection,
    root: &StableSymbol,
) -> Result<BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>, ErasureError> {
    let semantic = &package.artifact.semantic;
    let mut declarations = BTreeMap::new();
    let mut queue = vec![root.clone()];
    while let Some(symbol) = queue.pop() {
        if declarations.contains_key(&symbol) {
            continue;
        }
        let mut declaration_selection = selection.clone();
        declaration_selection.target_symbol = symbol.clone();
        let declaration = checked_core::checked_core_declaration_body_view(
            package,
            &declaration_selection,
            &symbol,
        )
        .map_err(|error| expression_view_error(root, error))?;
        let mut references = BTreeSet::new();
        collect_checked_body_declaration_refs(&declaration.body, &mut references);
        queue.extend(references.into_iter().filter(|reference| {
            semantic.declarations.contains_key(reference)
                && selection.reachable_declarations.contains(reference)
        }));
        declarations.insert(symbol, declaration);
    }
    Ok(declarations)
}

fn collect_checked_body_declaration_refs(
    term: &CheckedCoreBodyTerm,
    output: &mut BTreeSet<StableSymbol>,
) {
    match term {
        CheckedCoreBodyTerm::DirectDeclarationCall { symbol, .. }
        | CheckedCoreBodyTerm::RecursiveDeclarationCall(
            checked_core::CheckedCoreRecursiveCallView { symbol, .. },
        ) => {
            output.insert(symbol.clone());
        }
        CheckedCoreBodyTerm::Lambda { body, .. } => {
            collect_checked_body_declaration_refs(body, output);
        }
        CheckedCoreBodyTerm::Application { function, argument } => {
            collect_checked_body_declaration_refs(function, output);
            collect_checked_body_declaration_refs(argument, output);
        }
        CheckedCoreBodyTerm::Let { value, body, .. } => {
            collect_checked_body_declaration_refs(value, output);
            collect_checked_body_declaration_refs(body, output);
        }
        CheckedCoreBodyTerm::Match(view) => {
            collect_checked_body_declaration_refs(&view.scrutinee, output);
            for branch in &view.branches {
                collect_checked_body_declaration_refs(&branch.method, output);
            }
        }
        CheckedCoreBodyTerm::PrimitiveApplication(view) => {
            for argument in &view.arguments {
                collect_checked_body_declaration_refs(argument, output);
            }
        }
        CheckedCoreBodyTerm::RecordSigmaConstruction(view) => {
            for field in &view.fields {
                if let checked_core::CheckedCoreRecordSigmaFieldValue::Runtime { value, .. } = field
                {
                    collect_checked_body_declaration_refs(value, output);
                }
            }
        }
        CheckedCoreBodyTerm::RecordSigmaProjection(view) => {
            collect_checked_body_declaration_refs(&view.base, output);
        }
        CheckedCoreBodyTerm::DictionaryConstruction(view) => {
            for field in &view.fields {
                if let checked_core::CheckedCoreDictionaryFieldValue::Runtime { value, .. } = field
                {
                    collect_checked_body_declaration_refs(value, output);
                }
            }
        }
        CheckedCoreBodyTerm::Variable { .. }
        | CheckedCoreBodyTerm::IntegerLiteral { .. }
        | CheckedCoreBodyTerm::ImportedDeclarationCall(_)
        | CheckedCoreBodyTerm::PrimitiveLiteral(_)
        | CheckedCoreBodyTerm::ConstructorReference(_)
        | CheckedCoreBodyTerm::ErasedConstructorArgument { .. } => {}
    }
}

fn admitted_recursive_member(
    semantic: &checked_core::CheckedCoreSemanticInputs,
    symbol: &StableSymbol,
) -> bool {
    semantic.recursion_metadata.values().any(|metadata| {
        matches!(
            metadata.admission,
            checked_core::RecursionAdmission::AcceptedStructural
                | checked_core::RecursionAdmission::AcceptedSizeChange
        ) && metadata.group_members.contains(symbol)
    })
}

fn runtime_declaration_refs_in_kind(kind: &RuntimeDeclarationKind) -> Vec<String> {
    let mut symbols = BTreeSet::new();
    if let RuntimeDeclarationKind::Transparent { body } = kind {
        collect_runtime_declaration_refs(body, &mut symbols);
    }
    symbols.into_iter().collect()
}

fn collect_runtime_declaration_refs(expr: &RuntimeExpr, output: &mut BTreeSet<String>) {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. } => {
            collect_runtime_declaration_refs(body, output)
        }
        RuntimeExpr::DeclarationRef { symbol } => {
            output.insert(symbol.clone());
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for arg in args {
                collect_runtime_declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Let { value, body } => {
            collect_runtime_declaration_refs(value, output);
            collect_runtime_declaration_refs(body, output);
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            collect_runtime_declaration_refs(scrutinee, output);
            collect_runtime_declaration_refs(then_expr, output);
            collect_runtime_declaration_refs(else_expr, output);
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            collect_runtime_declaration_refs(scrutinee, output);
            for case in cases {
                collect_runtime_declaration_refs(&case.body, output);
            }
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            collect_runtime_declaration_refs(scrutinee, output);
            for case in cases {
                collect_runtime_declaration_refs(&case.body, output);
            }
        }
        RuntimeExpr::Record { fields } => {
            for (_, value) in fields {
                collect_runtime_declaration_refs(value, output);
            }
        }
        RuntimeExpr::Project { record, .. } => collect_runtime_declaration_refs(record, output),
        RuntimeExpr::Closure { body, .. } | RuntimeExpr::LexicalClosure { body, .. } => {
            collect_runtime_declaration_refs(body, output);
        }
        RuntimeExpr::Call { callee, args } => {
            collect_runtime_declaration_refs(callee, output);
            for arg in args {
                collect_runtime_declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                collect_runtime_declaration_refs(&capability.value, output);
            }
            for arg in args {
                collect_runtime_declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => {}
    }
}

/// Elaborator-private identities for the checked Program-I HostIO spine.
///
/// Values are resolved from the same live environment and stable-symbol table
/// as the entrypoint plan.  No source spelling is accepted at this boundary.
#[derive(Clone, Debug)]
pub(crate) struct CheckedHostSpineV1 {
    pub ret: StableSymbol,
    pub vis: StableSymbol,
    pub in_l: StableSymbol,
    pub in_r: StableSymbol,
    pub fs_family: StableSymbol,
    pub console_family: StableSymbol,
    pub clock_family: StableSymbol,
    pub entropy_family: StableSymbol,
    pub capability: StableSymbol,
    pub result_err: StableSymbol,
    pub result_ok: StableSymbol,
    pub option_some: StableSymbol,
    pub file_error: StableSymbol,
    pub file_operation_read: StableSymbol,
    pub file_operation_write: StableSymbol,
    pub file_operation_change_mode: StableSymbol,
    pub io_errors: Vec<StableSymbol>,
    pub resource_host_io: StableSymbol,
    pub resource_closed: StableSymbol,
    pub resource_malformed: StableSymbol,
    pub resource_right_not_held: StableSymbol,
    pub resource_release_failed: StableSymbol,
    pub resource_kind_mismatch: StableSymbol,
    pub resource_buffer_limit: StableSymbol,
    pub resource_allocation_failed: StableSymbol,
    pub resource_invalid_offset: StableSymbol,
    pub resource_invalid_bounds: StableSymbol,
    pub resource_no_progress: StableSymbol,
    pub resource_kind_fs_handle: StableSymbol,
    pub resource_kind_buffer: StableSymbol,
    pub resource_trace_identity: StableSymbol,
    pub nat_zero: StableSymbol,
    pub nat_suc: StableSymbol,
    pub private_buffer_span: StableSymbol,
    pub private_transfer_count: StableSymbol,
    pub read_some: StableSymbol,
    pub read_eof: StableSymbol,
    pub wrote: StableSymbol,
    pub unit: StableSymbol,
    pub bool_false: StableSymbol,
    pub bool_true: StableSymbol,
    pub operations: BTreeMap<StableSymbol, ken_host::HostOpV1>,
}

#[derive(Clone)]
pub(crate) struct CheckedJoinAnswerSymbols {
    pub int: StableSymbol,
    pub bool_: StableSymbol,
    pub structural_nat: StableSymbol,
    pub exit_code: StableSymbol,
}

/// Kernel-typed authority for one complete same-SCC recursive application.
/// Produced before erasure; consumed exactly once when lowering the matching
/// maximal checked application spine.
#[derive(Clone)]
pub(crate) struct CheckedRecursiveInvocationSeed {
    pub call_template_id: u64,
    pub owner: StableSymbol,
    pub occurrence_ordinal: u64,
    pub callee: StableSymbol,
    pub level_instantiation: Vec<Vec<u8>>,
    pub recursion_group: StableSymbol,
    pub scc_index: u64,
    pub admission: u8,
    pub arity: usize,
    pub local_telescope: Vec<ken_runtime::CheckedAnswerInterfaceV1>,
    pub result_interface: ken_runtime::CheckedAnswerInterfaceV1,
}

#[derive(Clone)]
pub(crate) struct CheckedComputationalIHSlotSeed {
    pub slot_template_id: u64,
    pub owner: StableSymbol,
    pub match_ordinal: u64,
    pub branch_ordinal: usize,
    pub constructor: StableSymbol,
    pub recursive_position: usize,
    pub method_binder_ordinal: usize,
    pub local_telescope: Vec<ken_runtime::CheckedAnswerInterfaceV1>,
    pub ih_interface: ken_runtime::CheckedAnswerInterfaceV1,
}

#[derive(Clone)]
pub(crate) struct CheckedComputationalIHCallSeed {
    pub call_template_id: u64,
    pub owner: StableSymbol,
    pub slot_template_id: u64,
    pub occurrence_ordinal: u64,
    pub arity: usize,
    pub local_telescope: Vec<ken_runtime::CheckedAnswerInterfaceV1>,
    pub result_interface: ken_runtime::CheckedAnswerInterfaceV1,
}

#[derive(Clone)]
struct NativeJoinPlanCollector {
    answer_symbols: CheckedJoinAnswerSymbols,
    next_site_id: u64,
    sites: Vec<ken_runtime::NativeJoinPlanSiteV1>,
}

#[derive(Clone)]
struct NativeLoweringPlanCollector {
    joins: NativeJoinPlanCollector,
    oriented: OrientedSubcontinuationPlanCollector,
    recursive_invocations: BTreeMap<(StableSymbol, u64), CheckedRecursiveInvocationSeed>,
    consumed_recursive_invocations: BTreeSet<u64>,
    next_recursive_ordinal: BTreeMap<StableSymbol, u64>,
    pending_recursive_calls: Vec<(CheckedRecursiveInvocationSeed, Vec<u64>, Option<u64>)>,
    computational_ih_slots:
        BTreeMap<(StableSymbol, u64, usize, usize), CheckedComputationalIHSlotSeed>,
    computational_ih_calls: BTreeMap<(u64, u64), CheckedComputationalIHCallSeed>,
    next_computational_match_ordinal: BTreeMap<StableSymbol, u64>,
    next_computational_ih_call_ordinal: BTreeMap<u64, u64>,
    consumed_computational_ih_slots: BTreeSet<u64>,
    consumed_computational_ih_calls: BTreeSet<u64>,
    pending_computational_ih_slots: Vec<(CheckedComputationalIHSlotSeed, Vec<u64>, u64)>,
    pending_computational_ih_calls: Vec<(
        CheckedComputationalIHCallSeed,
        Vec<u64>,
        Option<u64>,
        ComputationalIHConsumptionRoute,
    )>,
}

/// **`D7` checkpoint `1b` -- the CLOSED set of erasure routes that can consume
/// a checked computational IH application, and the SOLE authority for how many
/// operands each of them injects.**
///
/// ⭐⭐ **Two distinct lawful coordinates meet here, and collapsing them into
/// one number is the defect `1b` removes.**
///
/// | coordinate | what it counts | who owns it |
/// |---|---|---|
/// | source occurrence arity | the operands the checked application actually wrote | the checked seed, `CheckedComputationalIHCallSeed::arity` |
/// | complete emitted arity | the operands the emitted `RuntimeExpr::Call` carries | the consuming route |
///
/// The two differ exactly when a route **appends** an operand the source never
/// wrote. Precisely one route does: the Host-`Vis` continuation, which pushes
/// the host result as `RuntimeExpr::Var(0)`.
///
/// ⛔ **The delta is not a number a caller may supply.** An unconstrained
/// `usize` parameter is the same defect facing the other way: it happens to be
/// `1` on the measured path and nothing makes it wrong anywhere else. The delta
/// is a *property of the route*, so the route is what travels, and adding a
/// fourth route is a compile error in [`Self::injected_operands`] rather than a
/// silent zero.
///
/// ⛔ **The seed's own `arity` may NOT absorb the injected operand.** A nullary
/// force reaches the non-injecting routes too, where the source count and the
/// head's Pi telescope legitimately differ; moving the correction into the
/// producer therefore shifts applications that inject nothing. Measured: it
/// takes the five `fs_*` parity rows off their exact framed base refusal and
/// onto a stale-binding error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComputationalIHConsumptionRoute {
    /// `lower_body_term_with_plans` -- an ordinary erased application, which
    /// emits exactly the operands the source wrote.
    OrdinaryApplication,
    /// The head of `lower_checked_host_computation` -- a checked IH application
    /// standing as the whole host computation. It completes no `Vis`
    /// continuation, so it injects nothing either.
    CheckedHostComputationTail,
    /// The `Vis` continuation in `lower_checked_host_computation` -- the one
    /// route that appends the host result.
    CheckedHostVisContinuation,
}

impl ComputationalIHConsumptionRoute {
    /// How many operands this route APPENDS that the source application never
    /// wrote.
    ///
    /// ⭐ The three arms are spelled separately rather than folded into one
    /// `0 => ..` pattern: each route's answer is its own decision, and a future
    /// injecting route must be able to change exactly one of them.
    fn injected_operands(self) -> u64 {
        // The `cfg(test)` mutation below corrupts ONLY the number the template
        // is built from. The Host-`Vis` emitter still appends exactly one
        // `Var(0)`, so under either mutation the template and the emitted call
        // disagree -- which is the condition the Runtime marker gate exists to
        // refuse, and the condition the controls drive it with.
        #[cfg(test)]
        if matches!(self, Self::CheckedHostVisContinuation) {
            match HOST_VIS_INJECTION_MUTATION.with(std::cell::Cell::get) {
                HostVisInjectionMutation::None => {}
                HostVisInjectionMutation::OmitInjectedResult => return 0,
                HostVisInjectionMutation::DoubleCountInjectedResult => return 2,
            }
        }
        match self {
            Self::OrdinaryApplication => 0,
            Self::CheckedHostComputationTail => 0,
            Self::CheckedHostVisContinuation => 1,
        }
    }
}

/// The COMPLETE emitted application's operand count: the source occurrence's
/// own binding count plus exactly what its consuming route injects.
///
/// ⛔ Both steps are checked. No target Ken builds for has a `usize` wider than
/// `u64`, but the conversion is exactly where that assumption would live, so it
/// is stated rather than left implicit; and the addition cannot wrap into a
/// small arity that the marker gate would then happily accept.
fn complete_emitted_arity(
    source_arity: usize,
    route: ComputationalIHConsumptionRoute,
) -> Option<u64> {
    u64::try_from(source_arity)
        .ok()?
        .checked_add(route.injected_operands())
}

/// **`cfg(test)`-only corruption of the Host-`Vis` route's injected-operand
/// count.** Its whole purpose is to make the template disagree with the
/// application the same route emits, so that the disagreement can be driven
/// into the real Runtime marker gate rather than asserted arithmetically here.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostVisInjectionMutation {
    None,
    OmitInjectedResult,
    DoubleCountInjectedResult,
}

#[cfg(test)]
thread_local! {
    static HOST_VIS_INJECTION_MUTATION: std::cell::Cell<HostVisInjectionMutation> =
        const { std::cell::Cell::new(HostVisInjectionMutation::None) };
}

/// Run `body` with the Host-`Vis` injected-operand count mutated, then clear it.
///
/// ⛔ There is deliberately no restore-on-unwind guard. A panic inside the scope
/// means the fixture itself failed, and the test binary is going down with it;
/// catching the unwind here would only let a broken fixture look tidy. Rust runs
/// each test in its own thread, and the cell is thread-local, so a panicking row
/// cannot leak the mutation into a sibling row.
#[cfg(test)]
fn with_host_vis_injection_mutation<R>(
    mutation: HostVisInjectionMutation,
    body: impl FnOnce() -> R,
) -> R {
    HOST_VIS_INJECTION_MUTATION.with(|cell| cell.set(mutation));
    let result = body();
    HOST_VIS_INJECTION_MUTATION.with(|cell| cell.set(HostVisInjectionMutation::None));
    result
}

impl NativeLoweringPlanCollector {
    fn new(
        answer_symbols: CheckedJoinAnswerSymbols,
        recursive_invocations: Vec<CheckedRecursiveInvocationSeed>,
        computational_ih_slots: Vec<CheckedComputationalIHSlotSeed>,
        computational_ih_calls: Vec<CheckedComputationalIHCallSeed>,
    ) -> Self {
        let recursive_invocations = recursive_invocations
            .into_iter()
            .map(|seed| ((seed.owner.clone(), seed.occurrence_ordinal), seed))
            .collect();
        let computational_ih_slots = computational_ih_slots
            .into_iter()
            .map(|seed| {
                (
                    (
                        seed.owner.clone(),
                        seed.match_ordinal,
                        seed.branch_ordinal,
                        seed.method_binder_ordinal,
                    ),
                    seed,
                )
            })
            .collect();
        let computational_ih_calls = computational_ih_calls
            .into_iter()
            .map(|seed| ((seed.slot_template_id, seed.occurrence_ordinal), seed))
            .collect();
        Self {
            joins: NativeJoinPlanCollector::new(answer_symbols),
            oriented: OrientedSubcontinuationPlanCollector::default(),
            recursive_invocations,
            consumed_recursive_invocations: BTreeSet::new(),
            next_recursive_ordinal: BTreeMap::new(),
            pending_recursive_calls: Vec::new(),
            computational_ih_slots,
            computational_ih_calls,
            next_computational_match_ordinal: BTreeMap::new(),
            next_computational_ih_call_ordinal: BTreeMap::new(),
            consumed_computational_ih_slots: BTreeSet::new(),
            consumed_computational_ih_calls: BTreeSet::new(),
            pending_computational_ih_slots: Vec::new(),
            pending_computational_ih_calls: Vec::new(),
        }
    }

    fn finish(
        self,
    ) -> (
        ken_runtime::NativeJoinPlanV1,
        ken_runtime::OrientedSubcontinuationPlanV1,
    ) {
        (
            self.joins.finish(),
            self.oriented.finish(
                self.recursive_invocations,
                self.consumed_recursive_invocations,
                self.pending_recursive_calls,
                self.computational_ih_slots,
                self.computational_ih_calls,
                self.consumed_computational_ih_slots,
                self.consumed_computational_ih_calls,
                self.pending_computational_ih_slots,
                self.pending_computational_ih_calls,
            ),
        )
    }

    fn validate_total_computational_ih_seed_consumption(&self) -> Result<(), ErasureError> {
        if self.computational_ih_slots.len() != self.consumed_computational_ih_slots.len() {
            let seed = self
                .computational_ih_slots
                .values()
                .find(|seed| {
                    !self
                        .consumed_computational_ih_slots
                        .contains(&seed.slot_template_id)
                })
                .or_else(|| self.computational_ih_slots.values().next())
                .expect("a slot-count mismatch has at least one supplied slot");
            return Err(expression_lowering_error(
                &seed.owner,
                "checked_computational_ih_slot_unconsumed",
                "not every supplied computational IH slot template was consumed exactly once",
            ));
        }
        if self.computational_ih_calls.len() != self.consumed_computational_ih_calls.len() {
            let seed = self
                .computational_ih_calls
                .values()
                .find(|seed| {
                    !self
                        .consumed_computational_ih_calls
                        .contains(&seed.call_template_id)
                })
                .or_else(|| self.computational_ih_calls.values().next())
                .expect("a call-count mismatch has at least one supplied call");
            return Err(expression_lowering_error(
                &seed.owner,
                "checked_computational_ih_call_unconsumed",
                "not every supplied computational IH call template was consumed exactly once",
            ));
        }
        Ok(())
    }

    fn consume_recursive_invocation(
        &mut self,
        owner: &StableSymbol,
        view: &checked_core::CheckedCoreRecursiveCallView,
        arity: usize,
        occurrence_path: &[u64],
        parent_frame: Option<u64>,
    ) -> Result<u64, ErasureError> {
        let ordinal = self
            .next_recursive_ordinal
            .entry(owner.clone())
            .or_default();
        let key = (owner.clone(), *ordinal);
        *ordinal = ordinal
            .checked_add(1)
            .expect("compiler-private recursive occurrence ordinal exhausted");
        let seed = self
            .recursive_invocations
            .get(&key)
            .cloned()
            .ok_or_else(|| {
                expression_lowering_error(
                    owner,
                    "checked_recursive_invocation_missing",
                    format!(
                        "complete recursive application occurrence {key:?} has no checked template"
                    ),
                )
            })?;
        if seed.callee != view.symbol
            || seed.recursion_group != view.group_symbol
            || seed.scc_index != view.scc_index as u64
            || seed.arity != arity
        {
            return Err(expression_lowering_error(
                owner,
                "checked_recursive_invocation_mismatch",
                "recursive application template callee/group/SCC/arity binding is stale",
            ));
        }
        if !self
            .consumed_recursive_invocations
            .insert(seed.call_template_id)
        {
            return Err(expression_lowering_error(
                owner,
                "checked_recursive_invocation_duplicate",
                "recursive application template was consumed more than once",
            ));
        }
        self.pending_recursive_calls
            .push((seed.clone(), occurrence_path.to_vec(), parent_frame));
        Ok(seed.call_template_id)
    }

    fn consume_computational_ih_slots(
        &mut self,
        owner: &StableSymbol,
        view: &checked_core::CheckedCoreMatchView,
        occurrence_path: &[u64],
        frame_id: u64,
    ) -> Result<Vec<Vec<(u64, Vec<u64>)>>, ErasureError> {
        let ordinal = self
            .next_computational_match_ordinal
            .entry(owner.clone())
            .or_default();
        let match_ordinal = *ordinal;
        *ordinal = ordinal
            .checked_add(1)
            .expect("compiler-private computational match ordinal exhausted");
        let mut result = Vec::with_capacity(view.branches.len());
        for (branch_ordinal, branch) in view.branches.iter().enumerate() {
            let mut branch_slots = Vec::with_capacity(branch.constructor.recursive_positions.len());
            for (method_binder_ordinal, recursive_position) in branch
                .constructor
                .recursive_positions
                .iter()
                .copied()
                .enumerate()
            {
                let key = (
                    owner.clone(),
                    match_ordinal,
                    branch_ordinal,
                    method_binder_ordinal,
                );
                let seed = self.computational_ih_slots.get(&key).cloned().ok_or_else(|| {
                    let candidates = self
                        .computational_ih_slots
                        .values()
                        .filter(|seed| seed.owner == *owner)
                        .map(|seed| (seed.match_ordinal, seed.branch_ordinal, seed.method_binder_ordinal, seed.constructor.to_string(), seed.recursive_position))
                        .collect::<Vec<_>>();
                    expression_lowering_error(
                        owner,
                        "checked_computational_ih_slot_missing",
                        format!("computational IH binder {key:?} has no checked slot template; candidates={candidates:?}"),
                    )
                })?;
                if seed.constructor != branch.constructor.symbol
                    || seed.recursive_position != recursive_position
                {
                    return Err(expression_lowering_error(
                        owner,
                        "checked_computational_ih_slot_mismatch",
                        format!(
                            "computational IH slot constructor/recursive-position binding is stale: checked {} position {}, erased {} position {}",
                            seed.constructor,
                            seed.recursive_position,
                            branch.constructor.symbol,
                            recursive_position,
                        ),
                    ));
                }
                if !self
                    .consumed_computational_ih_slots
                    .insert(seed.slot_template_id)
                {
                    return Err(expression_lowering_error(
                        owner,
                        "checked_computational_ih_slot_duplicate",
                        "computational IH slot template was consumed more than once",
                    ));
                }
                let mut path = occurrence_path.to_vec();
                path.extend([21, branch_ordinal as u64, method_binder_ordinal as u64]);
                self.pending_computational_ih_slots
                    .push((seed.clone(), path.clone(), frame_id));
                branch_slots.push((seed.slot_template_id, path));
            }
            result.push(branch_slots);
        }
        Ok(result)
    }

    /// **`D7` checkpoint `1b` — bind this occurrence at its SOURCE arity, and
    /// record the ROUTE that will complete its emitted application.**
    ///
    /// ⭐ `arity` is the source application's own argument count, and it is what
    /// binds this occurrence to its seed -- that binding stays exactly as it
    /// was, because it is how a stale or mis-ordered seed is caught. `route` is
    /// the separate, closed authority for the operands the emitter appends; see
    /// [`ComputationalIHConsumptionRoute`] for why the delta travels as a route
    /// rather than as a number.
    fn consume_computational_ih_call(
        &mut self,
        owner: &StableSymbol,
        slot_template_id: u64,
        arity: usize,
        route: ComputationalIHConsumptionRoute,
        occurrence_path: &[u64],
        parent_frame: Option<u64>,
    ) -> Result<u64, ErasureError> {
        let ordinal = self
            .next_computational_ih_call_ordinal
            .entry(slot_template_id)
            .or_default();
        let key = (slot_template_id, *ordinal);
        *ordinal = ordinal
            .checked_add(1)
            .expect("compiler-private computational IH call ordinal exhausted");
        let seed = self
            .computational_ih_calls
            .get(&key)
            .cloned()
            .ok_or_else(|| {
                expression_lowering_error(
                    owner,
                    "checked_computational_ih_call_missing",
                    format!(
                        "complete computational IH application {key:?} has no checked template"
                    ),
                )
            })?;
        if seed.owner != *owner || seed.arity != arity {
            return Err(expression_lowering_error(
                owner,
                "checked_computational_ih_call_mismatch",
                "computational IH call owner/arity binding is stale",
            ));
        }
        if !self
            .consumed_computational_ih_calls
            .insert(seed.call_template_id)
        {
            return Err(expression_lowering_error(
                owner,
                "checked_computational_ih_call_duplicate",
                "computational IH call template was consumed more than once",
            ));
        }
        // The checked conversion and addition are performed HERE, where a
        // refusal is a real diagnostic rather than a panic in the finisher.
        // `finish` re-derives the same value from the same closed route, so its
        // own expect is unreachable by construction.
        if complete_emitted_arity(arity, route).is_none() {
            return Err(expression_lowering_error(
                owner,
                "checked_computational_ih_call_arity_overflow",
                "complete emitted computational IH application arity does not fit runtime IR",
            ));
        }
        self.pending_computational_ih_calls.push((
            seed.clone(),
            occurrence_path.to_vec(),
            parent_frame,
            route,
        ));
        Ok(seed.call_template_id)
    }
}

impl NativeJoinPlanCollector {
    fn new(answer_symbols: CheckedJoinAnswerSymbols) -> Self {
        Self {
            answer_symbols,
            next_site_id: 0,
            sites: Vec::new(),
        }
    }

    fn finish(self) -> ken_runtime::NativeJoinPlanV1 {
        ken_runtime::NativeJoinPlanV1 {
            representation_rule_version: ken_runtime::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
            sites: self.sites,
        }
    }

    fn record_root_exit_answer(&mut self, root: &StableSymbol, checked_type: &[u8]) {
        let site_id = self.next_site_id;
        self.next_site_id = self
            .next_site_id
            .checked_add(1)
            .expect("compiler-private join site identity exhausted");
        let path = vec![0];
        let type_fp = ken_runtime::fnv1a_64(checked_type);
        self.sites.push(ken_runtime::NativeJoinPlanSiteV1 {
            site_id,
            declaration: root.to_string(),
            checked_occurrence_path: path.clone(),
            checked_result_type_fingerprint: type_fp,
            occurrence_binding_fingerprint:
                ken_runtime::compiler_private_join_occurrence_binding_fingerprint(
                    site_id,
                    &root.to_string(),
                    &path,
                    type_fp,
                ),
            runtime_frame_fingerprint: ken_runtime::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1,
            answer_kind: ken_runtime::NativeJoinAnswerKindV1::ExitCode,
        });
    }

    fn record_match(
        &mut self,
        owner: &StableSymbol,
        path: &[u64],
        view: &checked_core::CheckedCoreMatchView,
        runtime: &RuntimeExpr,
    ) -> Result<Option<u64>, ErasureError> {
        let Some(result_type) = checked_core::checked_constant_motive_result_type(&view.motive)
            .map_err(|reason| {
                expression_lowering_error(owner, "native_join_plan_motive", reason)
            })?
        else {
            return Ok(None);
        };
        let Some(head) =
            checked_core::checked_type_head_symbol(&result_type).map_err(|reason| {
                expression_lowering_error(owner, "native_join_plan_result_type", reason)
            })?
        else {
            return Ok(None);
        };
        let answer_kind = if head == self.answer_symbols.int {
            ken_runtime::NativeJoinAnswerKindV1::Int
        } else if head == self.answer_symbols.bool_ {
            ken_runtime::NativeJoinAnswerKindV1::Bool
        } else if head == self.answer_symbols.structural_nat {
            ken_runtime::NativeJoinAnswerKindV1::StructuralNat
        } else if head == self.answer_symbols.exit_code {
            ken_runtime::NativeJoinAnswerKindV1::ExitCode
        } else {
            return Ok(None);
        };
        let runtime_frame_fingerprint = match runtime {
            RuntimeExpr::Match { cases, default, .. } => {
                ken_runtime::compiler_private_ordinary_match_frame_fingerprint(cases, default)
            }
            RuntimeExpr::ComputationalMatch { cases, default, .. } => {
                ken_runtime::compiler_private_computational_match_frame_fingerprint(cases, default)
            }
            _ => unreachable!("checked Match lowers to one Runtime Match form"),
        };
        let site_id = self.next_site_id;
        self.next_site_id = self
            .next_site_id
            .checked_add(1)
            .expect("compiler-private join site identity exhausted");
        let type_fp = ken_runtime::fnv1a_64(&result_type);
        self.sites.push(ken_runtime::NativeJoinPlanSiteV1 {
            site_id,
            declaration: owner.to_string(),
            checked_occurrence_path: path.to_vec(),
            checked_result_type_fingerprint: type_fp,
            occurrence_binding_fingerprint:
                ken_runtime::compiler_private_join_occurrence_binding_fingerprint(
                    site_id,
                    &owner.to_string(),
                    path,
                    type_fp,
                ),
            runtime_frame_fingerprint,
            answer_kind,
        });
        Ok(Some(site_id))
    }
}

#[derive(Clone, Default)]
struct OrientedSubcontinuationPlanCollector {
    next_frame_id: u64,
    next_semantic_position: u64,
    segment_by_frame: BTreeMap<u64, u64>,
    input_by_frame: BTreeMap<u64, ken_runtime::CheckedAnswerInterfaceV1>,
    frames: Vec<ken_runtime::OrientedSubcontinuationFramePlanV1>,
}

struct PendingOrientedFrame {
    frame_id: u64,
    segment_site_id: u64,
    declaration: String,
    checked_occurrence_path: Vec<u64>,
    input_interface: ken_runtime::CheckedAnswerInterfaceV1,
    output_interface: ken_runtime::CheckedAnswerInterfaceV1,
    control_witness: ken_runtime::OrientedControlWitnessV1,
}

impl OrientedSubcontinuationPlanCollector {
    fn begin_match(
        &mut self,
        owner: &StableSymbol,
        path: &[u64],
        parent_frame: Option<u64>,
        view: &checked_core::CheckedCoreMatchView,
    ) -> Result<Option<PendingOrientedFrame>, ErasureError> {
        if !view.computational_recursive_hypotheses {
            return Ok(None);
        }
        let frame_id = self.next_frame_id;
        self.next_frame_id = self
            .next_frame_id
            .checked_add(1)
            .expect("compiler-private oriented frame identity exhausted");
        let segment_site_id = parent_frame
            .and_then(|parent| self.segment_by_frame.get(&parent).copied())
            .unwrap_or(frame_id);
        self.segment_by_frame.insert(frame_id, segment_site_id);
        let input_interface = canonical_checked_answer_interface(
            &view.family_symbol,
            &view.level_args,
            view.parameters.iter().chain(view.indices.iter()),
        );
        let output_interface = parent_frame
            .and_then(|parent| self.input_by_frame.get(&parent).cloned())
            .unwrap_or_else(|| canonical_checked_motive_interface(&view.motive));
        self.input_by_frame
            .insert(frame_id, input_interface.clone());
        Ok(Some(PendingOrientedFrame {
            frame_id,
            segment_site_id,
            declaration: owner.to_string(),
            checked_occurrence_path: path.to_vec(),
            input_interface,
            output_interface,
            control_witness: parent_frame.map_or(
                ken_runtime::OrientedControlWitnessV1::DistinguishedRoot,
                ken_runtime::OrientedControlWitnessV1::ParentFrame,
            ),
        }))
    }

    fn finish_match(
        &mut self,
        pending: PendingOrientedFrame,
        runtime: &RuntimeExpr,
    ) -> Result<u64, ErasureError> {
        let runtime_frame_fingerprint = match runtime {
            RuntimeExpr::ComputationalMatch { cases, default, .. } => {
                ken_runtime::compiler_private_computational_match_frame_fingerprint(cases, default)
            }
            _ => unreachable!("oriented frame is emitted only for a computational Match"),
        };
        let semantic_position = self.next_semantic_position;
        self.next_semantic_position = self
            .next_semantic_position
            .checked_add(1)
            .expect("compiler-private semantic frame position exhausted");
        let mut frame = ken_runtime::OrientedSubcontinuationFramePlanV1 {
            frame_id: pending.frame_id,
            segment_site_id: pending.segment_site_id,
            declaration: pending.declaration,
            checked_occurrence_path: pending.checked_occurrence_path,
            semantic_position,
            input_interface: pending.input_interface,
            output_interface: pending.output_interface,
            runtime_frame_fingerprint,
            occurrence_binding_fingerprint: 0,
            control_witness: pending.control_witness,
        };
        frame.occurrence_binding_fingerprint =
            ken_runtime::compiler_private_oriented_occurrence_binding_fingerprint(&frame);
        self.frames.push(frame);
        Ok(pending.frame_id)
    }

    fn finish(
        mut self,
        _recursive_invocations: BTreeMap<(StableSymbol, u64), CheckedRecursiveInvocationSeed>,
        consumed_recursive_invocations: BTreeSet<u64>,
        pending_recursive_calls: Vec<(CheckedRecursiveInvocationSeed, Vec<u64>, Option<u64>)>,
        computational_ih_slot_seeds: BTreeMap<
            (StableSymbol, u64, usize, usize),
            CheckedComputationalIHSlotSeed,
        >,
        computational_ih_call_seeds: BTreeMap<(u64, u64), CheckedComputationalIHCallSeed>,
        consumed_computational_ih_slots: BTreeSet<u64>,
        consumed_computational_ih_calls: BTreeSet<u64>,
        pending_computational_ih_slots: Vec<(CheckedComputationalIHSlotSeed, Vec<u64>, u64)>,
        pending_computational_ih_calls: Vec<(
            CheckedComputationalIHCallSeed,
            Vec<u64>,
            Option<u64>,
            ComputationalIHConsumptionRoute,
        )>,
    ) -> ken_runtime::OrientedSubcontinuationPlanV1 {
        assert_eq!(
            pending_recursive_calls.len(),
            consumed_recursive_invocations.len(),
            "emitted recursive invocation templates close exactly over Runtime markers"
        );
        assert_eq!(
            pending_computational_ih_slots.len(),
            consumed_computational_ih_slots.len(),
            "emitted computational IH slot templates close exactly over Runtime case markers"
        );
        assert_eq!(
            computational_ih_slot_seeds.len(),
            consumed_computational_ih_slots.len(),
            "every supplied computational IH slot template is consumed exactly once"
        );
        assert_eq!(
            pending_computational_ih_calls.len(),
            consumed_computational_ih_calls.len(),
            "emitted computational IH call templates close exactly over Runtime call markers"
        );
        assert_eq!(
            computational_ih_call_seeds.len(),
            consumed_computational_ih_calls.len(),
            "every supplied computational IH call template is consumed exactly once"
        );
        let mut recursive_calls = Vec::with_capacity(pending_recursive_calls.len());
        for (seed, occurrence_path, parent_frame) in pending_recursive_calls {
            let mut callee_frames = self
                .frames
                .iter()
                .filter(|frame| frame.declaration == seed.callee.to_string())
                .map(|frame| frame.frame_id)
                .collect::<Vec<_>>();
            callee_frames.sort_by_key(|id| {
                self.frames
                    .iter()
                    .find(|frame| frame.frame_id == *id)
                    .expect("callee frame exists")
                    .semantic_position
            });
            if callee_frames.is_empty() {
                // The checked recursive-call census is intentionally broader
                // than oriented lowering. Preserve the established bare
                // recursive IR when the callee owns no oriented frame; only
                // calls that can instantiate a segment receive a plan row and
                // Runtime marker.
                continue;
            }
            let segment_site_id = self
                .frames
                .iter()
                .find(|frame| frame.frame_id == callee_frames[0])
                .expect("callee root frame exists")
                .segment_site_id;
            assert!(callee_frames.iter().all(|id| self
                .frames
                .iter()
                .find(|frame| frame.frame_id == *id)
                .is_some_and(|frame| frame.segment_site_id == segment_site_id)));

            // The kernel-inferred fully-applied result is the endpoint
            // authority for the reusable callee template.  Rebind the final
            // static frame endpoint before sealing occurrence fingerprints.
            let last_id = *callee_frames.last().expect("callee segment nonempty");
            let last = self
                .frames
                .iter_mut()
                .find(|frame| frame.frame_id == last_id)
                .expect("callee final frame exists");
            last.output_interface = seed.result_interface.clone();
            last.occurrence_binding_fingerprint =
                ken_runtime::compiler_private_oriented_occurrence_binding_fingerprint(last);

            let caller_interface = parent_frame
                .and_then(|id| self.frames.iter().find(|frame| frame.frame_id == id))
                .map(|frame| frame.input_interface.clone())
                .unwrap_or_else(|| seed.result_interface.clone());
            let mut call = ken_runtime::CheckedRecursiveInvocationTemplateV1 {
                call_template_id: seed.call_template_id,
                declaration: seed.owner.to_string(),
                checked_occurrence_path: occurrence_path,
                callee: seed.callee.to_string(),
                level_instantiation: seed.level_instantiation,
                recursion_group: seed.recursion_group.to_string(),
                scc_index: seed.scc_index,
                admission: seed.admission,
                arity: seed.arity as u64,
                local_telescope: seed.local_telescope,
                result_interface: seed.result_interface,
                callee_segment_site_id: segment_site_id,
                callee_frame_templates: callee_frames,
                caller_interface,
                runtime_marker_locations: Vec::new(),
                occurrence_binding_fingerprint: 0,
            };
            // For the currently supported non-dependent native subset, the
            // checked call result is exactly the caller continuation input.
            call.caller_interface = call.result_interface.clone();
            call.occurrence_binding_fingerprint =
                ken_runtime::compiler_private_recursive_call_binding_fingerprint(&call);
            recursive_calls.push(call);
        }
        let mut computational_ih_slots = Vec::with_capacity(pending_computational_ih_slots.len());
        for (seed, occurrence_path, frame_id) in pending_computational_ih_slots {
            let frame = self
                .frames
                .iter()
                .find(|frame| frame.frame_id == frame_id)
                .expect("computational IH slot frame exists");
            let mut slot = ken_runtime::CheckedComputationalIHSlotTemplateV1 {
                slot_template_id: seed.slot_template_id,
                declaration: seed.owner.to_string(),
                checked_match_ordinal: seed.match_ordinal,
                checked_occurrence_path: occurrence_path,
                frame_template_id: frame_id,
                constructor: seed.constructor.to_string(),
                recursive_position: seed.recursive_position as u64,
                method_binder_ordinal: seed.method_binder_ordinal as u64,
                local_telescope: seed.local_telescope,
                ih_interface: seed.ih_interface,
                segment_site_id: frame.segment_site_id,
                frame_templates: vec![frame_id],
                input_interface: frame.input_interface.clone(),
                output_interface: frame.output_interface.clone(),
                runtime_marker_locations: Vec::new(),
                occurrence_binding_fingerprint: 0,
            };
            slot.occurrence_binding_fingerprint =
                ken_runtime::compiler_private_computational_ih_slot_binding_fingerprint(&slot);
            computational_ih_slots.push(slot);
        }
        let mut computational_ih_calls = Vec::with_capacity(pending_computational_ih_calls.len());
        for (seed, occurrence_path, parent_frame, route) in pending_computational_ih_calls {
            let slot = computational_ih_slots
                .iter()
                .find(|slot| slot.slot_template_id == seed.slot_template_id)
                .expect("computational IH call slot exists");
            // The callee sequence is the reusable slot template only.  The
            // enclosing checked frame is the caller endpoint; including that
            // parent in the callee sequence would let one IH invocation claim
            // a continuation owned by a later, distinct marker.
            let callee_frame_templates = slot.frame_templates.clone();
            if parent_frame.is_some() && parent_frame != Some(slot.frame_template_id) {
                panic!("checked computational IH call is not enclosed by its slot frame");
            }
            // This is the exact checked frame enclosing the IH call occurrence.
            // Its own control witness describes that frame's static parent, a
            // different edge which must not be substituted for this dynamic
            // call-to-open-scope binding.
            let parent_frame_template_id = parent_frame;
            let parent = parent_frame_template_id
                .and_then(|id| self.frames.iter().find(|frame| frame.frame_id == id));
            let caller_interface = seed.result_interface.clone();
            let mut call = ken_runtime::CheckedComputationalIHCallTemplateV1 {
                call_template_id: seed.call_template_id,
                declaration: seed.owner.to_string(),
                checked_occurrence_path: occurrence_path,
                slot_template_id: seed.slot_template_id,
                // ⭐ **THE `1b` CORRECTION.** The template names the COMPLETE
                // erased Runtime application -- the checked arguments plus
                // exactly what the emitting route injects -- so the marker's
                // entry and its static-worker consumption compare a complete
                // `Call`'s argument count against an arity describing the same
                // application. ⛔ The marker law itself is untouched: both still
                // compare a complete count against an immutable arity.
                arity: complete_emitted_arity(seed.arity, route).expect(
                    "consume_computational_ih_call already refused an unrepresentable \
                     complete emitted arity",
                ),
                local_telescope: seed.local_telescope,
                result_interface: seed.result_interface.clone(),
                callee_segment_site_id: slot.segment_site_id,
                callee_frame_templates,
                parent_frame_template_id,
                parent_segment_site_id: parent.map(|frame| frame.segment_site_id),
                caller_interface,
                runtime_marker_locations: Vec::new(),
                occurrence_binding_fingerprint: 0,
            };
            call.occurrence_binding_fingerprint =
                ken_runtime::compiler_private_computational_ih_call_binding_fingerprint(&call);
            computational_ih_calls.push(call);
        }
        ken_runtime::OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                ken_runtime::OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: self.frames,
            recursive_calls,
            computational_ih_slots,
            computational_ih_calls,
        }
    }
}

fn canonical_checked_answer_interface<'a>(
    head: &StableSymbol,
    levels: &[CheckedCoreLevelView],
    arguments: impl IntoIterator<Item = &'a Vec<u8>>,
) -> ken_runtime::CheckedAnswerInterfaceV1 {
    fn put_u64(out: &mut Vec<u8>, value: u64) {
        out.extend_from_slice(&value.to_be_bytes());
    }
    fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
        put_u64(out, bytes.len() as u64);
        out.extend_from_slice(bytes);
    }
    fn put_level(out: &mut Vec<u8>, level: &CheckedCoreLevelView) {
        match level {
            CheckedCoreLevelView::Zero => out.push(0),
            CheckedCoreLevelView::Suc(inner) => {
                out.push(1);
                put_level(out, inner);
            }
            CheckedCoreLevelView::Max(left, right) => {
                out.push(2);
                put_level(out, left);
                put_level(out, right);
            }
            CheckedCoreLevelView::Var(index) => {
                out.push(3);
                put_u64(out, *index);
            }
        }
    }

    let mut canonical = ken_runtime::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
    put_bytes(&mut canonical, head.to_string().as_bytes());
    put_u64(&mut canonical, levels.len() as u64);
    for level in levels {
        put_level(&mut canonical, level);
    }
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    put_u64(&mut canonical, arguments.len() as u64);
    for argument in arguments {
        put_bytes(&mut canonical, argument);
    }
    ken_runtime::CheckedAnswerInterfaceV1::new(canonical)
        .expect("canonical answer interface carries its fixed header")
}

fn canonical_checked_motive_interface(motive: &[u8]) -> ken_runtime::CheckedAnswerInterfaceV1 {
    let mut canonical = ken_runtime::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
    canonical.extend_from_slice(b"motive\0");
    canonical.extend_from_slice(&(motive.len() as u64).to_be_bytes());
    canonical.extend_from_slice(motive);
    ken_runtime::CheckedAnswerInterfaceV1::new(canonical)
        .expect("compiler-private checked motive descriptor has its canonical header")
}

/// Deforest an identity-checked HostIO tree while erasing the selected target.
/// The tree does not survive into the artifact: every `Vis op k` becomes an
/// ordinary response-producing `Effect`, immediately bound by `Let` to the
/// recursively lowered continuation.
pub(crate) fn erase_checked_host_package_for_target<'a>(
    package: &CheckedCorePackage,
    target_closure: impl IntoIterator<Item = &'a StableSymbol>,
    root: &StableSymbol,
    spine: &CheckedHostSpineV1,
) -> Result<RuntimeProgram, ErasureError> {
    let targets: Vec<StableSymbol> = target_closure.into_iter().cloned().collect();
    erase_checked_package_with_host_root(package, targets, Some((root, spine)), None)
}

pub(crate) fn erase_checked_host_package_for_target_with_join_plan<'a>(
    package: &CheckedCorePackage,
    target_closure: impl IntoIterator<Item = &'a StableSymbol>,
    root: &StableSymbol,
    spine: &CheckedHostSpineV1,
    answer_symbols: CheckedJoinAnswerSymbols,
    recursive_invocations: Vec<CheckedRecursiveInvocationSeed>,
    computational_ih_slots: Vec<CheckedComputationalIHSlotSeed>,
    computational_ih_calls: Vec<CheckedComputationalIHCallSeed>,
) -> Result<
    (
        RuntimeProgram,
        ken_runtime::NativeJoinPlanV1,
        ken_runtime::OrientedSubcontinuationPlanV1,
    ),
    ErasureError,
> {
    let targets: Vec<StableSymbol> = target_closure.into_iter().cloned().collect();
    let mut collector = NativeLoweringPlanCollector::new(
        answer_symbols,
        recursive_invocations,
        computational_ih_slots,
        computational_ih_calls,
    );
    let mut program = erase_checked_package_with_host_root(
        package,
        targets,
        Some((root, spine)),
        Some(&mut collector),
    )?;
    collector.validate_total_computational_ih_seed_consumption()?;
    let (join_plan, mut oriented_plan) = collector.finish();
    let retained_recursive_calls = oriented_plan
        .recursive_calls
        .iter()
        .map(|call| call.call_template_id)
        .collect::<BTreeSet<_>>();
    for declaration in &mut program.declarations {
        if let RuntimeDeclarationKind::Transparent { body } = &mut declaration.kind {
            remove_unplanned_recursive_invocation_markers(body, &retained_recursive_calls);
        }
    }
    for example in &mut program.examples {
        remove_unplanned_recursive_invocation_markers(&mut example.ir, &retained_recursive_calls);
    }
    bind_oriented_runtime_marker_locations(root, &program, &mut oriented_plan)?;
    Ok((program, join_plan, oriented_plan))
}

fn remove_unplanned_recursive_invocation_markers(
    expression: &mut RuntimeExpr,
    retained: &BTreeSet<u64>,
) {
    loop {
        let remove = matches!(
            expression,
            RuntimeExpr::CheckedRecursiveInvocation {
                call_template_id,
                ..
            } if !retained.contains(call_template_id)
        );
        if !remove {
            break;
        }
        let RuntimeExpr::CheckedRecursiveInvocation { body, .. } =
            std::mem::replace(expression, RuntimeExpr::Value(RuntimeValue::Unknown))
        else {
            unreachable!("removal predicate selects one recursive marker")
        };
        *expression = *body;
    }

    match expression {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. }
        | RuntimeExpr::Project { record: body, .. }
        | RuntimeExpr::Closure { body, .. } => {
            remove_unplanned_recursive_invocation_markers(body, retained)
        }
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            for capture in captures {
                remove_unplanned_recursive_invocation_markers(capture, retained);
            }
            remove_unplanned_recursive_invocation_markers(body, retained);
        }
        RuntimeExpr::Let { value, body } => {
            remove_unplanned_recursive_invocation_markers(value, retained);
            remove_unplanned_recursive_invocation_markers(body, retained);
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            remove_unplanned_recursive_invocation_markers(scrutinee, retained);
            remove_unplanned_recursive_invocation_markers(then_expr, retained);
            remove_unplanned_recursive_invocation_markers(else_expr, retained);
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for argument in args {
                remove_unplanned_recursive_invocation_markers(argument, retained);
            }
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            remove_unplanned_recursive_invocation_markers(scrutinee, retained);
            for case in cases {
                remove_unplanned_recursive_invocation_markers(&mut case.body, retained);
            }
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            remove_unplanned_recursive_invocation_markers(scrutinee, retained);
            for case in cases {
                remove_unplanned_recursive_invocation_markers(&mut case.body, retained);
            }
        }
        RuntimeExpr::Record { fields } => {
            for (_, field) in fields {
                remove_unplanned_recursive_invocation_markers(field, retained);
            }
        }
        RuntimeExpr::Call { callee, args } => {
            remove_unplanned_recursive_invocation_markers(callee, retained);
            for argument in args {
                remove_unplanned_recursive_invocation_markers(argument, retained);
            }
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                remove_unplanned_recursive_invocation_markers(&mut capability.value, retained);
            }
            for argument in args {
                remove_unplanned_recursive_invocation_markers(argument, retained);
            }
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => {}
    }
}

#[derive(Default)]
struct OrientedRuntimeMarkerLocations {
    recursive_calls: BTreeMap<(u64, Vec<u64>), Vec<CheckedRuntimeMarkerLocationV1>>,
    computational_ih_slots: BTreeMap<(u64, Vec<u64>), Vec<CheckedRuntimeMarkerLocationV1>>,
    computational_ih_calls: BTreeMap<(u64, Vec<u64>), Vec<CheckedRuntimeMarkerLocationV1>>,
}

fn bind_oriented_runtime_marker_locations(
    root: &StableSymbol,
    program: &RuntimeProgram,
    plan: &mut OrientedSubcontinuationPlanV1,
) -> Result<(), ErasureError> {
    let mut locations = OrientedRuntimeMarkerLocations::default();
    for declaration in &program.declarations {
        let RuntimeDeclarationKind::Transparent { body } = &declaration.kind else {
            continue;
        };
        collect_oriented_runtime_marker_locations(
            body,
            &declaration.symbol,
            &mut Vec::new(),
            &mut locations,
        )?;
    }

    for call in &mut plan.recursive_calls {
        call.runtime_marker_locations = locations
            .recursive_calls
            .remove(&(call.call_template_id, call.checked_occurrence_path.clone()))
            .ok_or_else(|| {
                expression_lowering_error(
                    root,
                    "checked_oriented_marker_location",
                    format!(
                        "recursive-call template {} has no exact Runtime occurrence",
                        call.call_template_id
                    ),
                )
            })?;
        call.runtime_marker_locations.sort();
        call.occurrence_binding_fingerprint =
            compiler_private_recursive_call_binding_fingerprint(call);
    }
    for slot in &mut plan.computational_ih_slots {
        slot.runtime_marker_locations = locations
            .computational_ih_slots
            .remove(&(slot.slot_template_id, slot.checked_occurrence_path.clone()))
            .ok_or_else(|| {
                expression_lowering_error(
                    root,
                    "checked_oriented_marker_location",
                    format!(
                        "computational-IH slot template {} has no exact Runtime occurrence",
                        slot.slot_template_id
                    ),
                )
            })?;
        slot.runtime_marker_locations.sort();
        slot.occurrence_binding_fingerprint =
            compiler_private_computational_ih_slot_binding_fingerprint(slot);
    }
    for call in &mut plan.computational_ih_calls {
        call.runtime_marker_locations = locations
            .computational_ih_calls
            .remove(&(call.call_template_id, call.checked_occurrence_path.clone()))
            .ok_or_else(|| {
                expression_lowering_error(
                    root,
                    "checked_oriented_marker_location",
                    format!(
                        "computational-IH call template {} has no exact Runtime occurrence",
                        call.call_template_id
                    ),
                )
            })?;
        call.runtime_marker_locations.sort();
        call.occurrence_binding_fingerprint =
            compiler_private_computational_ih_call_binding_fingerprint(call);
    }
    if !locations.recursive_calls.is_empty()
        || !locations.computational_ih_slots.is_empty()
        || !locations.computational_ih_calls.is_empty()
    {
        return Err(expression_lowering_error(
            root,
            "checked_oriented_marker_location",
            "Runtime IR contains an oriented marker with no exact checked template",
        ));
    }
    plan.validate().map_err(|reason| {
        expression_lowering_error(root, "checked_oriented_marker_location", reason)
    })
}

fn collect_oriented_runtime_marker_locations(
    expression: &RuntimeExpr,
    declaration: &str,
    runtime_path: &mut Vec<u64>,
    locations: &mut OrientedRuntimeMarkerLocations,
) -> Result<(), ErasureError> {
    let location = || CheckedRuntimeMarkerLocationV1 {
        declaration: declaration.to_string(),
        runtime_path: runtime_path.clone(),
    };
    match expression {
        RuntimeExpr::CheckedRecursiveInvocation {
            call_template_id,
            checked_occurrence_path,
            body,
        } => {
            locations
                .recursive_calls
                .entry((*call_template_id, checked_occurrence_path.clone()))
                .or_default()
                .push(location());
            collect_oriented_runtime_marker_child(body, declaration, runtime_path, locations, 0)
        }
        RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids,
            checked_occurrence_paths,
            body,
        } => {
            assert_eq!(
                slot_template_ids.len(),
                checked_occurrence_paths.len(),
                "erasure emits one checked occurrence path per computational IH slot marker"
            );
            for (slot_template_id, checked_occurrence_path) in
                slot_template_ids.iter().zip(checked_occurrence_paths)
            {
                locations
                    .computational_ih_slots
                    .entry((*slot_template_id, checked_occurrence_path.clone()))
                    .or_default()
                    .push(location());
            }
            collect_oriented_runtime_marker_child(body, declaration, runtime_path, locations, 0)
        }
        RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id,
            checked_occurrence_path,
            body,
        } => {
            locations
                .computational_ih_calls
                .entry((*call_template_id, checked_occurrence_path.clone()))
                .or_default()
                .push(location());
            collect_oriented_runtime_marker_child(body, declaration, runtime_path, locations, 0)
        }
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. } => {
            collect_oriented_runtime_marker_child(body, declaration, runtime_path, locations, 0)
        }
        RuntimeExpr::Project { record, .. } => {
            collect_oriented_runtime_marker_child(record, declaration, runtime_path, locations, 1)
        }
        RuntimeExpr::Closure { body, .. } => {
            collect_oriented_runtime_marker_child(body, declaration, runtime_path, locations, 2)
        }
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            for (index, capture) in captures.iter().enumerate() {
                collect_oriented_runtime_marker_child(
                    capture,
                    declaration,
                    runtime_path,
                    locations,
                    10 + index as u64,
                )?;
            }
            collect_oriented_runtime_marker_child(body, declaration, runtime_path, locations, 3)
        }
        RuntimeExpr::Let { value, body } => {
            collect_oriented_runtime_marker_child(value, declaration, runtime_path, locations, 0)?;
            collect_oriented_runtime_marker_child(body, declaration, runtime_path, locations, 1)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            collect_oriented_runtime_marker_child(
                scrutinee,
                declaration,
                runtime_path,
                locations,
                0,
            )?;
            collect_oriented_runtime_marker_child(
                then_expr,
                declaration,
                runtime_path,
                locations,
                1,
            )?;
            collect_oriented_runtime_marker_child(
                else_expr,
                declaration,
                runtime_path,
                locations,
                2,
            )
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for (index, argument) in args.iter().enumerate() {
                collect_oriented_runtime_marker_child(
                    argument,
                    declaration,
                    runtime_path,
                    locations,
                    index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            collect_oriented_runtime_marker_child(
                scrutinee,
                declaration,
                runtime_path,
                locations,
                0,
            )?;
            for (index, case) in cases.iter().enumerate() {
                collect_oriented_runtime_marker_child(
                    &case.body,
                    declaration,
                    runtime_path,
                    locations,
                    1 + index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            collect_oriented_runtime_marker_child(
                scrutinee,
                declaration,
                runtime_path,
                locations,
                0,
            )?;
            for (index, case) in cases.iter().enumerate() {
                collect_oriented_runtime_marker_child(
                    &case.body,
                    declaration,
                    runtime_path,
                    locations,
                    1 + index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::Record { fields } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                collect_oriented_runtime_marker_child(
                    value,
                    declaration,
                    runtime_path,
                    locations,
                    index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::Call { callee, args } => {
            collect_oriented_runtime_marker_child(callee, declaration, runtime_path, locations, 0)?;
            for (index, argument) in args.iter().enumerate() {
                collect_oriented_runtime_marker_child(
                    argument,
                    declaration,
                    runtime_path,
                    locations,
                    1 + index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                collect_oriented_runtime_marker_child(
                    &capability.value,
                    declaration,
                    runtime_path,
                    locations,
                    0,
                )?;
            }
            for (index, argument) in args.iter().enumerate() {
                collect_oriented_runtime_marker_child(
                    argument,
                    declaration,
                    runtime_path,
                    locations,
                    1 + index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => Ok(()),
    }
}

fn collect_oriented_runtime_marker_child(
    expression: &RuntimeExpr,
    declaration: &str,
    runtime_path: &mut Vec<u64>,
    locations: &mut OrientedRuntimeMarkerLocations,
    edge: u64,
) -> Result<(), ErasureError> {
    runtime_path.push(edge);
    let result =
        collect_oriented_runtime_marker_locations(expression, declaration, runtime_path, locations);
    runtime_path.pop();
    result
}

fn lower_checked_host_root(
    package: &CheckedCorePackage,
    target_closure: &[StableSymbol],
    root: &StableSymbol,
    spine: &CheckedHostSpineV1,
    mut native_plans: Option<&mut NativeLoweringPlanCollector>,
) -> Result<RuntimeDeclarationKind, ErasureError> {
    let semantic = &package.artifact.semantic;
    // Decode exactly the finite executable declaration closure. Recursive
    // edges remain declaration references; they are never unfolded while the
    // checked HostIO tree is deforested.
    let reachable_declarations = checked_host_body_view_symbols(semantic, target_closure);
    let selection = CheckedCoreBodyViewSelection {
        package_identity: package.header.package_identity.clone(),
        package_core_semantic_hash: package.core_semantic_hash,
        package_artifact_hash: package.artifact_hash,
        target_symbol: root.clone(),
        reachable_declarations,
        external_symbols: external_declaration_symbols(semantic),
        dependency_semantic_hashes: semantic.dependency_semantic_hashes.clone(),
    };
    let declarations = checked_host_declaration_closure(package, &selection, root)?;
    let declaration = declarations.get(root).ok_or_else(|| {
        expression_lowering_error(
            root,
            "missing_expression_body_view",
            "body view did not return the checked host root",
        )
    })?;
    let CheckedCoreBodyTerm::Lambda { body, .. } = &declaration.body else {
        return Err(expression_lowering_error(
            root,
            "host_root_abi_shape",
            "checked host root must accept ProcessInput",
        ));
    };
    let CheckedCoreBodyTerm::Lambda { body, .. } = body.as_ref() else {
        return Err(expression_lowering_error(
            root,
            "host_root_abi_shape",
            "checked host root must accept ProgramCaps",
        ));
    };
    if let Some(native_plans) = native_plans.as_deref_mut() {
        native_plans
            .joins
            .record_root_exit_answer(root, &declaration.checked_type);
    }
    let mut stack = vec![root.clone()];
    let lowered = lower_checked_host_computation(
        body,
        &declarations,
        semantic,
        &mut stack,
        root,
        2,
        spine,
        None,
        &[1],
        native_plans,
        None,
    )?;
    Ok(RuntimeDeclarationKind::Transparent {
        body: RuntimeExpr::Closure {
            captures: Vec::new(),
            params: vec!["process_input".to_string(), "program_caps".to_string()],
            body: Box::new(lowered),
        },
    })
}

/// Lower a runtime value that appears inside the checked HostIO producer.
///
/// HostIO-valued callback lambdas are part of the same checked continuation
/// boundary as their caller.  Descend through those lambdas with a transactional
/// copy of the native plans; a genuinely pure lambda falls back to ordinary
/// erasure without leaking partially collected metadata.
#[allow(clippy::too_many_arguments)]
fn lower_checked_host_value(
    term: &CheckedCoreBodyTerm,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root: &StableSymbol,
    context_depth: usize,
    spine: &CheckedHostSpineV1,
    branch_remap: Option<&BranchBinderRemap>,
    path: &[u64],
    native_plans: Option<&mut NativeLoweringPlanCollector>,
    parent_oriented_frame: Option<u64>,
) -> Result<RuntimeExpr, ErasureError> {
    if let Some(native_plans) = native_plans {
        let mut trial_plans = native_plans.clone();
        let (candidate, candidate_depth, entered_remap, is_lambda) =
            if let CheckedCoreBodyTerm::Lambda { body, .. } = term {
                (
                    body.as_ref(),
                    context_depth + 1,
                    branch_remap.map(BranchBinderRemap::enter_binding),
                    true,
                )
            } else {
                (term, context_depth, branch_remap.cloned(), false)
            };
        match lower_checked_host_computation(
            candidate,
            declarations,
            semantic,
            stack,
            root,
            candidate_depth,
            spine,
            entered_remap.as_ref(),
            path,
            Some(&mut trial_plans),
            parent_oriented_frame,
        ) {
            Ok(body) => {
                *native_plans = trial_plans;
                if is_lambda {
                    let runtime_depth = branch_remap
                        .map(|remap| remap.runtime_depth(context_depth))
                        .unwrap_or(context_depth);
                    Ok(RuntimeExpr::LexicalClosure {
                        captures: (0..runtime_depth)
                            .map(|index| RuntimeExpr::Var(index as u32))
                            .collect(),
                        params: vec!["arg0".to_string()],
                        body: Box::new(body),
                    })
                } else {
                    Ok(body)
                }
            }
            Err(ErasureError::ExpressionLowering { lane, .. })
                if lane == "unrecognized_checked_host_computation" =>
            {
                lower_body_term_with_plans(
                    term,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth,
                    branch_remap,
                    path,
                    native_plans,
                    parent_oriented_frame,
                )
            }
            Err(error) => Err(error),
        }
    } else {
        lower_body_term_inner(
            term,
            declarations,
            semantic,
            stack,
            root,
            context_depth,
            branch_remap,
        )
    }
}

/// Ordinary checked-value erasure with the native answer plan threaded through
/// every expression form that can contain a computational eliminator.  This is
/// the pre-erasure closure needed by HostIO-valued callbacks: generic erasure
/// remains unchanged for non-native packages.
#[allow(clippy::too_many_arguments)]
fn lower_body_term_with_plans(
    term: &CheckedCoreBodyTerm,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<&BranchBinderRemap>,
    path: &[u64],
    native_plans: &mut NativeLoweringPlanCollector,
    parent_oriented_frame: Option<u64>,
) -> Result<RuntimeExpr, ErasureError> {
    let owner = stack
        .last()
        .expect("expression lowering stack always has an owner")
        .clone();
    if let Some((slot_template_id, arguments)) =
        computational_ih_application_spine(term, branch_remap)
    {
        let call_template_id = native_plans.consume_computational_ih_call(
            &owner,
            slot_template_id,
            arguments.len(),
            ComputationalIHConsumptionRoute::OrdinaryApplication,
            path,
            parent_oriented_frame,
        )?;
        let runtime_index = match term_application_head(term) {
            CheckedCoreBodyTerm::Variable { de_bruijn_index } => branch_remap
                .and_then(|remap| remap.runtime_index(*de_bruijn_index))
                .ok_or_else(|| {
                    expression_lowering_error(
                        root,
                        "checked_computational_ih_runtime_binding",
                        "checked computational IH has no runtime binder",
                    )
                })?,
            _ => unreachable!("computational IH spine has a variable head"),
        };
        let callee = RuntimeExpr::Var(u32::try_from(runtime_index).map_err(|_| {
            expression_lowering_error(
                root,
                "variable_index_overflow",
                "computational IH runtime index does not fit runtime IR",
            )
        })?);
        let mut args = Vec::with_capacity(arguments.len());
        for (index, argument) in arguments.into_iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.extend([9, index as u64]);
            args.push(lower_body_term_with_plans(
                argument,
                declarations,
                semantic,
                stack,
                root,
                context_depth,
                branch_remap,
                &child_path,
                native_plans,
                parent_oriented_frame,
            )?);
        }
        let body = if args.is_empty() {
            callee
        } else {
            RuntimeExpr::Call {
                callee: Box::new(callee),
                args,
            }
        };
        return Ok(RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id,
            checked_occurrence_path: path.to_vec(),
            body: Box::new(body),
        });
    }
    if let Some((view, arguments)) = recursive_application_spine(term) {
        let call_template_id = native_plans.consume_recursive_invocation(
            &owner,
            view,
            arguments.len(),
            path,
            parent_oriented_frame,
        )?;
        let callee = lower_recursive_declaration_call(view, declarations, root)?;
        let mut args = Vec::with_capacity(arguments.len());
        for (index, argument) in arguments.into_iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.extend([10, index as u64]);
            args.push(lower_body_term_with_plans(
                argument,
                declarations,
                semantic,
                stack,
                root,
                context_depth,
                branch_remap,
                &child_path,
                native_plans,
                parent_oriented_frame,
            )?);
        }
        return Ok(RuntimeExpr::CheckedRecursiveInvocation {
            call_template_id,
            checked_occurrence_path: path.to_vec(),
            body: Box::new(RuntimeExpr::Call {
                callee: Box::new(callee),
                args,
            }),
        });
    }
    if let Some((symbol, level_args, arguments)) = direct_application_spine(term) {
        reject_level_args(root, level_args)?;
        if let Some(declaration) = declarations.get(symbol) {
            let mut body = &declaration.body;
            let mut parameter_count = 0usize;
            while parameter_count < arguments.len() {
                let CheckedCoreBodyTerm::Lambda { body: inner, .. } = body else {
                    break;
                };
                parameter_count += 1;
                body = inner;
            }
            if parameter_count == arguments.len() && !admitted_recursive_member(semantic, symbol) {
                if stack.contains(symbol) {
                    return Err(expression_lowering_error(
                        root,
                        "direct_call_cycle",
                        format!("direct declaration call cycle from {owner} reaches {symbol}"),
                    ));
                }
                let mut values = Vec::with_capacity(arguments.len());
                for (index, argument) in arguments.iter().enumerate() {
                    let mut child_path = path.to_vec();
                    child_path.extend([11, index as u64]);
                    values.push(lower_body_term_with_plans(
                        argument,
                        declarations,
                        semantic,
                        stack,
                        root,
                        context_depth,
                        branch_remap,
                        &child_path,
                        native_plans,
                        parent_oriented_frame,
                    )?);
                }
                let mut inner_remap = branch_remap.cloned();
                for _ in 0..parameter_count {
                    inner_remap = inner_remap.map(|remap| remap.enter_binding());
                }
                stack.push(symbol.clone());
                let mut child_path = path.to_vec();
                child_path.extend([12, ken_runtime::fnv1a_64(symbol.to_string().as_bytes())]);
                let lowered = lower_body_term_with_plans(
                    body,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth + parameter_count,
                    inner_remap.as_ref(),
                    &child_path,
                    native_plans,
                    parent_oriented_frame,
                );
                stack.pop();
                let mut lowered = lowered?;
                for (index, value) in values.into_iter().enumerate().rev() {
                    lowered = RuntimeExpr::Let {
                        value: Box::new(shift_runtime_vars(value, index as u32, 0)),
                        body: Box::new(lowered),
                    };
                }
                return Ok(lowered);
            }
        }
    }
    if let Some((constructor, arguments)) = constructor_application_spine(term) {
        reject_level_args(root, &constructor.level_args)?;
        require_expression_supported(
            root,
            &constructor.family_symbol,
            &constructor.family_lowerability,
            "data_lowerability_blocked",
        )?;
        require_expression_supported(
            root,
            &constructor.symbol,
            &constructor.constructor_lowerability,
            "constructor_lowerability_blocked",
        )?;
        if constructor.family_index_count != 0 || constructor.target_index_count != 0 {
            return Err(expression_lowering_error(
                root,
                "dependent_constructor_lowering_unsupported",
                format!(
                    "constructor {} belongs to indexed family {}",
                    constructor.symbol, constructor.family_symbol
                ),
            ));
        }
        let expected = constructor.family_parameter_count + constructor.argument_count;
        if arguments.len() != expected {
            return Err(expression_lowering_error(
                root,
                "constructor_arity_mismatch",
                format!(
                    "constructor {} expects {} family parameters plus {} runtime arguments, got {}",
                    constructor.symbol,
                    constructor.family_parameter_count,
                    constructor.argument_count,
                    arguments.len()
                ),
            ));
        }
        let runtime_arguments = &arguments[constructor.family_parameter_count..];
        let mut args = Vec::with_capacity(runtime_arguments.len());
        for (index, argument) in runtime_arguments.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.extend([13, index as u64]);
            args.push(lower_body_term_with_plans(
                argument,
                declarations,
                semantic,
                stack,
                root,
                context_depth,
                branch_remap,
                &child_path,
                native_plans,
                parent_oriented_frame,
            )?);
        }
        return Ok(RuntimeExpr::Construct {
            constructor: constructor.symbol.to_string(),
            args,
        });
    }

    match term {
        CheckedCoreBodyTerm::Lambda { body, .. } => {
            let runtime_depth = branch_remap
                .map(|remap| remap.runtime_depth(context_depth))
                .unwrap_or(context_depth);
            let mut child_path = path.to_vec();
            child_path.push(14);
            Ok(RuntimeExpr::LexicalClosure {
                captures: (0..runtime_depth)
                    .map(|index| RuntimeExpr::Var(index as u32))
                    .collect(),
                params: vec!["arg0".to_string()],
                body: Box::new(lower_body_term_with_plans(
                    body,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth + 1,
                    branch_remap.map(BranchBinderRemap::enter_binding).as_ref(),
                    &child_path,
                    native_plans,
                    parent_oriented_frame,
                )?),
            })
        }
        CheckedCoreBodyTerm::Application { function, argument } => {
            let mut function_path = path.to_vec();
            function_path.push(15);
            let mut argument_path = path.to_vec();
            argument_path.push(16);
            Ok(RuntimeExpr::Call {
                callee: Box::new(lower_body_term_with_plans(
                    function,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth,
                    branch_remap,
                    &function_path,
                    native_plans,
                    parent_oriented_frame,
                )?),
                args: vec![lower_body_term_with_plans(
                    argument,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth,
                    branch_remap,
                    &argument_path,
                    native_plans,
                    parent_oriented_frame,
                )?],
            })
        }
        CheckedCoreBodyTerm::Let { value, body, .. } => {
            let mut value_path = path.to_vec();
            value_path.push(17);
            let mut body_path = path.to_vec();
            body_path.push(18);
            Ok(RuntimeExpr::Let {
                value: Box::new(lower_body_term_with_plans(
                    value,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth,
                    branch_remap,
                    &value_path,
                    native_plans,
                    parent_oriented_frame,
                )?),
                body: Box::new(lower_body_term_with_plans(
                    body,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth + 1,
                    branch_remap.map(BranchBinderRemap::enter_binding).as_ref(),
                    &body_path,
                    native_plans,
                    parent_oriented_frame,
                )?),
            })
        }
        CheckedCoreBodyTerm::Match(view) => {
            reject_level_args(root, &view.level_args)?;
            if !view.indices.is_empty() {
                return Err(expression_lowering_error(
                    root,
                    "unsupported_dependent_motive",
                    format!("match over {} carries runtime indices", view.family_symbol),
                ));
            }
            let computational = match_uses_computational_recursive_hypothesis(view, root)?;
            let pending = if computational {
                native_plans
                    .oriented
                    .begin_match(&owner, path, parent_oriented_frame, view)?
            } else {
                None
            };
            let nested_parent = pending
                .as_ref()
                .map(|pending| pending.frame_id)
                .or(parent_oriented_frame);
            let branch_slot_templates = if let Some(pending) = pending.as_ref() {
                native_plans.consume_computational_ih_slots(&owner, view, path, pending.frame_id)?
            } else {
                vec![Vec::new(); view.branches.len()]
            };
            let mut scrutinee_path = path.to_vec();
            scrutinee_path.push(19);
            let scrutinee = Box::new(lower_body_term_with_plans(
                &view.scrutinee,
                declarations,
                semantic,
                stack,
                root,
                context_depth,
                branch_remap,
                &scrutinee_path,
                native_plans,
                nested_parent,
            )?);
            let mut cases = Vec::with_capacity(view.branches.len());
            for (branch_index, branch) in view.branches.iter().enumerate() {
                let constructor = &branch.constructor;
                reject_level_args(root, &constructor.level_args)?;
                require_expression_supported(
                    root,
                    &constructor.family_symbol,
                    &constructor.family_lowerability,
                    "data_lowerability_blocked",
                )?;
                require_expression_supported(
                    root,
                    &constructor.symbol,
                    &constructor.constructor_lowerability,
                    "constructor_lowerability_blocked",
                )?;
                if constructor.family_index_count != 0 || constructor.target_index_count != 0 {
                    return Err(expression_lowering_error(
                        root,
                        "dependent_constructor_lowering_unsupported",
                        format!(
                            "match branch constructor {} belongs to indexed family {}",
                            constructor.symbol, constructor.family_symbol
                        ),
                    ));
                }
                let erased_count = constructor.recursive_positions.len();
                let source_binders = constructor.argument_count + erased_count;
                let method = peel_match_branch_method(
                    &branch.method,
                    source_binders,
                    root,
                    &constructor.symbol,
                )?;
                let slot_markers = branch_slot_templates[branch_index].clone();
                let slot_templates = slot_markers
                    .iter()
                    .map(|(slot_template_id, _)| *slot_template_id)
                    .collect::<Vec<_>>();
                let checked_occurrence_paths = slot_markers
                    .into_iter()
                    .map(|(_, path)| path)
                    .collect::<Vec<_>>();
                let remap = branch_remap.cloned().unwrap_or_default().enter_match(
                    constructor.argument_count,
                    erased_count,
                    computational,
                    slot_templates.clone(),
                );
                let mut branch_path = path.to_vec();
                branch_path.extend([20, branch_index as u64]);
                let body = lower_body_term_with_plans(
                    method,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth + source_binders,
                    Some(&remap),
                    &branch_path,
                    native_plans,
                    nested_parent,
                )?;
                let body = if slot_templates.is_empty() {
                    body
                } else {
                    RuntimeExpr::CheckedComputationalIHSlots {
                        slot_template_ids: slot_templates,
                        checked_occurrence_paths,
                        body: Box::new(body),
                    }
                };
                cases.push((constructor, body));
            }
            let default = RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: format!("no runtime match case selected for {}", view.family_symbol),
            };
            let runtime = if computational {
                RuntimeExpr::ComputationalMatch {
                    scrutinee,
                    cases: cases
                        .into_iter()
                        .map(|(constructor, body)| RuntimeComputationalMatchCase {
                            constructor: constructor.symbol.to_string(),
                            argument_binders: constructor.argument_count,
                            recursive_positions: constructor.recursive_positions.clone(),
                            body,
                        })
                        .collect(),
                    default,
                }
            } else {
                RuntimeExpr::Match {
                    scrutinee,
                    cases: cases
                        .into_iter()
                        .map(|(constructor, body)| RuntimeMatchCase {
                            constructor: constructor.symbol.to_string(),
                            binders: constructor.argument_count,
                            body,
                        })
                        .collect(),
                    default,
                }
            };
            let join_site = native_plans
                .joins
                .record_match(&owner, path, view, &runtime)?;
            let runtime = if let Some(pending) = pending {
                let frame_id = native_plans.oriented.finish_match(pending, &runtime)?;
                RuntimeExpr::CheckedSubcontinuationFrame {
                    frame_id,
                    body: Box::new(runtime),
                }
            } else {
                runtime
            };
            Ok(
                join_site.map_or(runtime.clone(), |site_id| RuntimeExpr::CheckedJoinSite {
                    site_id,
                    body: Box::new(runtime),
                }),
            )
        }
        _ => lower_body_term_inner(
            term,
            declarations,
            semantic,
            stack,
            root,
            context_depth,
            branch_remap,
        ),
    }
}

fn lower_checked_host_computation(
    term: &CheckedCoreBodyTerm,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root: &StableSymbol,
    context_depth: usize,
    spine: &CheckedHostSpineV1,
    branch_remap: Option<&BranchBinderRemap>,
    path: &[u64],
    mut native_plans: Option<&mut NativeLoweringPlanCollector>,
    parent_oriented_frame: Option<u64>,
) -> Result<RuntimeExpr, ErasureError> {
    let owner = stack
        .last()
        .expect("expression lowering stack always has an owner")
        .clone();
    if let Some((slot_template_id, arguments)) =
        computational_ih_application_spine(term, branch_remap)
    {
        let plans = native_plans.as_deref_mut().ok_or_else(|| {
            expression_lowering_error(
                root,
                "checked_computational_ih_plan_missing",
                "computational IH application reached native lowering without checked metadata",
            )
        })?;
        let call_template_id = plans.consume_computational_ih_call(
            &owner,
            slot_template_id,
            arguments.len(),
            ComputationalIHConsumptionRoute::CheckedHostComputationTail,
            path,
            parent_oriented_frame,
        )?;
        let de_bruijn_index = match term_application_head(term) {
            CheckedCoreBodyTerm::Variable { de_bruijn_index } => *de_bruijn_index,
            _ => unreachable!("computational IH spine has a variable head"),
        };
        let runtime_index = branch_remap
            .and_then(|remap| remap.runtime_index(de_bruijn_index))
            .ok_or_else(|| {
                expression_lowering_error(
                    root,
                    "checked_computational_ih_runtime_binding",
                    "checked computational IH has no runtime binder",
                )
            })?;
        let mut args = Vec::with_capacity(arguments.len());
        for (argument_index, argument) in arguments.into_iter().enumerate() {
            let mut argument_path = path.to_vec();
            argument_path.extend([3, argument_index as u64]);
            args.push(lower_checked_host_value(
                argument,
                declarations,
                semantic,
                stack,
                root,
                context_depth,
                spine,
                branch_remap,
                &argument_path,
                Some(&mut *plans),
                parent_oriented_frame,
            )?);
        }
        let callee = RuntimeExpr::Var(u32::try_from(runtime_index).map_err(|_| {
            expression_lowering_error(
                root,
                "variable_index_overflow",
                "computational IH runtime index does not fit runtime IR",
            )
        })?);
        let body = if args.is_empty() {
            callee
        } else {
            RuntimeExpr::Call {
                callee: Box::new(callee),
                args,
            }
        };
        return Ok(RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id,
            checked_occurrence_path: path.to_vec(),
            body: Box::new(body),
        });
    }
    if let CheckedCoreBodyTerm::Match(view) = term {
        reject_level_args(root, &view.level_args)?;
        if !view.indices.is_empty() {
            return Err(expression_lowering_error(
                root,
                "host_match_identity",
                "checked host Match carries runtime indices",
            ));
        }
        let computational = match_uses_computational_recursive_hypothesis(view, root)?;
        let pending_oriented_frame = if computational {
            native_plans
                .as_deref_mut()
                .map(|plans| {
                    plans
                        .oriented
                        .begin_match(&owner, path, parent_oriented_frame, view)
                })
                .transpose()?
                .flatten()
        } else {
            None
        };
        let nested_oriented_parent = pending_oriented_frame
            .as_ref()
            .map(|pending| pending.frame_id)
            .or(parent_oriented_frame);
        let branch_slot_templates = if let (Some(plans), Some(pending)) =
            (native_plans.as_deref_mut(), pending_oriented_frame.as_ref())
        {
            plans.consume_computational_ih_slots(&owner, view, path, pending.frame_id)?
        } else {
            vec![Vec::new(); view.branches.len()]
        };
        let scrutinee = if let Some(plans) = native_plans.as_deref_mut() {
            let mut scrutinee_path = path.to_vec();
            scrutinee_path.push(0);
            lower_body_term_with_plans(
                &view.scrutinee,
                declarations,
                semantic,
                stack,
                root,
                context_depth,
                branch_remap,
                &scrutinee_path,
                plans,
                nested_oriented_parent,
            )?
        } else {
            lower_body_term_inner(
                &view.scrutinee,
                declarations,
                semantic,
                stack,
                root,
                context_depth,
                branch_remap,
            )?
        };
        let mut cases = Vec::with_capacity(view.branches.len());
        for (branch_index, branch) in view.branches.iter().enumerate() {
            let constructor = &branch.constructor;
            reject_level_args(root, &constructor.level_args)?;
            require_expression_supported(
                root,
                &constructor.family_symbol,
                &constructor.family_lowerability,
                "data_lowerability_blocked",
            )?;
            require_expression_supported(
                root,
                &constructor.symbol,
                &constructor.constructor_lowerability,
                "constructor_lowerability_blocked",
            )?;
            if constructor.family_index_count != 0 || constructor.target_index_count != 0 {
                return Err(expression_lowering_error(
                    root,
                    "host_match_identity",
                    "checked host Match branch belongs to an indexed family",
                ));
            }
            let erased_count = constructor.recursive_positions.len();
            let source_binders = constructor.argument_count + erased_count;
            let method = peel_match_branch_method(
                &branch.method,
                source_binders,
                root,
                &constructor.symbol,
            )?;
            let slot_markers = branch_slot_templates[branch_index].clone();
            let slot_templates = slot_markers
                .iter()
                .map(|(slot_template_id, _)| *slot_template_id)
                .collect::<Vec<_>>();
            let checked_occurrence_paths = slot_markers
                .into_iter()
                .map(|(_, path)| path)
                .collect::<Vec<_>>();
            let remap = branch_remap.cloned().unwrap_or_default().enter_match(
                constructor.argument_count,
                erased_count,
                computational,
                slot_templates.clone(),
            );
            let body = lower_checked_host_computation(
                method,
                declarations,
                semantic,
                stack,
                root,
                context_depth + source_binders,
                spine,
                Some(&remap),
                &{
                    let mut p = path.to_vec();
                    p.extend([1, branch_index as u64]);
                    p
                },
                native_plans.as_deref_mut(),
                nested_oriented_parent,
            )?;
            let body = if slot_templates.is_empty() {
                body
            } else {
                RuntimeExpr::CheckedComputationalIHSlots {
                    slot_template_ids: slot_templates,
                    checked_occurrence_paths,
                    body: Box::new(body),
                }
            };
            cases.push((constructor, body));
        }
        let default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "checked HostIO match had no constructor arm".to_string(),
        };
        let runtime = if computational {
            RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(scrutinee),
                cases: cases
                    .into_iter()
                    .map(|(constructor, body)| RuntimeComputationalMatchCase {
                        constructor: constructor.symbol.to_string(),
                        argument_binders: constructor.argument_count,
                        recursive_positions: constructor.recursive_positions.clone(),
                        body,
                    })
                    .collect(),
                default,
            }
        } else {
            RuntimeExpr::Match {
                scrutinee: Box::new(scrutinee),
                cases: cases
                    .into_iter()
                    .map(|(constructor, body)| RuntimeMatchCase {
                        constructor: constructor.symbol.to_string(),
                        binders: constructor.argument_count,
                        body,
                    })
                    .collect(),
                default,
            }
        };
        let join_site = native_plans
            .as_deref_mut()
            .map(|plans| plans.joins.record_match(root, path, view, &runtime))
            .transpose()?
            .flatten();
        let runtime = if let Some(pending) = pending_oriented_frame {
            let frame_id = native_plans
                .as_deref_mut()
                .expect("pending oriented frame has its collector")
                .oriented
                .finish_match(pending, &runtime)?;
            RuntimeExpr::CheckedSubcontinuationFrame {
                frame_id,
                body: Box::new(runtime),
            }
        } else {
            runtime
        };
        return Ok(match join_site {
            Some(site_id) => RuntimeExpr::CheckedJoinSite {
                site_id,
                body: Box::new(runtime),
            },
            None => runtime,
        });
    }
    if let Some((symbol, level_args, arguments)) = direct_application_spine(term) {
        reject_level_args(root, level_args)?;
        if let Some(declaration) = declarations.get(symbol) {
            let mut declaration_body = &declaration.body;
            let mut parameter_count = 0usize;
            while parameter_count < arguments.len() {
                let CheckedCoreBodyTerm::Lambda { body, .. } = declaration_body else {
                    break;
                };
                parameter_count += 1;
                declaration_body = body;
            }
            if parameter_count == arguments.len()
                && !stack.contains(symbol)
                && !admitted_recursive_member(semantic, symbol)
            {
                let values = arguments
                    .iter()
                    .enumerate()
                    .map(|(argument_index, argument)| {
                        let mut argument_path = path.to_vec();
                        argument_path.extend([4, argument_index as u64]);
                        lower_checked_host_value(
                            argument,
                            declarations,
                            semantic,
                            stack,
                            root,
                            context_depth,
                            spine,
                            branch_remap,
                            &argument_path,
                            native_plans.as_deref_mut(),
                            parent_oriented_frame,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut inner_remap = branch_remap.cloned();
                for _ in 0..parameter_count {
                    inner_remap = inner_remap.map(|remap| remap.enter_binding());
                }
                stack.push(symbol.clone());
                let lowered = lower_checked_host_computation(
                    declaration_body,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth + parameter_count,
                    spine,
                    inner_remap.as_ref(),
                    &{
                        let mut p = path.to_vec();
                        p.extend([2, ken_runtime::fnv1a_64(symbol.to_string().as_bytes())]);
                        p
                    },
                    native_plans.as_deref_mut(),
                    parent_oriented_frame,
                );
                stack.pop();
                let mut lowered = lowered?;
                for (index, value) in values.into_iter().enumerate().rev() {
                    lowered = RuntimeExpr::Let {
                        value: Box::new(shift_runtime_vars(value, index as u32, 0)),
                        body: Box::new(lowered),
                    };
                }
                return Ok(lowered);
            }
        }
        if admitted_recursive_member(semantic, symbol) {
            return if let Some(plans) = native_plans.as_deref_mut() {
                lower_body_term_with_plans(
                    term,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth,
                    branch_remap,
                    path,
                    plans,
                    parent_oriented_frame,
                )
            } else {
                lower_body_term_inner(
                    term,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth,
                    branch_remap,
                )
            };
        }
    }
    if let Some((constructor, args)) = constructor_application_spine(term) {
        if constructor.symbol == spine.ret {
            let value = args.last().ok_or_else(|| {
                expression_lowering_error(root, "host_ret_arity", "Ret is missing its value")
            })?;
            return lower_body_term_inner(
                value,
                declarations,
                semantic,
                stack,
                root,
                context_depth,
                branch_remap,
            );
        }
        if constructor.symbol == spine.vis {
            if args.len() < 2 {
                return Err(expression_lowering_error(
                    root,
                    "host_vis_arity",
                    "Vis is missing its operation or continuation",
                ));
            }
            let operation_term = args[args.len() - 2];
            let continuation = args[args.len() - 1];
            let continuation_body = if let CheckedCoreBodyTerm::Lambda { body, .. } = continuation {
                lower_checked_host_computation(
                    body,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth + 1,
                    spine,
                    branch_remap.map(BranchBinderRemap::enter_binding).as_ref(),
                    &{
                        let mut p = path.to_vec();
                        p.push(3);
                        p
                    },
                    native_plans.as_deref_mut(),
                    parent_oriented_frame,
                )?
            } else if let Some((slot_template_id, arguments)) =
                computational_ih_application_spine(continuation, branch_remap)
            {
                let plans = native_plans.as_deref_mut().ok_or_else(|| {
                    expression_lowering_error(
                        root,
                        "checked_computational_ih_plan_missing",
                        "computational IH continuation reached native lowering without checked metadata",
                    )
                })?;
                let mut continuation_path = path.to_vec();
                continuation_path.push(3);
                // ⭐ The Host-`Vis` route appends the host result below, so the
                // application it emits is one operand longer than the one the
                // source wrote. ⛔ The seed binding still uses the source count;
                // the extra operand travels as the ROUTE, never as a number.
                let call_template_id = plans.consume_computational_ih_call(
                    &owner,
                    slot_template_id,
                    arguments.len(),
                    ComputationalIHConsumptionRoute::CheckedHostVisContinuation,
                    &continuation_path,
                    parent_oriented_frame,
                )?;
                let de_bruijn_index = match term_application_head(continuation) {
                    CheckedCoreBodyTerm::Variable { de_bruijn_index } => *de_bruijn_index,
                    _ => unreachable!("computational IH spine has a variable head"),
                };
                let runtime_index = branch_remap
                    .and_then(|remap| remap.runtime_index(de_bruijn_index))
                    .ok_or_else(|| {
                        expression_lowering_error(
                            root,
                            "checked_computational_ih_runtime_binding",
                            "checked computational IH continuation has no runtime binder",
                        )
                    })?;
                let callee = shift_runtime_vars(
                    RuntimeExpr::Var(u32::try_from(runtime_index).map_err(|_| {
                        expression_lowering_error(
                            root,
                            "variable_index_overflow",
                            "computational IH runtime index does not fit runtime IR",
                        )
                    })?),
                    1,
                    0,
                );
                let mut args = Vec::with_capacity(arguments.len() + 1);
                for (argument_index, argument) in arguments.into_iter().enumerate() {
                    let mut argument_path = continuation_path.clone();
                    argument_path.extend([3, argument_index as u64]);
                    args.push(shift_runtime_vars(
                        lower_checked_host_value(
                            argument,
                            declarations,
                            semantic,
                            stack,
                            root,
                            context_depth,
                            spine,
                            branch_remap,
                            &argument_path,
                            Some(&mut *plans),
                            parent_oriented_frame,
                        )?,
                        1,
                        0,
                    ));
                }
                // **The injected host result** -- the one operand
                // [`ComputationalIHConsumptionRoute::CheckedHostVisContinuation`]
                // declares, and the reason this route's template arity exceeds
                // its source binding count by exactly one.
                //
                // ⛔ **No local assertion is written here on purpose.** A check
                // that the emitted length equals `arguments.len() + 1` would
                // compare this route's arithmetic against itself, and would go
                // on passing if the template were built from a different number
                // entirely. The independent oracle is the Runtime marker gate,
                // which compares this `Call`'s length against the arity in the
                // plan -- a value produced by the other side of the seam, and
                // one that refuses before a function is defined.
                args.push(RuntimeExpr::Var(0));
                RuntimeExpr::CheckedComputationalIHInvocation {
                    call_template_id,
                    checked_occurrence_path: continuation_path,
                    body: Box::new(RuntimeExpr::Call {
                        callee: Box::new(callee),
                        args,
                    }),
                }
            } else {
                let callee = lower_body_term_inner(
                    continuation,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth,
                    branch_remap,
                )?;
                RuntimeExpr::Call {
                    callee: Box::new(shift_runtime_vars(callee, 1, 0)),
                    args: vec![RuntimeExpr::Var(0)],
                }
            };
            let decoded = match select_checked_host_operation(operation_term, spine, root, true)? {
                CheckedHostOperationSelection::Static(decoded) => decoded,
                CheckedHostOperationSelection::RuntimeSelected => {
                    let operation = lower_body_term_inner(
                        operation_term,
                        declarations,
                        semantic,
                        stack,
                        root,
                        context_depth,
                        branch_remap,
                    )?;
                    return lower_runtime_selected_host_operation(
                        operation,
                        continuation_body,
                        semantic,
                        spine,
                        root,
                    );
                }
            };
            let runtime_args = &decoded.args[decoded.constructor.family_parameter_count..];
            let (capability, semantic_args) =
                if static_host_operation_requires_capability(decoded.operation) {
                    let (cap, rest) = runtime_args.split_first().ok_or_else(|| {
                        expression_lowering_error(
                            root,
                            "host_capability_shape",
                            "FS operation is missing its live capability operand",
                        )
                    })?;
                    let value = lower_body_term_inner(
                        cap,
                        declarations,
                        semantic,
                        stack,
                        root,
                        context_depth,
                        branch_remap,
                    )?;
                    (
                        Some(RuntimeCapabilityUse {
                            identity: spine.capability.to_string(),
                            value: Box::new(value),
                        }),
                        rest,
                    )
                } else {
                    (None, runtime_args)
                };
            // `PrivateResourceRelease` is indexed by its resource kind in the
            // checked family, but that index is type-level protocol evidence,
            // not a second runtime operand.  The canonical host operation still
            // receives exactly the opaque resource token it has always used.
            let semantic_args = if decoded.operation == ken_host::HostOpV1::ResourceRelease {
                semantic_args.get(1..).ok_or_else(|| {
                    expression_lowering_error(
                        root,
                        "resource_release_shape",
                        "ResourceRelease is missing its indexed resource operand",
                    )
                })?
            } else {
                semantic_args
            };
            let args = semantic_args
                .iter()
                .map(|argument| {
                    lower_body_term_inner(
                        argument,
                        declarations,
                        semantic,
                        stack,
                        root,
                        context_depth,
                        branch_remap,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::Effect {
                    family: decoded.constructor.family_symbol.to_string(),
                    operation: decoded.operation,
                    capability,
                    args,
                }),
                body: Box::new(continuation_body),
            });
        }
    }
    if let Some((view, arguments)) = recursive_application_spine(term) {
        let call_template_id = native_plans
            .as_deref_mut()
            .map(|plans| {
                plans.consume_recursive_invocation(
                    &owner,
                    view,
                    arguments.len(),
                    path,
                    parent_oriented_frame,
                )
            })
            .transpose()?;
        let callee = lower_recursive_declaration_call(view, declarations, root)?;
        let args = arguments
            .into_iter()
            .enumerate()
            .map(|(argument_index, argument)| {
                let mut argument_path = path.to_vec();
                argument_path.extend([5, argument_index as u64]);
                lower_checked_host_value(
                    argument,
                    declarations,
                    semantic,
                    stack,
                    root,
                    context_depth,
                    spine,
                    branch_remap,
                    &argument_path,
                    native_plans.as_deref_mut(),
                    parent_oriented_frame,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let call = RuntimeExpr::Call {
            callee: Box::new(callee),
            args,
        };
        return Ok(call_template_id.map_or(call.clone(), |call_template_id| {
            RuntimeExpr::CheckedRecursiveInvocation {
                call_template_id,
                checked_occurrence_path: path.to_vec(),
                body: Box::new(call),
            }
        }));
    }
    Err(expression_lowering_error(
        root,
        "unrecognized_checked_host_computation",
        "normalized HostIO body is neither identity-checked Ret nor Vis",
    ))
}

#[derive(Debug)]
struct DecodedCheckedHostOperation<'a> {
    operation: ken_host::HostOpV1,
    constructor: &'a checked_core::CheckedCoreConstructorView,
    args: Vec<&'a CheckedCoreBodyTerm>,
}

#[derive(Debug)]
enum CheckedHostOperationSelection<'a> {
    Static(DecodedCheckedHostOperation<'a>),
    RuntimeSelected,
}

fn select_checked_host_operation<'a>(
    term: &'a CheckedCoreBodyTerm,
    spine: &CheckedHostSpineV1,
    root: &StableSymbol,
    allow_runtime_selected: bool,
) -> Result<CheckedHostOperationSelection<'a>, ErasureError> {
    match decode_checked_host_operation(term, spine, root) {
        Ok(decoded) => Ok(CheckedHostOperationSelection::Static(decoded)),
        Err(_error)
            if allow_runtime_selected && matches!(term, CheckedCoreBodyTerm::Variable { .. }) =>
        {
            Ok(CheckedHostOperationSelection::RuntimeSelected)
        }
        Err(error) => Err(error),
    }
}

fn decode_checked_host_operation<'a>(
    term: &'a CheckedCoreBodyTerm,
    spine: &CheckedHostSpineV1,
    root: &StableSymbol,
) -> Result<DecodedCheckedHostOperation<'a>, ErasureError> {
    let (outer, outer_args) = constructor_application_spine(term).ok_or_else(|| {
        expression_lowering_error(
            root,
            "host_coproduct_shape",
            format!("HostIO operation is not a checked coproduct constructor: {term:?}"),
        )
    })?;
    let leaf = if outer.symbol == spine.in_l {
        outer_args.last().copied()
    } else if outer.symbol == spine.in_r {
        let ambient = outer_args.last().copied().ok_or_else(|| {
            expression_lowering_error(
                root,
                "host_coproduct_arity",
                "ambient coproduct arm is empty",
            )
        })?;
        let (inner, inner_args) = constructor_application_spine(ambient).ok_or_else(|| {
            expression_lowering_error(
                root,
                "host_coproduct_shape",
                "ambient operation is not a checked coproduct constructor",
            )
        })?;
        if inner.symbol != spine.in_l && inner.symbol != spine.in_r {
            return Err(expression_lowering_error(
                root,
                "host_coproduct_identity",
                "ambient coproduct constructor identity changed",
            ));
        }
        inner_args.last().copied()
    } else {
        return Err(expression_lowering_error(
            root,
            "host_coproduct_identity",
            "HostIO coproduct constructor identity changed",
        ));
    }
    .ok_or_else(|| {
        expression_lowering_error(root, "host_coproduct_arity", "coproduct arm is empty")
    })?;
    let (constructor, args) = constructor_application_spine(leaf).ok_or_else(|| {
        expression_lowering_error(
            root,
            "host_operation_shape",
            "host operation is not a checked constructor application",
        )
    })?;
    let operation = spine
        .operations
        .get(&constructor.symbol)
        .copied()
        .ok_or_else(|| {
            expression_lowering_error(
                root,
                "unknown_host_operation_identity",
                format!("unrecognized checked host operation {}", constructor.symbol),
            )
        })?;
    let expected_family = match crate::export::host_operation_family_v1(operation) {
        crate::export::HostOpFamilyV1::Clock => &spine.clock_family,
        crate::export::HostOpFamilyV1::Console => &spine.console_family,
        crate::export::HostOpFamilyV1::Fs => &spine.fs_family,
        crate::export::HostOpFamilyV1::Entropy => &spine.entropy_family,
    };
    if &constructor.family_symbol != expected_family {
        return Err(expression_lowering_error(
            root,
            "host_operation_family_identity",
            "host operation constructor belongs to the wrong checked family",
        ));
    }
    let expected = constructor.family_parameter_count + constructor.argument_count;
    if args.len() != expected {
        return Err(expression_lowering_error(
            root,
            "host_operation_arity",
            format!(
                "{} expects {expected} operands, got {}",
                constructor.symbol,
                args.len()
            ),
        ));
    }
    Ok(DecodedCheckedHostOperation {
        operation,
        constructor,
        args,
    })
}

const fn static_host_operation_requires_capability(operation: ken_host::HostOpV1) -> bool {
    !operation.is_ambient()
        && !matches!(
            operation,
            ken_host::HostOpV1::FsHandleMetadata
                | ken_host::HostOpV1::BufferAllocate
                | ken_host::HostOpV1::FsReadAt
                | ken_host::HostOpV1::FsWriteAt
                | ken_host::HostOpV1::BufferFreeze
                | ken_host::HostOpV1::ResourceRelease
        )
}

const fn runtime_selected_host_operation_requires_capability(
    operation: ken_host::HostOpV1,
) -> bool {
    !operation.is_ambient()
        && !matches!(
            operation,
            ken_host::HostOpV1::FsHandleMetadata
                | ken_host::HostOpV1::BufferAllocate
                | ken_host::HostOpV1::FsReadAt
                | ken_host::HostOpV1::FsWriteAt
                | ken_host::HostOpV1::BufferFreeze
                | ken_host::HostOpV1::ResourceRelease
        )
}

fn lower_runtime_selected_host_operation(
    operation: RuntimeExpr,
    continuation_body: RuntimeExpr,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    spine: &CheckedHostSpineV1,
    root: &StableSymbol,
) -> Result<RuntimeExpr, ErasureError> {
    let leaf_dispatch = |family: &StableSymbol,
                         operation_expr: RuntimeExpr,
                         enclosing_binders: u32|
     -> Result<RuntimeExpr, ErasureError> {
        let data = semantic.data_metadata.get(family).ok_or_else(|| {
            expression_lowering_error(
                root,
                "host_operation_family_identity",
                format!("checked host family {family} has no data metadata"),
            )
        })?;
        let mut cases = Vec::new();
        for constructor in &data.constructors {
            let Some(host_operation) = spine.operations.get(&constructor.symbol).copied() else {
                return Err(expression_lowering_error(
                    root,
                    "unknown_host_operation_identity",
                    format!(
                        "checked host family {family} contains unadmitted constructor {}",
                        constructor.symbol
                    ),
                ));
            };
            let argument_count = constructor.argument_count;
            let argument_shift = u32::try_from(argument_count).map_err(|_| {
                expression_lowering_error(
                    root,
                    "host_operation_arity",
                    "host operation arity does not fit runtime IR",
                )
            })?;
            let expected_family =
                match crate::export::host_operation_family_v1(host_operation) {
                    crate::export::HostOpFamilyV1::Clock => &spine.clock_family,
                    crate::export::HostOpFamilyV1::Console => &spine.console_family,
                    crate::export::HostOpFamilyV1::Fs => &spine.fs_family,
                    crate::export::HostOpFamilyV1::Entropy => &spine.entropy_family,
                };
            if family != expected_family {
                return Err(expression_lowering_error(
                    root,
                    "host_operation_family_identity",
                    format!(
                        "operation {} belongs to {family}, expected {expected_family}",
                        constructor.symbol
                    ),
                ));
            }
            let runtime_args = (0..argument_count)
                .map(|index| RuntimeExpr::Var(index as u32))
                .collect::<Vec<_>>();
            let (capability, args) =
                if runtime_selected_host_operation_requires_capability(host_operation) {
                    let mut args = runtime_args.into_iter();
                    let cap = args.next().ok_or_else(|| {
                        expression_lowering_error(
                            root,
                            "host_capability_shape",
                            "FS operation is missing its live capability operand",
                        )
                    })?;
                    (
                        Some(RuntimeCapabilityUse {
                            identity: spine.capability.to_string(),
                            value: Box::new(cap),
                        }),
                        args.collect(),
                    )
                } else if host_operation == ken_host::HostOpV1::ResourceRelease {
                    let resource = runtime_args.get(1).cloned().ok_or_else(|| {
                        expression_lowering_error(
                            root,
                            "resource_release_shape",
                            "ResourceRelease is missing its indexed resource operand",
                        )
                    })?;
                    (None, vec![resource])
                } else {
                    (None, runtime_args)
                };
            cases.push(RuntimeMatchCase {
                constructor: constructor.symbol.to_string(),
                binders: argument_count,
                body: RuntimeExpr::Let {
                    value: Box::new(RuntimeExpr::Effect {
                        family: family.to_string(),
                        operation: host_operation,
                        capability,
                        args,
                    }),
                    body: Box::new(shift_runtime_vars(
                        continuation_body.clone(),
                        enclosing_binders + argument_shift,
                        1,
                    )),
                },
            });
        }
        Ok(RuntimeExpr::Match {
            scrutinee: Box::new(operation_expr),
            cases,
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: format!("runtime-selected operation was not in checked family {family}"),
            },
        })
    };

    let fs = leaf_dispatch(&spine.fs_family, RuntimeExpr::Var(0), 1)?;
    let console = leaf_dispatch(&spine.console_family, RuntimeExpr::Var(0), 2)?;
    let clock = leaf_dispatch(&spine.clock_family, RuntimeExpr::Var(0), 2)?;
    let entropy = leaf_dispatch(&spine.entropy_family, RuntimeExpr::Var(0), 2)?;
    // The ambient algebra is the closed three-way sum Console + Clock +
    // Entropy, represented as Coproduct ConsoleOp (Coproduct ClockOp
    // EntropyOp). The elimination mirrors that nesting exactly, and lives
    // only here so no other consumer reproduces the topology.
    let clock_or_entropy = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![
            RuntimeMatchCase {
                constructor: spine.in_l.to_string(),
                binders: 1,
                body: clock,
            },
            RuntimeMatchCase {
                constructor: spine.in_r.to_string(),
                binders: 1,
                body: entropy,
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "runtime-selected ambient tail operation had malformed coproduct identity"
                .to_string(),
        },
    };
    let ambient = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![
            RuntimeMatchCase {
                constructor: spine.in_l.to_string(),
                binders: 1,
                body: console,
            },
            RuntimeMatchCase {
                constructor: spine.in_r.to_string(),
                binders: 1,
                body: clock_or_entropy,
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "runtime-selected ambient operation had malformed coproduct identity"
                .to_string(),
        },
    };
    Ok(RuntimeExpr::Match {
        scrutinee: Box::new(operation),
        cases: vec![
            RuntimeMatchCase {
                constructor: spine.in_l.to_string(),
                binders: 1,
                body: fs,
            },
            RuntimeMatchCase {
                constructor: spine.in_r.to_string(),
                binders: 1,
                body: ambient,
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "runtime-selected HostIO operation had malformed coproduct identity"
                .to_string(),
        },
    })
}

pub fn emit_proof_erasure_boundary_witness(
    package: &CheckedCorePackage,
    program: &RuntimeProgram,
) -> Result<ProofErasureBoundaryWitness, ErasureError> {
    let expected_targets = program
        .erased_core
        .metadata
        .runtime_declaration_targets
        .clone();
    let record_symbols = package
        .artifact
        .semantic
        .record_sigma_metadata
        .keys()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    if !record_symbols.is_subset(&expected_targets) {
        return Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessMismatch,
            "runtime_declaration_targets",
            format!(
                "pair-only witness emission cannot distinguish non-target records from missing runtime targets: records={record_symbols:?}, runtime_targets={expected_targets:?}"
            ),
        )
        .into());
    }

    emit_proof_erasure_boundary_witness_with_targets(package, expected_targets, program)
}

pub fn emit_proof_erasure_boundary_witness_for_targets<'a>(
    package: &CheckedCorePackage,
    target_closure: impl IntoIterator<Item = &'a StableSymbol>,
    program: &RuntimeProgram,
) -> Result<ProofErasureBoundaryWitness, ErasureError> {
    let expected_targets = target_closure
        .into_iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    emit_proof_erasure_boundary_witness_with_targets(package, expected_targets, program)
}

fn emit_proof_erasure_boundary_witness_with_targets(
    package: &CheckedCorePackage,
    expected_targets: BTreeSet<String>,
    program: &RuntimeProgram,
) -> Result<ProofErasureBoundaryWitness, ErasureError> {
    validate_checked_core_package(package)?;

    let package_identity = RuntimeArtifactIdentity {
        package_identity: package.header.package_identity.to_string(),
        core_semantic_hash: package.core_semantic_hash,
        artifact_hash: package.artifact_hash,
    };
    let program_identity = RuntimeArtifactIdentity::from_program(program);
    if package_identity != program_identity {
        return Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessIdentity,
            "artifact_identity",
            format!(
                "CheckedCorePackage identity {:?} does not match RuntimeProgram identity {:?}",
                package_identity, program_identity
            ),
        )
        .into());
    }

    let package_facts = proof_erasure_boundary_facts_from_package(package, expected_targets);
    let program_facts = proof_erasure_boundary_facts_from_program(program);
    require_erasure_lane_match(
        &package_facts.runtime_declaration_targets,
        &program_facts.runtime_declaration_targets,
        "runtime_declaration_targets",
    )?;
    require_erasure_lane_match(
        &package_facts.record_field_statuses,
        &program_facts.record_field_statuses,
        "record_field_statuses",
    )?;
    require_erasure_lane_match(
        &package_facts.checked_core_record_field_statuses,
        &program_facts.checked_core_record_field_statuses,
        "checked_core_record_field_statuses",
    )?;
    require_erasure_lane_match(
        &package_facts.lowerability,
        &program_facts.lowerability,
        "lowerability",
    )?;
    require_erasure_lane_match(
        &package_facts.unsupported,
        &program_facts.unsupported,
        "unsupported",
    )?;
    require_erasure_lane_match(
        &package_facts.obligations,
        &program_facts.obligations,
        "obligations",
    )?;
    require_erasure_lane_match(
        &package_facts.obligation_metadata,
        &program_facts.obligation_metadata,
        "obligation_metadata",
    )?;
    require_erasure_lane_match(
        &package_facts.assumptions,
        &program_facts.assumptions,
        "assumptions",
    )?;
    require_erasure_lane_match(
        &package_facts.assumption_trust_metadata,
        &program_facts.assumption_trust_metadata,
        "assumption_trust_metadata",
    )?;
    require_erasure_lane_match(
        &package_facts.trusted_base_delta,
        &program_facts.trusted_base_delta,
        "trusted_base_delta",
    )?;

    let witness = ProofErasureBoundaryWitness {
        artifact: program_identity,
        facts: package_facts,
    };
    validate_proof_erasure_boundary_witness(program, &witness)?;
    Ok(witness)
}

fn reject_reachable_unsupported(
    package: &CheckedCorePackage,
    targets: &[StableSymbol],
) -> Result<(), ErasureError> {
    for target in targets {
        if package.artifact.semantic.unsupported.contains_key(target) {
            return Err(ErasureError::UnsupportedErasure {
                symbol: target.clone(),
                reason: "reachable checked-core unsupported entry".to_string(),
            });
        }
        if let Some(status) = package.artifact.semantic.lowerability.get(target) {
            if status.blocks_lowering() {
                return Err(ErasureError::UnsupportedErasure {
                    symbol: target.clone(),
                    reason: format!("lowerability is blocking: {status:?}"),
                });
            }
        }
    }
    Ok(())
}

fn lower_symbol(
    package: &CheckedCorePackage,
    target_closure: &[StableSymbol],
    symbol: &StableSymbol,
) -> Result<RuntimeDeclaration, ErasureError> {
    let semantic = &package.artifact.semantic;
    let kind = if let Some(meta) = semantic.primitive_metadata.get(symbol) {
        lower_primitive(symbol, meta)?
    } else if let Some(meta) = semantic.data_metadata.get(symbol) {
        lower_data(symbol, meta)?
    } else if let Some(meta) = semantic.record_sigma_metadata.get(symbol) {
        lower_record(symbol, meta)?
    } else if let Some(meta) = semantic.recursion_metadata.get(symbol) {
        lower_recursion(symbol, meta)?
    } else if let Some(meta) = semantic.effects_foreign_metadata.get(symbol) {
        lower_effects(symbol, meta)?
    } else if let Some(meta) = semantic.class_instance_metadata.get(symbol) {
        lower_class_instance(symbol, meta)?
    } else if semantic.declarations.contains_key(symbol) {
        lower_transparent_declaration(package, target_closure, symbol)?
    } else {
        return Err(ErasureError::MissingRuntimeMetadata {
            symbol: symbol.clone(),
            section: "runtime-lowerable metadata",
        });
    };

    Ok(RuntimeDeclaration {
        symbol: symbol.to_string(),
        kind,
        metadata: metadata_for_symbol(package, symbol),
    })
}

fn lower_transparent_declaration(
    package: &CheckedCorePackage,
    target_closure: &[StableSymbol],
    symbol: &StableSymbol,
) -> Result<RuntimeDeclarationKind, ErasureError> {
    let semantic = &package.artifact.semantic;
    let reachable_declarations = target_closure
        .iter()
        .filter(|candidate| {
            semantic.declarations.contains_key(*candidate)
                && !has_runtime_metadata(semantic, candidate)
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    let selection = CheckedCoreBodyViewSelection {
        package_identity: package.header.package_identity.clone(),
        package_core_semantic_hash: package.core_semantic_hash,
        package_artifact_hash: package.artifact_hash,
        target_symbol: symbol.clone(),
        reachable_declarations,
        external_symbols: external_declaration_symbols(&package.artifact.semantic),
        dependency_semantic_hashes: package.artifact.semantic.dependency_semantic_hashes.clone(),
    };
    let declarations = checked_host_declaration_closure(package, &selection, symbol)?;
    let declaration = declarations.get(symbol).ok_or_else(|| {
        expression_lowering_error(
            symbol,
            "missing_expression_body_view",
            "body view did not return the selected transparent declaration",
        )
    })?;
    let mut stack = vec![symbol.clone()];
    let body = lower_top_level_body(
        &declaration.body,
        &declarations,
        semantic,
        &mut stack,
        symbol,
    )?;
    Ok(RuntimeDeclarationKind::Transparent { body })
}

fn lower_top_level_body(
    term: &CheckedCoreBodyTerm,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
) -> Result<RuntimeExpr, ErasureError> {
    let mut parameter_count = 0usize;
    let mut body = term;
    while let CheckedCoreBodyTerm::Lambda { body: inner, .. } = body {
        parameter_count += 1;
        body = inner;
    }
    if parameter_count == 0 {
        return lower_body_term(body, declarations, semantic, stack, root_symbol, 0);
    }
    if has_free_variable_at_or_above(body, parameter_count) {
        return Err(expression_lowering_error(
            root_symbol,
            "implicit_closure_capture",
            "top-level lambda body references a de Bruijn binding outside its explicit parameter list",
        ));
    }
    let body = lower_body_term(
        body,
        declarations,
        semantic,
        stack,
        root_symbol,
        parameter_count,
    )?;
    Ok(RuntimeExpr::Closure {
        captures: Vec::new(),
        params: (0..parameter_count)
            .map(|index| format!("arg{index}"))
            .collect(),
        body: Box::new(body),
    })
}

fn has_runtime_metadata(
    semantic: &checked_core::CheckedCoreSemanticInputs,
    symbol: &StableSymbol,
) -> bool {
    semantic.primitive_metadata.contains_key(symbol)
        || semantic.data_metadata.contains_key(symbol)
        || semantic.record_sigma_metadata.contains_key(symbol)
        || semantic.recursion_metadata.contains_key(symbol)
        || semantic.effects_foreign_metadata.contains_key(symbol)
        || semantic.class_instance_metadata.contains_key(symbol)
}

fn checked_host_body_view_symbols(
    semantic: &checked_core::CheckedCoreSemanticInputs,
    _target_closure: &[StableSymbol],
) -> BTreeSet<StableSymbol> {
    semantic
        .declarations
        .keys()
        .filter(|symbol| !has_runtime_metadata(semantic, symbol))
        .cloned()
        .collect()
}

fn external_declaration_symbols(
    semantic: &checked_core::CheckedCoreSemanticInputs,
) -> BTreeSet<StableSymbol> {
    semantic
        .symbols
        .iter()
        .filter(|symbol| {
            !semantic.declarations.contains_key(*symbol)
                && !has_runtime_metadata(semantic, symbol)
                && semantic.lowerability.contains_key(*symbol)
        })
        .cloned()
        .collect()
}

fn lower_body_term(
    term: &CheckedCoreBodyTerm,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
) -> Result<RuntimeExpr, ErasureError> {
    lower_body_term_inner(
        term,
        declarations,
        semantic,
        stack,
        root_symbol,
        context_depth,
        None,
    )
}

#[derive(Clone, Default)]
struct BranchBinderRemap {
    groups: Vec<BranchBinderGroup>,
}

#[derive(Clone)]
struct BranchBinderGroup {
    source_start: usize,
    runtime_start: usize,
    argument_count: usize,
    recursive_count: usize,
    recursive_runtime: bool,
    recursive_slot_templates: Vec<u64>,
}

impl BranchBinderRemap {
    fn enter_binding(&self) -> Self {
        let mut remap = self.clone();
        for group in &mut remap.groups {
            group.source_start += 1;
            group.runtime_start += 1;
        }
        remap
    }

    fn enter_match(
        &self,
        argument_count: usize,
        recursive_count: usize,
        recursive_runtime: bool,
        recursive_slot_templates: Vec<u64>,
    ) -> Self {
        let mut remap = self.clone();
        for group in &mut remap.groups {
            group.source_start += argument_count + recursive_count;
            group.runtime_start += argument_count
                + if recursive_runtime {
                    recursive_count
                } else {
                    0
                };
        }
        remap.groups.push(BranchBinderGroup {
            source_start: 0,
            runtime_start: 0,
            argument_count,
            recursive_count,
            recursive_runtime,
            recursive_slot_templates,
        });
        remap
    }

    fn runtime_index(&self, de_bruijn_index: usize) -> Option<usize> {
        for group in &self.groups {
            let recursive_end = group.source_start + group.recursive_count;
            let group_end = recursive_end + group.argument_count;
            if (group.source_start..recursive_end).contains(&de_bruijn_index) {
                return group
                    .recursive_runtime
                    .then_some(group.runtime_start + de_bruijn_index - group.source_start);
            }
            if (recursive_end..group_end).contains(&de_bruijn_index) {
                let position = de_bruijn_index - recursive_end;
                let recursive_offset = if group.recursive_runtime {
                    group.recursive_count
                } else {
                    0
                };
                return Some(
                    group.runtime_start + recursive_offset + (group.argument_count - 1 - position),
                );
            }
        }
        let erased_below = self
            .groups
            .iter()
            .filter(|group| {
                de_bruijn_index >= group.source_start + group.recursive_count + group.argument_count
            })
            .map(|group| {
                if group.recursive_runtime {
                    0
                } else {
                    group.recursive_count
                }
            })
            .sum::<usize>();
        Some(de_bruijn_index - erased_below)
    }

    fn computational_ih_slot(&self, de_bruijn_index: usize) -> Option<u64> {
        for group in &self.groups {
            let recursive_end = group.source_start + group.recursive_count;
            if (group.source_start..recursive_end).contains(&de_bruijn_index) {
                let source_ordinal = de_bruijn_index - group.source_start;
                // IH binders are innermost-first in de Bruijn order, while
                // slot templates are stored in method telescope order.
                let method_ordinal = group.recursive_count - 1 - source_ordinal;
                return group.recursive_slot_templates.get(method_ordinal).copied();
            }
        }
        None
    }

    fn runtime_depth(&self, source_depth: usize) -> usize {
        (0..source_depth)
            .filter_map(|index| self.runtime_index(index))
            .max()
            .map_or(0, |index| index + 1)
    }
}

fn lower_body_term_inner(
    term: &CheckedCoreBodyTerm,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<&BranchBinderRemap>,
) -> Result<RuntimeExpr, ErasureError> {
    let owner = stack
        .last()
        .expect("expression lowering stack always has an owner")
        .clone();
    if let Some((view, arguments)) = recursive_application_spine(term) {
        let callee = lower_recursive_declaration_call(view, declarations, root_symbol)?;
        if arguments.is_empty() {
            return Ok(callee);
        }
        let args = arguments
            .into_iter()
            .map(|argument| {
                lower_body_term_inner(
                    argument,
                    declarations,
                    semantic,
                    stack,
                    root_symbol,
                    context_depth,
                    branch_remap,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(RuntimeExpr::Call {
            callee: Box::new(callee),
            args,
        });
    }
    if let Some((symbol, level_args, arguments)) = direct_application_spine(term) {
        reject_level_args(root_symbol, level_args)?;
        if let Some(declaration) = declarations.get(symbol) {
            let mut body = &declaration.body;
            let mut parameter_count = 0usize;
            while parameter_count < arguments.len() {
                let CheckedCoreBodyTerm::Lambda { body: inner, .. } = body else {
                    break;
                };
                parameter_count += 1;
                body = inner;
            }
            if parameter_count == arguments.len() {
                if stack.contains(symbol) {
                    return Err(expression_lowering_error(
                        root_symbol,
                        "direct_call_cycle",
                        format!("direct declaration call cycle from {owner} reaches {symbol}"),
                    ));
                }
                let values = arguments
                    .iter()
                    .map(|argument| {
                        lower_body_term_inner(
                            argument,
                            declarations,
                            semantic,
                            stack,
                            root_symbol,
                            context_depth,
                            branch_remap,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut inner_remap = branch_remap.cloned();
                for _ in 0..parameter_count {
                    inner_remap = inner_remap.map(|remap| remap.enter_binding());
                }
                stack.push(symbol.clone());
                let lowered = lower_body_term_inner(
                    body,
                    declarations,
                    semantic,
                    stack,
                    root_symbol,
                    context_depth + parameter_count,
                    inner_remap.as_ref(),
                );
                stack.pop();
                let mut lowered = lowered?;
                for (index, value) in values.into_iter().enumerate().rev() {
                    lowered = RuntimeExpr::Let {
                        value: Box::new(shift_runtime_vars(value, index as u32, 0)),
                        body: Box::new(lowered),
                    };
                }
                return Ok(lowered);
            }
        }
    }
    if let Some((constructor, args)) = constructor_application_spine(term) {
        return lower_constructor_application(
            constructor,
            &args,
            declarations,
            semantic,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        );
    }
    match term {
        CheckedCoreBodyTerm::Variable { de_bruijn_index } => {
            if *de_bruijn_index >= context_depth {
                return Err(expression_lowering_error(
                    root_symbol,
                    "unbound_de_bruijn_variable",
                    format!(
                        "variable index {de_bruijn_index} escapes runtime context depth {context_depth}"
                    ),
                ));
            }
            let runtime_index = branch_remap
                .map(|remap| remap.runtime_index(*de_bruijn_index))
                .unwrap_or(Some(*de_bruijn_index))
                .ok_or_else(|| {
                    expression_lowering_error(
                        root_symbol,
                        "erased_induction_hypothesis_reached_runtime",
                        format!(
                            "variable index {de_bruijn_index} names an erased match induction hypothesis"
                        ),
                    )
                })?;
            let index = u32::try_from(runtime_index).map_err(|_| {
                expression_lowering_error(
                    root_symbol,
                    "variable_index_overflow",
                    format!("variable index {runtime_index} does not fit runtime IR"),
                )
            })?;
            Ok(RuntimeExpr::Var(index))
        }
        CheckedCoreBodyTerm::IntegerLiteral { value } => {
            Ok(RuntimeExpr::Value(RuntimeValue::Int((*value).into())))
        }
        CheckedCoreBodyTerm::DirectDeclarationCall { symbol, level_args } => {
            reject_level_args(root_symbol, level_args)?;
            if stack.contains(symbol) {
                return Err(expression_lowering_error(
                    root_symbol,
                    "direct_call_cycle",
                    format!("direct declaration call cycle from {owner} reaches {symbol}"),
                ));
            }
            let declaration = declarations.get(symbol).ok_or_else(|| {
                expression_lowering_error(
                    root_symbol,
                    "unresolved_direct_declaration_call",
                    format!("body references {symbol} without a selected body view"),
                )
            })?;
            stack.push(symbol.clone());
            let lowered = lower_body_term_inner(
                &declaration.body,
                declarations,
                semantic,
                stack,
                root_symbol,
                context_depth,
                branch_remap,
            );
            stack.pop();
            lowered
        }
        CheckedCoreBodyTerm::RecursiveDeclarationCall(view) => {
            lower_recursive_declaration_call(view, declarations, root_symbol)
        }
        CheckedCoreBodyTerm::ImportedDeclarationCall(view) => {
            lower_imported_declaration_call(view, semantic, root_symbol)
        }
        CheckedCoreBodyTerm::PrimitiveLiteral(view) => lower_primitive_literal(root_symbol, view),
        CheckedCoreBodyTerm::PrimitiveApplication(view) => lower_primitive_application(
            view,
            declarations,
            semantic,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        ),
        CheckedCoreBodyTerm::Lambda { body, .. } => {
            let runtime_depth = branch_remap
                .map(|remap| remap.runtime_depth(context_depth))
                .unwrap_or(context_depth);
            Ok(RuntimeExpr::LexicalClosure {
                captures: (0..runtime_depth)
                    .map(|index| RuntimeExpr::Var(index as u32))
                    .collect(),
                params: vec!["arg0".to_string()],
                body: Box::new(lower_body_term_inner(
                    body,
                    declarations,
                    semantic,
                    stack,
                    root_symbol,
                    context_depth + 1,
                    branch_remap.map(BranchBinderRemap::enter_binding).as_ref(),
                )?),
            })
        }
        CheckedCoreBodyTerm::Application { function, argument } => Ok(RuntimeExpr::Call {
            callee: Box::new(lower_body_term_inner(
                function,
                declarations,
                semantic,
                stack,
                root_symbol,
                context_depth,
                branch_remap,
            )?),
            args: vec![lower_body_term_inner(
                argument,
                declarations,
                semantic,
                stack,
                root_symbol,
                context_depth,
                branch_remap,
            )?],
        }),
        CheckedCoreBodyTerm::Let { value, body, .. } => Ok(RuntimeExpr::Let {
            value: Box::new(lower_body_term_inner(
                value,
                declarations,
                semantic,
                stack,
                root_symbol,
                context_depth,
                branch_remap,
            )?),
            body: Box::new(lower_body_term_inner(
                body,
                declarations,
                semantic,
                stack,
                root_symbol,
                context_depth + 1,
                branch_remap.map(BranchBinderRemap::enter_binding).as_ref(),
            )?),
        }),
        CheckedCoreBodyTerm::ConstructorReference(_) => {
            unreachable!("constructor references are handled by constructor_application_spine")
        }
        CheckedCoreBodyTerm::ErasedConstructorArgument { .. } => Err(expression_lowering_error(
            root_symbol,
            "erased_constructor_argument_outside_constructor",
            "constructor family parameters are erased and cannot appear as runtime expressions",
        )),
        CheckedCoreBodyTerm::Match(view) => lower_match_view(
            view,
            declarations,
            semantic,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        ),
        CheckedCoreBodyTerm::RecordSigmaConstruction(view) => lower_record_sigma_construction(
            view,
            declarations,
            semantic,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        ),
        CheckedCoreBodyTerm::RecordSigmaProjection(view) => lower_record_sigma_projection(
            view,
            declarations,
            semantic,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        ),
        CheckedCoreBodyTerm::DictionaryConstruction(view) => lower_dictionary_construction(
            view,
            declarations,
            semantic,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        ),
    }
}

fn shift_runtime_vars(expr: RuntimeExpr, by: u32, cutoff: u32) -> RuntimeExpr {
    match expr {
        RuntimeExpr::CheckedJoinSite { site_id, body } => RuntimeExpr::CheckedJoinSite {
            site_id,
            body: Box::new(shift_runtime_vars(*body, by, cutoff)),
        },
        RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
            RuntimeExpr::CheckedSubcontinuationFrame {
                frame_id,
                body: Box::new(shift_runtime_vars(*body, by, cutoff)),
            }
        }
        RuntimeExpr::CheckedRecursiveInvocation {
            call_template_id,
            checked_occurrence_path,
            body,
        } => RuntimeExpr::CheckedRecursiveInvocation {
            call_template_id,
            checked_occurrence_path,
            body: Box::new(shift_runtime_vars(*body, by, cutoff)),
        },
        RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids,
            checked_occurrence_paths,
            body,
        } => RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids,
            checked_occurrence_paths,
            body: Box::new(shift_runtime_vars(*body, by, cutoff)),
        },
        RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id,
            checked_occurrence_path,
            body,
        } => RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id,
            checked_occurrence_path,
            body: Box::new(shift_runtime_vars(*body, by, cutoff)),
        },
        RuntimeExpr::Var(index) if index >= cutoff => RuntimeExpr::Var(index + by),
        RuntimeExpr::Var(_)
        | RuntimeExpr::Value(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => expr,
        RuntimeExpr::Let { value, body } => RuntimeExpr::Let {
            value: Box::new(shift_runtime_vars(*value, by, cutoff)),
            body: Box::new(shift_runtime_vars(*body, by, cutoff + 1)),
        },
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => RuntimeExpr::If {
            scrutinee: Box::new(shift_runtime_vars(*scrutinee, by, cutoff)),
            then_expr: Box::new(shift_runtime_vars(*then_expr, by, cutoff)),
            else_expr: Box::new(shift_runtime_vars(*else_expr, by, cutoff)),
        },
        RuntimeExpr::PrimitiveCall { primitive, args } => RuntimeExpr::PrimitiveCall {
            primitive,
            args: args
                .into_iter()
                .map(|arg| shift_runtime_vars(arg, by, cutoff))
                .collect(),
        },
        RuntimeExpr::Construct { constructor, args } => RuntimeExpr::Construct {
            constructor,
            args: args
                .into_iter()
                .map(|arg| shift_runtime_vars(arg, by, cutoff))
                .collect(),
        },
        RuntimeExpr::Match {
            scrutinee,
            cases,
            default,
        } => RuntimeExpr::Match {
            scrutinee: Box::new(shift_runtime_vars(*scrutinee, by, cutoff)),
            cases: cases
                .into_iter()
                .map(|case| RuntimeMatchCase {
                    constructor: case.constructor,
                    binders: case.binders,
                    body: shift_runtime_vars(case.body, by, cutoff + case.binders as u32),
                })
                .collect(),
            default,
        },
        RuntimeExpr::ComputationalMatch {
            scrutinee,
            cases,
            default,
        } => RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(shift_runtime_vars(*scrutinee, by, cutoff)),
            cases: cases
                .into_iter()
                .map(|case| {
                    let binders = case.argument_binders + case.recursive_positions.len();
                    RuntimeComputationalMatchCase {
                        constructor: case.constructor,
                        argument_binders: case.argument_binders,
                        recursive_positions: case.recursive_positions,
                        body: shift_runtime_vars(case.body, by, cutoff + binders as u32),
                    }
                })
                .collect(),
            default,
        },
        RuntimeExpr::Record { fields } => RuntimeExpr::Record {
            fields: fields
                .into_iter()
                .map(|(name, value)| (name, shift_runtime_vars(value, by, cutoff)))
                .collect(),
        },
        RuntimeExpr::Project { record, field } => RuntimeExpr::Project {
            record: Box::new(shift_runtime_vars(*record, by, cutoff)),
            field,
        },
        RuntimeExpr::Closure {
            captures,
            params,
            body,
        } => {
            let inner_cutoff = cutoff + params.len() as u32;
            RuntimeExpr::Closure {
                captures,
                params,
                body: Box::new(shift_runtime_vars(*body, by, inner_cutoff)),
            }
        }
        RuntimeExpr::LexicalClosure {
            captures,
            params,
            body,
        } => {
            let body_cutoff = cutoff + params.len() as u32 + captures.len() as u32;
            RuntimeExpr::LexicalClosure {
                captures: captures
                    .into_iter()
                    .map(|capture| shift_runtime_vars(capture, by, cutoff))
                    .collect(),
                params,
                body: Box::new(shift_runtime_vars(*body, by, body_cutoff)),
            }
        }
        RuntimeExpr::Call { callee, args } => RuntimeExpr::Call {
            callee: Box::new(shift_runtime_vars(*callee, by, cutoff)),
            args: args
                .into_iter()
                .map(|arg| shift_runtime_vars(arg, by, cutoff))
                .collect(),
        },
        RuntimeExpr::Effect {
            family,
            operation,
            capability,
            args,
        } => RuntimeExpr::Effect {
            family,
            operation,
            capability: capability.map(|capability| ken_runtime::RuntimeCapabilityUse {
                identity: capability.identity,
                value: Box::new(shift_runtime_vars(*capability.value, by, cutoff)),
            }),
            args: args
                .into_iter()
                .map(|arg| shift_runtime_vars(arg, by, cutoff))
                .collect(),
        },
    }
}

fn direct_application_spine<'a>(
    term: &'a CheckedCoreBodyTerm,
) -> Option<(
    &'a StableSymbol,
    &'a [CheckedCoreLevelView],
    Vec<&'a CheckedCoreBodyTerm>,
)> {
    let mut arguments = Vec::new();
    let mut current = term;
    while let CheckedCoreBodyTerm::Application { function, argument } = current {
        arguments.push(argument.as_ref());
        current = function.as_ref();
    }
    let CheckedCoreBodyTerm::DirectDeclarationCall { symbol, level_args } = current else {
        return None;
    };
    arguments.reverse();
    Some((symbol, level_args, arguments))
}

fn term_application_head(mut term: &CheckedCoreBodyTerm) -> &CheckedCoreBodyTerm {
    while let CheckedCoreBodyTerm::Application { function, .. } = term {
        term = function;
    }
    term
}

fn recursive_application_spine(
    term: &CheckedCoreBodyTerm,
) -> Option<(
    &checked_core::CheckedCoreRecursiveCallView,
    Vec<&CheckedCoreBodyTerm>,
)> {
    let mut arguments = Vec::new();
    let mut current = term;
    while let CheckedCoreBodyTerm::Application { function, argument } = current {
        arguments.push(argument.as_ref());
        current = function.as_ref();
    }
    let CheckedCoreBodyTerm::RecursiveDeclarationCall(view) = current else {
        return None;
    };
    arguments.reverse();
    Some((view, arguments))
}

fn computational_ih_application_spine<'a>(
    term: &'a CheckedCoreBodyTerm,
    branch_remap: Option<&BranchBinderRemap>,
) -> Option<(u64, Vec<&'a CheckedCoreBodyTerm>)> {
    let mut arguments = Vec::new();
    let mut current = term;
    while let CheckedCoreBodyTerm::Application { function, argument } = current {
        arguments.push(argument.as_ref());
        current = function.as_ref();
    }
    let CheckedCoreBodyTerm::Variable { de_bruijn_index } = current else {
        return None;
    };
    let slot_template_id = branch_remap?.computational_ih_slot(*de_bruijn_index)?;
    arguments.reverse();
    Some((slot_template_id, arguments))
}

fn lower_recursive_declaration_call(
    view: &checked_core::CheckedCoreRecursiveCallView,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    root_symbol: &StableSymbol,
) -> Result<RuntimeExpr, ErasureError> {
    reject_level_args(root_symbol, &view.level_args)?;
    require_expression_supported(
        root_symbol,
        &view.symbol,
        &view.lowerability,
        "recursive_lowerability_blocked",
    )?;
    if !matches!(
        view.admission,
        checked_core::RecursionAdmission::AcceptedStructural
            | checked_core::RecursionAdmission::AcceptedSizeChange
    ) {
        return Err(expression_lowering_error(
            root_symbol,
            "unsupported_recursive_shape",
            format!(
                "recursive call to {} has non-executable admission {:?}",
                view.symbol, view.admission
            ),
        ));
    }
    if !view.group_members.contains(&view.symbol) {
        return Err(expression_lowering_error(
            root_symbol,
            "stale_recursive_group_member",
            format!(
                "recursive call to {} is absent from group {}",
                view.symbol, view.group_symbol
            ),
        ));
    }
    if !declarations.contains_key(&view.symbol) {
        return Err(expression_lowering_error(
            root_symbol,
            "unresolved_recursive_declaration_call",
            format!(
                "recursive call to {} has no selected body view in group {}",
                view.symbol, view.group_symbol
            ),
        ));
    }
    Ok(RuntimeExpr::DeclarationRef {
        symbol: view.symbol.to_string(),
    })
}

fn lower_imported_declaration_call(
    view: &checked_core::CheckedCoreImportedDeclarationCallView,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    root_symbol: &StableSymbol,
) -> Result<RuntimeExpr, ErasureError> {
    reject_level_args(root_symbol, &view.level_args)?;
    let lowerability = semantic.lowerability.get(&view.symbol).ok_or_else(|| {
        expression_lowering_error(
            root_symbol,
            "imported_declaration_missing_lowerability",
            format!(
                "imported declaration {} has no lowerability metadata",
                view.symbol
            ),
        )
    })?;
    require_expression_supported(
        root_symbol,
        &view.symbol,
        lowerability,
        "imported_declaration_lowerability_blocked",
    )?;
    if view.dependency_semantic_hash.is_empty() {
        return Err(expression_lowering_error(
            root_symbol,
            "missing_dependency_identity",
            format!(
                "imported declaration {} through {} has an empty semantic hash",
                view.symbol, view.dependency
            ),
        ));
    }
    Ok(RuntimeExpr::ImportedDeclarationRef {
        symbol: view.symbol.to_string(),
        dependency: view.dependency.to_string(),
        dependency_semantic_hash: view.dependency_semantic_hash.clone(),
    })
}

fn lower_dictionary_construction(
    view: &checked_core::CheckedCoreDictionaryConstructionView,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<&BranchBinderRemap>,
) -> Result<RuntimeExpr, ErasureError> {
    require_expression_supported(
        root_symbol,
        &view.dictionary.symbol,
        &view.dictionary.lowerability,
        "dictionary_lowerability_blocked",
    )?;
    validate_dictionary_field_view(root_symbol, &view.dictionary)?;
    if view.fields.len() != view.dictionary.fields.len() {
        return Err(expression_lowering_error(
            root_symbol,
            "stale_dictionary_field_selection",
            format!(
                "dictionary construction for {} carries {} fields, expected {}",
                view.dictionary.symbol,
                view.fields.len(),
                view.dictionary.fields.len()
            ),
        ));
    }

    let mut runtime_fields = Vec::new();
    for (expected, value) in view.dictionary.fields.iter().zip(&view.fields) {
        match value {
            checked_core::CheckedCoreDictionaryFieldValue::Runtime { field, value } => {
                require_same_dictionary_field(root_symbol, expected, field)?;
                if !matches!(
                    field.runtime,
                    checked_core::DictionaryFieldRuntimeStatus::Runtime
                ) {
                    return Err(expression_lowering_error(
                        root_symbol,
                        "non_executable_dictionary_field_use",
                        format!("dictionary field {} is not executable", field.name),
                    ));
                }
                runtime_fields.push((
                    field.name.clone(),
                    lower_body_term_inner(
                        value,
                        declarations,
                        semantic,
                        stack,
                        root_symbol,
                        context_depth,
                        branch_remap,
                    )?,
                ));
            }
            checked_core::CheckedCoreDictionaryFieldValue::Erased { field, .. } => {
                require_same_dictionary_field(root_symbol, expected, field)?;
                if matches!(
                    field.runtime,
                    checked_core::DictionaryFieldRuntimeStatus::Runtime
                ) {
                    return Err(expression_lowering_error(
                        root_symbol,
                        "runtime_dictionary_field_erased_value",
                        format!(
                            "runtime dictionary field {} cannot be supplied by erased bytes",
                            field.name
                        ),
                    ));
                }
            }
        }
    }

    Ok(RuntimeExpr::Record {
        fields: runtime_fields,
    })
}

fn validate_dictionary_field_view(
    root_symbol: &StableSymbol,
    dictionary: &checked_core::CheckedCoreDictionaryView,
) -> Result<(), ErasureError> {
    for (expected_position, field) in dictionary.fields.iter().enumerate() {
        if field.position != expected_position {
            return Err(expression_lowering_error(
                root_symbol,
                "stale_dictionary_field_selection",
                format!(
                    "dictionary metadata for {} has field {} at position {}, expected {}",
                    dictionary.symbol, field.name, field.position, expected_position
                ),
            ));
        }
    }
    Ok(())
}

fn require_same_dictionary_field(
    root_symbol: &StableSymbol,
    expected: &checked_core::CheckedCoreDictionaryFieldView,
    actual: &checked_core::CheckedCoreDictionaryFieldView,
) -> Result<(), ErasureError> {
    if expected == actual {
        Ok(())
    } else {
        Err(expression_lowering_error(
            root_symbol,
            "stale_dictionary_field_selection",
            format!("dictionary field view changed: expected {expected:?}, got {actual:?}"),
        ))
    }
}

fn lower_record_sigma_construction(
    view: &checked_core::CheckedCoreRecordSigmaConstructionView,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<&BranchBinderRemap>,
) -> Result<RuntimeExpr, ErasureError> {
    require_expression_supported(
        root_symbol,
        &view.record.symbol,
        &view.record.lowerability,
        "record_lowerability_blocked",
    )?;
    validate_record_field_view(root_symbol, &view.record)?;
    if view.fields.len() != view.record.fields.len() {
        return Err(expression_lowering_error(
            root_symbol,
            "stale_field_identity_order",
            format!(
                "record/Sigma construction for {} carries {} fields, expected {}",
                view.record.symbol,
                view.fields.len(),
                view.record.fields.len()
            ),
        ));
    }

    let mut runtime_fields = Vec::new();
    for (expected, value) in view.record.fields.iter().zip(&view.fields) {
        match value {
            checked_core::CheckedCoreRecordSigmaFieldValue::Runtime { field, value } => {
                require_same_record_field(root_symbol, expected, field)?;
                if !matches!(field.runtime, checked_core::RuntimeFieldStatus::Runtime) {
                    return Err(expression_lowering_error(
                        root_symbol,
                        "non_runtime_record_field_value",
                        format!("field {} is not executable at runtime", field.name),
                    ));
                }
                runtime_fields.push((
                    field.name.clone(),
                    lower_body_term_inner(
                        value,
                        declarations,
                        semantic,
                        stack,
                        root_symbol,
                        context_depth,
                        branch_remap,
                    )?,
                ));
            }
            checked_core::CheckedCoreRecordSigmaFieldValue::Erased { field, .. } => {
                require_same_record_field(root_symbol, expected, field)?;
                if matches!(field.runtime, checked_core::RuntimeFieldStatus::Runtime) {
                    return Err(expression_lowering_error(
                        root_symbol,
                        "runtime_field_erased_value",
                        format!(
                            "runtime field {} cannot be supplied by erased bytes",
                            field.name
                        ),
                    ));
                }
            }
        }
    }

    Ok(RuntimeExpr::Record {
        fields: runtime_fields,
    })
}

fn lower_record_sigma_projection(
    view: &checked_core::CheckedCoreRecordSigmaProjectionView,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<&BranchBinderRemap>,
) -> Result<RuntimeExpr, ErasureError> {
    require_expression_supported(
        root_symbol,
        &view.record.symbol,
        &view.record.lowerability,
        "record_lowerability_blocked",
    )?;
    validate_record_field_view(root_symbol, &view.record)?;
    let expected = view.record.fields.get(view.field.position).ok_or_else(|| {
        expression_lowering_error(
            root_symbol,
            "stale_field_identity_order",
            format!(
                "record/Sigma projection for {} references missing field position {}",
                view.record.symbol, view.field.position
            ),
        )
    })?;
    require_same_record_field(root_symbol, expected, &view.field)?;
    if !matches!(
        view.field.runtime,
        checked_core::RuntimeFieldStatus::Runtime
    ) {
        return Err(expression_lowering_error(
            root_symbol,
            "non_executable_erased_field_projection",
            format!(
                "field {} of {} is erased and cannot become a runtime value",
                view.field.name, view.record.symbol
            ),
        ));
    }
    for skipped in &view.skipped_fields {
        let Some(expected) = view.record.fields.get(skipped.position) else {
            return Err(expression_lowering_error(
                root_symbol,
                "stale_field_identity_order",
                format!(
                    "record/Sigma projection for {} skips missing field position {}",
                    view.record.symbol, skipped.position
                ),
            ));
        };
        require_same_record_field(root_symbol, expected, skipped)?;
    }

    Ok(RuntimeExpr::Project {
        record: Box::new(lower_body_term_inner(
            &view.base,
            declarations,
            semantic,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        )?),
        field: view.field.name.clone(),
    })
}

fn validate_record_field_view(
    root_symbol: &StableSymbol,
    record: &checked_core::CheckedCoreRecordSigmaView,
) -> Result<(), ErasureError> {
    for (expected_position, field) in record.fields.iter().enumerate() {
        if field.position != expected_position {
            return Err(expression_lowering_error(
                root_symbol,
                "stale_field_identity_order",
                format!(
                    "record/Sigma metadata for {} has field {} at position {}, expected {}",
                    record.symbol, field.name, field.position, expected_position
                ),
            ));
        }
    }
    Ok(())
}

fn require_same_record_field(
    root_symbol: &StableSymbol,
    expected: &checked_core::CheckedCoreRecordSigmaFieldView,
    actual: &checked_core::CheckedCoreRecordSigmaFieldView,
) -> Result<(), ErasureError> {
    if expected == actual {
        Ok(())
    } else {
        Err(expression_lowering_error(
            root_symbol,
            "stale_field_identity_order",
            format!("record/Sigma field view changed: expected {expected:?}, got {actual:?}"),
        ))
    }
}

fn lower_primitive_literal(
    root_symbol: &StableSymbol,
    view: &checked_core::CheckedCorePrimitiveView,
) -> Result<RuntimeExpr, ErasureError> {
    require_expression_supported(
        root_symbol,
        &view.symbol,
        &view.lowerability,
        "primitive_lowerability_blocked",
    )?;
    if !matches!(
        view.reduction,
        checked_core::PrimitiveReductionMetadata::Literal
    ) {
        return Err(expression_lowering_error(
            root_symbol,
            "stale_primitive_metadata",
            format!(
                "primitive literal view for {} has non-literal reduction {:?}",
                view.symbol, view.reduction
            ),
        ));
    }
    if !matches!(view.partiality, PartialityMetadata::Total) {
        return Err(expression_lowering_error(
            root_symbol,
            "primitive_literal_partiality_unsupported",
            format!(
                "primitive literal {} carries partiality metadata",
                view.symbol
            ),
        ));
    }

    primitive_literal_value(&view.registry_symbol)
        .map(RuntimeExpr::Value)
        .ok_or_else(|| {
            expression_lowering_error(
                root_symbol,
                "unsupported_primitive_literal",
                format!(
                    "primitive literal {} has unsupported registry symbol {}",
                    view.symbol, view.registry_symbol
                ),
            )
        })
}

fn lower_primitive_application(
    view: &checked_core::CheckedCorePrimitiveApplicationView,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<&BranchBinderRemap>,
) -> Result<RuntimeExpr, ErasureError> {
    require_expression_supported(
        root_symbol,
        &view.primitive.symbol,
        &view.primitive.lowerability,
        "primitive_lowerability_blocked",
    )?;
    if !matches!(
        view.primitive.reduction,
        checked_core::PrimitiveReductionMetadata::Op
    ) {
        return Err(expression_lowering_error(
            root_symbol,
            "stale_primitive_metadata",
            format!(
                "primitive application view for {} has non-op reduction {:?}",
                view.primitive.symbol, view.primitive.reduction
            ),
        ));
    }

    let mut args = Vec::with_capacity(view.arguments.len());
    for argument in &view.arguments {
        args.push(lower_body_term_inner(
            argument,
            declarations,
            semantic,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        )?);
    }

    Ok(RuntimeExpr::PrimitiveCall {
        primitive: runtime_primitive_from_view(root_symbol, &view.primitive),
        args,
    })
}

fn primitive_literal_value(registry_symbol: &str) -> Option<RuntimeValue> {
    if let Some(raw) = registry_symbol.strip_prefix("lit_int_") {
        return raw
            .parse::<i64>()
            .ok()
            .map(|value| RuntimeValue::Int(value.into()));
    }
    match registry_symbol {
        "lit_bool_true" => return Some(RuntimeValue::Bool(true)),
        "lit_bool_false" => return Some(RuntimeValue::Bool(false)),
        _ => {}
    }
    if let Some(raw) = registry_symbol.strip_prefix("lit_string_") {
        return Some(RuntimeValue::String(raw.to_string()));
    }
    if let Some(raw) = registry_symbol.strip_prefix("lit_bytes_hex_") {
        return decode_hex_bytes(raw).map(RuntimeValue::Bytes);
    }
    None
}

fn decode_hex_bytes(raw: &str) -> Option<Vec<u8>> {
    let bytes = raw.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        out.push((high << 4) | low);
    }
    Some(out)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn runtime_primitive_from_view(
    root_symbol: &StableSymbol,
    view: &checked_core::CheckedCorePrimitiveView,
) -> RuntimePrimitive {
    let constructor = |family: &str, name: &str| {
        let package = root_symbol
            .components
            .first()
            .cloned()
            .unwrap_or_else(|| "package".to_string());
        let family = StableSymbol::new(
            checked_core::SymbolNamespace::Declaration,
            vec![package, family.to_string()],
        );
        StableSymbol::constructor(&family, name).to_string()
    };
    let partiality = match view.registry_symbol.as_str() {
        "bytes_at" => RuntimePartiality::SafeOption {
            none: constructor("Option", "None"),
            some: constructor("Option", "Some"),
            obligation: Some(StableSymbol::obligation("bytes_at.bounds").to_string()),
        },
        "bytes_slice" => RuntimePartiality::SafeOption {
            none: constructor("Option", "None"),
            some: constructor("Option", "Some"),
            obligation: None,
        },
        "bytes_decode" => RuntimePartiality::SafeResult {
            err: constructor("Result", "Err"),
            ok: constructor("Result", "Ok"),
            error: constructor("Utf8Error", "InvalidUtf8"),
        },
        _ => runtime_partiality_from_checked(&view.partiality),
    };
    RuntimePrimitive {
        symbol: view.registry_symbol.clone(),
        partiality,
    }
}

fn runtime_partiality_from_checked(partiality: &PartialityMetadata) -> RuntimePartiality {
    match partiality {
        PartialityMetadata::Total => RuntimePartiality::Total,
        PartialityMetadata::CheckedPartial { obligation } => RuntimePartiality::CheckedTrap {
            obligation: obligation.to_string(),
        },
        PartialityMetadata::TrustedPartial { assumption } => RuntimePartiality::TrustedTrap {
            assumption: assumption.to_string(),
        },
    }
}

fn constructor_application_spine<'a>(
    term: &'a CheckedCoreBodyTerm,
) -> Option<(
    &'a checked_core::CheckedCoreConstructorView,
    Vec<&'a CheckedCoreBodyTerm>,
)> {
    let mut args = Vec::new();
    let mut current = term;
    while let CheckedCoreBodyTerm::Application { function, argument } = current {
        args.push(argument.as_ref());
        current = function.as_ref();
    }
    let CheckedCoreBodyTerm::ConstructorReference(constructor) = current else {
        return None;
    };
    args.reverse();
    Some((constructor, args))
}

fn lower_constructor_application(
    constructor: &checked_core::CheckedCoreConstructorView,
    args: &[&CheckedCoreBodyTerm],
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<&BranchBinderRemap>,
) -> Result<RuntimeExpr, ErasureError> {
    reject_level_args(root_symbol, &constructor.level_args)?;
    require_expression_supported(
        root_symbol,
        &constructor.family_symbol,
        &constructor.family_lowerability,
        "data_lowerability_blocked",
    )?;
    require_expression_supported(
        root_symbol,
        &constructor.symbol,
        &constructor.constructor_lowerability,
        "constructor_lowerability_blocked",
    )?;
    if constructor.family_index_count != 0 || constructor.target_index_count != 0 {
        return Err(expression_lowering_error(
            root_symbol,
            "dependent_constructor_lowering_unsupported",
            format!(
                "constructor {} belongs to indexed family {}",
                constructor.symbol, constructor.family_symbol
            ),
        ));
    }
    let expected = constructor.family_parameter_count + constructor.argument_count;
    if args.len() != expected {
        return Err(expression_lowering_error(
            root_symbol,
            "constructor_arity_mismatch",
            format!(
                "constructor {} expects {} family parameters plus {} runtime arguments, got {}",
                constructor.symbol,
                constructor.family_parameter_count,
                constructor.argument_count,
                args.len()
            ),
        ));
    }
    let runtime_args = args[constructor.family_parameter_count..]
        .iter()
        .map(|arg| {
            lower_body_term_inner(
                arg,
                declarations,
                semantic,
                stack,
                root_symbol,
                context_depth,
                branch_remap,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RuntimeExpr::Construct {
        constructor: constructor.symbol.to_string(),
        args: runtime_args,
    })
}

fn lower_match_view(
    view: &checked_core::CheckedCoreMatchView,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    semantic: &checked_core::CheckedCoreSemanticInputs,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<&BranchBinderRemap>,
) -> Result<RuntimeExpr, ErasureError> {
    reject_level_args(root_symbol, &view.level_args)?;
    if !view.indices.is_empty() {
        return Err(expression_lowering_error(
            root_symbol,
            "unsupported_dependent_motive",
            format!("match over {} carries runtime indices", view.family_symbol),
        ));
    }
    let scrutinee = Box::new(lower_body_term_inner(
        &view.scrutinee,
        declarations,
        semantic,
        stack,
        root_symbol,
        context_depth,
        branch_remap,
    )?);
    let computational = match_uses_computational_recursive_hypothesis(view, root_symbol)?;
    let mut cases = Vec::with_capacity(view.branches.len());
    for branch in &view.branches {
        let constructor = &branch.constructor;
        reject_level_args(root_symbol, &constructor.level_args)?;
        require_expression_supported(
            root_symbol,
            &constructor.family_symbol,
            &constructor.family_lowerability,
            "data_lowerability_blocked",
        )?;
        require_expression_supported(
            root_symbol,
            &constructor.symbol,
            &constructor.constructor_lowerability,
            "constructor_lowerability_blocked",
        )?;
        if constructor.family_index_count != 0 || constructor.target_index_count != 0 {
            return Err(expression_lowering_error(
                root_symbol,
                "dependent_constructor_lowering_unsupported",
                format!(
                    "match branch constructor {} belongs to indexed family {}",
                    constructor.symbol, constructor.family_symbol
                ),
            ));
        }
        let erased_count = constructor.recursive_positions.len();
        let source_binder_count = constructor.argument_count + erased_count;
        let body = peel_match_branch_method(
            &branch.method,
            source_binder_count,
            root_symbol,
            &constructor.symbol,
        )?;
        let remap = branch_remap.cloned().unwrap_or_default().enter_match(
            constructor.argument_count,
            erased_count,
            computational,
            Vec::new(),
        );
        cases.push((
            constructor,
            lower_body_term_inner(
                body,
                declarations,
                semantic,
                stack,
                root_symbol,
                context_depth + source_binder_count,
                Some(&remap),
            )?,
        ));
    }
    let default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: format!("no runtime match case selected for {}", view.family_symbol),
    };
    if computational {
        Ok(RuntimeExpr::ComputationalMatch {
            scrutinee,
            cases: cases
                .into_iter()
                .map(|(constructor, body)| RuntimeComputationalMatchCase {
                    constructor: constructor.symbol.to_string(),
                    argument_binders: constructor.argument_count,
                    recursive_positions: constructor.recursive_positions.clone(),
                    body,
                })
                .collect(),
            default,
        })
    } else {
        Ok(RuntimeExpr::Match {
            scrutinee,
            cases: cases
                .into_iter()
                .map(|(constructor, body)| RuntimeMatchCase {
                    constructor: constructor.symbol.to_string(),
                    binders: constructor.argument_count,
                    body,
                })
                .collect(),
            default,
        })
    }
}

fn peel_match_branch_method<'a>(
    mut method: &'a CheckedCoreBodyTerm,
    binders: usize,
    root_symbol: &StableSymbol,
    constructor: &StableSymbol,
) -> Result<&'a CheckedCoreBodyTerm, ErasureError> {
    for position in 0..binders {
        let CheckedCoreBodyTerm::Lambda { body, .. } = method else {
            return Err(expression_lowering_error(
                root_symbol,
                "match_branch_arity_mismatch",
                format!(
                    "branch for constructor {constructor} is missing binder {position} of {binders}"
                ),
            ));
        };
        method = body.as_ref();
    }
    Ok(method)
}

fn match_uses_computational_recursive_hypothesis(
    view: &checked_core::CheckedCoreMatchView,
    root_symbol: &StableSymbol,
) -> Result<bool, ErasureError> {
    checked_core::checked_match_uses_computational_recursive_hypothesis(view).map_err(|error| {
        expression_lowering_error(
            root_symbol,
            "match_branch_arity_mismatch",
            format!(
                "branch for constructor {} is missing binder {} of {}",
                error.constructor, error.position, error.binders
            ),
        )
    })
}

fn require_expression_supported(
    root_symbol: &StableSymbol,
    symbol: &StableSymbol,
    status: &LowerabilityStatus,
    lane: &'static str,
) -> Result<(), ErasureError> {
    if status.blocks_lowering() {
        Err(expression_lowering_error(
            root_symbol,
            lane,
            format!("{symbol} lowerability is blocking: {status:?}"),
        ))
    } else {
        Ok(())
    }
}

fn reject_level_args(
    owner: &StableSymbol,
    level_args: &[CheckedCoreLevelView],
) -> Result<(), ErasureError> {
    if level_args.is_empty() {
        Ok(())
    } else {
        Err(expression_lowering_error(
            owner,
            "level_arguments_unsupported",
            "runtime expression lowering does not instantiate level-polymorphic direct calls",
        ))
    }
}

fn has_free_variable_at_or_above(term: &CheckedCoreBodyTerm, bound: usize) -> bool {
    match term {
        CheckedCoreBodyTerm::Variable { de_bruijn_index } => *de_bruijn_index >= bound,
        CheckedCoreBodyTerm::IntegerLiteral { .. } => false,
        CheckedCoreBodyTerm::DirectDeclarationCall { .. } => false,
        CheckedCoreBodyTerm::RecursiveDeclarationCall(_) => false,
        CheckedCoreBodyTerm::ImportedDeclarationCall(_) => false,
        CheckedCoreBodyTerm::PrimitiveLiteral(_) => false,
        CheckedCoreBodyTerm::PrimitiveApplication(view) => view
            .arguments
            .iter()
            .any(|argument| has_free_variable_at_or_above(argument, bound)),
        CheckedCoreBodyTerm::ConstructorReference(_) => false,
        CheckedCoreBodyTerm::ErasedConstructorArgument { .. } => false,
        CheckedCoreBodyTerm::Lambda { body, .. } => has_free_variable_at_or_above(body, bound + 1),
        CheckedCoreBodyTerm::Application { function, argument } => {
            has_free_variable_at_or_above(function, bound)
                || has_free_variable_at_or_above(argument, bound)
        }
        CheckedCoreBodyTerm::Let { value, body, .. } => {
            has_free_variable_at_or_above(value, bound)
                || has_free_variable_at_or_above(body, bound + 1)
        }
        CheckedCoreBodyTerm::Match(view) => {
            has_free_variable_at_or_above(&view.scrutinee, bound)
                || view
                    .branches
                    .iter()
                    .any(|branch| has_free_variable_at_or_above(&branch.method, bound))
        }
        CheckedCoreBodyTerm::RecordSigmaConstruction(view) => {
            view.fields.iter().any(|field| match field {
                checked_core::CheckedCoreRecordSigmaFieldValue::Runtime { value, .. } => {
                    has_free_variable_at_or_above(value, bound)
                }
                checked_core::CheckedCoreRecordSigmaFieldValue::Erased { .. } => false,
            })
        }
        CheckedCoreBodyTerm::RecordSigmaProjection(view) => {
            has_free_variable_at_or_above(&view.base, bound)
        }
        CheckedCoreBodyTerm::DictionaryConstruction(view) => {
            view.fields.iter().any(|field| match field {
                checked_core::CheckedCoreDictionaryFieldValue::Runtime { value, .. } => {
                    has_free_variable_at_or_above(value, bound)
                }
                checked_core::CheckedCoreDictionaryFieldValue::Erased { .. } => false,
            })
        }
    }
}

fn expression_view_error(symbol: &StableSymbol, err: CheckedCoreBodyViewError) -> ErasureError {
    expression_lowering_error(symbol, err.lane(), err.to_string())
}

fn expression_lowering_error(
    symbol: &StableSymbol,
    lane: &'static str,
    reason: impl Into<String>,
) -> ErasureError {
    ErasureError::ExpressionLowering {
        symbol: symbol.clone(),
        lane,
        reason: reason.into(),
    }
}

fn lower_primitive(
    symbol: &StableSymbol,
    meta: &PrimitiveMetadata,
) -> Result<RuntimeDeclarationKind, ErasureError> {
    require_supported(symbol, &meta.lowerability)?;
    Ok(RuntimeDeclarationKind::Primitive {
        op: RuntimePrimitive {
            symbol: meta.registry_symbol.clone(),
            partiality: runtime_partiality_from_checked(&meta.partiality),
        },
    })
}

fn lower_data(
    symbol: &StableSymbol,
    meta: &DataMetadata,
) -> Result<RuntimeDeclarationKind, ErasureError> {
    require_supported(symbol, &meta.lowerability)?;
    require_supported(symbol, &meta.eliminator)?;
    for ctor in &meta.constructors {
        require_supported(&ctor.symbol, &ctor.lowerability)?;
    }
    Ok(RuntimeDeclarationKind::Data {
        constructors: meta
            .constructors
            .iter()
            .map(|ctor| RuntimeConstructor {
                symbol: ctor.symbol.to_string(),
                runtime_arg_count: ctor.argument_count,
            })
            .collect(),
    })
}

fn lower_record(
    symbol: &StableSymbol,
    meta: &RecordSigmaMetadata,
) -> Result<RuntimeDeclarationKind, ErasureError> {
    require_supported(symbol, &meta.lowerability)?;
    Ok(RuntimeDeclarationKind::Record {
        fields: meta
            .fields
            .iter()
            .map(|field| RuntimeField {
                name: field.name.clone(),
                status: match field.runtime {
                    crate::checked_core::RuntimeFieldStatus::Runtime => RuntimeFieldStatus::Runtime,
                    crate::checked_core::RuntimeFieldStatus::ErasedLaw => {
                        RuntimeFieldStatus::ErasedLaw
                    }
                    crate::checked_core::RuntimeFieldStatus::ErasedProof => {
                        RuntimeFieldStatus::ErasedProof
                    }
                },
            })
            .collect(),
    })
}

fn lower_recursion(
    symbol: &StableSymbol,
    meta: &RecursionMetadata,
) -> Result<RuntimeDeclarationKind, ErasureError> {
    require_supported(symbol, &meta.lowerability)?;
    Ok(RuntimeDeclarationKind::RecursiveGroup {
        members: meta.group_members.iter().map(ToString::to_string).collect(),
    })
}

fn lower_effects(
    symbol: &StableSymbol,
    meta: &EffectsForeignMetadata,
) -> Result<RuntimeDeclarationKind, ErasureError> {
    require_supported(symbol, &meta.lowerability)?;
    Ok(RuntimeDeclarationKind::EffectBoundary {
        effects: meta.declared_effects.clone(),
    })
}

fn lower_class_instance(
    symbol: &StableSymbol,
    meta: &ClassInstanceMetadata,
) -> Result<RuntimeDeclarationKind, ErasureError> {
    require_supported(symbol, &meta.lowerability)?;
    match meta.kind {
        ClassInstanceKind::Class | ClassInstanceKind::Instance | ClassInstanceKind::Dictionary => {
            Ok(RuntimeDeclarationKind::MetadataOnly)
        }
    }
}

fn require_supported(
    symbol: &StableSymbol,
    status: &LowerabilityStatus,
) -> Result<(), ErasureError> {
    if status.blocks_lowering() {
        return Err(ErasureError::UnsupportedErasure {
            symbol: symbol.clone(),
            reason: format!("metadata lowerability is blocking: {status:?}"),
        });
    }
    Ok(())
}

fn metadata_for_symbol(
    package: &CheckedCorePackage,
    symbol: &StableSymbol,
) -> RuntimeSymbolMetadata {
    let semantic = &package.artifact.semantic;
    RuntimeSymbolMetadata {
        obligations: semantic
            .obligation_metadata
            .iter()
            .filter_map(|(obligation, meta)| {
                (meta.origin == *symbol).then(|| obligation.to_string())
            })
            .collect(),
        obligation_metadata: semantic
            .obligation_metadata
            .iter()
            .filter_map(|(obligation, meta)| {
                (meta.origin == *symbol)
                    .then(|| (obligation.to_string(), runtime_obligation_metadata(meta)))
            })
            .collect(),
        assumptions: semantic
            .assumption_trust_metadata
            .iter()
            .filter_map(|(assumption, meta)| {
                (meta.target == *symbol).then(|| assumption.to_string())
            })
            .collect(),
        assumption_trust_metadata: semantic
            .assumption_trust_metadata
            .iter()
            .filter_map(|(assumption, meta)| {
                (meta.target == *symbol).then(|| {
                    (
                        assumption.to_string(),
                        runtime_assumption_trust_metadata(meta),
                    )
                })
            })
            .collect(),
        trusted_base_delta: semantic
            .trusted_base_delta
            .keys()
            .filter(|trust| *trust == symbol)
            .map(ToString::to_string)
            .collect(),
        lowerability: semantic
            .lowerability
            .get(symbol)
            .map(runtime_lowerability_status),
        unsupported: semantic.unsupported.get(symbol).cloned(),
        runtime_checks: runtime_checks_for_targets(package, &[symbol.clone()]),
        capabilities: capabilities_for_targets(package, &[symbol.clone()]),
        effects: effects_for_targets(package, &[symbol.clone()]),
    }
}

fn proof_erasure_boundary_facts_from_package(
    package: &CheckedCorePackage,
    expected_targets: BTreeSet<String>,
) -> ProofErasureBoundaryFacts {
    let semantic = &package.artifact.semantic;
    ProofErasureBoundaryFacts {
        record_field_statuses: package_declaration_record_field_statuses(
            package,
            &expected_targets,
        ),
        runtime_declaration_targets: expected_targets,
        checked_core_record_field_statuses: package_record_field_statuses(package),
        lowerability: lowerability_map(&semantic.lowerability),
        unsupported: symbol_bytes_map(&semantic.unsupported),
        obligations: symbol_bytes_map(&semantic.obligations),
        obligation_metadata: obligation_metadata_map(&semantic.obligation_metadata),
        assumptions: symbol_bytes_map(&semantic.assumptions),
        assumption_trust_metadata: assumption_trust_metadata_map(
            &semantic.assumption_trust_metadata,
        ),
        trusted_base_delta: symbol_bytes_map(&semantic.trusted_base_delta),
    }
}

fn package_declaration_record_field_statuses(
    package: &CheckedCorePackage,
    expected_targets: &BTreeSet<String>,
) -> BTreeMap<String, Vec<ProofErasureFieldStatus>> {
    let package_records = package_record_field_statuses(package);
    expected_targets
        .iter()
        .filter_map(|symbol| {
            package_records
                .get(symbol)
                .cloned()
                .map(|fields| (symbol.clone(), fields))
        })
        .collect()
}

fn package_record_field_statuses(
    package: &CheckedCorePackage,
) -> BTreeMap<String, Vec<ProofErasureFieldStatus>> {
    package
        .artifact
        .semantic
        .record_sigma_metadata
        .iter()
        .map(|(symbol, meta)| {
            (
                symbol.to_string(),
                meta.fields
                    .iter()
                    .map(|field| ProofErasureFieldStatus {
                        name: field.name.clone(),
                        status: runtime_field_status(&field.runtime),
                    })
                    .collect(),
            )
        })
        .collect()
}

fn require_erasure_lane_match<T: PartialEq + fmt::Debug>(
    package: &T,
    program: &T,
    lane: &'static str,
) -> Result<(), ErasureError> {
    if package == program {
        Ok(())
    } else {
        Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessMismatch,
            lane,
            format!(
                "CheckedCorePackage lane does not match RuntimeProgram lane: package={package:?}, program={program:?}"
            ),
        )
        .into())
    }
}

fn symbol_bytes_map(map: &BTreeMap<StableSymbol, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    map.iter()
        .map(|(symbol, bytes)| (symbol.to_string(), bytes.clone()))
        .collect()
}

fn obligation_metadata_map(
    map: &BTreeMap<StableSymbol, checked_core::ObligationMetadata>,
) -> BTreeMap<String, RuntimeObligationMetadata> {
    map.iter()
        .map(|(symbol, meta)| (symbol.to_string(), runtime_obligation_metadata(meta)))
        .collect()
}

fn assumption_trust_metadata_map(
    map: &BTreeMap<StableSymbol, checked_core::AssumptionTrustMetadata>,
) -> BTreeMap<String, RuntimeAssumptionTrustMetadata> {
    map.iter()
        .map(|(symbol, meta)| (symbol.to_string(), runtime_assumption_trust_metadata(meta)))
        .collect()
}

fn lowerability_map(
    map: &BTreeMap<StableSymbol, LowerabilityStatus>,
) -> BTreeMap<String, RuntimeLowerabilityStatus> {
    map.iter()
        .map(|(symbol, status)| (symbol.to_string(), runtime_lowerability_status(status)))
        .collect()
}

fn checked_core_metadata(
    semantic: &checked_core::CheckedCoreSemanticInputs,
) -> RuntimeCheckedCoreMetadata {
    RuntimeCheckedCoreMetadata {
        primitive_metadata: semantic
            .primitive_metadata
            .iter()
            .map(|(symbol, meta)| (symbol.to_string(), runtime_primitive_metadata(meta)))
            .collect(),
        data_metadata: semantic
            .data_metadata
            .iter()
            .map(|(symbol, meta)| (symbol.to_string(), runtime_data_metadata(meta)))
            .collect(),
        record_sigma_metadata: semantic
            .record_sigma_metadata
            .iter()
            .map(|(symbol, meta)| (symbol.to_string(), runtime_record_sigma_metadata(meta)))
            .collect(),
        class_instance_metadata: semantic
            .class_instance_metadata
            .iter()
            .map(|(symbol, meta)| (symbol.to_string(), runtime_class_instance_metadata(meta)))
            .collect(),
        recursion_metadata: semantic
            .recursion_metadata
            .iter()
            .map(|(symbol, meta)| (symbol.to_string(), runtime_recursion_metadata(meta)))
            .collect(),
        effects_foreign_metadata: semantic
            .effects_foreign_metadata
            .iter()
            .map(|(symbol, meta)| (symbol.to_string(), runtime_effects_foreign_metadata(meta)))
            .collect(),
        metadata: symbol_bytes_map(&semantic.metadata),
    }
}

fn runtime_obligation_metadata(
    meta: &checked_core::ObligationMetadata,
) -> RuntimeObligationMetadata {
    RuntimeObligationMetadata {
        status: match meta.status {
            checked_core::ObligationStatus::Proved => RuntimeObligationStatus::Proved,
            checked_core::ObligationStatus::Tested => RuntimeObligationStatus::Tested,
            checked_core::ObligationStatus::Delegated => RuntimeObligationStatus::Delegated,
            checked_core::ObligationStatus::Unknown => RuntimeObligationStatus::Unknown,
            checked_core::ObligationStatus::Disproved => RuntimeObligationStatus::Disproved,
        },
        origin: meta.origin.to_string(),
        affects_runtime_meaning: meta.affects_runtime_meaning,
    }
}

fn runtime_assumption_trust_metadata(
    meta: &checked_core::AssumptionTrustMetadata,
) -> RuntimeAssumptionTrustMetadata {
    RuntimeAssumptionTrustMetadata {
        kind: match meta.kind {
            checked_core::AssumptionTrustKind::Postulate => RuntimeAssumptionTrustKind::Postulate,
            checked_core::AssumptionTrustKind::Hole => RuntimeAssumptionTrustKind::Hole,
            checked_core::AssumptionTrustKind::Foreign => RuntimeAssumptionTrustKind::Foreign,
            checked_core::AssumptionTrustKind::Declassify => RuntimeAssumptionTrustKind::Declassify,
            checked_core::AssumptionTrustKind::PrimitiveAssumption => {
                RuntimeAssumptionTrustKind::PrimitiveAssumption
            }
        },
        target: meta.target.to_string(),
        affects_runtime_meaning: meta.affects_runtime_meaning,
    }
}

fn runtime_lowerability_status(status: &LowerabilityStatus) -> RuntimeLowerabilityStatus {
    match status {
        LowerabilityStatus::Supported => RuntimeLowerabilityStatus::Supported,
        LowerabilityStatus::Unsupported { reason } => RuntimeLowerabilityStatus::Unsupported {
            reason: reason.clone(),
        },
        LowerabilityStatus::Deferred {
            later_stage,
            reason,
        } => RuntimeLowerabilityStatus::Deferred {
            later_stage: later_stage.clone(),
            reason: reason.clone(),
        },
        LowerabilityStatus::RequiresFeature { feature, reason } => {
            RuntimeLowerabilityStatus::RequiresFeature {
                feature: feature.clone(),
                reason: reason.clone(),
            }
        }
        LowerabilityStatus::Explicit { state, reason } => RuntimeLowerabilityStatus::Explicit {
            state: state.clone(),
            reason: reason.clone(),
        },
    }
}

fn runtime_primitive_metadata(meta: &PrimitiveMetadata) -> RuntimePrimitiveAuditMetadata {
    RuntimePrimitiveAuditMetadata {
        registry_symbol: meta.registry_symbol.clone(),
        reduction: match meta.reduction {
            checked_core::PrimitiveReductionMetadata::OpaqueType => {
                RuntimePrimitiveReductionMetadata::OpaqueType
            }
            checked_core::PrimitiveReductionMetadata::Literal => {
                RuntimePrimitiveReductionMetadata::Literal
            }
            checked_core::PrimitiveReductionMetadata::Op => RuntimePrimitiveReductionMetadata::Op,
        },
        partiality: match &meta.partiality {
            PartialityMetadata::Total => RuntimePartialityMetadata::Total,
            PartialityMetadata::CheckedPartial { obligation } => {
                RuntimePartialityMetadata::CheckedPartial {
                    obligation: obligation.to_string(),
                }
            }
            PartialityMetadata::TrustedPartial { assumption } => {
                RuntimePartialityMetadata::TrustedPartial {
                    assumption: assumption.to_string(),
                }
            }
        },
        lowerability: runtime_lowerability_status(&meta.lowerability),
    }
}

fn runtime_data_metadata(meta: &DataMetadata) -> RuntimeDataAuditMetadata {
    RuntimeDataAuditMetadata {
        parameter_count: meta.parameter_count,
        index_count: meta.index_count,
        constructors: meta
            .constructors
            .iter()
            .map(|ctor| RuntimeConstructorAuditMetadata {
                symbol: ctor.symbol.to_string(),
                argument_count: ctor.argument_count,
                target_index_count: ctor.target_index_count,
                recursive_positions: ctor.recursive_positions.clone(),
                lowerability: runtime_lowerability_status(&ctor.lowerability),
            })
            .collect(),
        eliminator: runtime_lowerability_status(&meta.eliminator),
        lowerability: runtime_lowerability_status(&meta.lowerability),
    }
}

fn runtime_record_sigma_metadata(meta: &RecordSigmaMetadata) -> RuntimeRecordSigmaAuditMetadata {
    RuntimeRecordSigmaAuditMetadata {
        kind: match meta.kind {
            checked_core::RecordSigmaKind::Record => RuntimeRecordSigmaKind::Record,
            checked_core::RecordSigmaKind::Sigma => RuntimeRecordSigmaKind::Sigma,
        },
        fields: meta
            .fields
            .iter()
            .map(|field| RuntimeFieldAuditMetadata {
                name: field.name.clone(),
                ty: field.ty.to_string(),
                runtime: runtime_field_status(&field.runtime),
            })
            .collect(),
        lowerability: runtime_lowerability_status(&meta.lowerability),
    }
}

fn runtime_class_instance_metadata(
    meta: &ClassInstanceMetadata,
) -> RuntimeClassInstanceAuditMetadata {
    RuntimeClassInstanceAuditMetadata {
        kind: match meta.kind {
            ClassInstanceKind::Class => RuntimeClassInstanceKind::Class,
            ClassInstanceKind::Instance => RuntimeClassInstanceKind::Instance,
            ClassInstanceKind::Dictionary => RuntimeClassInstanceKind::Dictionary,
        },
        class_symbol: meta.class_symbol.as_ref().map(ToString::to_string),
        dictionary_symbol: meta.dictionary_symbol.as_ref().map(ToString::to_string),
        head_symbol: meta.head_symbol.as_ref().map(ToString::to_string),
        field_order: meta.field_order.clone(),
        runtime_fields: meta.runtime_fields.clone(),
        law_fields: meta.law_fields.clone(),
        lowerability: runtime_lowerability_status(&meta.lowerability),
    }
}

fn runtime_recursion_metadata(meta: &RecursionMetadata) -> RuntimeRecursionAuditMetadata {
    RuntimeRecursionAuditMetadata {
        group_members: meta.group_members.iter().map(ToString::to_string).collect(),
        admission: match meta.admission {
            checked_core::RecursionAdmission::NonRecursive => {
                RuntimeRecursionAdmission::NonRecursive
            }
            checked_core::RecursionAdmission::AcceptedStructural => {
                RuntimeRecursionAdmission::AcceptedStructural
            }
            checked_core::RecursionAdmission::AcceptedSizeChange => {
                RuntimeRecursionAdmission::AcceptedSizeChange
            }
            checked_core::RecursionAdmission::Rejected => RuntimeRecursionAdmission::Rejected,
        },
        scc_index: meta.scc_index,
        lowerability: runtime_lowerability_status(&meta.lowerability),
    }
}

fn runtime_effects_foreign_metadata(
    meta: &EffectsForeignMetadata,
) -> RuntimeEffectsForeignAuditMetadata {
    RuntimeEffectsForeignAuditMetadata {
        declared_effects: meta.declared_effects.clone(),
        capabilities: meta.capabilities.iter().map(ToString::to_string).collect(),
        foreign_symbol: meta.foreign_symbol.clone(),
        boundary: match meta.boundary {
            EffectBoundary::Pure => RuntimeEffectBoundary::Pure,
            EffectBoundary::Effectful => RuntimeEffectBoundary::Effectful,
            EffectBoundary::Foreign => RuntimeEffectBoundary::Foreign,
        },
        runtime_checks: meta
            .runtime_checks
            .iter()
            .map(ToString::to_string)
            .collect(),
        lowerability: runtime_lowerability_status(&meta.lowerability),
    }
}

fn runtime_field_status(status: &checked_core::RuntimeFieldStatus) -> RuntimeFieldStatus {
    match status {
        checked_core::RuntimeFieldStatus::Runtime => RuntimeFieldStatus::Runtime,
        checked_core::RuntimeFieldStatus::ErasedLaw => RuntimeFieldStatus::ErasedLaw,
        checked_core::RuntimeFieldStatus::ErasedProof => RuntimeFieldStatus::ErasedProof,
    }
}

fn runtime_checks_for_targets(
    package: &CheckedCorePackage,
    targets: &[StableSymbol],
) -> BTreeSet<String> {
    package
        .artifact
        .semantic
        .effects_foreign_metadata
        .iter()
        .filter(|(symbol, _)| targets.contains(symbol))
        .flat_map(|(_, meta)| meta.runtime_checks.iter().map(ToString::to_string))
        .collect()
}

fn capabilities_for_targets(
    package: &CheckedCorePackage,
    targets: &[StableSymbol],
) -> BTreeSet<String> {
    package
        .artifact
        .semantic
        .effects_foreign_metadata
        .iter()
        .filter(|(symbol, _)| targets.contains(symbol))
        .flat_map(|(_, meta)| meta.capabilities.iter().map(ToString::to_string))
        .collect()
}

fn effects_for_targets(package: &CheckedCorePackage, targets: &[StableSymbol]) -> BTreeSet<String> {
    package
        .artifact
        .semantic
        .effects_foreign_metadata
        .iter()
        .filter(|(symbol, _)| targets.contains(symbol))
        .flat_map(|(_, meta)| meta.declared_effects.iter().cloned())
        .collect()
}

#[cfg(test)]
mod px7l_tests {
    use super::*;

    fn test_answer_interface() -> ken_runtime::CheckedAnswerInterfaceV1 {
        ken_runtime::CheckedAnswerInterfaceV1::new(
            ken_runtime::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec(),
        )
        .expect("the fixed checked-answer header is canonical")
    }

    fn test_answer_symbols() -> CheckedJoinAnswerSymbols {
        CheckedJoinAnswerSymbols {
            int: StableSymbol::declaration("px8ta-total-census", &[], "Int"),
            bool_: StableSymbol::declaration("px8ta-total-census", &[], "Bool"),
            structural_nat: StableSymbol::declaration("px8ta-total-census", &[], "Nat"),
            exit_code: StableSymbol::declaration("px8ta-total-census", &[], "ExitCode"),
        }
    }

    #[test]
    fn finish_rejects_an_unconsumed_computational_ih_slot_seed() {
        let owner = StableSymbol::declaration("px8ta-total-census", &[], "main");
        let constructor = StableSymbol::constructor(
            &StableSymbol::declaration("px8ta-total-census", &[], "Tree"),
            "Step",
        );
        let seed = CheckedComputationalIHSlotSeed {
            slot_template_id: 7,
            owner: owner.clone(),
            match_ordinal: 0,
            branch_ordinal: 0,
            constructor,
            recursive_position: 0,
            method_binder_ordinal: 0,
            local_telescope: Vec::new(),
            ih_interface: test_answer_interface(),
        };
        let collector = NativeLoweringPlanCollector::new(
            test_answer_symbols(),
            Vec::new(),
            vec![seed],
            Vec::new(),
        );
        assert_eq!(
            lane(
                collector
                    .validate_total_computational_ih_seed_consumption()
                    .unwrap_err()
            ),
            "checked_computational_ih_slot_unconsumed"
        );
    }

    #[test]
    fn finish_rejects_an_unconsumed_computational_ih_call_seed() {
        let owner = StableSymbol::declaration("px8ta-total-census", &[], "main");
        let seed = CheckedComputationalIHCallSeed {
            call_template_id: 11,
            owner,
            slot_template_id: 7,
            occurrence_ordinal: 0,
            arity: 0,
            local_telescope: Vec::new(),
            result_interface: test_answer_interface(),
        };
        let collector = NativeLoweringPlanCollector::new(
            test_answer_symbols(),
            Vec::new(),
            Vec::new(),
            vec![seed],
        );
        assert_eq!(
            lane(
                collector
                    .validate_total_computational_ih_seed_consumption()
                    .unwrap_err()
            ),
            "checked_computational_ih_call_unconsumed"
        );
    }

    // -- `D7` checkpoint `1b`: one complete application, one arity ----------
    //
    // The defect these pin is a template describing an application ONE OPERAND
    // SHORTER than the one the emitter produces, because the Host-`Vis` route
    // appends the host result and the template counted only what the source
    // wrote.
    //
    // ⭐⭐ **Every row below drives a REAL erasure route.** The predecessor
    // controls hand-built the slot, the seed, the frame and the injected count,
    // never traversed the Host-`Vis` producer, and compared the resulting number
    // against a separately computed `source + 1` -- arithmetic against itself.
    // Here the fixture supplies only the checked inputs a route consumes; the
    // route decides its own injected count, emits its own application, and both
    // sides of the relation are then READ OFF that one traversal.
    //
    // ⭐ And the two mutation rows do not assert a number at all: they feed the
    // real route's disagreeing template and marker into the EXISTING Runtime
    // marker gate, which is the independent oracle -- a different crate, reading
    // the plan rather than the emitter's arithmetic, refusing before a function
    // is defined. Delete that gate and those rows go red.

    /// Install the checked SLOT side of one IH occurrence, so the fixtures below
    /// vary exactly one thing: the route that consumes the CALL.
    ///
    /// ⛔ The slot and its enclosing frame are installed directly. They are not
    /// what is under test, and driving a whole match view through the slot
    /// consumer would add moving parts to a one-variable measurement.
    fn ih_call_plans(owner: &StableSymbol, source_arguments: usize) -> NativeLoweringPlanCollector {
        let constructor = StableSymbol::constructor(
            &StableSymbol::declaration("d7-1b-arity", &[], "Tree"),
            "Step",
        );
        let slot = CheckedComputationalIHSlotSeed {
            slot_template_id: 7,
            owner: owner.clone(),
            match_ordinal: 0,
            branch_ordinal: 0,
            constructor,
            recursive_position: 0,
            method_binder_ordinal: 0,
            local_telescope: Vec::new(),
            ih_interface: test_answer_interface(),
        };
        let call = CheckedComputationalIHCallSeed {
            call_template_id: 11,
            owner: owner.clone(),
            slot_template_id: 7,
            occurrence_ordinal: 0,
            // The SOURCE application's argument count, which is what binds this
            // occurrence to its seed and which `1b` deliberately leaves alone.
            arity: source_arguments,
            local_telescope: Vec::new(),
            result_interface: test_answer_interface(),
        };
        let mut collector = NativeLoweringPlanCollector::new(
            test_answer_symbols(),
            Vec::new(),
            vec![slot.clone()],
            vec![call],
        );
        collector
            .oriented
            .frames
            .push(ken_runtime::OrientedSubcontinuationFramePlanV1 {
                frame_id: 0,
                segment_site_id: 0,
                declaration: owner.to_string(),
                checked_occurrence_path: vec![0],
                semantic_position: 0,
                input_interface: test_answer_interface(),
                output_interface: test_answer_interface(),
                runtime_frame_fingerprint: 0,
                occurrence_binding_fingerprint: 0,
                control_witness: ken_runtime::OrientedControlWitnessV1::DistinguishedRoot,
            });
        collector
            .pending_computational_ih_slots
            .push((slot, vec![0], 0));
        collector.consumed_computational_ih_slots.insert(7);
        collector
    }

    /// The checked IH application the routes consume: the slot-`7` binder at
    /// de Bruijn `0`, applied to `source_arguments` ordinary operands.
    fn ih_application(source_arguments: usize) -> CheckedCoreBodyTerm {
        (0..source_arguments).fold(
            CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 },
            |function, _| CheckedCoreBodyTerm::Application {
                function: Box::new(function),
                argument: Box::new(CheckedCoreBodyTerm::Variable { de_bruijn_index: 1 }),
            },
        )
    }

    /// The binder run the routes see: one live computational IH at slot `7`,
    /// then two ordinary binders.
    fn ih_remap() -> BranchBinderRemap {
        BranchBinderRemap::default().enter_match(2, 1, true, vec![7])
    }

    /// How many operands the marker's application ACTUALLY carries.
    ///
    /// ⛔ A nullary ordinary IH emits its bare callee rather than a `Call`, so
    /// "no application" is `0` operands and not a fixture error. That shape is
    /// itself refused by the Runtime marker gate, on a different clause.
    fn emitted_operand_count(marker: &RuntimeExpr) -> usize {
        let RuntimeExpr::CheckedComputationalIHInvocation { body, .. } = marker else {
            panic!("the route must emit a checked computational IH marker, got {marker:?}")
        };
        match body.as_ref() {
            RuntimeExpr::Call { args, .. } => args.len(),
            _ => 0,
        }
    }

    /// A static `FSOp.ReadFile` operation reading the two ORDINARY binders.
    ///
    /// ⛔ Deliberately not the IH binder at `0`: the operation is scenery here,
    /// and reusing the induction hypothesis as a capability would make the
    /// fixture say something it does not mean.
    fn host_vis_operation(spine: &CheckedHostSpineV1) -> CheckedCoreBodyTerm {
        let leaf = apply_constructor(
            constructor_view(&spine.fs_family, "ReadFile", 1, 2),
            vec![
                erased(),
                CheckedCoreBodyTerm::Variable { de_bruijn_index: 1 },
                CheckedCoreBodyTerm::Variable { de_bruijn_index: 2 },
            ],
        );
        apply_constructor(
            checked_core::CheckedCoreConstructorView {
                symbol: spine.in_l.clone(),
                ..constructor_view(&family("Coproduct"), "InL", 2, 1)
            },
            vec![erased(), erased(), leaf],
        )
    }

    /// Drive the REAL Host-`Vis` continuation route on `Vis op k`, where `k` is
    /// the checked IH application. Returns the marker the route emitted and the
    /// plan the same traversal produced.
    fn host_vis_route(
        source_arguments: usize,
    ) -> (RuntimeExpr, ken_runtime::OrientedSubcontinuationPlanV1) {
        let owner = StableSymbol::declaration("d7-1b-arity", &[], "main");
        let spine = spine();
        let mut plans = ih_call_plans(&owner, source_arguments);
        let term = apply_constructor(
            checked_core::CheckedCoreConstructorView {
                symbol: spine.vis.clone(),
                ..constructor_view(&family("ITree"), "Vis", 2, 2)
            },
            vec![
                erased(),
                erased(),
                host_vis_operation(&spine),
                ih_application(source_arguments),
            ],
        );
        let remap = ih_remap();
        let mut stack = vec![owner.clone()];
        let lowered = lower_checked_host_computation(
            &term,
            &BTreeMap::new(),
            &checked_core::CheckedCoreSemanticInputs::default(),
            &mut stack,
            &owner,
            3,
            &spine,
            Some(&remap),
            &[0],
            Some(&mut plans),
            Some(0),
        )
        .expect("the Vis continuation lowers through the real Host-Vis producer");
        // `Vis` becomes the host effect bound around its continuation; the
        // continuation is the marker this checkpoint is about.
        let RuntimeExpr::Let { body, .. } = lowered else {
            panic!("a static Host-Vis lowers to its effect bound around the continuation")
        };
        let (_, plan) = plans.finish();
        (*body, plan)
    }

    /// Drive the REAL ordinary application route on the same checked IH.
    fn ordinary_route(
        source_arguments: usize,
    ) -> (RuntimeExpr, ken_runtime::OrientedSubcontinuationPlanV1) {
        let owner = StableSymbol::declaration("d7-1b-arity", &[], "main");
        let mut plans = ih_call_plans(&owner, source_arguments);
        let remap = ih_remap();
        let mut stack = vec![owner.clone()];
        let lowered = lower_body_term_with_plans(
            &ih_application(source_arguments),
            &BTreeMap::new(),
            &checked_core::CheckedCoreSemanticInputs::default(),
            &mut stack,
            &owner,
            3,
            Some(&remap),
            &[0],
            &mut plans,
            Some(0),
        )
        .expect("the ordinary application lowers through the real marker consumer");
        let (_, plan) = plans.finish();
        (lowered, plan)
    }

    /// The one template a route produced.
    fn only_template_arity(plan: &ken_runtime::OrientedSubcontinuationPlanV1) -> u64 {
        assert_eq!(plan.computational_ih_calls.len(), 1);
        plan.computational_ih_calls[0].arity
    }

    /// **Feed a route's own marker and its own plan into the EXISTING Runtime
    /// marker gate**, and report the refusal that gate produced.
    ///
    /// ⭐⭐ This is the independent oracle. The elaborator does not get to say
    /// whether its template and its emission agree: `ken-runtime` decodes the
    /// arity out of the encoded plan, counts the operands in the emitted `Call`,
    /// and refuses at marker ENTRY -- before any function is defined or any
    /// instruction emitted for it.
    ///
    /// ⛔ **Everything below the marker is transport scaffolding, and none of it
    /// is what the rows measure.** `ken-runtime` refuses to lower a checked
    /// marker that is not seated in the declaration shape its transport
    /// preflight recognises -- a frame marker over a computational match, the
    /// slot marker in the selected case, the call marker inside that -- so the
    /// fixture supplies exactly that seat. The MARKER and the PLAN's call
    /// template are the real route's output and are inserted unchanged.
    fn runtime_marker_gate_refusal(
        marker: RuntimeExpr,
        mut plan: ken_runtime::OrientedSubcontinuationPlanV1,
    ) -> Option<(&'static str, String)> {
        let owner = StableSymbol::declaration("d7-1b-arity", &[], "main");
        let symbol = owner.to_string();
        let constructor =
            StableSymbol::constructor(&StableSymbol::declaration("d7-1b-arity", &[], "Tree"), "Step")
                .to_string();
        let cases = vec![RuntimeComputationalMatchCase {
            constructor: constructor.clone(),
            argument_binders: 1,
            recursive_positions: vec![0],
            body: RuntimeExpr::CheckedComputationalIHSlots {
                slot_template_ids: vec![7],
                checked_occurrence_paths: vec![vec![0]],
                body: Box::new(marker),
            },
        }];
        let default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "d7-1b marker-gate fixture".to_string(),
        };
        // The frame marker's fingerprint is a fact about the match it wraps, so
        // it can only be settled once the match exists. The plan row it lands on
        // is the frame this fixture installed itself.
        let frame_fingerprint =
            ken_runtime::compiler_private_computational_match_frame_fingerprint(&cases, &default);
        let body = RuntimeExpr::Closure {
            captures: Vec::new(),
            params: vec!["scrutinee".to_string()],
            body: Box::new(RuntimeExpr::CheckedSubcontinuationFrame {
                frame_id: 0,
                body: Box::new(RuntimeExpr::ComputationalMatch {
                    scrutinee: Box::new(RuntimeExpr::Construct {
                        constructor,
                        args: vec![RuntimeExpr::Value(RuntimeValue::Int(0.into()))],
                    }),
                    cases,
                    default,
                }),
            }),
        };
        let declaration = RuntimeDeclaration {
            symbol: symbol.clone(),
            kind: RuntimeDeclarationKind::Transparent { body },
            metadata: RuntimeSymbolMetadata {
                lowerability: Some(RuntimeLowerabilityStatus::Supported),
                ..RuntimeSymbolMetadata::empty()
            },
        };
        let frame = plan
            .frames
            .iter_mut()
            .find(|frame| frame.frame_id == 0)
            .expect("the fixture installed exactly one frame");
        frame.runtime_frame_fingerprint = frame_fingerprint;
        frame.occurrence_binding_fingerprint =
            ken_runtime::compiler_private_oriented_occurrence_binding_fingerprint(frame);
        let mut program = RuntimeProgram {
            package_identity: "d7-1b-marker-gate".to_string(),
            core_semantic_hash: 0,
            artifact_hash: 0,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::new(),
                metadata: RuntimeMetadata::default(),
            },
            declarations: vec![declaration],
            examples: Vec::new(),
        };
        // ⭐ The marker LOCATIONS are bound by the same production function the
        // driver uses, not transcribed by hand: a fixture that authored them
        // would be free to author them consistently with a template it also
        // authored, and the preflight would then be measuring nothing.
        bind_oriented_runtime_marker_locations(&owner, &program, &mut plan)
            .expect("the fixture's declaration seats every marker the plan names");
        program
            .erased_core
            .metadata
            .checked_core
            .metadata
            .insert("oriented-plan".to_string(), plan.canonical_bytes());
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Bool(true));
        let example = RuntimeExample {
            name: "d7-1b-marker-gate".to_string(),
            checked_core_shape: "CheckedComputationalIHInvocation".to_string(),
            ir: RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::DeclarationRef { symbol }),
                args: vec![RuntimeExpr::Value(RuntimeValue::Int(0.into()))],
            },
            observation: observation.clone(),
        };
        program.examples.push(example.clone());
        // ⛔ **The OBJECT-emitting entry, deliberately.** It is the entry that
        // carries the checked plan into lowering at all -- the differential
        // entry drops it, and a marker with no plan is refused by a different
        // clause that would make every row below vacuous. It is also what makes
        // "before definition or emission" observable: an `Err` here is an object
        // that was never produced.
        let artifact = RuntimeArtifactIdentity {
            package_identity: program.package_identity.clone(),
            core_semantic_hash: program.core_semantic_hash,
            artifact_hash: program.artifact_hash,
        };
        let target = RuntimeIrTargetIdentity::from_example(&example);
        let unavailable_fact = |reason: &str| RuntimeIrEvidenceFact::Unavailable {
            reason: reason.to_string(),
        };
        let run_report = RuntimeIrRunReport {
            evaluator: RuntimeIrEvaluator::DirectRuntimeIrEvaluatorV1,
            target: target.clone(),
            artifact: artifact.clone(),
            observation: RuntimeIrObservation {
                artifact,
                target: target.clone(),
                observation,
                evidence_source: "D7 1b marker-gate control".to_string(),
            },
            evidence: RuntimeIrRunEvidence {
                package_identity: program.package_identity.clone(),
                core_semantic_hash: program.core_semantic_hash,
                runtime_artifact_hash: program.artifact_hash,
                target_example: target.example.clone(),
                checked_core_shape: target.checked_core_shape.clone(),
                evidence_sources: BTreeMap::new(),
                unavailable: BTreeSet::new(),
            },
            trust: RuntimeIrTrustReport {
                tier: RuntimeIrTrustTier::RuntimeIrObservation,
                evaluator: unavailable_fact("D7 1b marker-gate control does not run the example"),
                interpreter_oracle: unavailable_fact("no interpreter oracle in this control"),
                native_backend: unavailable_fact("the control measures lowering admission only"),
                object_artifact: unavailable_fact("the control measures lowering admission only"),
                linker: unavailable_fact("no linker in this control"),
                source_level_proof: unavailable_fact("not a source-level semantics proof"),
            },
        };
        match emit_runtime_ir_object_with_cranelift(
            &program,
            &run_report,
            &NativeSeedEnvironment::empty(),
            "ken_d7_1b_marker_gate",
        ) {
            Ok(_) => None,
            Err(CraneliftBackendError::Unsupported(UnsupportedLowering { construct, reason })) => {
                Some((construct, reason))
            }
            Err(other) => Some(("BackendFailure", format!("{other:?}"))),
        }
    }

    /// The exact substring of the Runtime marker gate's arity refusal.
    const MARKER_ARITY_REFUSAL: &str = "checked template names arity";

    /// The construct every plan-, transport- and marker-level refusal in
    /// `ken-runtime` is raised under.
    ///
    /// ⭐⭐ **This is the complete positive control, and it is why the unmutated
    /// row is evidence rather than a coincidence.** A fixture that never reached
    /// the arity comparison would also fail to produce the arity refusal -- so
    /// "no arity refusal" alone proves nothing. Refusing under a DIFFERENT
    /// construct proves more: the transport preflight accepted the marker
    /// ledger, the marker resolved its template, it wrapped a complete
    /// application, and the arity comparison ran and passed. Every one of those
    /// failures would have named this construct.
    ///
    /// Measured: the unmutated fixture refuses at `Call` / `callee is not a
    /// closure` -- a refusal from lowering the marker's BODY, downstream of the
    /// gate. The exact downstream text is not asserted; it belongs to the
    /// fixture's scaffolding rather than to this property, and pinning it would
    /// red this row on unrelated lowering work.
    const CHECKED_PLAN_CONSTRUCT: &str = "OrientedSubcontinuationPlanV1";

    /// A real nullary Host-`Vis` continuation is ONE operand at both
    /// coordinates: the route injects the host result, the emitted application
    /// carries it, and the template names it.
    ///
    /// ⭐ The assertion is the RELATION -- template arity against the operand
    /// count this same traversal emitted. The literal `1` is asserted separately
    /// so the row also pins which shape was exercised.
    #[test]
    fn a_real_nullary_host_vis_continuation_is_one_operand_throughout() {
        let (marker, plan) = host_vis_route(0);
        assert_eq!(
            emitted_operand_count(&marker),
            1,
            "the source wrote nothing, so the emitted application is the injected host result alone"
        );
        assert_eq!(
            only_template_arity(&plan),
            emitted_operand_count(&marker) as u64,
            "the template names the application the route actually emitted"
        );
    }

    /// A real n-argument Host-`Vis` continuation is `n + 1` at both coordinates.
    ///
    /// ⭐ Two values of n, because one row cannot distinguish "adds the injected
    /// operand" from "returns a constant".
    #[test]
    fn a_real_n_argument_host_vis_continuation_is_n_plus_one_throughout() {
        for source_arguments in [1usize, 3] {
            let (marker, plan) = host_vis_route(source_arguments);
            assert_eq!(
                emitted_operand_count(&marker),
                source_arguments + 1,
                "the Host-Vis route emits the checked arguments plus the injected host result"
            );
            assert_eq!(
                only_template_arity(&plan),
                emitted_operand_count(&marker) as u64,
                "the template names the application the route actually emitted"
            );
        }
    }

    /// A real ordinary non-Host application injects nothing and stays at `n`.
    ///
    /// ⭐⭐ **This is the scoping half, and without it every row above is
    /// equally consistent with a global `+1`** that would shift every
    /// application in the program -- which is exactly the defect the first
    /// attempt at this checkpoint shipped, and exactly what took the five `fs_*`
    /// parity rows off their framed base refusal.
    #[test]
    fn a_real_ordinary_application_injects_nothing_and_keeps_its_own_arity() {
        for source_arguments in [0usize, 1, 2] {
            let (marker, plan) = ordinary_route(source_arguments);
            assert_eq!(
                emitted_operand_count(&marker),
                source_arguments,
                "an ordinary application emits exactly what the source wrote"
            );
            assert_eq!(
                only_template_arity(&plan),
                source_arguments as u64,
                "and its template names exactly that, with no injected operand"
            );
        }
    }

    /// **The unmutated Host-`Vis` route gets PAST the Runtime marker gate's
    /// arity comparison.**
    ///
    /// ⭐⭐ This is the positive control for the two mutation rows, and it is
    /// what makes their refusals evidence rather than coincidence. A fixture
    /// that never reached the comparison would also produce "no arity refusal"
    /// here -- so the row additionally asserts that none of the gate's EARLIER
    /// clauses fired, which is only possible if the marker, its `Call` and its
    /// template were all present and well-formed at the comparison.
    #[test]
    fn the_unmutated_host_vis_marker_passes_the_runtime_marker_gate() {
        for source_arguments in [0usize, 2] {
            let (marker, plan) = host_vis_route(source_arguments);
            let Some((construct, reason)) = runtime_marker_gate_refusal(marker, plan) else {
                continue;
            };
            assert!(
                !reason.contains(MARKER_ARITY_REFUSAL),
                "the correct template agrees with the emitted call, got {reason:?}"
            );
            assert_ne!(
                construct, CHECKED_PLAN_CONSTRUCT,
                "the fixture must get past the checked-plan gates entirely, not stop inside \
                 them: {reason:?}"
            );
        }
    }

    /// **Omitting the injected result at the Host-`Vis` route is refused by the
    /// Runtime marker gate, before definition or emission.**
    ///
    /// The mutation changes only the count the TEMPLATE is built from; the route
    /// still appends exactly one `Var(0)`. So the emitted application carries one
    /// operand more than the template names, and the gate says so.
    #[test]
    fn omitting_the_injected_result_is_refused_by_the_runtime_marker_gate() {
        for source_arguments in [0usize, 2] {
            let (marker, plan) = with_host_vis_injection_mutation(
                HostVisInjectionMutation::OmitInjectedResult,
                || host_vis_route(source_arguments),
            );
            assert_eq!(
                emitted_operand_count(&marker),
                source_arguments + 1,
                "the mutation must leave the EMISSION alone; only the template moves"
            );
            assert_eq!(
                only_template_arity(&plan),
                source_arguments as u64,
                "omitting the injected result names one operand too few"
            );
            let (construct, reason) = runtime_marker_gate_refusal(marker, plan)
                .expect("a template that disagrees with its emission is refused");
            assert_eq!(construct, CHECKED_PLAN_CONSTRUCT, "{reason:?}");
            assert!(
                reason.contains(MARKER_ARITY_REFUSAL),
                "the Runtime marker gate must be the one that refuses, got {reason:?}"
            );
        }
    }

    /// **Double-counting the injected result is refused the same way.**
    ///
    /// ⭐⭐ **Both directions, because a correction that can only be wrong one
    /// way is half-measured.** The route emits `source + 1`, so exactly one
    /// injected count agrees; one too few and one too many are each caught by
    /// the same gate.
    #[test]
    fn double_counting_the_injected_result_is_refused_by_the_runtime_marker_gate() {
        for source_arguments in [0usize, 2] {
            let (marker, plan) = with_host_vis_injection_mutation(
                HostVisInjectionMutation::DoubleCountInjectedResult,
                || host_vis_route(source_arguments),
            );
            assert_eq!(
                emitted_operand_count(&marker),
                source_arguments + 1,
                "the mutation must leave the EMISSION alone; only the template moves"
            );
            assert_eq!(
                only_template_arity(&plan),
                source_arguments as u64 + 2,
                "double-counting it names one operand too many"
            );
            let (construct, reason) = runtime_marker_gate_refusal(marker, plan)
                .expect("a template that disagrees with its emission is refused");
            assert_eq!(construct, CHECKED_PLAN_CONSTRUCT, "{reason:?}");
            assert!(
                reason.contains(MARKER_ARITY_REFUSAL),
                "the Runtime marker gate must be the one that refuses, got {reason:?}"
            );
        }
    }

    #[test]
    fn erased_constructor_parameter_and_live_ih_argument_emit_one_runtime_marker() {
        let owner = StableSymbol::declaration("px8ta-runtime-call-census", &[], "main");
        let family = StableSymbol::declaration("px8ta-runtime-call-census", &[], "Box");
        let term = apply_constructor(
            constructor_view(&family, "MkBox", 1, 1),
            vec![
                CheckedCoreBodyTerm::ErasedConstructorArgument { term: vec![0] },
                CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 },
            ],
        );
        let seed = CheckedComputationalIHCallSeed {
            call_template_id: 11,
            owner: owner.clone(),
            slot_template_id: 7,
            occurrence_ordinal: 0,
            arity: 0,
            local_telescope: Vec::new(),
            result_interface: test_answer_interface(),
        };
        let mut plans = NativeLoweringPlanCollector::new(
            test_answer_symbols(),
            Vec::new(),
            Vec::new(),
            vec![seed],
        );
        let remap = BranchBinderRemap::default().enter_match(0, 1, true, vec![7]);
        let mut stack = vec![owner.clone()];
        let lowered = lower_body_term_with_plans(
            &term,
            &BTreeMap::new(),
            &checked_core::CheckedCoreSemanticInputs::default(),
            &mut stack,
            &owner,
            1,
            Some(&remap),
            &[],
            &mut plans,
            None,
        )
        .expect("the mixed constructor application lowers through the real marker consumer");

        let RuntimeExpr::Construct { args, .. } = lowered else {
            panic!("the mixed constructor must remain a Runtime constructor")
        };
        assert_eq!(args.len(), 1, "the erased family parameter is absent");
        assert!(matches!(
            &args[0],
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id: 11,
                body,
                ..
            } if matches!(body.as_ref(), RuntimeExpr::Var(0))
        ));
        assert_eq!(plans.consumed_computational_ih_calls, BTreeSet::from([11]));
        assert_eq!(plans.pending_computational_ih_calls.len(), 1);
        plans
            .validate_total_computational_ih_seed_consumption()
            .expect("the one supplied live call seed is consumed exactly once");
    }

    fn oriented_match_view(name: &str, motive: u8) -> checked_core::CheckedCoreMatchView {
        checked_core::CheckedCoreMatchView {
            family_symbol: StableSymbol::declaration("px8ta", &[], name),
            level_args: Vec::new(),
            parameters: Vec::new(),
            motive: vec![motive],
            indices: Vec::new(),
            scrutinee: Box::new(CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 }),
            branches: Vec::new(),
            computational_recursive_hypotheses: true,
        }
    }

    fn oriented_runtime_frame(name: &str) -> RuntimeExpr {
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: Vec::new(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: name.to_string(),
            },
        }
    }

    #[test]
    fn checked_producer_emits_inner_to_outer_semantic_chain_before_erasure() {
        let owner = StableSymbol::declaration("px8ta", &[], "main");
        let mut collector = OrientedSubcontinuationPlanCollector::default();
        let outer = collector
            .begin_match(&owner, &[0], None, &oriented_match_view("P0", 0))
            .unwrap()
            .unwrap();
        let middle = collector
            .begin_match(
                &owner,
                &[0, 1],
                Some(outer.frame_id),
                &oriented_match_view("P1", 1),
            )
            .unwrap()
            .unwrap();
        let inner = collector
            .begin_match(
                &owner,
                &[0, 1, 2],
                Some(middle.frame_id),
                &oriented_match_view("P2", 2),
            )
            .unwrap()
            .unwrap();
        collector
            .finish_match(inner, &oriented_runtime_frame("p2"))
            .unwrap();
        collector
            .finish_match(middle, &oriented_runtime_frame("p1"))
            .unwrap();
        collector
            .finish_match(outer, &oriented_runtime_frame("p0"))
            .unwrap();

        let plan = collector.finish(
            BTreeMap::new(),
            BTreeSet::new(),
            Vec::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            Vec::new(),
            Vec::new(),
        );
        plan.validate().unwrap();
        let mut frames = plan.frames.iter().collect::<Vec<_>>();
        frames.sort_by_key(|frame| frame.semantic_position);
        assert_eq!(
            frames
                .iter()
                .map(|frame| frame.frame_id)
                .collect::<Vec<_>>(),
            vec![2, 1, 0],
            "checked postorder is the semantic p2, p1, p0 chain"
        );
        assert!(
            frames
                .windows(2)
                .all(|pair| { pair[0].output_interface == pair[1].input_interface }),
            "every checked answer endpoint closes before Runtime erasure"
        );
        assert!(
            frames
                .iter()
                .all(|frame| frame.segment_site_id == frames[0].segment_site_id),
            "all three frames belong to one prompt region"
        );
    }

    fn family(name: &str) -> StableSymbol {
        StableSymbol::declaration("px7l", &[], name)
    }

    fn constructor_view(
        family: &StableSymbol,
        name: &str,
        family_parameter_count: usize,
        argument_count: usize,
    ) -> checked_core::CheckedCoreConstructorView {
        checked_core::CheckedCoreConstructorView {
            symbol: StableSymbol::constructor(family, name),
            family_symbol: family.clone(),
            level_args: Vec::new(),
            family_parameter_count,
            family_index_count: 0,
            argument_count,
            target_index_count: 0,
            recursive_positions: Vec::new(),
            constructor_lowerability: LowerabilityStatus::Supported,
            family_lowerability: LowerabilityStatus::Supported,
        }
    }

    fn apply_constructor(
        constructor: checked_core::CheckedCoreConstructorView,
        args: Vec<CheckedCoreBodyTerm>,
    ) -> CheckedCoreBodyTerm {
        args.into_iter().fold(
            CheckedCoreBodyTerm::ConstructorReference(constructor),
            |function, argument| CheckedCoreBodyTerm::Application {
                function: Box::new(function),
                argument: Box::new(argument),
            },
        )
    }

    fn spine() -> CheckedHostSpineV1 {
        let fs_family = family("FSOp");
        let console_family = family("ConsoleOp");
        let clock_family = family("ClockOp");
        let entropy_family = family("EntropyOp");
        let coproduct = family("Coproduct");
        let in_l = StableSymbol::constructor(&coproduct, "InL");
        let in_r = StableSymbol::constructor(&coproduct, "InR");
        let read = StableSymbol::constructor(&fs_family, "ReadFile");
        CheckedHostSpineV1 {
            ret: StableSymbol::constructor(&family("ITree"), "Ret"),
            vis: StableSymbol::constructor(&family("ITree"), "Vis"),
            in_l,
            in_r,
            fs_family,
            console_family,
            clock_family,
            entropy_family,
            capability: family("Cap"),
            result_err: StableSymbol::constructor(&family("Result"), "Err"),
            result_ok: StableSymbol::constructor(&family("Result"), "Ok"),
            option_some: StableSymbol::constructor(&family("Option"), "Some"),
            file_error: StableSymbol::constructor(&family("FileError"), "MkFileError"),
            file_operation_read: StableSymbol::constructor(&family("FileOperation"), "Read"),
            file_operation_write: StableSymbol::constructor(&family("FileOperation"), "Write"),
            file_operation_change_mode: StableSymbol::constructor(
                &family("FileOperation"),
                "ChangeMode",
            ),
            io_errors: Vec::new(),
            resource_host_io: StableSymbol::constructor(&family("ResourceError"), "ResourceHostIO"),
            resource_closed: StableSymbol::constructor(&family("ResourceError"), "Closed"),
            resource_malformed: StableSymbol::constructor(
                &family("ResourceError"),
                "MalformedResource",
            ),
            resource_right_not_held: StableSymbol::constructor(
                &family("ResourceError"),
                "RightNotHeld",
            ),
            resource_release_failed: StableSymbol::constructor(
                &family("ResourceError"),
                "ReleaseFailed",
            ),
            resource_kind_mismatch: StableSymbol::constructor(
                &family("ResourceError"),
                "ResourceKindMismatch",
            ),
            resource_buffer_limit: StableSymbol::constructor(
                &family("ResourceError"),
                "BufferLimit",
            ),
            resource_allocation_failed: StableSymbol::constructor(
                &family("ResourceError"),
                "AllocationFailed",
            ),
            resource_invalid_offset: StableSymbol::constructor(
                &family("ResourceError"),
                "InvalidOffset",
            ),
            resource_invalid_bounds: StableSymbol::constructor(
                &family("ResourceError"),
                "InvalidBounds",
            ),
            resource_no_progress: StableSymbol::constructor(&family("ResourceError"), "NoProgress"),
            resource_kind_fs_handle: StableSymbol::constructor(&family("ResourceKind"), "FsHandle"),
            resource_kind_buffer: StableSymbol::constructor(&family("ResourceKind"), "Buffer"),
            resource_trace_identity: StableSymbol::constructor(
                &family("ResourceTraceIdentity"),
                "PrivateResourceTraceIdentity",
            ),
            nat_zero: StableSymbol::constructor(&family("Nat"), "Zero"),
            nat_suc: StableSymbol::constructor(&family("Nat"), "Suc"),
            private_buffer_span: StableSymbol::constructor(
                &family("BufferSpan"),
                "PrivateBufferSpan",
            ),
            private_transfer_count: StableSymbol::constructor(
                &family("TransferCount"),
                "PrivateTransferCount",
            ),
            read_some: StableSymbol::constructor(&family("ReadProgress"), "ReadSome"),
            read_eof: StableSymbol::constructor(&family("ReadProgress"), "ReadEof"),
            wrote: StableSymbol::constructor(&family("WriteProgress"), "Wrote"),
            unit: StableSymbol::constructor(&family("Unit"), "MkUnit"),
            bool_false: StableSymbol::constructor(&family("Bool"), "False"),
            bool_true: StableSymbol::constructor(&family("Bool"), "True"),
            operations: BTreeMap::from([(read, ken_host::HostOpV1::FsReadFile)]),
        }
    }

    fn erased() -> CheckedCoreBodyTerm {
        CheckedCoreBodyTerm::ErasedConstructorArgument { term: Vec::new() }
    }

    fn static_fs_read(spine: &CheckedHostSpineV1) -> CheckedCoreBodyTerm {
        let read = constructor_view(&spine.fs_family, "ReadFile", 1, 2);
        let leaf = apply_constructor(
            read,
            vec![
                erased(),
                CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 },
                CheckedCoreBodyTerm::Variable { de_bruijn_index: 1 },
            ],
        );
        let coproduct = family("Coproduct");
        apply_constructor(
            checked_core::CheckedCoreConstructorView {
                symbol: spine.in_l.clone(),
                ..constructor_view(&coproduct, "InL", 2, 1)
            },
            vec![erased(), erased(), leaf],
        )
    }

    fn static_fs_read_with_leaf_family(
        spine: &CheckedHostSpineV1,
        leaf_family: &StableSymbol,
    ) -> CheckedCoreBodyTerm {
        let mut read = constructor_view(&spine.fs_family, "ReadFile", 1, 2);
        read.family_symbol = leaf_family.clone();
        let leaf = apply_constructor(
            read,
            vec![
                erased(),
                CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 },
                CheckedCoreBodyTerm::Variable { de_bruijn_index: 1 },
            ],
        );
        let coproduct = family("Coproduct");
        apply_constructor(
            checked_core::CheckedCoreConstructorView {
                symbol: spine.in_l.clone(),
                ..constructor_view(&coproduct, "InL", 2, 1)
            },
            vec![erased(), erased(), leaf],
        )
    }

    fn lane(error: ErasureError) -> &'static str {
        match error {
            ErasureError::ExpressionLowering { lane, .. } => lane,
            other => panic!("expected expression-lowering error, got {other:?}"),
        }
    }

    #[test]
    fn runtime_selected_dispatch_is_load_bearing_and_static_decode_stays_exact() {
        let spine = spine();
        let root = family("main");
        let dynamic = CheckedCoreBodyTerm::Variable { de_bruijn_index: 2 };
        assert!(matches!(
            select_checked_host_operation(&dynamic, &spine, &root, true).unwrap(),
            CheckedHostOperationSelection::RuntimeSelected
        ));
        assert_eq!(
            lane(select_checked_host_operation(&dynamic, &spine, &root, false).unwrap_err()),
            "host_coproduct_shape"
        );

        let static_read = static_fs_read(&spine);
        let CheckedHostOperationSelection::Static(decoded) =
            select_checked_host_operation(&static_read, &spine, &root, false).unwrap()
        else {
            panic!("visible constructor spine must retain the static path")
        };
        assert_eq!(decoded.operation, ken_host::HostOpV1::FsReadFile);
    }

    #[test]
    fn computational_ih_and_capture_order_have_independent_opposites() {
        let preserved = BranchBinderRemap::default().enter_match(2, 1, true, Vec::new());
        assert_eq!(preserved.runtime_index(0), Some(0), "IH is live");
        assert_eq!(preserved.runtime_index(1), Some(2), "continuation order");
        assert_eq!(preserved.runtime_index(2), Some(1), "operation order");

        let erased = BranchBinderRemap::default().enter_match(2, 1, false, Vec::new());
        assert_eq!(erased.runtime_index(0), None, "erased-IH mutation flips");
        let root = family("main");
        let mut stack = vec![root.clone()];
        let erased_error = lower_body_term_inner(
            &CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 },
            &BTreeMap::new(),
            &checked_core::CheckedCoreSemanticInputs::default(),
            &mut stack,
            &root,
            3,
            Some(&erased),
        )
        .unwrap_err();
        assert_eq!(
            lane(erased_error),
            "erased_induction_hypothesis_reached_runtime"
        );

        let under_lambda = preserved.enter_binding();
        assert_eq!(under_lambda.runtime_index(0), Some(0), "lambda parameter");
        assert_eq!(under_lambda.runtime_index(1), Some(1), "captured IH");
        assert_eq!(
            under_lambda.runtime_index(2),
            Some(3),
            "captured continuation"
        );
        assert_eq!(under_lambda.runtime_index(3), Some(2), "captured operation");
        assert_ne!(
            under_lambda.runtime_index(2),
            under_lambda.runtime_index(3),
            "capture-order swap is discriminating"
        );
    }

    #[test]
    fn runtime_selected_response_binder_is_not_shifted_with_its_free_environment() {
        let continuation = RuntimeExpr::Construct {
            constructor: "px7l::ResponseAndOuter".to_string(),
            args: vec![RuntimeExpr::Var(0), RuntimeExpr::Var(1)],
        };
        // ⛔ `RuntimeExpr: PartialEq` is gone (`RT-VALUE-TOTALITY-P2` `D2`): it
        // reached `RuntimeValue::ClosureRef`, so whole-expression equality was
        // ordinary-closure structural equality, which `41 §2.1` denies. The
        // property this control owns is a statement about TWO VAR INDICES, so
        // it is asserted directly — no whole-expression comparison, and
        // deliberately no shared projection helper, which would restore the
        // forbidden capability under a private spelling.
        //
        // Destructuring is itself part of the control: a wrong variant panics
        // loudly rather than comparing as unequal.
        let var_args = |expr: RuntimeExpr| -> Vec<u32> {
            let RuntimeExpr::Construct { constructor, args } = expr else {
                panic!("shift_runtime_vars did not return a Construct");
            };
            assert_eq!(constructor, "px7l::ResponseAndOuter");
            args.into_iter()
                .map(|arg| match arg {
                    RuntimeExpr::Var(index) => index,
                    other => panic!("expected a Var argument, got {other:?}"),
                })
                .collect()
        };

        // The accepted cutoff: the response binder stays bound at 0, and only
        // the free environment variable shifts 1 -> 4.
        assert_eq!(
            var_args(shift_runtime_vars(continuation.clone(), 3, 1)),
            vec![0, 4],
            "response Var(0) stays bound while only the free environment shifts"
        );

        // ⚠ The rejected cutoff-0 mutation, asserted by its EXACT result rather
        // than by mere inequality. `assert_ne!` on whole expressions also passed
        // if some unrelated field moved; pinning `[3, 4]` says precisely what
        // goes wrong — the live response binder is dragged from 0 to 3.
        assert_eq!(
            var_args(shift_runtime_vars(continuation, 3, 0)),
            vec![3, 4],
            "the rejected cutoff-0 mutation moves the live response binder"
        );
    }

    #[test]
    fn checked_host_identity_failures_remain_specific_and_closed() {
        let root = family("main");

        let mut unknown = spine();
        unknown.operations.clear();
        assert_eq!(
            lane(
                decode_checked_host_operation(&static_fs_read(&unknown), &unknown, &root)
                    .unwrap_err()
            ),
            "unknown_host_operation_identity"
        );

        let wrong_family = spine();
        assert_eq!(
            lane(
                decode_checked_host_operation(
                    &static_fs_read_with_leaf_family(&wrong_family, &wrong_family.console_family,),
                    &wrong_family,
                    &root,
                )
                .unwrap_err(),
            ),
            "host_operation_family_identity"
        );

        let malformed = spine();
        assert_eq!(
            lane(
                decode_checked_host_operation(
                    &CheckedCoreBodyTerm::Variable { de_bruijn_index: 7 },
                    &malformed,
                    &root,
                )
                .unwrap_err(),
            ),
            "host_coproduct_shape"
        );
    }

    #[test]
    fn static_and_runtime_selected_capability_policies_are_exhaustively_identical() {
        for operation in ken_host::HostOpV1::ALL {
            let expected = !operation.is_ambient()
                && !matches!(
                    operation,
                    ken_host::HostOpV1::FsHandleMetadata
                        | ken_host::HostOpV1::BufferAllocate
                        | ken_host::HostOpV1::FsReadAt
                        | ken_host::HostOpV1::FsWriteAt
                        | ken_host::HostOpV1::BufferFreeze
                        | ken_host::HostOpV1::ResourceRelease
                );
            assert_eq!(
                static_host_operation_requires_capability(operation),
                expected,
                "static capability policy drifted for {operation:?}"
            );
            assert_eq!(
                runtime_selected_host_operation_requires_capability(operation),
                expected,
                "runtime-selected capability policy drifted for {operation:?}"
            );
            assert_eq!(
                static_host_operation_requires_capability(operation),
                runtime_selected_host_operation_requires_capability(operation),
                "static and runtime-selected paths disagree for {operation:?}"
            );
        }
    }
}
