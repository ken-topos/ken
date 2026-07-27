//! Value types — `spec/40-runtime/41-values.md §1–2,§5,§6`.
//!
//! Scalars are immediate (never interned). Compounds are content-addressed.
//! `Unknown` is the third truth value for partially-verified programs.
//!
//! ⛔ **This is the CANONICAL carrier, and it is closure-free by construction.**
//! Content-addressing is total over it *because* an ordinary closure cannot be
//! built here at all — `41 §2.1` grants closures no structural equality,
//! ordering, canonical hash, slot identity, or persistence, so a carrier that
//! admits canonical encoding, hashing, interning and slot identity must not
//! admit them. Ordinary closures live on the *operational* carrier
//! (`ir::RuntimeValue::ClosureRef`) and reach this one only through the checked,
//! fail-closed projection in [`crate::canonical`], which proves a whole graph
//! closure-free **before** any byte, hash, or slot exists.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

/// A Ken value.  Scalars are immediate; compounds are content-addressed.
///
/// ⛔ `Clone` is **not** derived — see the hand-written iterative
/// [`Clone`] impl below. The derived one recursed through the nested child
/// collections and overflowed the host stack on a deep value; `Drop` and
/// [`fmt::Debug`] have the same hazard and the same treatment.
///
/// ⛔ **`Debug` is not derived either** (`AC-V11`), and that is the one most
/// easily re-added by reflex: a `#[derive(Debug)]` here compiles, reads as
/// harmless, and silently restores a host-recursive traversal reachable from
/// every `{:?}` in a panic handler or log line.
///
/// ⚠ The recursive **child positions** of this enum (`args`, `fields`,
/// `elements`, and `Map`'s entry values) are governed by the closed allow-list
/// in `canonical::child_positions`. Giving one of them reference / handle /
/// arena / slot / index indirection, or interior mutation, **will not compile**
/// — that is deliberate, and it is what keeps the unrepresentability of cycles
/// on this carrier from silently lapsing.
///
/// ⛔ **`PartialEq`/`Eq`/`PartialOrd`/`Ord`/`Hash` are NOT derived, and that is
/// `D3`.** They were, and they were **unsound** — independently of closures.
/// Two `BigInt`s `{limbs: [5]}` and `{limbs: [5, 0]}` encode to *identical*
/// canonical bytes (`minimal_limbs` strips trailing zero limbs) and compared
/// **unequal** under the derive; two NFC-distinct spellings of one `String` did
/// the same. ⇒ The derived relations **disagreed with canonical identity**,
/// which is exactly what `AC-V8` forbids.
///
/// ⭐ Equality, order and hash are therefore exposed **only** on
/// [`CanonicalWitness`], which *is* the canonical bytes. Agreement is
/// **definitional** rather than asserted — there is no second definition of
/// identity to keep in step — and comparison is **depth-total** because the
/// bytes come from P1's iterative encoder and comparing a flat `Vec<u8>`
/// recurses not at all (`AC-V12`).
///
/// # `AC-V4` — the forbidden capabilities are UNREACHABLE on this carrier
///
/// ⭐ **The claim is *reachability*, not the absence of a caller today**, so
/// every control below must fail to **compile**. ⭐ And each pins the **trait
/// implementation** rather than one operator spelling: a bound check
/// `fn requires_eq<T: PartialEq>` cannot be evaded by writing `a.eq(b)`,
/// `PartialEq::eq(a, b)`, or an inherent method, because the only way to make
/// it compile is to supply the impl — which *is* the forbidden capability.
///
/// ⛔ **The `EXXXX` codes below are DOCUMENTATION, not a check — measured, not
/// assumed.** Rewriting one block's `compile_fail,E0277` to `compile_fail,E0308`
/// — a code that block cannot possibly produce — left the doc-test **green**,
/// so rustdoc is not binding the annotation on this toolchain. They are kept
/// because they tell a reader which error is expected, and flagged here because
/// a fence that *looks* like a pin and is not one is worse than no fence.
///
/// ⇒ **MEASURED:** each block fails to compile, for some reason.
/// **CLAIMED:** it fails *because the trait impl is absent*.
/// **THE GAP:** closed by the **sibling**, not by the code annotation — the
/// sibling shares every import, helper and constructor, so a malformed fixture
/// reddens *it* instead of silently greening the negatives. The `Value::Closure`
/// block additionally has a direct non-vacuity control: substituting a variant
/// that *does* exist makes it compile, and the doc-test then fails.
///
/// ⛔ **An ordinary closure cannot even be NAMED here (`D1`)** — the variant
/// does not exist, so there is no value for a comparison to be about:
///
/// ```compile_fail,E0599
/// use ken_runtime::Value;
/// let _closure = Value::Closure { captured: vec![Value::Bool(true)] };
/// ```
///
/// **No structural equality (`D3`):**
///
/// ```compile_fail,E0277
/// use ken_runtime::Value;
/// fn requires_eq<T: PartialEq>(_: &T) {}
/// let v = Value::Record { type_id: 1, fields: vec![Value::Bool(true)] };
/// requires_eq(&v);
/// ```
///
/// **No ordering (`D3`):**
///
/// ```compile_fail,E0277
/// use ken_runtime::Value;
/// fn requires_ord<T: Ord>(_: &T) {}
/// let v = Value::Record { type_id: 1, fields: vec![Value::Bool(true)] };
/// requires_ord(&v);
/// ```
///
/// **No canonical hash (`D3`):**
///
/// ```compile_fail,E0277
/// use ken_runtime::Value;
/// fn requires_hash<T: std::hash::Hash>(_: &T) {}
/// let v = Value::Record { type_id: 1, fields: vec![Value::Bool(true)] };
/// requires_hash(&v);
/// ```
///
/// ⭐ **The sibling that MUST compile — without it, every block above is green
/// whether it failed for the stated reason or because the fixture was
/// malformed.** ⛔ A `compile_fail` block passes for *any* compilation error,
/// including a mistyped path or a missing import, so a negative-only set of
/// controls establishes nothing. This block has the same imports and the same
/// construction; the only difference is that the subject of each bound check is
/// the sealed witness. It also **runs**, so the capability is shown to be
/// genuinely available rather than merely well-typed:
///
/// ```
/// use ken_runtime::Value;
/// use ken_runtime::canonical::CanonicalWitness;
/// fn requires_eq<T: PartialEq>(_: &T) {}
/// fn requires_ord<T: Ord>(_: &T) {}
/// fn requires_hash<T: std::hash::Hash>(_: &T) {}
/// let v = Value::Record { type_id: 1, fields: vec![Value::Bool(true)] };
/// let w = CanonicalWitness::of(&v);
/// requires_eq(&w);
/// requires_ord(&w);
/// requires_hash(&w);
/// assert_eq!(w, CanonicalWitness::of(&v));
/// assert_ne!(w, CanonicalWitness::of(&Value::Bool(true)));
/// ```
pub enum Value {
    // --- immediate scalars (§1, §5 table) ---
    Bool(bool),
    Char(char),
    Float(u64),   // f64 bits; -0.0 ≠ +0.0 by bit pattern (design doc §1.1)
    Float32(u32), // f32 bits
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    SmallInt(i64), // Int within i64 range (§1 fast path)
    SmallDecimal {
        coefficient: i64,
        exponent: i32,
    },

