//! `Bytes` primitive + binary I/O registration (`38 §1`, `41`, `14 §5`).
//!
//! Registers `Bytes` (opaque immutable byte sequence), `String` (opaque, by-
//! construction valid UTF-8), and their core ops. The real `read_bytes`
//! surface operation and its effect row are registered later by the prelude.

use std::collections::{BTreeSet, HashMap};

use ken_kernel::env::PrimReduction;
use ken_kernel::{declare_postulate, declare_primitive, GlobalEnv, GlobalId, Level, Term};

use crate::effects::EffectRow;
use crate::error::ElabError;

/// All GlobalIds and I/O effect rows for the Bytes layer (`38 §1`, `41`).
pub struct BytesEnv {
    /// `Bytes : Type 0` (opaque immutable byte sequence).
    pub bytes_id: GlobalId,
    /// `String : Type 0` (opaque; by-construction valid UTF-8).
    pub string_id: GlobalId,
    /// `BytesRoundTripLaw : Ω₀` — oracle-tagged round-trip proposition
    /// (`38 §1.5`). AC5's `prove` obligation anchors here; the inductive
    /// proof is the L8 stdlib follow-on.
    pub bytes_round_trip_law_id: GlobalId,
    /// `bytes_to_list : Bytes → List UInt8` (`37 §2.6`, SUB-1).
    pub bytes_to_list_id: GlobalId,
    /// `list_to_bytes : List UInt8 → Bytes` (`37 §2.6`, SUB-1).
    pub list_to_bytes_id: GlobalId,
    /// `bytes_list_roundtrip : (bs : Bytes) →
    /// Equal Bytes (list_to_bytes (bytes_to_list bs)) bs`.
    pub bytes_list_roundtrip_id: GlobalId,
    /// `list_bytes_roundtrip : (xs : List UInt8) →
    /// Equal (List UInt8) (bytes_to_list (list_to_bytes xs)) xs`.
    pub list_bytes_roundtrip_id: GlobalId,
    /// The actual `trusted_base()` delta observed while installing SUB-1.
    /// Tests assert that this is exactly the four named ids above.
    pub structural_view_trusted_delta: Vec<GlobalId>,
    /// Legacy Bytes-layer effect-row registry (`36`/L5), intentionally empty.
    /// Real producers are registered only from their elaborated declarations
    /// in `ElabEnv.effect_rows`; this map cannot hand-feed an oracle.
    pub io_effect_rows: HashMap<String, EffectRow>,
}

