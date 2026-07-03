# L5 (effects, capabilities, state) conformance — seed cases

Format: `../../README.md`. These pin the **effect discipline** that **L5**
delivers (`docs/program/wp/L5-effects.md`, `spec/30-surface/36-effects.md`): a
statically-checked, transitively-inferred **effect row**; the **pure
interaction-tree** (`ITree`) denotation (the pure-kernel bridge);
**capabilities** as value tokens that gate ops; the **`space`** state model;
**tail-resumptive handlers**; and the **pure/impure** boundary hook L7 (FFI)
will plug into. They extend — and must not regress — the on-`main`
surface/elaboration invariants (`../seed-surface.md`,
`../elaboration/seed-elaboration.md`).

**Trust posture.** Effects are a **surface + elaboration + interaction-tree-
denotation** discipline; the kernel stays **pure** and re-checks only the pure
denotation, an ordinary inductive datatype (`ITree`, an admitted strictly-
positive `14` inductive — `OQ-8` "one pure kernel", `36 §2.1`). So a bug in
effect inference, capability gating, or handler folding **cannot make the kernel
unsound** — the emitted core term is still a well-typed pure tree. **No case
here is `(soundness)`** in the kernel-trust-root sense. The **one exception** —
the single thing that *would* be a genuine soundness regression — is **EFF2
`eff-kernel-checks-denotation-pure`**: if effect machinery leaked into the
**core term** the kernel checks, the small-TCB invariant breaks; that case is a
`(property)` the corpus must never let regress.

But the effect discipline's *own* guarantees are real **static-safety
properties** the security tier (`Sec1`/`Sec1ct`/`Sec2`) and behavioral export
(`B1`) build on (`36 §3.1`). An **undeclared effect escaping** (`§1.4`, the
*single soundness-relevant gate* of the row system), a **capability not gated**
(`§2.5`), a **multi-shot handler admitted** (`§5.2`), or **impure code
masquerading as pure** (`§7.2`) is a **discipline-level unsoundness** the kernel
will *not* catch (the tree is still pure data). Those are the load-bearing
guards. Per the **verdict-flip discipline** (`2cf1fc6`), each is pinned by a
**discriminating** case whose verdict **flips** on the targeted bug (right =
accept, wrong = reject); where a single verdict cannot be made to flip, the case
asserts a **structural** output (the inferred row, the `Vis`-tag tree shape, the
post-state) verdict-independently — the carry from V0/K2c
(`discriminating-conformance-verdict-must-flip`).

**Reconcile note (content-verified against landed `§1`–`§7`, `7129731`).**
Authored in parallel with spec-author, then reconciled against the *bodies* (not
just heading numbers — the V0 `§5.6` trap,
`conformance-oracle-grounding-fallback`). Five findings folded: (a) the tree
constructors are pinned `Ret`/`Vis` (`§2.1`) — `perform e = Vis e (λr. Ret r)`
is the smart-constructor, so structural assertions use **`Vis`** (no longer
`(oracle)`); (b) over-declaration is **accepted** — `ρ_inf ⊆ ρ_decl`,
declare-more-than-used is allowed (`§1.4`), so the former open `(oracle)` is now
the locked case `eff-overdeclared-upper-bound-accepted`; (c) the `ITree` **level
is forced** `max ℓ_R ℓ_op ℓ_resp` (`§2.1`, `§7.4`), now pinned by
`eff-itree-level-forced`; (d) attenuation/revocation internals are explicitly
`Sec2`/`62` (`§2.5`), so the attenuation case is **deferred** (below), not
locked here; (e) two spec-internal items flagged back to spec-author (below).

**Two flags to spec-author (independent-checker, not silently resolved).**
1. **Cite typo:** `§6` item 3 cites the cross-workstream contract as **`§3.4`**;
   the contract table + three guarantees are at **`§3.1`** (no `§3.4` exists).
2. **`§7.2` vs `§7.5.5` wording:** `§7.2` says a `pure` `foreign` is an
   *unchecked trusted claim* (allowed, postulate); `§7.5.5` says a
   `pure`-but-effectful `foreign` is *rejected*. Consistent only under "impure ≡
   non-empty row" (`§7.2`): a `foreign` whose **declared row is non-empty** but
   is labeled `pure` is self-contradictory (reject), while a fully-opaque `pure`
   `foreign` is the trusted postulate. Worth one clarifying clause; the exact
   `foreign` rule is `38`/L7's, so I pin only the fully-L5-settled escape at the
   boundary (EFF5).

**Tags.** `(oracle)` — confirmed at build time by the Spec enclave (safe:
effects are not in the kernel TCB): the **proposal-level surface *spelling***
(`visits ρ`, `using c : Cap`, `space`/`becomes`, handler/`do` notation — `36` is
*normative for the model and elaboration, `OQ-syntax` for spelling*), the
example op-tags (`Write`, `Get`/`Put`), and the error-kind strings. The **tree
constructors (`Ret`/`Vis`), the denotation `⟦·⟧`, the level formulae, and every
verdict** are **normative** (`§2`, `§7.4`) — not `(oracle)`. `(property)` — an
invariant over many inputs / an end-to-end closure, not a single trace.

