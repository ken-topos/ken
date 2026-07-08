use ken_elaborator::checked_core::{
    emit_checked_core_package, representative_checked_core_fixtures, AssumptionTrustKind,
    AssumptionTrustMetadata, CheckedCorePackage, LowerabilityStatus, ObligationMetadata,
    ObligationStatus, PartialityMetadata, RecordSigmaKind, RecordSigmaMetadata, StableSymbol,
    SymbolNamespace,
};
use ken_elaborator::erasure::{
    emit_proof_erasure_boundary_witness, emit_proof_erasure_boundary_witness_for_targets,
    erase_checked_core_package_for_target, ErasureError,
};
use ken_runtime::{
    validate_proof_erasure_boundary_witness, ProofErasureBoundaryWitnessStage,
    ProofErasureBoundaryWitnessTier, RuntimeAssumptionTrustKind, RuntimeDeclarationKind,
    RuntimeEffectBoundary, RuntimeFieldStatus, RuntimeLowerabilityStatus, RuntimeObligationStatus,
    RuntimePartiality,
};

fn fixture_package() -> CheckedCorePackage {
    representative_checked_core_fixtures()
        .expect("fixture emits")
        .into_iter()
        .next()
        .expect("representative fixture exists")
        .package
}

fn symbol(display: &str) -> StableSymbol {
    let package = fixture_package();
    package
        .artifact
        .semantic
        .symbols
        .into_iter()
        .find(|symbol| symbol.to_string() == display)
        .unwrap_or_else(|| panic!("missing fixture symbol {display}"))
}

fn reemit(mut package: CheckedCorePackage) -> CheckedCorePackage {
    package.header.dependency_semantic_hashes =
        package.artifact.semantic.dependency_semantic_hashes.clone();
    emit_checked_core_package(package.header, package.artifact).expect("package re-emits")
}

fn proof_erasure_record_package() -> (CheckedCorePackage, StableSymbol, StableSymbol, StableSymbol)
{
    let mut package = fixture_package();
    let target = StableSymbol::declaration("fixture", &["Proof"], "ErasureRecord");
    let obligation = StableSymbol::obligation("erasure-record.obligation");
    let assumption = StableSymbol::assumption(&target, "trusted-helper");
    let non_target_record = StableSymbol::declaration("fixture", &["Proof"], "NonTargetRecord");
    let non_target_unsupported = StableSymbol::new(
        SymbolNamespace::Unsupported,
        ["fixture".to_string(), "NonTarget".to_string()],
    );

    package.artifact.semantic.symbols.extend([
        target.clone(),
        non_target_record.clone(),
        obligation.clone(),
        assumption.clone(),
        non_target_unsupported.clone(),
    ]);
    package
        .artifact
        .semantic
        .declarations
        .insert(target.clone(), b"checked-decl:erasure-record".to_vec());
    package.artifact.semantic.declarations.insert(
        non_target_record.clone(),
        b"checked-decl:non-target-record".to_vec(),
    );
    package.artifact.semantic.record_sigma_metadata.insert(
        target.clone(),
        RecordSigmaMetadata {
            kind: RecordSigmaKind::Record,
            fields: vec![
                ken_elaborator::checked_core::FieldMetadata {
                    name: "runtime_payload".to_string(),
                    ty: target.clone(),
                    runtime: ken_elaborator::checked_core::RuntimeFieldStatus::Runtime,
                },
                ken_elaborator::checked_core::FieldMetadata {
                    name: "law_payload".to_string(),
                    ty: target.clone(),
                    runtime: ken_elaborator::checked_core::RuntimeFieldStatus::ErasedLaw,
                },
                ken_elaborator::checked_core::FieldMetadata {
                    name: "proof_payload".to_string(),
                    ty: target.clone(),
                    runtime: ken_elaborator::checked_core::RuntimeFieldStatus::ErasedProof,
                },
            ],
            lowerability: LowerabilityStatus::Supported,
        },
    );
    package.artifact.semantic.record_sigma_metadata.insert(
        non_target_record.clone(),
        RecordSigmaMetadata {
            kind: RecordSigmaKind::Record,
            fields: vec![ken_elaborator::checked_core::FieldMetadata {
                name: "non_target_proof".to_string(),
                ty: non_target_record.clone(),
                runtime: ken_elaborator::checked_core::RuntimeFieldStatus::ErasedProof,
            }],
            lowerability: LowerabilityStatus::Supported,
        },
    );
    package
        .artifact
        .semantic
        .lowerability
        .insert(target.clone(), LowerabilityStatus::Supported);
    package
        .artifact
        .semantic
        .lowerability
        .insert(non_target_record.clone(), LowerabilityStatus::Supported);
    package.artifact.semantic.obligations.insert(
        obligation.clone(),
        b"obligation survives the proof-erasure boundary".to_vec(),
    );
    package.artifact.semantic.obligation_metadata.insert(
        obligation.clone(),
        ObligationMetadata {
            status: ObligationStatus::Unknown,
            origin: target.clone(),
            affects_runtime_meaning: true,
        },
    );
    package.artifact.semantic.assumptions.insert(
        assumption.clone(),
        b"assumption survives the proof-erasure boundary".to_vec(),
    );
    package.artifact.semantic.assumption_trust_metadata.insert(
        assumption.clone(),
        AssumptionTrustMetadata {
            kind: AssumptionTrustKind::PrimitiveAssumption,
            target: target.clone(),
            affects_runtime_meaning: true,
        },
    );
    package.artifact.semantic.trusted_base_delta.insert(
        assumption.clone(),
        b"trusted-base delta survives the proof-erasure boundary".to_vec(),
    );
    package.artifact.semantic.lowerability.insert(
        non_target_unsupported.clone(),
        LowerabilityStatus::Unsupported {
            reason: "non-target unsupported lane remains auditable".to_string(),
        },
    );
    package.artifact.semantic.unsupported.insert(
        non_target_unsupported.clone(),
        b"non-target unsupported".to_vec(),
    );

    (reemit(package), target, obligation, assumption)
}

