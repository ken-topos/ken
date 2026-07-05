---
scope: roles/conformance-validator
audience: (see scope README)
source: private memory `conformance-validator-casts-spec-review-vote`
---

# conformance-validator casts the Spec review vote on every
spec/conformance Decision

**Operator decision, 2026-06-30, effective from the Sec1ct CT-D1 erratum
onward.** The **Spec review vote** on every merge Decision touching
`spec/`+`conformance/` is cast by **me, conformance-validator**
(`agt_37reqfr97xm00`) — *not* the dead `Spec` placeholder (`agt_37rekz81ceg00`:
`participant_type: agent` + `agent_adapter: null`, a non-running `moot init`
template actor, **never** a reviewer) and *not* spec-leader.

**Rationale (structural, model-agnostic):** CV *is* the independent-validation
role by design (re-derive + ground + reconcile-don't-cite), distinct from
spec-author who authors `/spec` — so the Spec gate is the highest-judgment
**independent** check, cast by the validator, not the author or the coordinator.

**The split — and the invariant it preserves (each piece checked by a
non-author):**
- **spec-author** authors `/spec`, never reviews its own work.
- **I** cast the **Spec** vote: attest the spec is correct via by-role
  independent validation (re-derive every structural claim from first
  principles, ground each cross-ref at its target, internal-consistency sweep) —
  not a content-match against the §-body.
- **spec-leader** proposes/assembles/routes the Decision; does **not** cast
  Spec. When assembling, names me as Spec reviewer with a real @mention to my
  actor_id (the dropped-mention failure mode of architect gate can be skipped
  review on main).
- **Architect** = external soundness gate, **always**.
- Independence preserved: **Architect checks my conformance; I check
  spec-author's spec.** When I author a conformance piece in the *same* Decision
  (e.g. Sec1ct CT-D1), I wear both hats — author the conformance edit AND cast
  Spec on the spec-author's spec edit — with the Architect the independent gate
  on my conformance half. No piece is self-reviewed on the axis that matters.

Verify any non-Architect review target is a *running* agent
(`agent_adapter: "mcp"`, recent `last_seen_at`) before trusting a routed vote —
the `moot init` placeholders are dead actors.

**Validated across 3 exercises (2026-06-30): librarian ASCII→Mermaid (caught a
diagram coupling the Architect's fidelity pass missed), Sec1ct CT-D1, L1
numbers.** All three: APPROVE scoped to my lane (semantic fidelity + conformance
validity), soundness explicitly disclaimed to the Architect, Decision left open
until both votes recorded (never resolve on my vote alone, §14).

**The reviewer hat extracts *more* than the author hat on the same artifact.**
When I *author* conformance I resolve a spec ambiguity by silently picking the
right semantics and moving on; I don't flag that the *spec* was ambiguous.
Wearing the Spec-reviewer hat forces the **"would a build team conflate this?"**
question, which surfaces author-side terminological/mapping ambiguities my
authoring alone wouldn't. L1: I encoded AC3's degrade as panic/`unknown` (right)
without noticing `35 §3.2` overloads **"checked"** — the *subsumed* runtime face
of an undischarged obligation (panic/`unknown`) vs the explicit
`checked_add → Option T` op class (`None`, no panic) — a conflation risk a build
team could hit. Only the reviewer pass caught it (a verdict mapping silence is a
latent conformance bug instance: one label → two distinct runtime behaviors). So
on a both-hats Decision, run the *reviewer* pass over the spec as its own gate,
not as a by-product of authoring — they catch different defect classes.

**Now 2/2 → standing practice (X2 confirmed, 2026-06-30).** Authoring
`seed-capacity` I cited `44 §2`'s "Placed in `43 §2` fault taxonomy" loosely and
never opened `43`; the independent Spec re-derivation did —
`43-termination.md §2` is a flat 4-item list with **no resource-fault class**
and was untouched on the branch, so `44 §2`'s placement (+ its `43 §2.2`
sub-cite) is a **dangling forward-ref** shipping into an authoritative spec (the
laundered citation authority pattern). The authoring pass cites *outward* and
structurally cannot catch this; only the reviewer pass *verifies the cite
resolves to real content*. **Standing rule: on a both-hats Decision the Spec
re-derivation MUST open every cited cross-file target and confirm it hosts the
claimed content** — distinct from per-case verdict-flip and from confirming the
§-number merely resolves. Tell: a cross-ref that resolves to a §-number but not
to the claimed *content*. (Both X2 flags — this + the
`total_interns`-false-under-exhaustion invariant — were non-blocking and shipped
UNFIXED under resolve-and-track at `e18f4aa`, becoming live errata; architect
gate can be skipped review on main erratum-on-main discipline applies — flag
them so "non-blocking" doesn't silently evaporate.)

**3/3 — L6 confirmed CLEAN (first clean run post-X2, 2026-06-30).** On `38 §1`
(Bytes + binary I/O) the Spec pass opened all **8** cited targets (`14 §5`,
`41 §3a`, `36 §1.4`, `31 §3`, `14 §8.4`, `34 §5`, `18 §5`, `43 §2`) — every one
hosts the claimed content. The rule **confirms good cross-refs, not only catches
bad ones**, and discriminated `38`'s *legitimate* `43 §2` partiality cite (item
2, partial-primitives) from X2's *dangling* `43 §2` resource-fault slot — **same
§-number, opposite verdict** is exactly the content-not-number check. So the
standing rule's value is symmetric: it greenlights clean elaborations as fast as
it flags dangling ones (vote APPROVE, no flags, merged `cb90621` no errata).

**Subsume-don't-proliferate is a CROSS-WP discipline (L6 carry, 2026-06-30).** A
kickoff's QA gate can over-scope its literal ask: L6 said "route a real I/O
signature through the actual `36 §1.4` escape check" — but that gate already has
its conformance home in the L5 `surface/effects/seed-effects.md` seed.
Re-pinning it = proliferation; assuming L5 covers L6 = under-coverage. Resolve
by separating the **bug targets** (*L5*: gate fails to check `⊆`; *L6*: an I/O
primitive declared **without** its mandatory row) — the new WP's cases
**reference** the prior home and pin only the **delta** (the operation-row
binding). The independent-checker move: locate the existing home **before**
authoring, assert the delta against it. Generalizes the within-file subsume (one
home per property) to the corpus level. Same review also: both-hats independence
held a 4th time clean; the off-grid-witness/fixed-point trap transferred to a
3rd domain (NFC renormalization — the non-NFC witness `0x[65 cc 81]`).

**5/5 — the cross-ref rule is AUTHOR-AGNOSTIC (B3, 2026-06-30, first run on a
seed I did NOT author).** X2/L6 were my own conformance (both-hats). B3's
`seed-trace.md` was *spec-author's*; the standing rule held **identically** —
opened every cite (`71 §2.1/§3.1/§3.2/§3.3/§5.1/§5.2/§6`,
`36 §2.1/§3.1/§4.1/ §4.4`, `41 §3`, `72 §3`, `21 §5/§5.4`, `63 §5a`, `export/`
EX-A/EX-C1/EX-E1), all host the claimed content. Near-trap survived it: I
expected a `63-discharge-attestation.md`; the file is **`63-supply-chain.md`**
and §5a hosts the real discharge-attestation content — resolved by checking the
**§-content, not the filename** (the same content-not-number check, the
filename-in-my-head being the wrong axis). So the reviewer-pass is a pure
independent re-derivation that works the same reviewing *anyone's* conformance,
not a self-check on my own authoring. Also B3-specific: a **load-bearing
reconcile-against-landed-code** claim (B1's `Σ` is effect-LABEL granularity, so
`Op`/`Resp` concretizes downstream) MUST be verified against the CODE
(`export.rs:162` = `alphabet: BTreeSet<String>`), not the spec prose (`71 §2`
narrative ascribes `Op`/`Resp` *verbatim* to `Σ` — the prose is ahead of the
landed code, the L5/L6 staging-staleness pattern again).

**Verify-on-main catches dropped INDEX pieces, not only dropped NORMATIVE pieces
(B3 carry).** multi-piece-verify-all-on-main found `conformance/ README.md`'s
**Seeds index** un-updated for `behavioral/trace` (export listed line 94 +
layout line 23 name only the export emitter; the B3 seed landed but its index
entry didn't — README untouched by the merge diff). Non-blocking (CI gates the
*seed*, not the index) but the human-facing corpus index is now incomplete. By
the seed-author-adds-the-Seeds-line precedent (I did it at L6), this is a
dropped *deliverable* piece even though it carries no normative content. So
verify-on-main must check the **index/README pointer** landed too, not just the
spec+seed+invariants — and flag the gap so "non-blocking" doesn't evaporate (the
erratum-on-main discipline of architect gate can be skipped review on main).

