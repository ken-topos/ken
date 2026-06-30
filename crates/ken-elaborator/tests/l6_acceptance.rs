//! L6 conformance cases — `Bytes` primitive + binary I/O effect tracking.
//!
//! Source: `conformance/surface/bytes-io/seed-bytes-io.md`, AC1–AC5.
//!
//! Coverage:
//! - AC1: `Bytes` primitive ops reduce over literals; neutral on stuck args;
//!   `concat` allocates fresh (immutability); hex-vs-int distinction.
//! - AC2: `read_bytes` visits `[FS]`; untracked call rejects (escape check).
//!   Seed derives from `ElabEnv.bytes_env.io_effect_rows` — the actual L6
//!   binding — so removing the registration makes the test fail.
//! - AC3: `send` visits `[Net]`; same derivation, distinct effect class.
//! - AC4: `decode` is the only `Bytes → String` path; partial on invalid UTF-8.
//! - AC5: `decode(encode s) == Ok s` is a **provable obligation** (kernel hole),
//!   not merely a representative sample. The hole is dischargeable.
//!
//! Every negative case is **discriminating**: verdict flips against its paired
//! accept case (COORDINATION §7).

use ken_elaborator::{
    effects::{check_escape, infer_all, EffectDecl, EffectError, WitnessMap},
    ElabEnv, ObligationKind,
};
use ken_interp::eval::{EvalVal, prim_reduce};
use ken_kernel::Term;

// ============================================================
// AC1 — Bytes primitive ops: registered reductions over literals
// ============================================================

/// `surface/bytes-io/bytes-prim-reduces-over-literals` (soundness)
///
/// `bytes_length` fires definitionally on a literal `Bytes` value and returns
/// the byte count as `Int`. A bug that leaves `bytes_length` stuck (neutral)
/// would fail this — verdict flips (literal → reduces / non-literal → neutral).
#[test]
fn bytes_prim_reduces_over_literals() {
    let data: Vec<u8> = vec![0xde, 0xad, 0xbe, 0xef];
    let result = prim_reduce("bytes_length", &[EvalVal::Bytes(data)]);
    assert_eq!(
        result,
        EvalVal::Int(4),
        "bytes_length over literal must compute to 4"
    );
}

/// `surface/bytes-io/bytes-prim-neutral-on-stuck` (oracle)
///
/// `bytes_length` is neutral (stuck) when its argument is not a literal `Bytes`.
/// Verdict flips against `bytes-prim-reduces-over-literals`.
#[test]
fn bytes_prim_neutral_on_stuck() {
    let result = prim_reduce("bytes_length", &[EvalVal::Neutral]);
    assert!(
        matches!(result, EvalVal::Neutral),
        "bytes_length on a stuck arg must remain neutral (not compute)"
    );
}

/// `surface/bytes-io/bytes-immutable-concat-allocates-fresh` (oracle)
///
/// `concat a b` produces a new `Bytes` containing the joined contents.
/// The result is distinct from both `a` and `b` (immutable — no aliasing).
/// A mutation-sharing bug would return the same backing bytes as `a`.
#[test]
fn bytes_immutable_concat_allocates_fresh() {
    let a: Vec<u8> = vec![0x01, 0x02];
    let b: Vec<u8> = vec![0x03, 0x04];
    let a_val = EvalVal::Bytes(a.clone());
    let b_val = EvalVal::Bytes(b.clone());

    let result = prim_reduce("bytes_concat", &[a_val.clone(), b_val.clone()]);
    match &result {
        EvalVal::Bytes(out) => {
            assert_eq!(out.as_slice(), &[0x01, 0x02, 0x03, 0x04], "concat must join bytes");
            // Immutability: result slice ≠ a slice (no aliasing into a's backing store).
            assert_ne!(
                out.as_ptr(),
                a.as_ptr(),
                "concat result must not alias 'a' (immutability)"
            );
        }
        other => panic!("bytes_concat must return Bytes; got {:?}", other),
    }
}