#[test]
fn erasure_consumes_package_only_and_preserves_metadata() {
    let mut package = fixture_package();
    let target = symbol("decl:fixture::Effects::print_line");
    let non_target_blocker = symbol("decl:fixture::Core::Bool");
    let obligation = StableSymbol::obligation("print_line.runtime.0");
    let assumption = StableSymbol::assumption(&target, "console-authority");

    package
        .artifact
        .semantic
        .symbols
        .extend([obligation.clone(), assumption.clone()]);
    package.artifact.semantic.obligations.insert(
        obligation.clone(),
        b"runtime obligation survives erasure".to_vec(),
    );
    package.artifact.semantic.obligation_metadata.insert(
        obligation.clone(),
        ObligationMetadata {
            status: ObligationStatus::Unknown,
            origin: target.clone(),
            affects_runtime_meaning: true,
        },
    );
    package.artifact.semantic.assumptions.insert(
        assumption.clone(),
        b"runtime assumption survives erasure".to_vec(),
    );
    package.artifact.semantic.assumption_trust_metadata.insert(
        assumption.clone(),
        AssumptionTrustMetadata {
            kind: AssumptionTrustKind::PrimitiveAssumption,
            target: target.clone(),
            affects_runtime_meaning: true,
        },
    );
    package.artifact.semantic.trusted_base_delta.insert(
        assumption.clone(),
        b"trusted base delta survives erasure".to_vec(),
    );
    package.artifact.semantic.lowerability.insert(
        non_target_blocker.clone(),
        LowerabilityStatus::Unsupported {
            reason: "non-target blocker remains auditable".to_string(),
        },
    );
    package.artifact.semantic.unsupported.insert(
        non_target_blocker.clone(),
        b"non-target unsupported".to_vec(),
    );
    package.artifact.source_identity.insert(
        "diagnostic-only".to_string(),
        "surface bytes ignored".to_string(),
    );
    let package = reemit(package);

    let program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");

    assert_eq!(program.core_semantic_hash, package.core_semantic_hash);
    assert_eq!(program.artifact_hash, package.artifact_hash);
    assert!(program
        .erased_core
        .metadata
        .obligations
        .contains_key(&obligation.to_string()));
    assert!(program
        .erased_core
        .metadata
        .assumptions
        .contains_key(&assumption.to_string()));
    assert!(program
        .erased_core
        .metadata
        .trusted_base_delta
        .contains_key(&assumption.to_string()));
    let obligation_audit = program
        .erased_core
        .metadata
        .obligation_metadata
        .get(&obligation.to_string())
        .expect("obligation metadata survives");
    assert_eq!(obligation_audit.status, RuntimeObligationStatus::Unknown);
    assert_eq!(obligation_audit.origin, target.to_string());
    assert!(obligation_audit.affects_runtime_meaning);

    let assumption_audit = program
        .erased_core
        .metadata
        .assumption_trust_metadata
        .get(&assumption.to_string())
        .expect("assumption/trust metadata survives");
    assert_eq!(
        assumption_audit.kind,
        RuntimeAssumptionTrustKind::PrimitiveAssumption
    );
    assert_eq!(assumption_audit.target, target.to_string());
    assert!(assumption_audit.affects_runtime_meaning);

    assert!(matches!(
        program
            .erased_core
            .metadata
            .lowerability
            .get(&target.to_string()),
        Some(RuntimeLowerabilityStatus::Supported)
    ));
    assert!(matches!(
        program
            .erased_core
            .metadata
            .lowerability
            .get(&non_target_blocker.to_string()),
        Some(RuntimeLowerabilityStatus::Unsupported { reason })
            if reason == "non-target blocker remains auditable"
    ));
    assert_eq!(
        program
            .erased_core
            .metadata
            .unsupported
            .get(&non_target_blocker.to_string()),
        Some(&b"non-target unsupported".to_vec())
    );
    let symbol_audit = &program.declarations[0].metadata;
    assert_eq!(
        symbol_audit
            .obligation_metadata
            .get(&obligation.to_string()),
        Some(obligation_audit)
    );
    assert_eq!(
        symbol_audit
            .assumption_trust_metadata
            .get(&assumption.to_string()),
        Some(assumption_audit)
    );
    assert!(matches!(
        symbol_audit.lowerability,
        Some(RuntimeLowerabilityStatus::Supported)
    ));

    let effect_audit = program
        .erased_core
        .metadata
        .checked_core
        .effects_foreign_metadata
        .get(&target.to_string())
        .expect("effects/foreign metadata lane survives");
    assert_eq!(effect_audit.boundary, RuntimeEffectBoundary::Effectful);
    assert!(effect_audit
        .capabilities
        .contains("meta:fixture::ConsoleCap"));
    assert!(
        !program.erased_core.metadata.effects.is_empty(),
        "effect metadata survives as IR metadata"
    );
}