    // --- interned compounds (§2, §5 table) ---
    /// Arbitrary-precision integer beyond i64 (§1 overflow path).
    BigInt {
        sign: Sign,
        limbs: Vec<u64>, // minimal-limb, LE (design doc §1.10)
    },
    /// Big Decimal (coefficient beyond i64 fast path, design doc §1.10.1).
    BigDecimal {
        sign: Sign,
        coefficient: Vec<u64>,
        exponent: i32,
    },
    /// Constructor application — `data` kind (design doc §1.2).
    Constructor {
        constructor_id: u32,
        args: Vec<Value>,
    },
    /// Named-field record — Σ-type (design doc §1.3).
    Record {
        type_id: u32,
        fields: Vec<Value>, // declaration order (normative)
    },
    /// NFC-normalized Unicode string (design doc §1.4 — K3 must normalize).
    String(String),
    /// Opaque byte sequence (design doc §1.5).
    Bytes(Vec<u8>),
    /// Indexed sequence (design doc §1.6).
    Array {
        elem_type_id: u32,
        elements: Vec<Value>,
    },
    /// Key-value mapping; keys stored as canonical bytes for lexicographic order
    /// (design doc §1.7).
    Map {
        key_type_id: u32,
        value_type_id: u32,
        entries: BTreeMap<Vec<u8>, Value>,
    },
    /// Unordered set; elements stored as canonical bytes (design doc §1.8).
    Set {
        elem_type_id: u32,
        elements: BTreeSet<Vec<u8>>,
    },
    // ⛔ **No `Closure` variant, and this absence is the deliverable.**
    // `41 §2.1` forbids ordinary closures structural equality, ordering and
    // canonical hashing, and a variant here would have granted all three the
    // moment anything derived them. An ordinary closure is
    // `ir::RuntimeValue::ClosureRef`; a future `FrozenClosure` /
    // `StaticCallableRef` is a *separate explicit type*, never a re-added arm.

