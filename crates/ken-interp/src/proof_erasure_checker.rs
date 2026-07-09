use std::collections::BTreeSet;
use std::fmt;

use ken_elaborator::ElabEnv;
use ken_runtime::{
    proof_erasure_boundary_facts_from_program, validate_proof_erasure_boundary_witness,
    KenCheckedProofErasureBoundaryReport, KenProofErasureBoundaryChecker,
    ProofErasureBoundaryFacts, ProofErasureBoundaryWitness, ProofErasureBoundaryWitnessStage,
    RuntimeArtifactIdentity, RuntimeProgram,
};

use crate::{eval, EvalStore, EvalVal};

pub const NC9_PROOF_ERASURE_BOUNDARY_CHECKER_SOURCE: &str =
    include_str!("../../../catalog/packages/verify/proof_erasure_boundary_checker.ken");

const LANE_ORDER: [&str; 11] = [
    "artifact_identity",
    "runtime_declaration_targets",
    "record_field_statuses",
    "checked_core_record_field_statuses",
    "lowerability",
    "unsupported",
    "obligations",
    "obligation_metadata",
    "assumptions",
    "assumption_trust_metadata",
    "trusted_base_delta",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KenProofErasureBoundaryCheckError {
    pub stage: KenProofErasureBoundaryCheckStage,
    pub lane: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KenProofErasureBoundaryCheckStage {
    CheckerKernelCheck,
    CheckerEvaluation,
    WitnessIdentity,
    WitnessMismatch,
    RustKenAgreement,
}

impl fmt::Display for KenProofErasureBoundaryCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{}: {}", self.stage, self.lane, self.reason)
    }
}

impl std::error::Error for KenProofErasureBoundaryCheckError {}

pub fn ken_check_proof_erasure_boundary_witness(
    program: &RuntimeProgram,
    witness: &ProofErasureBoundaryWitness,
) -> Result<KenCheckedProofErasureBoundaryReport, KenProofErasureBoundaryCheckError> {
    let lane_verdicts = project_lane_verdicts(program, witness);
    let ken_accepts = evaluate_ken_checker(&lane_verdicts)?;
    let runtime_result = validate_proof_erasure_boundary_witness(program, witness);

    match (ken_accepts, runtime_result) {
        (true, Ok(report)) => Ok(KenCheckedProofErasureBoundaryReport {
            tier: report.tier,
            artifact: report.artifact,
            checker: KenProofErasureBoundaryChecker::Nc9KenLaneVerdictCheckerV1,
            evidence_source: concat!(
                "catalog/packages/verify/proof_erasure_boundary_checker.ken elaborated/kernel-checked ",
                "and evaluated by ken-interp over NC9 lane verdicts projected from ",
                "ken-runtime proof_erasure_boundary_facts_from_program"
            )
            .to_string(),
            helper_assumptions: vec![
                "Rust projects the approved ProofErasureBoundaryWitness and RuntimeProgram into per-lane equality verdicts; the Ken checker does not parse raw Rust maps".to_string(),
            ],
            facts: report.facts,
        }),
        (false, Err(err)) => Err(KenProofErasureBoundaryCheckError {
            stage: match err.stage {
                ProofErasureBoundaryWitnessStage::WitnessIdentity => {
                    KenProofErasureBoundaryCheckStage::WitnessIdentity
                }
                ProofErasureBoundaryWitnessStage::WitnessMismatch => {
                    KenProofErasureBoundaryCheckStage::WitnessMismatch
                }
            },
            lane: err.lane,
            reason: err.reason,
        }),
        (true, Err(err)) => Err(KenProofErasureBoundaryCheckError {
            stage: KenProofErasureBoundaryCheckStage::RustKenAgreement,
            lane: err.lane,
            reason: format!("Ken checker accepted but Rust witness validation rejected: {err}"),
        }),
        (false, Ok(_)) => Err(KenProofErasureBoundaryCheckError {
            stage: KenProofErasureBoundaryCheckStage::RustKenAgreement,
            lane: first_failed_lane(&lane_verdicts).unwrap_or("proof_erasure_boundary_checker"),
            reason: "Ken checker rejected but Rust witness validation accepted".to_string(),
        }),
    }
}

fn project_lane_verdicts(
    program: &RuntimeProgram,
    witness: &ProofErasureBoundaryWitness,
) -> Vec<(&'static str, bool)> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    let recomputed = proof_erasure_boundary_facts_from_program(program);
    vec![
        ("artifact_identity", witness.artifact == artifact),
        (
            "runtime_declaration_targets",
            witness.facts.runtime_declaration_targets == recomputed.runtime_declaration_targets,
        ),
        (
            "record_field_statuses",
            witness.facts.record_field_statuses == recomputed.record_field_statuses,
        ),
        (
            "checked_core_record_field_statuses",
            witness.facts.checked_core_record_field_statuses
                == recomputed.checked_core_record_field_statuses,
        ),
        (
            "lowerability",
            witness.facts.lowerability == recomputed.lowerability,
        ),
        (
            "unsupported",
            witness.facts.unsupported == recomputed.unsupported,
        ),
        (
            "obligations",
            witness.facts.obligations == recomputed.obligations,
        ),
        (
            "obligation_metadata",
            witness.facts.obligation_metadata == recomputed.obligation_metadata,
        ),
        (
            "assumptions",
            witness.facts.assumptions == recomputed.assumptions,
        ),
        (
            "assumption_trust_metadata",
            witness.facts.assumption_trust_metadata == recomputed.assumption_trust_metadata,
        ),
        (
            "trusted_base_delta",
            witness.facts.trusted_base_delta == recomputed.trusted_base_delta,
        ),
    ]
}