/// Register the `Bytes` layer in `env`/`globals` and return a `BytesEnv`.
///
/// Must be called AFTER `register_numeric_env` (needs `Int` in globals).
pub fn register_bytes_env(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
) -> Result<BytesEnv, ElabError> {
    let type0 = Term::ty(Level::Zero);
    let omega0 = Term::omega(Level::Zero);

    // -- Bytes : Type 0 --
    let bytes_id = declare_primitive(env, vec![], type0.clone(), PrimReduction::OpaqueType)
        .map_err(|e| ElabError::Internal(format!("prim Bytes failed: {}", e)))?;
    globals.insert("Bytes".to_string(), bytes_id);

    // -- String : Type 0 --
    let string_id = declare_primitive(env, vec![], type0.clone(), PrimReduction::OpaqueType)
        .map_err(|e| ElabError::Internal(format!("prim String failed: {}", e)))?;
    globals.insert("String".to_string(), string_id);

    // Int is registered by the numeric tower before us.
    let int_id = globals
        .get("Int")
        .copied()
        .ok_or_else(|| ElabError::Internal("Int not registered before bytes layer".into()))?;

    let bytes_t = Term::const_(bytes_id, vec![]);
    let string_t = Term::const_(string_id, vec![]);
    let int_t = Term::const_(int_id, vec![]);

    // -- bytes_length : Bytes → Int --
    {
        let ty = Term::pi(bytes_t.clone(), int_t.clone());
        let id = declare_primitive(
            env,
            vec![],
            ty,
            PrimReduction::Op {
                symbol: "bytes_length",
            },
        )
        .map_err(|e| ElabError::Internal(format!("prim bytes_length failed: {}", e)))?;
        globals.insert("bytes_length".to_string(), id);
    }

    // -- bytes_concat : Bytes → Bytes → Bytes --
    {
        let ty = Term::pi(bytes_t.clone(), Term::pi(bytes_t.clone(), bytes_t.clone()));
        let id = declare_primitive(
            env,
            vec![],
            ty,
            PrimReduction::Op {
                symbol: "bytes_concat",
            },
        )
        .map_err(|e| ElabError::Internal(format!("prim bytes_concat failed: {}", e)))?;
        globals.insert("bytes_concat".to_string(), id);
    }

    // -- bytes_encode : String → Bytes (total) --
    {
        let ty = Term::pi(string_t.clone(), bytes_t.clone());
        let id = declare_primitive(
            env,
            vec![],
            ty,
            PrimReduction::Op {
                symbol: "bytes_encode",
            },
        )
        .map_err(|e| ElabError::Internal(format!("prim bytes_encode failed: {}", e)))?;
        globals.insert("bytes_encode".to_string(), id);
    }

    // The safe `Option`/`Result`-returning primitives are registered by
    // `register_safe_bytes_ops` after the prelude has declared those sum
    // types. `read_bytes` is declared for REAL in `prelude.rs` (FS-driver-build D1:
    // `Cap -> Bytes -> FS (Result Bytes IOError)`, a genuine kernel-rechecked
    // `view` reducing to a `Vis` node — `run_io`'s FS arm is the real driver,
    // `36 §2.1`). It can't be declared HERE: `register_bytes_env` runs before
    // `register_prelude`, so `ITree`/`Result`/`Cap`/`FSOp` don't exist yet.
    // No I/O row is installed here. `modules::register_effect_row` records
    // `read_bytes` from its final prelude declaration, so deleting `visits
    // [FS]` deletes the only producer-binding evidence.
    let io_effect_rows: HashMap<String, EffectRow> = HashMap::new();

    // -- BytesRoundTripLaw : Ω₀ (oracle-tagged, `38 §1.5`) --
    // Represents `∀ s : String, decode(encode s) = Ok s`.
    // Registered as a postulate proposition; the L8 stdlib provides the
    // inductive proof. AC5 elaborates `prove roundtrip : BytesRoundTripLaw`
    // and asserts the hole is dischargeable.
    let bytes_round_trip_law_id = declare_postulate(env, vec![], omega0)
        .map_err(|e| ElabError::Internal(format!("BytesRoundTripLaw failed: {}", e)))?;
    globals.insert("BytesRoundTripLaw".to_string(), bytes_round_trip_law_id);

    let _ = string_t;

    Ok(BytesEnv {
        bytes_id,
        string_id,
        bytes_round_trip_law_id,
        bytes_to_list_id: GlobalId(0),
        list_to_bytes_id: GlobalId(0),
        bytes_list_roundtrip_id: GlobalId(0),
        list_bytes_roundtrip_id: GlobalId(0),
        structural_view_trusted_delta: Vec::new(),
        io_effect_rows,
    })
}

