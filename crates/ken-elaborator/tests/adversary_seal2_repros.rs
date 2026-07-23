//! SEAL-2 starting evidence — the adversary's repros (`adversary/SEAL2-repros`,
//! filed against SPAN-SEAL `cd4184b8`, adversary→steward `evt_74mjc4txd9y1e`),
//! ported onto SEAL-2 and consuming the closed oracle in `seal2_support`.
//!
//! The four PROPERTY tests are kept: they assert something **true that must keep
//! holding** — a break is a live finding to route, not a test to weaken.
//!
//! The two GAP tests are **inverted** (AC-8). At `cd4184b8` they passed
//! *because the landed oracle was blind*; each carried an inline
//! `SEAL-2 INVERTS THIS` note. With the enumeration closed, each escape is now
//! SEEN, and the assertions flip accordingly. If these two ever revert to the
//! old "landed oracle is blind" shape, the enumeration is no longer closed.

mod seal2_support;

use std::collections::BTreeSet;

use ken_elaborator::ElabEnv;

use seal2_support::{closed_producers, conservative_deep_producers, head_only_producers};

const BUFFER_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/System/Buffer.ken.md");
const IO_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/System/IO.ken.md");

fn landed_surface() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude");
    env.elaborate_ken_md_file(BUFFER_KEN_MD)
        .expect("System.Buffer checked fences");
    env.elaborate_ken_md_file(IO_KEN_MD)
        .expect("System.IO checked fences");
    env
}

// ---------------------------------------------------------------------------
// PROPERTY tests — must keep holding.
// ---------------------------------------------------------------------------

/// The claim SPAN-SEAL actually cares about, checked three ways: no public
/// global sources a `BufferSpan`, wrapped or not. The closed oracle is the
/// strongest of the three and must also derive `{}`.
#[test]
fn carrier_closure_holds_deeply_on_the_landed_surface() {
    let env = landed_surface();
    assert_eq!(
        head_only_producers(&env, "BufferSpan"),
        BTreeSet::new(),
        "landed oracle's own head-only claim"
    );
    assert_eq!(
        conservative_deep_producers(&env, "BufferSpan"),
        BTreeSet::new(),
        "STRONGER: no wrapped/non-head BufferSpan producer either"
    );
    assert_eq!(
        closed_producers(&env, "BufferSpan"),
        BTreeSet::new(),
        "STRONGEST: the closed oracle (every namespace + position) also derives {{}}"
    );
}

/// S2: true today, but SPAN-SEAL asserts nothing about it. RT-PARITY's
/// BufferFreeze source-unreachability argument rests on this closure.
#[test]
fn transfer_count_has_no_public_producer() {
    let env = landed_surface();
    assert_eq!(head_only_producers(&env, "TransferCount"), BTreeSet::new());
    assert_eq!(
        conservative_deep_producers(&env, "TransferCount"),
        BTreeSet::new(),
    );
    assert_eq!(
        closed_producers(&env, "TransferCount"),
        BTreeSet::new(),
        "RT-PARITY's premise, verified over the closed enumeration rather than grepped"
    );
}

/// Every route by which checked source might mint a `BufferSpan`. The last arm
/// is the answer to the Steward's Q3 (transparency exploit).
#[test]
fn direct_buffer_span_forgery_routes_all_reject() {
    for (label, source) in [
        (
            "private constructor",
            "const forged : BufferSpan = PrivateBufferSpan 0 99",
        ),
        (
            "sealed transition",
            "fn forged (s : BufferSpan) (c : TransferCount) : BufferSpan = \
             write_all_advance_span s c",
        ),
        (
            "rebuild from a destructured ReadSome",
            "fn forged (p : ReadProgress) : BufferSpan = \
             match p { ReadSome span count |-> PrivateBufferSpan 0 99; ReadEof |-> \
             PrivateBufferSpan 0 0 }",
        ),
        (
            "transparency of the exact-prefix proposition",
            "fn forged (s : BufferSpan) (c : TransferCount) : BufferSpan = \
             write_all_exact_prefix_prop s c",
        ),
    ] {
        let mut env = ElabEnv::empty().expect("prelude");
        let outcome = env.elaborate_file(source);
        assert!(
            outcome.is_err(),
            "FORGERY SUCCEEDED via [{label}] — checked source minted a BufferSpan"
        );
        println!("buffer-span forgery [{label}] => {outcome:?}");
    }
}

#[test]
fn direct_transfer_count_forgery_routes_all_reject() {
    for (label, source) in [
        (
            "private constructor",
            "const forged : TransferCount = PrivateTransferCount 0 99",
        ),
        (
            "rebuild from a destructured Wrote",
            "fn forged (w : WriteProgress) : TransferCount = \
             match w { Wrote count |-> PrivateTransferCount 0 99 }",
        ),
    ] {
        let mut env = ElabEnv::empty().expect("prelude");
        let outcome = env.elaborate_file(source);
        assert!(
            outcome.is_err(),
            "FORGERY SUCCEEDED via [{label}] — checked source minted a TransferCount"
        );
        println!("transfer-count forgery [{label}] => {outcome:?}");
    }
}

// ---------------------------------------------------------------------------
// GAP tests — INVERTED (AC-8). At cd4184b8 these passed because the oracle was
// blind; now the closed oracle SEES each escape.
// ---------------------------------------------------------------------------

/// S1 family (A): a non-head result position. A `Result E BufferSpan` producer
/// is a well-formed declaration (so the landed classifier never panicked), and
/// its head is `Result` (so the head-only walk skipped it).
///
/// Was: `gap_wrapped_return_is_invisible_to_landed_oracle` — asserted the landed
/// oracle did NOT see `escaped_wrapped`. Now inverted: the closed oracle DOES.
#[test]
fn wrapped_return_is_seen_by_the_closed_oracle() {
    let mut env = ElabEnv::empty().expect("prelude");
    env.elaborate_file(
        "fn escaped_wrapped (span : BufferSpan) : Result ResourceError BufferSpan = \
         Ok ResourceError BufferSpan span",
    )
    .expect("wrapped producer elaborates");

    assert!(
        !head_only_producers(&env, "BufferSpan").contains("escaped_wrapped"),
        "the head-only oracle derives {{}} here — the gap SEAL-2 closes"
    );
    assert!(
        closed_producers(&env, "BufferSpan").contains("escaped_wrapped"),
        "SEAL-2 CLOSED: the wrapped producer is now seen"
    );
}

/// S1 family (B): an unenumerated namespace. Class field types live in
/// `class_env.classes[C].field_types` and are source-reachable by `d.field`
/// projection; they never enter `globals`.
///
/// Was: `gap_class_field_producer_is_invisible_to_landed_oracle` — asserted the
/// landed oracle's producer set stayed empty. Now inverted: the closed oracle
/// enumerates the class-field namespace and sees it.
#[test]
fn class_field_producer_is_seen_by_the_closed_oracle() {
    let mut env = ElabEnv::empty().expect("prelude");
    env.elaborate_decl("class SpanBox A { unbox : A → BufferSpan }")
        .expect("class with a BufferSpan-producing field elaborates");
    env.elaborate_decl("instance SpanBox BufferSpan { unbox = λs.s }")
        .expect("instance elaborates");

    assert!(
        head_only_producers(&env, "BufferSpan").is_empty(),
        "the globals-only oracle never reaches the class-field namespace — the gap"
    );
    assert!(
        closed_producers(&env, "BufferSpan").contains("SpanBox.unbox"),
        "SEAL-2 CLOSED: the class-field producer is now seen"
    );
}
