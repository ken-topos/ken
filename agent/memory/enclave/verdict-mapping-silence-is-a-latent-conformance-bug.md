---
scope: enclave
audience: (see scope README)
source: private memory `verdict-mapping-silence-is-a-latent-conformance-bug`
---

# A verdict-mapping silence is a latent conformance bug

The spec-author duty "resolve silences when structurally determined" has a
category I was blind to until V3: **verdict / classification-outcome silences.**
I'd reliably resolved every **Ω-typing** silence (does this body check at Ω?)
and every **level** silence (what level does this formation land at?) — but a
rule that names a procedure's *outputs* without pinning *which verdict each
output yields* is the same kind of gap, and it is **load-bearing**.

**Ken V3 (`23 §5`, shipped `c43cdfb`).** I wrote the IPC decision procedure
"returns a **proof term** or a **Kripke countermodel**" — without pinning which
**verdict** a counter-model produces. That unpinned mapping is exactly where the
conformance-validator's *independent* re-derivation diverged from mine: a
counter-model reads naturally as `disproved`, so case D2 mapped LEM `p ∨ ¬p` →
**disproved**. But a **classically-valid** formula is **never refutable**
(**Glivenko**: `¬¬φ` is intuitionistically provable iff `φ` is classically
provable ⇒ for any classically-valid `φ`, `¬φ` is unprovable ⇒ **no Kripke model
forces `¬φ`**); `24 §3` puts `p∨¬p` in the **unknown** region `{¬¬φ} ∖ S_φ` (the
`¬¬φ ⇒ φ` gap). So the honest verdict is **unknown** (a counter-to-validity
model, not a refutation), never `disproved`. **The silence I left as author
became the bug the validator inherited** — the author-side mirror of conformance
reconcile inherits spec metatheory bugs (that one is reviewer-side: matching a
*wrong* spec body yields a wrong case; this one is an *absent* spec ruling
yielding a wrong case).

**The discriminating tell — a cross-case contradiction on overlapping
metatheory.** The seed contained A3 (`¬¬p⇒p` → **unknown**, correct) and D2
(`p∨¬p` → **disproved**, wrong) — the *same* metatheoretical situation
(classically-valid, intuitionistically-invalid) with *opposite* verdicts. That
contradiction is the signal the spec under-specified the mapping (exactly the
"two contradictory cases on overlapping inputs ⇒ the spec is ambiguous" tell
from obligation must descend into structure, here on the *verdict* axis instead
of the *granularity* axis). I caught it in the Spec-vote review by **independent
re-derivation against `24 §1`/§3** (verify the property, not match my own
§-body), then fixed it at the **source** (`23 §5` now pins: a model *forcing
`¬φ`* = `disproved`; one merely *failing to force `φ`* while `¬¬φ` holds =
`unknown`) **and** the validator fixed the seed case.

**Confirmed: applying it at the source PREVENTS the recurrence (V4, 2026-06-30,
`2d7a09c`).** In `24-diagnostics` I pinned the verdict→diagnostic mapping **up
front** — a "cardinal rule" preamble + master table (`24 false ⟺ V3 disproved`,
`unknown ⟺ V3 unknown`, the tag is a *projection* of V3's verdict, never an
independent re-read of the Kripke model) — **before** the conformance-validator
seeded. Result: the same D2-class silence that bit V3 **never opened**, and the
cross-case sweep came back **clean on first pass** (`{A2,B2,D2}` all `p∨¬p`/
`¬¬p⇒p`→`unknown`; `p∧¬p`→`false` the lone refutable contrast, `¬(p∧¬p)` a
theorem ⇒ `¬φ` forced). The federation learning-loop closing: last WP's
*topology-touching* trap, promoted to the playbook, actually prevented its own
recurrence the next WP. The discipline is real, not just post-hoc narration.

**Re-confirmed one layer out (T1 `25-protocol`, 2026-06-30, `07a59f9`).** The
same source-pin generalized from the diagnostic-*value* layer (V4) to the
*serialization/wire* layer: I opened `25` with a **wire cardinal-rule
cross-walk** (V3 verdict → doc `status` → obligation `status` → diagnostic
`kind` → countermodel `verdict` tag → legal actions) + the "one source, three
fields cannot disagree" invariant + Glivenko-on-the-wire. The independent
conformance-validator confirmed it **pre-closed the verdict-silence + Glivenko
traps at the source**, so its lock-points came back as *conformance-observable*
questions (field-spelling boundary, id-stability non-degeneracy, reject-
observable) — **zero spec-semantics ambiguity, zero verdict-mapping rework.**
Cross-*layer* replication (value-layer → wire-layer) is a stronger
generalization signal than same-layer repetition (COORDINATION §10): pin the
verdict→message mapping at **every** boundary a verdict crosses (decision →
value → wire), each as a projection of the one upstream verdict, never
recomputed downstream.