#[test]
fn proof_erasure_boundary_witness_accepts_pair_derived_metadata() {
    let (package, target, obligation, assumption) = proof_erasure_record_package();
    let non_target_record = StableSymbol::declaration("fixture", &["Proof"], "NonTargetRecord");
    let program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");

    let witness = emit_proof_erasure_boundary_witness_for_targets(&package, [&target], &program)
        .expect("pair-derived witness emits");
    let report = validate_proof_erasure_boundary_witness(&program, &witness)
        .expect("runtime witness report validates");

    assert_eq!(
        report.tier,
        ProofErasureBoundaryWitnessTier::Nc9BoundedProofErasureBoundary
    );
    assert_eq!(
        report.artifact.package_identity,
        package.header.package_identity.to_string()
    );
    assert_eq!(
        report.artifact.core_semantic_hash,
        package.core_semantic_hash
    );
    assert_eq!(report.artifact.artifact_hash, package.artifact_hash);
    assert!(report
        .facts
        .runtime_declaration_targets
        .contains(&target.to_string()));
    assert!(!report
        .facts
        .runtime_declaration_targets
        .contains(&non_target_record.to_string()));
    let field_statuses = report
        .facts
        .record_field_statuses
        .get(&target.to_string())
        .expect("record field statuses present");
    assert_eq!(
        field_statuses
            .iter()
            .map(|field| (field.name.clone(), field.status.clone()))
            .collect::<Vec<_>>(),
        vec![
            ("runtime_payload".to_string(), RuntimeFieldStatus::Runtime),
            ("law_payload".to_string(), RuntimeFieldStatus::ErasedLaw),
            ("proof_payload".to_string(), RuntimeFieldStatus::ErasedProof),
        ]
    );
    assert_eq!(
        report
            .facts
            .checked_core_record_field_statuses
            .get(&target.to_string()),
        Some(field_statuses)
    );
    assert!(!report
        .facts
        .record_field_statuses
        .contains_key(&non_target_record.to_string()));
    assert!(report
        .facts
        .checked_core_record_field_statuses
        .contains_key(&non_target_record.to_string()));
    assert!(report
        .facts
        .obligation_metadata
        .contains_key(&obligation.to_string()));
    assert!(report
        .facts
        .assumption_trust_metadata
        .contains_key(&assumption.to_string()));
    assert!(report
        .facts
        .trusted_base_delta
        .contains_key(&assumption.to_string()));
    assert!(
        report
            .facts
            .unsupported
            .values()
            .any(|bytes| bytes == &b"non-target unsupported".to_vec()),
        "non-target unsupported lane survives in the witness"
    );
}