**Scope (frame cut; deferred coverage flagged).** `36 §3.2`/`§7.5` name **IFC
labels** and the **`@ct`** constant-time discipline, and `§2.5`/`§3.2` put
**attenuation/revocation** in `../60-security/62`. The **frame scope** + this
WP's 5 acceptance criteria put `Sec1` (IFC), `Sec1ct` (constant-time), and
`Sec2` (capability *enforcement* — attenuation/revocation) as **separate WS-Sec
WPs that ride L5**. I author the effect/capability/state **model** they build on
(the `§3.1` contract) and **defer** IFC-label, `@ct`-taint, and
attenuation-lattice conformance to those WPs. L5 pins capability
**presence**-gating (`Cap E` in scope ⇒ accept; absent ⇒ reject,
`§2.5`/`§7.3.2`), not the subsumption/attenuation lattice. This
frame-vs-`§3.2`/`§7.5` scope split is the **scope fork** (COORDINATION §6)
flagged for spec-leader — I add `@ct`/IFC/attenuation cases only if ruled
in-scope for L5.

**Citations.** `36-effects.md` `§1.1` (row lattice, latent-effect arrows
`A →[ρ] B`), `§1.2` (transitive `infer_row`), `§1.3` (call-graph least-fixpoint,
no SCT), `§1.4` (declared-row ⊆ check, escape error + witness); `§2.1` (`ITree`
`Ret`/`Vis`, forced level), `§2.2` (`ret`/`perform`/`bind` = grafting via
`elim_ITree`), `§2.3` (row signature `⊕`), `§2.4` (denotation `⟦·⟧`, pure
collapse `ITree 𝟘 R ≅ R`), `§2.5` (capability-passing, `Cap E` as Π param);
`§3.1` (cross-workstream contract); `§4.1` (`space` → `State S`), `§4.2`
(`runState` state-passing fold), `§4.3` (bounded Hoare, `old`), `§4.4`
(shared-nothing); `§5.1` (handler = `elim_ITree` fold), `§5.2`
(tail-resumptive); `§7.1` (pipeline), `§7.2` (pure/impure L7 hook), `§7.3`
(error classes), `§7.4` (level table), `§7.5` (conformance pointers). Cross:
`12 §2`/`§3`/`§4` (levels, predicative/non-cumulative), `13 §1`/`§3` (Π/Σ-Form,
η), `14 §1`/`§2`/`§3` (inductive level, strict positivity, ι-reduction),
`10-kernel` TCB (ADR 0001/0004/0005), `38-ffi-io §3` (foreign boundary).

---

## EFF1 — effect row: transitive inference + static check (frame AC1)

A `view` is **pure by default**; an effectful one carries a static **effect
row** (`visits ρ`) **inferred transitively** from its body (`infer_row`, `§1.2`)
and **checked** `ρ_inf ⊆ ρ_decl` (`§1.4`) — performing an effect outside the
declared bound is the **single soundness-relevant gate** (`§1.4`, frame AC1).

### surface/effects/eff-row-inferred-transitively (oracle)
- spec: `36 §1.2`, `§1.1`
- given: leaf prims `read_config (p:String):Config visits [FS]` and
  `now ():Instant visits [Clock]`; a `view setup () : Config = read_config "/x"`
  with **no declared row**.
- expect: `infer_row` assigns `setup` the row **`[FS]`** — `read_config`'s
  latent row released at the call (`§1.2`, `f a` clause). Accepts; the inferred
  row is **exactly `[FS]`** (not `[]`, not `[FS, Clock]`).
- why: pins transitive inference as a **structural output** asserted
  verdict-independently. A bug that fails to release `read_config`'s latent
  `[FS]` infers `[]` (wrong) while the program still "accepts" — caught only by
  asserting the row, not the accept. (the V0/K2c structural-output carry.)

### surface/effects/eff-row-union-two-effects (oracle)
- spec: `36 §1.2` (`let` / sequencing clause), `§1.1` (join `∪`)
- given: `view boot () = { read_config "/x" ; now () }` — calls both leaves; no
  declared row.
- expect: inferred row = the **join `[FS, Clock]`** (lattice `∪`, `§1.1`; set
  normalization `(oracle)`). Accepts.
- why: ≥2 distinct effects — the row is the **join** over the body's calls. A
  bug taking only the first/last call's effect infers `[FS]` or `[Clock]`; the
  asserted join flips the structural check. (≥2-effects guardrail.)

### surface/effects/eff-undeclared-escapes-rejected (oracle)
- spec: `36 §1.4` (`ρ_inf ⊄ ρ_decl` ⇒ EFFECT-ESCAPE), §6 acceptance 1
- given: `view logged () : Unit visits [Console] = { greet "hi" ; now () }` —
  declares `[Console]`; `infer_row` = `{Console, Clock}` (`greet` + `now`), so
  `Clock ∉ ρ_decl`.
- expect: **static error** `EffectEscapes` (kind `(oracle)`) that **names each
  escaping effect** `Clock` **and a witness** — the `now ()` call whose latent
  row introduces `Clock` (`§1.4`: not just a set difference, a source site).
- why: **the escape-rejection guard** (the single soundness-relevant gate,
  `§1.4`). Verdict **flips** against `eff-declared-matches-used-accepted`:
  declaring `[Console, Clock]` accepts, omitting `Clock` rejects. The targeted
  bug — inference that does not check `ρ_inf ⊆ ρ_decl` — would accept *both* (no
  flip ⇒ guards nothing), so the **pair** pins it. The named-effect + witness is
  a structural assertion beyond the bare reject. (escape-rejection +
  verdict-flip.)

