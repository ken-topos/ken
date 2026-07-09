# sct-reconstruction-descent — accept lexicographic / Ackermann-style nested recursion (sct-completeness follow-on (b))

**Steward frame → Team Kernel (build). KERNEL / TCB LANE — this WP MODIFIES the
trusted root** (`crates/ken-kernel/src/sct.rs`). It is the **(b)** half of the
`sct-completeness` decomposition ((a) VAL2 #12 landed `b34d4aa`). Its whole risk
is soundness, so **Architect's approach-review is the central, mandatory
up-front gate — not a light touch** (same posture as (a)). Goal: fix the
**completeness** of the size-change termination (SCT) checker so it *accepts*
genuinely-terminating **lexicographic / Ackermann-style** nested recursion it
currently *over-rejects*, **while preserving the termination guarantee** (still
reject **every** non-terminating program). Owner: **Kernel**. Gate: **Architect
approach-review — DONE, APPROVE** (`evt_2r390qa0jbknf`, grounded against post-(a)
`sct.rs @ b34d4aa`) → **Kernel builds** → **Architect soundness** on the candidate
(preserves termination) + **Kernel QA** + **CI**; **spec-leader confirms** whether
the SCT completeness-class needs a `spec/10-kernel` note. Findings → **Steward**.

Base: `origin/main` (post-(a), `b34d4aa`). Branch (pre-staged by Steward):
**`wp/sct-reconstruction-descent`**.

## ★ RATIFIED DECOMPOSITION (Architect approach-review `evt_2r390qa0jbknf`) — build to THIS

**VERDICT: sound to build. The design is right, the boundary (`DownEq`-only, exact
reconstruction) is correct, the criterion stays sacred.** Grounded against landed
`sct.rs @ b34d4aa`. Build to this section; the mechanism prose below it is the
(now-confirmed) hypothesis it ratifies.

**Mechanism — confirmed against landed code.** `size_rel` (`sct.rs:116`) returns
its provenance ordering **only for a bare `Term::Var`**; the trailing arm (`:124`)
sends `App(Constructor, Var)` — the reconstruction `Suc m2` — to `Unknown`. That is
the exact over-rejection. `dispatch_elim_methods` (`:272`) **already** threads
`field_prov = (pi, Down)` onto matched fields (the (a) infra). So the change is
narrow: **a new reconstruction case in `size_rel`, fed by a per-branch
reconstruction record threaded alongside `prov`.** The engine
(`composition_closure_self_loops`), `compose_ord`, `has_strict_diagonal`, and
`ScMatrix` are **untouched**.

**THE load-bearing soundness surface = the detection predicate
`is_exact_reconstruction` (this is where a Bottom hole would live).** It must fire
**iff** `arg` peels to `App*(Constructor{id}, args)` with **`id == the current
branch's matched ctor`**, **`args.len() == n_fields`**, and **each `args[j]` is
exactly `Var(the raw field binder j)`** — same ctor, positional, raw matched
fields, **no added structure, no reorder, no subst-wrap**. Anything else ⇒
`Unknown`. **Over-firing this predicate is the sole unsafe vector.**

**The soundness argument (the whole gate — checkable):**
1. **Engine already sound given TRUE edges** (Ben-Amram/Lee–Jones): idempotent
   self-loop + strict diagonal ⇒ no infinite descent. So the entire obligation
   reduces to: **is every emitted reconstruction edge TRUE?**
2. **Exact reconstruction is a TRUE `≤` edge.** In the `C_k` branch the scrutinee
   is definitionally `C_k(x̄)` (ι-rule) and `s ≤ pi` (descent, sound from (a)) ⇒
   `(pi, DownEq)` is true. It is **`DownEq` not `Down`** because `s` may *equal*
   `pi`; emitting `Down` over-claims strictness ⇒ `badAck` accepted ⇒ Bottom.
3. **The clinching invariant — the fix CANNOT accept a non-terminator.** The edge
   only ever emits `DownEq`; `compose_ord` yields `Down` **only from a `Down`
   input** (`:34`). So a self-loop with **no genuine field-descent `Down`** stays
   all-`DownEq`/`Unknown` under composition — it can **never** gain a strict
   diagonal. Reconstruction edges are **thread-PRESERVING** (`compose(Down,
   DownEq)=Down` — propagate a real `Down` across a same-size step), **never
   thread-CREATING**. `badAck` is all-`DownEq` ⇒ stays all-`DownEq` ⇒ no strict
   diagonal ⇒ **still rejected**.

**The accept/reject discriminator (precise):** does some **OTHER** param pass a
genuine FIELD (`Down`) while param 0 stays equal via reconstruction (`DownEq`)?
Ackermann inner `ack (Suc m) n` = `[p0: DownEq (recon), p1: Down (n is the field
of Suc n)]` ⇒ strict on p1 ⇒ **accept**. `badAck (Suc m2) n` = `[p0: DownEq, p1:
DownEq (n is the raw param, not a field)]` ⇒ no strict ⇒ **reject**. The `DownEq`
edge is **necessary but never sufficient**; a real independent `Down` must exist.

**Reuse (a)'s producers — don't re-derive.** The reconstruction record's
ctor/field identification must be consistent with how (a)'s
`dispatch_elim_methods`/`is_recursive_field` assign field provenance; a divergent
local re-derivation risks mis-firing the predicate (the (a) grep-the-producer
lesson — [[sct-completeness-fix-conservative-construction]]).

## The bug class — completeness, not soundness (why this is safe to attempt)

Identical posture to (a): SCT is **fail-CLOSED**; it over-rejects some **valid,
terminating** lexicographic-descent programs with
`KernelRejected(NotTerminating …)`. The safe direction. This WP moves the
accepting set **outward** toward the true terminating set — the load-bearing
obligation is that it **stays within** it (never admits a divergent program).
See [[kernel-rejects-is-completeness-fix-is-where-soundness-converts]]: the
rejection is positive evidence the checker is live; the soundness risk lives
entirely in the **fix**.

## The over-rejection shape — GROUND against landed `sct.rs`, this is PERISHABLE

**⚠ The mechanism below is a *hypothesis* to accelerate the build — verify the
exact current rejection against the landed `crates/ken-kernel/src/sct.rs`
(post-(a) `b34d4aa`) with `SCT_DEBUG` edge/matrix tracing before coding. Do not
trust this frame's mechanism over the code** (the (a) retro: hand-deriving de
Bruijn provenance through nested binders is error-prone even when the design is
right — dump the actual `edges`/`self_loops`).

