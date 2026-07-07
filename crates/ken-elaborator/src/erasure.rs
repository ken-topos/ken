//! NC5 erasure boundary from `CheckedCorePackage v0` to Ken runtime IR.
//!
//! This module consumes only the checked-core package artifact. Source identity
//! may remain in the package envelope for diagnostics and provenance, but it is
//! never an input to runtime meaning here.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ken_runtime::{
    nc5_seed_examples, ErasedExecutableCore, RuntimeConstructor, RuntimeDeclaration,
    RuntimeDeclarationKind, RuntimeField, RuntimeFieldStatus, RuntimeMetadata, RuntimePartiality,
    RuntimePrimitive, RuntimeProgram, RuntimeSymbolMetadata,
};

use crate::checked_core::{
    consume_checked_core_package_for_target, validate_checked_core_package, CheckedCorePackage,
    CheckedCorePackageError, ClassInstanceKind, ClassInstanceMetadata, DataMetadata,
    EffectBoundary, EffectsForeignMetadata, LowerabilityStatus, PartialityMetadata,
    PrimitiveMetadata, RecordSigmaMetadata, RecursionMetadata, StableSymbol,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErasureError {
    InvalidPackage(CheckedCorePackageError),
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
        assumptions: symbol_bytes_map(&semantic.assumptions),
        trusted_base_delta: symbol_bytes_map(&semantic.trusted_base_delta),
        dependency_semantic_hashes: semantic
            .dependency_semantic_hashes
            .iter()
            .map(|(symbol, hash)| (symbol.to_string(), hash.clone()))
            .collect(),
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
        assumptions: semantic
            .assumption_trust_metadata
            .iter()
            .filter_map(|(assumption, meta)| {
                (meta.target == *symbol).then(|| assumption.to_string())
            })
            .collect(),
        trusted_base_delta: semantic
            .trusted_base_delta
            .keys()
            .filter(|trust| *trust == symbol)
            .map(ToString::to_string)
            .collect(),
        runtime_checks: runtime_checks_for_targets(package, &[symbol.clone()]),
        capabilities: capabilities_for_targets(package, &[symbol.clone()]),
        effects: effects_for_targets(package, &[symbol.clone()]),
    }
}

fn symbol_bytes_map(map: &BTreeMap<StableSymbol, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    map.iter()
        .map(|(symbol, bytes)| (symbol.to_string(), bytes.clone()))
        .collect()
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
