//! Out-of-process depth-totality controls for `Value` — `RT-VALUE-TOTALITY-P1`
//! `AC-V1` and `AC-V3`.
//!
//! # Why these run out of process
//!
//! A Rust stack overflow **aborts** the process rather than unwinding, so an
//! in-process assertion cannot distinguish *"the guard fired"* from *"the binary
//! died."* Every control here therefore runs in a child process and the parent
//! asserts on the **process outcome**.
//!
//! ⛔ **There is no cycle-refusal control in this file, and adding one would be a
//! defect.** A back-edge in `Value` is *unconstructible*, not malformed
//! (`evt_45x5dn9jcrhhq`): every child position is owned by value, with no
//! indirection, interior mutation, or slot edge. An AC requiring a cycle
//! *witness* on this carrier is unsatisfiable, and its only available control
//! would be detector-side. The property is pinned structurally instead — by the
//! compiler, in `canonical::child_positions` — and the cycle obligation itself
//! was retargeted onto `RT-FNSPLIT-B2V`'s `BoundaryPersistentImage`, where a
//! cycle *is* constructible.
//!
//! # How `D` was chosen — measured, not guessed
//!
//! Bisected out of process against the **landed** (pre-change) mechanisms, on
//! this worktree, at two thread stack sizes:
//!
//! | thread stack | recursive `encode` | derived `Clone` | drop glue |
//! |---|---|---|---|
//! | 1 MiB | last ok 1121 / **died 1122** | 1252 / **1253** | 8142 / **8143** |
//! | 8 MiB | last ok 9031 / **died 9032** | 10074 / **10075** | 65486 / **65487** |
//!
//! `D = 131072` exceeds **every** one of those thresholds at **both** stack
//! sizes — 16x the 1 MiB drop threshold and 2.0x the 8 MiB one — so the choice
//! cannot be dismissed as an artificially small stack. All six landed
//! (mechanism, stack) pairs were confirmed to abort at exactly this `D`.
//!
//! Each scenario pins its own thread stack to [`STACK_BYTES`] so `D`'s adequacy
//! is a property of a **stated** stack rather than of the ambient `ulimit -s`
//! (8192 KiB on the measuring box, and not guaranteed equal in CI).

use ken_runtime::{Canonical, Value};
use std::process::{Command, Output};

/// Chosen above every measured landed threshold — see the module header.
const D: usize = 131_072;

/// The stated stack for every scenario. Fixed on purpose: the property is
/// "host-stack usage must not grow with value depth," and any *fixed* stack is
/// exceeded by some depth, so instantiating it at a stated size is what makes
/// the control deterministic instead of machine-dependent.
const STACK_BYTES: usize = 1 << 20;

const SCENARIO_ENV: &str = "KEN_RT_TOTALITY_SCENARIO";

/// The `Record` type id used by the unary chain, and its closed-form encoding.
const CHAIN_TYPE_ID: u32 = 1;
const CHAIN_LEAF: i64 = 7;

/// The test's **own** copy of the byte-format constants, so `AC-V1` step 3 is an
/// oracle *independent* of the production `tag` module rather than a restatement
/// of it. If production changes a tag, this reddens — deliberately: these bytes
/// are a normative compatibility vector, not an incidental detail.
mod expected_format {
    pub const TAG_RECORD: u8 = 0x03;
    pub const TAG_SMALL_INT: u8 = 0x1C;
}

/// Build a unary `Record` chain of `depth` nestings around a scalar leaf.
///
/// Construction is a **loop**, so it never recurses; only teardown and the
/// traversals under test can.
fn unary_chain(depth: usize) -> Value {
    let mut v = Value::SmallInt(CHAIN_LEAF);
    for _ in 0..depth {
        v = Value::Record {
            type_id: CHAIN_TYPE_ID,
            fields: vec![v],
        };
    }
    v
}

/// The closed-form canonical encoding of `unary_chain(depth)`, derived from the
/// format alone — `depth` repetitions of `[tag, type_id_le, arity=1_le]`, then
/// the leaf. ⛔ Deliberately does **not** call the production encoder.
fn expected_chain_bytes(depth: usize) -> Vec<u8> {
    let mut expected = Vec::with_capacity(7 * depth + 9);
    for _ in 0..depth {
        expected.push(expected_format::TAG_RECORD);
        expected.extend_from_slice(&CHAIN_TYPE_ID.to_le_bytes());
        expected.extend_from_slice(&1u16.to_le_bytes());
    }
    expected.push(expected_format::TAG_SMALL_INT);
    expected.extend_from_slice(&CHAIN_LEAF.to_le_bytes());
    expected
}