**The composition-closure engine is already correct.**
`composition_closure_self_loops` (`sct.rs`) implements the true size-change
principle (Ben-Amram / Lee–Jones): it keeps each distinct matrix triple
separately, closes under composition, and accepts iff **every** idempotent
self-loop has a strict diagonal. That engine **already handles lexicographic
descent in principle** — so **the gap is in matrix *construction*, not the
acceptance engine** (exactly as (a) was).

**The suspected construction gap — reconstruction loses `DownEq`.** Ackermann:
```
ack (Suc m) (Suc n) = ack m (ack (Suc m) n)
```
The inner call `ack (Suc m) n` passes **`Suc m`** back as param 0. Since the
caller matched param 0 **as** `Suc m`, that reconstructed `Suc m` is the **same
value / same size** as param 0 — size relation **`DownEq`**. But `size_rel`
(`sct.rs`) returns its provenance ordering **only for a bare `Term::Var`**;
`Suc m` is `App(Constructor, Var)`, so it falls to `SizeOrd::Unknown`. Losing
that `DownEq` on param 0 is what breaks the lexicographic thread: the composed
loop *inner-then-outer* (`ack (Suc m) n` ∘ `ack m …`) becomes an idempotent
all-`?` matrix with **no strict diagonal** → **false rejection**. With param 0
recognized as `DownEq`, that composition is `compose(DownEq, Down) = Down` on the
diagonal — the lexicographic argument (param 0 preserved while param 1 strictly
decreases, composing against the branch where param 0 strictly decreases) threads
correctly and the closure is all-strict-diagonal.

## The fix is to the ANALYSIS (matrix construction), never to the acceptance criterion

