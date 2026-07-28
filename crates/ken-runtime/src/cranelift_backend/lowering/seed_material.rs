//! **`RT-FNSPLIT-B2F` `D3` — the artifact-static seed material, minted.**
//!
//! `B2R` declared `AbiCarrier::GroundValueCarrier` as
//! `AbiOwnership::BorrowedForActivation` from `AbiStorageOwner::ArtifactStatic`
//! and **deliberately did not mint anything**. This module is the counterpart:
//! it materializes owned, read-only data objects into the compiled artifact
//! *before execution begins*, so that a seed capture has something real to
//! borrow from.
//!
//! ⛔ **The material this module mints is NOT the seed environment.**
//! `Lowering<'a>` holds `seed_env: &'a NativeSeedEnvironment` — a borrow that
//! exists only for the duration of compilation — while `CompiledModule<M>` has
//! no lifetime parameter, so nothing borrowed can reach the artifact (the
//! compiler refuses it, and
//! `escaping_a_source_borrow_into_the_compiled_artifact_does_not_typecheck`
//! pins exactly that). ⇒ A runtime activation **cannot** borrow the seed
//! environment. What it borrows is the encoded copy this module writes into the
//! module's data section, which outlives every activation because it is part of
//! the artifact rather than part of the compiler.
//!
//! ## ⭐ Why the objects are declared NOT writable
//!
//! `Linkage::Local` with `writable = false` is how `BorrowedForActivation` +
//! `ArtifactStatic` stop being prose. A borrower that cannot write cannot
//! reclaim, cannot mutate, and cannot hand ownership on — so the two declared
//! modes become a property of the **declaration** rather than a claim about the
//! emitter's good behaviour. ⛔ Flipping that flag is an ABI change, not a
//! tuning knob.
//!
//! ## ⚠ What this module does NOT establish
//!
//! **MEASURED:** every entry of the seed environment is encoded and minted as
//! one read-only artifact-static data object, and the encoding is total over
//! the whole `RuntimeGroundValue` family.
//! **CLAIMED:** artifact-static seed material exists for every seed capture the
//! artifact can perform.
//! **THE GAP:** ⛔ that a capture *reads* it. Minting is this module's; the
//! consumption seam is `Lowering::lower_seed_capture`, and a mint with no
//! reader is a data object the linker may drop. ⚠ The instrument that
//! distinguishes those two worlds is a **mutation of the minted bytes** — if
//! the observed answer does not move, the emitted code is not borrowing this
//! material regardless of how many objects were minted.

use super::*;

use cranelift_module::{DataDescription, DataId};

/// **The encoding tags.** One `u64` word each, little-endian, because every
/// field in this format is a word — see the module-level layout note.
///
/// ⛔ Exhaustive over `RuntimeGroundValue` by construction: `encode_into` matches
/// the enum without a `_` arm, so a new ground-value variant is a compile error
/// here rather than a value that silently encodes as something else.
mod tag {
    pub(super) const BOOL: u64 = 0;
    pub(super) const INT_SMALL: u64 = 1;
    pub(super) const INT_BIG: u64 = 2;
    pub(super) const BYTES: u64 = 3;
    pub(super) const STRING: u64 = 4;
    pub(super) const CONSTRUCTOR: u64 = 5;
    pub(super) const RECORD: u64 = 6;
}

/// Byte offset of a value's first payload word — immediately past the tag word.
///
/// ⭐ A constant rather than a computed offset **because the tag is always one
/// word wide for every variant.** Making the header variable-width would mean
/// the reader had to decode before it could address, which is the property this
/// layout exists to avoid.
pub(in crate::cranelift_backend) const SEED_PAYLOAD_OFFSET: i32 = 8;

/// Alignment of every minted object, in bytes.
///
/// ⚠ Matches `AbiCarrier::GroundValueCarrier`'s declared `align_bytes` (8). The
/// two are not derived from one another — `abi.rs` declares the *carrier's*
/// alignment and this declares the *material's* — so they are checked against
/// each other in `seed_material_alignment_matches_the_declared_carrier`, not
/// assumed equal because both happen to read `8`.
const SEED_ALIGN_BYTES: u64 = 8;

/// The deepest nesting `encode_into` will follow before failing closed.
///
/// ⛔ A cap, not a budget: exceeding it is an `Unsupported` rejection, never a
/// truncated encoding. A truncated encoding would be a *wrong* artifact-static
/// value that the reader could not tell from a right one.
const MAX_SEED_DEPTH: usize = 32;

