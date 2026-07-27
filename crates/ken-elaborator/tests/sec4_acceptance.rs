//! Sec4 trust-model acceptance binding for
//! `conformance/security/trust-model/seed-trust-model.md` (AC1–AC4).
//!
//! Promise classes: the trust-base and authorship pairs are durable
//! invariants; the dependency-graph check is the structural Invariant TT.
//!
//! Prior-coverage census (rows are cited, not duplicated):
//! - A1/A3: `i8_clock_effect.rs::
//!   clock_package_is_structural_zero_trust_and_declares_no_ordering_law`.
//! - A2: `km_literal_trust_accounting.rs::
//!   foreign_axiom_and_open_obligation_trust_entries_still_count`.
//! - B1/B3: `km_literal_trust_accounting.rs::
//!   foreign_axiom_and_open_obligation_trust_entries_still_count`.
//! - B4: `km_literal_trust_accounting.rs::
//!   literal_classification_is_the_only_primitive_accounting_exclusion`.
//! - B2 had only adjacent direct-kernel coverage in
//!   `b1_acceptance.rs::proved_postcondition_projects_to_q`; the test below
//!   closes the seed's stronger real-elaborator-admission requirement.
//! - `k5_absurd_trusted_base.rs::{absurd_proof_position_counted_in_
//!   trusted_base_delta,absurd_motive_position_counted_in_trusted_base_delta,
//!   absurd_with_no_postulate_reference_has_empty_delta}` covers dependency
//!   walking, not any missing Sec4 enumerator row.
//! - C1–C3 and D1 were not covered and are bound below.
//!
//! Deliberate exclusions: AC5 is documentation-prose fidelity and remains
//! conformance-validator evidence under operator policy. The external,
//! published independent kernel-audit report is deferred by `64 §6` and is an
//! operator governance decision, not an executable row.
//!
//! Stale seed locators re-derived at this branch: `trusted_base` is
//! `env.rs:492`, `declare_postulate` is `check.rs:1126`, the prover's unknown
//! hole admission is `prover.rs:494`, and `is_prelude` is `env.rs:330`.
//!
//! C1/C2 operand correction (`CONF-SEC4-REFL-PAIR`): `obs.rs:113` reduces
//! closed equal registered literals to `Top` and unequal ones to `Bottom`.
//! Thus closed C2 cannot reach `Refl`, while closed C1 rejects before
//! conversion. The authorized abstract pair below binds index convertibility
//! at a genuinely Eq-shaped goal; distinct abstract binders are unprovable,
//! not a closed false proposition.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;
use std::process::Command;

use ken_elaborator::ElabEnv;
use ken_kernel::{check, whnf, Context, GlobalId, KernelError, Term};
use num_bigint::BigInt;
use serde_json::{json, Value};

fn trusted_set(env: &ken_kernel::GlobalEnv) -> BTreeSet<GlobalId> {
    env.trusted_base().into_iter().collect()
}

fn new_trusted_entries(
    before: &BTreeSet<GlobalId>,
    after: &BTreeSet<GlobalId>,
) -> BTreeSet<GlobalId> {
    after.difference(before).copied().collect()
}

/// B2, `[by construction]`, durable invariant.
///
/// MEASURED: an `Axiom` introduced through surface elaboration adds one opaque
/// hole, and a kernel-checked body followed by the production discharge removes
/// that same id from `trusted_base()`.
/// CLAIMED: a discharged hole empties its trust-base delta.
/// THE GAP: the before/after observations entail the claim only because both
/// transitions use the real elaborator and `upgrade_to_transparent`, rather
/// than a hand-inserted `GlobalId`.
#[test]
fn discharged_elaborated_hole_empties_trusted_base_delta() {
    let mut elab = ElabEnv::new().expect("base elaboration environment");
    let before = trusted_set(&elab.env);

    let definition = elab
        .elaborate_decl("const sec4_hole : Int = Axiom")
        .expect("surface Axiom must elaborate through the real admission path");
    let admitted = trusted_set(&elab.env);
    let added = new_trusted_entries(&before, &admitted);
    assert_eq!(
        added.len(),
        1,
        "one surface Axiom must add exactly one trusted hole: {added:?}"
    );
    let hole = *added.iter().next().expect("exactly one admitted hole");
    assert!(
        !admitted.contains(&definition),
        "the checked transparent definition must not itself be trusted"
    );

    let (_, hole_ty) = elab.env.const_type(hole).expect("hole has a type");
    let body = Term::IntLit(BigInt::from(0));
    check(&elab.env, &Context::new(), &body, &hole_ty)
        .expect("replacement body must be kernel-checked before discharge");
    assert!(
        elab.env.upgrade_to_transparent(hole, body),
        "the admitted opaque hole must accept the real discharge transition"
    );

    let discharged = trusted_set(&elab.env);
    assert_eq!(
        new_trusted_entries(&before, &discharged),
        BTreeSet::new(),
        "discharging the sole added hole must restore the baseline trust set"
    );
    assert!(
        !discharged.contains(&hole),
        "the exact admitted hole must leave trusted_base after discharge"
    );
}