### surface/effects/eff-declared-matches-used-accepted (oracle)
- spec: `36 §1.4`
- given: the body of `eff-undeclared-escapes-rejected`, declared
  `visits [Console, Clock]`.
- expect: **accepts** — `ρ_inf = {Console, Clock} ⊆ ρ_decl` (here, equal).
- why: the **accept arm** that makes the escape case discriminating. Correct
  declaration accepts; the escaping one rejects → the verdict flips on the
  under-checking bug.

### surface/effects/eff-overdeclared-upper-bound-accepted (oracle)
- spec: `36 §1.4` ("`⊆`, not `=`: a function may declare more than it uses")
- given: the body of `eff-undeclared-escapes-rejected` (uses
  `{Console, Clock}`), declared `visits [Console, Clock, Net]` — `Net` is
  **never performed**.
- expect: **accepts** —
  `ρ_inf = {Console, Clock} ⊆ {Console, Clock, Net} = ρ_decl`. Declaring an
  **unused** effect is a legal upper bound (a stable interface reserving
  headroom), **not** an error.
- why: pins the row check as **`⊆` (upper bound)**, not `=` (exact) — the
  resolution of the over-declaration question `§1.4` settles. Verdict **flips**
  against a bug that checks `ρ_inf = ρ_decl`: that bug would *reject* this legal
  program. Locks the `⊆`-not-`=` semantics directly. (the reconcile-resolved
  case.)

### surface/effects/eff-pure-default-is-effect-free (oracle)
- spec: `36 §1.4` ("no `visits` ⇒ `ρ_decl = ∅`"), `§2.4` (pure collapse)
- given: `view double (n:Int):Int = n + n` — no effectful call, no row.
- expect: `infer_row = ∅`; accepts; denotes to `ITree 𝟘 ⟦Int⟧ ≅ ⟦Int⟧`, which
  the elaborator **collapses** to the plain term (`§2.4`) — usable where a pure
  function is required.
- why: the pure-default base case and the **EFF5 hinge**. A bug that infers a
  spurious effect for pure code (or breaks "no row ⇒ pure") is caught by the
  asserted empty row. Pairs with `pure-view-usable-in-pure-context`.

---

## EFF2 — pure interaction-tree (`ITree`) denotation: the pure-kernel bridge (AC2)

The effectful program **denotes to a pure `ITree`** — `Ret r` | `Vis e k`
(`§2.1`); `bind` is **tree grafting via `elim_ITree`** (`§2.2`, total); the
kernel checks the tree as an **ordinary inductive** with **zero effect
machinery** (`§2.1`, `§7.1`).

### surface/effects/eff-denotes-to-interaction-tree (oracle)
- spec: `36 §2.1` (`Ret`/`Vis`), `§2.2` (`perform`), `§2.4` (`⟦let⟧ = bind`),
  `§7.5.2`
- given: `view two_ops () visits [Console] = { greet "a" ; greet "b" }`, where
  `greet s ⤳ perform (Write s)` (`Console.Op = { Write String }`, `§2.1`).
- expect: the denotation is the **pure** `ITree` term
  `Vis (Write "a") (λ_. Vis (Write "b") (λ_. Ret unit))` — **two** `Vis` nodes
  (one per op, **in source order**), each a function-continuation over the
  response, terminating in `Ret unit`. Op-tag spelling `(oracle)`; the
  `Vis`/`Ret` **shape is normative** (`§2.1`).
- why: AC2 as a **structural** assertion — **the `Vis`-tag sequence**, not
  "elaborates" (`§7.5.2`). N ops ⇒ N nested `Vis` nodes in order, `Ret` at the
  leaf. A bug that drops an op, reorders, or mis-threads the continuation is
  caught structurally. (interaction-tree-structure guardrail.)

### surface/effects/eff-bind-is-tree-grafting (oracle)
- spec: `36 §2.2` (`bind (Ret a) k = k a` ;
  `bind (Vis e f) k = Vis e (λr. bind (f r) k)`)
- given: `bind m k` with `m = perform e` (`= Vis e (λr. Ret r)`, `§2.2`) and
  `k = λx. perform e2`.
- expect: bind **grafts `k` onto the `Ret` leaf**, threading the response, to
  `Vis e (λr. Vis e2 (λr2. Ret r2))` — by the `§2.2` equations:

  ```
  bind (Vis e (λr. Ret r)) k
    = Vis e (λr. bind (Ret r) k)      -- bind on Vis
    = Vis e (λr. k r)                 -- bind (Ret r) k = k r
    = Vis e (λr. Vis e2 (λr2. Ret r2))   -- k = λx. perform e2
  ```
- why: pins **bind = grafting** structurally (the exact `§2.2` equations). A bug
  that sequences by another rule (concatenating `Vis` nodes without threading
  the response through `f r`, or grafting at the wrong leaf) is caught; a
  response capture/threading bug shows here. `bind` is `elim_ITree` on `m`,
  hence total (`§2.2`, `14 §3`) — no SCT.

### surface/effects/eff-kernel-checks-denotation-pure (property)
- spec: `36 §2.1` (`ITree` is an admitted strictly-positive `14` inductive),
  `§2.4`, `§7.1` (kernel step: no effect rule), `10-kernel` TCB (ADR 0001/0004)