    // --- special (§6) ---
    /// Third truth value: the result of an open verification hole.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sign {
    NonNegative = 0,
    Negative = 1,
}

impl Value {
    /// Returns `true` iff this value is an immediate scalar (never interned).
    pub fn is_immediate(&self) -> bool {
        matches!(
            self,
            Value::Bool(_)
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
        )
    }

    /// Returns `true` iff this value is a compound (must be interned).
    pub fn is_compound(&self) -> bool {
        !self.is_immediate() && !matches!(self, Value::Unknown)
    }
}

/// Detach every child value of `value`, moving them onto `out`.
///
/// Leaves `value` childless, so its own teardown is O(1) and cannot recurse.
///
/// ⛔ **Exhaustive over every variant with no `_` arm.** A new variant carrying a
/// child position fails to compile until it is handled here, so `Drop` cannot
/// silently regain a recursive leg.
fn detach_children(value: &mut Value, out: &mut Vec<Value>) {
    match value {
        Value::Constructor { args: kids, .. }
        | Value::Record { fields: kids, .. }
        | Value::Array { elements: kids, .. } => out.append(kids),

        Value::Map { entries, .. } => out.extend(std::mem::take(entries).into_values()),

        // No child *values*: `BigInt`/`BigDecimal` hold limbs, `String`/`Bytes`
        // hold flat data, `Set` holds already-canonical element bytes, and the
        // scalars are immediate. Every one of these drops in O(1) depth.
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
        | Value::Unknown => {}
    }
}

/// Reassemble a clone of `proto`, taking its child values from `kids`.
///
/// ⛔ **Exhaustive over every variant with no `_` arm.** Every field that is
/// *not* a child position is cloned directly here; each of those clones is flat
/// (limbs, bytes, string data, canonical set elements) and so cannot recurse on
/// value depth. Leaf variants ignore `kids`, which is empty for them.
fn rebuild(proto: &Value, kids: Vec<Value>) -> Value {
    match proto {
        // --- child-bearing compounds: children come from `kids` ---
        Value::Constructor { constructor_id, .. } => Value::Constructor {
            constructor_id: *constructor_id,
            args: kids,
        },
        Value::Record { type_id, .. } => Value::Record {
            type_id: *type_id,
            fields: kids,
        },
        Value::Array { elem_type_id, .. } => Value::Array {
            elem_type_id: *elem_type_id,
            elements: kids,
        },
        // Keys are already-canonical bytes and are cloned flat; zipping against
        // `BTreeMap::keys()` is sound because the children were pushed in that
        // same iteration order.
        Value::Map {
            key_type_id,
            value_type_id,
            entries,
        } => Value::Map {
            key_type_id: *key_type_id,
            value_type_id: *value_type_id,
            entries: entries.keys().cloned().zip(kids).collect(),
        },

        // --- childless: flat clones ---
        Value::BigInt { sign, limbs } => Value::BigInt {
            sign: *sign,
            limbs: limbs.clone(),
        },
        Value::BigDecimal {
            sign,
            coefficient,
            exponent,
        } => Value::BigDecimal {
            sign: *sign,
            coefficient: coefficient.clone(),
            exponent: *exponent,
        },
        Value::String(s) => Value::String(s.clone()),
        Value::Bytes(b) => Value::Bytes(b.clone()),
        Value::Set {
            elem_type_id,
            elements,
        } => Value::Set {
            elem_type_id: *elem_type_id,
            elements: elements.clone(),
        },
        Value::Bool(v) => Value::Bool(*v),
        Value::Char(v) => Value::Char(*v),
        Value::Float(v) => Value::Float(*v),
        Value::Float32(v) => Value::Float32(*v),
        Value::Int8(v) => Value::Int8(*v),
        Value::Int16(v) => Value::Int16(*v),
        Value::Int32(v) => Value::Int32(*v),
        Value::Int64(v) => Value::Int64(*v),
        Value::UInt8(v) => Value::UInt8(*v),
        Value::UInt16(v) => Value::UInt16(*v),
        Value::UInt32(v) => Value::UInt32(*v),
        Value::UInt64(v) => Value::UInt64(*v),
        Value::SmallInt(v) => Value::SmallInt(*v),
        Value::SmallDecimal {
            coefficient,
            exponent,
        } => Value::SmallDecimal {
            coefficient: *coefficient,
            exponent: *exponent,
        },
        Value::Unknown => Value::Unknown,
    }
}