#[test]
fn proof_erasure_boundary_witness_rejects_missing_runtime_record_declaration() {
    let (package, target, _, _) = proof_erasure_record_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");
    program.declarations.clear();

    let err = emit_proof_erasure_boundary_witness_for_targets(&package, [&target], &program)
        .expect_err("missing runtime record declaration must reject");

    assert!(matches!(
        err,
        ErasureError::ProofErasureBoundaryWitness(witness)
            if witness.stage == ProofErasureBoundaryWitnessStage::WitnessMismatch
                && witness.lane == "record_field_statuses"
    ));
}

#[test]
fn proof_erasure_boundary_witness_rejects_missing_record_declaration_and_target_metadata() {
    let (package, target, _, _) = proof_erasure_record_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");
    program.declarations.clear();
    program
        .erased_core
        .metadata
        .runtime_declaration_targets
        .clear();

    let err = emit_proof_erasure_boundary_witness_for_targets(&package, [&target], &program)
        .expect_err("missing runtime record declaration and target metadata must reject");

    assert!(matches!(
        err,
        ErasureError::ProofErasureBoundaryWitness(witness)
            if witness.stage == ProofErasureBoundaryWitnessStage::WitnessMismatch
                && witness.lane == "runtime_declaration_targets"
    ));
}

#[test]
fn proof_erasure_boundary_pair_only_emitter_rejects_ambiguous_record_targets() {
    let (package, target, _, _) = proof_erasure_record_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");
    program.declarations.clear();
    program
        .erased_core
        .metadata
        .runtime_declaration_targets
        .clear();

    let err = emit_proof_erasure_boundary_witness(&package, &program)
        .expect_err("pair-only witness emission must fail closed without record target evidence");

    assert!(matches!(
        err,
        ErasureError::ProofErasureBoundaryWitness(witness)
            if witness.stage == ProofErasureBoundaryWitnessStage::WitnessMismatch
                && witness.lane == "runtime_declaration_targets"
    ));
}

#[test]
fn proof_erasure_boundary_witness_rejects_stale_identity() {
    let (package, target, _, _) = proof_erasure_record_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");
    program.artifact_hash += 1;

    let err = emit_proof_erasure_boundary_witness_for_targets(&package, [&target], &program)
        .expect_err("stale identity must reject before witness success");

    assert!(matches!(
        err,
        ErasureError::ProofErasureBoundaryWitness(witness)
            if witness.stage == ProofErasureBoundaryWitnessStage::WitnessIdentity
                && witness.lane == "artifact_identity"
    ));
}

