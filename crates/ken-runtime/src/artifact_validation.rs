//! NC8 artifact-local runtime certificate checker.
//!
//! This checker validates only facts recomputed from the concrete
//! `RuntimeProgram`. It does not certify Cranelift, native execution, object
//! layout, or whole-compiler correctness.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{
    RuntimeDeclarationKind, RuntimeEffectBoundary, RuntimeExpr, RuntimeLowerabilityStatus,
    RuntimeObligationMetadata, RuntimePartiality, RuntimeProgram, RuntimeSymbol, RuntimeValue,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeArtifactCertificate {
    pub package_identity: Option<String>,
    pub core_semantic_hash: Option<u64>,
    pub artifact_hash: Option<u64>,
    pub claim: Option<RuntimeArtifactValidationClaim>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeArtifactValidationClaim {
    pub validator: RuntimeArtifactValidator,
    pub facts: Option<RuntimeArtifactClaimFacts>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeArtifactValidator {
    Nc8SupportedRuntimeArtifactV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeArtifactClaimFacts {
    pub no_package_effects: bool,
    pub no_package_capabilities: bool,
    pub no_package_runtime_checks: bool,
    pub no_package_trust_metadata: bool,
    pub no_reachable_unsupported_entries: bool,
    pub all_reachable_lowerability_supported: bool,
    pub no_declaration_effects_or_capabilities: bool,
    pub no_declaration_trust_metadata: bool,
    pub no_foreign_or_effectful_boundaries: bool,
    pub declaration_count: Option<usize>,
    pub example_count: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeArtifactIdentity {
    pub package_identity: String,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeArtifactValidationFacts {
    pub no_package_effects: bool,
    pub no_package_capabilities: bool,
    pub no_package_runtime_checks: bool,
    pub no_package_trust_metadata: bool,
    pub no_reachable_unsupported_entries: bool,
    pub all_reachable_lowerability_supported: bool,
    pub no_declaration_effects_or_capabilities: bool,
    pub no_declaration_trust_metadata: bool,
    pub no_foreign_or_effectful_boundaries: bool,
    pub declaration_count: usize,
    pub example_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeArtifactValidationReport {
    pub tier: RuntimeArtifactValidationTier,
    pub artifact: RuntimeArtifactIdentity,
    pub validator: RuntimeArtifactValidator,
    pub evidence_source: String,
    pub facts: RuntimeArtifactValidationFacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeArtifactValidationTier {
    F2BoundedRuntimeArtifactValidation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofErasureBoundaryWitness {
    pub artifact: RuntimeArtifactIdentity,
    pub facts: ProofErasureBoundaryFacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofErasureBoundaryFacts {
    pub runtime_declaration_targets: BTreeSet<RuntimeSymbol>,
    pub record_field_statuses: BTreeMap<RuntimeSymbol, Vec<ProofErasureFieldStatus>>,
    pub checked_core_record_field_statuses: BTreeMap<RuntimeSymbol, Vec<ProofErasureFieldStatus>>,
    pub lowerability: BTreeMap<RuntimeSymbol, RuntimeLowerabilityStatus>,
    pub unsupported: BTreeMap<RuntimeSymbol, Vec<u8>>,
    pub obligations: BTreeMap<RuntimeSymbol, Vec<u8>>,
    pub obligation_metadata: BTreeMap<RuntimeSymbol, RuntimeObligationMetadata>,
    pub assumptions: BTreeMap<RuntimeSymbol, Vec<u8>>,
    pub assumption_trust_metadata: BTreeMap<RuntimeSymbol, crate::RuntimeAssumptionTrustMetadata>,
    pub trusted_base_delta: BTreeMap<RuntimeSymbol, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofErasureFieldStatus {
    pub name: String,
    pub status: crate::RuntimeFieldStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofErasureBoundaryWitnessReport {
    pub tier: ProofErasureBoundaryWitnessTier,
    pub artifact: RuntimeArtifactIdentity,
    pub validator: ProofErasureBoundaryWitnessValidator,
    pub evidence_source: String,
    pub facts: ProofErasureBoundaryFacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KenCheckedProofErasureBoundaryReport {
    pub tier: ProofErasureBoundaryWitnessTier,
    pub artifact: RuntimeArtifactIdentity,
    pub checker: KenProofErasureBoundaryChecker,
    pub evidence_source: String,
    pub helper_assumptions: Vec<String>,
    pub facts: ProofErasureBoundaryFacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofErasureBoundaryWitnessTier {
    Nc9BoundedProofErasureBoundary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KenProofErasureBoundaryChecker {
    Nc9KenLaneVerdictCheckerV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofErasureBoundaryWitnessValidator {
    Nc9ProofErasureBoundaryWitnessV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofErasureBoundaryWitnessError {
    pub stage: ProofErasureBoundaryWitnessStage,
    pub lane: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofErasureBoundaryWitnessStage {
    WitnessIdentity,
    WitnessMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeArtifactValidationError {
    pub stage: RuntimeArtifactValidationStage,
    pub fact: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeArtifactValidationStage {
    MalformedCertificate,
    ArtifactIdentity,
    ClaimRecompute,
    ClaimMismatch,
}

impl RuntimeArtifactCertificate {
    pub fn supported_runtime_artifact_for(program: &RuntimeProgram) -> Self {
        Self {
            package_identity: Some(program.package_identity.clone()),
            core_semantic_hash: Some(program.core_semantic_hash),
            artifact_hash: Some(program.artifact_hash),
            claim: Some(RuntimeArtifactValidationClaim {
                validator: RuntimeArtifactValidator::Nc8SupportedRuntimeArtifactV1,
                facts: Some(RuntimeArtifactClaimFacts {
                    no_package_effects: true,
                    no_package_capabilities: true,
                    no_package_runtime_checks: true,
                    no_package_trust_metadata: true,
                    no_reachable_unsupported_entries: true,
                    all_reachable_lowerability_supported: true,
                    no_declaration_effects_or_capabilities: true,
                    no_declaration_trust_metadata: true,
                    no_foreign_or_effectful_boundaries: true,
                    declaration_count: Some(program.declarations.len()),
                    example_count: Some(program.examples.len()),
                }),
            }),
        }
    }
}

impl RuntimeArtifactIdentity {
    pub fn from_program(program: &RuntimeProgram) -> Self {
        Self {
            package_identity: program.package_identity.clone(),
            core_semantic_hash: program.core_semantic_hash,
            artifact_hash: program.artifact_hash,
        }
    }
}

impl fmt::Display for RuntimeArtifactValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{}: {}", self.stage, self.fact, self.reason)
    }
}

impl std::error::Error for RuntimeArtifactValidationError {}

impl fmt::Display for ProofErasureBoundaryWitnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{}: {}", self.stage, self.lane, self.reason)
    }
}

impl std::error::Error for ProofErasureBoundaryWitnessError {}

pub fn validate_supported_runtime_artifact_certificate(
    program: &RuntimeProgram,
    certificate: &RuntimeArtifactCertificate,
) -> Result<RuntimeArtifactValidationReport, RuntimeArtifactValidationError> {
    let certificate_identity = certificate_identity(certificate)?;
    let artifact_identity = RuntimeArtifactIdentity::from_program(program);
    if certificate_identity != artifact_identity {
        return Err(validation_error(
            RuntimeArtifactValidationStage::ArtifactIdentity,
            "runtime_artifact_identity",
            format!(
                "certificate identity {:?} does not match RuntimeProgram identity {:?}",
                certificate_identity, artifact_identity
            ),
        ));
    }

    let claim = certificate.claim.as_ref().ok_or_else(|| {
        validation_error(
            RuntimeArtifactValidationStage::MalformedCertificate,
            "claim",
            "certificate is missing the validation claim",
        )
    })?;
    if claim.validator != RuntimeArtifactValidator::Nc8SupportedRuntimeArtifactV1 {
        return Err(validation_error(
            RuntimeArtifactValidationStage::MalformedCertificate,
            "validator",
            "certificate names an unsupported validator",
        ));
    }
    let claimed = claim.facts.as_ref().ok_or_else(|| {
        validation_error(
            RuntimeArtifactValidationStage::MalformedCertificate,
            "facts",
            "certificate is missing claimed validation facts",
        )
    })?;
    require_claimed_true(claimed.no_package_effects, "no_package_effects")?;
    require_claimed_true(claimed.no_package_capabilities, "no_package_capabilities")?;
    require_claimed_true(
        claimed.no_package_runtime_checks,
        "no_package_runtime_checks",
    )?;
    require_claimed_true(
        claimed.no_package_trust_metadata,
        "no_package_trust_metadata",
    )?;
    require_claimed_true(
        claimed.no_reachable_unsupported_entries,
        "no_reachable_unsupported_entries",
    )?;
    require_claimed_true(
        claimed.all_reachable_lowerability_supported,
        "all_reachable_lowerability_supported",
    )?;
    require_claimed_true(
        claimed.no_declaration_effects_or_capabilities,
        "no_declaration_effects_or_capabilities",
    )?;
    require_claimed_true(
        claimed.no_declaration_trust_metadata,
        "no_declaration_trust_metadata",
    )?;
    require_claimed_true(
        claimed.no_foreign_or_effectful_boundaries,
        "no_foreign_or_effectful_boundaries",
    )?;

    let facts = recompute_supported_runtime_artifact_facts(program)?;
    let declaration_count = claimed.declaration_count.ok_or_else(|| {
        validation_error(
            RuntimeArtifactValidationStage::MalformedCertificate,
            "declaration_count",
            "certificate is missing the claimed declaration count",
        )
    })?;
    if declaration_count != facts.declaration_count {
        return Err(validation_error(
            RuntimeArtifactValidationStage::ClaimMismatch,
            "declaration_count",
            format!(
                "certificate claims {declaration_count} declarations but RuntimeProgram has {}",
                facts.declaration_count
            ),
        ));
    }
    let example_count = claimed.example_count.ok_or_else(|| {
        validation_error(
            RuntimeArtifactValidationStage::MalformedCertificate,
            "example_count",
            "certificate is missing the claimed example count",
        )
    })?;
    if example_count != facts.example_count {
        return Err(validation_error(
            RuntimeArtifactValidationStage::ClaimMismatch,
            "example_count",
            format!(
                "certificate claims {example_count} examples but RuntimeProgram has {}",
                facts.example_count
            ),
        ));
    }

    Ok(RuntimeArtifactValidationReport {
        tier: RuntimeArtifactValidationTier::F2BoundedRuntimeArtifactValidation,
        artifact: artifact_identity,
        validator: RuntimeArtifactValidator::Nc8SupportedRuntimeArtifactV1,
        evidence_source:
            "ken-runtime NC8 checker recomputed supported-subset facts from RuntimeProgram"
                .to_string(),
        facts,
    })
}

pub fn proof_erasure_boundary_facts_from_program(
    program: &RuntimeProgram,
) -> ProofErasureBoundaryFacts {
    let metadata = &program.erased_core.metadata;
    ProofErasureBoundaryFacts {
        runtime_declaration_targets: metadata.runtime_declaration_targets.clone(),
        record_field_statuses: program_declaration_record_field_statuses(program),
        checked_core_record_field_statuses: program_checked_core_record_field_statuses(program),
        lowerability: metadata.lowerability.clone(),
        unsupported: metadata.unsupported.clone(),
        obligations: metadata.obligations.clone(),
        obligation_metadata: metadata.obligation_metadata.clone(),
        assumptions: metadata.assumptions.clone(),
        assumption_trust_metadata: metadata.assumption_trust_metadata.clone(),
        trusted_base_delta: metadata.trusted_base_delta.clone(),
    }
}

pub fn validate_proof_erasure_boundary_witness(
    program: &RuntimeProgram,
    witness: &ProofErasureBoundaryWitness,
) -> Result<ProofErasureBoundaryWitnessReport, ProofErasureBoundaryWitnessError> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if witness.artifact != artifact {
        return Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessIdentity,
            "artifact_identity",
            format!(
                "witness identity {:?} does not match RuntimeProgram identity {:?}",
                witness.artifact, artifact
            ),
        ));
    }

    let recomputed = proof_erasure_boundary_facts_from_program(program);
    require_witness_lane_match(
        &witness.facts.runtime_declaration_targets,
        &recomputed.runtime_declaration_targets,
        "runtime_declaration_targets",
    )?;
    require_witness_lane_match(
        &witness.facts.record_field_statuses,
        &recomputed.record_field_statuses,
        "record_field_statuses",
    )?;
    require_witness_lane_match(
        &witness.facts.checked_core_record_field_statuses,
        &recomputed.checked_core_record_field_statuses,
        "checked_core_record_field_statuses",
    )?;
    require_witness_lane_match(
        &witness.facts.lowerability,
        &recomputed.lowerability,
        "lowerability",
    )?;
    require_witness_lane_match(
        &witness.facts.unsupported,
        &recomputed.unsupported,
        "unsupported",
    )?;
    require_witness_lane_match(
        &witness.facts.obligations,
        &recomputed.obligations,
        "obligations",
    )?;
    require_witness_lane_match(
        &witness.facts.obligation_metadata,
        &recomputed.obligation_metadata,
        "obligation_metadata",
    )?;
    require_witness_lane_match(
        &witness.facts.assumptions,
        &recomputed.assumptions,
        "assumptions",
    )?;
    require_witness_lane_match(
        &witness.facts.assumption_trust_metadata,
        &recomputed.assumption_trust_metadata,
        "assumption_trust_metadata",
    )?;
    require_witness_lane_match(
        &witness.facts.trusted_base_delta,
        &recomputed.trusted_base_delta,
        "trusted_base_delta",
    )?;

    Ok(ProofErasureBoundaryWitnessReport {
        tier: ProofErasureBoundaryWitnessTier::Nc9BoundedProofErasureBoundary,
        artifact,
        validator: ProofErasureBoundaryWitnessValidator::Nc9ProofErasureBoundaryWitnessV1,
        evidence_source:
            "ken-runtime NC9 witness checker recomputed proof-erasure boundary facts from RuntimeProgram"
                .to_string(),
        facts: recomputed,
    })
}

pub fn recompute_supported_runtime_artifact_facts(
    program: &RuntimeProgram,
) -> Result<RuntimeArtifactValidationFacts, RuntimeArtifactValidationError> {
    let metadata = &program.erased_core.metadata;
    if !metadata.effects.is_empty() {
        return Err(claim_recompute_error(
            "no_package_effects",
            "package carries effect metadata outside the NC8 supported subset",
        ));
    }
    if !metadata.capabilities.is_empty() {
        return Err(claim_recompute_error(
            "no_package_capabilities",
            "package carries capability metadata outside the NC8 supported subset",
        ));
    }
    if !metadata.runtime_checks.is_empty() {
        return Err(claim_recompute_error(
            "no_package_runtime_checks",
            "package carries runtime-check metadata outside the NC8 supported subset",
        ));
    }
    if !metadata.assumptions.is_empty()
        || !metadata.assumption_trust_metadata.is_empty()
        || !metadata.trusted_base_delta.is_empty()
    {
        return Err(claim_recompute_error(
            "no_package_trust_metadata",
            "package carries trust metadata outside the NC8 supported subset",
        ));
    }

    let reachable: BTreeSet<_> = program
        .declarations
        .iter()
        .map(|declaration| declaration.symbol.clone())
        .collect();
    for declaration in &program.declarations {
        if declaration.metadata.unsupported.is_some()
            || metadata.unsupported.contains_key(&declaration.symbol)
        {
            return Err(claim_recompute_error(
                "no_reachable_unsupported_entries",
                format!("reachable unsupported entry {}", declaration.symbol),
            ));
        }

        let lowerability = declaration
            .metadata
            .lowerability
            .as_ref()
            .or_else(|| metadata.lowerability.get(&declaration.symbol))
            .ok_or_else(|| {
                claim_recompute_error(
                    "all_reachable_lowerability_supported",
                    format!(
                        "{} is missing runtime lowerability metadata",
                        declaration.symbol
                    ),
                )
            })?;
        require_supported_lowerability(
            lowerability,
            "all_reachable_lowerability_supported",
            &declaration.symbol,
        )?;

        if !declaration.metadata.effects.is_empty()
            || !declaration.metadata.capabilities.is_empty()
            || !declaration.metadata.runtime_checks.is_empty()
        {
            return Err(claim_recompute_error(
                "no_declaration_effects_or_capabilities",
                format!(
                    "{} carries effect/capability/runtime-check metadata",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.assumptions.is_empty()
            || !declaration.metadata.assumption_trust_metadata.is_empty()
            || !declaration.metadata.trusted_base_delta.is_empty()
        {
            return Err(claim_recompute_error(
                "no_declaration_trust_metadata",
                format!("{} carries declaration trust metadata", declaration.symbol),
            ));
        }

        validate_declaration_shape(&declaration.kind, &declaration.symbol)?;
        validate_checked_core_metadata(program, &declaration.symbol)?;
    }
    for example in &program.examples {
        validate_runtime_expr(&example.ir, "runtime_example")?;
    }

    Ok(RuntimeArtifactValidationFacts {
        no_package_effects: true,
        no_package_capabilities: true,
        no_package_runtime_checks: true,
        no_package_trust_metadata: true,
        no_reachable_unsupported_entries: true,
        all_reachable_lowerability_supported: true,
        no_declaration_effects_or_capabilities: true,
        no_declaration_trust_metadata: true,
        no_foreign_or_effectful_boundaries: true,
        declaration_count: reachable.len(),
        example_count: program.examples.len(),
    })
}

fn certificate_identity(
    certificate: &RuntimeArtifactCertificate,
) -> Result<RuntimeArtifactIdentity, RuntimeArtifactValidationError> {
    let package_identity = certificate.package_identity.clone().ok_or_else(|| {
        validation_error(
            RuntimeArtifactValidationStage::MalformedCertificate,
            "package_identity",
            "certificate is missing package_identity",
        )
    })?;
    let core_semantic_hash = certificate.core_semantic_hash.ok_or_else(|| {
        validation_error(
            RuntimeArtifactValidationStage::MalformedCertificate,
            "core_semantic_hash",
            "certificate is missing core_semantic_hash",
        )
    })?;
    let artifact_hash = certificate.artifact_hash.ok_or_else(|| {
        validation_error(
            RuntimeArtifactValidationStage::MalformedCertificate,
            "artifact_hash",
            "certificate is missing artifact_hash",
        )
    })?;
    Ok(RuntimeArtifactIdentity {
        package_identity,
        core_semantic_hash,
        artifact_hash,
    })
}

fn require_claimed_true(
    claimed: bool,
    fact: &'static str,
) -> Result<(), RuntimeArtifactValidationError> {
    if claimed {
        Ok(())
    } else {
        Err(validation_error(
            RuntimeArtifactValidationStage::MalformedCertificate,
            fact,
            "certificate does not claim this required supported-subset fact",
        ))
    }
}

fn validate_declaration_shape(
    kind: &RuntimeDeclarationKind,
    symbol: &RuntimeSymbol,
) -> Result<(), RuntimeArtifactValidationError> {
    match kind {
        RuntimeDeclarationKind::Transparent { body } => validate_runtime_expr(body, symbol),
        RuntimeDeclarationKind::Primitive { op } => {
            if matches!(op.partiality, RuntimePartiality::TrustedTrap { .. }) {
                return Err(claim_recompute_error(
                    "no_declaration_trust_metadata",
                    format!("{symbol} uses trusted partial primitive trap metadata"),
                ));
            }
            Ok(())
        }
        RuntimeDeclarationKind::EffectBoundary { effects } => {
            if effects.is_empty() {
                Ok(())
            } else {
                Err(claim_recompute_error(
                    "no_foreign_or_effectful_boundaries",
                    format!("{symbol} declares effect boundary metadata"),
                ))
            }
        }
        RuntimeDeclarationKind::Data { .. }
        | RuntimeDeclarationKind::Record { .. }
        | RuntimeDeclarationKind::RecursiveGroup { .. }
        | RuntimeDeclarationKind::MetadataOnly => Ok(()),
    }
}

fn validate_checked_core_metadata(
    program: &RuntimeProgram,
    symbol: &RuntimeSymbol,
) -> Result<(), RuntimeArtifactValidationError> {
    let checked = &program.erased_core.metadata.checked_core;
    if let Some(meta) = checked.primitive_metadata.get(symbol) {
        require_supported_lowerability(
            &meta.lowerability,
            "all_reachable_lowerability_supported",
            symbol,
        )?;
        if matches!(
            meta.partiality,
            crate::RuntimePartialityMetadata::TrustedPartial { .. }
        ) {
            return Err(claim_recompute_error(
                "no_declaration_trust_metadata",
                format!("{symbol} has trusted partiality metadata"),
            ));
        }
    }
    if let Some(meta) = checked.data_metadata.get(symbol) {
        require_supported_lowerability(
            &meta.lowerability,
            "all_reachable_lowerability_supported",
            symbol,
        )?;
        require_supported_lowerability(
            &meta.eliminator,
            "all_reachable_lowerability_supported",
            symbol,
        )?;
        for constructor in &meta.constructors {
            require_supported_lowerability(
                &constructor.lowerability,
                "all_reachable_lowerability_supported",
                &constructor.symbol,
            )?;
        }
    }
    if let Some(meta) = checked.record_sigma_metadata.get(symbol) {
        require_supported_lowerability(
            &meta.lowerability,
            "all_reachable_lowerability_supported",
            symbol,
        )?;
    }
    if let Some(meta) = checked.class_instance_metadata.get(symbol) {
        require_supported_lowerability(
            &meta.lowerability,
            "all_reachable_lowerability_supported",
            symbol,
        )?;
    }
    if let Some(meta) = checked.recursion_metadata.get(symbol) {
        require_supported_lowerability(
            &meta.lowerability,
            "all_reachable_lowerability_supported",
            symbol,
        )?;
    }
    if let Some(meta) = checked.effects_foreign_metadata.get(symbol) {
        require_supported_lowerability(
            &meta.lowerability,
            "all_reachable_lowerability_supported",
            symbol,
        )?;
        if meta.boundary != RuntimeEffectBoundary::Pure
            || meta.foreign_symbol.is_some()
            || !meta.declared_effects.is_empty()
            || !meta.capabilities.is_empty()
            || !meta.runtime_checks.is_empty()
        {
            return Err(claim_recompute_error(
                "no_foreign_or_effectful_boundaries",
                format!("{symbol} carries effects/foreign metadata"),
            ));
        }
    }
    Ok(())
}

fn validate_runtime_expr(
    expr: &RuntimeExpr,
    fact_subject: &str,
) -> Result<(), RuntimeArtifactValidationError> {
    match expr {
        RuntimeExpr::Value(value) => validate_runtime_value(value, fact_subject),
        RuntimeExpr::Var(_) => Err(unsupported_runtime_expr_error("Var", fact_subject)),
        RuntimeExpr::Let { .. } => Err(unsupported_runtime_expr_error("Let", fact_subject)),
        RuntimeExpr::If { .. } => Err(unsupported_runtime_expr_error("If", fact_subject)),
        RuntimeExpr::PrimitiveCall { primitive, args } => {
            match &primitive.partiality {
                RuntimePartiality::Total
                | RuntimePartiality::SafeOption { .. }
                | RuntimePartiality::SafeResult { .. } => {
                    validate_total_primitive_call(&primitive.symbol, args, fact_subject)?;
                }
                RuntimePartiality::CheckedTrap { .. } => {}
                RuntimePartiality::TrustedTrap { .. } => {
                    return Err(claim_recompute_error(
                        "no_declaration_trust_metadata",
                        format!("{fact_subject} uses trusted partial primitive trap metadata"),
                    ));
                }
            }
            for arg in args {
                validate_runtime_expr(arg, fact_subject)?;
            }
            Ok(())
        }
        RuntimeExpr::Construct { .. } => {
            Err(unsupported_runtime_expr_error("Construct", fact_subject))
        }
        RuntimeExpr::Match { .. } => Err(unsupported_runtime_expr_error("Match", fact_subject)),
        RuntimeExpr::Record { .. } => Err(unsupported_runtime_expr_error("Record", fact_subject)),
        RuntimeExpr::Project { .. } => Err(unsupported_runtime_expr_error("Project", fact_subject)),
        RuntimeExpr::Closure { .. } => Err(unsupported_runtime_expr_error("Closure", fact_subject)),
        RuntimeExpr::DeclarationRef { .. } => Err(unsupported_runtime_expr_error(
            "DeclarationRef",
            fact_subject,
        )),
        RuntimeExpr::ImportedDeclarationRef { .. } => Err(unsupported_runtime_expr_error(
            "ImportedDeclarationRef",
            fact_subject,
        )),
        RuntimeExpr::Call { .. } => Err(unsupported_runtime_expr_error("Call", fact_subject)),
        RuntimeExpr::Effect { .. } => Err(claim_recompute_error(
            "no_foreign_or_effectful_boundaries",
            format!("{fact_subject} contains an effectful runtime expression"),
        )),
        RuntimeExpr::Trap(_) => Ok(()),
    }
}

fn unsupported_runtime_expr_error(
    construct: &'static str,
    fact_subject: &str,
) -> RuntimeArtifactValidationError {
    claim_recompute_error(
        "all_runtime_expressions_supported",
        format!(
            "{fact_subject} contains {construct}, which is outside the NC6 seed-example subset"
        ),
    )
}

fn validate_total_primitive_call(
    symbol: &str,
    args: &[RuntimeExpr],
    fact_subject: &str,
) -> Result<(), RuntimeArtifactValidationError> {
    if symbol != "add_int" {
        return Err(claim_recompute_error(
            "all_runtime_primitives_supported",
            format!("{fact_subject} uses total primitive {symbol}, outside the NC6 supported set"),
        ));
    }
    if args.len() != 2 {
        return Err(claim_recompute_error(
            "all_runtime_primitives_supported",
            format!(
                "{fact_subject} uses add_int with arity {}, expected 2",
                args.len()
            ),
        ));
    }
    for arg in args {
        if !matches!(arg, RuntimeExpr::Value(RuntimeValue::Int(_))) {
            return Err(claim_recompute_error(
                "all_runtime_primitives_supported",
                format!("{fact_subject} uses add_int with a non-literal-Int operand shape"),
            ));
        }
    }
    Ok(())
}

fn validate_runtime_value(
    value: &RuntimeValue,
    fact_subject: &str,
) -> Result<(), RuntimeArtifactValidationError> {
    match value {
        RuntimeValue::Constructor { args, .. } => {
            for arg in args {
                validate_runtime_value(arg, fact_subject)?;
            }
            Ok(())
        }
        RuntimeValue::Record { fields } => {
            for (_, field) in fields {
                validate_runtime_value(field, fact_subject)?;
            }
            Ok(())
        }
        RuntimeValue::ClosureRef { .. } => Err(claim_recompute_error(
            "no_foreign_or_effectful_boundaries",
            format!("{fact_subject} contains a closure reference value"),
        )),
        RuntimeValue::Unknown => Err(claim_recompute_error(
            "all_runtime_values_supported",
            format!("{fact_subject} contains unknown runtime data"),
        )),
        RuntimeValue::Bool(_)
        | RuntimeValue::Int(_)
        | RuntimeValue::Bytes(_)
        | RuntimeValue::String(_) => Ok(()),
    }
}

fn program_declaration_record_field_statuses(
    program: &RuntimeProgram,
) -> BTreeMap<RuntimeSymbol, Vec<ProofErasureFieldStatus>> {
    let mut statuses = BTreeMap::new();
    for declaration in &program.declarations {
        collect_declaration_field_statuses(&mut statuses, &declaration.symbol, &declaration.kind);
    }
    statuses
}

fn program_checked_core_record_field_statuses(
    program: &RuntimeProgram,
) -> BTreeMap<RuntimeSymbol, Vec<ProofErasureFieldStatus>> {
    program
        .erased_core
        .metadata
        .checked_core
        .record_sigma_metadata
        .iter()
        .map(|(symbol, meta)| {
            (
                symbol.clone(),
                meta.fields
                    .iter()
                    .map(|field| ProofErasureFieldStatus {
                        name: field.name.clone(),
                        status: field.runtime.clone(),
                    })
                    .collect(),
            )
        })
        .collect()
}

fn collect_declaration_field_statuses(
    statuses: &mut BTreeMap<RuntimeSymbol, Vec<ProofErasureFieldStatus>>,
    symbol: &RuntimeSymbol,
    kind: &RuntimeDeclarationKind,
) {
    if let RuntimeDeclarationKind::Record { fields } = kind {
        statuses.insert(
            symbol.clone(),
            fields
                .iter()
                .map(|field| ProofErasureFieldStatus {
                    name: field.name.clone(),
                    status: field.status.clone(),
                })
                .collect(),
        );
    }
}

fn require_witness_lane_match<T: PartialEq + fmt::Debug>(
    witness: &T,
    recomputed: &T,
    lane: &'static str,
) -> Result<(), ProofErasureBoundaryWitnessError> {
    if witness == recomputed {
        Ok(())
    } else {
        Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessMismatch,
            lane,
            format!(
                "witness lane does not match RuntimeProgram recomputation: witness={witness:?}, recomputed={recomputed:?}"
            ),
        ))
    }
}

pub fn proof_erasure_witness_error(
    stage: ProofErasureBoundaryWitnessStage,
    lane: &'static str,
    reason: impl Into<String>,
) -> ProofErasureBoundaryWitnessError {
    ProofErasureBoundaryWitnessError {
        stage,
        lane,
        reason: reason.into(),
    }
}

fn require_supported_lowerability(
    status: &RuntimeLowerabilityStatus,
    fact: &'static str,
    symbol: &RuntimeSymbol,
) -> Result<(), RuntimeArtifactValidationError> {
    if matches!(status, RuntimeLowerabilityStatus::Supported) {
        Ok(())
    } else {
        Err(claim_recompute_error(
            fact,
            format!("{symbol} has blocking lowerability metadata: {status:?}"),
        ))
    }
}

fn claim_recompute_error(
    fact: &'static str,
    reason: impl Into<String>,
) -> RuntimeArtifactValidationError {
    validation_error(RuntimeArtifactValidationStage::ClaimRecompute, fact, reason)
}

fn validation_error(
    stage: RuntimeArtifactValidationStage,
    fact: &'static str,
    reason: impl Into<String>,
) -> RuntimeArtifactValidationError {
    RuntimeArtifactValidationError {
        stage,
        fact,
        reason: reason.into(),
    }
}
