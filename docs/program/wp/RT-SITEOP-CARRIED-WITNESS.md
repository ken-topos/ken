# RT-SITEOP-CARRIED-WITNESS — a witness for a carried site-bound operand

**A synthesized `SiteOperand` demands a compile-time `Lowered` template from
the same effect seat that byte-span activation wants carried. The two demands
are in direct structural conflict, so 29 quarantined rows and four seats cannot
be discharged by any byte-span mechanism. This node resolves the conflict.**

**Owner:** Team Runtime. **Branch:** `wp/RT-SITEOP-CARRIED-WITNESS`.
**Size:** L — provisional, and it is **not** a sizing you should trust yet; see
§3, the mechanism is unruled and the mechanism sets the size.
**Risk:** medium-high — the touched typing exists to make an unsound
substitution unstateable.

**Status `draft`. The gap is measured; the MECHANISM is an open Architect
fork.** This frame states the gap and the fork. It does **not** pick the
answer, and a reader must not treat §3's options as a recommendation.

---

## 1. Fixed inputs

Measured at `origin/main = 11bc4c4a`, plus the held `RT-CARRIER-BYTESPAN-OBSERVE`
`D5` candidate `4244d082915bbd6fe154a5e727c6a23c879f1f37`.

| anchor | what it is |
|---|---|
| `lowering/mod.rs:11354-11362` | `site_operand_argument` — the sole template projection |
| `lowering/mod.rs:11640-11667` | the `SiteOperand` reconciliation arm and its refusal |
| `lowering/mod.rs:11316-11333` | `site_operand_witness` — what a witness may be |
| `lowering/mod.rs:11300-11304` | why `None` is a refusal rather than a fallback |

**Re-pin at pickup.** `D5` lands in this region before this node starts, and
these anchors will move. They are recorded so the *derivation* below can be
checked against what changed — not so the line numbers can be trusted.

## 2. The conflict, exactly

`site_operand_argument` calls
`seats.specialized(EffectSeatSlot::Argument(index))?`. That is a hard demand
for the compile-time template. A seat activated to `EITHER_PHASE` may deliver
its value as a boundary word instead, and then `specialized` errors — which is
the refusal, propagated unchanged.

The reconciliation arm is deliberate about this, and its own comment is the
clearest statement of the design intent this node must not break:

> A declared `SiteOperand` whose claimed operand is CARRIED refuses at that
> exact seat, propagated from `specialized`. It does not reconstruct a
> template, widen the carrier, borrow a sibling, or fall back — reconciliation
> needs a compile-time witness, and there is none.

⇒ **The refusal is not a bug and this node must not "fix" it by weakening it.**
The comment at `:11300-11304` says a permissive fallback *"would reopen the
substitution for exactly the variants nobody thought about."* Any candidate
that makes the refusal quieter, rather than making the witness available, is
the wrong shape and should be rejected on sight.

## 3. THE OPEN FORK — the Architect rules this before the node goes `ready`

**The question:** how does a site-bound operand obtain a witness when its seat's
value is carried?

**This frame does not choose.** Sketches of the shape the answer might take are
recorded only so the fork is legible; **none is endorsed, and the list is not
asserted to be exhaustive:**

- give the reconciliation a witness form that a carried span can satisfy;
- give the site path a provenance independent of the seat, so the synthesized
  node never reads a seat that activation may carry;
- constrain the two readers so a seat consumed as `SiteOperand` is structurally
  ineligible for activation, making the conflict unstateable rather than
  refused.

**Each has a materially different blast radius on the soundness argument**, and
picking between them is a component-design call — `COORDINATION §9`, the
`any → Architect` edge. **Route it as "which mechanism?", never as "may I do
X?"**, which presumes the answer.

**Until the Architect rules, this node is `draft` and nobody builds against
it.** A ring that starts here without the ruling will produce a candidate whose
mechanism is unreviewed at exactly the layer that most needs review.

## 4. Deliverables

**Provisional. `D2` onward depend on §3's ruling and will be re-cut with it.**

- **`D1` — carry the measurement in, do not re-derive it.**
  [[RT-CARRIER-BYTESPAN-OBSERVE]]'s `D5` established the blocker on two
  independent routes, and its candidate is
  reproducible green evidence. Confirm it still holds at your base and report
  what moved; **do not spend a turn re-establishing a settled fact.**
- **`D1a` — the exact population.** Name every one of the 29 rows and the four
  seats, and confirm each one's *measured* cause is this blocker and not
  something that has since diverged. **A row whose cause changed is a finding
  and comes back to the Steward**, not something to absorb.
- **`D2` onward — the mechanism.** Cut against §3's ruling.

## 5. Acceptance criteria

**Provisional except `AC-1` and `AC-2`, which hold whatever the mechanism.**

- **`AC-1` — the refusal still refuses.** The substitution the `SiteOperand`
  typing exists to prevent is still unstateable, demonstrated by a control that
  is **seen to fail** before it passes. **A candidate that only makes the error
  go away has removed a soundness net, not supplied a witness** — this AC is the
  one that discriminates those two outcomes, so it is not optional and it is not
  discharged by a green suite.
- **`AC-2` — the residue is attributed, per row.** Every row this node
  un-skips is green and named; every row it does **not** un-skip carries its
  measured cause. Report **ignored separately from passed**, per file — a bare
  `passed / failed` pair reads green while nothing has been un-skipped.
- **`AC-3` (no-regression).** Workspace green **in CI** — never a local
  `--workspace` run (`COORDINATION §12`).
- Further ACs land with §3's ruling.

## 6. Inherited: the `D6` activation-gate discharge pass

**Moved here from [[RT-CARRIER-BYTESPAN-OBSERVE]] `D6`**, because its premise is
"the activation" and this node is where the activation completes. Its
specification moves verbatim; read it there.

**What is already measured and must NOT be re-derived:** the family-2a sentinel
asserts zero applications and the `ken-runtime` lib suite is green at the `D5`
candidate, so the partial activation did not make the carried source-Match route
executable — **the dormancy premise is intact.** Start from that.

The split-phase rig remains the named producer for the outcome-1 propagation
witness.

## 7. Inherited obligation — a seat activated with no end-to-end row

**`(FsWriteFile, Argument(2))` was activated by `D5` on per-seat evidence:
measured reach and measured observation, with no committed row exercising it
end-to-end**, because its sibling path seat blocks every program that reaches
it. That satisfied `RT-CARRIER-BYTESPAN-OBSERVE.AC-4` as written, which asks for
per-seat evidence.

⇒ **This node is the first that can exercise that seat end-to-end, and the
hazard is one of attribution.** If the activation is subtly wrong, the failure
surfaces inside *this* node's candidate and reads as this node's regression.

**It is not.** A failure at `(FsWriteFile, Argument(2))` traceable to the
activation itself belongs to `RT-CARRIER-BYTESPAN-OBSERVE` — report it as such
and return it to the Steward rather than absorbing it. **Reverting the
activation is a one-line change plus its pin**, and that remains available.

## 8. Banned scope

- **Weakening the `SiteOperand` refusal or `site_operand_witness`'s `None`
  arm** to make the error disappear. See §2 — that is the failure mode, not the
  fix.
- **Building against §3 before the Architect has ruled it.**
- **Absorbing a row whose measured cause turns out not to be this blocker.**
  That is a finding and a Steward recut.
- **Re-deriving the `D5` measurement** instead of carrying it in.

## 9. Hard stop

Stop and return the seam if the ruled mechanism turns out to require changing
what a `Lowered` witness *is* for readers other than `SiteOperand`, or if the
29 rows split across more than one cause after `D1a`.