/// A **host-recursive** traversal of the real `Value` population.
///
/// This is the population-side positive control for `AC-V1` step 2: it runs the
/// same recursive shape the landed encoder had, over the genuine carrier, so its
/// death at `D` proves the population is deep enough to break a recursive
/// traversal. Leaf emission delegates to the production encoder (depth-1 for a
/// leaf); the recursion on **child positions** is what is under test here.
fn recursive_encode(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::Record { type_id, fields } => {
            out.push(expected_format::TAG_RECORD);
            out.extend_from_slice(&type_id.to_le_bytes());
            out.extend_from_slice(&(fields.len().min(65535) as u16).to_le_bytes());
            for field in fields {
                recursive_encode(field, out);
            }
        }
        leaf => leaf.encode_canonical(out),
    }
}

/// A **host-recursive** deep clone of the real `Value` population — the
/// population-side positive control for the `Clone` half of `AC-V3c`.
fn recursive_clone(value: &Value) -> Value {
    match value {
        Value::Record { type_id, fields } => Value::Record {
            type_id: *type_id,
            fields: fields.iter().map(recursive_clone).collect(),
        },
        leaf => leaf.clone(),
    }
}

/// A structural analogue of `Value`'s former shape, carrying **derived** `Clone`
/// and **automatic drop glue**.
///
/// ⚠ **Stated honestly:** this is the drop half of `AC-V3c`, and it does *not*
/// exercise `Value`'s own drop glue — that glue no longer exists, because
/// replacing it is this WP's deliverable. It exercises the same *mechanism
/// class* (a nested owned collection torn down by compiler-generated glue) on an
/// analogous per-level frame. The evidence that the **genuine** pre-change
/// `Value` drop glue died at `D` is the recorded out-of-process bisect in the
/// module header, measured against the landed code before it was replaced.
// The payloads are never *read* — they exist to create the nested owned
// structure whose compiler-generated teardown is the subject under test.
#[allow(dead_code)]
#[derive(Clone)]
enum GlueTwin {
    Leaf(i64),
    Node(Vec<GlueTwin>),
}

fn twin_chain(depth: usize) -> GlueTwin {
    let mut t = GlueTwin::Leaf(CHAIN_LEAF);
    for _ in 0..depth {
        t = GlueTwin::Node(vec![t]);
    }
    t
}

// ---------------------------------------------------------------- child worker

/// Runs one scenario in a child process. Inert in the parent (no env var set).
#[test]
fn scenario_worker() {
    let scenario = match std::env::var(SCENARIO_ENV) {
        Ok(s) => s,
        Err(_) => return,
    };

    let worker = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || run_scenario(&scenario))
        .expect("spawn scenario thread");
    worker.join().expect("scenario thread joined");
    std::process::exit(0);
}

fn run_scenario(scenario: &str) {
    match scenario {
        // --- AC-V1 step 3: the NEW encoder succeeds at D with exact bytes ---
        "new_encoder_exact_bytes" => {
            let chain = unary_chain(D);
            let mut out = Vec::new();
            chain.encode_canonical(&mut out);
            assert_eq!(
                out,
                expected_chain_bytes(D),
                "iterative encoder must emit the closed-form bytes at depth {D}"
            );
            println!("OK new_encoder_exact_bytes bytes={}", out.len());
        }

        // --- AC-V1 step 2: a host-recursive traversal DIES on this population ---
        "recursive_encoder_dies" => {
            let chain = unary_chain(D);
            let mut out = Vec::new();
            recursive_encode(&chain, &mut out);
            println!("UNEXPECTED recursive_encoder survived bytes={}", out.len());
        }

        // --- AC-V3b: clone at D, then drop BOTH copies ---
        "clone_then_drop_both" => {
            let chain = unary_chain(D);
            let copy = chain.clone();
            // Encode both and compare, so "cloned" is a positive claim about
            // content rather than merely "did not crash".
            let mut a = Vec::new();
            let mut b = Vec::new();
            chain.encode_canonical(&mut a);
            copy.encode_canonical(&mut b);
            assert_eq!(a, b, "clone must be byte-identical at depth {D}");
            assert_eq!(a, expected_chain_bytes(D));
            drop(chain);
            drop(copy);
            println!("OK clone_then_drop_both bytes={}", a.len());
        }

        // --- AC-V3c (Clone half): recursive clone DIES on the real carrier ---
        "recursive_clone_dies" => {
            let chain = unary_chain(D);
            let copy = recursive_clone(&chain);
            println!("UNEXPECTED recursive_clone survived {}", copy.is_compound());
        }

        // --- AC-V3c (drop half): derived drop glue DIES on the analogue ---
        "derived_drop_glue_dies" => {
            let twin = twin_chain(D);
            // Touch it so construction cannot be optimized away, then drop.
            assert!(matches!(twin, GlueTwin::Node(_)));
            drop(twin);
            println!("UNEXPECTED derived_drop_glue survived");
        }

        // --- AC-V3d: DROP alone at D — no encode, no clone in the body ---
        "drop_only" => {
            let chain = unary_chain(D);
            assert!(chain.is_compound());
            drop(chain);
            println!("OK drop_only depth={D}");
        }

        // --- harness positive control: the parent CAN observe a survivor ---
        "harness_survives" => {
            println!("OK harness_survives");
        }

        other => panic!("unknown scenario {other}"),
    }
}

