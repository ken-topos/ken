//! `ABI-S3` AC-3c, surface half — `Entropy` is VISIBLE in the program type.
//!
//! Architect decision `dec_50pzvb14nnbt0` D3 makes kernel entropy ambient **at
//! the host-dispatch layer** — no `EntropyCap`, no `ProgramCaps` field, no
//! capability token in the request — while requiring it to remain an
//! **explicit** effect in the program's type and trace. Both halves are
//! required: showing only the ambient half is compatible with a *hidden*
//! ambient read, which the ruling forbids.
//!
//! This file discharges the visibility half behaviorally, by reading the
//! elaborated effect rows. The dispatch half is discharged in `ken-host` by
//! `ac3c_entropy_needs_no_capability_token_while_a_gated_op_still_does`.
//!
//! ⚠ Presence of a `visits [Entropy]` annotation in `prelude.rs`, or a
//! statement in a report, is **not** a control — neither is executable. What is
//! asserted here is the effect row the elaborator actually produced.

use ken_elaborator::effects::EffectRow;
use ken_elaborator::ElabEnv;

/// Membership, phrased through the public row API.
fn carries(env: &ElabEnv, procedure: &str, effect: &str) -> bool {
    let row = env
        .effect_rows
        .get(procedure)
        .unwrap_or_else(|| panic!("{procedure} has an elaborated effect row"))
        .concrete_effects();
    EffectRow::singleton(effect).is_subset_of(&row)
}

/// MEASURED: the elaborated effect row of the entropy procedure contains
/// `Entropy`, and the row of a clock procedure does not.
/// CLAIMED: the entropy effect is visible in the type of code that performs it,
/// so it cannot be performed ambiently by code that does not declare it.
/// THE GAP: a probe that reported "contains Entropy" for everything would
/// satisfy the first half vacuously — so the same probe is required to answer
/// NO for a procedure that genuinely lacks the effect, and to answer YES for a
/// different effect that is genuinely present.
#[test]
fn abi_s3_entropy_is_visible_in_the_effect_row_and_absent_where_it_should_be() {
    let env = ElabEnv::new().expect("prelude elaborates");

    // The property: performing entropy is visible in the type.
    assert!(
        carries(&env, "random_bytes", "Entropy"),
        "random_bytes must carry Entropy in its effect row -- ambient at \
         dispatch does not mean invisible in the program type"
    );

    // DISCRIMINATOR -- the same probe answers NO for a procedure that does not
    // perform entropy. Without this, "contains Entropy" would be compatible
    // with a probe that says yes to everything.
    assert!(
        !carries(&env, "wall_now", "Entropy"),
        "wall_now must not carry Entropy; otherwise the assertion above is vacuous"
    );

    // POSITIVE CONTROL -- the probe does find effects that ARE present, so a
    // NO above is a reading rather than an inability to see anything.
    assert!(
        carries(&env, "wall_now", "Clock"),
        "the probe must be able to observe an effect that is present"
    );
    assert!(carries(&env, "read", "Console"));

    // And the ambient injection that carries entropy into HostIO keeps the
    // effect visible rather than laundering it away.
    assert!(
        carries(&env, "host_entropy", "Entropy"),
        "the named ambient injection must preserve the Entropy effect"
    );
    assert!(
        !carries(&env, "host_clock", "Entropy"),
        "the clock injection must not acquire Entropy"
    );
}
