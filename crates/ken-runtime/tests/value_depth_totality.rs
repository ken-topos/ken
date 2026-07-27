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
    pub const TAG_DATA: u8 = 0x02;
    pub const TAG_RECORD: u8 = 0x03;
    pub const TAG_ARRAY: u8 = 0x06;
    pub const TAG_MAP: u8 = 0x07;
    // ⛔ `0x09` (the retired closure tag) is deliberately absent: the canonical
    // carrier has no closure variant, so no encoding can emit it.
    pub const TAG_SMALL_INT: u8 = 0x1C;
}

// Distinct ids per child-position kind, so a mis-ordered or mis-attributed
// header shows up as a byte difference rather than coincidentally matching.
const MIXED_RECORD_TYPE_ID: u32 = 0x1111_1111;
const MIXED_CTOR_ID: u32 = 0x2222_2222;
const MIXED_ARRAY_ELEM_TYPE_ID: u32 = 0x3333_3333;
const MIXED_MAP_KEY_TYPE_ID: u32 = 0x4444_4444;
const MIXED_MAP_VALUE_TYPE_ID: u32 = 0x5555_5555;

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

/// Build a deep chain that **cycles through every one of the four child
/// positions** — `Record.fields`, `Constructor.args`, `Array.elements`, and
/// `Map`'s entry values.
///
/// ⚠ There were five until `RT-VALUE-TOTALITY-P2` removed `Closure.captured`
/// with the variant that owned it. The claim below is therefore over **four**
/// positions, and four is now the whole surface — not four fifths of it.
///
/// ⭐ **Why this exists, and it is not redundant with [`unary_chain`]:** a
/// unary-`Record` chain is the only population the depth controls would
/// otherwise have, and a *hybrid* encoder — iterative for `Record`, still
/// host-recursive for the other four — passes every one of those controls while
/// leaving the other original recursion sites intact. This chain is what makes
/// the depth claim cover the whole child-position surface rather than one slice
/// of it. Found by attempting that exact evasion (`AC-V5`, row 1).
fn mixed_chain(depth: usize) -> Value {
    let mut v = Value::SmallInt(CHAIN_LEAF);
    for j in 0..depth {
        v = match j % 4 {
            0 => Value::Record {
                type_id: MIXED_RECORD_TYPE_ID,
                fields: vec![v],
            },
            1 => Value::Constructor {
                constructor_id: MIXED_CTOR_ID,
                args: vec![v],
            },
            2 => Value::Array {
                elem_type_id: MIXED_ARRAY_ELEM_TYPE_ID,
                elements: vec![v],
            },
            _ => {
                let mut entries = std::collections::BTreeMap::new();
                entries.insert(mixed_map_key(), v);
                Value::Map {
                    key_type_id: MIXED_MAP_KEY_TYPE_ID,
                    value_type_id: MIXED_MAP_VALUE_TYPE_ID,
                    entries,
                }
            }
        };
    }
    v
}

/// The single `Map` key used by [`mixed_chain`] — the canonical encoding of
/// `SmallInt(0)`, spelled out here rather than obtained from the encoder so the
/// expectation stays independent of the subject.
fn mixed_map_key() -> Vec<u8> {
    let mut key = vec![expected_format::TAG_SMALL_INT];
    key.extend_from_slice(&0i64.to_le_bytes());
    key
}

/// The closed-form canonical encoding of [`mixed_chain`], derived from the
/// format alone. Encoding is pre-order, so the outermost wrapper's header comes
/// first: headers run from `j = depth-1` down to `j = 0`, then the leaf.
fn expected_mixed_chain_bytes(depth: usize) -> Vec<u8> {
    let mut expected = Vec::new();
    for j in (0..depth).rev() {
        // ⚠ Must stay in lockstep with `mixed_chain`'s rotation: this builder is
        // the independent oracle, so a divergent modulus here would compare two
        // different chains and pass for the wrong reason.
        match j % 4 {
            0 => {
                expected.push(expected_format::TAG_RECORD);
                expected.extend_from_slice(&MIXED_RECORD_TYPE_ID.to_le_bytes());
                expected.extend_from_slice(&1u16.to_le_bytes());
            }
            1 => {
                expected.push(expected_format::TAG_DATA);
                expected.extend_from_slice(&MIXED_CTOR_ID.to_le_bytes());
                expected.extend_from_slice(&1u16.to_le_bytes());
            }
            2 => {
                expected.push(expected_format::TAG_ARRAY);
                expected.extend_from_slice(&MIXED_ARRAY_ELEM_TYPE_ID.to_le_bytes());
                expected.extend_from_slice(&1u32.to_le_bytes());
            }
            _ => {
                let key = mixed_map_key();
                expected.push(expected_format::TAG_MAP);
                expected.extend_from_slice(&MIXED_MAP_KEY_TYPE_ID.to_le_bytes());
                expected.extend_from_slice(&MIXED_MAP_VALUE_TYPE_ID.to_le_bytes());
                expected.extend_from_slice(&1u32.to_le_bytes());
                expected.extend_from_slice(&(key.len() as u32).to_le_bytes());
                expected.extend_from_slice(&key);
            }
        }
    }
    expected.push(expected_format::TAG_SMALL_INT);
    expected.extend_from_slice(&CHAIN_LEAF.to_le_bytes());
    expected
}

