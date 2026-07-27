//! Out-of-process depth-totality controls for `Value` — `RT-VALUE-TOTALITY-P1`
//! `AC-V1`/`AC-V3`, and `P3` `AC-V11` (`AC-P3a`–`AC-P3c`).
//!
//! ⛔ **P3 extends this file rather than adding a second harness.** The
//! out-of-process discipline, the stated stack, the measured `D`, and the
//! survivor/death assertions are the same evidence machinery; a parallel copy
//! would drift from this one and the next depth defect would land in whichever
//! the reader did not check.
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

use ken_runtime::canonical::{
    project_operational_to_canonical, CanonicalProjectionRefusal, CanonicalWitness,
};
use ken_runtime::{Canonical, RuntimeValue, Value};
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

/// The leaf `AC-P3c` looks for in the rendered string.
///
/// ⭐ Deliberately a long, unusual digit run rather than [`CHAIN_LEAF`]: `7`
/// occurs incidentally in a `type_id` or a byte and would make the
/// leaf-reachability assertion pass without the traversal ever reaching bottom.
const DEBUG_PROBE_LEAF: i64 = 987_654_321;

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
    /// The leaf of the operational chains `AC-V9` projects.
    pub const TAG_BOOL: u8 = 0x10;
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
    unary_chain_with_leaf(depth, CHAIN_LEAF)
}

/// [`unary_chain`] over a chosen leaf. `AC-V12`'s ordering arm needs two chains
/// that are identical except for one field, so that the comparison's verdict is
/// attributable to a known difference rather than to two unrelated values.
fn unary_chain_with_leaf(depth: usize, leaf: i64) -> Value {
    let mut v = Value::SmallInt(leaf);
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
    expected_chain_bytes_with_leaf(depth, CHAIN_LEAF)
}

fn expected_chain_bytes_with_leaf(depth: usize, leaf: i64) -> Vec<u8> {
    let mut expected = Vec::with_capacity(7 * depth + 9);
    for _ in 0..depth {
        expected.push(expected_format::TAG_RECORD);
        expected.extend_from_slice(&CHAIN_TYPE_ID.to_le_bytes());
        expected.extend_from_slice(&1u16.to_le_bytes());
    }
    expected.push(expected_format::TAG_SMALL_INT);
    expected.extend_from_slice(&leaf.to_le_bytes());
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
    mixed_chain_with_leaf(depth, CHAIN_LEAF)
}

/// How many arms [`mixed_chain_with_leaf`] cycles through before repeating —
/// the `j % 4` in its body.
///
/// ⛔ **This describes the FIXTURE, not [`ALL_CHAIN_VARIANTS`], and the
/// distinction is the whole point.** The coverage control keyed on it must not
/// iterate the inventory it is auditing: a 4→3 omission would shrink the loop
/// and the omitted arm would never be looked at. An independent authority has to
/// supply the trip count.
const MIXED_CHAIN_CYCLE: usize = 4;

/// [`mixed_chain`] over a chosen leaf, mirroring [`unary_chain_with_leaf`].
///
/// `AC-P3c` needs a leaf it can *recognise in the rendered string*, so that
/// "the renderer reached depth `D`" is a positive claim rather than an inference
/// from output size alone.
fn mixed_chain_with_leaf(depth: usize, leaf: i64) -> Value {
    let mut v = Value::SmallInt(leaf);
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

/// The child-bearing `Value` arms, as same-variant chain kinds.
///
/// ⛔ **This is a CURRENT-POPULATION inventory. It is NOT compiler-enforced, and
/// an earlier revision of this comment claimed it was.** That claim was false
/// and `runtime-qa` disproved it by mutation: dropping `ChainVariant::Array`
/// from [`ALL_CHAIN_VARIANTS`] *and* classifying `Value::Array` as `None` in
/// [`chain_variant_of`] **compiles**, and left every control here green.
///
/// ⚠ The reasoning error is worth keeping, because it is subtle and this file
/// contains the correct version of it a few hundred lines away (on
/// `debug_header`): **an exhaustive match forces a new variant to receive *an*
/// arm; it does not force the *right* arm.** `None` is a legal answer. And
/// nothing links this array to the classifier at all — so the inventory can omit
/// a real child-bearing arm and then certify itself.
///
/// ⭐ **What actually guards it** is
/// [`all_chain_variants_covers_every_arm_the_mixed_fixture_nests`], which keys
/// off `mixed_chain` — a fixture authored by P1, independently of this list —
/// and reddens under exactly the omission above.
///
/// ⚠ **Residual, stated rather than implied:** that control pins this inventory
/// against *today's* `mixed_chain`. If a future child-bearing variant is added
/// and **neither** `mixed_chain` nor this list learns about it, nothing reddens.
/// Rust has no reflection over enum variants, so any in-test "list of
/// child-bearing arms" is ultimately another hand-written match a future author
/// can mis-answer. That gap is **review-enforced, not mechanically guarded**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChainVariant {
    Record,
    Constructor,
    Array,
    Map,
}