/// Iterative teardown (`D3`).
///
/// ⛔ **Drop cannot return an error**, so a total encoder does not make
/// deallocation total: automatic drop glue recurses through the nested
/// `Vec<Value>` / `BTreeMap<_, Value>` owners, and a value shallow enough to
/// construct can overflow while being *dropped*. This dismantles the tree
/// against an explicit heap worklist instead, so host-stack usage is O(1) in
/// depth.
///
/// **The traversal is depth-first, not breadth-first.** `detach_children` moves
/// a node's children onto `pending`, and `Vec::pop` takes the most recently
/// pushed, so the walk descends the *last* child first and only then unwinds to
/// its siblings — a LIFO worklist, not a FIFO frontier. ⚠ That distinction is
/// part of the mechanism contract rather than a wording preference: a LIFO
/// worklist holds the unvisited siblings along the current root-to-node path,
/// whereas a FIFO frontier holds an entire level of the tree. Those are
/// different live-frontier memory bounds, and neither dominates the other for
/// every shape — so a maintainer reasoning about teardown memory needs to know
/// which one this is.
impl Drop for Value {
    fn drop(&mut self) {
        let mut pending: Vec<Value> = Vec::new();
        detach_children(self, &mut pending);
        while let Some(mut child) = pending.pop() {
            detach_children(&mut child, &mut pending);
            // `child` is childless now, so its own drop at end of scope is
            // shallow and re-enters `detach_children` exactly once more.
        }
    }
}