fn check_refl_at_abstract_int_equality(
    elab: &ElabEnv,
    same_index: bool,
) -> Result<(), KernelError> {
    let int_ty = Term::const_(elab.numeric_env.int_id, vec![]);
    let mut context = Context::new();
    context.push(int_ty.clone());
    context.push(int_ty.clone());
    let left = Term::var(1);
    let right = if same_index {
        left.clone()
    } else {
        Term::var(0)
    };
    let proposition = Term::Eq(Box::new(int_ty), Box::new(left.clone()), Box::new(right));

    // This four-argument call is also C3's compile-time structural pin: the
    // public check API accepts only environment, context, term, and type.
    check(
        &elab.env,
        &context,
        &Term::Refl(Box::new(left)),
        &proposition,
    )
}

/// C1–C3, `[structural]`, durable invariant.
///
/// MEASURED: in one two-binder context, the same `Refl x` certificate is
/// accepted at `x = x` and rejected with `BadEliminator` at `x = y` by the
/// four-argument kernel API.
/// CLAIMED: certificate admission at a genuinely Eq-shaped goal depends on
/// index convertibility, not author provenance.
/// THE GAP: distinct binders are unprovable rather than a closed false
/// proposition; this deliberately does not retain the seed's truth-valued
/// framing.
#[test]
fn kernel_check_flips_on_abstract_index_convertibility_without_provenance() {
    let elab = ElabEnv::new().expect("numeric elaboration environment");

    check_refl_at_abstract_int_equality(&elab, true)
        .expect("Refl x must prove the genuinely Eq-shaped x = x goal");
    let error = check_refl_at_abstract_int_equality(&elab, false)
        .expect_err("the same Refl x shape must not prove x = y");
    assert!(
        matches!(error, KernelError::BadEliminator(_)),
        "distinct abstract indices must fail at conversion, got {error:?}"
    );
}

fn closed_int_equality(elab: &ElabEnv, left: i64, right: i64) -> Term {
    Term::Eq(
        Box::new(Term::const_(elab.numeric_env.int_id, vec![])),
        Box::new(Term::IntLit(BigInt::from(left))),
        Box::new(Term::IntLit(BigInt::from(right))),
    )
}

/// Honest control for the superseded C1/C2 operands.
///
/// MEASURED: the registered-literal reducer maps closed `0 = 0` to `Top` and
/// `0 = 1` to `Bottom`; offering `Refl` at the latter rejects with
/// `TypeMismatch` before the Eq conversion arm.
/// CLAIMED: the seed's closed pair does not measure AC3's conversion boundary.
/// THE GAP: this pins landed reducer behavior only; it is not counted as the
/// authorship-independence pair above.
#[test]
fn closed_literal_equalities_bypass_refl_via_top_and_bottom() {
    let elab = ElabEnv::new().expect("numeric elaboration environment");
    let context = Context::new();
    let equal = closed_int_equality(&elab, 0, 0);
    let unequal = closed_int_equality(&elab, 0, 1);

    assert_eq!(
        whnf(&elab.env, &context, &equal),
        Term::const_(elab.env.top_id(), vec![]),
        "closed equal literals must collapse to Top before Refl"
    );
    assert_eq!(
        whnf(&elab.env, &context, &unequal),
        Term::const_(elab.env.bottom_id(), vec![]),
        "closed unequal literals must collapse to Bottom before Refl"
    );

    let error = check(
        &elab.env,
        &context,
        &Term::Refl(Box::new(Term::IntLit(BigInt::from(0)))),
        &unequal,
    )
    .expect_err("Refl at a Bottom-collapsed goal must reject");
    assert!(
        matches!(error, KernelError::TypeMismatch { .. }),
        "closed unequal literals must reject before Eq conversion, got {error:?}"
    );
}

