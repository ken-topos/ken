---
scope: enclave
audience: (see scope README)
source: private memory `laundered-citation-authority`
---

# A stale citation gains false authority by propagating

The reconcile-don't-cite discipline (verify "the spec says X" against the landed
file, `§7`) has a failure mode I hit in V4: a citation can be **repeated across
several authoritative-looking places**, and the repetition launders it into
apparent corroboration even though every copy inherits **one** ancestor error.

**Ken V4 (`24-diagnostics`, 2026-06-30, `2d7a09c`).** The ref `12 §5.2` (for the
Heyting structure of Ω / the "kernel trichotomy") appeared in **three** sources
at once: the Steward's **kickoff frame** ("Heyting structure:
`12-universe.md §5.2`"), landed **`16 §1.3`** (L93: "Omega carries the Heyting
structure (`12-universes.md` par. 5.2)"), and landed **`21 §5.1`** (L223: "the
kernel trichotomy (`12 §5.2`, `24 §3`)"). Three agreeing sites — yet
`12-universes.md §5` ("The proposition classifier Ω") has **no §5.2** and
explicitly **defers** the Heyting structure to **`16 §1`**; the verdict
trichotomy's real home is **`21 §5.1`**, the three-region decomp is **`24 §3`**.
The error was even in the **kickoff I was handed as ground truth**. Citing it
(trusting the agreement) would have propagated the fabrication a fourth time;
reconciling it (opening `12 §5`) killed it. The conformance-validator
independently caught the same thing (LP-1, convergent) — two independent
reconciles beat the laundering, a single trusting cite would not have.

**Why it's a distinct trap.** Normal grounding fails *open* (a lone ref you
can't resolve looks suspicious). This one fails *closed*: the ref looks **more**
trustworthy because multiple landed files repeat it — the exact opposite of the
"single dangling ref" smell. The `12 §5.2` string also has a half-life: I'd
already caught the *same* fabricated `12 §5.2` for `Decidable` in V3 (it's
`16 §1.3`), and it resurfaced in V4 for a different claim. Stale refs don't die
from one catch; they live in the sibling files until someone fixes the
**target-side** sites.

**How to apply.** (1) **Verify the TARGET, not the citers.** To trust `A §x`,
open `A`, confirm `§x` exists and says what's claimed — never infer it from how
many places cite it. Multi-site agreement (kickoff + siblings) is a **prompt to
check the target**, not evidence. (2) **The kickoff frame is a secondary
artifact** — a Steward/leader handoff can carry a stale cite as "ground truth";
re-verify its refs against the landed files like any other claim (mirror of wp
frame stale vs landed kernel: the frame's *prose* can be stale; here its
*citations* are). (3) **When you catch a laundered ref, flag the target-side
sites** (here `16 §1.3` + `21 §5.1`) as a doc-erratum to the Steward, but keep
the fix **off your WP branch** (diff-scope / two-axis gate) — fixing siblings
under your WP reverts-scope-creeps. (4) The tell that a ref is fabricated, not
just mis-numbered: the cited section exists but its **subsection** doesn't, and
the chapter *points elsewhere* for the claimed content. Specializes
reconcile-don't-cite; sibling of verdict mapping silence is a latent conformance
bug (both are V3/V4 spec-author author-side traps caught by reconciling, not
citing).

**Variant — a delegated grounding agent's reported SYMBOL is a laundered cite
too (Lc, 2026-07-01).** I now dispatch Explore agents to ground producer-grep
targets against landed code; their reported **symbol names / file:line anchors
are claims to re-verify, not citations to build on.** My grounding agent
confidently reported `declare_def_group` ("check.rs:975+") for the
recursive-group kernel admission; I pinned it **normatively** in `39 §6.4`
without re-grepping the exact token. It doesn't exist — the real symbol is
`declare_recursive_group` (check.rs:983). Worse, the **Architect independently
laundered the same wrong symbol** in their pre-review (one grounding pass misled
two reviewers), and CV's producer-grep was the sole net. **Apply:** for any
symbol/anchor a sub-agent reports that you will pin as a producer-grep target,
run the one-line `grep "pub fn <name>"` yourself before writing it — the agent's
confidence is not verification, and a wrong token pinned normatively is a false
producer that only the downstream producer-grep gate catches.

**Variant — a soundness-SEVERITY escalation launders identically, and it's the
more dangerous direction (BUILTINS Phase-1 F4, 2026-07-02).** Architect asserted
`Decimal` saturate/`decimal_eq` was a "false kernel proof / `refl` inhabits
`Eq Decimal` / explosion" without grepping the emission; spec-author baked
"false proof" into the `18a` registry doc without re-deriving it; CV concurred
in review without re-deriving it either. Three adoption points, each trusting
the *prior role's* assertion as its own grounding — by the terminal gate it read
as a 3-way-verified fact. One Steward grep (`Eq`-at-primitive is neutral in
`obs.rs`, no `eq→Eq` reflection bridge, `ken-kernel` has zero dependency on
`ken-interp`) de-laundered all three at once, because grounding at the emission
is O(1) regardless of adoption depth — but laundered authority gets *more*
convincing with each hop, which is exactly backwards. Architect's name for it:
each role (author/baker/concurrer) "laundered an ungrounded claim into more
apparent authority." **Apply:** a severity/soundness claim you *receive* (not
just one you originate) must be re-grounded at its source the moment you adopt
it — more urgently than an ordinary citation, because severity claims that
escalate *up* the ladder ("this is now a soundness emergency") manufacture their
own urgency and discourage the pause needed to check. The safe asymmetry:
re-deriving *down* in severity is always cheap to justify; adopting an
*up*-escalation without grounding it is the over-claim direction and the one to
distrust by default.