/// The largest single encoded object, in bytes.
///
/// ⚠ Boundary-tested at the limit and at limit+1 rather than at a typical
/// magnitude, because the failure mode a size cap exists to prevent only
/// appears near the cap.
const MAX_SEED_BYTES: usize = 1 << 20;

/// **`AC-2` — what the compiled module ACTUALLY contains, data-object half.**
///
/// ⭐ **This is the instrument that carries the population claim for minted
/// data**, and it exists because the source-text census in `control.rs` cannot:
/// that census is a needle list whose default branch is *"needle not found ⇒
/// nothing emitted"*, so it fails **open** for every emission spelling nobody
/// enumerated — which is exactly how an entire data-object population stayed
/// invisible across three separate repairs of it.
///
/// ⛔ **The two instruments are not corroboration and must not be read as such.**
/// This counter observes what *is there*; the census searches for what someone
/// expected. Only one of them can see an unanticipated spelling.
///
/// Records `(declared, defined)` for the most recent compile on this thread.
#[cfg(test)]
thread_local! {
    static B2F_SEED_MATERIAL_EMISSION: std::cell::Cell<(usize, usize)> =
        const { std::cell::Cell::new((0, 0)) };
}

/// The `(declared, defined)` artifact-static object counts from the most recent
/// compile.
#[cfg(test)]
pub(in crate::cranelift_backend) fn b2f_last_seed_material_emission() -> (usize, usize) {
    B2F_SEED_MATERIAL_EMISSION.with(std::cell::Cell::get)
}

