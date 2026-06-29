# L5 (effects, capabilities, state) conformance — seed cases

Format: `../../README.md`. These pin the **effect discipline** that **L5**
delivers (`docs/program/wp/L5-effects.md`, `spec/30-surface/36-effects.md`): a
statically-checked, transitively-inferred **effect row**; the **pure
interaction-tree** denotation (the pure-kernel bridge); **capabilities** as
value tokens that gate ops; the **`space`** state model; **tail-resumptive
handlers**; and the **pure/impure** boundary hook L7 (FFI) will plug into. They
extend — and must not regress — the on-`main` surface/elaboration invariants
(`../seed-surface.md`, `../elaboration/seed-elaboration.md`).

**Trust posture.** Effects are a **surface + elaboration + interaction-tree-
denotation** discipline; the kernel stays **pure** and re-checks only the pure
denotation, an ordinary inductive datatype (`OQ-8` "one pure kernel", `36 §2`).
So a bug in effect inference, capability gating, or handler folding **cannot
make the kernel unsound** — the emitted core term is still a well-typed pure
tree. **No case here is `(soundness)`** in the kernel-trust-root sense. The
**one exception** — the single thing that *would* be a genuine soundness
regression — is **EFF2 `eff-kernel-checks-denotation-pure`**: if effect
machinery leaked into the **core term** the kernel checks, the small-TCB
invariant breaks; that case is a `(property)` the corpus must never let regress.

But the effect discipline's *own* guarantees are real **static-safety
properties** the security tier (`Sec1`/`Sec1ct`/`Sec2`) and behavioral export
(`B1`) build on. An **undeclared effect escaping**, a **capability not gated**,
a **multi-shot handler admitted**, or **impure code masquerading as pure** is a
**discipline-level unsoundness** the kernel will *not* catch (the tree is still
pure data). Those are the load-bearing guards. Per the **verdict-flip
discipline** (`2cf1fc6`), each is pinned by a **discriminating** case whose
verdict **flips** on the targeted bug (right = accept, wrong = reject); where a
single verdict cannot be made to flip, the case asserts a **structural** output
(the inferred row, the tree shape, the post-state) verdict-independently — the
carry from V0/K2c (`discriminating-conformance-verdict-must-flip`).

**Tags.** `(oracle)` — confirmed at build time by the Spec enclave (safe:
effects are not in the kernel TCB): the **proposal-level surface syntax**
(`visits [E]`, `using c : Cap`, `space`/`becomes`, handler/`do` notation — `36`
is *normative for the model, proposal-level for syntax*), the exact
tree-constructor names, the error-kind strings, and the attenuation lattice.
`(property)` — an invariant over many inputs / an end-to-end closure, not a
single trace.

**Scope flag (raised to spec-leader, not silently resolved).** `36 §3`/`§6`
mention **IFC labels** and the **`@ct`** constant-time discipline as part of
what WS-L delivers, but the **frame scope** and this WP's 5 acceptance criteria
put `Sec1` (IFC-by-typing) and `Sec1ct` (constant-time) as **separate WS-Sec
WPs that ride L5, not part of it**. I author the **capability/effect/state
model** those WPs build on, and **defer** IFC-label and `@ct`-taint conformance
to `Sec1`/`Sec1ct`. This frame-vs-`§6` scope tension is a **scope fork**
(COORDINATION §6) — flagged for spec-leader to rule; I add `@ct`/IFC cases here
only if it is ruled in-scope. Likewise **attenuation/revocation** *enforcement*
is `Sec2` (`62`); EFF3 pins only the L5-exposed gating **contract** (subsumption
direction), lattice specifics `(oracle)`.

**Citations.** `36-effects.md` §1 (static row, transitive inference, pure-by-
default), §2 (three-layer encoding; interaction tree `Ret`/`perform`; bind =
grafting; one pure kernel), §3 (capabilities = `requires`-as-capability; value
tokens `OQ-8a`; attenuation), §4 (`space`, `becomes`, state-passing fold, `old`
scoped to space ops, shared-nothing isolation, `OQ-Space`), §5 (tail-resumptive
handlers = folds; multi-shot excluded, `OQ-9`), §6 (deliverable + acceptance).
Cross: `12-universes` (levels), `10-kernel` purity/TCB (ADR 0001/0004/0005),
`38-ffi-io §3` (foreign boundary). The §-cites anchor to the **current**
`36-effects` model; spec-author is elaborating `§6` into concrete sub-sections
in parallel — these are **reconciled against the landed `§6` content** (not just
the heading numbers) before the merge Decision, per the V0 `§5.6`
stage-refinement trap (`conformance-oracle-grounding-fallback`).