fn evaluate_ken_checker(
    lane_verdicts: &[(&'static str, bool)],
) -> Result<bool, KenProofErasureBoundaryCheckError> {
    let mut seen = BTreeSet::new();
    for (lane, _) in lane_verdicts {
        seen.insert(*lane);
    }
    for lane in LANE_ORDER {
        if !seen.contains(lane) {
            return Err(check_error(
                KenProofErasureBoundaryCheckStage::CheckerEvaluation,
                lane,
                "missing projected lane verdict",
            ));
        }
    }

    let mut env = ElabEnv::new().map_err(|err| {
        check_error(
            KenProofErasureBoundaryCheckStage::CheckerKernelCheck,
            "prelude",
            format!("failed to construct Ken elaboration environment: {err}"),
        )
    })?;
    let before_trust: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_file(NC9_PROOF_ERASURE_BOUNDARY_CHECKER_SOURCE)
        .map_err(|err| {
            check_error(
                KenProofErasureBoundaryCheckStage::CheckerKernelCheck,
                "ken_checker_source",
                format!("Ken checker failed to elaborate/kernel-check: {err}"),
            )
        })?;
    let after_trust: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    if before_trust != after_trust {
        return Err(check_error(
            KenProofErasureBoundaryCheckStage::CheckerKernelCheck,
            "trusted_base_delta",
            "Ken checker added trusted-base entries",
        ));
    }

    let args = LANE_ORDER
        .iter()
        .map(|lane| {
            let accepted = lane_verdicts
                .iter()
                .find_map(|(candidate, verdict)| (*candidate == *lane).then_some(*verdict))
                .expect("lane presence checked above");
            if accepted {
                "True"
            } else {
                "False"
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let result_id = env
        .elaborate_decl(&format!(
            "const nc9_checker_result : Bool = nc9_proof_erasure_boundary_check {args}"
        ))
        .map_err(|err| {
            check_error(
                KenProofErasureBoundaryCheckStage::CheckerEvaluation,
                "nc9_checker_result",
                format!("Ken checker invocation failed to elaborate/kernel-check: {err}"),
            )
        })?;
    let (_, body) = env.env.transparent_body(result_id).ok_or_else(|| {
        check_error(
            KenProofErasureBoundaryCheckStage::CheckerEvaluation,
            "nc9_checker_result",
            "Ken checker result did not elaborate to a transparent definition",
        )
    })?;
    let mut store = EvalStore::new();
    let value = eval(&[], &body, &env.env, &mut store);
    decode_ken_bool(&env, value)
}

fn decode_ken_bool(
    env: &ElabEnv,
    value: EvalVal,
) -> Result<bool, KenProofErasureBoundaryCheckError> {
    let true_id = *env.globals.get("True").ok_or_else(|| {
        check_error(
            KenProofErasureBoundaryCheckStage::CheckerEvaluation,
            "Bool",
            "Ken prelude has no True constructor",
        )
    })?;
    let false_id = *env.globals.get("False").ok_or_else(|| {
        check_error(
            KenProofErasureBoundaryCheckStage::CheckerEvaluation,
            "Bool",
            "Ken prelude has no False constructor",
        )
    })?;
    match value {
        EvalVal::Bool(value) => Ok(value),
        EvalVal::Ctor { id, .. } if id == true_id => Ok(true),
        EvalVal::Ctor { id, .. } if id == false_id => Ok(false),
        other => Err(check_error(
            KenProofErasureBoundaryCheckStage::CheckerEvaluation,
            "nc9_checker_result",
            format!("Ken checker returned non-Bool value {other:?}"),
        )),
    }
}

fn first_failed_lane(lane_verdicts: &[(&'static str, bool)]) -> Option<&'static str> {
    lane_verdicts
        .iter()
        .find_map(|(lane, accepted)| (!*accepted).then_some(*lane))
}

fn check_error(
    stage: KenProofErasureBoundaryCheckStage,
    lane: &'static str,
    reason: impl Into<String>,
) -> KenProofErasureBoundaryCheckError {
    KenProofErasureBoundaryCheckError {
        stage,
        lane,
        reason: reason.into(),
    }
}

#[allow(dead_code)]
fn _assert_facts_are_part_of_public_surface(_: &ProofErasureBoundaryFacts) {}