#[test]
fn proof_erasure_boundary_witness_names_dropped_metadata_lanes() {
    for lane in [
        "obligation_metadata",
        "assumption_trust_metadata",
        "lowerability",
        "unsupported",
    ] {
        let (package, target, _, _) = proof_erasure_record_package();
        let mut program =
            erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");
        match lane {
            "obligation_metadata" => program.erased_core.metadata.obligation_metadata.clear(),
            "assumption_trust_metadata" => {
                program
                    .erased_core
                    .metadata
                    .assumption_trust_metadata
                    .clear();
            }
            "lowerability" => {
                program
                    .erased_core
                    .metadata
                    .lowerability
                    .remove(&target.to_string());
            }
            "unsupported" => program.erased_core.metadata.unsupported.clear(),
            _ => unreachable!("test lanes are exhaustive"),
        }

        let err =
            match emit_proof_erasure_boundary_witness_for_targets(&package, [&target], &program) {
                Ok(_) => panic!("{lane} drop must reject"),
                Err(err) => err,
            };

        assert!(matches!(
            err,
            ErasureError::ProofErasureBoundaryWitness(witness)
                if witness.stage == ProofErasureBoundaryWitnessStage::WitnessMismatch
                    && witness.lane == lane
        ));
    }
}

#[test]
fn proof_erasure_boundary_witness_rejects_field_status_drift() {
    let (package, target, _, _) = proof_erasure_record_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");
    let RuntimeDeclarationKind::Record { fields } = &mut program.declarations[0].kind else {
        panic!("fixture target lowers to record");
    };
    fields[1].status = RuntimeFieldStatus::Runtime;

    let err = emit_proof_erasure_boundary_witness_for_targets(&package, [&target], &program)
        .expect_err("field status drift must reject");

    assert!(matches!(
        err,
        ErasureError::ProofErasureBoundaryWitness(witness)
            if witness.stage == ProofErasureBoundaryWitnessStage::WitnessMismatch
                && witness.lane == "record_field_statuses"
    ));
}

#[test]
fn proof_erasure_boundary_witness_rejects_checked_core_field_status_drift() {
    let (package, target, _, _) = proof_erasure_record_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");
    let metadata = program
        .erased_core
        .metadata
        .checked_core
        .record_sigma_metadata
        .get_mut(&target.to_string())
        .expect("checked-core record metadata present");
    metadata.fields[2].runtime = RuntimeFieldStatus::Runtime;

    let err = emit_proof_erasure_boundary_witness_for_targets(&package, [&target], &program)
        .expect_err("checked-core metadata field status drift must reject");

    assert!(matches!(
        err,
        ErasureError::ProofErasureBoundaryWitness(witness)
            if witness.stage == ProofErasureBoundaryWitnessStage::WitnessMismatch
                && witness.lane == "checked_core_record_field_statuses"
    ));
}

#[test]
fn proof_erasure_boundary_witness_report_rejects_witness_program_mismatch() {
    let (package, target, _, _) = proof_erasure_record_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");
    let witness = emit_proof_erasure_boundary_witness_for_targets(&package, [&target], &program)
        .expect("pair-derived witness emits");
    program.erased_core.metadata.obligations.clear();

    let err = validate_proof_erasure_boundary_witness(&program, &witness)
        .expect_err("witness/program mismatch must reject");

    assert_eq!(err.stage, ProofErasureBoundaryWitnessStage::WitnessMismatch);
    assert_eq!(err.lane, "obligations");
}

#[test]
fn reachable_lowerability_blocker_rejects_before_runtime_ir() {
    let mut package = fixture_package();
    let target = symbol("decl:fixture::Core::Bool");
    package.artifact.semantic.lowerability.insert(
        target.clone(),
        LowerabilityStatus::Unsupported {
            reason: "test blocker".to_string(),
        },
    );
    package
        .artifact
        .semantic
        .unsupported
        .insert(target.clone(), b"test blocker".to_vec());
    let package = reemit(package);

    let err = erase_checked_core_package_for_target(&package, [&target])
        .expect_err("reachable unsupported target must reject");

    assert!(matches!(
        err,
        ErasureError::InvalidPackage(
            ken_elaborator::checked_core::CheckedCorePackageError::LoweringReadiness(_)
        )
    ));
}