/// Every chain kind in one place. Paired with [`chain_variant_of`]'s
/// exhaustiveness, this is what closes the same-variant population.
const ALL_CHAIN_VARIANTS: [ChainVariant; 4] = [
    ChainVariant::Record,
    ChainVariant::Constructor,
    ChainVariant::Array,
    ChainVariant::Map,
];

const SAME_SURVIVE_PREFIX: &str = "debug_at_depth_d_same_";
const SAME_DIES_PREFIX: &str = "recursive_debug_dies_same_";

impl ChainVariant {
    /// The scenario-name suffix this arm's controls are spelled with.
    fn tag(self) -> &'static str {
        match self {
            ChainVariant::Record => "record",
            ChainVariant::Constructor => "constructor",
            ChainVariant::Array => "array",
            ChainVariant::Map => "map",
        }
    }

    fn from_tag(tag: &str) -> ChainVariant {
        *ALL_CHAIN_VARIANTS
            .iter()
            .find(|variant| variant.tag() == tag)
            .unwrap_or_else(|| panic!("unknown chain-variant tag {tag}"))
    }
}

/// Classify a value's child-bearing arm.
///
/// ⛔ **Exhaustive over every `Value` variant with no `_` arm — and that buys
/// LESS than it looks like.** A new variant is forced to receive *an* arm here;
/// it is **not** forced to receive the *correct* one, because `None` compiles.
/// ⇒ This match is a prompt to classify, not a proof of classification. See
/// [`ALL_CHAIN_VARIANTS`] for the mutation that demonstrated the difference.
fn chain_variant_of(value: &Value) -> Option<ChainVariant> {
    match value {
        Value::Record { .. } => Some(ChainVariant::Record),
        Value::Constructor { .. } => Some(ChainVariant::Constructor),
        Value::Array { .. } => Some(ChainVariant::Array),
        Value::Map { .. } => Some(ChainVariant::Map),

        Value::BigInt { .. }
        | Value::BigDecimal { .. }
        | Value::String(_)
        | Value::Bytes(_)
        | Value::Set { .. }
        | Value::Bool(_)
        | Value::Char(_)
        | Value::Float(_)
        | Value::Float32(_)
        | Value::Int8(_)
        | Value::Int16(_)
        | Value::Int32(_)
        | Value::Int64(_)
        | Value::UInt8(_)
        | Value::UInt16(_)
        | Value::UInt32(_)
        | Value::UInt64(_)
        | Value::SmallInt(_)
        | Value::SmallDecimal { .. }
        | Value::Unknown => None,
    }
}

