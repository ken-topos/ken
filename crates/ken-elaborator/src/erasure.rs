//! NC5 erasure boundary from `CheckedCorePackage v0` to Ken runtime IR.
//!
//! This module consumes only the checked-core package artifact. Source identity
//! may remain in the package envelope for diagnostics and provenance, but it is
//! never an input to runtime meaning here.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ken_runtime::*;

use crate::checked_core::{
    self, consume_checked_core_package_for_target, validate_checked_core_package,
    CheckedCorePackage, CheckedCorePackageError, ClassInstanceKind, ClassInstanceMetadata,
    DataMetadata, EffectBoundary, EffectsForeignMetadata, LowerabilityStatus, PartialityMetadata,
    PrimitiveMetadata, RecordSigmaMetadata, RecursionMetadata, StableSymbol,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErasureError {
    InvalidPackage(CheckedCorePackageError),
    ProofErasureBoundaryWitness(ProofErasureBoundaryWitnessError),
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
        declarations.push(lower_symbol(package, target)?);
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

    let package_facts = proof_erasure_boundary_facts_from_package(package, program);
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
        return Err(ErasureError::UnsupportedErasure {
            symbol: symbol.clone(),
            reason: "checked declaration body lowering is not in NC5 seed".to_string(),
        });
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
            reason: "NC5 does not assign foreign boundary runtime meaning".to_string(),
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
    program: &RuntimeProgram,
) -> ProofErasureBoundaryFacts {
    let semantic = &package.artifact.semantic;
    ProofErasureBoundaryFacts {
        runtime_declaration_targets: program
            .erased_core
            .metadata
            .runtime_declaration_targets
            .clone(),
        record_field_statuses: package_declaration_record_field_statuses(package, program),
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
    program: &RuntimeProgram,
) -> BTreeMap<String, Vec<ProofErasureFieldStatus>> {
    let package_records = package_record_field_statuses(package);
    program
        .erased_core
        .metadata
        .runtime_declaration_targets
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
