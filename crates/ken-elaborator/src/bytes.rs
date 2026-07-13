//! `Bytes` primitive + binary I/O registration (`38 §1`, `41`, `14 §5`).
//!
//! Registers `Bytes` (opaque immutable byte sequence), `String` (opaque, by-
//! construction valid UTF-8), and their core ops. The real `read_bytes`
//! surface operation is registered later by the prelude and contributes the
//! sole Bytes-layer effect-row seed.
//!
//! Pattern mirrors `numbers.rs`: declare types + ops as kernel primitives;
//! store the I/O op effect rows in `BytesEnv.io_effect_rows` so AC2/AC3 tests
//! derive their seed from the actual L6 binding (not a hand-fed literal).

use std::collections::HashMap;

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
    /// Effect rows for registered I/O ops (`36`/L5).
    ///
    /// Keyed by real op name (`"read_bytes"` → `[FS]`).
    /// AC2/AC3 tests derive their `infer_all` seed from this map — NOT from
    /// a hard-coded literal — so removing the L6 registration makes those
    /// tests fail (green-vs-green is structurally impossible).
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
    // The `io_effect_rows` seed below is a name-keyed static-analysis table
    // (`effects/infer.rs`), independent of the kernel `GlobalId` — it stays
    // valid regardless of where/how `read_bytes` is actually declared.

    // -- I/O effect row registry (real producers only) --
    let mut io_effect_rows: HashMap<String, EffectRow> = HashMap::new();
    io_effect_rows.insert("read_bytes".to_string(), EffectRow::singleton("FS"));

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
        io_effect_rows,
    })
}

/// Register the safe Bytes operations after `Option`, `Result`, and
/// `Utf8Error` have been installed by the prelude.
pub fn register_safe_bytes_ops(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
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
    let utf8_error_t = Term::indformer(lookup("Utf8Error")?, vec![]);
    let option = Term::indformer(lookup("Option")?, vec![]);
    let result = Term::indformer(lookup("Result")?, vec![]);
    let option_uint8 = Term::app(option.clone(), uint8_t);
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
        Term::pi(bytes_t, result_string),
    )?;
    Ok(())
}
