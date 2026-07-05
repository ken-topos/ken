---
scope: enclave
audience: (see scope README)
source: private memory `soundness-AC-static-vs-runtime-face`
---

# A soundness AC often has a static face and a deferrable runtime face

When reviewing a build against a **(soundness)-tagged AC**, ask: *does this
property have a RUNTIME face distinct from its STATIC face?* Many do:

- **"No silent wrap" (L1 AC4):** static face = the bare fixed-width op *emits*
  the no-overflow obligation + the sealed dispatch (bare→obligation, `+%`→wrap);
  runtime face = an *undischarged* overflow *degrades to a runtime check
  (panic/unknown)*, never wraps. L1-build delivered the static face; the interp
  **silently wraps** an undischarged overflow ("wrapping at the interp level for
  now"), and the AC4 test does **not** assert the runtime value.
- **"Loud refusal, no silent drop" (X2 AC2):** static face = the store records
  `CapacityExhausted` (side-channel) + the discriminating test; runtime face =
  *every layer above the store surfaces it, never maps to NULL_SLOT*. X2-build
  records the error but the three real eval call-sites (`eval.rs:260/277/296`)
  **don't read `take_capacity_error()`** — set-but-unread; the eval layer still
  swallows it.

**Why it matters:** delivering only the static face is **acceptable** (no live
soundness hole — Ken still flags the obligation `unknown` / the store still
records the error, and the static net is the discriminating one) **iff** the
runtime face is a **NAMED tracked follow-on**, not a buried `// for now` comment
or an unread side-channel. The risk: "no silent wrap" silently degrades to "no
silent wrap *at compile time only*"; "loud refusal" to "loud *at the store API
only*" — the runtime path is still silently wrong, and the test's static-only
assertion **masks** the gap.

**A THIRD face — semantic-groundedness of the obligation's PREDICATES
(everyday-surface design, 2026-07-01).** Emission-completeness (the static face
— "the VC carries both conjuncts `isSorted ∧ Perm`", my L3b gate) is necessary
but **NOT sufficient**: if a predicate *named in the obligation* is itself a
**postulate** (opaque, no definition), the obligation is **vacuous** — either
**undischargeable** (you can't prove an uninterpreted predicate holds of a
concrete term) or discharged **circularly** (the "proof" postulates the
conclusion). CV's irredundancy table caught this on the LANDED verified-`sort`:
`isSorted`/`Perm` are `declare_postulate`d in `prelude.rs`, so the refinement
`{ys | isSorted ys ∧ Perm ys xs}` I approved (emission-complete) **proves
nothing about sortedness** until those predicates become real `definition`s the
prover can **unfold**. So emission-completeness (my gate) +
predicate-**definedness** are duals — *both* required or the verified feature is
honest-looking but empty. **Review tell:** for any refinement/VC obligation,
grep whether the predicates in the goal are `declare_def`/`data` (real,
unfoldable) or `declare_postulate` (opaque → vacuous obligation); a postulated
predicate is *also* a needless `trusted_base()` entry (surface TB-Sound). This
is **not** a kernel false-`proved` (the kernel certifies honestly) — it's a
**claim-honesty** gap: the feature over-claims its guarantee. State it as a
no-over-claim boundary ("guarantee pending predicate-definedness"), don't carry
it silently. Sibling of obligation must descend into structure (obligation
*shape*) — this is obligation *semantic content*.

**AUTHOR-SIDE DUAL — a static gate that fires EARLIER than the AC's stated
verdict-site is a completeness-masking bug, not a strengthening
(fs-read-file-lines-flip D2, 2026-07-04, spec-author).** The faces above are a
*review* lesson (a build delivers only the static face). The authoring dual:
when an AC pins a **runtime/driver-level verdict** (X4's AC4: an insufficient
`main` is "refused **at the driver** with `CapabilityDenied`"), the type must
carry the **declaration** and stay **polymorphic** — sufficiency belongs to the
runtime net, NOT a static type-gate. The reflex "type-enforce the capability
requirement" (make `read_bytes : Cap APartial -> …` instead of
`(a:Auth) -> Cap a -> …`) rejects the insufficient `main` at **elaboration** —
*wrong reason* — which (i) collapses the AC's discriminating pair to
green-vs-green (the insufficient arm never reaches the driver, so the pair no
longer isolates "granted==declared" from "granted full") **and** (ii) deadens
the sole runtime `authorizes` net the AC exists to exercise. Reached
independently by me, CV's SEAM-A, and Architect (α forced by AC4). **Tell:** a
proposed static gate/subtyping/obligation whose rejection point is *upstream* of
where the AC says the verdict is delivered — the earlier rejection LOOKS
stronger (fails faster) but is the completeness-masking dual of the runtime-face
gap. Static may *complement* a runtime verdict only if it does NOT pre-empt the
AC's verdict-site (else you can't build the discriminator). Sibling of
operational semantics name nonstrict positions (a paradigm reflex silently
breaks a property the obvious corpus won't catch).

**How to apply:** (1) For each (soundness) AC, name its static vs runtime face.
(2) Read the test **critically** — does it assert the runtime behavior, or only
the static emission/recording with a "for now" / unread-side-channel escape? (3)
**Grep the real runtime call-sites** (not just the test) for whether they
*consume* the propagated signal (the obligation hole gating eval; the
side-channel being read). (4) Don't certify "fixed" / "no silent X" when only
the static face is done — state **precisely which face** is delivered (honesty;
avoids the over-vouch of scope review vote to my lane) and require the runtime
face be a tracked follow-on. Validated 2x back-to-back (L1-build, X2-build,
2026-06-30). Complements untrusted layer backstop hole for omissions (name
what's not delivered).