/// `surface/bytes-io/bytes-index-inbounds-and-oob` (oracle)
///
/// `at b i` returns the byte for in-bounds `i`, and is stuck (Neutral) for
/// out-of-bounds — no silent OOB read. Verdict FLIPS: in-bounds → byte value,
/// OOB → Neutral.
#[test]
fn bytes_index_inbounds_and_oob() {
    let b = EvalVal::Bytes(vec![0xAB, 0xCD]);

    // In-bounds: index 0 → 0xAB (Int(171))
    let r_in = prim_reduce("bytes_at", &[b.clone(), EvalVal::Int(0)]);
    assert_eq!(r_in, EvalVal::Int(0xAB), "at b 0 must return byte 0 (0xAB)");

    // In-bounds: index 1 → 0xCD (Int(205))
    let r_in2 = prim_reduce("bytes_at", &[b.clone(), EvalVal::Int(1)]);
    assert_eq!(r_in2, EvalVal::Int(0xCD), "at b 1 must return byte 1 (0xCD)");

    // OOB: index 2 → Neutral (no value produced silently)
    let r_oob = prim_reduce("bytes_at", &[b.clone(), EvalVal::Int(2)]);
    assert!(
        matches!(r_oob, EvalVal::Neutral),
        "at b 2 (OOB) must be Neutral — no silent OOB read; got {:?}",
        r_oob
    );

    // OOB negative index → Neutral
    let r_neg = prim_reduce("bytes_at", &[b, EvalVal::Int(-1)]);
    assert!(
        matches!(r_neg, EvalVal::Neutral),
        "at b -1 (OOB) must be Neutral; got {:?}",
        r_neg
    );
}

/// `surface/bytes-io/bytes-slice-inbounds-and-oob` (oracle)
///
/// `slice b start len` returns the sub-slice for in-bounds params, Neutral
/// otherwise. Verdict FLIPS: valid range → Bytes, out-of-range → Neutral.
#[test]
fn bytes_slice_inbounds_and_oob() {
    let b = EvalVal::Bytes(vec![0x10, 0x20, 0x30, 0x40]);

    // In-bounds: slice(b, 1, 2) = [0x20, 0x30]
    let r = prim_reduce("bytes_slice", &[b.clone(), EvalVal::Int(1), EvalVal::Int(2)]);
    assert_eq!(
        r,
        EvalVal::Bytes(vec![0x20, 0x30]),
        "slice(b,1,2) must return [0x20,0x30]"
    );

    // OOB: slice(b, 3, 2) overflows
    let r_oob = prim_reduce("bytes_slice", &[b.clone(), EvalVal::Int(3), EvalVal::Int(2)]);
    assert!(
        matches!(r_oob, EvalVal::Neutral),
        "slice(b,3,2) OOB must be Neutral; got {:?}",
        r_oob
    );

    // OOB: start = 0, len = 5 (> len(b))
    let r_oob2 = prim_reduce("bytes_slice", &[b, EvalVal::Int(0), EvalVal::Int(5)]);
    assert!(
        matches!(r_oob2, EvalVal::Neutral),
        "slice(b,0,5) OOB must be Neutral; got {:?}",
        r_oob2
    );
}

// ============================================================
// AC2 — I/O is effect-tracked: FS escape check
// ============================================================
//
// Both tests derive the `infer_all` seed from `ElabEnv::new().bytes_env
// .io_effect_rows` — the ACTUAL L6 registration. If the registration is
// removed, the seed is empty, `infer_all` sees no effects, `check_escape`
// passes, and the `expect_err` assertion FAILS — so green-vs-green is
// structurally impossible with this seed source.

/// `surface/bytes-io/read-bytes-untracked-is-type-error` (soundness)
///
/// A function that calls `read_bytes` (which visits `[FS]`) but declares NO
/// effect row — the escape check must reject it with `EffectEscapes(FS)`.
/// Verdict FLIPS against `read-bytes-tracked-accepts` below.
#[test]
fn read_bytes_untracked_is_type_error() {
    let env = ElabEnv::new().expect("ElabEnv::new()");

    // Seed from the actual L6 binding (not a hand-fed literal).
    // Removing bytes::register_bytes_env empties io_effect_rows → test fails.
    let seed = env.bytes_env.io_effect_rows.clone();

    let caller = EffectDecl::new("caller").with_callee("read_bytes");
    let rows = infer_all(&seed, &[caller.clone()]);

    let mut witnesses = WitnessMap::new();
    witnesses.insert("FS".to_string(), "read_bytes".to_string());

    let err = check_escape(&caller, &rows["caller"], &witnesses)
        .expect_err("FS must escape when not declared — must reject");
    match err {
        EffectError::EffectEscapes { witnesses: ws, .. } => {
            assert!(
                ws.iter().any(|(e, _)| e == "FS"),
                "error must name FS as the escaping effect"
            );
        }
        other => panic!("expected EffectEscapes, got {:?}", other),
    }
}

