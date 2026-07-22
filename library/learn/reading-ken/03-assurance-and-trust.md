# 03 — Assurance and trust: proved, tested, delegated, unknown

Chapter [02](02-types-contracts-and-proofs.md) closed on a deliberate gap: a
passing `ken check` tells you a stated claim was proved, but not, by itself,
which of Ken's several honest verification statuses that claim actually
carries. This chapter closes that gap, and then asks the harder question a
signature and a proof together still don't answer — what, exactly, are you
trusting when you trust this file?

## 1. Four honest statuses, not one silent "verified"

Every specification claim in Ken carries one of exactly four **epistemic
statuses**, visible in the source rather than collapsed into a single
pass/fail
(`spec/20-verification/21-spec-syntax.md`
[§5.2](../../../spec/20-verification/21-spec-syntax.md#52-the-epistemic-status-per-claim-export-facing--oq-spec-decided)):

- **`proved`** — the obligation was discharged and the kernel re-checked the
  certificate. The default for a contract that goes through; no annotation,
  it simply holds.
- **`tested`** — a property that cannot yet be proved, asserted instead with
  a runtime/test obligation. Visible as such: a reader knows this guarantee
  rests on tests, not proof.
- **`delegated`** — a temporal/behavioral property (liveness, ordering,
  eventual consistency) that isn't a static proposition over a pure function,
  so the kernel cannot close it. Ken states it and hands it to a runtime
  sibling to model-check and monitor.
- **`unknown`** — not discharged, and no test or delegation given either: an
  admitted typed hole. The program still runs; the result carries `unknown`
  wherever the unproven property is observed.

This four-way status is a different classification from the three-way
**verdict** (`proved`/`disproved`/`unknown`) a single obligation gets when
you attempt it
(`spec/20-verification/21-spec-syntax.md`
[§5.1](../../../spec/20-verification/21-spec-syntax.md#51-the-verdict-per-obligation-operational)) —
the verdict is the operational outcome of one attempt; the epistemic status
is the label the claim carries afterward, and it is the one worth reading
for. The rest of this chapter walks all four against real fragments.

## 2. `proved`, read from a real certificate

A `proved` verdict is a fact about one **claim and its certificate**, not
about a file as a whole. `catalog/packages/Core/Logic/EmptyDec.ken.md`'s
own Laws & proofs section states two such claims directly:

```ken
proof yes_is_true for decide : Equal Bool (decide (Equal Bool True True) true_is_true) True =
  Proved

proof no_is_false for decide
    : Equal Bool (decide (Equal Bool True False) true_is_not_false) False =
  Proved
```

Each is `proved`: the kernel re-checked the `Proved` certificate against
its stated `Equal Bool …` claim, and this file's whole-file elaboration —
Definition, Using it, and this Laws & proofs section together — is exactly
what `ac4_bridge_demonstrated_over_deceq_bool_not_only_deceq_int`
(`crates/ken-elaborator/tests/ds1_empty_dec_acceptance.rs`) exercises when
it loads the entry standalone
(`spec/20-verification/21-spec-syntax.md`
[§5.3](../../../spec/20-verification/21-spec-syntax.md#53-how-the-verdict-and-the-status-relate-the-projection)).
A `proved` verdict adds nothing to what a consumer must trust beyond the
kernel itself — the certificate is a closed term `check` validates, and a
wrong certificate simply fails to validate; it cannot manufacture a false
`proved`.

That per-claim fact is separate from what a file's **Trust &
derivation** section reports about the assumptions the whole artifact
inherits — a different kind of accounting, not itself a verdict.
`catalog/packages/Core/Logic/Transport.ken.md`'s own Trust & derivation
section states, exactly: **"`trusted_base()` delta. Zero."** That
sentence is checked by a real producer,
`transport_package_adds_zero_trusted_base_delta`
(`crates/ken-elaborator/tests/surface_transport_acceptance.rs`), which
loads the package and asserts `trusted_base()` is set-equal before and
after — every combinator reduces through the already-trusted `J`/`cast`,
adding **nothing**.

`EmptyDec.ken.md`'s own Trust & derivation section says something
narrower, in different words: **"Zero new trust category."** That
distinction is real, not a paraphrase of the same fact. Its own producer,
`ac3_trusted_base_delta_is_ordinary_inductive_admission_only`
(same file), establishes that `Empty`/`Dec`'s own *admission* is ordinary
inductive machinery — zero `declare_primitive`/`declare_postulate` delta
for the two new inductives themselves — which is a narrower claim than
"every claim reachable through this file is postulate-free regardless of
instance." Section 5 below shows exactly why the difference is real
rather than pedantic: the same entry's own Design notes name a possible
*instantiation* whose delta is not zero, even though the entry's own §3
worked examples never actually build that instantiation.

## 3. `delegated`, read from a fragment's own honest prose

`catalog/packages/Capability/System/IO.ken.md` states its own boundary
directly, in its opening paragraph: its five `lemma`s are real, kernel-
checked proof terms, but "exactly-once settlement and liveness remain
runtime-enforced, delegated boundary properties." That sentence names, in
the fragment's own words, exactly the shape `delegated` describes in the
spec — a property about *behavior over time* (does the write eventually
settle? exactly once?), not a static proposition a pure function's body can
close, so it is stated and hands off to the runtime rather than proved here
(`spec/20-verification/21-spec-syntax.md` §5.2). The file is honest about
the line: five proofs are proved; settlement and liveness are not, and it
says so in its own text rather than leaving a reader to assume the whole
file carries one uniform guarantee.

As with `tested` below, read that carefully: this is the fragment's own
honest prose naming the *concept* `delegated` describes, not an instance
of the formal, tagged `delegate` clause itself. That concrete clause
spelling is deferred exactly like `tested`'s — both are named but left
un-spelled pending the behavioral sibling and the test/generator framework
(`spec/20-verification/21-spec-syntax.md`
[§5.5](../../../spec/20-verification/21-spec-syntax.md#55-scope-ruling--disposition-tag-syntax-is-deferred)) —
so treat the formal `delegated` status, like `tested`, as **unavailable**
in this curriculum's checked fragments, even though the concept it names
is stated plainly in real corpus prose.

## 4. `tested` — the concept, honestly labelled as not yet exhibited here

`catalog/packages/Tooling/Testing/Property.ken.md` draws a related but
different line, in its own Motivation section: "Properties here are
computations, not propositions. They test the executable shadow of a
contract without assuming or proving that contract." That is a real,
current illustration of *why* testing and proving are different activities
— useful groundwork for reading `tested` — but it is not the same thing as
an example of the spec's formal `tested` epistemic status, which is
produced by an `assume`/`test`-tagged clause lowering a `requires`/`ensures`
to a runtime assertion. That concrete clause grammar is still
proposal-level (`OQ-syntax`, `spec/20-verification/21-spec-syntax.md` §5.2,
§5.5), and none of this curriculum's registered fragments exhibit it. So:
read `Property.ken.md` for the *concept* `tested` names — a check that
exercises code rather than proving a claim about it — and take the formal
tagged construct itself as **unavailable** in the current fragment set,
rather than assume the two are the same thing because they rhyme.

## 5. `unknown` and the postulate it names

`EmptyDec.ken.md`'s own Design notes section states a caveat about an
instantiation the entry itself never builds as a **worked example**:
`dec_eq_decides Int (DecEq Int) x y` type-checks and is usable, but
`DecEq Int.sound` is "`Axiom`-backed (`Int` is an opaque primitive, no
induction)" — its `Yes` branch's proof rides that axiom rather than a
kernel-checked derivation. An axiom admitted this way is exactly a
**postulate**: an assumed proposition entered via `declare_postulate`, one
of Ken's exactly three trusted-computing-base categories, alongside the
kernel itself and the primitive declarations
(`spec/60-security/64-trust-model.md`
[§1](../../../spec/60-security/64-trust-model.md#1-the-trusted-computing-base-tcb-precisely)).
Nothing else is trusted — not the elaborator, not the prover, not the
surface compiler — they all produce artifacts the kernel re-checks. This is
exactly *why* the entry's actual §3 worked examples deliberately use
`DecEq Bool`
instead: an inductive carrier proved by no-confusion, honest and
zero-delta, so the showcase itself is not quietly resting on the axiom the
Design notes names.

## 6. Reading a fragment's `trusted_base()` claim as a checked fact, not a promise

Chapter [01](01-anatomy.md) §2 already showed you that every selected
fragment's Trust & derivation section states a `trusted_base()` delta. Now
you can read what that number actually certifies — and, per §2 above, what
it does not: the sentence in a fragment's prose is only as trustworthy as
the producer that actually computed it. The kernel's `trusted_base()`
enumerates, on demand, every postulate and primitive declaration an
artifact rests on; it is complete by construction — the only two ways an
unchecked assumption can enter the program are `declare_postulate` and
`declare_primitive`, and both land exactly the declarations the enumerator
lists, so no assumption can hide
(`spec/60-security/64-trust-model.md`
[§1.1](../../../spec/60-security/64-trust-model.md#11-the-enumeration-contract-soundness-landed-producer),
[§1.2](../../../spec/60-security/64-trust-model.md#12-the-completeness-net-no-hidden-assumption-by-construction)).
An **empty** delta is the "fully verified, nothing assumed" signal; a
**non-empty** one lists exactly what you inherit. Crucially, this is
decidable from the kernel's own state, not from a label a file's prose
chooses to print: a claim is `proved` **iff** its certificate checks *and*
no postulate carrying its goal sits in `trusted_base()` — so a file cannot
claim `proved` for something the kernel itself would list as assumed
(`spec/20-verification/21-spec-syntax.md`
[§5.4](../../../spec/20-verification/21-spec-syntax.md#54-the-honesty-guard-unknowntesteddelegated-never-read-proved)).
For Transport, §2's cited producer is exactly this check, run for real:
`trusted_base()` before loading the package equals `trusted_base()` after.
For EmptyDec, the corresponding producer instead confirms the narrower,
accurately-worded claim its own prose makes: the admission itself carries
no primitive/postulate delta, which is a different, smaller fact than
"every claim in this file is delta-free regardless of instance." Reading a
fragment's own Trust & derivation sentence is a reasonable first read, but
what makes it a *checked fact* rather than an author's promise is the
producer behind it — read the sentence, then, when the claim matters, find
and read the producer that established it.

## 7. Honest limits stated in the fragment's own voice

`catalog/packages/Capability/Filesystem/Errors.ken.md` states its own
boundary in its second paragraph, before any code: "the current authority
check is coarse and is **not** path-confined. An `AFull` capability permits
writes and deletes anywhere the host process can access." That is the same
posture the trust model itself insists on at the language level — a
verified system that over-claims is itself a security risk, so the honest
limits are first-class, not buried
(`spec/60-security/64-trust-model.md`
[§4](../../../spec/60-security/64-trust-model.md#4-the-honest-limits-what-a-language-cannot-fix-normative)).
This entry's own capability, `AFull`, and what it does and doesn't confine
is chapter [04](04-effects-capabilities-and-authority.md)'s subject; here,
notice only that stating your own limitation honestly is a discipline the
fragments themselves practice, not just a rule stated about them from
outside.

## Reader can now answer

- Given a claim in a Ken file, which of the four epistemic statuses does it
  carry, and what producer — not just which sentence of prose — would you
  point to if asked to prove it?
- What does an **empty** `trusted_base()` delta certify, and why is that
  certification not just the author's word for it — and why can two
  fragments word a zero-delta claim differently and mean genuinely
  different things?
- Why are `Property.ken.md`'s property runner and `System/IO.ken.md`'s
  "delegated boundary properties" prose both useful for understanding what
  `tested` and `delegated` *mean*, without either being an example of the
  formal, tagged construct?

---

**Grounds this page:**
`spec/20-verification/21-spec-syntax.md` §§5, 5.1, 5.2, 5.3, 5.4, 5.5;
`spec/60-security/64-trust-model.md` §§1, 1.1, 1.2, 4;
`crates/ken-elaborator/tests/surface_transport_acceptance.rs`
(`transport_package_adds_zero_trusted_base_delta`);
`crates/ken-elaborator/tests/ds1_empty_dec_acceptance.rs`
(`ac3_trusted_base_delta_is_ordinary_inductive_admission_only`,
`ac4_bridge_demonstrated_over_deceq_bool_not_only_deceq_int`).
Authority class: `explanatory` — this page orders and interprets those
sections and the cited fragments' own text; it does not assert a rule they
do not already state. Fragments cited are drawn from the already-selected,
registered set in [`fragments.md`](fragments.md); this chapter does not
introduce a fresh selection.
