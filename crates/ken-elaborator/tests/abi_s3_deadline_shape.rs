//! `ABI-S3` AC-3b, surface half — the **`Deadline` type** carries no
//! cancellation field, token, or status.
//!
//! AC-3b names two representations: *"the `Deadline` type **and** the
//! `SleepUntil` request."* The `ken-host` triad
//! (`ac3b_the_sleep_request_carries_a_deadline_and_no_cancellation_surface`)
//! covers the canonical host request. It cannot cover this one: an extra
//! surface field is discarded during decoding, **before** any wire image or C
//! record exists, so every downstream control stays green while the forbidden
//! surface survives. That is the gap `runtime-qa` identified on `c7ffb0d7`,
//! and this file closes it at the representation the AC actually names.
//!
//! The measurement is the **elaborated constructor telescope** — what the
//! kernel admitted — not any statement in `prelude.rs`.

use ken_elaborator::ElabEnv;

/// Number of arguments the kernel admitted for a constructor.
fn constructor_arity(env: &ElabEnv, name: &str) -> usize {
    let id = *env
        .globals
        .get(name)
        .unwrap_or_else(|| panic!("{name} is a registered global"));
    let (inductive, index) = env
        .env
        .constructor(id)
        .unwrap_or_else(|| panic!("{name} is a constructor"));
    inductive.constructors[index].args.len()
}

/// MEASURED: the admitted argument telescopes of `MkDeadline`,
/// `MkMonotonicInstant`, and `SleepUntil` each hold exactly one argument.
/// CLAIMED: no cancellation field, token, or status rides on the `Deadline`
/// type or on the surface sleep operation.
/// THE GAP: an arity of one is only evidence if the probe can read arities
/// other than one — so the same probe is required to report `0` for the
/// nullary clock operations and `2` for a genuinely binary constructor. Without
/// that, "exactly one argument" is indistinguishable from a probe that returns
/// one for everything.
#[test]
fn abi_s3_the_deadline_type_carries_exactly_its_reading_and_no_cancellation_field() {
    let env = ElabEnv::new().expect("prelude elaborates");

    // POSITIVE CONTROL — the probe reads real, differing arities.
    assert_eq!(
        constructor_arity(&env, "WallNow"),
        0,
        "the probe must be able to report a nullary constructor"
    );
    assert_eq!(constructor_arity(&env, "MonotonicNow"), 0);
    assert_eq!(
        constructor_arity(&env, "Read"),
        2,
        "the probe must be able to report an arity above one, or 'exactly one' \
         below is indistinguishable from a probe that always answers one"
    );

    // THE PROPERTY — one argument each, so there is no second position for a
    // cancellation token, status, or reserved field to occupy.
    assert_eq!(
        constructor_arity(&env, "MkDeadline"),
        1,
        "Deadline wraps exactly one monotonic reading; a second argument would \
         be the cancellation surface D2 forbids"
    );
    assert_eq!(
        constructor_arity(&env, "MkMonotonicInstant"),
        1,
        "a monotonic instant is exactly its reading"
    );
    assert_eq!(
        constructor_arity(&env, "SleepUntil"),
        1,
        "SleepUntil takes exactly a Deadline; a second argument would be a \
         cancellation token or status"
    );

    // `Deadline` is a one-constructor type, so the arity above is the whole
    // shape rather than one arm of several.
    let deadline = *env.globals.get("Deadline").expect("Deadline is registered");
    let admitted = env
        .env
        .constructor(*env.globals.get("MkDeadline").expect("MkDeadline"))
        .expect("MkDeadline is a constructor")
        .0;
    assert_eq!(admitted.id, deadline, "MkDeadline belongs to Deadline");
    assert_eq!(
        admitted.constructors.len(),
        1,
        "Deadline has exactly one constructor, so its single arity is its \
         entire surface"
    );
    // Control for that count: a multi-constructor family reports more.
    let clock_op = env
        .env
        .constructor(*env.globals.get("WallNow").expect("WallNow"))
        .expect("WallNow is a constructor")
        .0;
    assert!(
        clock_op.constructors.len() > 1,
        "the constructor-count probe distinguishes families"
    );
}