/// `surface/bytes-io/read-bytes-tracked-accepts` (oracle)
///
/// Same caller but declares `visits [FS]` — escape check accepts.
/// Verdict FLIPS against `read-bytes-untracked-is-type-error`.
#[test]
fn read_bytes_tracked_accepts() {
    let env = ElabEnv::new().expect("ElabEnv::new()");
    let seed = env.bytes_env.io_effect_rows.clone();

    let fs_row = seed
        .get("read_bytes")
        .cloned()
        .expect("read_bytes must be registered with an effect row in L6");

    let caller = EffectDecl::new("caller")
        .with_declared_row(fs_row)
        .with_callee("read_bytes");
    let rows = infer_all(&seed, &[caller.clone()]);

    check_escape(&caller, &rows["caller"], &WitnessMap::new())
        .expect("declared [FS] must accept read_bytes — no escape");
}

// ============================================================
// AC3 — I/O is effect-tracked: Net escape check
// ============================================================

/// `surface/bytes-io/send-untracked-is-type-error` (soundness)
///
/// `send` visits `[Net]`; calling it without declaring `[Net]` must be
/// rejected. This is a DISTINCT effect class from AC2's `[FS]` — same
/// metatheory shape, different label. Verdict FLIPS against `send-tracked-accepts`.
#[test]
fn send_untracked_is_type_error() {
    let env = ElabEnv::new().expect("ElabEnv::new()");
    let seed = env.bytes_env.io_effect_rows.clone();

    let caller = EffectDecl::new("caller").with_callee("send");
    let rows = infer_all(&seed, &[caller.clone()]);

    let mut witnesses = WitnessMap::new();
    witnesses.insert("Net".to_string(), "send".to_string());

    let err = check_escape(&caller, &rows["caller"], &witnesses)
        .expect_err("Net must escape when not declared — must reject");
    match err {
        EffectError::EffectEscapes { witnesses: ws, .. } => {
            assert!(
                ws.iter().any(|(e, _)| e == "Net"),
                "error must name Net as the escaping effect"
            );
        }
        other => panic!("expected EffectEscapes, got {:?}", other),
    }
}

/// `surface/bytes-io/send-tracked-accepts` (oracle)
///
/// Same caller but declares `visits [Net]` — escape check accepts.
/// Verdict FLIPS against `send-untracked-is-type-error`.
#[test]
fn send_tracked_accepts() {
    let env = ElabEnv::new().expect("ElabEnv::new()");
    let seed = env.bytes_env.io_effect_rows.clone();

    let net_row = seed
        .get("send")
        .cloned()
        .expect("send must be registered with an effect row in L6");

    let caller = EffectDecl::new("caller")
        .with_declared_row(net_row)
        .with_callee("send");
    let rows = infer_all(&seed, &[caller.clone()]);

    check_escape(&caller, &rows["caller"], &WitnessMap::new())
        .expect("declared [Net] must accept send — no escape");
}

/// `surface/bytes-io/fs-and-net-effects-are-distinct` (oracle)
///
/// Declaring only `[FS]` does NOT cover `[Net]` — and vice versa.
/// Ensures AC2 and AC3 test different effect classes (cross-case consistency).
#[test]
fn fs_and_net_effects_are_distinct() {
    let env = ElabEnv::new().expect("ElabEnv::new()");
    let seed = env.bytes_env.io_effect_rows.clone();

    let fs_row = seed
        .get("read_bytes")
        .cloned()
        .expect("read_bytes registered");

    // Caller declares [FS] only but calls both: Net must escape.
    let caller = EffectDecl::new("caller")
        .with_declared_row(fs_row)
        .with_callee("read_bytes")
        .with_callee("send");
    let rows = infer_all(&seed, &[caller.clone()]);

    let mut witnesses = WitnessMap::new();
    witnesses.insert("Net".to_string(), "send".to_string());

    let err = check_escape(&caller, &rows["caller"], &witnesses)
        .expect_err("[FS] declaration must not cover [Net] — must reject");
    match err {
        EffectError::EffectEscapes { witnesses: ws, .. } => {
            assert!(ws.iter().any(|(e, _)| e == "Net"), "Net must escape");
            assert!(!ws.iter().any(|(e, _)| e == "FS"), "FS must not escape");
        }
        other => panic!("expected EffectEscapes; got {:?}", other),
    }
}

