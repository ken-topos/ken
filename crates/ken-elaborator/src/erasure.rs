//! Erasure boundary from `CheckedCorePackage v0` to Ken runtime IR.
//!
//! This module consumes only the checked-core package artifact. Source identity
//! may remain in the package envelope for diagnostics and provenance, but it is
//! never an input to runtime meaning here.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ken_runtime::*;

use crate::checked_core::{
    self, checked_core_body_view_for_selection, consume_checked_core_package_for_target,
    validate_checked_core_package, CheckedCoreBodyTerm, CheckedCoreBodyViewError,
    CheckedCoreBodyViewSelection, CheckedCoreLevelView, CheckedCorePackage,
    CheckedCorePackageError, ClassInstanceKind, ClassInstanceMetadata, DataMetadata,
    EffectBoundary, EffectsForeignMetadata, LowerabilityStatus, PartialityMetadata,
    PrimitiveMetadata, RecordSigmaMetadata, RecursionMetadata, StableSymbol,
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
    validate_checked_core_package(package)?;
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
        declarations.push(lower_symbol(package, &targets, target)?);
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
    let reachable_declarations = target_closure
        .iter()
        .filter(|candidate| {
            package
                .artifact
                .semantic
                .declarations
                .contains_key(*candidate)
                && !has_runtime_metadata(&package.artifact.semantic, candidate)
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    let selection = CheckedCoreBodyViewSelection {
        package_identity: package.header.package_identity.clone(),
        package_core_semantic_hash: package.core_semantic_hash,
        package_artifact_hash: package.artifact_hash,
        target_symbol: symbol.clone(),
        reachable_declarations,
    };
    let view = checked_core_body_view_for_selection(package, &selection)
        .map_err(|err| expression_view_error(symbol, err))?;
    let declaration = view.declarations.get(symbol).ok_or_else(|| {
        expression_lowering_error(
            symbol,
            "missing_expression_body_view",
            "body view did not return the selected transparent declaration",
        )
    })?;
    let mut stack = vec![symbol.clone()];
    let body = lower_body_term(&declaration.body, &view.declarations, &mut stack, symbol, 0)?;
    Ok(RuntimeDeclarationKind::Transparent { body })
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

fn lower_body_term(
    term: &CheckedCoreBodyTerm,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
) -> Result<RuntimeExpr, ErasureError> {
    lower_body_term_inner(term, declarations, stack, root_symbol, context_depth, None)
}

#[derive(Clone, Copy)]
struct BranchBinderRemap {
    start: usize,
    count: usize,
}

impl BranchBinderRemap {
    fn enter_binding(self) -> Self {
        Self {
            start: self.start + 1,
            count: self.count,
        }
    }

    fn runtime_index(self, de_bruijn_index: usize) -> usize {
        if (self.start..self.start + self.count).contains(&de_bruijn_index) {
            self.start + (self.count - 1 - (de_bruijn_index - self.start))
        } else {
            de_bruijn_index
        }
    }
}

fn lower_body_term_inner(
    term: &CheckedCoreBodyTerm,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<BranchBinderRemap>,
) -> Result<RuntimeExpr, ErasureError> {
    let owner = stack
        .last()
        .expect("expression lowering stack always has an owner")
        .clone();
    if let Some((constructor, args)) = constructor_application_spine(term) {
        return lower_constructor_application(
            constructor,
            &args,
            declarations,
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
                .unwrap_or(*de_bruijn_index);
            let index = u32::try_from(runtime_index).map_err(|_| {
                expression_lowering_error(
                    root_symbol,
                    "variable_index_overflow",
                    format!("variable index {runtime_index} does not fit runtime IR"),
                )
            })?;
            Ok(RuntimeExpr::Var(index))
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
                stack,
                root_symbol,
                context_depth,
                branch_remap,
            );
            stack.pop();
            lowered
        }
        CheckedCoreBodyTerm::Lambda { body, .. } => {
            if captures_outer_variable(body) {
                return Err(expression_lowering_error(
                    root_symbol,
                    "implicit_closure_capture",
                    "lambda body references an outer de Bruijn binding without an explicit runtime capture lane",
                ));
            }
            Ok(RuntimeExpr::Closure {
                captures: Vec::new(),
                params: vec!["arg0".to_string()],
                body: Box::new(lower_body_term_inner(
                    body,
                    declarations,
                    stack,
                    root_symbol,
                    context_depth + 1,
                    branch_remap.map(BranchBinderRemap::enter_binding),
                )?),
            })
        }
        CheckedCoreBodyTerm::Application { function, argument } => Ok(RuntimeExpr::Call {
            callee: Box::new(lower_body_term_inner(
                function,
                declarations,
                stack,
                root_symbol,
                context_depth,
                branch_remap,
            )?),
            args: vec![lower_body_term_inner(
                argument,
                declarations,
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
                stack,
                root_symbol,
                context_depth,
                branch_remap,
            )?),
            body: Box::new(lower_body_term_inner(
                body,
                declarations,
                stack,
                root_symbol,
                context_depth + 1,
                branch_remap.map(BranchBinderRemap::enter_binding),
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
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        ),
        CheckedCoreBodyTerm::RecordSigmaConstruction(view) => lower_record_sigma_construction(
            view,
            declarations,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        ),
        CheckedCoreBodyTerm::RecordSigmaProjection(view) => lower_record_sigma_projection(
            view,
            declarations,
            stack,
            root_symbol,
            context_depth,
            branch_remap,
        ),
    }
}

fn lower_record_sigma_construction(
    view: &checked_core::CheckedCoreRecordSigmaConstructionView,
    declarations: &BTreeMap<StableSymbol, checked_core::CheckedCoreDeclarationBodyView>,
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<BranchBinderRemap>,
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
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<BranchBinderRemap>,
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
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<BranchBinderRemap>,
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
    stack: &mut Vec<StableSymbol>,
    root_symbol: &StableSymbol,
    context_depth: usize,
    branch_remap: Option<BranchBinderRemap>,
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
        stack,
        root_symbol,
        context_depth,
        branch_remap,
    )?);
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
        let body = peel_match_branch_method(
            &branch.method,
            constructor.argument_count,
            root_symbol,
            &constructor.symbol,
        )?;
        cases.push(RuntimeMatchCase {
            constructor: constructor.symbol.to_string(),
            binders: constructor.argument_count,
            body: lower_body_term_inner(
                body,
                declarations,
                stack,
                root_symbol,
                context_depth + constructor.argument_count,
                Some(BranchBinderRemap {
                    start: branch_remap.map(|remap| remap.start).unwrap_or(0),
                    count: constructor.argument_count,
                }),
            )?,
        });
    }
    Ok(RuntimeExpr::Match {
        scrutinee,
        cases,
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: format!("no runtime match case selected for {}", view.family_symbol),
        },
    })
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

fn captures_outer_variable(term: &CheckedCoreBodyTerm) -> bool {
    has_free_variable_at_or_above(term, 1)
}

fn has_free_variable_at_or_above(term: &CheckedCoreBodyTerm, bound: usize) -> bool {
    match term {
        CheckedCoreBodyTerm::Variable { de_bruijn_index } => *de_bruijn_index >= bound,
        CheckedCoreBodyTerm::DirectDeclarationCall { .. } => false,
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
            partiality: match &meta.partiality {
                PartialityMetadata::Total => RuntimePartiality::Total,
                PartialityMetadata::CheckedPartial { obligation } => {
                    RuntimePartiality::CheckedTrap {
                        obligation: obligation.to_string(),
                    }
                }
                PartialityMetadata::TrustedPartial { assumption } => {
                    RuntimePartiality::TrustedTrap {
                        assumption: assumption.to_string(),
                    }
                }
            },
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
    if meta.boundary == EffectBoundary::Foreign || meta.foreign_symbol.is_some() {
        return Err(ErasureError::UnsupportedErasure {
            symbol: symbol.clone(),
            reason: "runtime erasure does not assign foreign boundary runtime meaning".to_string(),
        });
    }
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