fn cargo_metadata() -> Value {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO"))
        .current_dir(repository)
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("cargo metadata must execute");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata emits JSON")
}

fn reachable_workspace_dependencies(metadata: &Value, root_name: &str) -> BTreeSet<String> {
    let packages = metadata["packages"]
        .as_array()
        .expect("metadata packages array");
    let names_by_id = packages
        .iter()
        .map(|package| {
            (
                package["id"].as_str().expect("package id").to_owned(),
                package["name"].as_str().expect("package name").to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let root_id = names_by_id
        .iter()
        .find_map(|(id, name)| (name == root_name).then(|| id.clone()))
        .expect("root package exists");
    let workspace_members = metadata["workspace_members"]
        .as_array()
        .expect("workspace members array")
        .iter()
        .map(|id| id.as_str().expect("workspace member id").to_owned())
        .collect::<BTreeSet<_>>();
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .expect("resolved dependency nodes");
    let dependencies_by_id = nodes
        .iter()
        .map(|node| {
            let id = node["id"].as_str().expect("resolved node id").to_owned();
            let dependencies = node["deps"]
                .as_array()
                .expect("resolved dependency list")
                .iter()
                .map(|dependency| {
                    dependency["pkg"]
                        .as_str()
                        .expect("resolved dependency package id")
                        .to_owned()
                })
                .collect::<Vec<_>>();
            (id, dependencies)
        })
        .collect::<BTreeMap<_, _>>();

    let mut reached = BTreeSet::new();
    let mut queue = VecDeque::from([root_id.clone()]);
    while let Some(id) = queue.pop_front() {
        for dependency in dependencies_by_id
            .get(&id)
            .expect("every package has a resolved node")
        {
            if reached.insert(dependency.clone()) {
                queue.push_back(dependency.clone());
            }
        }
    }

    reached
        .into_iter()
        .filter(|id| workspace_members.contains(id) && id != &root_id)
        .map(|id| {
            names_by_id
                .get(&id)
                .expect("reachable package has a name")
                .clone()
        })
        .collect()
}

fn add_synthetic_workspace_dependency(
    metadata: &mut Value,
    root_name: &str,
    dependency_name: &str,
) {
    let packages = metadata["packages"]
        .as_array()
        .expect("metadata packages array");
    let id_for = |name: &str| {
        packages
            .iter()
            .find_map(|package| {
                (package["name"].as_str() == Some(name))
                    .then(|| package["id"].as_str().expect("package id").to_owned())
            })
            .expect("named package exists")
    };
    let root_id = id_for(root_name);
    let dependency_id = id_for(dependency_name);
    let nodes = metadata["resolve"]["nodes"]
        .as_array_mut()
        .expect("resolved dependency nodes");
    let root = nodes
        .iter_mut()
        .find(|node| node["id"].as_str() == Some(&root_id))
        .expect("root resolved node");
    root["deps"]
        .as_array_mut()
        .expect("root dependency list")
        .push(json!({
            "name": "sec4_synthetic_generated_dependency",
            "pkg": dependency_id,
            "dep_kinds": [{"kind": null, "target": null}]
        }));
}

/// D1, `[structural / architectural]`, durable Invariant TT.
///
/// MEASURED: `cargo metadata`'s resolved graph has no workspace package
/// reachable from `ken-kernel`; adding a synthetic edge to `ken-elaborator`
/// makes the same graph traversal report it.
/// CLAIMED: the Rust kernel has no dependency on a Ken-generated workspace
/// artifact.
/// THE GAP: workspace membership is the bounded set of Ken-built artifacts in
/// this repository; external dependency provenance is outside this invariant.
#[test]
fn kernel_resolved_graph_has_no_ken_generated_workspace_dependency() {
    let metadata = cargo_metadata();
    assert_eq!(
        reachable_workspace_dependencies(&metadata, "ken-kernel"),
        BTreeSet::new(),
        "ken-kernel must remain independent of every Ken workspace artifact"
    );

    let mut control = metadata;
    add_synthetic_workspace_dependency(&mut control, "ken-kernel", "ken-elaborator");
    let detected = reachable_workspace_dependencies(&control, "ken-kernel");
    assert!(
        detected.contains("ken-elaborator"),
        "the resolved-graph oracle must detect the injected dependency: {detected:?}"
    );
}