/// A **host-recursive** traversal of the real `Value` population, covering
/// **every** child position rather than only `Record`.
fn recursive_encode_mixed(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::Record { fields, .. } => {
            out.push(expected_format::TAG_RECORD);
            out.extend_from_slice(&MIXED_RECORD_TYPE_ID.to_le_bytes());
            out.extend_from_slice(&1u16.to_le_bytes());
            for f in fields {
                recursive_encode_mixed(f, out);
            }
        }
        Value::Constructor { args, .. } => {
            out.push(expected_format::TAG_DATA);
            for a in args {
                recursive_encode_mixed(a, out);
            }
        }
        Value::Array { elements, .. } => {
            out.push(expected_format::TAG_ARRAY);
            for e in elements {
                recursive_encode_mixed(e, out);
            }
        }
        Value::Map { entries, .. } => {
            out.push(expected_format::TAG_MAP);
            for v in entries.values() {
                recursive_encode_mixed(v, out);
            }
        }
        leaf => leaf.encode_canonical(out),
    }
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

        // --- AC-V1 step 3, over EVERY child position ---
        "new_encoder_exact_bytes_mixed" => {
            let chain = mixed_chain(D);
            let mut out = Vec::new();
            chain.encode_canonical(&mut out);
            assert_eq!(
                out,
                expected_mixed_chain_bytes(D),
                "iterative encoder must emit the closed-form bytes at depth {D} \
                 through all five child positions"
            );
            println!("OK new_encoder_exact_bytes_mixed bytes={}", out.len());
        }

        // --- AC-V1 step 2, over EVERY child position ---
        "recursive_encoder_dies_mixed" => {
            let chain = mixed_chain(D);
            let mut out = Vec::new();
            recursive_encode_mixed(&chain, &mut out);
            println!("UNEXPECTED recursive_encoder_mixed survived {}", out.len());
        }

        // --- AC-V3b, over EVERY child position ---
        "clone_then_drop_both_mixed" => {
            let chain = mixed_chain(D);
            let copy = chain.clone();
            let mut a = Vec::new();
            let mut b = Vec::new();
            chain.encode_canonical(&mut a);
            copy.encode_canonical(&mut b);
            assert_eq!(a, b, "clone must be byte-identical at depth {D}");
            assert_eq!(a, expected_mixed_chain_bytes(D));
            drop(chain);
            drop(copy);
            println!("OK clone_then_drop_both_mixed bytes={}", a.len());
        }

        // --- AC-V3d, over EVERY child position ---
        "drop_only_mixed" => {
            let chain = mixed_chain(D);
            assert!(chain.is_compound());
            drop(chain);
            println!("OK drop_only_mixed depth={D}");
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

// ---------------------------------------------------------------------------
// The same four claims, over a chain that cycles through ALL FIVE child
// positions. ⭐ Without these, a hybrid encoder that stayed host-recursive for
// `Constructor` / `Array` / `Map` / `Closure` — four of the five original
// recursion sites — passes every control above. See `mixed_chain`.
// ---------------------------------------------------------------------------

/// `AC-V1` step 3 across every child position.
#[test]
fn ac_v1_new_encoder_exact_bytes_at_depth_d_through_all_child_positions() {
    assert_survives("new_encoder_exact_bytes_mixed");
}

/// `AC-V1` step 2 across every child position — the population-side control
/// that makes the claim above cover the whole surface.
#[test]
fn ac_v1_host_recursion_dies_at_depth_d_through_all_child_positions() {
    assert_dies_of_stack_overflow("recursive_encoder_dies_mixed");
}

/// `AC-V3b` across every child position.
#[test]
fn ac_v3b_clone_then_drop_both_at_depth_d_through_all_child_positions() {
    assert_survives("clone_then_drop_both_mixed");
}

/// `AC-V3d` across every child position.
#[test]
fn ac_v3d_drop_alone_is_total_at_depth_d_through_all_child_positions() {
    assert_survives("drop_only_mixed");
}
