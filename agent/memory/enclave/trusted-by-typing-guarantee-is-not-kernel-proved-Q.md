---
scope: enclave
audience: (see scope README)
source: private memory `trusted-by-typing-guarantee-is-not-kernel-proved-Q`
---

# A by-typing trusted guarantee is not kernel-proved; it projects to P, never Q

When a behavioral/assume-guarantee export defines its **guarantees field `Q`
operationally as kernel-certified** (e.g. B1 `71 §2.1` invariant I1: `Q` iff the
certificate `check`s **and** the goal ∉ `GlobalEnv::trusted_base()`), a claim
that Ken established by a **trusted meta-theorem rather than a kernel
certificate** must **not** land in `Q`. The IFC/`@ct` by-typing flow rules are
exactly this: labels are **erased before the kernel** (N1), so the kernel issues
no certificate for the property — it is *proved-by-trusted-typing*, which §H
honestly calls "proven by typing — **but the flow rule is trusted**." Such a
claim projects to **`P` tagged `tested`** (a trusted-by-typing boundary
obligation), never the kernel-proved `Q`.

**Why:** filing a trusted-by-typing guarantee under `Q` **over-claims kernel
certification** to a downstream that reads `Q` as kernel-proved — the precise
no-over-claim failure the export's honesty discriminator exists to prevent.
Under-claiming it as `P`/`tested` is the **safe direction** (honest about the
weaker provenance); over-claiming is the dangerous one. (Open refinement: a
trusted-by-typing guarantee is a *third* epistemic category the proved/tested/
delegated/unknown model collapses — `P`/`tested` slightly mislabels a
Ken-*established* guarantee as an "assumption"; a distinct "assured-by-trusted-
meta-theorem" status may be warranted if the under-claim is too lossy. Decision-
worthy, not urgent.)

**How to apply:** (1) For any security/verification property that is
trusted-by-typing (erased-before-kernel) AND produces an export entry, route it
to `P`/`tested`, never `Q`. (2) **Concrete live erratum (2026-06-30):** Sec1ct
`61 §5a.4` + conformance `ct/seed-ct.md` CT-D1 (both on `main`, `af14bf3`) say
the CT-in-parameter promise "emits onto the **guarantees (Q)** channel" — wrong;
B1 `71 §2.1` correctly routes it to `P`. Reconcile both Sec1ct pieces to
`P`/`tested` (verify-both-on-main, multi piece erratum landing integrity); the
fix is on Sec1ct, off B1's branch (B1 is the authority). (3) **Meta-method:**
this surfaced only by reviewing the two **coupled** WPs (Sec1ct, then B1)
back-to-back — extend the cross-case consistency sweep (verdict mapping silence
is a latent conformance bug) **across WP boundaries**, not just within one seed;
coupled specs (a producer + the contract that consumes it) are where the
field/verdict-mapping inconsistency hides. Specializes untrusted layer backstop
hole for omissions (the erased-before- kernel trusted layer) to the
export-projection domain.

**The mirror error — UNDER-weighting a trust characterization (Lc coherence,
2026-07-01).** My usual catch is an OVER-claim (trusted→`Q`); here I made the
opposite. I APPROVED (and wrote into the resolution) that typeclass
**coherence** is "elaborator-convention **predictability, not soundness**." But
locked **ADR 0008** (lines 20-21/34) frames it as **"soundness-*adjacent* for
*client* reasoning, not merely an ergonomic preference"** — the *third category*
this note anticipates, now concrete. Flattening it to "just ergonomics"
under-weights the client-reasoning layer. The precise picture (worth reusing):
in a **dependently-typed** setting coherence is **not a kernel-soundness
requirement** — Ken reifies dictionaries as **typed, law-carrying values**, so
an incoherent pick surfaces as a **type mismatch the kernel catches** (or
resolves a *different but still-lawful* instance) — **never a false proof** (the
Haskell-*erased*-dict contrast is load-bearing). Yet it **is**
soundness-adjacent because a client lemma "about *the* `Monoid A`" relies on
canonicity; the kernel backstops type-correctness + law proofs but does **not**
enforce the *convention* (canonicity/orphan/overlap) — that is elaborator-level,
conformance the sole net. **How to apply:** (a) check a trust characterization
against the **locked ADR**, not just first-principles — a locked
"soundness-adjacent" must not be flattened to "ergonomic/predictability"; (b)
when my own already-resolved review record carries the under-weighted phrasing,
**amend the resolution** (honesty-on-the- record) — verdict stays,
characterization corrected — and hold the merge so the ADR-aligned prose is what
lands. Same trust-precision discipline, opposite sign.