---

## EFF1 — effect row: transitive inference + static check (frame AC1)

A `view` is **pure by default**; an effectful one carries a static **effect
row** (`visits [E]`) that is **inferred transitively** from its body and
**checked** — performing an effect outside the declared/inferred bound is a
**static error** (`36 §1`, §6).

### surface/effects/eff-row-inferred-transitively (oracle)
- spec: `36 §1`
- given: leaf prims `read_config (p:String):Config visits [FS]` and
  `now ():Instant visits [Clock]`; a `view setup () : Config = read_config "/x"`
  with **no declared row**.
- expect: inference assigns `setup` the row **`[FS]`** — transitively from its
  one effectful call. Accepts; the inferred row is **exactly `[FS]`** (not `[]`,
  not `[FS, Clock]`).
- why: pins transitive inference as a **structural output** asserted
  verdict-independently. A bug that fails to propagate `read_config`'s `[FS]`
  infers `[]` (wrong) while the program still "accepts" — caught only by
  asserting the row, not the accept. (the V0/K2c structural-output carry.)

### surface/effects/eff-row-union-two-effects (oracle)
- spec: `36 §1`
- given: `view boot () = { read_config "/x" ; now () }` — calls both leaves; no
  declared row.
- expect: inferred row = the **union `[FS, Clock]`** (set order / normalization
  `(oracle)`). Accepts.
- why: ≥2 distinct effects — the row is a **set union** over the body's calls. A
  bug taking only the first/last call's effect infers `[FS]` or `[Clock]`; the
  asserted union flips the structural check. (≥2-effects guardrail.)

### surface/effects/eff-undeclared-escapes-rejected (oracle)
- spec: `36 §1`, §6 ("performing an undeclared effect is a compile error")
- given: `view logged () : Unit visits [Console] = { greet "hi" ; now () }` —
  declares `[Console]`; body uses `Console` (`greet`) **and** `Clock` (`now`),
  but `Clock` is **not** declared.
- expect: **static error** — `EffectEscapes(Clock)` (kind `(oracle)`): the
  declared row omits an effect the body actually performs.
- why: **the escape-rejection guard.** Verdict **flips** against
  `eff-declared-matches-used-accepted` below: declaring `[Console, Clock]`
  accepts, omitting `Clock` rejects. The targeted bug — inference that does not
  check the declared row against used effects — would accept *both* (no flip ⇒
  guards nothing), so the **pair** pins it. (escape-rejection + verdict-flip.)

### surface/effects/eff-declared-matches-used-accepted (oracle)
- spec: `36 §1`
- given: the body of `eff-undeclared-escapes-rejected`, declared
  `visits [Console, Clock]`.
- expect: **accepts** — declared row ⊇ used `{Console, Clock}` (here, equal).
- why: the **accept arm** that makes the escape case discriminating. Correct
  declaration accepts; the escaping one rejects → the verdict flips on the
  under-checking bug.
- **Open `(oracle)` — flagged to spec-author for `§6`:** is a declared
  **superset** naming an *unused* effect (e.g. `[Console, Clock, Net]` with
  `Net` never performed) an **error** or an allowed **upper bound**? `§1`
  settles only that omitting a *used* effect is an error; it does **not** settle
  over-declaration. Not locking an unground verdict — a case lands once `§6`
  pins exact-row vs. upper-bound.

