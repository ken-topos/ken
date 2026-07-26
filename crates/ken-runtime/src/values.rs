//! Value types — `spec/40-runtime/41-values.md §1–2,§5,§6`.
//!
//! Scalars are immediate (never interned). Compounds are content-addressed.
//! `Unknown` is the third truth value for partially-verified programs.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// A Ken value.  Scalars are immediate; compounds are content-addressed.
///
/// ⛔ `Clone` is **not** derived — see the hand-written iterative
/// [`Clone`] impl below. The derived one recursed through the nested child
/// collections and overflowed the host stack on a deep value; `Drop` has the
/// same hazard and the same treatment.
///
/// ⚠ The recursive **child positions** of this enum (`args`, `fields`,
/// `elements`, `captured`, and `Map`'s entry values) are governed by the closed
/// allow-list in `canonical::child_positions`. Giving one of them reference /
/// handle / arena / slot / index indirection, or interior mutation, **will not
/// compile** — that is deliberate, and it is what keeps the unrepresentability
/// of cycles on this carrier from silently lapsing.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    /// Closure — code pointer + full canonical captured environment (design doc §1.9).
    Closure {
        code_id: u64,
        captured: Vec<Value>, // in capture order; encoded inline (memcmp-exact)
    },

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
        | Value::Array { elements: kids, .. }
        | Value::Closure { captured: kids, .. } => out.append(kids),

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
        Value::Closure { code_id, .. } => Value::Closure {
            code_id: *code_id,
            captured: kids,
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
/// breadth-first onto an explicit heap stack instead, so host-stack usage is
/// O(1) in depth.
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
                    | Value::Array { elements: kids, .. }
                    | Value::Closure { captured: kids, .. } => {
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