/// **`AC-12` — how many reads from artifact-static storage the emitter issued.**
///
/// ⭐ **This is the ownership-mode control, and it is a count of EMITTED LOADS
/// rather than a re-reading of a declaration.** `AbiCarrier::ownership` and
/// `storage_owner` are `const fn`s over a closed enum; an assertion that reads a
/// mode back out of them re-measures the declaration and discharges nothing.
/// What `AC-12` wants is whether the *emitted code* obeys it, and the observable
/// difference between obeying and ignoring `BorrowedForActivation` +
/// `ArtifactStatic` is whether the value arrives by a load from durable storage
/// or by a constant folded into the instruction stream.
///
/// ⚠ Monotone across a process, never reset by production. A reader compares two
/// readings around one compile; ⛔ it must not be read as a per-compile total.
#[cfg(test)]
thread_local! {
    static B2F_ARTIFACT_STATIC_LOADS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Reads issued from artifact-static storage so far on this thread.
#[cfg(test)]
pub(in crate::cranelift_backend) fn b2f_artifact_static_loads() -> usize {
    B2F_ARTIFACT_STATIC_LOADS.with(std::cell::Cell::get)
}

/// Every artifact-static seed object this module minted, keyed by the symbol
/// whose value it holds.
///
/// ⭐ **Keyed by the seed symbol, never by the mint ordinal.** The ordinal is an
/// artifact of `BTreeMap` iteration order; the symbol is what a capture names.
/// A map keyed on position would resolve a capture to whichever value happened
/// to sort into that slot — the identity-alias failure `B2O` removed from
/// `SemanticDescriptor` and `D1` refused to reintroduce for units.
#[derive(Default)]
pub(in crate::cranelift_backend) struct SeedMaterial {
    objects: BTreeMap<String, DataId>,
}

impl SeedMaterial {
    /// Resolve every minted object into one generated function.
    ///
    /// ⚠ Called once per emitted function that can read seed material. A unit
    /// body defined in `units.rs` builds its own `Function` and would need its
    /// own call — ⛔ **this does not make the material reachable from bodies it
    /// was never declared into.**
    pub(in crate::cranelift_backend) fn declare_in_func<M: Module>(
        &self,
        module: &mut M,
        func: &mut Function,
    ) -> SeedMaterialRefs {
        let bases = self
            .objects
            .iter()
            .map(|(symbol, id)| (symbol.clone(), module.declare_data_in_func(*id, func)))
            .collect();
        SeedMaterialRefs {
            bases,
            pointer_type: module.target_config().pointer_type(),
        }
    }
}

/// The minted objects, resolved into one generated function as addressable
/// global values.
///
/// ⭐ **Declared into the function once, up front, exactly as `C1`'s boundary
/// carrier helpers and the native-int helpers are.** A `DataId` is a
/// module-level identity and cannot be addressed from inside a body; the
/// `GlobalValue` is that identity resolved into one `Function`. Resolving it
/// lazily at each capture would need `&mut Module` at a point that holds only
/// the builder, which is the same reason those two helper families are
/// pre-declared.
pub(in crate::cranelift_backend) struct SeedMaterialRefs {
    bases: BTreeMap<String, cranelift_codegen::ir::GlobalValue>,
    pointer_type: cranelift_codegen::ir::Type,
}

impl SeedMaterialRefs {
    /// Load one symbol's scalar payload word out of artifact-static storage.
    ///
    /// ⛔ **This is the borrow.** The returned `ir::Value` is the result of a
    /// `load` from material the artifact owns, not an `iconst` folded at
    /// compile time — which is the entire difference `D3` exists to make.
    ///
    /// ⛔ `None` is a real answer: a symbol with no minted object has nothing to
    /// borrow, and fabricating a zero word would bind a capture to a value the
    /// environment never contained.
    pub(in crate::cranelift_backend) fn payload_word(
        &self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &str,
    ) -> Option<cranelift_codegen::ir::Value> {
        let base = *self.bases.get(symbol)?;
        let address = builder.ins().global_value(self.pointer_type, base);
        let word = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), address, SEED_PAYLOAD_OFFSET);
        // ⛔ Counted HERE, adjacent to the `load` that IS the borrow — not at
        // the call site in `lower_seed_capture`, and not on entry to this
        // method. ⭐ A counter one frame out measures that the capture path was
        // *entered*; this measures that a read from artifact-static storage was
        // *emitted*. An early return, a guard, or a folded fallback all change
        // this number precisely because the number is produced by the thing
        // under test.
        #[cfg(test)]
        B2F_ARTIFACT_STATIC_LOADS.with(|cell| cell.set(cell.get() + 1));
        Some(word)
    }

    /// How many minted objects are addressable from this function.
    #[cfg(test)]
    pub(in crate::cranelift_backend) fn len(&self) -> usize {
        self.bases.len()
    }

    /// ⛔ **No addressable material at all**, for the lowering harnesses that
    /// build a `Lowering` directly against an inert plan rather than through
    /// `compile_expr_into_module`.
    ///
    /// ⭐ **Empty is the honest value, not a stub.** Those harnesses never mint
    /// anything, so a seed capture attempted through one has genuinely nothing
    /// to borrow — and `payload_word` returning `None` makes it **fail closed**
    /// there exactly as it would in production. ⛔ A harness-only fallback that
    /// folded the compile-time value instead would give the test suite a
    /// second, quieter authority — the one thing `D3` removes.
    #[cfg(test)]
    pub(in crate::cranelift_backend) fn none_for_tests() -> Self {
        Self {
            bases: BTreeMap::new(),
            pointer_type: types::I64,
        }
    }
}