/// Iterative deep clone (`D3`).
///
/// `Clone` is the one **postorder** traversal here — a parent cannot be built
/// until its children exist — so it uses pending parent frames plus a
/// completed-children buffer. ⚠ This is deliberately *not* the same machine as
/// the encoder's streaming pre-order emitter; fusing them would be wrong.
impl Clone for Value {
    fn clone(&self) -> Value {
        enum Job<'a> {
            /// Expand this value: push its frame, then its children.
            Visit(&'a Value),
            /// Its `children` clones are the last `children` entries of `done`.
            Finish { proto: &'a Value, children: usize },
        }

        let mut jobs: Vec<Job<'_>> = vec![Job::Visit(self)];
        let mut done: Vec<Value> = Vec::new();

        while let Some(job) = jobs.pop() {
            match job {
                Job::Visit(value) => match value {
                    Value::Constructor { args: kids, .. }
                    | Value::Record { fields: kids, .. }
                    | Value::Array { elements: kids, .. } => {
                        jobs.push(Job::Finish {
                            proto: value,
                            children: kids.len(),
                        });
                        // Reversed: LIFO pops restore declaration order, so the
                        // completed clones land in `done` in that order too.
                        for kid in kids.iter().rev() {
                            jobs.push(Job::Visit(kid));
                        }
                    }
                    Value::Map { entries, .. } => {
                        jobs.push(Job::Finish {
                            proto: value,
                            children: entries.len(),
                        });
                        for val in entries.values().rev() {
                            jobs.push(Job::Visit(val));
                        }
                    }
                    // Childless: clone flat, no frame needed.
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
                    | Value::Unknown => done.push(rebuild(value, Vec::new())),
                },
                Job::Finish { proto, children } => {
                    let kids = done.split_off(done.len() - children);
                    done.push(rebuild(proto, kids));
                }
            }
        }

        done.pop()
            .expect("the traversal assembles exactly one root clone")
    }
}

/// One unit of pending rendering work for the iterative [`fmt::Debug`] impl
/// (`AC-V11` / `AC-P3a`).
///
/// ⭐ **This is P1's *encoder* shape, deliberately — not `Clone`'s.** The three
/// existing traversals are not interchangeable, and picking the wrong one to
/// mirror would have cost a postorder fold this rendering does not need:
///
/// | traversal | receiver | needs a `Finish` fold? |
/// |---|---|---|
/// | `Drop` | `&mut self`, destructive | no — plain LIFO |
/// | `Clone` | `&self`, **constructive** | **yes** — a parent cannot be built before its children |
/// | `canonical::encode_canonical` | `&self`, **streaming emit** | no — headers are length/arity-prefixed |
/// | **this** | `&self`, **streaming emit** | **no** — see below |
///
/// A parent's rendered text never depends on a child's, exactly as with the
/// canonical encoder: the opening text is written on the way *down*, and the
/// closing delimiter is scheduled as a pending literal that pops after the
/// children. So one explicit work stack replaces host recursion with **no
/// completed-children buffer** — the `Lit` arm is what the encoder's
/// [`Raw`](crate::canonical) arm is for it.
enum DebugStep<'a> {
    /// Render this value's own header, then push its children.
    Val(&'a Value),
    /// Emit this literal delimiter text. Scheduled *before* a node's children
    /// are pushed, so LIFO pops it *after* them — that is what closes brackets
    /// without a postorder fold.
    Lit(&'static str),
    /// Emit a `Map` key's already-canonical bytes. Flat: keys are `Vec<u8>`,
    /// never `Value`, so this arm cannot recurse on value depth.
    Key(&'a [u8]),
}

/// Push `kids` so they pop in declaration order, comma-separated.
///
/// ⚠ The reversal is the same LIFO discipline `Clone` and the encoder document:
/// `Vec::pop` takes the most recently pushed, so pushing in reverse restores
/// declaration order on the way out.
fn push_debug_children<'a>(kids: &'a [Value], stack: &mut Vec<DebugStep<'a>>) {
    for (i, kid) in kids.iter().enumerate().rev() {
        stack.push(DebugStep::Val(kid));
        if i > 0 {
            stack.push(DebugStep::Lit(", "));
        }
    }
}

/// Write `value`'s own opening text, then push its children and its closer.
///
/// ⛔ **Exhaustive over every variant with no `_` arm** — the same discipline
/// `detach_children`, `rebuild` and `canonical::encode_header` carry, and for
/// the same reason: a new variant carrying a child position fails to compile
/// until it is handled here, so `Debug` cannot silently regain a recursive leg.
///
/// ⚠ **Honest residual:** exhaustiveness forces a *new variant* to get an arm;
/// it does not force that arm to route its children through `stack` rather than
/// recursing. That gap is closed behaviourally instead, by the depth controls in
/// `tests/value_depth_totality.rs`, which render at `D` out of process and
/// assert the output actually scales with `D`.
fn debug_header<'a>(
    value: &'a Value,
    f: &mut fmt::Formatter<'_>,
    stack: &mut Vec<DebugStep<'a>>,
) -> fmt::Result {
    match value {
        // --- child-bearing compounds: opening text now, closer scheduled ---
        Value::Constructor {
            constructor_id,
            args,
        } => {
            write!(f, "Constructor {{ constructor_id: {constructor_id}, args: [")?;
            stack.push(DebugStep::Lit("] }"));
            push_debug_children(args, stack);
            Ok(())
        }

        Value::Record { type_id, fields } => {
            write!(f, "Record {{ type_id: {type_id}, fields: [")?;
            stack.push(DebugStep::Lit("] }"));
            push_debug_children(fields, stack);
            Ok(())
        }

        Value::Array {
            elem_type_id,
            elements,
        } => {
            write!(f, "Array {{ elem_type_id: {elem_type_id}, elements: [")?;
            stack.push(DebugStep::Lit("] }"));
            push_debug_children(elements, stack);
            Ok(())
        }

        Value::Map {
            key_type_id,
            value_type_id,
            entries,
        } => {
            write!(
                f,
                "Map {{ key_type_id: {key_type_id}, value_type_id: {value_type_id}, entries: {{"
            )?;
            stack.push(DebugStep::Lit("} }"));
            for (i, (key, val)) in entries.iter().enumerate().rev() {
                stack.push(DebugStep::Val(val));
                stack.push(DebugStep::Lit(": "));
                stack.push(DebugStep::Key(key.as_slice()));
                if i > 0 {
                    stack.push(DebugStep::Lit(", "));
                }
            }
            Ok(())
        }

        // --- childless: rendered flat, cannot recurse on value depth ---
        // `BigInt`/`BigDecimal` hold limbs, `String`/`Bytes` hold flat data,
        // `Set` holds already-canonical element bytes, and the scalars are
        // immediate. Each delegates to the derived `Debug` of a non-`Value` type.
        Value::BigInt { sign, limbs } => {
            write!(f, "BigInt {{ sign: {sign:?}, limbs: {limbs:?} }}")
        }
        Value::BigDecimal {
            sign,
            coefficient,
            exponent,
        } => write!(
            f,
            "BigDecimal {{ sign: {sign:?}, coefficient: {coefficient:?}, exponent: {exponent:?} }}"
        ),
        Value::String(s) => write!(f, "String({s:?})"),
        Value::Bytes(b) => write!(f, "Bytes({b:?})"),
        Value::Set {
            elem_type_id,
            elements,
        } => write!(
            f,
            "Set {{ elem_type_id: {elem_type_id}, elements: {elements:?} }}"
        ),
        Value::Bool(v) => write!(f, "Bool({v:?})"),
        Value::Char(v) => write!(f, "Char({v:?})"),
        Value::Float(v) => write!(f, "Float({v:?})"),
        Value::Float32(v) => write!(f, "Float32({v:?})"),
        Value::Int8(v) => write!(f, "Int8({v:?})"),
        Value::Int16(v) => write!(f, "Int16({v:?})"),
        Value::Int32(v) => write!(f, "Int32({v:?})"),
        Value::Int64(v) => write!(f, "Int64({v:?})"),
        Value::UInt8(v) => write!(f, "UInt8({v:?})"),
        Value::UInt16(v) => write!(f, "UInt16({v:?})"),
        Value::UInt32(v) => write!(f, "UInt32({v:?})"),
        Value::UInt64(v) => write!(f, "UInt64({v:?})"),
        Value::SmallInt(v) => write!(f, "SmallInt({v:?})"),
        Value::SmallDecimal {
            coefficient,
            exponent,
        } => write!(
            f,
            "SmallDecimal {{ coefficient: {coefficient:?}, exponent: {exponent:?} }}"
        ),
        Value::Unknown => f.write_str("Unknown"),
    }
}

/// Iterative rendering (`AC-V11`).
///
/// ⛔ **`Debug` is not derived**, because the derived impl recursed through the
/// nested child collections and aborted the process on a deep value. That is a
/// worse failure than it first reads: every *other* deep traversal left on this
/// carrier is reached from a deliberate call — an identity comparison, an encode
/// — whereas `{:?}` is reached from a **panic handler, a log line, or an
/// `assert_eq!` failure message**. ⇒ The abort fires while a maintainer is
/// diagnosing something else, and it destroys the diagnostic being produced.
///
/// ⭐ **The mechanism, which is the claim — not any particular depth.** The only
/// stack frames here are `fmt` → `debug_header`, one deep, for *every* node: a
/// node's children are pushed onto the heap-allocated `stack` and popped by the
/// same loop, never visited by a nested call. Host-stack usage is therefore
/// O(1) in value depth, and depth is bounded by allocation — an ordinary
/// resource boundary — exactly as it is for `Clone`, `Drop` and the canonical
/// encoder. ⛔ There is no `MAX_DEPTH`.
///
/// ⚠ Corroboration, **not** the pin: the landed derived impl was measured dying
/// of stack overflow at `D = 131072` out of process, and the replacement returns
/// at that same `D`. A finite probe supports a structural claim; it does not
/// constitute one.
///
/// ⚠ **The rendered text is unspecified and must not be pinned.** It is kept
/// close to the derived rendering as a courtesy to existing logs, but the claim
/// under test is *does it return*, not *what does it print*.
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stack: Vec<DebugStep<'_>> = vec![DebugStep::Val(self)];
        while let Some(step) = stack.pop() {
            match step {
                DebugStep::Lit(text) => f.write_str(text)?,
                DebugStep::Key(bytes) => write!(f, "{bytes:?}")?,
                DebugStep::Val(value) => debug_header(value, f, &mut stack)?,
            }
        }
        Ok(())
    }
}