**Tier vs projection — state BOTH, not one for the other (Sec5 policy,
2026-07-01).** A trusted-by-typing guarantee has two distinct coordinates: the
**tier** = *trusted* (not kernel-proved) and the **status projection** =
`P`/`tested` (never `Q`). Writing only "trusted, never `Q`" (my Sec5 AC4 draft)
**under-specifies** — it names the tier but drops the projection a consumer
reads off the status field; writing only "`P`/`tested`" drops the *why* (the
trusted tier). The honest two-part line: **"trusted-by-typing → `P`/`tested`,
never `Q`/kernel-backed."** Reconciled my Sec5 seed's two sites to it against
the landed `65 §4` body before commit → clean Fidelity, no erratum. Applies to
every `60-security/` + `70-behavioral/` trust-level line.

**Forward-net: pin a corrected trust level in each NEW consumer (Sec6,
2026-07-01).** Once an erratum fixes a projection (here the `@ct` →
`P`-never-`Q` correction, `61 §5a.4`), the fix can **silently regress** as *new
consumers* of that projection land later. So when a WP adds a consumer of a
corrected trust-level projection, author a discriminating case pinning the
erratum's invariant **in the new context** — a cross-WP no-regression net. Live:
Sec6's discharge attestation is a new consumer of the four-way status; its
`qct-channel-discharge-stays- delegated` case pins that a `Q@ct`-channel
discharge stays `delegated`/`tested` (never kernel-`Q`) **despite the `Q@ct`
label** — netting a build that special-cases the `Q@ct` channel as
kernel-certified, and holding the `61 §5a.4` correction forward. Both reviewers
flagged the tie independently. The erratum correctness lives across every
consumer, not just the site it was fixed at.

**Classify epistemic status, NOT counterparty mechanism (Sec6 trust boundary,
2026-07-01).** For any seam where Ken **consumes** a lower-trust counterparty's
results (the Ward discharge attestation), Ken's contract depends on the result's
**epistemic status** (decided / partial-under-a-stated-bound / observed /
negative), **never** on the **mechanism** that produced it (depth-`k` vs sample,
the policy, the CT method). One principle justified **both** Sec6 ratification
calls: the `bounded` single label (depth-`k` vs sample = *mechanism* →
counterparty-internal, so no fifth label) **and** the Ward-internal boundary
(Ken couples to *that* it discharged + the pinned `ward.version`, never *how* —
no Ken correctness judgment may read `policy`/`bound`/`evidence`/`ct.method`/
`regression`). **Collapsing a mechanism distinction into one epistemic label
loses no soundness** (Architect-confirmed: a strict consumer rejects the whole
weaker-status class, e.g. "no `bounded` on `Q@ct`"; the finer split is a
*policy-expressiveness* limit gated on the counterparty-internal value, not a
soundness hole — fallback-to-two-labels is forced only by a soundness
distinction, and none exists). **Meta-rule for a co-owned contract:** a
**narrowing** (a carried field reclassified counterparty-internal) is ambiguous
— a *new Ken divergence* vs *Ken catching up to the counterparty's finalized
contract* — and only the **upstream artifact** disambiguates (out of your
clean-room lane), so **flag it to the cross-repo owner (Steward), don't
assert**. Live: §5a's policy/sampling→Ward-internal narrowing was Ken catching
up (Ward's handoff already excluded them); the Steward's cross-check closed it.
Generalizes the tier/projection discipline above from a single guarantee to the
whole consuming-seam abstraction boundary.