/// A chain of `depth` nestings of **one** arm inside itself.
///
/// ⭐ **Why this exists and neither [`unary_chain`] nor [`mixed_chain`] replaces
/// it.** `mixed_chain` *cycles* the four arms, so no variant is ever its own
/// child. A per-arm host-recursive leg therefore descends exactly **one** level
/// there before the next, iterative, arm queues the rest — and the control stays
/// green. Measured live by `runtime-qa`: replacing only the `Constructor` arm's
/// worklist enqueue with direct descent left the mixed control passing.
/// `unary_chain` supplies same-variant nesting for `Record` alone, so before
/// this the other three arms had **no same-variant population at all**.
///
/// ⚠ The general lesson, since it outlives this fixture: "deep" was being read
/// as *depth of the value*, when the property under test needs *depth of nesting
/// within a single arm*. A mutation that reddens proves a detector fired; it
/// says nothing about the arms the population never exercised.
fn same_variant_chain(variant: ChainVariant, depth: usize, leaf: i64) -> Value {
    let mut v = Value::SmallInt(leaf);
    for _ in 0..depth {
        v = match variant {
            ChainVariant::Record => Value::Record {
                type_id: MIXED_RECORD_TYPE_ID,
                fields: vec![v],
            },
            ChainVariant::Constructor => Value::Constructor {
                constructor_id: MIXED_CTOR_ID,
                args: vec![v],
            },
            ChainVariant::Array => Value::Array {
                elem_type_id: MIXED_ARRAY_ELEM_TYPE_ID,
                elements: vec![v],
            },
            ChainVariant::Map => {
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

// ------------------------------------------------- RT-VALUE-TOTALITY-P2 D4/D3
// Builders and oracles for the operational carrier and the sealed witness.
// `AC-V9` (projection is transitive/iterative/fail-closed at depth) and
// `AC-V12` (the comparison is depth-total) both need a *deep* population that
// P1's controls above do not supply, because they only ever build `Value`.

/// The single record field name used by every operational chain below.
const RT_FIELD: &str = "f";

/// The intern key the projection is *required* to derive for [`RT_FIELD`] —
/// `record:` followed by the field names in order (`canonical.rs`'s stated
/// convention). ⭐ [`strict_intern`] asserts on it, so these scenarios pin the
/// **interning convention** as well as the bytes; a projection that interned
/// the bare field name, or the type's name, reddens here rather than silently
/// producing a differently-keyed record.
const RT_RECORD_KEY: &str = "record:f";

/// Deliberately not derivable from [`RT_RECORD_KEY`] by any expression the
/// production code also evaluates — an oracle that recomputed the id the way
/// production does would be a restatement of it, not a check on it.
const RT_TYPE_ID: u32 = 0x6060_6060;

/// The leaf of the operational chains. `Bool` rather than an integer so the
/// projected leaf tag differs from [`CHAIN_LEAF`]'s, and a scenario that
/// silently fell back to the `Value` builders would not match.
const RT_LEAF: bool = true;

fn strict_intern(name: &str) -> u32 {
    assert_eq!(
        name, RT_RECORD_KEY,
        "the projection interned an unexpected key — the `record:<fields>` \
         convention this oracle depends on has changed"
    );
    RT_TYPE_ID
}

/// A unary `RuntimeValue::Record` chain of `depth` nestings around a `Bool`.
/// Built by **loop**, so construction never recurses.
fn runtime_unary_chain(depth: usize) -> RuntimeValue {
    let mut v = RuntimeValue::Bool(RT_LEAF);
    for _ in 0..depth {
        v = RuntimeValue::Record {
            fields: vec![(RT_FIELD.to_string(), v)],
        };
    }
    v
}

/// The same chain, but the innermost node is an **ordinary closure** — so the
/// closure sits at depth `depth` from the root.
///
/// ⛔ It carries no captures on purpose: an empty capture must not change an
/// ordinary closure's class (`AC-V5b`, ruling pin 5), so this is the *weakest*
/// closure the refusal has to catch.
fn runtime_chain_with_closure_at(depth: usize) -> RuntimeValue {
    let mut v = RuntimeValue::ClosureRef {
        symbol: RT_FIELD.to_string(),
        captured: vec![],
    };
    for _ in 0..depth {
        v = RuntimeValue::Record {
            fields: vec![(RT_FIELD.to_string(), v)],
        };
    }
    v
}

/// The closed-form canonical encoding of the *projection* of
/// [`runtime_unary_chain`] — derived from the byte format alone, not by calling
/// the encoder. The operational record is named-field and the canonical one is
/// positional, so the projection collapses `{f: _}` onto a one-field `Record`
/// whose `type_id` is the interned [`RT_RECORD_KEY`].
fn expected_projected_chain_bytes(depth: usize) -> Vec<u8> {
    let mut expected = Vec::with_capacity(7 * depth + 1);
    for _ in 0..depth {
        expected.push(expected_format::TAG_RECORD);
        expected.extend_from_slice(&RT_TYPE_ID.to_le_bytes());
        expected.extend_from_slice(&1u16.to_le_bytes());
    }
    expected.push(expected_format::TAG_BOOL);
    expected.push(u8::from(RT_LEAF));
    expected
}

/// ⛔ **`RuntimeValue` has ordinary derived drop glue and is NOT depth-total.**
/// P1 made the *canonical* carrier total; the operational carrier was never in
/// that scope, and `scenario_runtime_drop_glue_dies` measures it rather than
/// leaving it as an assumption. Every scenario that builds a deep
/// `RuntimeValue` therefore ends with `std::mem::forget`.
///
/// ⚠ This is a leak, and it is correct here for a stated reason: the child
/// process calls `exit(0)` immediately after, and the subject under test is the
/// **projection**, not the input carrier's teardown. ⛔ It is not a workaround
/// concealing a defect — the defect is measured, and named as a residual in the
/// handoff.
fn forget_deep(value: RuntimeValue) {
    std::mem::forget(value);
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

/// A **host-recursive** rendering of the real `Value` population — the
/// population-side positive control for `AC-P3a`/`AC-P3b`.
///
/// ⭐ Without this, "the new `Debug` survived at `D`" is compatible with `D`
/// being too shallow to have broken a recursive renderer in the first place.
/// Leaf rendering delegates to the production impl (depth-1 for a leaf); the
/// recursion on **child positions** is what is under test.
fn recursive_debug(value: &Value, out: &mut String) {
    use std::fmt::Write as _;
    match value {
        Value::Record { type_id, fields } => {
            let _ = write!(out, "Record {{ type_id: {type_id}, fields: [");
            for field in fields {
                recursive_debug(field, out);
            }
            out.push_str("] }");
        }
        leaf => {
            let _ = write!(out, "{leaf:?}");
        }
    }
}

/// [`recursive_debug`] over **every** child position, not just `Record`.
///
/// ⚠ The `AC-V5` hybrid-evasion lesson applies verbatim here: an impl that is
/// iterative for `Record` and still host-recursive for the other four passes
/// every unary-chain control. This is the population that refuses that evasion.
fn recursive_debug_mixed(value: &Value, out: &mut String) {
    use std::fmt::Write as _;
    match value {
        Value::Record { fields: kids, .. }
        | Value::Constructor { args: kids, .. }
        | Value::Array { elements: kids, .. } => {
            out.push('[');
            for kid in kids {
                recursive_debug_mixed(kid, out);
            }
            out.push(']');
        }
        Value::Map { entries, .. } => {
            out.push('{');
            for val in entries.values() {
                recursive_debug_mixed(val, out);
            }
            out.push('}');
        }
        leaf => {
            let _ = write!(out, "{leaf:?}");
        }
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

/// Two witnesses of *identical* depth-`D` chains, plus one whose chain differs
/// only in its leaf — the fixture every `AC-V12` arm shares.
///
/// ⭐ Returning both an equal pair and a differing one is what makes each arm
/// two-sided. A single pair leaves *"the operation agrees"* indistinguishable
/// from *"the operation returns the same answer for everything."*
///
/// The chains are dropped here: `Value`'s teardown is iterative (P1), so only
/// the three flat byte vectors survive into the scenario.
fn witness_trio() -> ((CanonicalWitness, CanonicalWitness), CanonicalWitness) {
    let a = CanonicalWitness::of(&unary_chain(D));
    let b = CanonicalWitness::of(&unary_chain(D));
    let c = CanonicalWitness::of(&unary_chain_with_leaf(D, CHAIN_LEAF + 1));
    ((a, b), c)
}

/// ⛔ **Every `AC-V12` arm asserts this, and none may inherit it from another.**
/// The witness must genuinely BE the depth-`D` canonical encoding — otherwise a
/// witness of an empty value satisfies `==`, `<` and `hash` alike, and each arm
/// reports the same green it would report having exercised nothing.
///
/// ⚠ Deliberately not hoisted into [`witness_trio`]: an arm run in isolation
/// must still carry its own evidence, and a shared fixture that asserted once
/// would leave two of the three arms silently depending on the first.
fn assert_depth_d_witness(witness: &CanonicalWitness) {
    assert_eq!(
        witness.bytes(),
        expected_chain_bytes(D).as_slice(),
        "the witness must be the depth-{D} canonical encoding"
    );
}

fn hash_of(witness: &CanonicalWitness) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    witness.hash(&mut hasher);
    hasher.finish()
}

// ---------------------------------------------------------------- child worker

/// Runs one scenario in a child process. Inert in the parent (no env var set).
/// `AC-P3c` — the positive control for `AC-P3b`.
///
/// ⛔ **`AC-P3b` cannot fail honestly on its own.** "The child exited 0" is a
/// negative check: a `Debug` that rendered *nothing*, or that stopped after the
/// outermost node, exits 0 exactly like one that walked all `D` levels. These
/// two assertions are what separate those outcomes.
///
/// ⚠ **Both are deliberately format-INDEPENDENT, per `AC-P3e`.** `Debug` output
/// is unspecified and must not be frozen, so neither assertion names a
/// delimiter, a field label, or an exact length:
///
/// - **Scale** — a *lower bound* of `D` bytes. Any rendering that descends `D`
///   levels emits at least one byte per level, and no rendering that stops
///   early can reach `D` bytes. A shallow render of this value is tens of bytes
///   against `D` = 131072.
/// - **Reachability** — the deepest leaf's *digits* appear. This is a claim
///   about the leaf **value**, not about how the renderer spells a node, and it
///   is what catches a traversal that emits `D` opening headers and then never
///   reaches bottom — which the length bound alone would pass.
fn assert_rendered_to_depth_d(rendered: &str, scenario: &str) {
    assert!(
        rendered.len() >= D,
        "{scenario}: rendered only {} bytes at depth {D}. A render that walked \
         every level emits at least one byte per level, so this is a render \
         that stopped early — and a bare exit code could not have told us.",
        rendered.len()
    );
    assert!(
        rendered.contains(&DEBUG_PROBE_LEAF.to_string()),
        "{scenario}: the deepest leaf {DEBUG_PROBE_LEAF} never appears, so the \
         traversal emitted depth-scale output without reaching bottom."
    );
}

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
                 through all four child positions"
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

        // -------------------------------------------------------------------
        // `AC-V9` — the projection is transitive, iterative and fail-closed AT
        // DEPTH. ⚠ `D` is stated as a number in every marker below, because a
        // control that projected *nothing* would report the same bare green as
        // one that projected a 131072-deep value.
        // -------------------------------------------------------------------

        // Positive arm: the projection completes at D and yields exactly the
        // closed-form canonical bytes. ⛔ Not "returned Ok": that is a negative
        // check a projection emitting an empty record would also pass.
        "projection_succeeds_at_depth" => {
            let chain = runtime_unary_chain(D);
            let mut intern = strict_intern;
            let projected = project_operational_to_canonical(&chain, &mut intern)
                .expect("a closure-free operational chain must project");
            let mut bytes = Vec::new();
            projected.encode_canonical(&mut bytes);
            assert_eq!(
                bytes,
                expected_projected_chain_bytes(D),
                "the projection of a depth-{D} operational chain must encode to \
                 the closed-form bytes"
            );
            forget_deep(chain);
            println!("OK projection_succeeds_at_depth D={D} bytes={}", bytes.len());
        }

        // Negative arm: a closure at depth D-1 is refused, transitively.
        // ⛔ A refusal at depth 1 establishes nothing about transitivity — the
        // whole point is that 131071 ancestors were already visited and the
        // projection still produced NO image.
        "projection_refuses_closure_at_depth" => {
            let closure_depth = D - 1;
            let chain = runtime_chain_with_closure_at(closure_depth);
            let mut intern = strict_intern;
            let outcome = project_operational_to_canonical(&chain, &mut intern);
            assert!(
                matches!(outcome, Err(CanonicalProjectionRefusal::OrdinaryClosure)),
                "a closure at depth {closure_depth} must be refused as \
                 OrdinaryClosure, not merely as some error"
            );
            // ⭐ "Refuses WITHOUT having produced bytes" is enforced by the
            // signature, and this line is what makes that visible: the only way
            // to obtain bytes is to encode a `Value`, and the `Err` arm holds
            // none. There is no partial image to inspect because none can exist.
            assert!(outcome.is_err());
            forget_deep(chain);
            println!("OK projection_refuses_closure_at_depth D={D} closure_at={closure_depth}");
        }

        // Population-side control, and the residual measurement that justifies
        // `forget_deep`: the OPERATIONAL carrier's derived drop glue dies on
        // this very population. ⇒ The chains above are genuinely deep, and
        // `RuntimeValue` is not depth-total.
        "runtime_drop_glue_dies" => {
            let chain = runtime_unary_chain(D);
            assert!(matches!(chain, RuntimeValue::Record { .. }));
            drop(chain);
            println!("UNEXPECTED runtime_drop_glue survived D={D}");
        }

        // -------------------------------------------------------------------
        // `AC-V12` — the chosen `AC-V8` mechanism is DEPTH-TOTAL, one arm per
        // comparison operation. ⛔ One arm does not stand in for the others.
        //
        // ⭐ The CLAIM is the mechanism: a `CanonicalWitness` *is* the canonical
        // bytes, which P1's iterative encoder produces, so `==` / `<` / `hash`
        // walk a flat `Vec<u8>` and do not recurse on value depth at all. These
        // measurements are corroboration beside that claim, not the claim —
        // they are green against one depth on one platform and would re-derive
        // nothing if the traversal changed.
        // -------------------------------------------------------------------
        "witness_eq_at_depth" => {
            let (same, other) = witness_trio();
            assert_depth_d_witness(&same.0);
            assert!(same.0 == same.1, "equal chains must have equal witnesses");
            assert!(
                same.0 != other,
                "chains differing in their leaf must NOT have equal witnesses — \
                 without this arm, a witness that ignored its input passes"
            );
            println!("OK witness_eq_at_depth D={D} bytes={}", same.0.bytes().len());
        }

        "witness_ord_at_depth" => {
            let (same, other) = witness_trio();
            assert_depth_d_witness(&same.0);
            // The two encodings are the same length and differ only in the
            // final 8 leaf bytes (`CHAIN_LEAF` vs `CHAIN_LEAF + 1`, LE), so
            // lexicographic order is decided there and the direction is known.
            assert!(same.0 < other, "leaf {CHAIN_LEAF} must order before its successor");
            assert!(!(other < same.0), "and the order must be antisymmetric");
            assert!(!(same.0 < same.1), "equal witnesses must not order strictly");
            assert_eq!(same.0.cmp(&same.1), std::cmp::Ordering::Equal);
            println!("OK witness_ord_at_depth D={D} bytes={}", same.0.bytes().len());
        }

        "witness_hash_at_depth" => {
            let (same, other) = witness_trio();
            assert_depth_d_witness(&same.0);
            assert_eq!(
                hash_of(&same.0),
                hash_of(&same.1),
                "the `Hash`/`Eq` contract: equal witnesses must hash equal"
            );
            // ⚠ Non-vacuity, with a stated caveat: hash inequality is not part
            // of the contract, so this arm could in principle red on a genuine
            // collision (~2^-64). It is here because "equal hashes equal" alone
            // is satisfied by a hasher that returns a constant.
            assert_ne!(
                hash_of(&same.0),
                hash_of(&other),
                "witnesses of different chains must not hash alike"
            );
            println!("OK witness_hash_at_depth D={D} bytes={}", same.0.bytes().len());
        }

        // --- AC-P3b/AC-P3c: the NEW Debug returns at D, and actually rendered ---
        "debug_at_depth_d" => {
            let chain = unary_chain_with_leaf(D, DEBUG_PROBE_LEAF);
            let rendered = format!("{chain:?}");
            assert_rendered_to_depth_d(&rendered, "debug_at_depth_d");
            println!("OK debug_at_depth_d len={}", rendered.len());
        }

        // --- AC-P3b/AC-P3c, over EVERY child position ---
        "debug_at_depth_d_mixed" => {
            let chain = mixed_chain_with_leaf(D, DEBUG_PROBE_LEAF);
            let rendered = format!("{chain:?}");
            assert_rendered_to_depth_d(&rendered, "debug_at_depth_d_mixed");
            println!("OK debug_at_depth_d_mixed len={}", rendered.len());
        }

        // --- AC-P3a population control: a host-recursive renderer DIES at D ---
        "recursive_debug_dies" => {
            let chain = unary_chain_with_leaf(D, DEBUG_PROBE_LEAF);
            let mut out = String::new();
            recursive_debug(&chain, &mut out);
            println!("UNEXPECTED recursive_debug survived len={}", out.len());
        }

        // --- AC-P3a population control, over EVERY child position ---
        "recursive_debug_dies_mixed" => {
            let chain = mixed_chain_with_leaf(D, DEBUG_PROBE_LEAF);
            let mut out = String::new();
            recursive_debug_mixed(&chain, &mut out);
            println!("UNEXPECTED recursive_debug_mixed survived len={}", out.len());
        }

        // --- harness positive control: the parent CAN observe a survivor ---
        "harness_survives" => {
            println!("OK harness_survives");
        }

        // --- AC-P3b/AC-P3c per child-bearing arm, SAME-VARIANT nesting ---
        s if s.starts_with(SAME_SURVIVE_PREFIX) => {
            let variant = ChainVariant::from_tag(&s[SAME_SURVIVE_PREFIX.len()..]);
            let chain = same_variant_chain(variant, D, DEBUG_PROBE_LEAF);
            let rendered = format!("{chain:?}");
            assert_rendered_to_depth_d(&rendered, s);
            println!("OK {s} len={}", rendered.len());
        }

        // --- AC-P3a population control per arm: recursion DIES on this shape ---
        s if s.starts_with(SAME_DIES_PREFIX) => {
            let variant = ChainVariant::from_tag(&s[SAME_DIES_PREFIX.len()..]);
            let chain = same_variant_chain(variant, D, DEBUG_PROBE_LEAF);
            let mut out = String::new();
            recursive_debug_mixed(&chain, &mut out);
            println!("UNEXPECTED {s} survived len={}", out.len());
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

// ------------------------------------------------- RT-VALUE-TOTALITY-P3 (V11)
//
// `AC-P3a` is discharged by the **mechanism**, not by these controls: the
// hand-written `impl Debug for Value` drives the same explicit heap worklist
// P1's canonical encoder uses (`DebugStep::{Val, Lit, Key}`, one `Vec` on the
// heap), so the only host frames are `fmt` -> `debug_header`, one deep, for
// every node regardless of depth.
//
// ⛔ These controls are **corroboration**. `D = 131072` is a single finite probe
// on one platform; it supports the structural claim and does not constitute it.
// The depth is stated here, before any run, as the module constant `D`.

/// `AC-P3a` population control — a host-recursive renderer dies on this very
/// population at `D`. Without it, the survival controls below are compatible
/// with `D` being too shallow to have broken a recursive renderer at all.
#[test]
fn ac_p3a_host_recursive_debug_dies_at_depth_d() {
    assert_dies_of_stack_overflow("recursive_debug_dies");
}

/// `AC-P3a` population control, over **every** child position — refuses the
/// `AC-V5` hybrid evasion (iterative for `Record`, recursive for the rest).
#[test]
fn ac_p3a_host_recursive_debug_dies_at_depth_d_through_all_child_positions() {
    assert_dies_of_stack_overflow("recursive_debug_dies_mixed");
}

/// `AC-P3b`+`AC-P3c` — `{:?}` at `D` returns, **and actually rendered**.
/// ⛔ Not "exited 0": see [`assert_rendered_to_depth_d`] for why that alone
/// would pass for a renderer that emitted nothing.
#[test]
fn ac_p3b_debug_returns_and_renders_at_depth_d() {
    assert_survives("debug_at_depth_d");
}

/// `AC-P3b`+`AC-P3c`, over **every** child position.
#[test]
fn ac_p3b_debug_returns_and_renders_at_depth_d_through_all_child_positions() {
    assert_survives("debug_at_depth_d_mixed");
}

// --------------------------------------------- same-variant depth, per arm
//
// ⛔ **These are not redundant with the two controls above, and the gap they
// close was a live false green** (`runtime-qa`, BLOCK on `82918ace`).
//
// `mixed_chain` cycles the four arms, so no variant is ever its own child: a
// recursive leg in one arm descends a single level before the next, iterative,
// arm queues the rest. `unary_chain` supplies same-variant nesting for `Record`
// only. ⇒ `Constructor`, `Array` and `Map` each had a host-recursive leg that
// **no control could observe**.
//
// ⭐ Both tests iterate `ALL_CHAIN_VARIANTS` rather than naming arms, so an arm
// added to that inventory is covered with no new test code. ⛔ That is a
// convenience, **not** a guarantee that the inventory is complete — the
// inventory is audited separately, and only against today's population, by
// `all_chain_variants_covers_every_arm_the_mixed_fixture_nests`.

/// Non-vacuity for the population itself: each chain kind really builds **its
/// own** arm. ⛔ Without this, a copy-paste making the `Array` chain build
/// `Record`s would leave both controls below green and `Array` unexercised —
/// the same false-green shape one level down.
#[test]
fn every_chain_variant_builds_a_chain_of_that_variant() {
    for variant in ALL_CHAIN_VARIANTS {
        let built = same_variant_chain(variant, 1, CHAIN_LEAF);
        assert_eq!(
            chain_variant_of(&built),
            Some(variant),
            "same_variant_chain({variant:?}) built a different arm, so that \
             arm's depth controls would exercise the wrong shape"
        );
    }
}

/// ⭐ **The inventory audit — this is what `ALL_CHAIN_VARIANTS` is actually
/// guarded by**, since the exhaustive match is not the guarantee an earlier
/// revision claimed.
///
/// **MEASURED:** every arm that `mixed_chain` nests is classified by
/// [`chain_variant_of`] as child-bearing *and* appears in
/// [`ALL_CHAIN_VARIANTS`], with no duplicates and none left over.
/// **CLAIMED:** every child-bearing `Value` arm has a same-variant depth
/// control.
/// **THE GAP:** `mixed_chain` must itself nest every child-bearing arm. It does
/// today, and it is P1's fixture — authored before this WP and independently of
/// this inventory, which is what makes it a usable authority rather than a
/// restatement. ⚠ If a future variant is added and **neither** `mixed_chain`
/// nor `ALL_CHAIN_VARIANTS` learns about it, nothing here reddens. That arm is
/// **review-enforced, not mechanically guarded** — see [`ALL_CHAIN_VARIANTS`].
///
/// ⭐ **Why it iterates [`MIXED_CHAIN_CYCLE`] and never
/// `ALL_CHAIN_VARIANTS.len()`:** iterating the list under audit lets an omission
/// shrink the loop and hide itself. The trip count must come from the authority,
/// not from the subject.
///
/// Reddens under the exact mutation that defeated the previous revision
/// (`runtime-qa`): drop `Array` from the inventory **and** classify
/// `Value::Array` as `None`. Cycle position 2 then classifies as childless and
/// this fails, naming that position.
#[test]
fn all_chain_variants_covers_every_arm_the_mixed_fixture_nests() {
    // `mixed_chain_with_leaf` wraps with `j % MIXED_CHAIN_CYCLE`, so the
    // OUTERMOST node of `mixed_chain(j + 1)` is the arm at cycle position `j`.
    // Reading the outermost node needs no child-enumerator of our own — which
    // matters, because a private enumerator would be one more hand-written
    // match able to make the same mistake this control exists to catch.
    let mut seen: Vec<ChainVariant> = Vec::new();

    for j in 0..MIXED_CHAIN_CYCLE {
        let node = mixed_chain(j + 1);
        let variant = chain_variant_of(&node).unwrap_or_else(|| {
            panic!(
                "mixed_chain nests a child-bearing arm at cycle position {j} \
                 that chain_variant_of reports as CHILDLESS. That arm is \
                 invisible to ALL_CHAIN_VARIANTS, so it has no same-variant \
                 depth control and a host-recursive leg in it would not be \
                 observed by any test in this file."
            )
        });
        assert!(
            ALL_CHAIN_VARIANTS.contains(&variant),
            "cycle position {j} is {variant:?}, a child-bearing arm absent from \
             ALL_CHAIN_VARIANTS — the per-arm controls never build a \
             same-variant chain for it"
        );
        assert!(
            !seen.contains(&variant),
            "cycle position {j} repeats {variant:?}; the fixture is no longer \
             covering one arm per position, so this audit is measuring fewer \
             arms than it appears to"
        );
        seen.push(variant);
    }

    assert_eq!(
        seen.len(),
        ALL_CHAIN_VARIANTS.len(),
        "ALL_CHAIN_VARIANTS lists {} arms but the mixed fixture nests {}: \
         {seen:?} vs {ALL_CHAIN_VARIANTS:?}. A listed arm the fixture never \
         nests is unaudited by this control; an arm the fixture nests that is \
         not listed has no same-variant depth control at all.",
        ALL_CHAIN_VARIANTS.len(),
        seen.len()
    );
}

/// `AC-P3a` population control, per arm — a host-recursive renderer dies at `D`
/// on a same-variant chain of **each** child-bearing arm.
#[test]
fn ac_p3a_host_recursive_debug_dies_at_depth_d_for_every_same_variant_chain() {
    for variant in ALL_CHAIN_VARIANTS {
        assert_dies_of_stack_overflow(&format!("{SAME_DIES_PREFIX}{}", variant.tag()));
    }
}

/// `AC-P3b`+`AC-P3c`, per arm — `{:?}` returns and renders at `D` on a
/// same-variant chain of **each** child-bearing arm. This is the control that
/// reddens under a single-arm worklist bypass.
#[test]
fn ac_p3b_debug_returns_and_renders_at_depth_d_for_every_same_variant_chain() {
    for variant in ALL_CHAIN_VARIANTS {
        assert_survives(&format!("{SAME_SURVIVE_PREFIX}{}", variant.tag()));
    }
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
// The same four claims, over a chain that cycles through ALL FOUR child
// positions. ⭐ Without these, a hybrid encoder that stayed host-recursive for
// `Constructor` / `Array` / `Map` — three of the four surviving recursion sites
// — passes every control above. See `mixed_chain`.
//
// ⚠ Said "five" until `RT-VALUE-TOTALITY-P2` `D1` removed `Closure.captured`
// with the variant that owned it. Four is now the whole surface, not four
// fifths of it.
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

// ---------------------------------------------------------------------------
// `RT-VALUE-TOTALITY-P2` — `AC-V9` (the checked projection) and `AC-V12` (the
// sealed witness's comparison), both at the same `D` the controls above use.
//
// ⭐ **CAUSAL MUTATION EVIDENCE.** Each arm below was proven to fail for its own
// reason by perturbing the production mechanism at its natural site, confirming
// the redness, and restoring byte-identically (`git diff --quiet`, exit 0):
//
// | mutation, in `canonical.rs` | what reddened |
// |---|---|
// | `minimal_limbs` bypassed at the `BigInt` encode site | `AC-V8`'s bigint arm, on its **premise**; ⭐ the NFC arm stayed **green** |
// | `CanonicalWitness::of` appends `{value:?}`, so the witness is no longer the bytes | **both** `AC-V8` arms, each on its **agreement** assertion, both premises intact |
// | the `ClosureRef` refusal fires only when `ptr::eq(value, root)` | all three transitivity controls, incl. the depth-`D-1` arm below |
// | `Rv::Record` projected by self-recursion instead of the worklist | ⛔ **only the two depth arms below** |
//
// ⛔ **Read the last row before deleting anything here.** Under a *recursive*
// projection — the exact hazard `D4` exists to prevent — **all 443 lib tests
// stayed green** and only the two out-of-process arms in this file caught it,
// dying of a confirmed `fatal runtime error: stack overflow`. ⇒ The in-process
// controls in `canonical.rs` are structurally blind to iterativeness, so these
// arms are the only thing standing between the codebase and a silent
// reintroduction of P1's overflow one layer out.
// ---------------------------------------------------------------------------

/// `AC-V9`, positive arm — the operational → canonical projection completes at
/// `D = 131072` and emits exactly the closed-form bytes.
///
/// ⚠ `D` is stated as a number in the scenario's marker, per the AC: a control
/// that projected nothing would otherwise report the same green as this one.
#[test]
fn ac_v9_projection_completes_at_depth_d_with_exact_closed_form_bytes() {
    assert_survives("projection_succeeds_at_depth");
}

/// `AC-V9`, negative arm — a closure at depth `D-1` is refused **transitively**,
/// with no image produced. ⛔ A refusal at depth 1 does not establish this.
#[test]
fn ac_v9_a_closure_at_depth_d_minus_one_is_refused_transitively() {
    assert_survives("projection_refuses_closure_at_depth");
}

/// The population-side control for both arms above: the **operational**
/// carrier's derived drop glue dies on this very chain.
///
/// ⭐ Without it, the two `AC-V9` arms are compatible with `D` being too shallow
/// to have exercised anything — this is the `AC-V1` step-2 discipline applied to
/// `RuntimeValue`.
///
/// ⚠ **It also records a residual, and that is deliberate.** `RuntimeValue` is
/// *not* depth-total: P1 made the canonical carrier total and the operational
/// carrier was never in that scope. Making the fact a green control means the
/// next reader inherits a measurement instead of an assumption, and it is why
/// the projection scenarios `std::mem::forget` their input.
#[test]
fn the_operational_carriers_drop_glue_dies_on_this_population_at_depth_d() {
    assert_dies_of_stack_overflow("runtime_drop_glue_dies");
}

/// `AC-V12` arm 1 of 3 — `==` is total at `D`.
#[test]
fn ac_v12_witness_equality_is_total_at_depth_d() {
    assert_survives("witness_eq_at_depth");
}

/// `AC-V12` arm 2 of 3 — `<` is total at `D`. ⛔ Not implied by the `==` arm:
/// they are separate trait methods over the same bytes, and only one of them
/// was measured by the other.
#[test]
fn ac_v12_witness_ordering_is_total_at_depth_d() {
    assert_survives("witness_ord_at_depth");
}

/// `AC-V12` arm 3 of 3 — `hash` is total at `D`.
#[test]
fn ac_v12_witness_hashing_is_total_at_depth_d() {
    assert_survives("witness_hash_at_depth");
}
