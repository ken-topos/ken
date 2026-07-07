use ken_elaborator::checked_core::{
    emit_checked_core_package, representative_checked_core_fixtures, AssumptionTrustKind,
    AssumptionTrustMetadata, CheckedCorePackage, LowerabilityStatus, ObligationMetadata,
    ObligationStatus, PartialityMetadata, StableSymbol, SymbolNamespace,
};
use ken_elaborator::erasure::{erase_checked_core_package_for_target, ErasureError};
use ken_runtime::{RuntimeDeclarationKind, RuntimePartiality};

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

#[test]
fn erasure_consumes_package_only_and_preserves_metadata() {
    let mut package = fixture_package();
    let target = symbol("decl:fixture::Effects::print_line");
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
    assert!(
        !program.erased_core.metadata.effects.is_empty(),
        "effect metadata survives as IR metadata"
    );
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
fn foreign_boundary_rejects_without_backend_or_ffi_semantics() {
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

    let err = erase_checked_core_package_for_target(&package, [&target])
        .expect_err("foreign boundary must reject in NC5");

    assert!(matches!(
        err,
        ErasureError::UnsupportedErasure { symbol, .. } if symbol == target
    ));
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
