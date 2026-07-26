---
scope: fleet
audience: (see scope README) — anyone who writes or accepts a "mutate the
  authority and the output must change" control: QA, implementers, frame authors
  writing acceptance criteria, and reviewers confirming an axis is closed
source: RT-FNSPLIT-B2V, 2026-07-26. Measured by runtime-implementer against its
  OWN confirmed axis; it self-indicted an axis the Architect had already
  CONFIRMED, and the Architect withdrew the confirmation. Ruled per-site in
  `evt_51xk9sxqdtzgt` (frame `RULING R4`).
---

# A differential over an aggregate is an EXISTENTIAL, not the universal it reads as

⛔ **"Perturb the authority, observe the whole artifact change" proves *someone*
consumed it. It does not prove *every* consumer did.** The control reads as
*"the emitter consumes the plan"* and measures *"at least one site consumes the
plan"* — and with N sites, N−1 of them can be disconnected while it stays green.

## MEASURED / CLAIMED / THE GAP

`class_guard(&mut b, node, plan.int_magnitude_classes())` appeared at **five**
sites. The implementer replaced **one** of them with the literal
`&[BoundaryClass::Int]` it used to be, left the other four consuming the plan, and
ran the suite:

```
test result: ok. 439 passed; 0 failed
```

- **MEASURED:** the whole-graph differential
  `recut2_the_emitted_helper_graph_changes_when_the_authority_changes` is green
  with a site disconnected from the authority.
- **CLAIMED (by the pin's name and by everyone reading it):** the emitted helper
  graph changes when the authority changes ⇒ the emitter consumes the authority.
- **THE GAP:** the four remaining consumers still move the aggregate, so the
  required difference is still there. **The pin cannot perceive a per-site
  defection at all** — not weakly, *not at all*, for any of the five sites.

⇒ The governing ruling: **causal coverage is per-site.** Every consuming site
needs its own behavioural differential, **or must be NAMED as probe-unreachable.**
Four consumers moving an aggregate cannot prove the fifth consumes anything.

## How to apply

- **Count the consuming sites, then ask what a control at N−1 of them looks
  like.** If the answer is "the same green", the control is an existential. This
  is one command's worth of work and it is the whole check.
- **Where the probe cannot reach a site, say so IN THE TEST.** A source scan over
  *all* sites pinning the allowed form (every argument comes from `plan`) is a
  legitimate second mechanism — with **undetermined-parse ⇒ FAIL** and a positive
  control on the site count. Two mechanisms, because neither covers the surface
  alone.
- ⛔ **Name the residual that survives both.** A helper laundering a literal into
  plan-shaped text is detectable by neither the behavioural pin nor the scan. An
  unnamed residual reads as coverage.

## ⛔ AND A CONFIRMED DELIVERABLE IS NOT A REASON TO SKIP THE MUTATION

★ The Architect had **confirmed** this axis. The confirmation was **accurate about
the code it read** — the wiring was real, the literals were gone, the derivation
was the partition's. What a code review **structurally cannot report** is whether
the pin standing behind it is as strong as both parties are treating it as. ⇒ **A
review confirms the code; only a mutation confirms the evidence.** The
confirmation was withdrawn on the implementer's own measurement.

⭐ **The seat best placed to find this was the one that had already proved the same
shape elsewhere** — it had just fixed the identical hole on the *tag* axis, said
the same argument *probably* indicted the confirmed class axis, and then **did not
leave it at probably**. One command turned a suspicion into a withdrawal. ⇒ When
you fix a defect on one axis, **run the same disconnection on every sibling axis
you or anyone else has called closed.**

## Sibling shapes — same family, and this is the general form

The corpus already had two narrower cases, both about *bundles*:

- [[bundled-changes-need-per-mechanism-isolation-flip]] — two mechanisms in one
  diff; a bundle-level flip shows "5 tests newly pass" and is silent on which
  change caused which pass.
- [[discriminating-flip-must-be-checked-per-test]] — a suite where two tests
  genuinely discriminate and two pass on **both** sides of the flip.

⇒ Those say *split the bundle* and *check test-by-test*. **This one is the same
defect with no bundle at all:** ONE mechanism, ONE pin, N call sites, and the
aggregation happens *inside the artifact under test* rather than across a diff or
a suite. So "isolate the changes" does not reach it — you have to isolate the
**consumption sites**.

Adjacent, and worth reading together:

- [[an-enumeration-needs-a-proven-closure-not-a-better-grep]] — you cannot
  per-site anything until you know the site *count* is closed. Here the located
  list initially missed two sites.
- [[a-probe-truncated-before-the-grep-is-not-a-measurement]] — the other way a
  green number means *could not see* rather than *none*.
