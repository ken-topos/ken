---
scope: enclave
audience: (see scope README)
source: private memory `spec-claim-kernel-admittance-vs-staging`
---

# Don't claim kernel admittance by trusting a sibling chapter's prose

When a `/spec` section asserts the **kernel admits** some construct, ground that
against the **kernel that exists now**, not against another spec chapter's
prose. Spec chapters describe the *eventual* kernel and can run **ahead of** the
implemented subset, or **diverge** from it. In L5 I declared `ITree`'s `Vis`
constructor "already in the pure kernel — admitted, `elim_ITree` generated,"
citing `14 §2`'s "Allowed: `(Nat → D) → D` (W-style)". But `Vis` is a **Π-bound
(W-style) recursive occurrence**, and the **current kernel rejects exactly that
shape** (`check_no_pi_bound_recursive`) — W-style admittance + its Π-abstracted-
IH eliminator are deferred to **K1.5**. `14 §2`'s "allowed" prose was itself
stale (a reconciliation pre-cleared but never merged), so a sibling chapter
*confirmed* my wrong claim. The Architect caught it as an L5 blocker: the design
is sound (positivity is genuine), but the *admittance* was an **unstated hard
dependency presented as already-satisfied**. Fix: declare the K1.5 dependency
(§2.1 + a §7.0 gate), split blocked-on-K1.5 (the ITree denotation half) from
buildable-now-K1-only (the row lattice / inference / escape check), and cite the
shape as "what K1.5 will admit."

**Two specifics that bit:** (1) **positivity ≠ admittance.** `14 §8.2`'s
strict-positivity algorithm *accepts* the W-style arg (it is strictly positive),
but the kernel has a *separate* admission gate (`check_no_pi_bound_recursive`)
restricting K1 to non-Π-bound recursion because eliminator generation for a
Π-abstracted IH isn't built yet. "Strictly positive" did not imply "the kernel
will take it." (2) **K1 is an explicit subset** (`14 §8.4`: non-mutual,
non-nested; W-style → K1.5) — so any spec claim that the kernel admits a shape
must check it against the *staged* kernel, not the full design.

**Why:** this is the spec-author flavor of COORDINATION §7 ("a spec claim about
the kernel must be checked against the kernel, not assumed"). The sharpening:
the false confirmer was *another spec chapter*. Grounding "X exists / is
admitted" by grep-ing the **spec** is not grounding it against the **kernel** —
and when the language is staged (K1/K1.5/K2/K2c), the gap between "the spec
allows it" and "the implemented kernel admits it" is a standing source of
unstated dependencies. Extends trust root test coverage discipline; sibling of
the over-equating traps in spec conv omega shortcut trap (there the guard
over-fired; here the admittance claim over-reached).

**Generalization — reconcile-don't-cite covers THREE claim-kinds, and the
erratum-honesty template must be applied UNIFORMLY across a document (X2
`44-capacity`, 2026-06-30, both CV+Architect-caught, non-blocking).** The
"verify the claim against what exists NOW in the target" rule is not just for
kernel- admittance / data-shape claims — it equally governs **(a) cross-file
*structural* references** (a taxonomy slot, a subsection number, a class in a
sibling file) and **(b) asserted *invariants*.** In X2 I ran a rigorous
reconcile on the store's **data shapes** (6 perishable-frame catches) and
flagged the **`41` cross-file claim** honestly ("raised as an erratum to `41` —
not changed on this branch"), yet **one section later** wrote that
`CapacityExhausted` **"is placed in the `43 §2` fault taxonomy"** citing
`43 §2.2` — a slot that **does not exist** (I'd **Read `43 §2` that same
session**: a flat 4-item list, no §2.2, no resource-fault class). I asserted a
**forward-ref to a not-yet-existing taxonomy entry and invented a subsection
number** — the exact thing the `41` flag avoided, with the correct template
**one section away and not applied**. Separately I asserted an accounting
**invariant** (`total_interns = total_slots + dedup_hits`) the landed code
**literally violates under exhaustion** (`store.rs:238` counts a refused call) —
I checked the happy regime, never **ran it against the refusal/boundary path**.
Two rules: **(1)** a forward-ref into another file's structure ⇒ verify against
that file's CURRENT body; slot absent ⇒ "*should be registered in* … (follow-on
edit)" + track it, never "is placed in" + a made-up §number. **(2)** an asserted
invariant must be **run against its adversarial/boundary path** before commit
(the invariant analog of discriminating conformance verdict must flip's
worked-example-flip). **The catch-all tell:** run a **uniformity sweep** over
every cross-file claim in the document before handoff — if §1 flags `41`
honestly and §2 hard-asserts `43`, the inconsistency *is* the bug. Cite-side
sibling: laundered citation authority (a fabricated cite gains false authority);
the fabricated `43 §2.2` is the same half-life, here on a *taxonomy placement*.

**Bidirectional — a staging-DEPENDENCY claim is as perishable as an admittance
claim, and the under-claim is NOT the safe direction (L6 `38-ffi-io`,
2026-06-30, clean ship, Architect-confirmed, zero flags).** L5 was the
*over-claim* (spec ran AHEAD of the kernel: claimed an admittance the kernel
lacked → phantom capability). L6 is the *under-claim*: `36`'s prose says "L5's
implementation is **gated on K1.5**," and the reflex is to forward that block
into L6's I/O denotation. But **K1.5 had shipped** —
`check_no_pi_bound_recursive` is *retired* (`inductive.rs:21`/`check.rs:881`;
`14 §8.4` is literally titled "W-style ... admitted in K1.5"). Forwarding the
stale block would have handed the build team a **phantom dependency** (wait on a
gate that no longer exists) — the temporal dual of K2c-s2's "rebuild removed
unsoundness." So **"X is blocked on stage S" must be verified against S's
CURRENT landed state before forwarding**, exactly like "the kernel admits X."
Staleness has a **direction-agnostic half-life**: a prose claim can run ahead of
the kernel (phantom capability) OR behind it (phantom block), and **neither** is
the safe direction — both misdirect the build team. The
reconcile-against-landed-code pass catches both; the fix is the same (cite the
landed state, raise the stale prose as a tracked doc-erratum — here the `36`
"gated on K1.5" line, not changed on the L6 branch). **And the held that paid
off:** the X2 uniformity-sweep carry (open every cross-file cite against its
current body) ran pre-handoff on L6 and the independent Spec reviewer confirmed
**all 8 cites resolve to real content** — the X2 dangling-`43 §2.2` failure mode
did **not** recur. Last WP's trap → this WP's held, one ring later.

**How to apply:** for every spec sentence of the form "the kernel admits /
checks / generates X," verify against the *current* kernel — grep the kernel
impl's admission checks (`check_*`) and the chapter's own "what K1/K2 delivers /
defers" scope notes (e.g. `14 §6`, `14 §8.4`), not just the permissive examples.
If the construct needs a not-yet-landed kernel feature, **declare the dependency
explicitly** (which stage gates it) and **partition the deliverable** into the
part buildable on today's kernel and the part that waits. Treat a sibling
chapter's "allowed" as a claim to re-verify, not a citation to lean on —
especially when a reconciliation may be pending/unmerged.