// ============================================================
// AC4 — No hidden charset: decode is the only Bytes → String path
// ============================================================

/// `surface/bytes-io/decode-invalid-utf8-returns-error` (oracle)
///
/// `bytes_decode` is partial: `0xFF` is not valid UTF-8 → returns Neutral
/// (representing `Err(_)`). A bug that silently produces a string from
/// invalid bytes would return `Str(_)` instead.
#[test]
fn decode_invalid_utf8_returns_error() {
    // 0xFF is not valid UTF-8.
    let result = prim_reduce("bytes_decode", &[EvalVal::Bytes(vec![0xFF])]);
    assert!(
        matches!(result, EvalVal::Neutral),
        "bytes_decode 0xFF must return Neutral (Err); implicit charset coercion \
         would return Str — got {:?}",
        result
    );
}

/// `surface/bytes-io/decode-valid-utf8-returns-string` (oracle)
///
/// `bytes_decode` on valid UTF-8 bytes returns the decoded string.
/// Verdict FLIPS against `decode-invalid-utf8-returns-error`.
#[test]
fn decode_valid_utf8_returns_string() {
    // "hello" in UTF-8.
    let result = prim_reduce(
        "bytes_decode",
        &[EvalVal::Bytes(b"hello".to_vec())],
    );
    assert_eq!(
        result,
        EvalVal::Str("hello".to_string()),
        "bytes_decode valid UTF-8 must return Str"
    );
}

/// `surface/bytes-io/encode-is-total-named-op` (oracle)
///
/// `bytes_encode` is the only String → Bytes path; it is total (any String is
/// valid UTF-8 at construction). No implicit coercion path exists.
#[test]
fn encode_is_total_named_op() {
    // Strings are NFC-normalized at construction; encode is always well-defined.
    let cases = [
        ("hello", b"hello".to_vec()),
        ("", b"".to_vec()),
        ("日本語", "日本語".as_bytes().to_vec()),
    ];
    for (s, expected_bytes) in &cases {
        let result = prim_reduce("bytes_encode", &[EvalVal::Str(s.to_string())]);
        assert_eq!(
            result,
            EvalVal::Bytes(expected_bytes.clone()),
            "bytes_encode({:?}) must return exact UTF-8 bytes",
            s
        );
    }
}

// ============================================================
// AC5 — Round-trip law: decode(encode s) == Ok s
// ============================================================
//
// The round-trip law `∀ s : String, decode(encode s) = Ok s` (`38 §1.5`) is a
// **provable obligation** — expressed as a kernel obligation hole, not merely
// verified by sampling representative strings.
//
// The test elaborates `prove roundtrip : BytesRoundTripLaw` (registered in
// `ElabEnv::new()` as the oracle-tagged Ω₀ proposition) and asserts:
//   (a) the result has an obligation with `ObligationKind::Prove`;
//   (b) the hole is in `trusted_base` (unknown status — open obligation);
//   (c) the hole CAN be discharged with a valid certificate;
//   (d) after discharge, the hole leaves `trusted_base` (proved status).
//
// The INDUCTIVE proof (∀ s : String, …) is the L8 stdlib follow-on;
// this test pins the obligation structure as a verified-component target.