/// **`D3` — mint one read-only artifact-static object per seed-environment
/// entry.**
///
/// ⛔ **Minted from the environment, not from the plan.** Resolving which
/// symbols a unit actually captures would require an `origin -> expression`
/// lookup, and `AC-4` holds that count at exactly one (through
/// `retained_body_occurrence`). ⇒ The population is the environment itself,
/// which needs no lookup at all and cannot drift from what a capture can name.
///
/// ⚠ **The cost of that choice, stated rather than buried:** an environment
/// entry no capture reads is still minted. That is deliberate — the alternative
/// spends `AC-4`'s single lookup to save data-section bytes — but it means
/// `SeedMaterial::len()` is an upper bound on *read* material, not a count of
/// it.
pub(in crate::cranelift_backend) fn mint_seed_material<M: Module>(
    module: &mut M,
    seed_env: &NativeSeedEnvironment,
) -> Result<SeedMaterial, CraneliftBackendError> {
    let mut objects = BTreeMap::new();
    #[cfg(test)]
    let mut declared = 0usize;
    #[cfg(test)]
    let mut defined = 0usize;
    for (ordinal, (symbol, value)) in seed_env.values.iter().enumerate() {
        let encoded = encode_ground_value(value)?;
        // The symbol carries the dense ordinal purely so the linker sees
        // distinct names. ⛔ It is NOT an identity: nothing resolves material by
        // parsing this string, and `objects` is keyed by the seed symbol.
        let name = format!("ken_seed_{ordinal}");
        let id = module
            .declare_data(
                &name,
                Linkage::Local,
                // ⛔ Not writable. This is `BorrowedForActivation` +
                // `ArtifactStatic` made structural rather than asserted: an
                // activation that cannot write cannot reclaim or transfer.
                false,
                // Not thread-local. Artifact-static material is shared by every
                // activation on every thread; a per-thread copy would give each
                // borrower a different owner.
                false,
            )
            .map_err(|err| backend_module(err.to_string()))?;
        #[cfg(test)]
        {
            declared += 1;
        }
        let mut description = DataDescription::new();
        description.define(encoded.into_boxed_slice());
        description.set_align(SEED_ALIGN_BYTES);
        module
            .define_data(id, &description)
            .map_err(|err| backend_module(err.to_string()))?;
        // ⛔ Counted HERE, adjacent to `define_data`, and NOT beside the call in
        // an enclosing loop. ⭐ A counter incremented once per iteration
        // compares a collection's length to itself and stays green for every
        // mutation of the emission path — measured on this WP's unit counter,
        // where gating the definition off entirely left the control passing.
        #[cfg(test)]
        {
            defined += 1;
        }
        if objects.insert(symbol.clone(), id).is_some() {
            // ⛔ Fails closed. `seed_env.values` is a `BTreeMap`, so a duplicate
            // key is unrepresentable at the source — which means reaching this
            // arm indicates the environment is not the type it claims to be,
            // and silently keeping the last object would bind a capture to the
            // wrong value.
            return Err(backend_module(
                "two seed environment entries claim one symbol".to_string(),
            ));
        }
    }
    #[cfg(test)]
    B2F_SEED_MATERIAL_EMISSION.with(|cell| cell.set((declared, defined)));
    Ok(SeedMaterial { objects })
}

/// Encode one ground value into its artifact-static byte image.
///
/// ⭐ **Total over the whole `RuntimeGroundValue` family**, including the four
/// variants the frame recorded as having *no runtime representation at all*
/// (`Bytes`, `String`, `Constructor`, `Record` hold the compiler's own Rust
/// values and are specialized away by `Lowered`). Giving them a representation
/// is the substance of `D3`: a family the ABI's carrier doc says it can carry
/// must actually be carryable, or the carrier is overclaiming.
///
/// ⛔ **Self-describing and relocation-free.** Nested values are encoded inline
/// rather than as pointers to further objects, so one seed symbol is one object
/// and there is no second address space for a reader to get wrong.
fn encode_ground_value(
    value: &RuntimeGroundValue,
) -> Result<Vec<u8>, CraneliftBackendError> {
    let mut out = Vec::new();
    encode_into(value, 0, &mut out)?;
    if out.len() > MAX_SEED_BYTES {
        return Err(unsupported(
            "Closure",
            format!(
                "seed ground value encodes to {} bytes, over the {MAX_SEED_BYTES}-byte \
                 artifact-static limit",
                out.len()
            ),
        ));
    }
    Ok(out)
}

