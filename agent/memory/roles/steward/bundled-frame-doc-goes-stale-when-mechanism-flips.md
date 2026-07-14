---
scope: roles/steward
audience: (see scope README)
source: private memory `bundled-frame-doc-goes-stale-when-mechanism-flips`
---

# A bundled frame doc goes stale when a WP's mechanism flips mid-build

Steward gate-check, learned live on the Map `obs-eq-termination` fix
(2026-07-03). The WP's **mechanism flipped twice under empirical pressure**
mid-build: framed as a **funext** obs-reducer divergence (occurs-guard/
fail-closed envelope) → refuted by direct instrumentation (zero hits) → re-read
as a **δ-unfold** non-termination → finally grounded as an **eager-δ in
`conv_struct`** whose recurring goal is genuinely **TRUE**, fixed by a
**congruence-first / lazy-δ** fast path. The code the kernel team built matched
the *final* mechanism. **But the WP frame doc I authored
(`docs/program/wp/ obs-eq-termination.md`) still stated the original funext
scope + the ruled-out envelope — and the kernel-implementer had bundled that
frame doc INTO the merge candidate** (`3a5e2ab`: `conv.rs` +40, 2 test files,
**+ frame doc +171**). Everyone at the gate was watching the *code* diff; the
stale *doc* rode alongside it, about to land on `main` asserting a mechanism the
code contradicts.

**Why it matters:** a WP frame on `main` whose stated "what's broken / how we
fix it" the merged code directly refutes is an **actively misleading artifact —
a stale "what's-broken" is worse than none** (it misdirects the next reader/
build). Frame docs are Steward-owned; their correctness on `main` is my lane.

**How to apply:**
- **At any merge where the WP's mechanism/scope moved during the build, diff the
  bundled frame doc against the AS-BUILT code, not the original hypothesis.**
  `git show <candidate-sha>:docs/program/wp/<id>.md | grep -i <refuted-scope>` —
  if the refuted mechanism is still stated as *current* scope, it's stale. The
  doc is easy to forget precisely because attention is on the code diff.
- **Fix it WITHOUT holding the green code** (merge-on-green). Two clean paths,
  in preference order: (1) if the branch owner is still committing in the gate
  loop, hand them the corrected doc to **fold in atomically** (doc-only → a new
  SHA whose code/tests are byte-identical → **orthogonal to the soundness
  gate**, no re-cert needed; `main` honest day-one) — this is what happened
  (`3a5e2ab → 4c6824a`, Architect's CERTIFY carried verbatim). (2) else let it
  merge on green and land the corrected frame as a **Steward erratum on `main`**
  immediately after (trust level prose vs locked adr crosscheck erratum
  pattern). NEVER hold sound+green code for a doc-prose correction.
- **The deeper fix is upstream (already in the playbook):** frame by **objective
  + acceptance**, treat the mechanism/"what's-broken" as **perishable /
  verify-against-landed-code**, don't bake a mid-build hypothesis in as fixed
  scope. I violated that by baking Architect's *earlier armchair* funext call
  into the frame as settled scope — the empirical instrumentation refuted it.
  The frame's *acceptance* (deliverable-4: law 4 builds clean) held perfectly
  and was the load-bearing gate; only the *mechanism prose* went stale. So:
  acceptance is durable, mechanism is perishable — and the perishable half must
  be re-synced at the gate.

**Second trigger — a Steward frame amendment made AFTER the ring branched does
not propagate to their candidate** (CC2, 2026-07-13). The Architect ruled a
mid-build scope refinement (`show_int : Int → String` un-buildable on opaque
`Int` → defer as a named substrate gap); I amended the frame doc accordingly.
But the receiving ring had already branched, and **its worktree had the WP
branch checked out**, so `git branch -f wp/<id> <new>` fails
(`cannot force update the branch … used by worktree`) — the amendment landed on
**`steward/work` only**, and the implementer's released candidate carried the
**pre-amendment** frame doc. The **code was correct** (it followed the ruling
relayed in-thread, not the stale doc); only the frame *doc* text lagged. **QA
caught it** ("authoritative frame still has superseded wording"), a review round
I could have saved. How to apply, in addition to the merge-gate diff above:
- **When you amend a frame after handoff, do NOT assume the WP-branch ref
  moved.** If the ring holds it checked out, the ref is locked and your amend is
  stranded on `steward/work`. **Immediately hand the ring the amended frame to
  fold into its next candidate** — reachable across worktrees via the shared
  `.git`: `git show steward/work:docs/program/wp/<id>.md > docs/program/wp/<id>.md`
  (no fetch — linked worktrees share refs). Verify `git diff` shows only the
  frame doc.
- Better still, **fold a known-pending ruling into the frame BEFORE the ring
  branches**; a post-branch amendment always risks this propagation gap.

Sibling of correcting scope must sweep whole doc (sweep the WHOLE doc, not one
section) and trust level prose vs locked adr crosscheck (shipped prose gains
false authority on `main`). The common core across both triggers: the stale
artifact is a **frame doc bundled in the code candidate** and attention is on the
code diff, so the doc rides stale — whether it went stale by a **mechanism-flip
mid-build** (caught at the merge gate) or by a **post-branch Steward amendment
that couldn't propagate** (caught at QA). Re-sync the frame from `steward/work`
via the shared `.git` in both cases.
