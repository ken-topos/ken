//! `Bytes` primitive + binary I/O registration (`38 §1`, `41`, `14 §5`).
//!
//! Registers `Bytes` (opaque immutable byte sequence), `String` (opaque, by-
//! construction valid UTF-8), their core ops, and the effect-tracked I/O ops
//! (`read_bytes`/`write_bytes`/`send`/`recv`) with their declared effect rows.
//!
//! Pattern mirrors `numbers.rs`: declare types + ops as kernel primitives;
//! store the I/O op effect rows in `BytesEnv.io_effect_rows` so AC2/AC3 tests
//! derive their seed from the actual L6 binding (not a hand-fed literal).

use std::collections::HashMap;

use ken_kernel::{declare_postulate, declare_primitive, GlobalEnv, GlobalId, Level, Term};
use ken_kernel::env::PrimReduction;

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
    /// Keyed by op name (e.g. `"read_bytes"` → `[FS]`, `"send"` → `[Net]`).
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
    let bytes_id =
        declare_primitive(env, vec![], type0.clone(), PrimReduction::OpaqueType)
            .map_err(|e| ElabError::Internal(format!("prim Bytes failed: {}", e)))?;
    globals.insert("Bytes".to_string(), bytes_id);

    // -- String : Type 0 --
    let string_id =
        declare_primitive(env, vec![], type0.clone(), PrimReduction::OpaqueType)
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
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "bytes_length" })
            .map_err(|e| ElabError::Internal(format!("prim bytes_length failed: {}", e)))?;
        globals.insert("bytes_length".to_string(), id);
    }

    // -- bytes_at : Bytes → Int → Int --
    {
        let ty = Term::pi(bytes_t.clone(), Term::pi(int_t.clone(), int_t.clone()));
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "bytes_at" })
            .map_err(|e| ElabError::Internal(format!("prim bytes_at failed: {}", e)))?;
        globals.insert("bytes_at".to_string(), id);
    }

    // -- bytes_slice : Bytes → Int → Int → Bytes --
    {
        let ty = Term::pi(
            bytes_t.clone(),
            Term::pi(int_t.clone(), Term::pi(int_t.clone(), bytes_t.clone())),
        );
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "bytes_slice" })
            .map_err(|e| ElabError::Internal(format!("prim bytes_slice failed: {}", e)))?;
        globals.insert("bytes_slice".to_string(), id);
    }

    // -- bytes_concat : Bytes → Bytes → Bytes --
    {
        let ty = Term::pi(bytes_t.clone(), Term::pi(bytes_t.clone(), bytes_t.clone()));
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "bytes_concat" })
            .map_err(|e| ElabError::Internal(format!("prim bytes_concat failed: {}", e)))?;
        globals.insert("bytes_concat".to_string(), id);
    }

    // -- bytes_encode : String → Bytes (total) --
    {
        let ty = Term::pi(string_t.clone(), bytes_t.clone());
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "bytes_encode" })
            .map_err(|e| ElabError::Internal(format!("prim bytes_encode failed: {}", e)))?;
        globals.insert("bytes_encode".to_string(), id);
    }

    // -- bytes_decode : Bytes → String (partial — Neutral on invalid UTF-8) --
    {
        let ty = Term::pi(bytes_t.clone(), string_t.clone());
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "bytes_decode" })
            .map_err(|e| ElabError::Internal(format!("prim bytes_decode failed: {}", e)))?;
        globals.insert("bytes_decode".to_string(), id);
    }

    // -- Effect-tracked I/O ops (`36`/L5): typed as Bytes → Bytes (placeholder).
    //    Real argument types (Path, Socket) are L7 (FFI surface) concerns.
    //    The EFFECT ROW is what matters for AC2/AC3; stored in io_effect_rows. --

    // read_bytes : Bytes → Bytes  visits [FS]
    {
        let ty = Term::pi(bytes_t.clone(), bytes_t.clone());
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "read_bytes" })
            .map_err(|e| ElabError::Internal(format!("prim read_bytes failed: {}", e)))?;
        globals.insert("read_bytes".to_string(), id);
    }

    // write_bytes : Bytes → Bytes → Bytes  visits [FS]
    {
        let ty = Term::pi(bytes_t.clone(), Term::pi(bytes_t.clone(), bytes_t.clone()));
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "write_bytes" })
            .map_err(|e| ElabError::Internal(format!("prim write_bytes failed: {}", e)))?;
        globals.insert("write_bytes".to_string(), id);
    }

    // append : Bytes → Bytes → Bytes  visits [FS]
    {
        let ty = Term::pi(bytes_t.clone(), Term::pi(bytes_t.clone(), bytes_t.clone()));
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "append" })
            .map_err(|e| ElabError::Internal(format!("prim append failed: {}", e)))?;
        globals.insert("append".to_string(), id);
    }

    // send : Bytes → Bytes → Bytes  visits [Net]
    {
        let ty = Term::pi(bytes_t.clone(), Term::pi(bytes_t.clone(), bytes_t.clone()));
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "send" })
            .map_err(|e| ElabError::Internal(format!("prim send failed: {}", e)))?;
        globals.insert("send".to_string(), id);
    }

    // recv : Bytes → Bytes  visits [Net]
    {
        let ty = Term::pi(bytes_t.clone(), bytes_t.clone());
        let id = declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "recv" })
            .map_err(|e| ElabError::Internal(format!("prim recv failed: {}", e)))?;
        globals.insert("recv".to_string(), id);
    }

    // -- I/O effect row registry (AC2/AC3 test seed source) --
    let mut io_effect_rows: HashMap<String, EffectRow> = HashMap::new();
    io_effect_rows.insert("read_bytes".to_string(), EffectRow::singleton("FS"));
    io_effect_rows.insert("write_bytes".to_string(), EffectRow::singleton("FS"));
    io_effect_rows.insert("append".to_string(), EffectRow::singleton("FS"));
    io_effect_rows.insert("send".to_string(), EffectRow::singleton("Net"));
    io_effect_rows.insert("recv".to_string(), EffectRow::singleton("Net"));

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