### surface/effects/eff-pure-default-is-effect-free (oracle)
- spec: `36 §1` (pure by default), §6 ("a pure-typed view is provably
  effect-free")
- given: `view double (n:Int):Int = n + n` — no effectful call, no row.
- expect: inferred row = **`[]`** (empty); accepts; usable where a pure function
  is required (the verification layer may treat it as a mathematical function).
- why: the pure-default base case and the **EFF5 hinge**. A bug that infers a
  spurious effect for pure code (or breaks "no row ⇒ pure") is caught by the
  asserted empty row. Pairs with `pure-view-usable-in-pure-context`.

---

## EFF2 — pure interaction-tree denotation: the pure-kernel bridge (frame AC2)

The effectful program **denotes to a pure interaction tree** (`Ret a` |
`perform e then continue with the response`); `Eff` bind is **tree grafting**;
the kernel checks the tree as an **ordinary inductive** with **zero effect
machinery** (`36 §2`).

### surface/effects/eff-denotes-to-interaction-tree (oracle)
- spec: `36 §2` (layer 2)
- given: `view two_ops () visits [Console] = { greet "a" ; greet "b" }`, denoted
  through the `Eff` encoding.
- expect: the denotation is the **pure** term
  `perform (greet "a") (\_. perform (greet "b") (\_. Ret unit))` — **two**
  `perform` nodes (one per op, **in source order**), each carrying the op and a
  continuation binding its response, terminating in `Ret unit`. Constructor
  names `perform`/`Ret` `(oracle)`; the **shape** is normative.
- why: AC2 as a **structural** assertion — not "compiles" but the exact tree:
  N ops ⇒ N nested `perform` nodes in order, `Ret` at the leaf, continuations
  binding responses. A bug that drops an op, reorders, or flattens the
  continuation is caught structurally. (interaction-tree-structure guardrail.)

### surface/effects/eff-bind-is-tree-grafting (oracle)
- spec: `36 §2` ("Eff's bind is tree grafting")
- given: `m >>= k` with `m = perform e (\r. Ret r)` and
  `k = \x. perform e2 (\r2. Ret r2)`.
- expect: bind **grafts `k` onto every `Ret` leaf of `m`**, threading the
  response: `perform e (\r. perform e2 (\r2. Ret r2))`. The `Ret r` leaf of `m`
  is replaced by `k r`, with `r` (the response of `e`) in scope where `k` runs.
- why: pins **bind = grafting** structurally. A bug that sequences by another
  rule (concatenating performs without threading the response, or grafting at
  the wrong leaf) is caught; a response capture/threading bug shows here.

### surface/effects/eff-kernel-checks-denotation-pure (property)
- spec: `36 §2` (one pure kernel), `10-kernel` TCB (ADR 0001/0004/0005), frame
  AC2
- given: the denotation term from `eff-denotes-to-interaction-tree`, handed to
  the kernel as a value of the interaction-tree inductive type.
- expect: kernel-check **Ok** — the kernel sees **only** an inductive datatype
  (`Ret`/`perform` over a pure effect-descriptor type). **No effect primitive,
  no row, no capability appears in the core term** the kernel checks; the effect
  row lives entirely in the surface/elaboration layer.
- why: **the `OQ-8` "one pure kernel" invariant, end-to-end** (frame AC2). A bug
  that leaks an effect primitive into the core term — making the kernel reason
  about effects — violates the small-TCB invariant. **This is the one genuine
  soundness regression in the file** (effect machinery in the TCB), hence a
  `(property)` over *every* effectful program's denotation that must never
  regress.

---

## EFF3 — capabilities gate effectful ops (frame AC3)

A **capability** is a value token (`using c : Cap`, `OQ-8a`) a computation must
be **given** to act; an op requiring it is **rejected without** the token,
**accepted with** it (`36 §3`). ≥2 distinct caps; denial path on each.

### surface/effects/cap-op-without-token-rejected (oracle)
- spec: `36 §3` (`requires`-as-capability; value tokens `OQ-8a`)
- given: `write_file` declared `using fs : FsCap`; a
  `view dump () : Unit visits [FS] = write_file "/x" data` with **no** `fs` in
  scope (no parameter, no enclosing handler provides it).
- expect: **static error** — `MissingCapability(FsCap)` (kind `(oracle)`): the
  op is gated on the token's presence in scope, unprovided.
- why: the **capability-denial path**.

### surface/effects/cap-op-with-token-accepted (oracle)
- spec: `36 §3`
- given: the same op, but `dump` takes `using fs : FsCap` (or an enclosing
  handler provides it).
- expect: **accepts** — `FsCap` is in scope; gating satisfied.
- why: the **flip** for `cap-op-without-token-rejected`. With the token accepts,
  without rejects → the verdict flips on the exact bug; a checker that ignored
  the capability requirement would accept *both* (no flip), so the pair pins it.
  (capability-gating + verdict-flip.)

### surface/effects/cap-two-distinct-caps-each-gated (oracle)
- spec: `36 §3`
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

### surface/effects/cap-attenuation-by-subsumption (oracle)
- spec: `36 §3` (attenuation — derive a **strictly weaker** token); enforcement
  is `Sec2`/`62`
- given: an op requiring `ReadCap`; two variants: a full `FsCap` **attenuated
  down** to `ReadCap`, vs. a `WriteCap` that does **not** subsume `ReadCap`.
- expect: the attenuated `ReadCap` (any token ⊒ `ReadCap`) **accepts**; the
  `WriteCap` **rejects** `MissingCapability(ReadCap)`. The lattice ordering is
  `(oracle)` (`62` pins it); the **gating direction** — a token gates an op iff
  it **subsumes** the required authority — is the normative L5-exposed contract.
- why: the **least-authority** contract `Sec2` rides. Verdict **flips** on a bug
  that checks token **equality** instead of **subsumption** (would wrongly
  reject the attenuated-but-sufficient token). Lattice specifics deferred to
  `Sec2` `(oracle)`; only the L5 gating-by-subsumption property is pinned here.

---

## EFF4 — `space` state + tail-resumptive handlers (frame AC4)

`space` is the **only** place identity-bearing mutable state lives; `becomes`
denotes to a **state-passing fold** of the interaction tree (`36 §4`). Handlers
are **folds** over the tree, **tail-resumptive only** (`36 §5`, `OQ-9`).

### surface/effects/space-becomes-threads-state (oracle)
- spec: `36 §4` (`space`, `becomes`, state-passing fold)
- given:

  ```
  space Counter {
    mut n : Int = 0
    view inc () : Unit visits [Counter] = n becomes n + 1
    view get () : Int  visits [Counter] = n
  }
  ```

  program `{ inc() ; inc() ; get() }`.
- expect: result **`2`**; the denotation is a **state-passing fold** threading
  `n: 0 → 1 → 2`. Assert both the final read `= 2` **and** that `becomes`
  denotes to state-passing (post-state after `inc();inc()` is `n = 2`).
- why: `space` semantics — `becomes` is cell update, reads/writes ordered by the
  effect discipline, denoting to a state-passing fold. A bug that fails to
  thread state (each `inc` reads the initial `n = 0`) yields `1`, not `2` — the
  asserted value flips. (structural/value assertion.)

### surface/effects/space-old-scoped-to-ensures (oracle)
- spec: `36 §4` (`old(e)` admitted, **scoped to the operation's `ensures`**)
- given: `view inc() visits [Counter] ensures n == old(n) + 1 = n becomes n+1`;
  and a variant asserting `n == old(n) + 2`.
- expect: the **`+1`** `ensures` **discharges** (`old(n)` is the pre-call cell
  value, well-defined because the denotation is state-passing); the **`+2`**
  variant is **disproved** (countermodel: post = pre + 1 ≠ pre + 2).
- why: pins `old` as **scoped to the op's `ensures`** (not a global `\old`),
  grounded in the state-passing denotation. Verdict **flips**: the correct
  postcondition proves, the wrong one disproves. (bounded-Hoare guardrail,
  `§4`.)

### surface/effects/space-shared-nothing-no-cross-space-alias (oracle)
- spec: `36 §4` (shared-nothing message-passing; isolation **guarantee**)
- given: two spaces `A`, `B`; (a) `A` **directly** reads/writes `B`'s `mut` cell
  (aliases `B`'s `n`); (b) `A` **sends** an immutable value to `B` by
  message-passing.
- expect: (a) **static error** `CrossSpaceAlias` (kind `(oracle)`); (b)
  **accepts**.
- why: the **shared-nothing isolation** guarantee (no shared mutable state ⇒ no
  data races) on which capability confinement (`Sec2`, `62 §4`) rests. Verdict
  **flips**: legal message-passing accepts, illegal aliasing rejects. A bug
  permitting cross-space aliasing breaks isolation **silently** (the program
  still kernel-checks) — caught only here. (isolation guardrail.)

### surface/effects/handler-tail-resumptive-folds (oracle)
- spec: `36 §5` (handler = fold; tail-resumptive: continuation invoked **at most
  once, in tail position**)
- given: a handler for `[Console]` interpreting `greet s` by collecting `s` and
  resuming with `unit` in tail position; handle `{ greet "a" ; greet "b" }`.
- expect: the handler **folds** the interaction tree, resuming **once per**
  `perform` node **in tail position**; result = the folded accumulation (e.g.
  `["a","b"]`) with the continuation run to `Ret`. Assert the fold visits
  **both** `perform` nodes **in order**, resuming once each.
- why: tail-resumptive handler = **fold over the tree** (`§2` layer 2 ↔ `§5`).
  Structural: each `perform` node consumed exactly once, in order, resumed in
  tail position. (handler-resume guardrail.)

### surface/effects/handler-multishot-rejected (oracle)
- spec: `36 §5` (`OQ-9` DECIDED: **tail-resumptive only**; multi-shot
  **excluded**)
- given: two handlers for the same effect: (a) **tail-resumptive** (resumes
  once, tail position); (b) a handler invoking the captured continuation
  **twice** (or in **non-tail** position) — a multi-shot / `shift`-style
  handler.
- expect: (a) **accepts**; (b) **static error** `NonTailResumptive` (kind
  `(oracle)`) — the continuation is used more than once / not in tail position.
- why: **the `OQ-9` exclusion guard** — multi-shot is a *positive design
  exclusion* (keeps the fold well-founded, preserves totality `17 §4`, keeps
  single-consumption WP reasoning sound). Verdict **flips**: tail-resumptive
  accepts, multi-shot rejects. A bug admitting multi-shot continuations breaks
  totality/WP-soundness for effectful code, which the **kernel will not catch**
  (the tree is still pure data) — a load-bearing discipline guard. (`§5`
  exclusion + verdict-flip.)

---

## EFF5 — pure/impure boundary hook for L7 FFI (frame AC5)

L5 exposes the **`pure`/`impure` boundary** as a hook L7 (FFI) plugs into; it
does **not** implement FFI (`36 §6`, `38-ffi-io §3`).

### surface/effects/pure-view-usable-in-pure-context (oracle)
- spec: `36 §1`, §6 ("a pure-typed view is provably effect-free")
- given: `double` (row `[]`, from `eff-pure-default-is-effect-free`) used where
  a pure function is required — inside a `requires`/`ensures` predicate or a
  total pure combinator.
- expect: **accepts** — the empty row certifies effect-freedom, so the
  verification layer treats `double` as a mathematical function (its `ensures`
  are about values, not world-state).
- why: AC5's **pure half** — the `[]` row is the certificate L7/verification
  rely on. Pairs with `eff-pure-default-is-effect-free`.

### surface/effects/impure-boundary-marker-exposed (property, oracle)
- spec: `36 §6` (`pure`/`impure` boundary exposed for L7), `38-ffi-io §3`
- given: a `foreign`/`impure`-marked op stub (the FFI boundary L7 will fill)
  whose type carries the impure marker (a row entry `visits [Foreign]` or a
  `pure=false` flag — exact spelling `(oracle)`); and a `view` calling it.
- expect: the boundary marker is **visible in the op's type**, and a caller of
  the impure op **inherits** the impure marker in its inferred row (it
  propagates transitively like any effect). L5 exposes the **hook** without
  implementing FFI.
- why: AC5 — L5 provides only the wiring point; L7 plugs FFI in. Pins that the
  impure marker is *exposed* and *propagates*, so L7 has a hook. (`property`:
  the marker propagates through inference for every caller.)

### surface/effects/impure-masquerading-as-pure-rejected (oracle)
- spec: `36 §1`, §6
- given: a pure-typed `view safe () : Int = read_clock ()` where `read_clock` is
  impure (`visits [Clock]` / foreign), but `safe` declares **no row** (claims
  purity).
- expect: **static error** — `EffectEscapes(Clock)` / `ImpureInPureContext`
  (kind `(oracle)`): an impure op cannot be called from a pure-typed (empty-row)
  view without surfacing the effect.
- why: the boundary's **integrity** — impure cannot silently masquerade as pure,
  the property the "no row ⇒ pure" certificate (and all of verification, IFC, CT
  downstream) depends on. Verdict **flips**: declaring `visits [Clock]` accepts,
  claiming purity rejects. This is EFF1's escape-rejection **re-applied at the
  pure/impure boundary** — the AC5 integrity guard. A bug here lets impure code
  be trusted as pure fleet-wide. (verdict-flip.)

---

## Regression — L5 is additive over surface/V0

### surface/effects/existing-surface-invariants-still-green (property)
- spec: `../seed-surface.md` (`well-typed-output`, `ambiguity-is-an-error`),
  `../elaboration/seed-elaboration.md` (the V0 pipeline)
- given: the on-`main` surface/elaboration invariants and the V0
  `lex → parse → resolve → elaborate → kernel-check` seeds.
- expect: **unchanged** — L5 **extends** surface conformance with effects; it
  must not regress pure-elaboration or the V0 pipeline. A pure `view` (no row)
  elaborates to the **identical** core term with or without the effect layer
  present (effects are additive: no row ⇒ the V0 path is untouched).
- why: L5 is **additive** over V0/surface. Pins that introducing the effect
  discipline does not perturb pure-program elaboration. (`property`: regression
  guard over the existing corpus.)