// ---------------------------------------------------------------- parent side

fn run_child(scenario: &str) -> Output {
    Command::new(std::env::current_exe().expect("current_exe"))
        .args(["--exact", "scenario_worker", "--nocapture"])
        .env(SCENARIO_ENV, scenario)
        .output()
        .expect("spawn child process")
}

/// Assert the scenario completed and **did the work**, evidenced by its marker.
fn assert_survives(scenario: &str) {
    let out = run_child(scenario);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{scenario}: expected success, got {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains(&format!("OK {scenario}")),
        "{scenario}: exited 0 without its completion marker — a bare exit code \
         cannot distinguish 'did the work' from 'never ran it'.\nstdout: {stdout}"
    );
}

/// Assert the scenario died **of stack exhaustion specifically**.
///
/// ⛔ A non-zero exit passes for any reason — an unknown-scenario panic, a failed
/// assertion, a missing binary. So this also requires the runtime's stack-
/// overflow diagnostic, which is what makes the death attributable.
fn assert_dies_of_stack_overflow(scenario: &str) {
    let out = run_child(scenario);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "{scenario}: expected stack exhaustion at depth {D}, but it SURVIVED. \
         If this fires, D is no longer above the recursive threshold and every \
         depth control in this file has stopped proving anything.\nstdout: {stdout}"
    );
    assert!(
        stderr.contains("overflowed its stack") || stderr.contains("stack overflow"),
        "{scenario}: died, but not of stack exhaustion — so this control is \
         passing for the wrong reason.\nstatus: {:?}\nstderr: {stderr}",
        out.status
    );
}

/// The harness itself must be able to observe a *survivor*, or every
/// `assert_survives` above could be vacuous.
#[test]
fn harness_can_observe_a_survivor() {
    assert_survives("harness_survives");
}

/// `AC-V1` step 3 — the new encoder succeeds at `D` and emits exactly the
/// closed-form bytes. ⛔ Not "completed without overflowing": that is a negative
/// check that an encoder emitting nothing would also pass.
#[test]
fn ac_v1_new_encoder_emits_exact_closed_form_bytes_at_depth_d() {
    assert_survives("new_encoder_exact_bytes");
}

/// `AC-V1` step 2 — the load-bearing **population-side** control: a host-
/// recursive traversal of this very population dies at `D`. Without this, step 3
/// passing is compatible with `D` being too shallow to have ever mattered.
#[test]
fn ac_v1_a_host_recursive_traversal_dies_on_this_population_at_depth_d() {
    assert_dies_of_stack_overflow("recursive_encoder_dies");
}

/// `AC-V3b` — clone at `D`, then drop **both** copies.
#[test]
fn ac_v3b_clone_then_drop_both_copies_at_depth_d() {
    assert_survives("clone_then_drop_both");
}

/// `AC-V3c` (`Clone` half) — a host-recursive deep clone of the real carrier
/// dies at `D`.
#[test]
fn ac_v3c_recursive_clone_dies_on_the_real_carrier_at_depth_d() {
    assert_dies_of_stack_overflow("recursive_clone_dies");
}

/// `AC-V3c` (drop half) — compiler-generated drop glue over a nested owned
/// collection dies at `D`. ⚠ On the [`GlueTwin`] analogue, for the reason stated
/// at its definition: `Value`'s own glue no longer exists to be measured.
#[test]
fn ac_v3c_derived_drop_glue_dies_on_the_analogue_at_depth_d() {
    assert_dies_of_stack_overflow("derived_drop_glue_dies");
}

/// `AC-V3d` — drop in isolation: constructed and dropped at `D`, with no encode
/// and no clone in the body. Drop cannot signal failure, so it needs its own arm.
#[test]
fn ac_v3d_drop_alone_is_total_at_depth_d() {
    assert_survives("drop_only");
}