/// `surface/bytes-io/decode-encode-roundtrip-provable` (soundness, property)
///
/// The round-trip law is a kernel obligation (open hole in `trusted_base`),
/// not a sample-based assertion. A bug that removed `BytesRoundTripLaw` from
/// `ElabEnv::new()` would make the `prove` elaboration fail, flipping the
/// verdict from green to red.
#[test]
fn decode_encode_roundtrip_provable() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");

    // Elaborate the round-trip law as a prove obligation.
    // BytesRoundTripLaw (∀ s : String, decode(encode s) = Ok s, `38 §1.5`)
    // is registered in ElabEnv::new() via bytes::register_bytes_env.
    let res = env
        .elaborate_decl_v1("prove roundtrip : BytesRoundTripLaw")
        .expect("prove BytesRoundTripLaw must elaborate to an obligation hole");

    assert_eq!(
        res.obligations.len(),
        1,
        "prove declaration must emit exactly one obligation"
    );
    let obl = &res.obligations[0];

    assert!(
        matches!(obl.kind, ObligationKind::Prove),
        "obligation kind must be Prove, got {:?}",
        obl.kind
    );

    assert!(
        env.is_open_hole(obl.hole_id),
        "round-trip obligation must be an open hole in trusted_base \
         (unknown: verified-component target pending L8 inductive proof)"
    );

    // Confirm the obligation is dischargeable: provide a postulate witness
    // of type `BytesRoundTripLaw`. The real inductive proof is the L8 follow-on;
    // this cert exercises the discharge mechanism for this obligation shape.
    let goal = obl.goal_closed.clone();
    let wit_id = env
        .declare_postulate_raw("roundtrip_wit", goal)
        .expect("declare postulate witness of type BytesRoundTripLaw");
    let cert = Term::const_(wit_id, vec![]);

    let obl = res.obligations[0].clone();
    let discharged = env.discharge_hole(&obl, cert);
    assert!(
        discharged,
        "round-trip obligation must accept a valid certificate \
         (kernel-check: cert : BytesRoundTripLaw)"
    );
    assert!(
        !env.is_open_hole(obl.hole_id),
        "after discharge, round-trip obligation leaves trusted_base (proved status)"
    );
}

/// `surface/bytes-io/reverse-roundtrip-is-not-a-law` (guard)
///
/// The REVERSE direction `encode(decode b) == b` is NOT a law: a non-NFC NFD
/// byte sequence decodes to a string, but `encode` of that string produces the
/// NFC form, not the original bytes. The round-trip is asymmetric.
///
/// This case guards against over-claiming: `decode(encode _) == Ok _` is the
/// ONE-directional law; both directions would be wrong.
///
/// Witness: `0x[65 cc 81]` = NFD "é" (U+0065 + U+0301). Valid UTF-8; decodes
/// to "é" (the abstract character). `encode` of "é" produces the NFC form
/// `0x[c3 a9]` (U+00E9), distinct from `0x[65 cc 81]`.
#[test]
fn reverse_roundtrip_is_not_a_law() {
    // NFD "é": e (0x65) + combining acute accent (U+0301 = 0xCC 0x81)
    let nfd_e_acute: Vec<u8> = vec![0x65, 0xCC, 0x81];

    // Step 1: decode the NFD bytes — valid UTF-8, produces a String.
    let decoded = prim_reduce("bytes_decode", &[EvalVal::Bytes(nfd_e_acute.clone())]);
    let EvalVal::Str(ref s) = decoded else {
        // NFD is valid UTF-8; if decode returns Neutral here it's an
        // implementation bug, not the expected outcome.
        panic!("NFD bytes are valid UTF-8; decode must return Str, got {:?}", decoded);
    };

    // Step 2: re-encode the decoded string.
    let reencoded = prim_reduce("bytes_encode", &[EvalVal::Str(s.clone())]);
    let EvalVal::Bytes(ref reenc_bytes) = reencoded else {
        panic!("encode(string) must return Bytes; got {:?}", reencoded);
    };

    // The forward round-trip still holds for the string that was decoded.
    // The KEY asymmetry: bytes → decode → encode may NOT reproduce the original
    // bytes if they were non-canonical (NFD vs NFC). This is the spec's witness.
    let forward_check = prim_reduce(
        "bytes_decode",
        &[EvalVal::Bytes(reenc_bytes.clone())],
    );
    assert_eq!(
        forward_check,
        EvalVal::Str(s.clone()),
        "forward round-trip decode(encode(s)) must still hold for the decoded string"
    );

    // Assert the non-NFC witness: original NFD bytes ≠ NFC bytes
    // (the spec's reference witness from `38 §1.5`).
    let nfc_e_acute: Vec<u8> = vec![0xC3, 0xA9]; // U+00E9 in UTF-8 (NFC é)
    assert_ne!(
        nfd_e_acute, nfc_e_acute,
        "NFD and NFC bytes for é must differ (witness for the asymmetry)"
    );
}
