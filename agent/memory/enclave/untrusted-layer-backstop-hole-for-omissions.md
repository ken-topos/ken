---
scope: enclave
audience: (see scope README)
source: private memory `untrusted-layer-backstop-hole-for-omissions`
---

# An untrusted layer's bug is safe only for what it supplies, not what it omits

The reflexive framing for an **untrusted producer the kernel re-checks** —
"everything it emits is re-checked, so a bug here is a wrong verdict / poor
diagnostic, **never** unsoundness" — has a **hole**: the kernel backstops what
the layer **supplies** (certificates it re-checks), **not** what the layer
**fails to produce**. A *silent omission* is invisible to the backstop.

**V2 instance (2026-06-30, Architect-caught on my conformance preamble).** I
wrote that a V2 (obligation-extractor) bug is "a missed obligation … a wrong
verdict **caught downstream** … **never** unsoundness." Wrong. A **truly
missed** obligation — one the extractor **never emits** — supplies **no**
certificate, so the V1 §5.4 honesty guard (`trusted_base()`, which catches
**generated-but- undischarged** holes) **never sees it**. The property reads
**`proved`** though unproven. So it is **not** caught downstream, and it **is**
a *verification*- soundness gap. "Never unsoundness" is true only for **kernel**
soundness (the kernel re-checks every *supplied* cert; a missed obligation
supplies none). The precise framing: *a bug never breaks **kernel** soundness,
but **completeness-of- extraction** is the **verification**-soundness linchpin,
backstopped by **nothing but** the absent-clause scan.*

**Sec1 generalization (2026-06-30, security domain, 2 new instances) — enumerate
every TRUSTED LAYER between the property and the kernel re-check; conformance is
the SOLE net for each.** The omission hole is one face of a broader rule: the
kernel backstops only what it **sees**, so any layer whose output the kernel
does **not** re-check (or re-checks *faithlessly*) is trusted, and
discriminating conformance is its only net. Sec1's honesty chapter (Architect
N1/N2, folded into `61` `a5c82ea`) named two such layers I'd otherwise have
under-pinned:
- **(N1) erased indices.** IFC labels are **erased** before the kernel
  (`61 §3`), so a flow-typing bug (wrong `⊑` in `L-SINK`, a dropped `pc`-join, a
  label-dropping `bind`/`incl`) emits a **well-typed core term the kernel
  accepts** while non-interference is violated → the by-typing flow rules are
  *trusted*, and the discriminating flip cases (`label-survives-effect-routing`
  C1 et al.) **ARE** the trust boundary, not a backstop *to* the kernel.
- **(N2) an untrusted reduction.** The by-proof path reduces 2-safety to a unary
  obligation via a product-program construction done by the **untrusted**
  verifier; the kernel re-checks the cert **for the handed obligation, NOT its
  faithfulness to 2-safety** (`cert-recheck ≠ reduction-faithfulness`), so a
  too-weak `Φ_post` / dropped `coterminates_ζ` yields a **kernel-valid cert for
  a non-NI claim** — a **false `proved`** the forged-cert reject (a
  *non-typechecking* cert) cannot cover. The sole net is a **positive-soundness
  case**: a *known-interfering* program must reduce to `disproved` (D5) — the
  reduction can't be massaged to make a leak look `proved`.

**Authoring rule (forward-looking).** Before writing "kernel-re-checked ⇒ safe",
**list what the kernel does NOT see** — erased annotations, the faithfulness of
an untrusted reduction, an un-emitted obligation — each is a trusted layer
needing a discriminating (value-flip), **positive-soundness** (known-bad →
caught), or **exhaustiveness** (no silent omission) conformance case. That list
IS the load-bearing part of the seed. (Both Sec1 instances spec-author-credited
to this discipline; the design-time lock-points LP-1/2 pre-pinned the boundary,
so the seed shipped clean even as N1/N2 sharpened the spec under it.)

**Why it matters (not pedantic).** "All obligations discharged ⇒ correct" — the
entire value of a VC extractor — is only as strong as the guarantee that **no
burden was silently skipped**. The loose framing ("kernel backstops everything")
invites *under-investing* in the one safeguard that actually holds the property
(the absent-clause scan / completeness audit). Calling it "the load-bearing
safeguard, not a nicety" is the correction.

**The structural safeguard against omission: exhaustiveness by construction.**
Enumerating the *known* no-emit positions with guards is necessary but not
sufficient — the gap is a construct with **no rule at all** (where a *future*
burden-bearing form gets silently no-emitted). State normatively + pin in
conformance: the traversal is **exhaustive by construction** — every form is an
emit site, an explicit context-extension, or an **explicitly-guarded** no-emit;
anything unmatched is **emit-or-error, never a silent recurse-past** (no
catch-all `_ ⇒ skip`). This is a **structural/absence** conformance assertion
(no value flip — assert the *shape* of the producer's traversal), the dual of
discriminating conformance verdict must flip's X1 complement (move the assertion
to the structural layer where the bug is visible).

**How to apply (the whole verification spine V2/V3/V4, and any "untrusted
producer the kernel re-checks").** When you write the "untrusted,
kernel-backstopped" framing, split it: (1) what the layer **supplies** is
re-checked → genuinely no *kernel*-soundness risk; (2) what the layer can **omit
/ fail to produce** is **not** backstopped → the completeness/exhaustiveness
audit is the *sole* safeguard, and a missed-production must be a **visible**
failure (exhaustive traversal, no silent skip), not a downstream catch.
Distinguish *re-checked artifacts* from *silent omissions* every time. Extends
trust root test coverage discipline (a green corpus is evidence only if it pins
the property) and the absent-clause/absence-assertion gate of conformance
reconcile inherits spec metatheory bugs to the producer's *coverage*, not just
each case's.