#[test]
fn nested_constructor_lowerability_blocker_rejects() {
    let mut package = fixture_package();
    let target = symbol("decl:fixture::Core::Bool");
    let true_ctor = symbol("ctor:fixture::Core::Bool::True");
    let bool_meta = package
        .artifact
        .semantic
        .data_metadata
        .get_mut(&target)
        .expect("Bool data metadata exists");
    let ctor = bool_meta
        .constructors
        .iter_mut()
        .find(|ctor| ctor.symbol == true_ctor)
        .expect("True constructor metadata exists");
    ctor.lowerability = LowerabilityStatus::Unsupported {
        reason: "constructor blocker".to_string(),
    };
    let package = reemit(package);

    let err = erase_checked_core_package_for_target(&package, [&target])
        .expect_err("nested constructor blocker must reject");

    assert!(matches!(
        err,
        ErasureError::UnsupportedErasure { symbol, reason }
            if symbol == true_ctor && reason.contains("constructor blocker")
    ));
}

#[test]
fn checked_partial_primitive_lowers_to_explicit_trap_face() {
    let mut package = fixture_package();
    let target = symbol("prim:nat_add");
    let obligation = StableSymbol::obligation("nat_add.bounds");
    package.artifact.semantic.symbols.insert(obligation.clone());
    package.artifact.semantic.obligations.insert(
        obligation.clone(),
        b"partial primitive check obligation".to_vec(),
    );
    package.artifact.semantic.obligation_metadata.insert(
        obligation.clone(),
        ObligationMetadata {
            status: ObligationStatus::Tested,
            origin: target.clone(),
            affects_runtime_meaning: true,
        },
    );
    package
        .artifact
        .semantic
        .primitive_metadata
        .get_mut(&target)
        .expect("primitive metadata exists")
        .partiality = PartialityMetadata::CheckedPartial {
        obligation: obligation.clone(),
    };
    let package = reemit(package);

    let program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");

    let RuntimeDeclarationKind::Primitive { op } = &program.declarations[0].kind else {
        panic!("target lowers to primitive declaration");
    };
    assert!(matches!(
        &op.partiality,
        RuntimePartiality::CheckedTrap { obligation: got }
            if got == &obligation.to_string()
    ));
}

#[test]
fn foreign_boundary_survives_as_metadata_without_ffi_semantics() {
    let mut package = fixture_package();
    let target = symbol("decl:fixture::Effects::print_line");
    let meta = package
        .artifact
        .semantic
        .effects_foreign_metadata
        .get_mut(&target)
        .expect("effect metadata exists");
    meta.boundary = ken_elaborator::checked_core::EffectBoundary::Foreign;
    meta.foreign_symbol = Some("host.console.print_line".to_string());
    let package = reemit(package);

    let program =
        erase_checked_core_package_for_target(&package, [&target]).expect("erasure succeeds");

    let RuntimeDeclarationKind::EffectBoundary { effects } = &program.declarations[0].kind else {
        panic!("foreign target lowers to an explicit effect boundary");
    };
    assert!(effects.contains("Console"));
    let effect_audit = program
        .erased_core
        .metadata
        .checked_core
        .effects_foreign_metadata
        .get(&target.to_string())
        .expect("foreign metadata survives");
    assert_eq!(effect_audit.boundary, RuntimeEffectBoundary::Foreign);
    assert_eq!(
        effect_audit.foreign_symbol.as_deref(),
        Some("host.console.print_line")
    );
}

#[test]
fn symbol_without_runtime_metadata_rejects_loudly() {
    let mut package = fixture_package();
    let target = StableSymbol::new(
        SymbolNamespace::Declaration,
        [
            "fixture".to_string(),
            "Missing".to_string(),
            "body".to_string(),
        ],
    );
    package.artifact.semantic.symbols.insert(target.clone());
    package
        .artifact
        .semantic
        .lowerability
        .insert(target.clone(), LowerabilityStatus::Supported);
    let package = reemit(package);

    let err = erase_checked_core_package_for_target(&package, [&target])
        .expect_err("metadata gap must reject");

    assert!(matches!(
        err,
        ErasureError::MissingRuntimeMetadata { symbol, section }
            if symbol == target && section == "runtime-lowerable metadata"
    ));
}