- given: the denotation term from `eff-denotes-to-interaction-tree`, handed to
  the kernel as a value of the `ITree ⟦ρ⟧ R` inductive type.
- expect: kernel-check **Ok** — the kernel sees **only** Π/Σ/inductive/`ITree`
  (`§7.1`); `Vis`'s recursive argument `E.Resp e → ITree E R` is strictly
  positive (`§2.1`, `14 §2`). **No effect primitive, no row, no capability
  appears in the core term**; rows are discharged by inference, authority by Π
  over `Cap`, before the kernel (`§7.1`).
- why: **the `OQ-8` "one pure kernel" invariant, end-to-end** (frame AC2,
  `§7.1`). A bug that leaks an effect primitive into the core term violates the
  small-TCB invariant. **The one genuine soundness regression in the file**
  (effect machinery in the TCB), hence a `(property)` over *every* effectful
  denotation that must never regress.

### surface/effects/eff-itree-level-forced (oracle)
- spec: `36 §2.1` ("the level is *forced*"), `§7.4` (level table), `12 §2`/`§3`,
  `14 §1`
- given: `ITree E (R : Type ℓ_R)` over an effect `E` with `Op : Type ℓ_op`,
  `Resp : Op → Type ℓ_resp`; concretely (a) first-order `Console` with
  everything at level 0, `R = Unit : Type 0`; (b) `State S` over `S : Type ℓ_S`.
- expect: (a) `ITree Console Unit : Type 0` — the least level
  `max ℓ_R ℓ_op ℓ_resp = max 0 0 0 = 0`; (b)
  `ITree (State S) R : Type (max ℓ_R ℓ_S)` (`§7.4`). The level is the
  **predicative `max`** of the parts (`12 §2`), **non-cumulative** (no implicit
  lift, `12 §3`); the elaborator emits it explicitly and the kernel re-checks
  (`12 §4`). `Effect : Type (suc (max ℓ_op ℓ_resp))`.
- why: pins the **exact** forced level (skill: assert the precise level, never a
  loose "some universe"). A bug that picks `Type 0` unconditionally, or lifts
  cumulatively, or drops a universe is caught by the asserted `max`. The
  level-discipline reconcile (`§7.4`) made executable. (level-precision guard.)

---

## EFF3 — capabilities gate effectful ops (frame AC3)

A **capability** is a value token (`Cap E`, `§2.5`/`OQ-8a`) a `perform_E op`
requires **in scope** — threaded by Π/λ or minted by an enclosing handler
(`§5.1`); absent ⇒ **missing-capability** error (`§7.3.2`). ≥2 distinct caps;
denial path on each. (L5 pins **presence**-gating; subsumption/attenuation is
`Sec2`/`62`, deferred.)

### surface/effects/cap-op-without-token-rejected (oracle)
- spec: `36 §2.5` (`perform_E` well-formed only if `Cap E` in scope), `§7.3.2`
- given: `write_file` declared `using fs : FsCap`; a
  `view dump () : Unit visits [FS] = write_file "/x" data` with **no** `Cap FS`
  in scope (no capability parameter, no enclosing handler provides it).
- expect: **static error** — `MissingCapability(FsCap)` (kind `(oracle)`,
  `§7.3.2`): the `perform` is gated on the `Cap E` value's presence, unprovided.
- why: the **capability-denial path** (`§7.3.2`).

### surface/effects/cap-op-with-token-accepted (oracle)
- spec: `36 §2.5`, `§5.1` (a handler provides the capability)
- given: the same op, but `dump` takes `using fs : FsCap` (or an enclosing
  handler for `FS` provides it, `§5.1`).
- expect: **accepts** — `Cap FS` is in scope (a Π parameter, `§2.5`); gating
  satisfied.
- why: the **flip** for `cap-op-without-token-rejected` (the `§7.5.3`
  denial-path flip). With the token accepts, without rejects → the verdict flips
  on the exact bug; a checker ignoring the `Cap E` parameter would accept *both*
  (no flip), so the pair pins it. (capability-gating + verdict-flip.)

### surface/effects/cap-two-distinct-caps-each-gated (oracle)
- spec: `36 §2.5` (one `Cap E` parameter per un-handled effect)
- given: `view exfil () visits [FS, Net] = { write_file "/x" d ; send sock d }`
  — `write_file using fs:FsCap`, `send using net:NetCap`. Three variants: (a)
  both caps in scope; (b) only `fs`; (c) only `net`.
- expect: (a) **accepts**; (b) **rejects** `MissingCapability(NetCap)`; (c)
  **rejects** `MissingCapability(FsCap)`.
- why: ≥2 distinct capabilities, **each independently gated** — one case per
  guard position (COORDINATION §7). A bug checking only the first cap admits
  (c)'s missing `FsCap`; a bug checking only the last admits (b)'s missing
  `NetCap`. Each per-cap reject flips independently. (capability-denial on each
  of two caps.)

---

## EFF4 — `space` state + tail-resumptive handlers (frame AC4)

A `space` desugars to a `State S` effect (`§4.1`); `becomes` is a
`Get`-then-`Put` on the pure tree, discharged by `runState`, the canonical
tail-resumptive fold (`§4.2`). Handlers are `elim_ITree` folds,
**tail-resumptive only** (`§5`, `OQ-9`).