fn encode_into(
    value: &RuntimeGroundValue,
    depth: usize,
    out: &mut Vec<u8>,
) -> Result<(), CraneliftBackendError> {
    if depth > MAX_SEED_DEPTH {
        // ⛔ Rejects rather than truncating. A truncated encoding is a *wrong*
        // artifact-static value the reader cannot distinguish from a right one.
        return Err(unsupported(
            "Closure",
            format!("seed ground value nests deeper than {MAX_SEED_DEPTH} levels"),
        ));
    }
    // ⛔ Wildcard-free: a new `RuntimeGroundValue` variant must be given an
    // encoding here rather than inheriting another variant's by default.
    match value {
        RuntimeGroundValue::Bool(flag) => {
            push_word(out, tag::BOOL);
            push_word(out, u64::from(*flag));
        }
        RuntimeGroundValue::Int(crate::RuntimeIntV1::Small(small)) => {
            push_word(out, tag::INT_SMALL);
            // Two's complement, so the reader loads one `i64` with no decoding.
            push_word(out, *small as u64);
        }
        RuntimeGroundValue::Int(crate::RuntimeIntV1::Big { sign, limbs }) => {
            push_word(out, tag::INT_BIG);
            push_word(out, sign_code(*sign));
            push_word(out, limbs.len() as u64);
            for limb in limbs {
                push_word(out, *limb);
            }
        }
        RuntimeGroundValue::Bytes(bytes) => {
            push_word(out, tag::BYTES);
            push_word(out, bytes.len() as u64);
            push_padded(out, bytes);
        }
        RuntimeGroundValue::String(text) => {
            push_word(out, tag::STRING);
            push_word(out, text.len() as u64);
            push_padded(out, text.as_bytes());
        }
        RuntimeGroundValue::Constructor { constructor, args } => {
            push_word(out, tag::CONSTRUCTOR);
            push_word(out, constructor.len() as u64);
            push_padded(out, constructor.as_bytes());
            push_word(out, args.len() as u64);
            for arg in args {
                encode_into(arg, depth + 1, out)?;
            }
        }
        RuntimeGroundValue::Record { fields } => {
            push_word(out, tag::RECORD);
            push_word(out, fields.len() as u64);
            for (name, field) in fields {
                push_word(out, name.len() as u64);
                push_padded(out, name.as_bytes());
                encode_into(field, depth + 1, out)?;
            }
        }
    }
    // ⚠ Checked on the way out of every level, not only at the top: a value that
    // only exceeds the cap after a thousand shallow siblings would otherwise be
    // fully materialized in memory before anything rejected it.
    if out.len() > MAX_SEED_BYTES {
        return Err(unsupported(
            "Closure",
            format!("seed ground value exceeds the {MAX_SEED_BYTES}-byte artifact-static limit"),
        ));
    }
    Ok(())
}

/// The wire code for a big integer's sign.
///
/// ⛔ Written out rather than taken from the enum's `as u64` discriminant. The
/// discriminant is `values.rs`'s to change; this is the artifact's ABI, and
/// silently inheriting a renumbering there would reinterpret every minted big
/// integer's sign with nothing going red.
const fn sign_code(sign: crate::Sign) -> u64 {
    match sign {
        crate::Sign::NonNegative => 0,
        crate::Sign::Negative => 1,
    }
}

fn push_word(out: &mut Vec<u8>, word: u64) {
    out.extend_from_slice(&word.to_le_bytes());
}