The accepting rule — *"every idempotent self-loop matrix has ≥1 strict `↓` on
its diagonal"* (`has_strict_diagonal`) — is **correct and stays verbatim**, as
does the composition-closure engine. The gap is upstream: the matrix
**construction** (`size_rel` / provenance) misses the `DownEq` edge for an exact
same-size **reconstruction** of a destructured parameter. **Add that edge; do NOT
weaken the acceptance test or the closure.** Any change that makes acceptance
*easier* by short-circuiting the criterion (a new early-accept path, an
`edges.is_empty() ⇒ accept`, dropping the strict-diagonal requirement) is the
[[sct-unapplied-self-reference-over-accepts]] anti-pattern — it would inhabit
Bottom. **The criterion is sacred; only the matrix feeding it becomes more
complete.**

## The soundness boundary — the load-bearing part (`DownEq` only, NEVER `Down`, only for EXACT reconstruction)

A reconstruction is **never strictly smaller** than what it reconstructs — so the
new edge is **`DownEq` (=), never `Down` (<)**. And it is `DownEq` **only for an
*exact same-size* reconstruction**: the constructor and its fields must be
**exactly** the shape destructured from that parameter (positionally-raw matched
fields, same constructor, no added structure, no reordering, no substitution
wrapping). **Any structural addition makes it strictly larger → must stay
`Unknown`.** Two concrete divergent near-misses fix the boundary — both **must
still be REJECTED**:

- **`badAck m n = match m { Zero => n ; Suc m2 => badAck (Suc m2) n }`** — passes
  the *exact* reconstruction `Suc m2` (`DownEq`) and `n` (`DownEq`): the self-loop
  is all-`DownEq`, **no strict diagonal → correctly rejected.** (It diverges: param
  0 never shrinks.) This proves the `DownEq` edge alone does **not** over-accept —
  it must **not** be `Down`. *If the fix emits `Down` for a reconstruction,
  `badAck` is wrongly accepted → Bottom.*
- **`badAck2 m n = match m { Zero => n ; Suc m2 => badAck2 (Suc (Suc m2)) n }`** —
  passes a **size-INCREASING** reconstruction `Suc (Suc m2)` (strictly larger than
  param 0 = `Suc m2`). This **must stay `Unknown` → rejected.** *If the fix emits
  `DownEq` for any constructor-wrapping of a destructured field (rather than only
  the exact same-size shape), `badAck2` is wrongly accepted → Bottom.* This is the
  precise over-scope vector: `DownEq` fires **iff** the reconstruction exactly
  cancels the destructuring depth, never for a net structural increase.

## Acceptance criteria

- **AC1 — SOUNDNESS PRESERVED (load-bearing, the whole gate). EXPANDED
  adversarial net (Architect `evt_2r390qa0jbknf` — `badAck`/`badAck2` pin only
  DEPTH; add the predicate-exactness near-misses).** A curated **divergent /
  must-stay-`Unknown` set STILL trips `KernelRejected(NotTerminating)`**, each case
  isolating one clause of `is_exact_reconstruction`:
  - `c := c`, `loop := id loop`, recursion-through-`map` — classic non-terminators
    ([[sct-unapplied-self-reference-over-accepts]]), unchanged reject.
  - **`badAck`** — exact reconstruction, all-`DownEq`, no strict diagonal (proves
    `DownEq` ≠ `Down`; a `Down` edge would accept it ⇒ Bottom).
  - **`badAck2`** `Suc (Suc m2)` — depth-INCREASE; `args[0]` is `App(Ctor,Var)` not
    a raw `Var` ⇒ `Unknown` (proves exact-depth-only).
  - **REORDER** — `C(x1, x0)` for a 2-field ctor: positional mismatch ⇒ `Unknown`
    (a `DownEq` here is unsound — the swapped reconstruction is **not** ≡ the
    scrutinee).
  - **WRONG-FIELD** — `C(y)` where `y` is any non-matched-field var ⇒ `Unknown`.
  - **WRONG-CTOR** — `D(x)` where `D` ≠ the branch's ctor ⇒ `Unknown`.
  These are the **discriminating negatives**: they share the reconstruction *shape*
  of the now-accepted Ackermann but diverge / are not size-preserving, so a fix
  that accepts them is the soundness hole. **The acceptance criterion and the
  closure engine are unchanged** — grep the diff: no new early-accept/short-circuit
  path, `has_strict_diagonal`'s strict-`↓` and `composition_closure_self_loops`
  verbatim; the only additions are the `size_rel` reconstruction case + the
  per-branch reconstruction-record threading.