### surface/effects/space-becomes-threads-state (oracle)
- spec: `36 §4.1` (`becomes ⤳ Get`-then-`Put`), `§4.2` (`runState`)
- given:

  ```
  space Counter {
    mut n : Int = 0
    view inc () : Unit visits [Counter] = n becomes n + 1
    view get () : Int  visits [Counter] = n
  }
  ```

  program `{ inc() ; inc() ; get() }`, discharged by `runState 0`.
- expect: `runState 0 ⟦body⟧` returns **`(2, 2)`** — the `get()` result `2`
  paired with the **final state** `n = 2` (`§4.2`: `runState` returns `R × S`);
  when the outer row is `𝟘` it collapses (`§2.4`) to the value `(2, 2)`. The
  fold threads `n: 0 → 1 → 2` via `Get`/`Put`.
- why: `space` semantics — `becomes` is a `Get`-then-`Put` (`§4.1`), discharged
  by the state-passing fold `runState` (`§4.2`). A bug that fails to thread
  state (each `inc` reads the initial `n = 0`) yields `(1, 1)`, not `(2, 2)` —
  the asserted final-state value flips (`§7.5.4`). (structural/value assertion.)

### surface/effects/space-old-scoped-to-ensures (oracle)
- spec: `36 §4.3` (`old(e)` = `e` in the pre-state; worked `inc` example)
- given: `view inc() visits [Counter] ensures n == old(n) + 1 = n becomes n+1`;
  and a variant asserting `n == old(n) + 2`.
- expect: the **`+1`** `ensures` **discharges** — `inc` denotes to the
  transformer `λ s. (tt, s with .n := s.n+1)`, and the obligation computes
  (record-β/η, `13 §3`) to `s.n+1 == s.n+1`, closed by `refl` (`§4.3`, `16 §2`).
  The **`+2`** variant is **disproved** (obligation `s.n+1 == s.n+2`, no model).
- why: pins `old` as the pre-state value **scoped to the op's `ensures`** (not a
  global `\old`), grounded in the state-transformer denotation (`§4.3`). Verdict
  **flips**: the correct postcondition proves, the wrong one disproves.
  (bounded-Hoare guardrail, `§4.3`/`OQ-Space`.)