/// Register the safe Bytes operations after `Option`, `Result`, and
/// `Utf8Error` have been installed by the prelude.
pub fn register_safe_bytes_ops(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    bytes_env: &mut BytesEnv,
) -> Result<(), ElabError> {
    let lookup = |name: &str| {
        globals
            .get(name)
            .copied()
            .ok_or_else(|| ElabError::Internal(format!("safe Bytes op: '{name}' not registered")))
    };
    let bytes_t = Term::const_(lookup("Bytes")?, vec![]);
    let string_t = Term::const_(lookup("String")?, vec![]);
    let int_t = Term::const_(lookup("Int")?, vec![]);
    let uint8_t = Term::const_(lookup("UInt8")?, vec![]);
    let list_id = lookup("List")?;
    let utf8_error_t = Term::indformer(lookup("Utf8Error")?, vec![]);
    let option = Term::indformer(lookup("Option")?, vec![]);
    let result = Term::indformer(lookup("Result")?, vec![]);
    let option_uint8 = Term::app(option.clone(), uint8_t.clone());
    let option_bytes = Term::app(option, bytes_t.clone());
    let result_string = Term::app(Term::app(result, utf8_error_t), string_t);

    let register = |env: &mut GlobalEnv,
                    globals: &mut HashMap<String, GlobalId>,
                    name: &'static str,
                    ty: Term|
     -> Result<(), ElabError> {
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: name })
            .map_err(|e| ElabError::Internal(format!("prim {name} failed: {e}")))?;
        globals.insert(name.to_string(), id);
        Ok(())
    };

    register(
        env,
        globals,
        "bytes_at",
        Term::pi(bytes_t.clone(), Term::pi(int_t.clone(), option_uint8)),
    )?;
    register(
        env,
        globals,
        "bytes_slice",
        Term::pi(
            bytes_t.clone(),
            Term::pi(int_t.clone(), Term::pi(int_t, option_bytes)),
        ),
    )?;
    register(
        env,
        globals,
        "bytes_decode",
        Term::pi(bytes_t.clone(), result_string),
    )?;

    // SUB-1's bounded structural view is installed only after `List` and
    // `UInt8` are both reachable here. Keep the trust accounting around this
    // block deliberately narrow: the older safe Bytes primitives above are the
    // baseline, and exactly this pair plus its two reasoning propositions may
    // enter the delta.
    let trusted_before: BTreeSet<_> = env.trusted_base().into_iter().collect();
    let list_uint8_t = Term::app(Term::indformer(list_id, vec![]), uint8_t);

    let bytes_to_list_id = declare_primitive(
        env,
        vec![],
        Term::pi(bytes_t.clone(), list_uint8_t.clone()),
        PrimReduction::Op {
            symbol: "bytes_to_list",
        },
    )
    .map_err(|e| ElabError::Internal(format!("prim bytes_to_list failed: {e}")))?;
    globals.insert("bytes_to_list".to_string(), bytes_to_list_id);

    let list_to_bytes_id = declare_primitive(
        env,
        vec![],
        Term::pi(list_uint8_t.clone(), bytes_t.clone()),
        PrimReduction::Op {
            symbol: "list_to_bytes",
        },
    )
    .map_err(|e| ElabError::Internal(format!("prim list_to_bytes failed: {e}")))?;
    globals.insert("list_to_bytes".to_string(), list_to_bytes_id);

    // Primitive operations are deliberately opaque to kernel conversion.
    // The inverse guarantees therefore live as explicit, named postulates:
    // one fixed trust cost per direction instead of an unbounded Axiom at each
    // consumer.
    let bytes_to_list = Term::const_(bytes_to_list_id, vec![]);
    let list_to_bytes = Term::const_(list_to_bytes_id, vec![]);
    let bytes_roundtrip_ty = Term::pi(
        bytes_t.clone(),
        Term::Eq(
            Box::new(bytes_t.clone()),
            Box::new(Term::app(
                list_to_bytes.clone(),
                Term::app(bytes_to_list.clone(), Term::var(0)),
            )),
            Box::new(Term::var(0)),
        ),
    );
    let bytes_list_roundtrip_id = declare_postulate(env, vec![], bytes_roundtrip_ty)
        .map_err(|e| ElabError::Internal(format!("bytes_list_roundtrip failed: {e}")))?;
    globals.insert("bytes_list_roundtrip".to_string(), bytes_list_roundtrip_id);

    let list_roundtrip_ty = Term::pi(
        list_uint8_t.clone(),
        Term::Eq(
            Box::new(list_uint8_t),
            Box::new(Term::app(
                bytes_to_list,
                Term::app(list_to_bytes, Term::var(0)),
            )),
            Box::new(Term::var(0)),
        ),
    );
    let list_bytes_roundtrip_id = declare_postulate(env, vec![], list_roundtrip_ty)
        .map_err(|e| ElabError::Internal(format!("list_bytes_roundtrip failed: {e}")))?;
    globals.insert("list_bytes_roundtrip".to_string(), list_bytes_roundtrip_id);

    let trusted_after: BTreeSet<_> = env.trusted_base().into_iter().collect();
    let structural_view_trusted_delta: Vec<_> =
        trusted_after.difference(&trusted_before).copied().collect();
    let expected_delta = BTreeSet::from([
        bytes_to_list_id,
        list_to_bytes_id,
        bytes_list_roundtrip_id,
        list_bytes_roundtrip_id,
    ]);
    let actual_delta: BTreeSet<_> = structural_view_trusted_delta.iter().copied().collect();
    if actual_delta != expected_delta {
        return Err(ElabError::Internal(format!(
            "SUB-1 trusted-base delta must be exactly the primitive pair and propositions: expected {expected_delta:?}, got {actual_delta:?}"
        )));
    }

    bytes_env.bytes_to_list_id = bytes_to_list_id;
    bytes_env.list_to_bytes_id = list_to_bytes_id;
    bytes_env.bytes_list_roundtrip_id = bytes_list_roundtrip_id;
    bytes_env.list_bytes_roundtrip_id = list_bytes_roundtrip_id;
    bytes_env.structural_view_trusted_delta = structural_view_trusted_delta;
    Ok(())
}