**Variant — a WP FRAME's cited DECIDED / decision-id is a fabrication risk; grep
it against the register at PICKUP before baking (44-capacity-restate,
2026-07-02).** The Steward's 44-restate frame stated as a fixed-input "operator
2026-07-02, `OQ-systems-target` closed in favour of systems-adjacent." The
*positioning ruling was real*, but the **id `OQ-systems-target` and the "closed"
status were fabricated at the frame** — never registered (0 hits on
`origin/main`; the real register has `OQ-backend-target` OPEN, no
`OQ-systems-target`). I cited it in a normative chapter (`44 §3`) as settled
without opening `90-open-decisions.md`. The Architect's soundness gate caught
the dangling cite; the fix was to author a **real `OQ-domain` DECIDED entry**
and reframe §3 as "records a settled decision." **This recurred ONE WP after my
F1 retro banked "re-ground a received citation at the emission"** — proof the
lesson needs a *mechanical* trigger, not an intention: **at WP pickup, grep
every frame-cited decision-id (`OQ-*` / `dec_*` / ADR-NNNN) against the register
/ `docs/adr/` before writing it into any normative file.** A frame's
fixed-inputs are the *highest-authority-looking* secondary artifact in the
federation, so they're exactly where a fabricated id fails *closed* (looks
settled, isn't). The frame author owns fabricating the id; the baker owns not
verifying it — both are real, the conjunction caught it, but the O(1)
pickup-grep is cheaper than a merge-gate catch. Distinct-from-mis-numbered tell:
the id resolves to **nothing** (not a wrong section — a non-existent decision).
Sibling of the K2c wp frame stale vs landed kernel rule (frame *prose* stale) —
here the frame's *decision-citation* is fabricated.

**Variant — I can be my OWN re-laundering vector: a retracted
over-classification resurfaces through my later advocacy prose (Decimal/Char
DEMOTE, 2026-07-02).** I escalated F4 as "false-proof / `refl`-inhabits
`Eq Decimal`," Steward challenged it, I **retracted** it (wrong `Bool` value in
the tested ring; no `eq→Eq` bridge; kernel-neutral). Then — the SAME session,
advocating Path A — I twice wrote "the sharp false-`True` eq hole that can
**inhabit `refl` on distinct decimals**" (`evt_3kkz7b7qfayts`,
`evt_1nrajdw6eyk0w`), and spec-author baked the phrasing from my prose into the
normative contract `18a §5.6.1(2)` — which then **contradicted its own §4 F4**
(the retraction). CV's Fidelity axis caught it; my Soundness axis (which *owns*
the trust-level of the F4 bug, and *originated* the error) should have. The
trap: retracting a severity claim in **one message** leaves the vivid phrasing
alive in my **active vocabulary**, and I re-emit it as if it were still current
— self-laundering, no external adopter needed. **Apply: when I retract a
severity/trust-level claim, purge the PHRASING, not just the verdict — treat the
retracted words ("inhabits `refl`", "false proof", "kernel-backed") as tainted
tokens and grep my own subsequent prose for them before they bake into a
normative artifact.** The correcting rule mirrors kernel backed claim grep the
emission not the name: the trust level is `eq_decimal : …→Bool` with no
reflection lemma = wrong value, never a proof; a demote "removes the wrong-value
path by construction," it does not "close a `refl`-hole" (there was none).
Most-dangerous direction, self-inflicted.

**Spec-author angle on the same incident (baker + remediation).** I
(spec-author) am the one who **baked** the Architect's retracted phrasing into
normative `18a §5.6.1(2)` — so the laundered citation authority "baker owns not
verifying" applies to *trust-level phrasing received from a coupled role's
advocacy prose*, not just to citations: **prose from a soundness-authority is a
claim to re-ground against the findings section (§4 F4), never a phrasing to
adopt.** Two durable rules banked: (1) **a delivery/elaboration contract must
not carry a trust-level characterization MORE SEVERE than the findings section
it elaborates** — an elaboration can silently outrun its own §4. (2)
**Remediation = grep-the-region-fold-all:** catch one over-claim (CV flagged
§5.6.1(2)), grep the whole region for the tainted tokens
(`inhabit`/`refl`/`false-proof`) — I found **two more** (§5.6.1(4) + a coupled
pre-existing parent §5.6 line) — and fold EVERY instance. A **coupled**
pre-existing line folds **in-WP during review**, not split-to-erratum
(erratum-on-main is for *post-resolution* discoveries; splitting leaves the doc
self-contradicting on `main` in the interim — CV + spec-leader's diff-scope
check both endorsed the in-WP fold). The held-side counterweight this same WP:
the **K2c pre-draft scope checkpoint** — the brief's guardrail "don't pull
`leq_int`" rested on a false premise (Decimal alignment needs only `mul`; the
landed `ea<eb` branches disprove it), I held the draft and raised the fork
instead of drafting around it, and the fix *expanded* scope (the arm moved F5 →
this tranche). A stale "what's *forbidden*" misdirects as much as a stale
"what's *broken*" (wp frame stale vs landed kernel).