### surface/effects/space-shared-nothing-no-cross-space-alias (oracle)
- spec: `36 §4.4` (shared-nothing message-passing; isolation **guarantee**)
- given: two spaces `A`, `B`; (a) `A` **directly** reads/writes `B`'s `mut` cell
  (aliases `B`'s `n`); (b) `A` **sends** an immutable, content-addressed value
  to `B` by message-passing.
- expect: (a) **static error** `CrossSpaceAlias` (kind `(oracle)`); (b)
  **accepts**.
- why: the **shared-nothing isolation** guarantee — no shared mutable state ⇒ no
  data races (`§4.4`), on which capability confinement (`Sec2`, `62 §4`) rests.
  Verdict **flips**: legal message-passing accepts, illegal aliasing rejects. A
  bug permitting cross-space aliasing breaks isolation **silently** (the program
  still kernel-checks — each space's `State S` is well-typed) — caught only
  here. (isolation guardrail.)

### surface/effects/handler-tail-resumptive-folds (oracle)
- spec: `36 §5.1` (handler = `elim_ITree` fold), `§5.2` (resume once, tail
  position)
- given: a handler for `Console` interpreting `Write s` by collecting `s` and
  resuming with `unit` in tail position (`ops e (λr. handle ret ops (k r))`,
  `§5.1`); handle `{ greet "a" ; greet "b" }`.
- expect: the handler **folds** the `ITree`, the resume `λr. handle … (k r)`
  invoked **once per** `Vis` node **in tail position**; result = the folded
  accumulation (e.g. `["a","b"]`) with `Ret` mapped by `ret` (`§5.1`). Assert
  the fold visits **both** `Vis` nodes **in order**, resuming once each.
- why: tail-resumptive handler = **structural `elim_ITree` fold** (`§5.1`,
  catamorphism). Structural: each `Vis` node consumed exactly once, in order,
  resumed in tail position. (handler-resume guardrail, `§7.5.4`.)

### surface/effects/handler-multishot-rejected (oracle)
- spec: `36 §5.2` (`OQ-9`: resume **at most once, in tail position**), `§7.3.3`
- given: two handlers for the same effect: (a) **tail-resumptive** (resumes
  once, tail position); (b) a handler invoking the resume **twice** (or in
  **non-tail** position, or capturing it as a first-class value) — a multi-shot
  / `shift`-style handler.
- expect: (a) **accepts**; (b) **static error** `NonTailResumptive` (kind
  `(oracle)`, `§7.3.3`) — `ops` resumes more than once / not in tail position /
  reifies the resumption.
- why: **the `OQ-9` exclusion guard** — single-shot keeps `handle` a plain
  `elim_ITree` catamorphism, preserving totality (`§5.2`, `17 §4`) and
  single-consumption WP reasoning. Verdict **flips**: tail-resumptive accepts,
  multi-shot rejects. A bug admitting multi-shot continuations breaks
  totality/WP-soundness for effectful code, which the **kernel will not catch**
  (the tree is still pure data) — a load-bearing discipline guard. (`§5.2`
  exclusion + verdict-flip.)

---

## EFF5 — pure/impure boundary hook for L7 FFI (frame AC5)

`pure ≡ empty row`, `impure ≡ non-empty row` (`§7.2`). A `foreign` carries a
**mandatory** row; its operations are the **`Vis` nodes at the world frontier**.
L5 fixes the interface (the `Effect` signature + every foreign op is a `Vis`
node); **L7 supplies the interpreters** (`§7.2`).

### surface/effects/pure-view-usable-in-pure-context (oracle)
- spec: `36 §1.4`/`§7.2` (`pure ≡ ρ = ∅`), `§2.4` (collapse)
- given: `double` (row `∅`, from `eff-pure-default-is-effect-free`) used where a
  pure function is required — inside a `requires`/`ensures` predicate or a total
  pure combinator.
- expect: **accepts** — the empty row collapses the denotation to the plain term
  (`ITree 𝟘 R ≅ R`, `§2.4`), so the verification layer treats `double` as a
  mathematical function (its `ensures` are about values, not world-state).
- why: AC5's **pure half** — the `∅` row + collapse is the certificate
  L7/verification rely on. Pairs with `eff-pure-default-is-effect-free`.

### surface/effects/impure-boundary-marker-exposed (property, oracle)
- spec: `36 §7.2` (the L7 FFI hook), `§3.1` (every authority act is a `Vis`
  node)
- given: a `foreign` op with a **non-empty** declared row (e.g.
  `foreign read_clock () : Instant visits [Clock]`) — its operation is a `Vis`
  node at the world frontier; and a `view` calling it.
- expect: the impure marker is the **non-empty row** (`impure ≡ ρ ≠ ∅`, `§7.2`),
  **visible in the type**; a caller **inherits** `Clock` in its inferred row
  (`§1.2`, propagates transitively like any effect). L5 exposes the `Effect`
  signature + `Vis`-node interface; **L7 plugs in the interpreter** (the
  real-world `handle` whose `ops` perform actual I/O, `§7.2`) — no L5 FFI impl.
- why: AC5 — L5 provides only the wiring point (`§7.2`). Pins that the impure
  marker (a non-empty row) is *exposed* and *propagates*, and that the foreign
  op is a `Vis` node (the `§3.1` guarantee L7/Ward read off). (`property`: the
  marker propagates through inference for every caller.)

### surface/effects/impure-masquerading-as-pure-rejected (oracle)
- spec: `36 §1.4` (escape), `§7.2` (`impure ≡ non-empty row`)
- given: a pure-typed `view safe () : Int = read_clock ()` where `read_clock` is
  impure (`visits [Clock]`), but `safe` declares **no row** (`ρ_decl = ∅`,
  claims purity).
- expect: **static error** — `EffectEscapes(Clock)` (kind `(oracle)`):
  `ρ_inf = {Clock} ⊄ ∅`, so an impure op cannot be called from a pure-typed
  (empty-row) view without surfacing the effect (`§1.4`).
- why: the boundary's **integrity** — impure cannot silently masquerade as pure,
  the property the "no row ⇒ pure" certificate (and all of verification, IFC, CT
  downstream) depends on. Verdict **flips**: declaring `visits [Clock]` accepts,
  claiming purity rejects. EFF1's escape **re-applied at the pure/impure
  boundary** — the AC5 integrity guard. (The `foreign`-declared-`pure` variant
  of `§7.5.5` is the same principle at the `38`/L7 boundary; deferred per the
  flag above.) A bug here lets impure code be trusted as pure fleet-wide.
  (verdict-flip.)

---

## EFF6 — the direct `[State s]` effect surface (`OQ-C`·C2, frame AC-State)

A **second door** to the same `State S` denotation (`§2.1`): writing
`get`/`put`/`runState` **directly** as a first-class monadic effect — the
`accumulator-factory` shape (VAL2 #10) — instead of through the imperative
`space` sugar EFF4 pins. It introduces **no new denotation**: `runState` on the
direct surface is **exactly** `§4.2`'s state-passing `elim_ITree` fold at
`F = 𝟘` (`§4.5.3`), and `get`/`put` are `§2.1`'s `Get`/`Put` under `§2.2`'s
`perform` (`§4.5.2`). So EFF6 shares EFF4's trust posture — a bug here is a
**wrong value in a pure tree**, never a kernel unsoundness (the emitted core is
the same well-typed `elim_ITree` term); **no case here is `(soundness)`** in the
kernel-trust-root sense (seed preamble). `runState` is a **derived total
definition**, not a trusted primitive, so `[State s]` is **zero-`trusted_base`
delta** (frame AC1) — see `direct-state-kernel-untouched` below.

**Build-FORCING — these cases are the acceptance *target*, RED on `main` until
built (do NOT hand-feed them green).** Grounding (`origin/main`, verified at
authoring, git-object-store): the landed **runnable** `ITree` a program
executes is **Console-hardwired** — `data ITree r = Ret r | Vis ConsoleOp
(Unit -> ITree r)` (`crates/ken-elaborator/src/prelude.rs:170`, `IO A := ITree
A`): a **fixed `Unit` response**, not `E`-parameterized, so it cannot yet
express `get`/`put` with the non-`Unit` `Resp Get = s` (`§4.5.2`). (The
elaborator's `effects/itree.rs` is a static-analysis stand-in with a fixed
`u64` response; the full dependent-`Resp` `ops` version is deferred there too.)
The interpreter's `runState`-over-`Vis` fold is a **K1.5-IH-deferred
placeholder** (`crates/ken-interp/src/lib.rs`
`eff4_runstate_discharges_in_pure_section3` /
`eff4_handled_discharges_unhandled_reaches_driver`). So driving `next` through
the **real interpreter** (frame AC2) **cannot pass today** — it flips green only
when the build lands the three `§36`-sanctioned **outer-ring** lifts
(`§4.5.6`): **(a)** the **dependent response** `E.Resp e` (`State s` is the
first effect whose `Resp Get = s` is non-`Unit`), **(b)** the container
coproduct `⊕` for `State s ⊕ F`, **(c)** **named-effect dispatch** so `runState`
peels `State` and passes other ops through. All three are `§36`-normative and
admitted by K1.5's generic `elim_ITree` (`kernel/tests/k1p5_wstyle.rs`) — **the
kernel is untouched** (frame AC1). These cases are authored as the deliverable's
acceptance target — like EFF4's own deferred `eff4_runstate` interp test — **not
green against a pre-supplied `(result, state)`**, which is exactly the
[[conformance-hand-feeds-the-deliverable]] green-vs-green the frame warns
against. The case **defines "done"**; it is correctly red until Runtime/Language
build it.

### surface/effects/direct-state-next-post-increment (oracle, build-forcing)
- spec: `36 §4.5.3` (`runState` = `§4.2` fold at `F = 𝟘`; worked `next`),
  `§4.5.2` (`get`/`put` = `§2.1` `Get`/`Put` under `perform`), `§2.4` (`𝟘`
  collapse)
- given: the direct-surface post-increment `next` — returns the current value,
  bumps the state:

  ```
  next () = bind (get ()) (λ n. bind (put (n + 1)) (λ _. Ret n))
  ```

  driven through the **real interpreter** under `runState 0` and `runState 41`
  (not a hand-fed harness — per the build-forcing note above).
- expect: `runState 0 next` reduces to the pair **`(0, 1)`** and `runState 41
  next` to **`(41, 42)`** — `(result, final-state)`: the result is the **old**
  value (`get` observes the pre-increment state), the second component is the
  **final** state after `put (n+1)`. Re-derive (`§4.2` clauses): `runState 0
  (Vis Get k) = runState 0 (k 0)` (answer with current `0`) → `k 0 = Vis (Put 1)
  (λ_. Ret 0)` → `runState 0 (Vis (Put 1) k') = runState 1 (k' tt) = runState 1
  (Ret 0) = Ret (0, 1)`; at `F = 𝟘` collapses (`§2.4`) to `(0, 1)`.
- why: **verdict-independent structural value** — the pair **flips** on the two
  bugs the direct surface must exclude, and (by `result 0 ≠ state 1`, chosen so)
  pins the pair **order** the equal-valued EFF4 `(2,2)` cannot: **(a)** a
  state-threading bug (`Put` not adopted — `runState s (Vis (Put s') k) =
  runState s (k tt)`) yields **`(0, 0)`**; **(b)** a pair-order swap
  (`runState s (Ret r) = Ret (s, r)`) yields **`(1, 0)`**. Both differ from
  `(0, 1)` regardless of downstream type-checking. Promoted from EFF4's
  `space-becomes-threads-state` to the **direct monadic door**; the order pin is
  new. (structural/value assertion; build-forcing.)
- oracle: the denotation returns the **Σ-pair `R × S`** (`§4.2`; interp
  `EvalVal::Pair`), *not* the also-landed inductive `data Prod a b = MkProd a b`
  (`prelude.rs:160`) — a distinct construct (`§4.5.3`). Assert the pair's
  **components and order** and its Σ-pair denotation; the **surface constructor
  spelling** is deferred `OQ-syntax` (`§4.5.2`), so do **not** freeze a
  constructor name (over-freezing a deferred spelling would falsely fail a valid
  build once the stdlib pair name finalizes).

### surface/effects/direct-state-console-commute (oracle, build-forcing)
- spec: `36 §4.5.4` (composition via `⊕`; handler nesting = discharge order;
  `State`/`Console` commute), `§4.2` (the `Vis (inr o)` pass-through clause)
- given: a `[State Int, Console]` program that threads state **and** logs —
  `logged_next () = bind (get ()) (λ n. bind (put (n + 1)) (λ _. bind (perform
  Console (Write "log")) (λ _. Ret n)))` — discharged two ways: `runState 0
  (handleConsole m)` (Console peeled first) and `handleConsole (runState 0 m)`
  (State peeled first).
- expect: **both type-check and thread state identically** — result pair
  `(0, 1)` and one `Console.Write "log"`, whichever handler peels first.
  `runState` discharges the `State` summand by its `inl` clauses and **passes
  every Console op through untouched** by `§4.2`'s `Vis (inr o) k` clause;
  `handleConsole` never reads `State` ops — so the two handlers **commute**.
- why: composition (frame AC4) — a bug where `runState`'s pass-through clause
  **consumes/drops/reorders** the `Console` op (or mis-threads state across an
  interleaved foreign `Vis`) makes the two discharge orders **diverge** (a
  different final state, or a dropped/duplicated `Write`) — the commutation
  **flips**. A single one-order case would be green-vs-green under a
  pass-through bug; the two orders are the net. (discriminating on
  discharge-order; build-forcing.)

### surface/effects/direct-state-no-cross-run (oracle, build-forcing)
- spec: `36 §4.5.5` (purity/totality; result = function of `s₀` + pure tree;
  the C3-forbidden face), `§4.5.6`/`OQ-C` (C3 mutable refs forbidden)
- given: the **same** pure tree `next` (above) run twice from two initial
  states in one computation — `runState 0 next` then `runState 41 next` — with
  **no** `runState` nesting between them (two independent top-level discharges).
- expect: **two independent results `(0, 1)` and `(41, 42)`** with **no
  cross-run state** — the second run re-threads from `41`, *not* from the first
  run's final state `1`. The result is determined **entirely by `s₀` and the
  pure tree** (`§4.5.5`); `runState s₀ m` is re-runnable and idempotent in `m`.
- why: the **C2/C3 boundary** the WP must preserve (frame AC3) — state is
  `runState`'s **parameter**, threaded functionally, never a cell. A bug that
  introduced a **shared mutable cell** (the forbidden C3 shape) would leak
  run-1's final state into run-2, yielding **`(1, 2)`** for the second run
  instead of `(41, 42)` — the re-runnability **flips**. The pure fold
  structurally cannot leak; this case is the executable witness that no in-place
  mutation entered on the value path (grep-face: no `RefCell`/`Cell`/interior
  mutability, `§4.5.5`). (discriminating on cross-run isolation; build-forcing.)

### surface/effects/direct-state-kernel-untouched (property)
- spec: `36 §4.5.3` (`runState` is a derived total def, kernel-re-checked, not a
  trusted Rust primitive), `§4.5.6` (no new `Term`/`Decl` variant; outer-ring
  lift only), frame AC1
- given: the delivered `[State s]` surface — its `get`/`put` denotations, the
  `runState` handler, and the effect-row machinery for `State s ⊕ F`.
- expect: **`git diff origin/main -- crates/ken-kernel/` stays empty** and
  `trusted_base()` is unchanged — no `State`/`Get`/`Put`/`runState` kernel
  `Term` or `Decl` variant, no new `declare_primitive`/`declare_postulate`.
  `runState` is an **ordinary total Ken definition** (the `§4.2` `elim_ITree`
  fold) the kernel re-checks like any term; the runtime merely *evaluates* it
  (it is **not** an I/O driver like Console's `run_io`, `§7.2` — pure state
  threading performs no I/O).
- why: frame AC1 is **load-bearing and structural** — `[State s]`'s
  zero-`trusted_base`-delta is grounded in the **kernel being untouched**
  (grep: no `State` effect machinery in `ken-kernel`; K1.5's generic
  `elim_ITree` already admits the `Vis`-tree), **not** a runtime assertion. If a
  build path ever needs a kernel `Term`/`Decl` variant for state, **that is the
  finding** (C2 → C3 boundary breach). The `runState`-derived-not-postulated
  split is what keeps the direct surface as cheap as EFF4's `space` door.
  (`property`: TCB-surface invariant, no value-flip — asserts the *absence* of a
  kernel delta.)

**Cross-case reconcile (EFF6 ↔ EFF4 — same fold, two doors).**
EFF4's `space-becomes-threads-state` (`runState 0 {inc;inc;get} = (2,2)`) and
EFF6's `direct-state-next-post-increment` (`runState 0 next = (0,1)`) exercise
the **same** `§4.2` `runState` fold through **different surfaces** — the
imperative `space` sugar (`§4.1`) vs the direct monadic `get`/`put` (`§4.5`).
They are **not** contradictory: different programs, same handler. EFF6
**strengthens** the shared mechanism on the **pair-order** dimension EFF4 leaves
unpinned — EFF4's `(2,2)` has `result == state`, so a pair-order swap is
invisible to it; EFF6's `result 0 ≠ state 1` makes the swap flip. Mechanism
check (per V2): both assert `runState s₀ ⟦m⟧ = (result, final-state)` with the
result the observed value and the second component the final state — the direct
and space doors **agree on the fold's shape**.

---

## Regression — L5 is additive over surface/V0

### surface/effects/existing-surface-invariants-still-green (property)
- spec: `../seed-surface.md` (`well-typed-output`, `ambiguity-is-an-error`),
  `../elaboration/seed-elaboration.md` (the V0 pipeline), `36 §2.4` (pure
  collapse)
- given: the on-`main` surface/elaboration invariants and the V0
  `lex → parse → resolve → elaborate → kernel-check` seeds.
- expect: **unchanged** — L5 **extends** surface conformance with effects; it
  must not regress pure-elaboration or the V0 pipeline. A pure `view` (row `∅`)
  denotes to `ITree 𝟘 R`, which **collapses to the identical core term** the V0
  elaborator emits (`§2.4`) — effects are additive: no row ⇒ the V0 path is
  untouched.
- why: L5 is **additive** over V0/surface, and `§2.4`'s collapse is what makes
  it literally so (pure code pays nothing for the encoding). Pins that
  introducing the effect discipline does not perturb pure-program elaboration.
  (`property`: regression guard over the existing corpus.)