**6/6 — re-derive every spec *example* against the normative rules, not just
reconcile the §-body (L2, 2026-06-30).** Cross-ref sweep held 6/6 (2nd on a
non-self-authored seed) and *validated* spec-author's own `12 §5` (= Ω
classifier) vs `21 §2` (= refinement former) hygiene correction — the rule
greenlights a good correction as fast as it flags a bad cite (the L6 symmetric-
value property again). **New trap sub-class caught (the carry):** a spec
*illustrative example* can contradict its own normative rules **while every
§-body content-reconciles clean**. L2 §5's
`head {a} (xs : NonEmpty a) = match xs { Cons x _ => x }` (omits `Nil`) is
**non-exhaustive** under §5's *own* carrier encoding + §4.1: `NonEmpty a` →
carrier `List a`, the `xs ≠ Nil` proof **erased**, so inside `head` the
scrutinee is `List a`, `Nil` is type-possible, **no proof in Γ** to refute it →
§4.1 *requires* the `Nil` arm. It conflates refinement-non-emptiness (erased ⇒
zero coverage effect) with §2's *index*-non- emptiness (in the type ⇒ drives
§4.3 absurdity-omission, soundly pinned by AC5). A §-body reconcile **passes**
it (§5's encoding prose is internally consistent); the defect only surfaces by
re-deriving the *example's behavior* against a **different** section's rule
(§4.1 coverage over the §5 carrier). Non-blocking (normative rules + corpus are
sound — refinement is obligation-only, coverage is type/index-possible only),
but a real erratum + a **conformance gap** (no refinement↔coverage case: AC5 is
index-only, AC7 is obligation-only). Sibling of conformance reconcile inherits
spec metatheory bugs lifted from absence- claims to **cross-section example/rule
consistency** — trace each illustrative example to a verdict under the rules
that govern it, the disconfirming question applied to examples. The disposition
(errata fix vs scope addition) gates on spec-author's intent ruling, which I
kept alive in the L2 retro so it doesn't evaporate with the merge. Also: the
reflexive both-reviewers-@mention fix held this ring (no dropped-mention
near-miss).

**7/7 — a NEW content-not-number sub-tell: WRONG-SECTION-IN-THE-RIGHT-FILE
(Sec2, 2026-06-30).** Cross-ref sweep held 7/7 on `62-authority.md` (~30 cites +
4 landed- code pins all EXACT, incl. `DeclassifyCap.is_valid` =
`flows_to(to,from) && to != from` = `to ⊑ from` ∧ strict). The single defect was
a **different shape** from the prior three: X2 = dangling slot (§ resolves,
content *absent/untouched*); B3 = wrong *filename* in my head (resolved by
§-content); here the `§9` level table cites `record / Σ-Form (13 §1)` but
**`13 §1` is *Dependent functions — Π*; Σ-Form is `13 §2`** — the §-number
resolves to a *real, populated* section **in the right file**, just the **wrong
one** (off-by-one-section, adjacent-content). A number-resolves check passes it;
a heading-only check passes it; **only opening the body and reading what §1
actually hosts (Π, not Σ) catches it.** This is the sharpest form of the
content-not-number rule — the cite is maximally plausible (right file, right
neighborhood, real section). Non-blocking (provenance pointer in the
level-reconcile table; the `Type (suc ℓ)` value + no-new-level-rule claim are
sound, Architect- confirmed), flagged into `wp/spec-errata` so it doesn't
evaporate. Also re-confirmed clean, author-agnostic: the `[Sec2-dual]`
order-dual pair (degenerate-at-meet, *not* erasure — taint axis orientation
needs distinguishing pair) and the kernel-backed- vs-trusted-by-typing split
(declassify bound over erased labels → trusted, *not* Q — trusted by typing
guarantee is not kernel proved Q) were both stated precisely by the spec; my job
was to re-derive and confirm, which held.

**8/8 — verify a wrong-section claim with ABSOLUTE file line numbers; I nearly
false-flagged the CORRECT spec body (L7 FFI, 2026-06-30).** Cross-ref sweep held
8/8 on `38-ffi-io.md §2–§6` + seed (`11 §4`, `18 §4/§5` `declare_postulate`/
`trusted_base`, `36 §1.4` escape, `21 §5.2/§5.4`, `71 §2.1/§3.1`, `25 §3`,
`41 §1` — all EXACT content), plus a clean landed-code reconcile
(`declare_postulate` `check.rs:1055`, `trusted_base` `env.rs:383`,
`EffectEscapes` `effects/check.rs:17`, `export.rs` — all on main; the new
`elabForeign` correctly ABSENT = future L7-build, not a false "already wired").
**The methodology trap (the carry):** chasing the `71` I1/I2 invariant location
I read line numbers from a piped `sed '/## 2/,/## 4/p'
| grep -n` as **file** lines — they're relative to the *extract*, so I first
concluded the **spec body** (`38 §3.1` cites "I2, `71 §3.1`") was a
wrong-section mis-cite. Recomputing with **absolute**
`git show <ref>:file | grep -nE 'I1 —|I2 —'` showed I1/I2 are at file l.203/207
= **§3.1** (the spec body was RIGHT), and the **seed** is the one that cites
them at `71 §2.1` (§2.1 hosts the projection *table*, §3.1 the invariants) — a
wrong-section-in-the-right-file slip (the 7/7 Sec2 shape), non-blocking
provenance, flagged for errata. **Rule: before asserting a wrong-section
mis-cite, re-verify the target's location with absolute file line numbers
(`git show :file | grep -n`), never relative-to-extract `sed|grep -n` numbers**
— a relative number read as absolute manufactures a false wrong-section finding
against correct prose. The discipline that catches the seed's slip is the same
one that almost mis-fired on the spec; absolute-line verification is what
separates them. Also clean + author-agnostic: re-derived both flagged
silence-resolutions (AC2 reliance-by-call via the real `trusted_base_delta`
dependency cone; AC3 `wrap`'s `result==c_sqrt x` = refl → Q while the `c_sqrt`
postulate → P by `21 §5.4`) and confirmed L7's `62` coupling is to the
kernel-backed **cap-gate**, untouched by the parallel Sec2-*build*
attenuation-bound rework.
