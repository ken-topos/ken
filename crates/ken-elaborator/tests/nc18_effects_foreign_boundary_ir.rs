use std::collections::BTreeSet;

use ken_elaborator::checked_core::{
    emit_checked_core_package, representative_checked_core_fixtures, CheckedCorePackage,
    EffectBoundary, LowerabilityStatus, ObligationMetadata, ObligationStatus, StableSymbol,
};
use ken_elaborator::erasure::erase_checked_core_package_for_target;
use ken_runtime::{
    compare_runtime_ir_with_interpreter_observation, evaluate_runtime_ir_example,
    summarize_runtime_ir_program, RuntimeDeclarationKind, RuntimeEffectBoundary,
    RuntimeGroundValue, RuntimeInterpreterObservation, RuntimeIrDifferentialStage,
    RuntimeIrDifferentialVerdict, RuntimeIrNativePhaseGate, RuntimeIrSeedEnvironment,
    RuntimeIrTargetIdentity, RuntimeObservation, RuntimeTrap, RuntimeTrapCode,
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

fn nc18_effect_package() -> (CheckedCorePackage, StableSymbol, StableSymbol) {
    let mut package = fixture_package();
    let target = symbol("decl:fixture::Effects::print_line");
    let runtime_check = StableSymbol::obligation("print_line.runtime.authority");
    package
        .artifact
        .semantic
        .symbols
        .insert(runtime_check.clone());
    package.artifact.semantic.obligations.insert(
        runtime_check.clone(),
        b"console authority is checked before host-effect execution".to_vec(),
    );
    package.artifact.semantic.obligation_metadata.insert(
        runtime_check.clone(),
        ObligationMetadata {
            status: ObligationStatus::Unknown,
            origin: target.clone(),
            affects_runtime_meaning: true,
        },
    );
    let meta = package
        .artifact
        .semantic
        .effects_foreign_metadata
        .get_mut(&target)
        .expect("effect metadata exists");
    meta.declared_effects = BTreeSet::from(["Console".to_string()]);
    meta.boundary = EffectBoundary::Effectful;
    meta.foreign_symbol = None;
    meta.runtime_checks = BTreeSet::from([runtime_check.clone()]);
    meta.lowerability = LowerabilityStatus::Supported;
    (reemit(package), target, runtime_check)
}

fn unavailable_effect_example() -> ken_runtime::RuntimeExample {
    ken_runtime::RuntimeExample {
        name: "nc18-console-effect".to_string(),
        checked_core_shape: "print_line \"hello\"".to_string(),
        ir: ken_runtime::RuntimeExpr::Effect {
            effect: "Console".to_string(),
            capability: Some("meta:fixture::ConsoleCap".to_string()),
            args: vec![],
        },
        observation: RuntimeObservation::Trapped(RuntimeTrap {
            code: RuntimeTrapCode::UnsupportedErasure,
            message: "host-effect execution unavailable in NC18 runtime IR".to_string(),
        }),
    }
}

fn oracle_for(
    program: &ken_runtime::RuntimeProgram,
    example: &ken_runtime::RuntimeExample,
) -> RuntimeInterpreterObservation {
    RuntimeInterpreterObservation {
        artifact: ken_runtime::RuntimeArtifactIdentity::from_program(program),
        target: RuntimeIrTargetIdentity::from_example(example),
        observation: example.observation.clone(),
        evidence_source:
            "caller-supplied NC18 interpreter observation for unavailable effect fixture"
                .to_string(),
    }
}

#[test]
fn represented_effect_target_survives_and_reports_comparison_unavailable() {
    let (package, target, runtime_check) = nc18_effect_package();
    let mut program = erase_checked_core_package_for_target(&package, [&target])
        .expect("effect-bearing target lowers to explicit metadata");
    let example = unavailable_effect_example();
    program.examples = vec![example.clone()];

    let RuntimeDeclarationKind::EffectBoundary { effects } = &program.declarations[0].kind else {
        panic!("effect target lowers to an effect-boundary declaration");
    };
    assert_eq!(effects, &BTreeSet::from(["Console".to_string()]));
    assert!(program.erased_core.metadata.effects.contains("Console"));
    assert!(program.declarations[0]
        .metadata
        .capabilities
        .contains("meta:fixture::ConsoleCap"));
    assert!(program.declarations[0]
        .metadata
        .runtime_checks
        .contains(&runtime_check.to_string()));

    let audit = program
        .erased_core
        .metadata
        .checked_core
        .effects_foreign_metadata
        .get(&target.to_string())
        .expect("checked-core effect metadata survives");
    assert_eq!(audit.boundary, RuntimeEffectBoundary::Effectful);
    assert!(audit.runtime_checks.contains(&runtime_check.to_string()));

    let report = summarize_runtime_ir_program(&program);
    assert_eq!(
        report
            .comparison_unavailable_targets
            .get(&target.to_string())
            .map(String::as_str),
        Some(
            "effect/capability/runtime-check facts are represented, but host-effect execution is unavailable"
        )
    );
    assert!(matches!(
        report.native_phase_gate,
        RuntimeIrNativePhaseGate::Blocked { .. }
    ));

    let diff = compare_runtime_ir_with_interpreter_observation(
        &program,
        &example,
        &RuntimeIrSeedEnvironment::empty(),
        oracle_for(&program, &example),
    );
    assert!(diff.runtime_ir.is_none());
    assert!(matches!(
        diff.verdict,
        RuntimeIrDifferentialVerdict::Unsupported {
            stage: RuntimeIrDifferentialStage::BoundaryPreflight,
            construct: "RuntimeProgram",
            ref reason,
        } if reason.contains("effect metadata")
    ));
}

#[test]
fn foreign_boundary_facts_survive_without_becoming_ffi_execution() {
    let (mut package, target, _) = nc18_effect_package();
    let meta = package
        .artifact
        .semantic
        .effects_foreign_metadata
        .get_mut(&target)
        .expect("effect metadata exists");
    meta.boundary = EffectBoundary::Foreign;
    meta.foreign_symbol = Some("host.console.print_line".to_string());
    let mut program = erase_checked_core_package_for_target(&reemit(package), [&target])
        .expect("foreign boundary lowers to explicit runtime metadata");
    let example = unavailable_effect_example();
    program.examples = vec![example.clone()];

    let audit = program
        .erased_core
        .metadata
        .checked_core
        .effects_foreign_metadata
        .get(&target.to_string())
        .expect("foreign audit metadata survives");
    assert_eq!(audit.boundary, RuntimeEffectBoundary::Foreign);
    assert_eq!(
        audit.foreign_symbol.as_deref(),
        Some("host.console.print_line")
    );

    let report = summarize_runtime_ir_program(&program);
    assert_eq!(
        report
            .comparison_unavailable_targets
            .get(&target.to_string())
            .map(String::as_str),
        Some("foreign-boundary facts are represented, but host FFI execution is unavailable")
    );

    let diff = compare_runtime_ir_with_interpreter_observation(
        &program,
        &example,
        &RuntimeIrSeedEnvironment::empty(),
        oracle_for(&program, &example),
    );
    assert!(matches!(
        diff.verdict,
        RuntimeIrDifferentialVerdict::Unsupported {
            stage: RuntimeIrDifferentialStage::BoundaryPreflight,
            ..
        }
    ));
}

#[test]
fn hidden_foreign_fact_rejects_before_runtime_success() {
    let (mut package, target, _) = nc18_effect_package();
    let meta = package
        .artifact
        .semantic
        .effects_foreign_metadata
        .get_mut(&target)
        .expect("effect metadata exists");
    meta.boundary = EffectBoundary::Foreign;
    meta.foreign_symbol = Some("host.console.print_line".to_string());
    meta.declared_effects.clear();
    meta.capabilities.clear();
    meta.runtime_checks.clear();
    let mut program = erase_checked_core_package_for_target(&reemit(package), [&target])
        .expect("foreign metadata lowers");
    let example = unavailable_effect_example();
    program.examples = vec![example.clone()];

    let err = evaluate_runtime_ir_example(&program, &example, &RuntimeIrSeedEnvironment::empty())
        .expect_err("hidden foreign facts must not evaluate successfully");

    assert_eq!(err.construct, "RuntimeProgram");
    assert!(err.reason.contains("effects/foreign metadata"));
}

#[test]
fn missing_runtime_check_fact_rejects_before_runtime_success() {
    let (package, target, runtime_check) = nc18_effect_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("effect metadata lowers");
    let example = unavailable_effect_example();
    program.examples = vec![example.clone()];
    program.declarations[0]
        .metadata
        .runtime_checks
        .remove(&runtime_check.to_string());

    let report = summarize_runtime_ir_program(&program);
    assert!(report
        .comparison_unavailable_targets
        .get(&target.to_string())
        .is_none());
    assert!(matches!(
        report
            .unsupported_targets
            .get(&target.to_string())
            .map(String::as_str),
        Some(reason)
            if reason.contains("runtime_checks")
                && reason.contains("stale or missing")
    ));
    assert!(matches!(
        report.native_phase_gate,
        RuntimeIrNativePhaseGate::Blocked { blockers }
            if blockers.iter().any(|reason| reason.contains("runtime_checks"))
    ));

    let err = evaluate_runtime_ir_example(&program, &example, &RuntimeIrSeedEnvironment::empty())
        .expect_err("missing runtime-check authority must reject before success");

    assert_eq!(err.construct, "RuntimeProgram");
    assert!(err.reason.contains("runtime_checks"));
    assert!(err.reason.contains("stale or missing"));
}

#[test]
fn stale_capability_identity_rejects_before_runtime_success() {
    let (package, target, _) = nc18_effect_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("effect metadata lowers");
    let example = unavailable_effect_example();
    program.examples = vec![example.clone()];
    program.declarations[0].metadata.capabilities =
        BTreeSet::from(["meta:fixture::OtherCap".to_string()]);

    let report = summarize_runtime_ir_program(&program);
    assert!(report
        .comparison_unavailable_targets
        .get(&target.to_string())
        .is_none());
    assert!(matches!(
        report
            .unsupported_targets
            .get(&target.to_string())
            .map(String::as_str),
        Some(reason)
            if reason.contains("capabilities")
                && reason.contains("stale or missing")
    ));
    assert!(matches!(
        report.native_phase_gate,
        RuntimeIrNativePhaseGate::Blocked { blockers }
            if blockers.iter().any(|reason| reason.contains("capabilities"))
    ));

    let err = evaluate_runtime_ir_example(&program, &example, &RuntimeIrSeedEnvironment::empty())
        .expect_err("stale capability authority must reject before success");

    assert_eq!(err.construct, "RuntimeProgram");
    assert!(err.reason.contains("capabilities"));
    assert!(err.reason.contains("stale or missing"));
}

#[test]
fn stale_interpreter_identity_still_rejects_before_unavailable_effect_report() {
    let (package, target, _) = nc18_effect_package();
    let mut program =
        erase_checked_core_package_for_target(&package, [&target]).expect("effect metadata lowers");
    let example = unavailable_effect_example();
    program.examples = vec![example.clone()];
    let mut oracle = oracle_for(&program, &example);
    oracle.artifact.artifact_hash += 1;

    let diff = compare_runtime_ir_with_interpreter_observation(
        &program,
        &example,
        &RuntimeIrSeedEnvironment::empty(),
        oracle,
    );

    assert!(diff.runtime_ir.is_none());
    assert!(matches!(
        diff.verdict,
        RuntimeIrDifferentialVerdict::Unsupported {
            stage: RuntimeIrDifferentialStage::BoundaryPreflight,
            construct: "RuntimeInterpreterObservation",
            ..
        }
    ));
}

#[test]
fn pure_program_report_keeps_starter_subset_ready() {
    let package = fixture_package();
    let target = symbol("decl:fixture::Core::Bool");
    let program =
        erase_checked_core_package_for_target(&package, [&target]).expect("pure target lowers");

    let report = summarize_runtime_ir_program(&program);

    assert!(report
        .supported_runtime_targets
        .contains(&target.to_string()));
    assert!(report.comparison_unavailable_targets.is_empty());
    assert!(matches!(
        report.native_phase_gate,
        RuntimeIrNativePhaseGate::ReadyForStarterKenOnlyExecutableSubset
    ));
    assert!(matches!(
        program.examples[0].observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Int(5))
    ));
}
