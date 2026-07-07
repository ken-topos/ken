//! NC8 artifact-local runtime certificate checker.
//!
//! This checker validates only facts recomputed from the concrete
//! `RuntimeProgram`. It does not certify Cranelift, native execution, object
//! layout, or whole-compiler correctness.

use std::collections::BTreeSet;
use std::fmt;

use crate::{
    RuntimeDeclarationKind, RuntimeEffectBoundary, RuntimeExpr, RuntimeLowerabilityStatus,
    RuntimePartiality, RuntimeProgram, RuntimeSymbol, RuntimeValue,
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
        RuntimeExpr::Var(_) => Ok(()),
        RuntimeExpr::Let { value, body } => {
            validate_runtime_expr(value, fact_subject)?;
            validate_runtime_expr(body, fact_subject)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            validate_runtime_expr(scrutinee, fact_subject)?;
            validate_runtime_expr(then_expr, fact_subject)?;
            validate_runtime_expr(else_expr, fact_subject)
        }
        RuntimeExpr::PrimitiveCall { primitive, args } => {
            if matches!(primitive.partiality, RuntimePartiality::TrustedTrap { .. }) {
                return Err(claim_recompute_error(
                    "no_declaration_trust_metadata",
                    format!("{fact_subject} uses trusted partial primitive trap metadata"),
                ));
            }
            for arg in args {
                validate_runtime_expr(arg, fact_subject)?;
            }
            Ok(())
        }
        RuntimeExpr::Construct { args, .. } => {
            for arg in args {
                validate_runtime_expr(arg, fact_subject)?;
            }
            Ok(())
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            validate_runtime_expr(scrutinee, fact_subject)?;
            for case in cases {
                validate_runtime_expr(&case.body, fact_subject)?;
            }
            Ok(())
        }
        RuntimeExpr::Record { fields } => {
            for (_, field) in fields {
                validate_runtime_expr(field, fact_subject)?;
            }
            Ok(())
        }
        RuntimeExpr::Project { record, .. } => validate_runtime_expr(record, fact_subject),
        RuntimeExpr::Closure { body, .. } => validate_runtime_expr(body, fact_subject),
        RuntimeExpr::Call { callee, args } => {
            validate_runtime_expr(callee, fact_subject)?;
            for arg in args {
                validate_runtime_expr(arg, fact_subject)?;
            }
            Ok(())
        }
        RuntimeExpr::Effect { .. } => Err(claim_recompute_error(
            "no_foreign_or_effectful_boundaries",
            format!("{fact_subject} contains an effectful runtime expression"),
        )),
        RuntimeExpr::Trap(_) => Ok(()),
    }
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