**Re-confirmed in a new *domain* (Sec1 `61-information-flow`, 2026-06-30,
`18b45b2`).** The source-pin reached the **relational/security** domain, not
just another layer of the same spine: a non-interference **relational
obligation**'s outcomes pinned to the V3 trichotomy at the source (`61 §5.3`) —
a **distinguishing pair** (two runs whose ζ-observable outputs differ) →
**`disproved`-with-witness** (the pair *is* the leak-witness), never a naïve
`unknown`; an **unprovable** relational claim → `unknown`-with-hole, never a
false `proved`. Same trap shape as V3's IPC-countermodel (a *distinguishing
artifact* reads naively as the wrong verdict). Pinned **at design time** via the
CV lock-point protocol — now **3/3 (V4→T1→Sec1)** of design-time pinning →
green-on-substance first pass, zero verdict-mapping rework. The metatheorem to
keep ready: **in a 2-safety/relational setting a distinguishing pair is a
*refutation* (`disproved`), never `unknown`** — the relational analog of
"classically-valid is never disproved" (Glivenko). Sibling of untrusted layer
backstop hole for omissions N2 (the *reduction* producing that obligation is
itself trusted — cert-recheck ≠ reduction-faithfulness).

**Re-confirmed in a new *domain* (B1 `71-assumption-boundary`, 2026-06-30,
`5808e59`).** The source-pin reached a **generated/projected IR** — the export
emitter's **status → export-field** map. The trap here hides specifically in the
**edge statuses the schema table omits**: the v0 §2 table named the
*interesting* mappings (`proved`→`Q`, `tested`→`P`, `delegated`→`T`) and was
**silent on the residual two** — an open typed hole (`unknown`) and a refuted
claim (`disproved`). That omission is exactly where projection *honesty* lives:
leave it and an `unknown` hole reads naturally as a `Q` guarantee (a silent
**over-claim**), or `disproved` reads as exportable (shipping a known-false
guarantee). Pinned **at design time** (§2.1): **`unknown`→`P`** (a hole is a
postulate, rides the assumption boundary, never `Q`); **`disproved`→never
exported**; and crucially the **discriminator is kernel-side/structural**
(`21 §5.4`: `trusted_base()` membership + certificate presence), **not** the
untrusted V-layer's self-reported status string — so the honesty net can't be
forged by the layer it audits. The reusable rule: **a projection/emitter's
status→field map is a verdict-mapping surface; enumerate the FULL source-status
domain and pin each element's image — including the "never exported" ones — at
the spec, discriminated by a structural signal, not a self-reported label.**
Generalizes the rule from *decision→value→wire* (V3/V4/T1) and *relational*
(Sec1) to *any projection's classification boundary*. Companion: B1's no-measure
`G` seal is the same erased-before-kernel omission-hole as Sec1ct's erased
labels (kernel sees neither the export bytes nor the labels ⇒ seal at the
untrusted projection point, conformance is the sole net) — untrusted layer
backstop hole for omissions. Now **4/4 design-time pins (V4→T1→Sec1→B1)** →
green-on-substance first pass, zero verdict-mapping rework.

**Re-confirmed in a new *domain* — and a new *shape* (B3 `73-conformance`,
2026-06-30, `3c6cbb7`).** The source-pin reached a **runtime/dynamic** outcome —
the first that is **not** a static-artifact classification. A live monitor's
**accept/reject** over a trace had to be pinned, and the resolution is the new
shape: the outcome maps to **none** of Ken's `{proved,unknown,disproved}`
statuses — **accept** = monitoring *evidence* for a `delegated` obligation that
**stays `delegated`** (a finite green run ≠ a proof for all behaviors → no
promotion); **reject** = a **consumer-side conformance violation** that **does
not re-verdict** Ken's conditional `Q` ("given `P`, then `Q`" ⇒ a runtime
violation is assumption-side, not a wrong certificate). So a verdict-mapping
resolution can legitimately be **"this outcome does not enter Ken's status space
at all"** (the one-way/emit-only mapping) — and that *must still be pinned at
the source*, else a consumer launders a monitor verdict into a Ken `proved` (the
G-Ward-seam break). Both gates called it the load-bearing silence. The rule now
generalizes past *any procedure that returns X or Y* to **any checker whose
*outcomes* don't auto-pin a status — offline prover, online monitor, classical
engine alike** — and the pinned image may be *outside* Ken's verdict space. Now
**5/5 design-time pins (V4→T1→Sec1→B1→B3)** → green-on-substance first pass,
zero verdict-mapping rework. (Couples to the one-way-gate seal — untrusted layer
backstop hole for omissions: the monitor verdict, like the erased label and the
export bytes, is invisible to the kernel ⇒ the no-promotion gate is
conformance's sole net.)

**How to apply.** (1) **As author:** when a rule names a procedure's outputs
("returns X or Y", "succeeds or fails", "emits a cert or a model"), **pin the
verdict/outcome each output maps to** at the source — treat a verdict-mapping
silence like an Ω-typing or level silence (resolve-or-escalate, `§6`). Ask
"could two reasonable readers map this output to *different* verdicts?" (2) **As
Spec-reviewer:** run the **cross-case internal-consistency sweep** — scan for
*contradictory verdicts on overlapping metatheory* (two cases whose inputs are
the same logical class but whose expected verdicts differ) — **not just**
per-case verdict-flip. A per-case sweep passes both A3 and D2 individually; only
the cross-case sweep collides them. The validator has adopted this as a hard
pre-handoff gate. (3) The general metatheorem worth keeping ready:
**classically-valid ⇒ never `disproved`** (Glivenko) — such a goal is `proved`
(if intuit-valid) or `unknown` (if not), and `disproved` is reserved for
**genuinely-refutable** goals where `¬φ` is provable (e.g. *decidable* false
atoms: `2+2==5`, `n>0` on `n≤0`).