- **AC2 — Completeness (Ackermann).** A terminating Ackermann (ground the exact
  shape from a real `.ken` repro) **passes SCT** and **evaluates to the correct
  value** through the interpreter (accept *and* run, not just accept). Include at
  least one further lexicographic-descent shape that is **not** Ackermann-identical
  (e.g. a two-argument descent where the outer stays equal via reconstruction while
  the inner shrinks) to show the fix is the general reconstruction-`DownEq`
  mechanism, not an Ackermann-specific special-case.
- **AC3 — `DownEq`-only, exact-reconstruction scope (the boundary is tested, not
  just asserted).** A dedicated test pins that the new edge is `DownEq` **only**
  for an exact same-size reconstruction and `Unknown` for any net structural
  increase/reorder/subst-wrap (the `badAck2` discriminator + a reorder variant).
  **No reconstruction ever yields `Down`.**
- **AC4 — Monotone, no regression.** `cargo test --workspace` green — a kernel
  change's blast radius is workspace-wide, validate the **full workspace**, not
  just `-p ken-kernel` ([[kernel-reduction-change-full-workspace-green]]).
  Completeness is **monotone**: every program SCT accepted before (all of (a)'s
  #12 nested-split cases included) is **still** accepted. Together with AC1 this
  pins the new boundary: `{old accepted} ⊊ {new accepted} ⊆ {terminating}`.
- **AC5 — Trust surface: no GROWTH.** `trusted_base()` **unchanged** — SCT is
  *already* in the TCB; this makes its analysis more complete, it does **not** add
  a `declare_primitive`/`declare_postulate` or a new `Term`/`Decl` variant. Confirm
  by grep (no new trusted declaration; no new variant).

## Guardrails (do-not-reopen)

- **SOUNDNESS FIRST — if you cannot clearly argue the reconstruction edge admits
  only terminating programs, STOP and escalate.** A single admitted non-terminator
  is a Bottom inhabitant, not a completeness win.
- **`DownEq`, never `Down`; exact reconstruction only.** The boundary is the
  `is_exact_reconstruction` predicate; the full near-miss net (`badAck`, `badAck2`,
  REORDER, WRONG-FIELD, WRONG-CTOR + classic non-terminators, AC1) must all reject
  / stay `Unknown`. Over-firing the predicate is the sole unsafe vector.
- **Fix the matrix construction, not the acceptance criterion or the closure
  engine.** No new early-accept path; `has_strict_diagonal` and
  `composition_closure_self_loops` stay verbatim.
- **Ground the current-rejection mechanism against landed `sct.rs`** (post-(a)
  `b34d4aa`) with `SCT_DEBUG` tracing — the root-cause above is a hypothesis.
- **This IS a kernel change** — no "kernel-untouched" AC; the obligation is
  *guarantee preserved*, verified by the AC1 adversarial net + the Architect
  soundness gate, not by an empty diff.

## Sequencing

- **Gate:** **Architect approach-review** (vet that the reconstruction-`DownEq`
  edge preserves the termination guarantee — the central, up-front gate for a
  trust-root change) → Kernel builds → **Architect soundness** on the candidate +
  **Kernel QA** + **CI**. **spec-leader confirms** whether the completeness-class
  needs a `spec/10-kernel` note (likely an implementation-completeness fix the
  spec's "terminating recursion is accepted via size-change" already covers — à la
  #5/(a) — but spec-leader makes the call; if `/spec` is touched, CV reviews).
- **Lane:** Kernel. Independent of `surface-transport`, the `[FS]` frame, and any
  in-flight `catalog/packages/` build — disjoint crate (`ken-kernel`), no contention;
  Kernel is idle and can start on approach-review now.
- **Downstream benefit:** general lexicographic/nested recursion (Ackermann and
  its kin) becomes admissible — a standing completeness win for the kernel, and it
  retires the last named piece of the `sct-completeness` decomposition.