/// Append `bytes`, then zero-pad to the next word boundary.
///
/// ⚠ The pad is part of the format, not slack: every field starts on a word
/// boundary, which is what lets a reader address the field after a byte payload
/// without tracking a running misalignment.
fn push_padded(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(bytes);
    let remainder = bytes.len() % 8;
    if remainder != 0 {
        out.resize(out.len() + (8 - remainder), 0u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reads the `index`-th word of an encoding.
    fn word(encoded: &[u8], index: usize) -> u64 {
        let start = index * 8;
        u64::from_le_bytes(encoded[start..start + 8].try_into().expect("eight bytes"))
    }

    /// **Promise class: normative compatibility vector.** These byte positions
    /// *are* the artifact's contract with its own reader — changing one is a
    /// contract decision, not a refactor.
    #[test]
    fn every_ground_value_variant_encodes_to_a_distinct_tag() {
        let cases = [
            (RuntimeGroundValue::Bool(true), tag::BOOL),
            (RuntimeGroundValue::Int(7i64.into()), tag::INT_SMALL),
            (
                RuntimeGroundValue::Int(crate::RuntimeIntV1::Big {
                    sign: crate::Sign::Negative,
                    limbs: vec![1, 2],
                }),
                tag::INT_BIG,
            ),
            (RuntimeGroundValue::Bytes(vec![1, 2, 3]), tag::BYTES),
            (RuntimeGroundValue::String("hi".to_string()), tag::STRING),
            (
                RuntimeGroundValue::Constructor {
                    constructor: "Mk".to_string(),
                    args: vec![RuntimeGroundValue::Bool(false)],
                },
                tag::CONSTRUCTOR,
            ),
            (
                RuntimeGroundValue::Record {
                    fields: vec![("f".to_string(), RuntimeGroundValue::Bool(false))],
                },
                tag::RECORD,
            ),
        ];
        let mut seen = BTreeSet::new();
        for (value, expected) in cases {
            let encoded = encode_ground_value(&value).expect("encodes");
            assert_eq!(
                word(&encoded, 0),
                expected,
                "{value:?} must encode under its own tag"
            );
            assert!(
                seen.insert(expected),
                "two variants share tag {expected}; a reader cannot tell them apart"
            );
            assert_eq!(
                encoded.len() % 8,
                0,
                "{value:?} must encode to whole words so the next field is addressable"
            );
        }
        assert_eq!(
            seen.len(),
            7,
            "the tag space must cover every RuntimeGroundValue variant"
        );
    }

    /// The scalar payload is at `SEED_PAYLOAD_OFFSET` for **both** scalar
    /// variants, which is what makes that a constant rather than a per-variant
    /// computation.
    #[test]
    fn scalar_payload_sits_at_the_declared_offset() {
        let flag = encode_ground_value(&RuntimeGroundValue::Bool(true)).expect("encodes");
        let small = encode_ground_value(&RuntimeGroundValue::Int((-9i64).into())).expect("encodes");
        let offset = usize::try_from(SEED_PAYLOAD_OFFSET).expect("non-negative offset") / 8;
        assert_eq!(word(&flag, offset), 1, "Bool(true) payload word");
        assert_eq!(
            word(&small, offset) as i64,
            -9,
            "Int payload must be two's complement so the reader needs no decoding"
        );
    }

    /// **The positive control for the depth cap.** ⛔ A rejection test that
    /// never sees an accepted case passes for any reason, including an encoder
    /// that rejects everything.
    #[test]
    fn nesting_is_accepted_below_the_cap_and_rejected_above_it() {
        fn nest(depth: usize) -> RuntimeGroundValue {
            let mut value = RuntimeGroundValue::Bool(true);
            for _ in 0..depth {
                value = RuntimeGroundValue::Constructor {
                    constructor: "S".to_string(),
                    args: vec![value],
                };
            }
            value
        }
        // At the limit: the innermost value sits at depth MAX_SEED_DEPTH.
        assert!(
            encode_ground_value(&nest(MAX_SEED_DEPTH)).is_ok(),
            "a value exactly at the depth cap must encode"
        );
        let over = encode_ground_value(&nest(MAX_SEED_DEPTH + 1));
        assert!(
            matches!(over, Err(CraneliftBackendError::Unsupported(_))),
            "a value past the depth cap must be rejected, never truncated: {over:?}"
        );
    }

    /// The size cap likewise fails closed, and likewise with a positive control
    /// immediately below it.
    #[test]
    fn size_is_accepted_below_the_cap_and_rejected_above_it() {
        // Two header words precede the payload, so this lands just inside.
        let under = RuntimeGroundValue::Bytes(vec![0u8; MAX_SEED_BYTES - 16]);
        assert!(
            encode_ground_value(&under).is_ok(),
            "a value just under the size cap must encode"
        );
        let over = RuntimeGroundValue::Bytes(vec![0u8; MAX_SEED_BYTES]);
        assert!(
            matches!(
                encode_ground_value(&over),
                Err(CraneliftBackendError::Unsupported(_))
            ),
            "a value past the size cap must be rejected, never truncated"
        );
    }

    /// ⭐ **The non-vacuity control for the whole encoder.** Two values that
    /// differ only inside a nested position must produce different bytes — an
    /// encoder that dropped nested payloads would satisfy every assertion above
    /// and fail this one.
    #[test]
    fn a_nested_difference_reaches_the_encoded_bytes() {
        let left = RuntimeGroundValue::Record {
            fields: vec![(
                "f".to_string(),
                RuntimeGroundValue::Constructor {
                    constructor: "Mk".to_string(),
                    args: vec![RuntimeGroundValue::Int(1i64.into())],
                },
            )],
        };
        let right = RuntimeGroundValue::Record {
            fields: vec![(
                "f".to_string(),
                RuntimeGroundValue::Constructor {
                    constructor: "Mk".to_string(),
                    args: vec![RuntimeGroundValue::Int(2i64.into())],
                },
            )],
        };
        assert_ne!(
            encode_ground_value(&left).expect("encodes"),
            encode_ground_value(&right).expect("encodes"),
            "a difference nested two levels down must be visible in the encoding"
        );
    }

    /// The material's declared alignment and the carrier's declared alignment
    /// are two separate declarations in two separate files. ⛔ They are checked
    /// against each other rather than assumed equal because both read `8`.
    #[test]
    fn seed_material_alignment_matches_the_declared_carrier() {
        assert_eq!(
            SEED_ALIGN_BYTES,
            u64::from(AbiCarrier::GroundValueCarrier.align_bytes()),
            "artifact-static material must be aligned for the carrier that addresses it"
        );
    }
}
