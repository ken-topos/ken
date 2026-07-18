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
    erase_checked_package_with_host_root(package, targets, None)
}

fn erase_checked_package_with_host_root(
    package: &CheckedCorePackage,
    mut targets: Vec<StableSymbol>,
    host_root: Option<(&StableSymbol, &CheckedHostSpineV1)>,
) -> Result<RuntimeProgram, ErasureError> {
    validate_checked_core_package(package)?;
    let requested_targets = targets.clone();
    let mut prelowered = BTreeMap::new();
    if let Some((root, spine)) = host_root {
        let root_kind = lower_checked_host_root(package, &requested_targets, root, spine)?;
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
            let declaration =
                match lower_checked_host_declaration(package, &requested_targets, &symbol, spine) {
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
    erase_checked_package_with_host_root(package, targets, Some((root, spine)))
}

fn lower_checked_host_root(
    package: &CheckedCorePackage,
    target_closure: &[StableSymbol],
    root: &StableSymbol,
    spine: &CheckedHostSpineV1,
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
    )?;
    Ok(RuntimeDeclarationKind::Transparent {
        body: RuntimeExpr::Closure {
            captures: Vec::new(),
            params: vec!["process_input".to_string(), "program_caps".to_string()],
            body: Box::new(lowered),
        },
    })
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
) -> Result<RuntimeExpr, ErasureError> {
    if let CheckedCoreBodyTerm::Match(view) = term {
        reject_level_args(root, &view.level_args)?;
        if !view.indices.is_empty() {
            return Err(expression_lowering_error(
                root,
                "host_match_identity",
                "checked host Match carries runtime indices",
            ));
        }
        let scrutinee = lower_body_term_inner(
            &view.scrutinee,
            declarations,
            semantic,
            stack,
            root,
            context_depth,
            branch_remap,
        )?;
        let computational = match_uses_computational_recursive_hypothesis(view, root)?;
        let cases = view
            .branches
            .iter()
            .map(|branch| {
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
                let remap = branch_remap.cloned().unwrap_or_default().enter_match(
                    constructor.argument_count,
                    erased_count,
                    computational,
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
                )?;
                Ok((constructor, body))
            })
            .collect::<Result<Vec<_>, ErasureError>>()?;
        let default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "checked HostIO match had no constructor arm".to_string(),
        };
        return if computational {
            Ok(RuntimeExpr::ComputationalMatch {
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
            })
        } else {
            Ok(RuntimeExpr::Match {
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
            })
        };
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
            if parameter_count == arguments.len() && !stack.contains(symbol) {
                let values = arguments
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
                )?
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
    if is_recursive_declaration_application(term) {
        return lower_body_term_inner(
            term,
            declarations,
            semantic,
            stack,
            root,
            context_depth,
            branch_remap,
        );
    }
    Err(expression_lowering_error(
        root,
        "unrecognized_checked_host_computation",
        "normalized HostIO body is neither identity-checked Ret nor Vis",
    ))
}

fn is_recursive_declaration_application(term: &CheckedCoreBodyTerm) -> bool {
    let mut current = term;
    while let CheckedCoreBodyTerm::Application { function, .. } = current {
        current = function;
    }
    match current {
        CheckedCoreBodyTerm::RecursiveDeclarationCall(_) => true,
        _ => false,
    }
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
    let expected_family = if operation == ken_host::HostOpV1::ClockWallNow {
        &spine.clock_family
    } else if operation.is_ambient() {
        &spine.console_family
    } else {
        &spine.fs_family
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
            let expected_family = if host_operation == ken_host::HostOpV1::ClockWallNow {
                &spine.clock_family
            } else if host_operation.is_ambient() {
                &spine.console_family
            } else {
                &spine.fs_family
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
                body: clock,
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
            Ok(RuntimeExpr::Value(RuntimeValue::Int(*value)))
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
        return raw.parse::<i64>().ok().map(RuntimeValue::Int);
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
    if !view.computational_recursive_hypotheses {
        return Ok(false);
    }
    for branch in &view.branches {
        let recursive_count = branch.constructor.recursive_positions.len();
        if recursive_count == 0 {
            continue;
        }
        let body = peel_match_branch_method(
            &branch.method,
            branch.constructor.argument_count + recursive_count,
            root_symbol,
            &branch.constructor.symbol,
        )?;
        if references_outer_binder_range(body, 0, recursive_count, 0) {
            return Ok(true);
        }
    }
    Ok(false)
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

fn references_outer_binder_range(
    term: &CheckedCoreBodyTerm,
    start: usize,
    end: usize,
    local_depth: usize,
) -> bool {
    match term {
        CheckedCoreBodyTerm::Variable { de_bruijn_index } => {
            (local_depth + start..local_depth + end).contains(de_bruijn_index)
        }
        CheckedCoreBodyTerm::IntegerLiteral { .. }
        | CheckedCoreBodyTerm::DirectDeclarationCall { .. }
        | CheckedCoreBodyTerm::RecursiveDeclarationCall(_)
        | CheckedCoreBodyTerm::ImportedDeclarationCall(_)
        | CheckedCoreBodyTerm::PrimitiveLiteral(_)
        | CheckedCoreBodyTerm::ConstructorReference(_)
        | CheckedCoreBodyTerm::ErasedConstructorArgument { .. } => false,
        CheckedCoreBodyTerm::PrimitiveApplication(view) => view
            .arguments
            .iter()
            .any(|term| references_outer_binder_range(term, start, end, local_depth)),
        CheckedCoreBodyTerm::Lambda { body, .. } => {
            references_outer_binder_range(body, start, end, local_depth + 1)
        }
        CheckedCoreBodyTerm::Application { function, argument } => {
            references_outer_binder_range(function, start, end, local_depth)
                || references_outer_binder_range(argument, start, end, local_depth)
        }
        CheckedCoreBodyTerm::Let { value, body, .. } => {
            references_outer_binder_range(value, start, end, local_depth)
                || references_outer_binder_range(body, start, end, local_depth + 1)
        }
        CheckedCoreBodyTerm::Match(view) => {
            references_outer_binder_range(&view.scrutinee, start, end, local_depth)
                || view.branches.iter().any(|branch| {
                    references_outer_binder_range(&branch.method, start, end, local_depth)
                })
        }
        CheckedCoreBodyTerm::RecordSigmaConstruction(view) => {
            view.fields.iter().any(|field| match field {
                checked_core::CheckedCoreRecordSigmaFieldValue::Runtime { value, .. } => {
                    references_outer_binder_range(value, start, end, local_depth)
                }
                checked_core::CheckedCoreRecordSigmaFieldValue::Erased { .. } => false,
            })
        }
        CheckedCoreBodyTerm::RecordSigmaProjection(view) => {
            references_outer_binder_range(&view.base, start, end, local_depth)
        }
        CheckedCoreBodyTerm::DictionaryConstruction(view) => {
            view.fields.iter().any(|field| match field {
                checked_core::CheckedCoreDictionaryFieldValue::Runtime { value, .. } => {
                    references_outer_binder_range(value, start, end, local_depth)
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
        let preserved = BranchBinderRemap::default().enter_match(2, 1, true);
        assert_eq!(preserved.runtime_index(0), Some(0), "IH is live");
        assert_eq!(preserved.runtime_index(1), Some(2), "continuation order");
        assert_eq!(preserved.runtime_index(2), Some(1), "operation order");

        let erased = BranchBinderRemap::default().enter_match(2, 1, false);
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
        let corrected = shift_runtime_vars(continuation.clone(), 3, 1);
        assert_eq!(
            corrected,
            RuntimeExpr::Construct {
                constructor: "px7l::ResponseAndOuter".to_string(),
                args: vec![RuntimeExpr::Var(0), RuntimeExpr::Var(4)],
            },
            "response Var(0) stays bound while only the free environment shifts"
        );
        assert_ne!(
            shift_runtime_vars(continuation, 3, 0),
            corrected,
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
